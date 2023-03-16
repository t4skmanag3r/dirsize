use crate::structs::{Dir, SizeFormat};
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEventKind},
    queue, style, terminal, Result,
};
use opener::open;
use std::io::Write;

impl Dir {
    fn display_menu(&self, size_fmt: &SizeFormat, max_len: Option<usize>) -> String {
        let (formated_size, format_str) = self.size_formated(size_fmt);
        let max_len = max_len.unwrap_or(25);
        format!(
            "{:<max_len$} - {:.2} {}",
            self.name(),
            formated_size,
            format_str,
            max_len = max_len
        )
    }
    fn color(&self) -> style::Color {
        if self.is_file {
            style::Color::Red
        } else {
            style::Color::White
        }
    }
}

const SIZE_FILTER_MIN: u64 = 1000000;

pub struct Menu<'a> {
    root_dir: &'a Dir,
    selected_dir: &'a Dir,
    filtered: Vec<&'a Dir>,
    cursor_pos: usize,
    size_fmt: SizeFormat,
    last_selected: Vec<usize>,
}

impl<'a> Menu<'a> {
    pub fn new(dir: &'a mut Dir, size_fmt: SizeFormat) -> Self {
        let cursor_pos = 0;
        let filtered = dir
            .filter_size(SIZE_FILTER_MIN)
            .expect("Expected dir to have files/dirs above a minimum size of 1mb"); // filters dirs for size above a threshold
        let last_selected = vec![]; // used to track the directory tree traversal
        Self {
            root_dir: dir,
            selected_dir: dir,
            filtered,
            cursor_pos,
            size_fmt,
            last_selected,
        }
    }

    fn draw_directory_path(&self, stdout: &mut impl Write) -> Result<()> {
        queue!(stdout, cursor::MoveTo(0, 0))?;
        queue!(stdout, style::SetForegroundColor(style::Color::Grey))?;
        queue!(stdout, style::Print(self.selected_dir.path.display()))?;
        Ok(())
    }

    fn draw_navigation_info(&self, stdout: &mut impl Write) -> Result<()> {
        let (_, terminal_height) = terminal::size().unwrap();
        queue!(stdout, cursor::MoveToRow(terminal_height))?;
        queue!(stdout, cursor::MoveToColumn(0))?;
        queue!(
            stdout,
            style::Print("move with (↑ & ↓), navigate dirs (→ or [Enter] & ← or [Backspace]), [Esc] to exit program, [o] open dir")
        )?;
        Ok(())
    }

    fn draw_warning(
        &self,
        stdout: &mut impl Write,
        message: &str,
        color: style::Color,
    ) -> Result<()> {
        let (_, terminal_height) = terminal::size().unwrap();
        queue!(stdout, cursor::MoveToRow(terminal_height))?;
        queue!(stdout, cursor::MoveToColumn(0))?;
        queue!(stdout, terminal::Clear(terminal::ClearType::CurrentLine))?;
        queue!(stdout, style::SetForegroundColor(color))?;
        queue!(stdout, style::Print(message))?;
        stdout.flush()?;
        Ok(())
    }

    /// Draws the menu to the scren
    fn draw(&self, stdout: &mut impl Write) -> Result<()> {
        queue!(stdout, cursor::MoveTo(0, 0))?;
        queue!(stdout, terminal::Clear(terminal::ClearType::All))?;
        self.draw_directory_path(stdout)?;
        queue!(stdout, cursor::MoveDown(1))?;
        queue!(stdout, cursor::MoveToColumn(0))?;
        for (i, item) in self.filtered.iter().enumerate() {
            let (start_index, end_index) = self.calculate_index_bounds();
            if (i >= start_index) & (i <= end_index) {
                // Printing the cursor
                if i == self.cursor_pos {
                    queue!(stdout, style::SetForegroundColor(style::Color::White))?;
                    queue!(stdout, style::Print("> "))?;
                } else {
                    queue!(stdout, style::Print("  "))?;
                }
                // Printing the items (dirrectories)
                queue!(stdout, style::SetForegroundColor(item.color()))?;
                let max_str_len = self.calculate_max_len();
                queue!(
                    stdout,
                    style::Print(item.display_menu(&self.size_fmt, Some(max_str_len)))
                )?;
                queue!(stdout, cursor::MoveDown(1))?;
                queue!(stdout, cursor::MoveToColumn(0))?;
            }
        }
        queue!(stdout, style::SetForegroundColor(style::Color::White))?;
        self.draw_navigation_info(stdout)?;
        stdout.flush()?;
        Ok(())
    }

    /// Calculates the maximum directory name length of Vec<&Dir>
    fn calculate_max_len(&self) -> usize {
        self.filtered
            .iter()
            .map(|dir| dir.name().len())
            .fold(0, |acc, l| if l > acc { l } else { acc })
    }

    /// Start the menu
    pub fn run(&mut self) -> Result<()> {
        // menu setup
        let mut stdout = std::io::stdout();
        terminal::enable_raw_mode().unwrap();
        queue!(stdout, terminal::EnterAlternateScreen)?;
        queue!(stdout, terminal::Clear(terminal::ClearType::All))?;
        queue!(stdout, cursor::Hide)?;
        stdout.flush()?;

        // menu input handling loop
        loop {
            self.draw(&mut stdout)?;

            if let Ok(Event::Key(key_event)) = crossterm::event::read() {
                if let KeyEventKind::Press = key_event.kind {
                    match key_event.code {
                        KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Up => {
                            if self.cursor_pos > 0 {
                                self.cursor_pos -= 1;
                            } else {
                                self.cursor_pos = self.filtered.len() - 1
                            }
                        }
                        KeyCode::Down => {
                            if self.cursor_pos < self.filtered.len() - 1 {
                                self.cursor_pos += 1;
                            } else {
                                self.cursor_pos = 0;
                            }
                        }
                        KeyCode::Enter | KeyCode::Right => {
                            self.select_item();
                        }
                        KeyCode::Backspace | KeyCode::Left => {
                            self.go_back();
                        }
                        KeyCode::Char('o') => {
                            if open(self.selected_dir.path.as_os_str()).is_err() {
                                self.draw_warning(
                                    &mut stdout,
                                    "Failed to open the directory",
                                    style::Color::Red,
                                )?;
                                block_until_key_press();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        // menu teardown
        queue!(stdout, terminal::LeaveAlternateScreen)?;
        stdout.flush()?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    /// Calculates the index range to print menu elements
    fn calculate_index_bounds(&self) -> (usize, usize) {
        let items_len = self.filtered.len() - 1;
        let (terminal_width, terminal_height) = terminal::size().unwrap();
        let (_, terminal_height) = (terminal_width as usize - 1, terminal_height as usize - 3);
        if self.cursor_pos <= (terminal_height / 2) {
            (0, terminal_height)
        } else if items_len > self.cursor_pos + (terminal_height / 2) {
            (
                self.cursor_pos - (terminal_height / 2) - 1,
                self.cursor_pos + (terminal_height / 2),
            )
        } else if items_len > terminal_height {
            (
                self.cursor_pos - (terminal_height - (items_len - self.cursor_pos)),
                items_len,
            )
        } else {
            (0, items_len)
        }
    }

    /// Select a menu item
    fn select_item(&mut self) {
        let select = self
            .filtered
            .get(self.cursor_pos)
            .expect("Out of range index for selected item");
        if select.contents.is_none() {
            return;
        }
        if let Some(filter) = select.filter_size(SIZE_FILTER_MIN) {
            self.selected_dir = select;
            self.filtered = filter;

            self.last_selected.push(self.cursor_pos);
            self.cursor_pos = 0;
        }
    }

    /// Go back to the previuous menu item
    fn go_back(&mut self) {
        self.selected_dir = self.root_dir.find(&self.selected_dir.path);
        self.filtered = self.selected_dir.filter_size(SIZE_FILTER_MIN).unwrap();
        self.cursor_pos = self.last_selected.pop().unwrap_or(0);
    }
}

fn block_until_key_press() {
    loop {
        if let Ok(Event::Key(key)) = crossterm::event::read() {
            if let KeyEventKind::Press = key.kind {
                break;
            }
        };
    }
}

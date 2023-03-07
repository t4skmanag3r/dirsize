extern crate dirsize;
use crossterm::{cursor, execute, queue, style, terminal, Result};
use dirsize::scanning::make_dir_tree_multithreaded;
use dirsize::structs::Dir;
use dirsize::structs::SizeFormat;
use std::io::{stdout, Write};
use std::path::Path;

const SIZE_MIN: u64 = 1000000;

fn print_menu<W: Write>(
    stdout: &mut W,
    items: &Dir,
    selected_index: usize,
    scroll_offset: usize,
) -> Result<()> {
    queue!(stdout, cursor::MoveToRow(0))?;
    let filtered = items.filter_size(SIZE_MIN);
    let mut y = 0;
    for (i, item) in filtered.as_ref().unwrap().iter().enumerate() {
        queue!(stdout, cursor::MoveToColumn(0))?;
        if i < scroll_offset {
            continue;
        }
        if y >= terminal::size()?.1 - 1 {
            break;
        }
        if i == selected_index {
            queue!(stdout, style::Print("> "))?;
        } else {
            queue!(stdout, style::Print("  "))?;
        }
        y += 1;
        let (formated_size, format_str) = item.size_formated(SizeFormat::MEGABYTES);
        queue!(
            stdout,
            style::Print(format!(
                "{} - {:.2} {}\n",
                item.path.file_name().unwrap().to_str().unwrap(),
                formated_size,
                format_str
            ))
        )?;
    }
    stdout.flush()?;
    Ok(())
}

fn main() -> Result<()> {
    // Logging
    std::env::set_var("RUST_LOG", "error");
    env_logger::init();

    // Creating path to directory
    let path = std::env::args().nth(1).expect("missing path to directory");
    let root = Path::new(&path);

    // Scaning the directory structure
    println!("Running size calculation for directory: {}", root.display());
    let mut result = make_dir_tree_multithreaded(root.to_path_buf());

    // Sorting the directory from bigest to smallest
    result.sort_by_size();

    // println!("{}", result.display_default());
    // for f in result.contents.unwrap().iter() {
    //     println!("{}", f.display_default())
    // }
    // io::stdin().read_line(&mut String::new()).unwrap();

    // Define the items in the menu.
    let mut selected = &result;
    let mut items = selected;

    // Set up the cursor position and the selected item index.
    let mut cursor_pos = 0;
    let mut selected_index = 0;
    let mut scroll_offset = 0;
    let mut last_selected = 0;
    let mut went_back = false;

    // Initialize the terminal and clear the screen.
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout, cursor::Hide)?;

    print_menu(&mut stdout, selected, selected_index, scroll_offset)?;

    // Handle input from the user.
    loop {
        if let Ok(event) = crossterm::event::read() {
            let filtered = items.filter_size(SIZE_MIN);
            match event {
                crossterm::event::Event::Key(key_event) => match key_event.kind {
                    // If key pressed down
                    crossterm::event::KeyEventKind::Press => match key_event.code {
                        // Move the cursor up or down with the arrow keys.
                        crossterm::event::KeyCode::Up => {
                            if cursor_pos > 0 {
                                cursor_pos -= 1;
                                selected_index -= 1;
                                if cursor_pos < scroll_offset {
                                    scroll_offset -= 1;
                                }
                                // Move the cursor up one row.
                                queue!(stdout, cursor::MoveUp(1))?;
                            } else {
                                let filter_len = filtered.unwrap().len() - 1;
                                cursor_pos = filter_len;
                                selected_index = cursor_pos;
                                if filter_len > terminal::size()?.1 as usize {
                                    scroll_offset = filter_len - terminal::size()?.1 as usize + 2;
                                }
                                queue!(stdout, cursor::MoveToRow(cursor_pos as u16))?;
                            }
                        }
                        crossterm::event::KeyCode::Down => {
                            if cursor_pos < filtered.unwrap().len() - 1 {
                                cursor_pos += 1;
                                selected_index += 1;
                                if cursor_pos >= scroll_offset + terminal::size()?.1 as usize {
                                    scroll_offset += 1;
                                }
                                // Move the cursor down one row.
                                queue!(stdout, cursor::MoveDown(1))?;
                            } else {
                                cursor_pos = 0;
                                selected_index = 0;
                                if cursor_pos < scroll_offset {
                                    scroll_offset = 0;
                                }
                                queue!(stdout, cursor::MoveToRow(0))?;
                            }
                        }
                        // Quit the program with the escape key.
                        crossterm::event::KeyCode::Esc => {
                            break;
                        }
                        crossterm::event::KeyCode::Enter | crossterm::event::KeyCode::Right => {
                            match &filtered {
                                Some(c) => {
                                    // if selected_index >= c.len() as usize {
                                    //     went_back = false;
                                    //     cursor_pos = 0;
                                    //     selected_index = 0;
                                    //     continue;
                                    // }
                                    let s = &c[selected_index];
                                    if s.filter_size(SIZE_MIN).is_none() {
                                        continue;
                                    }
                                    match s.contents {
                                        Some(_) => {
                                            selected = s;
                                            last_selected = selected_index;
                                            cursor_pos = 0;
                                            selected_index = 0;
                                            scroll_offset = 0;
                                            went_back = false;
                                            queue!(stdout, cursor::MoveToRow(0))?;
                                        }
                                        None => {}
                                    }
                                }
                                None => continue,
                            };
                        }
                        crossterm::event::KeyCode::Backspace | crossterm::event::KeyCode::Left => {
                            let find = result.find(&items.path);
                            selected = find;
                            if !went_back {
                                went_back = true;
                                cursor_pos = last_selected;
                                selected_index = last_selected;
                            } else {
                                went_back = false;
                                cursor_pos = 0;
                                selected_index = 0;
                                last_selected = 0;
                            }
                            scroll_offset = 0;
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
            // Clear the screen and re-print the menu.
            items = selected;
            execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
            print_menu(&mut stdout, selected, selected_index, scroll_offset)?;
        }
    }

    // Restore the original terminal state and exit the program.
    execute!(stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

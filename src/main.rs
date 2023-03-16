extern crate dirsize;
use crossterm::Result;
use dirsize::menu::Menu;
use dirsize::scanning::make_dir_tree_multithreaded;
use dirsize::structs::SizeFormat;
use std::path::Path;

// Seting default size formating
const SIZE_FMT_DEFAULT: SizeFormat = SizeFormat::MEGABYTES;

fn main() -> Result<()> {
    // Logging
    std::env::set_var("RUST_LOG", "error");
    env_logger::init();

    // Creating path to directory
    let path = std::env::args().nth(1).expect("missing path to directory");
    let root = Path::new(&path);

    // Scaning the directory structure
    println!("Running size calculation for directory: {}", root.display());
    let mut dir = make_dir_tree_multithreaded(root.to_path_buf());

    // Sorting the directory from bigest to smallest
    dir.sort_by_size();

    // Starting menu
    let mut menu = Menu::new(&mut dir, SIZE_FMT_DEFAULT);
    menu.run()
}

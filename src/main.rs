extern crate dirsize;
use clap::Parser;
use crossterm::Result;
use dirsize::menu::Menu;
use dirsize::scanning::make_dir_tree_multithreaded;
use dirsize::structs::SizeFormat;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to dirrectory
    #[arg()]
    path: PathBuf,
    /// size format, possible values : [gb, mb, kb, b]
    #[arg(short, long, default_value = "mb")]
    size: Option<SizeFormat>,
}

fn main() -> Result<()> {
    // Parsing arguments
    let args = Args::parse();
    let root_path = args.path;
    let size_format = args.size.unwrap();

    // Scaning the directory structure
    println!(
        "Running size calculation for directory: {}",
        root_path.display()
    );
    let mut dir = make_dir_tree_multithreaded(root_path);

    // Sorting the directory from bigest to smallest
    dir.sort_by_size();

    // Starting menu
    let mut menu = Menu::new(&mut dir, size_format);
    menu.run()
}

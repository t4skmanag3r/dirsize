extern crate dirsize;
use clap::{error::ErrorKind, CommandFactory, Parser};
use crossterm::Result;
use dirsize::menu::Menu;
use dirsize::scanning::make_dir_tree_multithreaded;
use dirsize::structs::SizeFormat;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to dirrectory
    path: Option<PathBuf>,
    /// size format, possible values : [gb, mb, b]
    #[arg(short, long, default_value = "mb")]
    size: Option<SizeFormat>,
}

fn main() -> Result<()> {
    // Parsing arguments
    let args = Args::parse();
    let dir_path: &Path = match args.path.as_deref() {
        Some(path) => path,
        None => {
            let mut cmd = Args::command();
            cmd.error(
                ErrorKind::MissingRequiredArgument,
                "Missing dirrectory path",
            )
            .exit()
        }
    };
    let size_format = args.size.unwrap();
    println!("{}", dir_path.display());

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
    let mut menu = Menu::new(&mut dir, size_format);
    menu.run()
}

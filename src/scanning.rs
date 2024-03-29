use crate::structs::Dir;
use log::{debug, warn};
use rayon::prelude::*;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Scans a directory recursively and finds all files contained within the directory and makes a directory tree (Dir)
///
/// Args:
/// - path_to_dir - The path to the directory
///
/// Examples:
///
/// ```
/// use std::path::Path;
///
/// let root = Path::new(r".");
/// let result = dirsize::scanning::make_dir_tree(root.to_path_buf());
///
/// println!("{}", result.display_default());
/// for f in result.contents.unwrap().iter() {
///     println!("{}", f.display_default())
/// }
/// ```
pub fn make_dir_tree(path_to_dir: PathBuf) -> Dir {
    let mut contents = Vec::new();
    let r_dir = match fs::read_dir(&path_to_dir) {
        Ok(dir) => dir,
        Err(err) => {
            warn!(
                "Error occured when trying to read {} error: {}",
                path_to_dir.display(),
                err
            );
            return Dir::new(0, path_to_dir, None, false);
        }
    };

    for entry in r_dir {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            debug!("{} is a directory", path.display());
            contents.push(make_dir_tree(path));
        } else {
            let size = match fs::metadata(&path) {
                Ok(file) => file.len(),
                Err(err) => match err.kind() {
                    ErrorKind::PermissionDenied => {
                        warn!(
                            "Permission denied when accesing file/directory: {}",
                            path.display()
                        );
                        continue;
                    }
                    _ => {
                        warn!(
                            "Error occured when trying to read {} error: {}",
                            path.display(),
                            err
                        );
                        continue;
                    }
                },
            };
            debug!("{} is a file with size: {} bytes", path.display(), size);
            let is_file = path.is_file();
            contents.push(Dir::new(size, path, None, is_file));
        }
    }
    let sizes: Vec<u64> = contents.iter().map(|x: &Dir| x.size).collect();
    let dir = Dir::new(sizes.iter().sum(), path_to_dir, Some(contents), false);
    dir
}

pub fn make_dir_tree_parallel(path_to_dir: PathBuf) -> Dir {
    let r_dir = match fs::read_dir(&path_to_dir) {
        Ok(dir) => dir,
        Err(err) => {
            warn!(
                "Error occured when trying to read {} error: {}",
                path_to_dir.display(),
                err
            );
            return Dir::new(0, path_to_dir, None, false);
        }
    };
    let contents = r_dir
        .into_iter()
        .par_bridge()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_dir() {
                debug!("{} is a directory", path.display());
                Some(make_dir_tree(path))
            } else {
                let size = match fs::metadata(&path) {
                    Ok(file) => file.len(),
                    Err(err) => match err.kind() {
                        ErrorKind::PermissionDenied => {
                            warn!(
                                "Permission denied when accesing file/directory: {}",
                                path.display()
                            );
                            return None;
                        }
                        _ => {
                            warn!(
                                "Error occured when trying to read {} error: {}",
                                path.display(),
                                err
                            );
                            return None;
                        }
                    },
                };
                debug!("{} is a file with size: {} bytes", path.display(), size);
                let is_file = path.is_file();
                Some(Dir::new(size, path, None, is_file))
            }
        })
        .collect::<Vec<Dir>>();

    let sizes: Vec<u64> = contents.iter().map(|x: &Dir| x.size).collect();
    let dir = Dir::new(sizes.iter().sum(), path_to_dir, Some(contents), false);
    dir
}

fn _benchmark_make_dir_tree(func: fn(PathBuf) -> Dir, n: i32) -> f32 {
    let mut times: Vec<Duration> = vec![];
    for _i in 0..n {
        let start = Instant::now();
        let root = Path::new(r".");
        let _result = func(root.to_path_buf());
        let end = Instant::now();
        let time = end - start;
        times.push(time)
    }
    let total_time = times
        .iter()
        .fold(Duration::default(), |acc, &x| acc + x)
        .as_secs_f32();
    let average_time: f32 = total_time / times.len() as f32;

    println!(
        "function ran {} times and took an average of {} seconds, total: {} seconds",
        n, average_time, total_time
    );
    average_time
}

#[cfg(test)]
mod bench {
    use crate::scanning::{_benchmark_make_dir_tree, make_dir_tree, make_dir_tree_parallel};

    #[test]
    #[ignore]
    fn benchmark_make_dir_tree() {
        _benchmark_make_dir_tree(make_dir_tree, 10);
    }

    #[test]
    #[ignore]
    fn benchmark_make_dir_tree_parallel() {
        _benchmark_make_dir_tree(make_dir_tree_parallel, 10);
    }
}

#[cfg(test)]
mod test {
    use crate::scanning::{make_dir_tree, make_dir_tree_parallel};
    use std::path::Path;

    #[test]
    fn test_make_dir_tree() {
        let root = Path::new(r".");
        let tree = make_dir_tree(root.to_path_buf());
        for sub_dir in tree.contents.unwrap().iter() {
            println!("{}", sub_dir.display_default())
        }
    }

    #[test]
    fn test_make_dir_tree_parallel() {
        let root = Path::new(r".");
        let tree = make_dir_tree_parallel(root.to_path_buf());
        for sub_dir in tree.contents.unwrap().iter() {
            println!("{}", sub_dir.display_default())
        }
    }
}

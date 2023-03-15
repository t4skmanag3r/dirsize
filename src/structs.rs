use std::fmt;
use std::fs;
use std::io::Error;
use std::path::PathBuf;

/// Enum representation of different size formats
pub enum SizeFormat {
    BYTES,
    MEGABYTES,
    GIGABYTES,
}

/// Structure that represents the directory tree or file
///
/// contains:
/// - size - the size of the directory/file in bytes
/// - path - the path to the directory/file
/// - contents - the contents of the directory (if it's a directory)
pub struct Dir {
    pub size: u64,
    pub path: PathBuf,
    pub contents: Option<Vec<Dir>>,
    pub is_file: bool,
}
impl Dir {
    /// Create a new directory/file
    ///
    /// Args:
    /// - size - the size of the directory/file
    /// - path - the path to the directory/file
    /// - contents - the contents of the directory (if it's a directory)
    pub fn new(size: u64, path: PathBuf, contents: Option<Vec<Dir>>, is_file: bool) -> Self {
        Self {
            size,
            path,
            contents,
            is_file,
        }
    }

    pub fn from_entry(entry: fs::DirEntry) -> Result<Dir, Error> {
        let path = entry.path();
        let metadata = entry.metadata()?;
        let size = metadata.len();
        let is_file = path.is_file();

        Ok(Dir::new(size, path, None, is_file))
    }

    pub fn len(&self) -> usize {
        match &self.contents {
            Some(c) => c.len(),
            None => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self.contents {
            Some(c) => c.is_empty(),
            None => false,
        }
    }

    pub fn name(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }

    pub fn size_formated(&self, size_fmt: SizeFormat) -> (f32, &str) {
        let formated_size: f32 = match size_fmt {
            SizeFormat::BYTES => self.size as f32,
            SizeFormat::MEGABYTES => self.size as f32 / 1000000.0,
            SizeFormat::GIGABYTES => self.size as f32 / 1000000.0 / 1000.0,
        };
        let format_str: &str = match size_fmt {
            SizeFormat::BYTES => "bytes",
            SizeFormat::MEGABYTES => "mb",
            SizeFormat::GIGABYTES => "gb",
        };
        (formated_size, format_str)
    }
    /// String representation of the directory/file
    pub fn display(&self, size_fmt: SizeFormat) -> String {
        let (formated_size, format_str) = self.size_formated(size_fmt);
        format!(
            "path: \"{}\" size: {:.2} {}",
            self.path.display(),
            formated_size,
            format_str
        )
    }
    pub fn display_default(&self) -> String {
        self.display(SizeFormat::MEGABYTES)
    }

    /// Recursively finds the parent Dir of a given path in the Dir structure
    pub fn find(&self, path: &PathBuf) -> &Self {
        let parent_dir = match path.parent() {
            Some(path) => path,
            None => return self,
        };
        if self.path == parent_dir {
            return self;
        } else {
            let contents = match self.contents.as_ref() {
                Some(c) => c,
                None => panic!(
                    "dirrectory with name {} was not found",
                    parent_dir.display()
                ),
            };
            for sub_dir in contents.iter() {
                if path.starts_with(&sub_dir.path) {
                    return sub_dir.find(path);
                }
            }
        }
        self
    }

    /// Filters the contents of dir that is bigger than size_min and returns  new vector containing references to Dirs
    pub fn filter_size(&self, size_min: u64) -> Option<Vec<&Dir>> {
        match &self.contents {
            None => None,
            Some(contents) => {
                let filtered: Vec<&Dir> =
                    contents.iter().filter(|dir| dir.size > size_min).collect();
                if filtered.is_empty() {
                    None
                } else {
                    Some(filtered)
                }
            }
        }
    }

    /// Sorts the complete contents tree by size
    pub fn sort_by_size(&mut self) {
        if let Some(contents) = self.contents.as_mut() {
            contents.sort_by(|dir, dir2| dir2.size.cmp(&dir.size));
            for subdir in contents.iter_mut() {
                subdir.sort_by_size();
            }
        }
    }
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display_default())
    }
}

#[cfg(test)]
mod test {
    use crate::scanning::make_dir_tree_multithreaded;
    use std::path::Path;

    #[test]
    fn test_dir_find() {
        let root = Path::new(".");
        let tree = make_dir_tree_multithreaded(root.to_path_buf());
        let found = tree.find(&Path::new("./src/lib.rs").to_path_buf());

        assert_eq!(found.path, Path::new("./src/").to_path_buf());
    }

    #[test]
    fn test_filter_size() {
        let root = Path::new(".");
        let tree = make_dir_tree_multithreaded(root.to_path_buf());
        let size_min = 1000000;
        let filtered = tree.filter_size(size_min);

        for filt in filtered.unwrap().iter() {
            // println!("{}", filt.display_default())
            assert!(filt.size > size_min);
        }
    }
}

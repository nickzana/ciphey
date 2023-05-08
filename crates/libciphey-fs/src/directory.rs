use std::fmt::Display;
use std::io;
use std::path::{Path, PathBuf};

use super::file::File;

/// A wrapper type for a `PathBuf` that validates the path as a directory.
#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct Directory {
    path: PathBuf,
}

impl AsRef<Path> for Directory {
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

impl Directory {
    /// Creates a new `Directory` pointing to the provided `path`.
    ///
    /// # Errors
    /// Returns a [`io::ErrorKind::NotADirectory`] error if the path is a file.
    pub fn new<P>(path: P) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        // If something exists at the path, validate it as a directory.
        if path.exists() {
            let metadata = path.metadata()?;
            if !metadata.is_dir() {
                return Err(io::Error::from(io::ErrorKind::NotADirectory));
            }
        }

        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    /// Gets a subdirectory of this directory.
    ///
    /// # Errors
    /// Fails if the new subpath is not a valid directory by the same conditions
    /// as [`Directory::new`].
    pub fn subdirectory<P>(&self, extension: P) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        let mut path = self.path.clone();
        path.push(extension);
        Self::new(path)
    }

    /// Accesses a file that is a child of this directory at the provided path.
    ///
    /// # Errors
    /// Fails if the new subpath is not a valid file by the same conditions as
    /// [`File::new`].
    pub fn subfile<P>(&self, extension: P) -> Result<File, io::Error>
    where
        P: AsRef<Path>,
    {
        let mut path = self.path.clone();
        path.push(extension);
        File::new(path)
    }
}

impl Display for Directory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.path.display().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, create_dir_all, OpenOptions};
    use std::io;

    use super::Directory;
    use crate::tests::{random_string, temporary_path};

    #[test]
    fn test_new_directory() {
        let path = temporary_path();
        let dir = Directory::new(path);
        assert!(dir.is_ok());
    }

    #[test]
    fn test_dir_does_not_exist() {
        // Generate a new path that does not exist.
        let path = temporary_path();

        // Create the Directory, this should not create a directoy on the
        // filesystem.
        let dir = Directory::new(&path).unwrap();
        drop(dir);

        // Attempt to read the entries of the directory at path, will be an
        // error if the directory does not exist.
        let entries = path.read_dir().unwrap_err();

        // Should be a not found error.
        assert_eq!(entries.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn test_dir_path_is_a_file() {
        // Generate a new path that does not exist.
        let path = temporary_path();

        // Create a file at the path.
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        drop(file);

        // Create a Directory pointing to the path. Will be an error if the
        // entity at path is a file.
        let directory = Directory::new(&path).unwrap_err();

        // Should be a NotADirectory error.
        assert_eq!(directory.kind(), io::ErrorKind::NotADirectory);
    }

    #[test]
    fn test_dir_exists() {
        // Generate a new path that does not exist.
        let path = temporary_path();

        // Create a directory at the path.
        fs::create_dir_all(&path).unwrap();

        // Create a Directory pointing to the path.
        let dir = Directory::new(&path).unwrap();
        // The path of the Directory should exist and be a directory.
        assert!(dir.path.exists());
        assert!(dir.path.is_dir());
    }

    #[test]
    fn test_display_dir() {
        // Generate a new path that does not exist.
        let path = temporary_path();
        // Create a Directory pointing to the path.
        let dir = Directory::new(&path).unwrap();

        // The Display implementation of Directory should match that of Path.
        assert_eq!(format!("{}", &path.display()), format!("{}", &dir));
    }

    // TODO: Finish writing this test
    #[test]
    fn test_subdir_is_file() {
        // Generate a new path that does not exist.
        let path = temporary_path();
        // Create a file with the provided name.
        let filename = random_string(16);
    }

    // TODO: Finish writing this test
    #[test]
    fn test_subdir() {
        // Generate a new path that does not exist.
        let path = temporary_path();

        // Create the directory at path and 8 random subdiretories.
        let subdirs: Vec<String> = (0..8).map(|_| random_string(16)).collect();
        for dir in subdirs {
            create_dir_all(path.join(dir)).unwrap();
        }

        // Create a directory pointing to the path.
        let dir = Directory::new(&path).unwrap();
    }
}

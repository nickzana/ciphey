use std::fmt::Display;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::{fs, io};

use libciphey::storage::Reference;

/// A wrapper type for a `PathBuf` that validates the path as a file.
#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct File {
    path: PathBuf,
}

impl File {
    /// Creates a new `File` pointing to the provided path.
    ///
    /// # Errors
    /// Returns an [`io::ErrorKind::IsADirectory`] error if the provided path
    /// is a directory.
    pub fn new<P>(path: P) -> Result<Self, io::Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        // If something exists at the path, validate it as a file.
        if path.exists() {
            let metadata = path.metadata()?;
            if metadata.is_dir() {
                return Err(io::Error::from(io::ErrorKind::IsADirectory));
            }
        }

        Ok(File {
            path: path.to_path_buf(),
        })
    }
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.path.as_path().display(), f)
    }
}

impl Reference for File {
    type Reader = fs::File;
    type Writer = fs::File;

    fn reader(&self) -> Result<Self::Reader, io::Error> {
        OpenOptions::new().read(true).write(false).open(&self.path)
    }

    fn writer(&mut self) -> Result<Self::Writer, io::Error> {
        OpenOptions::new()
            .create_new(true) // Ensure that no entry is ever overwritten
            .write(true)
            .open(&self.path)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;
    use std::io::{Read, Write};
    use std::{fs, io};

    use libciphey::storage::Reference;

    use super::File;
    use crate::tests::{random_string, temporary_path};

    #[test]
    fn test_new_file() {
        let path = temporary_path();
        let file = File::new(path);
        assert!(file.is_ok());
    }

    #[test]
    fn test_file_does_not_exist() {
        // Generate a new path that does not exist.
        let path = temporary_path();

        // Create the File struct. This should not create a file on the
        // filesystem.
        let file = File::new(&path).unwrap();
        drop(file);

        // Create a File pointing to the path. Will be an error if the file does
        // not exist.
        let file = fs::File::open(&path).unwrap_err();

        // Should be a not found error.
        assert_eq!(file.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn test_file_path_is_dir() {
        // Generate a new path that does not exist.
        let path = temporary_path();

        // Create a directory at the path.
        fs::create_dir_all(&path).unwrap();

        // Create a File pointing to path. Will be error if the entity at path
        // is a directory.
        let file = File::new(&path).unwrap_err();

        // Should be an IsADirectory error.
        assert_eq!(file.kind(), io::ErrorKind::IsADirectory);
    }

    #[test]
    fn test_file_exists() {
        // Generate a new path that does not exist.
        let path = temporary_path();

        // Create a file at the path.
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        drop(file);

        // Create a File pointing to the path.
        let file = File::new(&path).unwrap();
        // The path of the File should exist and be a file.
        assert!(file.path.exists());
        assert!(file.path.is_file());
    }

    #[test]
    fn test_display_file() {
        // Generate a new path that does not exist.
        let path = temporary_path();
        // Create a File pointing to the path.
        let file = File::new(&path).unwrap();

        // The Display implementation of File should match that of Path.
        assert_eq!(format!("{}", &path.display()), format!("{}", &file));
    }

    #[test]
    fn test_reader() {
        // Generate a new path that does not exist.
        let path = temporary_path();

        // Create a file at the path.
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();

        // Write a random token to the file.
        let token = random_string(128);
        write!(&mut file, "{}", &token).unwrap();

        // Be certain that the write completed and the file handle is dropped.
        drop(file);

        // Create a File pointing to the path.
        let file = File::new(&path).unwrap();
        // Get a reader of the data at the path.
        let mut reader = file.reader().unwrap();

        // Read the data to a buffer.
        let mut buf = String::new();
        reader.read_to_string(&mut buf).unwrap();

        // The data in the buffer and the random token should match.
        assert_eq!(token, buf);
    }

    #[test]
    fn test_writer() {
        // Generate a new path that does not exist.
        let path = temporary_path();

        // Create a File pointing to the path.
        let mut file = File::new(&path).unwrap();
        // Get a writer that creates, then writes to the file.
        let mut writer = file.writer().unwrap();

        // Write a random token to the file.
        let token = random_string(128);
        write!(&mut writer, "{}", &token).unwrap();

        // Be certain that the write completed and the file handle is dropped.
        drop(file);

        // Read the data from the file to a buffer.
        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .open(&path)
            .unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();

        // The data in the buffer and the random token should match.
        assert_eq!(token, buf);
    }
}

#![feature(io_error_more)]

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

use directory::Directory;
use file::File;
use libciphey::storage::Backend;
use uuid::Uuid;

pub mod directory;
pub mod file;

#[cfg(test)]
mod tests;

// A filesystem-based store
pub struct Filesystem {
    root: Directory,
}

impl Filesystem {
    /// Creates a new `Filesystem` with a root directory at the provided `path`.
    ///
    /// # Errors
    /// Fails if the `root` path is not a valid directory by the same conditions
    /// as [`Directory::new`].
    pub fn new(root: &Path) -> Result<Self, io::Error> {
        let root = Directory::new(root)?;
        Ok(Self { root })
    }
}

impl Filesystem {
    fn entries_path(&self) -> Result<Directory, io::Error> {
        let path = self.root.clone();
        path.subdirectory("entries")
    }

    /// Reads the entries directory of the store
    fn entries_dir(&self) -> Result<fs::ReadDir, io::Error> {
        let path = self.entries_path()?;
        fs::read_dir(&path)
    }
}

impl Backend for Filesystem {
    type Reference = File;

    /// Returns a list of files that represent entries in the store.
    fn entries(&self) -> Result<HashMap<Uuid, Self::Reference>, io::Error> {
        let dir: fs::ReadDir = self.entries_dir()?;

        let mut map = HashMap::new();

        for entry in dir {
            let entry = entry?;
            // Extract the entry's path
            let path = entry.path();

            // Skip directories (this behavior may change in the future)
            if path.is_dir() {
                continue;
            }

            // Only try to parse files with the "age" extension (this behavior
            // may change in the future)
            if path.extension().map_or(true, |ext| ext != "age") {
                continue;
            }

            // Extract UUID from filename stem
            let raw_stem: &OsStr = match path.file_stem() {
                Some(s) => s,
                None => continue,
            };

            // Only convert if valid UTF-8
            let stem: &str = match raw_stem.to_str() {
                Some(s) => s,
                None => continue,
            };

            // Silently skip .age files with invalid filenames (this behavior
            // may change in the future)
            let uuid = match Uuid::from_str(stem) {
                Ok(u) => u,
                Err(_) => continue,
            };

            // Create an AsyncRead from the file
            let file = File::new(path)?;

            map.insert(uuid, file);
        }

        Ok(map)
    }

    /// Adds an entry to the store.
    ///
    /// Entries are addressed by their UUIDs. An entry being added to the store
    /// requires that no entry with that UUID currently exists in the store.
    /// This protects against accidentally overwriting data.
    ///
    /// In order to replace an entry in the store, the existing entry must first
    /// be removed. To keep track of modified entries, either use a version
    /// control system (such as git), a version indicator within the entry such
    /// as a timestamp, or an index.
    ///
    /// This function will error if the "entries" directory is not present.
    fn add_entry(&mut self, uuid: &Uuid) -> Result<Self::Reference, io::Error> {
        let formatted_uuid = uuid.hyphenated().to_string();
        let mut filename = PathBuf::new();
        filename.set_file_name(formatted_uuid);
        filename.set_extension("age");

        let path = self.entries_path()?;
        let file = path.subfile(filename)?;

        Ok(file)
    }

    fn create(&mut self) -> Result<(), io::Error> {
        let path = self.entries_path()?;

        if path.as_ref().exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{}", path),
            ));
        }

        std::fs::create_dir_all(path)
    }
}

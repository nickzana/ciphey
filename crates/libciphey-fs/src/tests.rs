//! These tests aim to ensure that filesystem-related tasks behave as expected.

use std::collections::HashMap;
use std::fs::read_dir;
use std::io::{copy, Read};
use std::path::PathBuf;
use std::{fs, io};

use libciphey::storage::{Backend, Reference};
use uuid::Uuid;

use crate::Filesystem;

// Returns a pseudorandom alphanumeric string of length `len`.
pub fn random_string(len: usize) -> String {
    std::iter::repeat_with(fastrand::alphanumeric)
        .take(len)
        .collect()
}

// Gets a path to a new directory with a random file name in the system's
// temporary directory.
//
// It does NOT create the directory on the filesystem, only generates the path.
pub fn temporary_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.extend(Some(random_string(32)));
    path
}

#[test]
fn test_create_new_filesystem() {
    // Creates a temporary directory to use as a test store.
    let path = temporary_path();
    // The new path should not exist yet.
    let handle = read_dir(&path).unwrap_err();
    // The error should be a NotFound error.
    assert_eq!(handle.kind(), io::ErrorKind::NotFound);

    // Create the backend.
    let mut backend = Filesystem::new(&path).unwrap();

    // The path should still not exist on the filesystem.
    let handle = read_dir(&path).unwrap_err();
    // The error should be a NotFound error.
    assert_eq!(handle.kind(), io::ErrorKind::NotFound);

    // Create the root directory in the filesystem.
    backend.create().unwrap();

    // The path should exist on the filesystem.
    drop(read_dir(&path).unwrap());
}

#[test]
// Tests that the filesystem returns the expected error when the provided root
// directory cannot be found.
fn test_entries_dir_does_not_exist() {
    // Create a temporary directory to use as a test store
    let path = temporary_path();
    std::fs::create_dir(&path).unwrap();

    // Create the filesystem backend
    let backend = Filesystem::new(&path).unwrap();

    // Verify that the root directory exists and can be accessed

    // Attempt to read backend's entries
    let entries = backend.entries_dir();

    // TODO: Test other properties of the error. More specifically, find a
    // way to ensure that the error is coming from the entires directory and
    // not the root directory.
    assert_eq!(
        entries.err().map(|e| e.kind()),
        Some(io::ErrorKind::NotFound)
    );
}

#[test]
// Tests that the filesystem returns the expected error when the program has
// insufficient permissions to access the entries directory.
//
// Unix is required for `std::os::unix::fs::PermissionsExt::set_mode`.
//
// TODO: Look into whether it is possible to write an equivalent test for
// non-Unix platforms.
fn test_entries_dir_insufficient_permissions() {
    use std::fs::Permissions;
    use std::os::unix::prelude::PermissionsExt;

    // Create a temporary directory to use as a test store
    let root_path = temporary_path();
    std::fs::create_dir(&root_path).unwrap();

    let mut entries_path: PathBuf = root_path.clone();
    entries_path.extend(Some("entries"));
    std::fs::create_dir(&entries_path).unwrap();

    // Remove entry directory's user/group/other file permissions
    std::fs::set_permissions(&entries_path, Permissions::from_mode(0o000))
        .unwrap();

    // Create the filesystem backend
    let backend = Filesystem::new(&root_path).unwrap();

    let entries: Result<fs::ReadDir, std::io::Error> = backend.entries_dir();

    assert_eq!(
        entries.err().map(|e| e.kind()),
        Some(io::ErrorKind::PermissionDenied)
    );
}

#[test]
// Tests that `Filesystem::entries_dir` method can successfully read a newly
// created entries directory.
fn test_entries_dir_ok() {
    // Create a temporary directory to use as a test store
    let root_path = temporary_path();
    std::fs::create_dir(&root_path).unwrap();

    // Create the entries directory
    let mut entries_path: PathBuf = root_path.clone();
    entries_path.extend(Some("entries"));
    std::fs::create_dir(&entries_path).unwrap();

    // Create the filesystem backend
    let backend = Filesystem::new(&root_path).unwrap();

    let entries = backend.entries_dir();

    assert!(entries.is_ok());
}

// TODO: Consider testing error conditions for `Filesystem::entries`.

#[test]
// Tests that `Filesystem::entries` method can successfully read several
// files.
fn test_entries_ok() {
    // Create a temporary directory to use as a test store
    let root_path = temporary_path();
    std::fs::create_dir(&root_path).unwrap();

    // Create the entries directory
    let mut entries_path: PathBuf = root_path.clone();
    entries_path.extend(Some("entries"));
    std::fs::create_dir(&entries_path).unwrap();

    // The data to write to the files
    let mut generated_data: HashMap<Uuid, Vec<u8>> = HashMap::new();

    // Create 10 random entries with random data in them
    for _ in 0..10 {
        // Generate a random UUID
        let uuid = Uuid::new_v4();

        // Create an entry file with the UUID as the filename and the ".age"
        // extension
        let mut file = entries_path.clone();
        file.extend(Some(uuid.to_string()));
        file.set_extension(".age");

        // Generate random data to put in the file
        let random_data = random_string(128).as_bytes().to_vec();

        // Save the data and write it to disk
        fs::write(file, &random_data).unwrap();
        generated_data.insert(uuid, random_data);
    }

    let backend = Filesystem::new(&root_path).unwrap();

    // Ensure that the entries can be read
    let entries = backend.entries().unwrap();

    for (uuid, file) in entries {
        let mut file = file.reader().unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        assert_eq!(generated_data.get(&uuid).unwrap(), &data);
    }
}

#[test]
// Tests that `Filesystem::add_entry` can successfully write several entries
fn test_add_entry_ok() {
    // Create a temporary directory to use as a test store
    let root_path = temporary_path();
    std::fs::create_dir(&root_path).unwrap();

    // Create the entries directory
    let mut entries_path: PathBuf = root_path.clone();
    entries_path.extend(Some("entries"));
    std::fs::create_dir(&entries_path).unwrap();

    // Generate random data
    let mut generated_data = HashMap::<Uuid, Vec<u8>>::new();

    for _ in 0..10 {
        let uuid = Uuid::new_v4();
        generated_data.insert(uuid, random_string(256).into());
    }

    let mut backend = Filesystem::new(&root_path).unwrap();

    // Insert generated data into the backend
    for (uuid, data) in generated_data {
        let mut reference = backend.add_entry(&uuid).unwrap();
        let mut writer = reference.writer().unwrap();
        copy(&mut data.as_slice(), &mut writer).unwrap();
    }

    let entries = backend.entries().unwrap();
    for (uuid, file) in entries {
        let mut file = file.reader().unwrap();
        let mut read_data = Vec::new();
        file.read_to_end(&mut read_data).unwrap();
        println!("{}: {}", uuid, read_data.len());
    }
}

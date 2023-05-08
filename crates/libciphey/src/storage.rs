use std::collections::HashMap;
use std::fmt::Display;
use std::io::{Error, Read, Write};

use uuid::Uuid;

/// Marks a type that holds the necessary information to create a reader or
/// writer over the data in a `StorageBackend`.
///
/// The reference must provide an implementation of `instance` so that
/// the underlying data can be read multiple times. This prevents the need to
/// read every entry into memory. If creating the reader is expensive, the
/// encrypted data can be read into a memory buffer when the data is suitably
/// small.
pub trait Reference: Display {
    type Reader: Read;
    type Writer: Write;

    /// Returns a new instance of a reader of the underlying data.
    fn reader(&self) -> Result<Self::Reader, Error>;

    /// Returns a new instance of a writer to persist the data.
    fn writer(&mut self) -> Result<Self::Writer, Error>;
}

pub trait Backend: Unpin {
    type Reference: Reference;

    /// Performs any initialization necessary to create a new store in the
    /// backend.
    fn create(&mut self) -> Result<(), Error>;

    /// Returns a map of all entries in the database. The key is the UUID of the
    /// entry and the value is a reference to the entry in the backend.
    fn entries(&self) -> Result<HashMap<Uuid, Self::Reference>, Error>;

    /// Adds an entry to the database with the provided UUID. The data to be
    /// persisted must be read from `source` in its entirety or return an error.
    ///
    /// Returns a reference to the newly created entry in the underlying store.
    fn add_entry(&mut self, uuid: &Uuid) -> Result<Self::Reference, Error>;
}

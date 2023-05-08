use std::io::{Error, Read, Write};

use crate::crypto::Encrypted;

pub trait Filetype: Sized {
    type Error: std::error::Error;
    type Options;

    /// Constructs a new instance of `Self` from data in the provided reader.
    ///
    /// # Errors
    /// Returns an error if the data read from the reader is invalid. Also
    /// propogates any errors from `std::io::Read`.
    fn deserialize<R>(reader: &mut R) -> Result<Self, Self::Error>
    where
        R: Read;

    /// Writes the data of the entry to the provided writer.
    ///
    /// # Errors
    /// Types impelementing `Filetype` should not be able to represent invalid
    /// state and thus every instance of `Self` should be serializable.
    ///
    /// Propogates any errors from `std::io::Write`.
    fn serialize<W, E>(self, writer: &mut E) -> Result<(), Error>
    where
        W: Write,
        E: Encrypted<W>;

    /// Write the contents of the entry in a readable format suitable for
    /// displaying to a user.
    ///
    /// If `show_secrets` is `false`, do not write any "sensitive" material.
    /// What is consisdered "sensitive" or "insensitive" is at the discretion of
    /// the filetype, however it is strongly recommended to allow users to
    /// toggle the sensitivity of each distinct piece of data.
    ///
    /// For example, a simple login credential should show the "password" field
    /// only when `show_secrets` is `true`.
    // TODO: Replace show_secrets with an enum?
    fn display<W>(
        &self,
        writer: &mut W,
        display_options: Self::Options,
        show_secrets: bool,
    ) -> Result<(), Error>
    where
        W: Write;
}

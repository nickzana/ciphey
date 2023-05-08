use std::error::Error;
use std::io::{Read, Write};

/// Marks that a type only writes encrypted data that is safe to persist to the
/// storage backend. Types that implement Encrypted MUST only write encrypted
/// data.
///
/// `W` is the type of the underlying writer.
pub trait Encrypted<W: Write>: Write {
    type Error: Error;
}

/// Marks that a type provides a decrypted stream of data.
///
/// `R` is the type of the underlying reader.
pub trait Decrypted<R: Read>: Read {
    type Error: Error;
}

/// A type that provides a public key for the [`crypto::Backend`] to encrypt to.
pub trait Recipient: TryFrom<String> + Clone {}

/// Types that implement `CryptoBackend` are expected to be initialized with any
/// identities necessary for decryption.
pub trait Backend {
    type Recipient: Recipient;
    type Decrypted<R: Read>: Decrypted<R>;
    type Encrypted<W: Write>: Encrypted<W>;
    type Error: Error + 'static;

    /// Creates a wrapper around a writer that will encrypt its input.
    /// Returns errors from the underlying writer while writing the header.
    fn encrypt_output<W: Write>(
        &self,
        output: W,
        recipients: Vec<Self::Recipient>,
    ) -> Result<Self::Encrypted<W>, Self::Error>;

    fn decrypt_input<R: Read>(
        &self,
        ciphertext: R,
    ) -> Result<Self::Decrypted<R>, Self::Error>;
}

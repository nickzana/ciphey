use std::io::{self, Read, Write};

use age::stream::{StreamReader, StreamWriter};
use age::{
    DecryptError, Decryptor, EncryptError, Encryptor, Identity, Recipient,
};
use libciphey::crypto::{self, Decrypted, Encrypted};

#[cfg(test)]
mod tests;

struct Age {}

impl Age {
    pub fn new(identities: &[Box<dyn Identity>]) -> Self {
        Age {}
    }
}

struct DecryptedReader<R>(StreamReader<R>);

impl<R: Read> DecryptedReader<R> {
    fn new<'a>(
        input: R,
        identities: &[&'a dyn Identity],
    ) -> Result<Self, DecryptError> {
        match Decryptor::new(input)? {
            Decryptor::Recipients(d) => {
                Ok(Self(d.decrypt::<'a>(identities.iter().copied())?))
            }
            Decryptor::Passphrase(_) => todo!(),
        }
    }
}

impl<R: Read> Decrypted<R> for DecryptedReader<R> {
    type Error = Error;
}

struct EncryptedWriter<W: Write>(StreamWriter<W>);

impl<W: Write> EncryptedWriter<W> {
    fn new(
        output: W,
        recipients: Vec<Box<dyn Recipient>>,
    ) -> Result<Self, EncryptError> {
        let encryptor =
            Encryptor::with_recipients(recipients).wrap_output(output)?;
        Ok(Self(encryptor))
    }
}

impl<W: Write> Write for EncryptedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl<W: Write> Encrypted<W> for EncryptedWriter<W> {
    type Error = Error;
}

impl<W: Write> EncryptedWriter<W> {
    pub fn finish(self) -> io::Result<W> {
        self.0.finish()
    }
}

impl crypto::Backend for Age {
    type Decrypted<R: Read> = DecryptedReader<R>;
    type Encrypted<W: Write> = EncryptedWriter<W>;
    type Error = Error;
    type Recipient = Box<dyn Recipient>;

    fn encrypt_entry<W: Write>(
        &self,
        output: W,
        recipients: Vec<Self::Recipient>,
    ) -> Result<EncryptedWriter<W>, Self::Error> {
        EncryptedWriter::new(output, recipients).map_err(Into::into)
    }

    fn decrypt_input<R: Read>(
        &self,
        ciphertext: R,
    ) -> Result<Self::Decrypted<R>, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Encrypt(age::EncryptError),
    Decrypt(age::DecryptError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<age::EncryptError> for Error {
    fn from(e: age::EncryptError) -> Self {
        Self::Encrypt(e)
    }
}

impl From<age::DecryptError> for Error {
    fn from(e: age::DecryptError) -> Self {
        Self::Decrypt(e)
    }
}

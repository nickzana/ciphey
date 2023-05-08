use std::io::{self, BufRead, BufReader, Error, Read, Write};

use libciphey::crypto;

#[cfg(test)]
mod tests;

pub struct Transparent {}

#[derive(Clone)]
pub struct Recipient(String);

impl crypto::Recipient for Recipient {}

impl From<String> for Recipient {
    fn from(s: String) -> Self {
        Self(s)
    }
}

pub struct Decrypted<R: Read>(BufReader<R>);
impl<R: Read> crypto::Decrypted<R> for Decrypted<R> {
    type Error = Error;
}

impl<R: Read> Read for Decrypted<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

pub struct Encrypted<W: Write>(W);

impl<W: Write> Write for Encrypted<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<W: Write> crypto::Encrypted<W> for Encrypted<W> {
    type Error = Error;
}

impl crypto::Backend for Transparent {
    type Decrypted<R: io::Read> = Decrypted<R>;
    type Encrypted<W: io::Write> = Encrypted<W>;
    type Error = Error;
    type Recipient = Recipient;

    fn encrypt_output<W: io::Write>(
        &self,
        mut output: W,
        recipients: Vec<Self::Recipient>,
    ) -> Result<Self::Encrypted<W>, Self::Error> {
        // Add recipients header
        for recipient in recipients {
            writeln!(output, "-> {}", recipient.0)?;
        }
        // Add separator
        writeln!(output, "---")?;
        Ok(Encrypted(output))
    }

    fn decrypt_input<R: io::Read>(
        &self,
        ciphertext: R,
    ) -> Result<Self::Decrypted<R>, Self::Error> {
        // Get all recipients
        let mut recipients = Vec::new();
        let mut reader = BufReader::new(ciphertext);

        while let mut line = String::new()
                && let _ = reader.read_line(&mut line)?
                && line.starts_with("-> ") {
            let recipient = line.trim_start_matches("-> ");
            recipients.push(recipient.to_string());
            line = String::new();
            reader.read_line(&mut line)?;
        }

        Ok(Decrypted(reader))
    }
}

use std::ffi::OsString;
use std::fmt::Display;
use std::io;
use std::process::Termination;

#[derive(Debug)]
pub enum Error {
    // TODO: Should we create a wrapper type to handle all supported backends?
    Storage(io::Error),
    Crypto(Box<dyn std::error::Error>),
    Filetype(Box<dyn std::error::Error>),
    Input(io::Error),
    Xflags(xflags::Error),
    OsStringConversionError(OsString),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Storage(io_err) | Error::Input(io_err) => io_err.fmt(f),
            Error::Xflags(xflags_err) => xflags_err.fmt(f),
            Error::Crypto(crypto_err) => crypto_err.fmt(f),
            Error::Filetype(deser_err) => deser_err.fmt(f),
            Error::OsStringConversionError(os_str) => {
                write!(f, "Could not parse invalid input: {:#?}", os_str)
            }
        }
    }
}

impl From<xflags::Error> for Error {
    fn from(err: xflags::Error) -> Self {
        Self::Xflags(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::Storage(err)
    }
}

impl Termination for Error {
    fn report(self) -> std::process::ExitCode {
        // TODO: Add more precise exit codes
        std::process::ExitCode::FAILURE
    }
}

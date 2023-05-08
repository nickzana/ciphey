use std::cmp::min;
use std::collections::HashSet;
use std::convert::Infallible;
use std::fmt::Display;
use std::io::{self, BufRead, BufReader};
use std::slice::Iter;
use std::str::FromStr;

use libciphey::crypto::Encrypted;
use libciphey::filetype::Filetype;

const DELIMETER: char = '=';
const SENSITIVITY: char = '!';

/// The key of a [`KeyValuePair`].
///
/// Some keys are handled as special cases by client applications. These keys
/// are enumerated and their meanings are documented.
///
/// All other unknown keys are handled by `Other`.
#[derive(PartialEq, Eq, Clone, Hash)]
pub enum Key {
    /// A name for the entry.
    Name,
    /// A username for an account, usually a website.
    Username,
    /// The email associated with the account.
    Email,
    /// The password or passphrase used to sign into the account.
    Password,
    /// A URL of the service. This can be a
    Url,
    Notes,
    Other(String),
}

impl FromStr for Key {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "name" => Self::Name,
            "username" => Self::Username,
            "email" => Self::Email,
            "password" => Self::Password,
            "url" => Self::Url,
            "notes" => Self::Notes,
            _ => Self::Other(s.to_string()),
        })
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        // Unwrap is safe because [`Self::FromStr::Err`] is [`Infallible`].
        Self::from_str(s).unwrap()
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let as_str = match self {
            Key::Name => "name",
            Key::Username => "username",
            Key::Email => "email",
            Key::Password => "password",
            Key::Url => "url",
            Key::Notes => "notes",
            Key::Other(v) => v,
        };

        write!(f, "{}", as_str)
    }
}

/// Contains the data associated with a [`Key`] in a [`KeyValuePair`].
///
/// A [`Sensitive`] value indicates to client applications that its data is
/// secret material. Client applications should avoid exposing this
/// data to potentially untrusted areas, such as on the display or in a search
/// query, without explicit user intention to do so.
///
/// A [`Sensitive`] value does NOT change the cryptographic credentials
/// necessary to access the value associated with the [`KeyValuePair`]. The
/// treatment of [`Sensitive`] and [`Insensitive`] values is entirely up to the
/// client applications.
pub enum Value {
    Sensitive(String),
    Insensitive(String),
}

pub struct KeyValuePair {
    pub key: Key,
    pub value: Value,
}

impl KeyValuePair {
    pub fn new(key: impl Into<Key>, value: impl Into<Value>) -> Self {
        let key = key.into();
        let value = value.into();

        Self { key, value }
    }
}

impl FromStr for KeyValuePair {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((key, value)) = s.split_once(DELIMETER) {
            let mut key: String = key.to_string();

            let value: Value = match key.ends_with(SENSITIVITY) {
                true => Value::Sensitive(value.to_string()),
                false => Value::Insensitive(value.to_string()),
            };

            if key.ends_with(SENSITIVITY) {
                assert_eq!(key.pop(), Some(SENSITIVITY));
            }

            let key: Key = Key::from(key.as_str());
            Ok(Self { key, value })
        } else {
            Err(Error::MissingDelimeter(s.to_string()))
        }
    }
}

impl Display for KeyValuePair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sensitivity = if let Value::Sensitive(_) = self.value {
            SENSITIVITY.to_string()
        } else {
            String::default()
        };

        let value = match &self.value {
            Value::Sensitive(value) => value,
            Value::Insensitive(value) => value,
        };

        write!(f, "{}{}{}{}", self.key, sensitivity, DELIMETER, value)
    }
}

/// An ordered key/value store for arbitrary keys and values.
pub struct KvStore {
    key_value_pairs: Vec<KeyValuePair>,
}

impl KvStore {
    pub fn new(key_value_pairs: Vec<KeyValuePair>) -> Self {
        Self { key_value_pairs }
    }

    pub fn iter(&self) -> Iter<'_, KeyValuePair> {
        self.key_value_pairs.iter()
    }
}

impl IntoIterator for KvStore {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = KeyValuePair;

    fn into_iter(self) -> Self::IntoIter {
        self.key_value_pairs.into_iter()
    }
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    MissingDelimeter(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl std::error::Error for Error {}

pub struct DisplayOptions {
    /// Whether to show all keys.
    ///
    /// This does *not* affect the visibility of secret material. However, the
    /// keys for secret material will be shown with the actual secret '*'ed
    /// out.
    pub show_all: bool,
    pub enabled_keys: HashSet<Key>,
}

impl Filetype for KvStore {
    type Error = Error;
    type Options = DisplayOptions;

    fn deserialize<R>(reader: &mut R) -> Result<Self, Self::Error>
    where
        R: io::Read,
    {
        let lines = BufReader::new(reader).lines();

        let key_value_pairs: Result<Vec<String>, Error> =
            lines.map(|line| line.map_err(Error::Io)).collect();

        let key_value_pairs: Vec<String> = key_value_pairs?;

        let key_value_pairs: Result<Vec<KeyValuePair>, Error> = key_value_pairs
            .into_iter()
            .map(|line| KeyValuePair::from_str(&line))
            .collect();

        let key_value_pairs: Vec<KeyValuePair> = key_value_pairs?;

        Ok(Self { key_value_pairs })
    }

    fn serialize<W, E>(self, mut writer: &mut E) -> Result<(), io::Error>
    where
        W: io::Write,
        E: Encrypted<W>,
    {
        for key_value_pair in self.into_iter() {
            writeln!(&mut writer, "{}", key_value_pair)?;
        }

        Ok(())
    }

    fn display<W>(
        &self,
        writer: &mut W,
        opts: DisplayOptions,
        show_secrets: bool,
    ) -> Result<(), io::Error>
    where
        W: io::Write,
    {
        for kv_pair in self.iter() {
            let key = &kv_pair.key;

            // Determine whether to show the value
            let value: String = match &kv_pair.value {
                Value::Sensitive(value) => {
                    if show_secrets {
                        // Only show sensitive values if secret_visibility is
                        // Show
                        value.clone()
                    } else {
                        // Otherwise, redact the secret with '*'s
                        // Show up to 16 '*'s.
                        let len = min(value.len(), 16);
                        "*".repeat(len)
                    }
                }
                // Always show insensitive values
                Value::Insensitive(value) => value.to_string(),
            };

            if opts.show_all || opts.enabled_keys.contains(key) {
                writeln!(writer, "{}: {}", key, value)?;
            }
        }

        Ok(())
    }
}

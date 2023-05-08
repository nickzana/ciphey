use std::ffi::OsString;
use std::path::PathBuf;

xflags::xflags! {
    /// A password manager based on age cryptography.
    cmd ciphey {
        /// Show help message
        optional -h, --help
        /// The path to the ciphey store, defaults to ~/.local/share/ciphey
        optional -p, --path path: PathBuf
        /// Display secret data in output.
        optional --show
        /// Initializes a store at the provided path or the ciphey default
        cmd init {}
        /// Create a new password entry
        cmd new {
            /// The name of the entry.
            optional -n, --name name: OsString
            /// Additional recipients who can access the entry.
            repeated -r, --recipient recipients: OsString
            /// Add additional key/value pairs to the entry.
            /// Key and value are split by the first equals sign.
            /// EXAMPLE: ciphey new -k email=user@example.com
            repeated -k, --key pair: OsString
            /// Optionally pass entry secret in via command line.
            optional -s, --secret secret: OsString
        }
        /// Lists the name and username of each entry.
        /// By default, shows 'name', 'username', 'email', and 'url'.
        cmd list {
            /// Display all of each entry's fields.
            optional -a, --all
            /// Do not display the default keys.
            optional -n, --no-default
            /// Also display values for the provided key.
            /// EXAMPLE: ciphey list --display tags
            repeated -d, --display key: OsString
            // TODO: should this become a ciphey-wide command?
            /// Only display explicitly requested output. Useful for scripts.
            optional --quiet
        }
        default cmd help {}
    }
}

pub mod util {
    use std::ffi::OsString;
    use std::str::FromStr;

    use ciphey_kvstore::KeyValuePair;
    use libciphey::crypto::Recipient;
    use xflags::Error;

    use crate::cli;

    pub fn parse_os_str<'a>(
        s: &'a OsString,
        message: &str,
    ) -> Result<&'a str, Error> {
        s.to_str()
            .ok_or(format!("{}: {:#?}", message, s))
            .map_err(Error::new)
    }

    /// Parses a list of recipients passed in as command line arguments.
    pub fn parse_recipients<R>(recipients: &[OsString]) -> Result<Vec<R>, Error>
    where
        R: Recipient,
    {
        let mut parsed_recipients: Vec<R> = Vec::new();
        for recipient in recipients {
            // Parse OsString as Rust String
            let recipient: &str = parse_os_str(
                recipient,
                "Recipient contains invalid characters",
            )?;

            // Parse recipient String as a Recipient for the crypto backend
            let recipient = recipient
                .to_string()
                .try_into()
                .map_err(|_| format!("Invalid Recipient: {}", recipient))
                .map_err(xflags::Error::new)?;

            parsed_recipients.push(recipient);
        }

        Ok(parsed_recipients)
    }

    pub fn parse_key_value_pairs(
        key_value_pairs: &[OsString],
    ) -> Result<Vec<KeyValuePair>, cli::Error> {
        let validated_strings: Vec<&str> = key_value_pairs
            .iter()
            .map(|os_str| {
                parse_os_str(
                    os_str,
                    "Invalid characters in key/value pair: {:#?}",
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let key_value_pairs = validated_strings
            .into_iter()
            .map(KeyValuePair::from_str)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Box::new)
            .map_err(|e| {
                cli::Error::Filetype(e as Box<dyn std::error::Error>)
            })?;

        Ok(key_value_pairs)
    }
}

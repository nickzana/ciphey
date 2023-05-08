use std::collections::HashSet;
use std::io::{BufRead, Write};

use ciphey_kvstore::{DisplayOptions, Key, KeyValuePair, KvStore, Value};
use libciphey::crypto;
use libciphey::filetype::Filetype;
use libciphey::storage::{self, Reference};
use uuid::Uuid;

use crate::flags::util::{
    parse_key_value_pairs, parse_os_str, parse_recipients,
};
use crate::flags::{Ciphey, List, New};

pub mod defaults;
pub mod error;
pub mod util;

pub use error::*;

use self::util::prompt_input;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SecretVisibility {
    Show,
    Hide,
}

impl Default for SecretVisibility {
    fn default() -> Self {
        Self::Hide
    }
}

/// Prints the command line help message.
pub fn help() {
    println!("{}", Ciphey::HELP);
}

/// Initializes a new vault at the provided path.
pub fn init<S>(storage: &mut S) -> Result<(), Error>
where
    S: storage::Backend,
{
    storage.create()?;
    Ok(())
}

/// Creates a new entry in the provided vault.
pub fn new<C, S, R, W>(
    opts: &New,
    crypto: &C,
    storage: &mut S,
    input: &mut R,
    output: &mut W,
) -> Result<(), Error>
where
    C: crypto::Backend,
    S: storage::Backend,
    R: BufRead,
    W: Write,
{
    let recipients = parse_recipients::<C::Recipient>(&opts.recipient)?;

    // Prompt for name if it was not passed in as an argument
    let name = match &opts.name {
        Some(s) => parse_os_str(s, "Invalid Name")
            .map(str::to_string)
            .map_err(Error::Xflags),
        None => prompt_input(false, "Entry Name: ", input, output)
            .map_err(Error::Input),
    }?;

    // Prompt for secret if it was not passed in as an argument
    let secret = match &opts.secret {
        // Secret was passed in as argument
        Some(s) => parse_os_str(s, "Invalid Secret")
            .map(str::to_string)
            .map_err(Error::Xflags),
        // Prompt for secret
        None => {
            prompt_input(true, "Secret: ", input, output).map_err(Error::Input)
        }
    }?;

    // Parse all other key/value pairs passed in as arguments
    let mut key_value_pairs = parse_key_value_pairs(&opts.key)?;

    // It's convenient to have the name as the first field, so insert it at
    // the front of the list.
    key_value_pairs
        .insert(0, KeyValuePair::new("name", Value::Insensitive(name)));

    key_value_pairs
        .insert(1, KeyValuePair::new("secret", Value::Sensitive(secret)));

    let store = KvStore::new(key_value_pairs);

    // Save the content to storage
    let uuid = Uuid::new_v4();
    let mut reference = storage.add_entry(&uuid)?;
    let writer = reference.writer()?;
    // TODO: Handle crypto error
    let mut encrypted = crypto.encrypt_output(writer, recipients).unwrap();

    store.serialize(&mut encrypted)?;

    writeln!(output, "Created new entry at path: {}", &reference)?;

    Ok(())
}

/// Lists all entries within the provided vault.
pub fn list<C, S, W>(
    opts: &List,
    secret_visibility: SecretVisibility,
    crypto: &C,
    storage: &mut S,
    output: &mut W,
) -> Result<(), Error>
where
    C: crypto::Backend,
    S: storage::Backend,
    W: Write,
{
    let entries = storage.entries()?;

    let references = entries.values();

    // Display statistics if quiet flag is not set
    if !opts.quiet {
        let count = &references.len();

        // Because English is weird
        let plural = if *count == 1 { "Entry" } else { "Entries" };

        writeln!(output, "Found {} {}", count, plural)?;
    }

    for reference in references {
        // Print a separator between every entry
        // TODO: Should this be included on the first entry?
        writeln!(output, "---")?;

        let reader = reference.reader()?;

        // Get a decrpted reader over the contents of the entry
        let mut decrypted = crypto
            .decrypt_input(reader)
            .map_err(|err| Error::Crypto(Box::new(err)))?;

        // Display options for all KvStores
        let show_secrets = match secret_visibility {
            SecretVisibility::Show => true,
            SecretVisibility::Hide => false,
        };

        let store = KvStore::deserialize(&mut decrypted)
            .map_err(Box::new)
            .map_err(|e| Error::Filetype(e as Box<dyn std::error::Error>))?;

        // Enable default keys, or no keys if "no-default" flag is set
        let mut enabled_keys: HashSet<Key> = if !opts.no_default {
            HashSet::from_iter(defaults::KEYS.iter().cloned())
        } else {
            HashSet::new()
        };

        // Enable any additional keys that the user explicitly asked to show.
        for key in &opts.display {
            let key = key
                .clone()
                .into_string()
                .map_err(Error::OsStringConversionError)?;
            enabled_keys.insert(Key::from(key.as_str()));
        }

        let opts = DisplayOptions {
            show_all: opts.all,
            enabled_keys,
        };

        store.display(output, opts, show_secrets)?;
    }

    Ok(())
}

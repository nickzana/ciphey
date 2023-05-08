#![feature(generic_associated_types, io_error_more)]
use std::io::{stdin, stdout, BufReader};

use cli::{defaults, SecretVisibility};
use flags::Ciphey;
use libciphey_fs::Filesystem;

use crate::backends::crypto::transparent::Transparent;

#[cfg(test)]
pub mod tests;

mod backends;
mod cli;
mod flags;

fn main() -> Result<(), cli::Error> {
    // Parse arguments into generated xflags structs
    let args = Ciphey::from_env()?;

    // Check for help flag
    if args.help {
        cli::help();
        return Ok(());
    }

    // Indicates whether to show or hide secret material in the output
    let visibility = if args.show {
        SecretVisibility::Show
    } else {
        SecretVisibility::default()
    };

    // The provided path to the ciphey store. If no path was provided, the
    // default path will be used.
    let store_path = args.path.unwrap_or_else(defaults::store_dir);

    // TODO: Add mechanism for detecting/choosing crypto algorithm
    let crypto = Transparent {};
    let mut storage = Filesystem::new(&store_path)?;

    let mut output = stdout();
    let input = stdin();
    let mut input = BufReader::new(input);

    match args.subcommand {
        flags::CipheyCmd::Help(_) => {
            cli::help();
            Ok(())
        }
        flags::CipheyCmd::Init(..) => {
            cli::init(&mut storage)?;
            println!(
                "Successfully created vault at path: {}",
                store_path.display()
            );
            Ok(())
        }
        flags::CipheyCmd::New(opts) => {
            cli::new(&opts, &crypto, &mut storage, &mut input, &mut output)
        }
        flags::CipheyCmd::List(mut opts) => {
            cli::list(&mut opts, visibility, &crypto, &mut storage, &mut output)
        }
    }
}

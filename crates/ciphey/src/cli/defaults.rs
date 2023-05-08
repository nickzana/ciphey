use std::path::PathBuf;

use ciphey_kvstore::Key;

// Default path for ciphey store
pub const STORE_DIR: &[&str] = &[env!("HOME"), ".local", "share", "ciphey"];
// pub const RECIPIENTS_PATH: &[&str] = &[".identities"];

// Default field keys to display
pub const KEYS: &[Key] = &[Key::Name, Key::Username, Key::Email, Key::Url];

// Returns `PathBuf` of default path to ciphey store.
pub fn store_dir() -> PathBuf {
    STORE_DIR.iter().collect()
}

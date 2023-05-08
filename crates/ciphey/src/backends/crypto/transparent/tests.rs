use std::io::{Read, Write};

use libciphey::crypto::Backend;

use super::Transparent;

const PLAINTEXT: &str = "Secret Data";
const CIPHERTEXT: &str = r"-> Public Key A
-> Public Key B
---
Secret Data";

#[test]
fn test_encrypt() {
    let crypto = Transparent {};

    let mut buf = Vec::new();
    let recipients = vec![
        "Public Key A".to_string().into(),
        "Public Key B".to_string().into(),
    ];

    let mut dump = crypto.encrypt_output(&mut buf, recipients).unwrap();
    write!(&mut dump, "{}", PLAINTEXT).unwrap();

    assert_eq!(String::from_utf8(buf).unwrap(), CIPHERTEXT.to_string());
}

#[test]
fn test_decrypt() {
    let crypto = Transparent {};

    let mut plaintext = String::new();
    let mut plaintext_reader =
        crypto.decrypt_input(CIPHERTEXT.as_bytes()).unwrap();

    plaintext_reader.read_to_string(&mut plaintext).unwrap();

    assert_eq!(plaintext, PLAINTEXT);
}

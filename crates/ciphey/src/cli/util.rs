use std::io::{BufRead, Write};

use rpassword::prompt_password;

/// Prompt the user for a line of text.
pub fn prompt_input<R, W>(
    hidden: bool,
    prompt: &str,
    input: &mut R,
    output: &mut W,
) -> std::io::Result<String>
where
    R: BufRead,
    W: Write,
{
    if hidden {
        prompt_password(prompt)
    } else {
        prompt_shown_input(prompt, input, output)
    }
}

pub fn prompt_shown_input<R, W>(
    prompt: &str,
    input: &mut R,
    output: &mut W,
) -> std::io::Result<String>
where
    R: BufRead,
    W: Write,
{
    // Print the prompt.
    write!(output, "{}", prompt)?;
    output.flush()?;

    // Read a line from the input reader.
    let mut value = String::new();
    input.read_line(&mut value)?;
    value = value.trim_end().to_string();

    Ok(value)
}

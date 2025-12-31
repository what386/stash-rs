use std::io::{self, Write};

/// Read a single line from stdin, trimmed
fn read_line() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Prompt the user with a yes/no question.
/// Keeps asking until a valid response is entered.
pub fn prompt_bool(prompt: &str) -> io::Result<bool> {
    loop {
        print!("{prompt} ");
        io::stdout().flush()?; // ensure prompt is shown

        let input = read_line()?.to_lowercase();

        match input.as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {
                println!("Please enter 'y' or 'n'.");
            }
        }
    }
}

/// Prompt the user for a string (non-empty).
pub fn prompt_string(prompt: &str) -> io::Result<String> {
    loop {
        print!("{prompt} ");
        io::stdout().flush()?;

        let input = read_line()?;
        if !input.is_empty() {
            return Ok(input);
        }

        println!("Input cannot be empty.");
    }
}

/// Prompt the user for an optional string.
/// Empty input returns None.
pub fn prompt_optional_string(prompt: &str) -> io::Result<Option<String>> {
    print!("{prompt} ");
    io::stdout().flush()?;

    let input = read_line()?;
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

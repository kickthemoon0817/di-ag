use super::read_input;

pub fn run(input: &str, check: bool) -> Result<(), String> {
    let content = read_input(input)?;

    let mut formatted = String::new();
    let mut prev_blank = false;

    for line in content.lines() {
        let trimmed = line.trim_end();
        let is_blank = trimmed.is_empty();

        if is_blank {
            if !prev_blank {
                formatted.push('\n');
            }
            prev_blank = true;
        } else {
            formatted.push_str(trimmed);
            formatted.push('\n');
            prev_blank = false;
        }
    }

    let formatted = formatted.trim_end().to_string() + "\n";

    if check {
        if content != formatted {
            eprintln!("File needs formatting: {}", input);
            std::process::exit(1);
        } else {
            eprintln!("File is formatted correctly.");
        }
    } else if input != "-" {
        std::fs::write(input, &formatted)
            .map_err(|e| format!("Failed to write '{}': {}", input, e))?;
        eprintln!("Formatted {}", input);
    } else {
        print!("{}", formatted);
    }

    Ok(())
}

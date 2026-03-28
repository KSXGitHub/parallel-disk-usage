use crate::args::Args;
use clap::CommandFactory;
use clap_mangen::Man;
use std::io;

/// Renders the man page for `pdu` as a string.
pub fn render_man_page() -> io::Result<String> {
    let command = Args::command();
    let man = Man::new(command);
    let mut buffer = Vec::new();
    man.render(&mut buffer)?;
    let content = String::from_utf8(buffer)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(postprocess_man_page(&content))
}

fn postprocess_man_page(content: &str) -> String {
    let mut output = String::with_capacity(content.len());
    let mut in_examples = false;

    for line in content.lines() {
        if line == ".SH EXTRA" {
            output.push_str(".SH EXAMPLES\n");
            in_examples = true;
            continue;
        }

        if in_examples && line.starts_with(".SH ") {
            in_examples = false;
        }

        if in_examples {
            let trimmed = line.strip_prefix("    ").unwrap_or(line);
            if trimmed == "Examples:" {
                continue;
            }
            output.push_str(trimmed);
        } else {
            output.push_str(line);
        }
        output.push('\n');
    }

    output
}

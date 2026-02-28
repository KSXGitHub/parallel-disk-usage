fn main() {
    let help = include_str!("../exports/long.help");
    println!("{}", render(help).trim_end());
}

fn render(input: &str) -> String {
    let mut out = String::new();
    let lines: Vec<&str> = input.lines().collect();

    let section_positions: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| is_section_header(l))
        .map(|(i, _)| i)
        .collect();

    let first_section = section_positions.first().copied().unwrap_or(lines.len());
    render_preamble(&lines[..first_section], &mut out);

    for (idx, &start) in section_positions.iter().enumerate() {
        let end = section_positions
            .get(idx + 1)
            .copied()
            .unwrap_or(lines.len());
        let name = lines[start].strip_suffix(':').unwrap_or(lines[start]);
        render_section(name, &lines[start + 1..end], &mut out);
    }

    out
}

/// A top-level section header is a single unindented word followed by `:`
/// (e.g. `Arguments:`, `Options:`, `Examples:`).
/// Lines like `Usage: pdu …` or `Copyright: …` do not qualify because they
/// either contain spaces before `:` content or have trailing content.
fn is_section_header(line: &str) -> bool {
    !line.starts_with(' ')
        && line
            .strip_suffix(':')
            .is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_alphabetic()))
}

fn render_preamble(lines: &[&str], out: &mut String) {
    for line in lines {
        if let Some(rest) = line.strip_prefix("Usage: ") {
            out.push_str("# Usage\n\n```sh\n");
            out.push_str(rest);
            out.push_str("\n```\n\n");
        }
    }
}

fn render_section(name: &str, lines: &[&str], out: &mut String) {
    out.push_str(&format!("## {name}\n\n"));
    match name {
        "Arguments" | "Options" => render_flag_section(lines, out),
        "Examples" => render_examples_section(lines, out),
        _ => {}
    }
}

fn render_flag_section(lines: &[&str], out: &mut String) {
    // Flag / argument names are indented 2–6 spaces; descriptions are indented 10+.
    let item_starts: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| {
            let trimmed = l.trim_start();
            if trimmed.is_empty() {
                return false;
            }
            let indent = l.len() - trimmed.len();
            (2..=6).contains(&indent)
        })
        .map(|(i, _)| i)
        .collect();

    for (idx, &start) in item_starts.iter().enumerate() {
        let end = item_starts.get(idx + 1).copied().unwrap_or(lines.len());
        render_flag_item(&lines[start..end], out);
    }
    out.push('\n');
}

/// Format a flag line and its aliases as a comma-separated list of backtick-quoted names.
///
/// For example, `-b, --bytes-format <BYTES_FORMAT>` with no aliases becomes
/// `` `-b <BYTES_FORMAT>`, `--bytes-format <BYTES_FORMAT>` ``.
///
/// For `-H, --deduplicate-hardlinks` with aliases `--detect-links, --dedupe-links` the
/// result is `` `-H`, `--deduplicate-hardlinks`, `--detect-links`, `--dedupe-links` ``.
fn format_flag_names(flag_line: &str, aliases: Option<&str>) -> String {
    let parts: Vec<&str> = flag_line.split(", ").collect();

    // Extract the value placeholder suffix from the last part (e.g. " <BYTES_FORMAT>").
    let value_suffix = parts
        .last()
        .and_then(|p| p.find(' ').map(|i| &p[i..]))
        .unwrap_or("");

    let mut names: Vec<String> = parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            if i < parts.len() - 1 && !value_suffix.is_empty() {
                format!("{part}{value_suffix}")
            } else {
                part.to_string()
            }
        })
        .collect();

    if let Some(a) = aliases {
        for alias in a.split(", ") {
            if value_suffix.is_empty() {
                names.push(alias.to_string());
            } else {
                names.push(format!("{alias}{value_suffix}"));
            }
        }
    }

    names
        .iter()
        .map(|n| format!("`{n}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_flag_item(lines: &[&str], out: &mut String) {
    if lines.is_empty() {
        return;
    }
    let flag = lines[0].trim();

    let mut desc_parts: Vec<&str> = Vec::new();
    let mut possible_values: Vec<(&str, &str)> = Vec::new();
    let mut default_value: Option<&str> = None;
    let mut aliases_value: Option<&str> = None;
    let mut in_possible_values = false;

    for line in &lines[1..] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            in_possible_values = false;
            continue;
        }
        if trimmed == "Possible values:" {
            in_possible_values = true;
            continue;
        }
        if let Some(inner) = trimmed
            .strip_prefix("[default: ")
            .and_then(|s| s.strip_suffix(']'))
        {
            default_value = Some(inner);
            continue;
        }
        if let Some(inner) = trimmed
            .strip_prefix("[aliases: ")
            .and_then(|s| s.strip_suffix(']'))
        {
            aliases_value = Some(inner);
            continue;
        }
        if in_possible_values {
            if let Some(entry) = trimmed.strip_prefix("- ") {
                let (name, desc) = if let Some(colon) = entry.find(':') {
                    (entry[..colon].trim(), entry[colon + 1..].trim())
                } else {
                    (entry.trim(), "")
                };
                possible_values.push((name, desc));
            }
            continue;
        }
        desc_parts.push(trimmed);
    }

    let names = format_flag_names(flag, aliases_value);
    let description = desc_parts.join(" ");
    out.push_str(&format!("* {names}: {description}"));
    if let Some(d) = default_value {
        out.push_str(&format!(" (default: `{d}`)"));
    }
    out.push('\n');

    for (name, desc) in &possible_values {
        if desc.is_empty() {
            out.push_str(&format!("  * `{name}`\n"));
        } else {
            out.push_str(&format!("  * `{name}`: {desc}\n"));
        }
    }
}

fn render_examples_section(lines: &[&str], out: &mut String) {
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() {
            i += 1;
            continue;
        }
        // A line starting with `$ ` is a bare command (no preceding description).
        if let Some(cmd) = trimmed.strip_prefix("$ ") {
            out.push_str(&format!("### `{cmd}`\n\n```sh\n{cmd}\n```\n\n"));
            i += 1;
        } else {
            // Description line — the very next non-empty line should be `$ <cmd>`.
            let desc = trimmed;
            i += 1;
            while i < lines.len() && lines[i].trim().is_empty() {
                i += 1;
            }
            if i < lines.len() {
                if let Some(cmd) = lines[i].trim().strip_prefix("$ ") {
                    out.push_str(&format!("### {desc}\n\n```sh\n{cmd}\n```\n\n"));
                    i += 1;
                    continue;
                }
            }
            out.push_str(&format!("### {desc}\n\n"));
        }
    }
}

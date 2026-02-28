use itertools::Itertools;

pub fn render(input: &str) -> String {
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

/// A top-level section header is a single unindented non-empty alphanumeric word followed by `:`.
fn is_section_header(line: &str) -> bool {
    if line.starts_with(' ') || line.starts_with('\t') {
        return false;
    }

    let Some(prefix) = line.strip_suffix(':') else {
        return false;
    };

    if prefix.is_empty() {
        return false;
    }

    prefix.chars().all(|c| c.is_alphanumeric())
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
        "Arguments" => render_arguments_section(lines, out),
        "Options" => render_options_section(lines, out),
        "Examples" => render_examples_section(lines, out),
        _ => {}
    }
}

/// Find the start positions of flag/argument items within a section's lines.
///
/// Flags, arguments, enum choices, and descriptions are indented by either tab or spaces.
/// The name of flag starts with `-` or `--`, followed by a kebab-case word.
/// The name of an argument is an UPPER_SNAKE_CASE enclosed in square brackets.
/// The name of an enum choice is a kebab-case word enclosed in square brackets.
fn find_item_starts(lines: &[&str]) -> Vec<usize> {
    lines
        .iter()
        .enumerate()
        .filter(|(_, l)| is_item_start(l))
        .map(|(i, _)| i)
        .collect()
}

fn is_item_start(line: &str) -> bool {
    if !line.starts_with(' ') && !line.starts_with('\t') {
        return false;
    }
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    // Flag: starts with `-` followed by `-` or alphanumeric (excludes `- ` bullet lists)
    if let Some(rest) = trimmed.strip_prefix('-') {
        if rest.starts_with('-') || rest.starts_with(|c: char| c.is_alphanumeric()) {
            return true;
        }
    }
    // Argument: [UPPER_SNAKE_CASE]
    if let Some(rest) = trimmed.strip_prefix('[') {
        let name = rest.split(']').next().unwrap_or("");
        return !name.is_empty() && name.chars().all(|c| c.is_uppercase() || c == '_');
    }
    false
}

fn render_arguments_section(lines: &[&str], out: &mut String) {
    let item_starts = find_item_starts(lines);
    for (idx, &start) in item_starts.iter().enumerate() {
        let end = item_starts.get(idx + 1).copied().unwrap_or(lines.len());
        render_argument_item(&lines[start..end], out);
    }
    out.push('\n');
}

fn render_argument_item(lines: &[&str], out: &mut String) {
    if lines.is_empty() {
        return;
    }
    let name = lines[0].trim();
    let desc = lines[1..]
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .join(" ");
    let desc = ensure_ends_with_period(&desc);
    out.push_str(&format!("* `{name}`: {desc}\n"));
}

fn render_options_section(lines: &[&str], out: &mut String) {
    let item_starts = find_item_starts(lines);
    for (idx, &start) in item_starts.iter().enumerate() {
        let end = item_starts.get(idx + 1).copied().unwrap_or(lines.len());
        render_option_item(&lines[start..end], out);
    }
}

fn render_option_item(lines: &[&str], out: &mut String) {
    if lines.is_empty() {
        return;
    }
    let flag_line = lines[0].trim();

    let (primary_name, short_alias) = parse_flag_primary(flag_line);

    let mut desc_parts: Vec<&str> = Vec::new();
    let mut possible_values: Vec<(&str, &str)> = Vec::new();
    let mut default_value: Option<&str> = None;
    let mut long_aliases_str: Option<&str> = None;
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
            long_aliases_str = Some(inner);
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

    // Build aliases list: short alias first, then long aliases from [aliases: ...]
    let mut aliases: Vec<String> = Vec::new();
    if let Some(s) = &short_alias {
        aliases.push(s.clone());
    }
    if let Some(a) = long_aliases_str {
        for alias in a.split(", ") {
            aliases.push(alias.to_string());
        }
    }

    // Invisible anchors: short aliases (prefixed with "option-"), then primary name, then long aliases
    let primary_anchor = primary_name.trim_start_matches('-');
    let mut anchor_ids: Vec<String> = Vec::new();
    if let Some(s) = &short_alias {
        let bare = s.trim_start_matches('-');
        anchor_ids.push(format!("option-{bare}"));
    }
    anchor_ids.push(primary_anchor.to_string());
    if let Some(a) = long_aliases_str {
        for alias in a.split(", ") {
            anchor_ids.push(alias.trim_start_matches('-').to_string());
        }
    }
    let anchors: String = anchor_ids
        .iter()
        .map(|id| format!(r#"<a id="{id}" name="{id}"></a>"#))
        .join("");
    out.push_str(&format!("{anchors}\n"));

    // Heading
    out.push_str(&format!("### `{primary_name}`\n\n"));

    // Metadata bullets
    let has_metadata =
        !aliases.is_empty() || default_value.is_some() || !possible_values.is_empty();

    if !aliases.is_empty() {
        let aliases_str = aliases.iter().map(|a| format!("`{a}`")).join(", ");
        out.push_str(&format!("* _Aliases:_ {aliases_str}.\n"));
    }
    if let Some(d) = default_value {
        out.push_str(&format!("* _Default:_ `{d}`.\n"));
    }
    if !possible_values.is_empty() {
        out.push_str("* _Choices:_\n");
        for (name, desc) in &possible_values {
            if desc.is_empty() {
                out.push_str(&format!("  - `{name}`\n"));
            } else {
                out.push_str(&format!("  - `{name}`: {desc}\n"));
            }
        }
    }

    if has_metadata {
        out.push('\n');
    }

    // Description
    let description = desc_parts.join(" ");
    if !description.is_empty() {
        let description = ensure_ends_with_period(&description);
        out.push_str(&format!("{description}\n\n"));
    } else {
        out.push('\n');
    }
}

/// Extract the primary (long) flag name and optional short alias from a flag definition line.
///
/// E.g. `-b, --bytes-format <BYTES_FORMAT>` → (`--bytes-format`, `Some("-b")`)
fn parse_flag_primary(flag_line: &str) -> (String, Option<String>) {
    let mut short_alias: Option<String> = None;
    let mut primary_name = flag_line.to_string();

    for part in flag_line.split(", ") {
        let name_only = part.split_whitespace().next().unwrap_or(part);
        if name_only.starts_with("--") {
            primary_name = name_only.to_string();
        } else if name_only.starts_with('-') {
            short_alias = Some(name_only.to_string());
        }
    }

    (primary_name, short_alias)
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

fn ensure_ends_with_period(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    if s.ends_with('.') || s.ends_with('!') || s.ends_with('?') {
        s.to_string()
    } else {
        format!("{s}.")
    }
}

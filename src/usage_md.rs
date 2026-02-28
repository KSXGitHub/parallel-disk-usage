use itertools::Itertools;
use std::borrow::Cow;

use clap::{Arg, Command, CommandFactory};

use crate::args::Args;

/// Renders a Markdown reference page for `pdu`'s CLI.
pub fn render_usage_md() -> String {
    let mut command: Command = Args::command();
    let mut out = String::new();

    let usage = command.render_usage().to_string();
    if let Some(usage) = usage.strip_prefix("Usage:") {
        out.push_str("# Usage\n\n```sh\n");
        out.push_str(usage.trim());
        out.push_str("\n```\n\n");
    }

    let mut arguments_heading_written = false;
    for arg in command.get_arguments() {
        if !arg.is_positional() || arg.is_hide_set() || arg.is_hide_long_help_set() {
            continue;
        }
        if !arguments_heading_written {
            arguments_heading_written = true;
            out.push_str("## Arguments\n\n");
        }
        render_argument(arg, &mut out);
    }
    if arguments_heading_written {
        out.push('\n');
    }

    let mut options_heading_written = false;
    for arg in command.get_arguments() {
        if arg.is_positional() || arg.is_hide_set() || arg.is_hide_long_help_set() {
            continue;
        }
        if !options_heading_written {
            options_heading_written = true;
            out.push_str("## Options\n\n");
        }
        render_option(arg, &mut out);
    }

    if let Some(after_help) = command.get_after_long_help() {
        let text = after_help.to_string();
        let mut lines_iter = text.lines();
        let mut has_examples = false;
        for line in lines_iter.by_ref() {
            if line.trim() == "Examples:" {
                has_examples = true;
                break;
            }
        }
        if has_examples {
            out.push_str("## Examples\n\n");
            render_examples_section(lines_iter, &mut out);
        }
    }

    out
}

fn render_argument(arg: &Arg, out: &mut String) {
    let name = arg
        .get_value_names()
        .and_then(|names| names.first())
        .map(|n| n.as_str())
        .unwrap_or_else(|| arg.get_id().as_str());
    let is_multiple = arg
        .get_num_args()
        .map(|r| r.max_values() > 1)
        .unwrap_or(false);
    let display_name = if is_multiple {
        format!("[{name}]...")
    } else {
        format!("[{name}]")
    };
    let desc = get_help_text(arg);
    let desc = ensure_ends_with_period(&desc);
    out.push_str(&format!("* `{display_name}`: {desc}\n"));
}

fn render_option(arg: &Arg, out: &mut String) {
    let primary_long = arg.get_long().expect("option must have a long flag");
    let primary_name = format!("--{primary_long}");

    let short = arg.get_short();
    let visible_long_aliases: Vec<&str> = arg.get_visible_aliases().unwrap_or_default();
    let visible_short_aliases: Vec<char> = arg.get_visible_short_aliases().unwrap_or_default();

    // Invisible anchors: short first, then primary long, then long aliases
    let mut anchor_ids: Vec<String> = Vec::new();
    if let Some(s) = short {
        anchor_ids.push(format!("option-{s}"));
    }
    anchor_ids.push(primary_long.to_string());
    for &a in &visible_long_aliases {
        anchor_ids.push(a.to_string());
    }
    for c in &visible_short_aliases {
        anchor_ids.push(format!("option-{c}"));
    }
    for id in &anchor_ids {
        out.push_str(&format!(r#"<a id="{id}" name="{id}"></a>"#));
    }
    out.push('\n');

    // Heading
    out.push_str(&format!("### `{primary_name}`\n\n"));

    // Aliases for display in metadata
    let mut aliases: Vec<String> = Vec::new();
    if let Some(s) = short {
        aliases.push(format!("-{s}"));
    }
    for &a in &visible_long_aliases {
        aliases.push(format!("--{a}"));
    }
    for c in &visible_short_aliases {
        aliases.push(format!("-{c}"));
    }

    // Default values – skip "false" (clap's implicit default for boolean flags)
    let default_values: Vec<_> = if arg.is_hide_default_value_set() {
        Vec::new()
    } else {
        arg.get_default_values()
            .iter()
            .filter(|v| v.to_string_lossy() != "false")
            .collect()
    };

    // Possible values (choices)
    let possible_values: Vec<_> = if arg.is_hide_possible_values_set() {
        Vec::new()
    } else {
        arg.get_possible_values()
            .into_iter()
            .filter(|pv| !pv.is_hide_set())
            .collect()
    };

    let has_metadata =
        !aliases.is_empty() || !default_values.is_empty() || !possible_values.is_empty();

    if !aliases.is_empty() {
        let aliases_str = aliases.iter().map(|a| format!("`{a}`")).join(", ");
        out.push_str(&format!("* _Aliases:_ {aliases_str}.\n"));
    }
    if !default_values.is_empty() {
        let default_str = default_values
            .iter()
            .map(|v| v.to_string_lossy())
            .join(", ");
        out.push_str(&format!("* _Default:_ `{default_str}`.\n"));
    }
    if !possible_values.is_empty() {
        out.push_str("* _Choices:_\n");
        for pv in &possible_values {
            let name = pv.get_name();
            if let Some(help) = pv.get_help() {
                out.push_str(&format!("  - `{name}`: {help}\n"));
            } else {
                out.push_str(&format!("  - `{name}`\n"));
            }
        }
    }

    if has_metadata {
        out.push('\n');
    }

    // Description: short help, with long help appended if also set
    let description = get_help_text(arg);
    if !description.is_empty() {
        let description = ensure_ends_with_period(&description);
        out.push_str(&format!("{description}\n\n"));
    } else {
        out.push('\n');
    }
}

fn get_help_text(arg: &Arg) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(h) = arg.get_help() {
        parts.push(h.to_string());
    }
    if let Some(lh) = arg.get_long_help() {
        parts.push(lh.to_string());
    }
    parts.join("\n")
}

fn render_examples_section<'a>(lines: impl Iterator<Item = &'a str>, out: &mut String) {
    let mut current_title: Option<&'a str> = None;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(cmd) = trimmed.strip_prefix('$').map(str::trim) {
            let heading = if let Some(title) = current_title.take() {
                title.to_string()
            } else {
                format!("`{cmd}`")
            };
            out.push_str(&format!("### {heading}\n\n```sh\n{cmd}\n```\n\n"));
        } else {
            current_title = Some(trimmed);
        }
    }
}

fn ensure_ends_with_period(line: &str) -> Cow<'_, str> {
    if line.is_empty() || line.ends_with('.') || line.ends_with('!') || line.ends_with('?') {
        Cow::Borrowed(line)
    } else {
        Cow::Owned(format!("{line}."))
    }
}

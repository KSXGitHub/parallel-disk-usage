use crate::args::Args;
use clap::builder::PossibleValue;
use clap::{Arg, Command, CommandFactory};
use itertools::Itertools;
use pipe_trait::Pipe;
use std::borrow::Cow;

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
        render_argument(&mut out, arg);
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
        render_option(&mut out, arg);
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
            render_examples_section(&mut out, lines_iter);
        }
    }

    out
}

fn render_argument(out: &mut String, arg: &Arg) {
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
    let desc = ensure_ends_with_punctuation(&desc);
    out.push_str(&format!("* `{display_name}`: {desc}\n"));
}

fn render_option(out: &mut String, arg: &Arg) {
    let primary_long = arg.get_long().expect("option must have a long flag");
    let primary_name = format!("--{primary_long}");

    write_option_anchors(out, arg, primary_long);
    out.push_str(&format!("### `{primary_name}`\n\n"));

    let aliases = collect_option_display_aliases(arg);
    let default_values = collect_option_default_values(arg);
    let possible_values = collect_option_possible_values(arg);

    let has_metadata =
        !aliases.is_empty() || !default_values.is_empty() || !possible_values.is_empty();

    if !aliases.is_empty() {
        let aliases_str = aliases.iter().map(|alias| format!("`{alias}`")).join(", ");
        out.push_str(&format!("* _Aliases:_ {aliases_str}.\n"));
    }
    if !default_values.is_empty() {
        let default_values_str = default_values.join(", ");
        out.push_str(&format!("* _Default:_ `{default_values_str}`.\n"));
    }
    if !possible_values.is_empty() {
        out.push_str("* _Choices:_\n");
        for possible_value in &possible_values {
            let name = possible_value.get_name();
            if let Some(help) = possible_value.get_help() {
                out.push_str(&format!("  - `{name}`: {help}\n"));
            } else {
                out.push_str(&format!("  - `{name}`\n"));
            }
        }
    }

    if has_metadata {
        out.push('\n');
    }

    write_option_description(out, arg);
}

fn write_option_anchors(out: &mut String, arg: &Arg, primary_long: &str) {
    let append_anchor = |out: &mut String, id: &str| {
        out.push_str(&format!(r#"<a id="{id}" name="{id}"></a>"#));
    };
    let append_anchor_for_short = |out: &mut String, short: char| {
        append_anchor(out, &format!("option-{short}"));
    };
    if let Some(short) = arg.get_short() {
        append_anchor_for_short(out, short);
    }
    append_anchor(out, primary_long);
    for alias in arg.get_visible_aliases().unwrap_or_default() {
        append_anchor(out, alias);
    }
    for short in arg.get_visible_short_aliases().unwrap_or_default() {
        append_anchor_for_short(out, short);
    }
    out.push('\n');
}

fn collect_option_display_aliases(arg: &Arg) -> Vec<String> {
    let mut aliases = Vec::<String>::new();
    if let Some(short) = arg.get_short() {
        aliases.push(format!("-{short}"));
    }
    for alias in arg.get_visible_aliases().unwrap_or_default() {
        aliases.push(format!("--{alias}"));
    }
    for alias in arg.get_visible_short_aliases().unwrap_or_default() {
        aliases.push(format!("-{alias}"));
    }
    aliases
}

fn collect_option_default_values(arg: &Arg) -> Vec<String> {
    if arg.is_hide_default_value_set() {
        return Vec::new();
    }
    arg.get_default_values()
        .iter()
        .map(|value| value.to_string_lossy())
        .filter(|value| value != "false")
        .map(|value| value.to_string())
        .collect()
}

fn collect_option_possible_values(arg: &Arg) -> Vec<PossibleValue> {
    if arg.is_hide_possible_values_set() {
        return Vec::new();
    }
    arg.get_possible_values()
        .into_iter()
        .filter(|possible_value| !possible_value.is_hide_set())
        .collect()
}

fn write_option_description(out: &mut String, arg: &Arg) {
    let description = get_help_text(arg);
    if !description.is_empty() {
        let description = ensure_ends_with_punctuation(&description);
        out.push_str(&format!("{description}\n\n"));
    } else {
        out.push('\n');
    }
}

fn get_help_text(arg: &Arg) -> Cow<'static, str> {
    match (arg.get_help(), arg.get_long_help()) {
        (None, None) => Cow::Borrowed(""),
        (Some(help), None) | (None, Some(help)) => help.to_string().pipe(Cow::Owned),
        (Some(short), Some(long)) => Cow::Owned(format!("{short}\n{long}")),
    }
}

fn render_examples_section<'a>(out: &mut String, lines: impl Iterator<Item = &'a str>) {
    for line in lines {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        if let Some(command) = line.strip_prefix('$') {
            let command = command.trim();
            out.push_str(&format!("```sh\n{command}\n```\n\n"));
            continue;
        }

        out.push_str(&format!("### {line}\n\n"));
    }
}

fn ensure_ends_with_punctuation(line: &str) -> Cow<'_, str> {
    if line.is_empty() || line.ends_with('.') || line.ends_with('!') || line.ends_with('?') {
        Cow::Borrowed(line)
    } else {
        Cow::Owned(format!("{line}."))
    }
}

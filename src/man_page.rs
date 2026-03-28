use crate::args::Args;
use clap::{Arg, ArgAction, Command, CommandFactory};
use std::{borrow::Cow, collections::BTreeMap, fmt::Write};

/// A map from argument ID to the set of argument IDs it conflicts with (bidirectional).
type ConflictMap = BTreeMap<String, Vec<String>>;

/// Renders the man page for `pdu` as a string in roff format.
pub fn render_man_page() -> String {
    let mut command = Args::command();
    command.build();
    let conflict_map = build_conflict_map(&command);
    let mut out = String::new();
    render_title(&mut out, &command);
    render_name_section(&mut out, &command);
    render_synopsis_section(&mut out, &command);
    render_description_section(&mut out, &command);
    render_options_section(&mut out, &command, &conflict_map);
    render_examples_section(&mut out, &command);
    render_version_section(&mut out, &command);
    out
}

/// Builds a bidirectional conflict map from clap's one-directional conflict declarations.
fn build_conflict_map(command: &Command) -> ConflictMap {
    let mut map = ConflictMap::new();
    for arg in command.get_arguments() {
        let arg_id = arg.get_id().to_string();
        for conflict in command.get_arg_conflicts_with(arg) {
            let conflict_id = conflict.get_id().to_string();
            map.entry(arg_id.clone())
                .or_default()
                .push(conflict_id.clone());
            map.entry(conflict_id).or_default().push(arg_id.clone());
        }
    }
    // Deduplicate each entry
    for conflicts in map.values_mut() {
        conflicts.sort();
        conflicts.dedup();
    }
    map
}

/// Resolves an argument ID to its `--long` flag name for display.
fn resolve_flag_name(command: &Command, arg_id: &str) -> Option<String> {
    command
        .get_arguments()
        .find(|arg| arg.get_id().as_str() == arg_id)
        .and_then(|arg| arg.get_long())
        .map(|long| format!("\\fB\\-\\-{}\\fR", roff_escape(long)))
}

/// Escapes a string for roff by replacing hyphens with `\-`.
fn roff_escape(text: &str) -> String {
    text.replace('-', r"\-")
}

fn render_title(out: &mut String, command: &Command) {
    let name = command.get_name();
    let version = command.get_version().unwrap_or_default();
    writeln!(out, ".TH {name} 1 \"{name} {version}\"").unwrap();
}

fn render_name_section(out: &mut String, command: &Command) {
    let name = command.get_name();
    let about = command
        .get_about()
        .map(ToString::to_string)
        .unwrap_or_default();
    writeln!(out, ".SH NAME").unwrap();
    writeln!(out, "{name} \\- {}", roff_escape(&about)).unwrap();
}

fn render_synopsis_section(out: &mut String, command: &Command) {
    out.push_str(".SH SYNOPSIS\n");
    out.push_str(&format!("\\fB{}\\fR", command.get_name()));
    for arg in command.get_arguments() {
        if arg.is_positional() {
            continue;
        }
        if arg.is_hide_set() {
            continue;
        }
        out.push(' ');
        render_synopsis_option(out, arg);
    }
    for arg in command.get_arguments() {
        if !arg.is_positional() || arg.is_hide_set() {
            continue;
        }
        out.push(' ');
        render_synopsis_positional(out, arg);
    }
    out.push('\n');
}

fn render_synopsis_option(out: &mut String, arg: &Arg) {
    out.push('[');
    if let Some(short) = arg.get_short() {
        write!(out, "\\fB\\-{}\\fR", roff_escape(&short.to_string())).unwrap();
        if arg.get_long().is_some() {
            out.push('|');
        }
    }
    if let Some(long) = arg.get_long() {
        write!(out, "\\fB\\-\\-{}\\fR", roff_escape(long)).unwrap();
    }
    if arg.get_action().takes_values() {
        if let Some(value_names) = arg.get_value_names() {
            for name in value_names {
                write!(out, " \\fI{}\\fR", roff_escape(name)).unwrap();
            }
        }
    }
    out.push(']');
}

fn render_synopsis_positional(out: &mut String, arg: &Arg) {
    let name = arg
        .get_value_names()
        .and_then(|names| names.first())
        .map(|name| name.as_str())
        .unwrap_or_else(|| arg.get_id().as_str());
    if arg.is_required_set() {
        write!(out, "\\fI{}\\fR", roff_escape(name)).unwrap();
    } else {
        write!(out, "[\\fI{}\\fR]", roff_escape(name)).unwrap();
    }
}

fn render_description_section(out: &mut String, command: &Command) {
    out.push_str(".SH DESCRIPTION\n");
    let text = command
        .get_long_about()
        .or_else(|| command.get_about())
        .map(ToString::to_string)
        .unwrap_or_default();
    render_paragraph_text(out, &text);
}

/// Renders multi-line text with proper roff paragraph breaks.
///
/// Empty lines in the input produce `.PP` (new paragraph) in the output.
/// Consecutive non-empty lines are joined with `.br` (line break).
fn render_paragraph_text(out: &mut String, text: &str) {
    let mut need_paragraph = false;
    let mut first = true;
    for line in text.lines() {
        if line.is_empty() {
            need_paragraph = true;
            continue;
        }
        if need_paragraph && !first {
            out.push_str(".PP\n");
            need_paragraph = false;
        } else if !first {
            out.push_str(".br\n");
        }
        first = false;
        writeln!(out, "{}", roff_escape(line)).unwrap();
    }
}

fn render_options_section(out: &mut String, command: &Command, conflict_map: &ConflictMap) {
    out.push_str(".SH OPTIONS\n");
    for arg in command.get_arguments() {
        if arg.is_hide_set() {
            continue;
        }
        render_option_entry(out, command, arg, conflict_map);
    }
}

fn render_option_entry(out: &mut String, command: &Command, arg: &Arg, conflict_map: &ConflictMap) {
    out.push_str(".TP\n");
    if arg.is_positional() {
        render_option_header_positional(out, arg);
    } else {
        render_option_header_flag(out, arg);
    }
    let help = arg
        .get_long_help()
        .or_else(|| arg.get_help())
        .map(ToString::to_string)
        .unwrap_or_default();
    writeln!(out, "{}", roff_escape(&help)).unwrap();
    render_possible_values(out, arg);
    render_conflicts(out, command, arg, conflict_map);
}

fn render_option_header_positional(out: &mut String, arg: &Arg) {
    let name = arg
        .get_value_names()
        .and_then(|names| names.first())
        .map(|name| name.as_str())
        .unwrap_or_else(|| arg.get_id().as_str());
    if arg.is_required_set() {
        writeln!(out, "\\fI{name}\\fR").unwrap();
    } else {
        writeln!(out, "[\\fI{name}\\fR]").unwrap();
    }
}

fn render_option_header_flag(out: &mut String, arg: &Arg) {
    let mut parts = Vec::new();
    if let Some(short) = arg.get_short() {
        parts.push(format!("\\fB\\-{}\\fR", roff_escape(&short.to_string())));
    }
    if let Some(long) = arg.get_long() {
        parts.push(format!("\\fB\\-\\-{}\\fR", roff_escape(long)));
    }
    for alias in arg.get_visible_aliases().unwrap_or_default() {
        parts.push(format!("\\fB\\-\\-{}\\fR", roff_escape(alias)));
    }
    let header = parts.join(", ");
    if arg.get_action().takes_values() {
        let value_str = render_value_hint(arg);
        writeln!(out, "{header} {value_str}").unwrap();
    } else {
        writeln!(out, "{header}").unwrap();
    }
}

fn render_value_hint(arg: &Arg) -> String {
    let mut parts = Vec::new();
    if let Some(value_names) = arg.get_value_names() {
        for name in value_names {
            parts.push(format!("\\fI<{}>\\fR", roff_escape(name)));
        }
    } else {
        parts.push(format!("\\fI<{}>\\fR", roff_escape(arg.get_id().as_str())));
    }
    let value_part = parts.join("\\fI \\fR");
    let defaults: Vec<_> = arg
        .get_default_values()
        .iter()
        .map(|value| value.to_string_lossy())
        .map(Cow::into_owned)
        .collect();
    if defaults.is_empty()
        || arg.is_hide_default_value_set()
        || matches!(arg.get_action(), ArgAction::SetTrue)
    {
        value_part
    } else {
        format!("{value_part} [default: {}]", defaults.join(", "))
    }
}

fn render_possible_values(out: &mut String, arg: &Arg) {
    if arg.is_hide_possible_values_set() {
        return;
    }
    if matches!(
        arg.get_action(),
        ArgAction::SetTrue | ArgAction::SetFalse | ArgAction::Count
    ) {
        return;
    }
    let possible_values: Vec<_> = arg
        .get_possible_values()
        .into_iter()
        .filter(|value| !value.is_hide_set())
        .collect();
    if possible_values.is_empty() {
        return;
    }
    let flag = arg
        .get_long()
        .map(|long| format!("\\-\\-{}", roff_escape(long)))
        .unwrap_or_default();
    out.push_str(".RS\n");
    for value in &possible_values {
        let name = value.get_name();
        if let Some(help) = value.get_help() {
            writeln!(
                out,
                ".TP\n\\fB{flag} {}\\fR\n{}",
                roff_escape(name),
                roff_escape(&help.to_string()),
            )
            .unwrap();
        } else {
            writeln!(out, ".TP\n\\fB{flag} {}\\fR", roff_escape(name)).unwrap();
        }
    }
    out.push_str(".RE\n");
}

fn render_conflicts(out: &mut String, command: &Command, arg: &Arg, conflict_map: &ConflictMap) {
    let arg_id = arg.get_id().as_str();
    let conflict_ids = match conflict_map.get(arg_id) {
        Some(ids) if !ids.is_empty() => ids,
        _ => return,
    };
    let conflict_names: Vec<_> = conflict_ids
        .iter()
        .filter_map(|conflict_id| resolve_flag_name(command, conflict_id))
        .collect();
    if conflict_names.is_empty() {
        return;
    }
    writeln!(
        out,
        ".RS\n.PP\nCannot be used with {}.\n.RE",
        conflict_names.join(", ")
    )
    .unwrap();
}

fn render_examples_section(out: &mut String, command: &Command) {
    let text = match command.get_after_long_help() {
        Some(text) => text.to_string(),
        None => return,
    };
    let mut lines = text.lines();
    let mut has_examples = false;
    for line in lines.by_ref() {
        if line.trim() == "Examples:" {
            has_examples = true;
            break;
        }
    }
    if !has_examples {
        return;
    }
    out.push_str(".SH EXAMPLES\n");
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(example_command) = line.strip_prefix("$ ") {
            writeln!(out, ".nf\n\\fB$ {}\\fR\n.fi", roff_escape(example_command)).unwrap();
        } else {
            writeln!(out, ".TP\n{}", roff_escape(line)).unwrap();
        }
    }
}

fn render_version_section(out: &mut String, command: &Command) {
    if let Some(version) = command.get_version() {
        writeln!(out, ".SH VERSION\nv{version}").unwrap();
    }
}

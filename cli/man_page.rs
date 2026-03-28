use clap::{Parser, ValueEnum};
use parallel_disk_usage::man_page::render_man_page;
use std::{
    fs,
    process::{Command, ExitCode},
};

const LINE_LENGTH: &str = "120";

/// Manage generated man pages.
#[derive(Debug, Parser)]
struct Args {
    /// Action to take.
    #[clap(value_enum)]
    action: Action,
    /// Type of file to target.
    #[clap(value_enum)]
    kind: Kind,
    /// Number of the man page.
    #[clap(value_enum)]
    page: Page,
}

#[derive(Debug, Clone, ValueEnum)]
enum Action {
    /// Check whether the man page is up-to-date.
    Check,
    /// Generate the man page.
    Generate,
}

#[derive(Debug, Clone, ValueEnum)]
enum Kind {
    /// Check or generate the roff file (`pdu.N`) from `Args`.
    Roff,
    /// Check or generate the man file (`pdu.N.man`) from the generated roff file (`pdu.N`).
    Man,
}

#[derive(Debug, Clone, ValueEnum)]
enum Page {
    #[clap(name = "1")]
    One,
}

impl Page {
    fn number(&self) -> u8 {
        match self {
            Page::One => 1,
        }
    }
}

fn roff_path(page_num: u8) -> String {
    format!("exports/pdu.{page_num}")
}

fn man_path(page_num: u8) -> String {
    format!("exports/pdu.{page_num}.man")
}

fn render_man_output(page_num: u8) -> Result<String, String> {
    let roff_file = roff_path(page_num);
    let output = Command::new("groff")
        .args(["-man", "-Tutf8", "-P-cbou"])
        .arg(format!("-rLL={LINE_LENGTH}n"))
        .arg(format!("./{roff_file}"))
        .output()
        .map_err(|error| format!("failed to run groff: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("groff failed: {stderr}"));
    }
    let content = String::from_utf8(output.stdout)
        .map_err(|error| format!("groff output is not UTF-8: {error}"))?;
    Ok(normalize_text(&strip_formatting(&content)))
}

/// Strips terminal formatting from grotty output.
///
/// Handles two styles grotty may use:
/// - **SGR mode** (default): ANSI escape sequences like `\x1b[1m` (bold), `\x1b[0m` (reset).
/// - **Legacy mode** (`-c`): Backspace overstrikes like `X\x08X` (bold), `_\x08X` (underline).
fn strip_formatting(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\x1b' && index + 1 < chars.len() && chars[index + 1] == '[' {
            // Skip ANSI escape: ESC [ ... m
            index += 2;
            while index < chars.len() && chars[index] != 'm' {
                index += 1;
            }
            if index < chars.len() {
                index += 1; // skip the 'm'
            }
        } else if index + 1 < chars.len() && chars[index + 1] == '\x08' {
            // Skip backspace overstrike: char + BS
            index += 2;
        } else {
            result.push(chars[index]);
            index += 1;
        }
    }
    result
}

/// Strips trailing whitespace per line, trims trailing blank lines,
/// and ensures the output ends with exactly one newline.
fn normalize_text(text: &str) -> String {
    let mut result: String = text
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    let trimmed_len = result.trim_end().len();
    result.truncate(trimmed_len);
    result.push('\n');
    result
}

fn write_file(path: &str, content: &str) -> ExitCode {
    match fs::write(path, content) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error writing {path}: {error}");
            ExitCode::FAILURE
        }
    }
}

fn check_file(path: &str, expected: &str) -> ExitCode {
    match fs::read_to_string(path) {
        Ok(actual) if actual == expected => ExitCode::SUCCESS,
        Ok(_) => {
            eprintln!("{path} is outdated, run ./generate-completions.sh to update it");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("error reading {path}: {error}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let args = Args::parse();
    let page_num = args.page.number();
    match (args.action, args.kind) {
        (Action::Generate, Kind::Roff) => write_file(&roff_path(page_num), &render_man_page()),
        (Action::Generate, Kind::Man) => match render_man_output(page_num) {
            Ok(content) => write_file(&man_path(page_num), &content),
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        },
        (Action::Check, Kind::Roff) => check_file(&roff_path(page_num), &render_man_page()),
        (Action::Check, Kind::Man) => match render_man_output(page_num) {
            Ok(expected) => check_file(&man_path(page_num), &expected),
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        },
    }
}

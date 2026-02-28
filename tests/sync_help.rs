//! The following tests check whether the help text files are outdated.
//!
//! If the tests fail, run `./generate-completions.sh` on the root of the repo to update the help files.

// Since the CLI in Windows look a little different, and I am way too lazy to make two versions
// of help files, the following tests would only run in UNIX-like environment.
#![cfg(unix)]
#![cfg(feature = "cli")]

use clap::CommandFactory;
use parallel_disk_usage::{args::Args, usage_md::render_usage_md};
use pipe_trait::Pipe;

fn normalize_help(text: &str) -> String {
    text.lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn long_help_is_up_to_date() {
    let actual = Args::command()
        .render_long_help()
        .to_string()
        .pipe_as_ref(normalize_help);
    let expected = include_str!("../exports/long.help");
    assert!(
        actual.trim_end() == expected.trim_end(),
        "help text is outdated, run ./generate-completions.sh to update it",
    );
}

#[test]
fn short_help_is_up_to_date() {
    let actual = Args::command()
        .render_help()
        .to_string()
        .pipe_as_ref(normalize_help);
    let expected = include_str!("../exports/short.help");
    assert!(
        actual.trim_end() == expected.trim_end(),
        "help text is outdated, run ./generate-completions.sh to update it",
    );
}

#[test]
fn usage_md_is_up_to_date() {
    let actual = render_usage_md();
    let expected = include_str!("../USAGE.md");
    assert!(
        actual.trim_end() == expected.trim_end(),
        "USAGE.md is outdated, run ./generate-completions.sh to update it",
    );
}

#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use pipe_trait::Pipe;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

fn exports_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("exports")
}

fn pdu_help(long: bool) -> String {
    let flag = if long { "--help" } else { "-h" };
    Command::new(PDU)
        .with_arg(flag)
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::null())
        .output()
        .unwrap_or_else(|error| panic!("failed to spawn pdu {flag}: {error}"))
        .pipe(stdout_text)
}

#[test]
fn long_help_is_up_to_date() {
    let actual = pdu_help(true);
    let expected = fs::read_to_string(exports_dir().join("long.help"))
        .expect("read exports/long.help")
        .trim_end()
        .to_string();
    assert_eq!(actual, expected);
}

#[test]
fn short_help_is_up_to_date() {
    let actual = pdu_help(false);
    let expected = fs::read_to_string(exports_dir().join("short.help"))
        .expect("read exports/short.help")
        .trim_end()
        .to_string();
    assert_eq!(actual, expected);
}

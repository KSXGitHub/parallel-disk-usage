//! The following tests check whether the help text files are outdated.
//!
//! If the tests fail, run `./generate-completions.sh` on the root of the repo to update the help files.

// Since the CLI in Windows look a little different, and I am way too lazy to make two versions
// of help files, the following tests would only run in UNIX-like environment.
#![cfg(unix)]
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use pipe_trait::Pipe;
use std::process::{Command, Stdio};

macro_rules! check {
    ($name:ident: $flag:literal => $path:literal) => {
        #[test]
        fn $name() {
            let actual = Command::new(PDU)
                .with_arg($flag)
                .with_stdin(Stdio::null())
                .with_stdout(Stdio::piped())
                .with_stderr(Stdio::null())
                .output()
                .expect("get actual help text")
                .pipe(stdout_text);
            let expected = include_str!($path);
            assert!(
                actual == expected.trim_end(),
                "help text is outdated, run ./generate-completions.sh to update it",
            );
        }
    };
}

check!(long_help_is_up_to_date: "--help" => "../exports/long.help");
check!(short_help_is_up_to_date: "-h" => "../exports/short.help");

#[test]
fn usage_md_is_up_to_date() {
    let actual = Command::new(PDU_USAGE_MD)
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::null())
        .output()
        .expect("get actual help text")
        .pipe(stdout_text);
    let expected = include_str!("../USAGE.md");
    assert!(
        actual == expected.trim_end(),
        "USAGE.md is outdated, run ./generate-completions.sh to update it",
    );
}

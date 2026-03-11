#![cfg(unix)] // This feature is not available in Windows
#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::process::{Command, Stdio};

fn stdio(command: Command) -> Command {
    command
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
}

const EXPECTED_ERROR: &str =
    "[error] DeduplicateHardlinkMultipleArgs: --deduplicate-hardlinks cannot be used with multiple arguments";

#[test]
fn two_args() {
    let sizes = [200_000, 220_000, 310_000, 110_000, 210_000];
    let workspace = SampleWorkspace::simple_tree_with_some_hardlinks(sizes);

    let output = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("main/sources")
        .with_arg("main/internal-hardlinks")
        .pipe(stdio)
        .output()
        .expect("spawn command");

    let stderr = String::from_utf8(output.stderr).expect("parse stderr as UTF-8");
    let stderr = stderr.trim_end();
    assert!(!output.status.success());
    assert_eq!(stderr, EXPECTED_ERROR);
    assert_eq!(&output.stdout, &[] as &[u8]);
}

#[test]
fn three_args() {
    let links = 10;
    let workspace = SampleWorkspace::multiple_hardlinks_to_a_single_file(100_000, links);

    let output = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--deduplicate-hardlinks")
        .with_arg("file.txt")
        .with_arg("link.3")
        .with_arg("link.5")
        .pipe(stdio)
        .output()
        .expect("spawn command");

    let stderr = String::from_utf8(output.stderr).expect("parse stderr as UTF-8");
    let stderr = stderr.trim_end();
    assert!(!output.status.success());
    assert_eq!(stderr, EXPECTED_ERROR);
    assert_eq!(&output.stdout, &[] as &[u8]);
}

#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::process::{Command, Output, Stdio};

fn stdio(command: Command) -> Command {
    command
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
}

#[test]
fn min_ratio_1() {
    let workspace = SampleWorkspace::default();
    let Output {
        status,
        stdout,
        stderr,
    } = Command::new(PDU)
        .with_current_dir(workspace.as_path())
        .with_arg("--min-ratio=1")
        .pipe(stdio)
        .output()
        .expect("spawn command");
    let stderr = String::from_utf8(stderr).expect("parse stderr as UTF-8");
    let stderr = stderr.trim();
    dbg!(&status);
    eprintln!("STDERR:\n{}\n", stderr);
    assert!(!status.success());
    assert_eq!(
        stderr,
        "error: Invalid value for '--min-ratio <min-ratio>': greater than or equal to 1"
    );
    assert_eq!(&stdout, &[] as &[u8]);
}

#[test]
fn max_depth_0() {
    let workspace = SampleWorkspace::default();
    let Output {
        status,
        stdout,
        stderr,
    } = Command::new(PDU)
        .with_current_dir(workspace.as_path())
        .with_arg("--max-depth=0")
        .pipe(stdio)
        .output()
        .expect("spawn command");
    let stderr = String::from_utf8(stderr).expect("parse stderr as UTF-8");
    let stderr = stderr.trim();
    dbg!(&status);
    eprintln!("STDERR:\n{}\n", stderr);
    assert!(!status.success());
    assert_eq!(
        stderr,
        "error: Invalid value for '--max-depth <max-depth>': number would be zero for non-zero type"
    );
    assert_eq!(&stdout, &[] as &[u8]);
}

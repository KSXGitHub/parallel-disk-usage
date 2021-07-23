#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use std::process::Stdio;

/// There are branches of similar shapes in `/src/app.rs` that call
/// the `sub!` macro. This test suite is to ensure that no combination
/// of variant is left out by programmer's mistake.
#[test]
fn flag_combinations() {
    #[cfg(unix)]
    let quantity = ["len", "blksize", "blocks"];
    #[cfg(windows)]
    let quantity = ["len"];

    let list = CommandList::default()
        .option_matrix("--quantity", quantity)
        .flag_matrix("--progress");

    for command in list.commands() {
        dbg!(&command);
        let workspace = SampleWorkspace::default();
        dbg!(workspace.as_path());
        let output = command
            .with_current_dir(workspace.as_path())
            .with_stdin(Stdio::null())
            .with_stdout(Stdio::null())
            .with_stderr(Stdio::piped())
            .output()
            .expect("execute command");
        inspect_stderr(&output.stderr);
        let status = output.status;
        assert!(status.success(), "status: {:?}", status.code())
    }
}

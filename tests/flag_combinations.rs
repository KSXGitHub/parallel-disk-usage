pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use pipe_trait::Pipe;
use std::process::Stdio;

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
        let error = output.stderr.pipe_as_ref(String::from_utf8_lossy);
        let error = error.trim();
        if !error.is_empty() {
            eprintln!("STDERR:\n{}\n", error);
        }
        let status = output.status;
        assert!(status.success(), "status: {:?}", status.code())
    }
}

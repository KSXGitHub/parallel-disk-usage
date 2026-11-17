//! Inspection of the output of a finished child process.

use pipe_trait::Pipe;
use std::process::Output;

/// Make sure that status code is 0, print stderr if it's not empty,
/// and turn stdin into a string.
pub fn stdout_text(
    Output {
        status,
        stdout,
        stderr,
    }: Output,
) -> String {
    inspect_stderr(&stderr);
    assert!(
        status.success(),
        "progress exits with non-zero status: {status:?}",
    );
    stdout
        .pipe(String::from_utf8)
        .expect("parse stdout as UTF-8")
        .trim_end()
        .to_string()
}

/// Print stderr if it's not empty.
pub fn inspect_stderr(stderr: &[u8]) {
    let text = String::from_utf8_lossy(stderr);
    let text = text.trim();
    if !text.is_empty() {
        eprintln!("STDERR:\n{text}\n");
    }
}

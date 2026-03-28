//! The following tests check whether the man page files are outdated.
//!
//! If the tests fail, run `./generate-completions.sh` on the root of the repo to update the man page.

// Since the CLI in Windows looks a little different, and I am way too lazy to make two versions
// of man page files, the following tests would only run in UNIX-like environment.
#![cfg(unix)]
#![cfg(feature = "cli")]

use command_extra::CommandExtra;
use std::process::Command;

const PDU_MAN_PAGE: &str = env!("CARGO_BIN_EXE_pdu-man-page");

fn check(kind: &str, page: &str) {
    let output = Command::new(PDU_MAN_PAGE)
        .with_args(["check", kind, page])
        .with_current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("spawn pdu-man-page");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout.trim();
    if !stdout.is_empty() {
        eprintln!("STDOUT:\n{stdout}\n");
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        eprintln!("STDERR:\n{stderr}\n");
    }
    assert!(
        output.status.success(),
        "man page is outdated, run ./generate-completions.sh to update it",
    );
}

#[test]
fn roff() {
    check("roff", "1");
}

#[test]
#[cfg_attr(
    not(target_os = "linux"),
    ignore = "groff is only installed on Linux CI"
)]
fn man() {
    if which::which("groff").is_err() {
        panic!(
            "{}\n{}",
            "error: This test requires `groff` but it was not found.",
            "hint: Install `groff` (or `groff-base`) for your platform, \
             or rerun via `TEST_SKIP='man' ./test.sh` to skip this test.",
        );
    }
    check("man", "1");
}

//! The following tests check whether the help text files are outdated.
//!
//! If the tests fail, run `./generate-completions.sh` on the root of the repo to update the help files.

// Since the CLI in Windows look a little different, and I am way too lazy to make two versions
// of help files, the following tests would only run in UNIX-like environment.
#![cfg(unix)]
#![cfg(feature = "cli")]

use parallel_disk_usage::usage_md::render_usage_md;

#[test]
fn usage_md() {
    let actual = render_usage_md();
    let expected = include_str!("../USAGE.md");
    assert!(
        actual.trim_end() == expected.trim_end(),
        "USAGE.md is outdated, run ./generate-completions.sh to update it",
    );
}

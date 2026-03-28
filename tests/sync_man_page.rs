//! The following test checks whether the man page file is outdated.
//!
//! If the test fails, run `./generate-completions.sh` on the root of the repo to update the man page.

// Since the CLI in Windows looks a little different, and I am way too lazy to make two versions
// of man page files, the following test would only run in UNIX-like environment.
#![cfg(unix)]
#![cfg(feature = "cli-man")]

use parallel_disk_usage::man_page::render_man_page;

#[test]
fn man_page() {
    let received = render_man_page().expect("render man page");
    let expected = include_str!("../exports/pdu.1");
    assert!(
        received == expected,
        "man page is outdated, run ./generate-completions.sh to update it",
    );
}

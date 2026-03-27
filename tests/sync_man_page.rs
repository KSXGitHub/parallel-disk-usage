//! The following test checks whether the man page file is outdated.
//!
//! If the test fails, run `./generate-completions.sh` on the root of the repo to update the man page.

// Since the CLI in Windows looks a little different, and I am way too lazy to make two versions
// of man page files, the following test would only run in UNIX-like environment.
#![cfg(unix)]
#![cfg(feature = "cli")]

use clap::CommandFactory;
use clap_mangen::Man;
use parallel_disk_usage::args::Args;

#[test]
fn man_page() {
    let command = Args::command();
    let man = Man::new(command);
    let mut buffer = Vec::new();
    man.render(&mut buffer).expect("render man page to buffer");
    let received = String::from_utf8(buffer).expect("man page should be valid UTF-8");
    let expected = include_str!("../exports/pdu.1");
    assert!(
        received == expected,
        "man page is outdated, run ./generate-completions.sh to update it",
    );
}

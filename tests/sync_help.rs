//! The following tests check whether the help text files are outdated.
//!
//! If the tests fail, run `./generate-completions.sh` on the root of the repo to update the help files.

// Since the CLI in Windows look a little different, and I am way too lazy to make two versions
// of help files, the following tests would only run in UNIX-like environment.
#![cfg(unix)]
#![cfg(feature = "cli")]

use clap::CommandFactory;
use itertools::Itertools;
use parallel_disk_usage::args::Args;

macro_rules! check {
    ($name:ident: $render_help:ident => $path:literal) => {
        #[test]
        fn $name() {
            eprintln!(
                "check!({name}: {method} => {path});",
                name = stringify!($name),
                method = stringify!($render_help),
                path = $path,
            );
            let received = Args::command()
                .$render_help()
                .to_string()
                .lines()
                .map(str::trim_end)
                .join("\n");
            let expected = include_str!($path);
            assert!(
                received.trim_end() == expected.trim_end(),
                "help text is outdated, run ./generate-completions.sh to update it",
            );
        }
    };
}

check!(long: render_long_help => "../exports/long.help");
check!(short: render_help => "../exports/short.help");

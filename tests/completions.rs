//! The following tests check whether the shell completion files are outdated.
//!
//! If the tests fail, run `./generate-completions.sh` on the root of the repo to update the completion files.

// Since the CLI in Windows looks a little different, and I am way too lazy to make two versions
// of completion files, the following tests would only run in UNIX-like environment.
#![cfg(unix)]
#![cfg(feature = "cli")]

use clap_complete::Shell;
use clap_utilities::CommandFactoryExtra;
use parallel_disk_usage::args::Args;

macro_rules! check {
    ($name:ident: $shell:ident => $path:literal) => {
        #[test]
        fn $name() {
            eprintln!(
                "check!({name}: {shell} => {path});",
                name = stringify!($name),
                shell = stringify!($shell),
                path = $path,
            );
            let received =
                Args::get_completion_string("pdu", Shell::$shell).expect("get completion string");
            let expected = include_str!($path);
            assert!(
                received == expected,
                "completion is outdated, run ./generate-completions.sh to update them",
            );
        }
    };
}

check!(bash: Bash => "../exports/completion.bash");
check!(fish: Fish => "../exports/completion.fish");
check!(zsh: Zsh => "../exports/completion.zsh");
check!(powershell: PowerShell => "../exports/completion.ps1");
check!(elvish: Elvish => "../exports/completion.elv");

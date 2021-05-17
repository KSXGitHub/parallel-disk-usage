use parallel_disk_usage::{args::Args, clap::Shell, structopt::StructOpt};

macro_rules! test_case {
    ($name:ident, $variant:ident, $path:literal) => {
        #[test]
        fn $name() {
            let actual = include_str!($path).as_bytes();
            let mut expected = Vec::new();
            Args::clap().gen_completions_to("pdu", Shell::$variant, &mut expected);
            if actual != expected {
                panic!(concat!(
                    stringify!($variant),
                    " completion is outdated. Re-run generate-completions.sh to update",
                ));
            }
        }
    };
}

test_case!(bash, Bash, "../exports/completion.bash");
test_case!(fish, Fish, "../exports/completion.fish");
test_case!(zsh, Zsh, "../exports/completion.zsh");
test_case!(powershell, PowerShell, "../exports/completion.ps1");
test_case!(elvish, Elvish, "../exports/completion.elv");

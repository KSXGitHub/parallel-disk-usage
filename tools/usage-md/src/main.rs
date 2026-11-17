//! Render the content of `USAGE.md` from the `pdu` command-line interface.
//!
//! `generate-completions.sh` runs this tool and writes the result to
//! `USAGE.md`.

use parallel_disk_usage::usage_md::render_usage_md;
use std::process::ExitCode;

fn main() -> ExitCode {
    match render_usage_md() {
        Ok(content) => {
            println!("{}", content.trim_end());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

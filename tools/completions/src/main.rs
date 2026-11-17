//! Render the shell completion scripts of the `pdu` command.
//!
//! `generate-completions.sh` runs this tool once per shell and writes
//! each result to `exports/completion.*`.

use parallel_disk_usage::args::Args;
use parallel_disk_usage::clap_utilities::CommandFactoryExtra;
use std::process::ExitCode;

fn main() -> ExitCode {
    Args::run_completion_generator()
}

use clap_utilities::CommandFactoryExtra;
use parallel_disk_usage::args::Args;
use std::process::ExitCode;

fn main() -> ExitCode {
    Args::run_completion_generator()
}

use parallel_disk_usage::args::Args;
use parallel_disk_usage::clap_utilities::CommandFactoryExtra;
use std::process::ExitCode;

fn main() -> ExitCode {
    Args::run_completion_generator()
}

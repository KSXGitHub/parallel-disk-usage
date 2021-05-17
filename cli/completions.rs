use parallel_disk_usage::args::Args;
use structopt_utilities::StructOptUtils;

fn main() {
    Args::run_completion_generator("pdu-completions", "pdu");
}

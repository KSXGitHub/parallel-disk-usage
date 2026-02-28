use clap::CommandFactory;
use parallel_disk_usage::{args::Args, usage_md::render};

fn main() {
    println!("{}", render(Args::command()).trim_end());
}

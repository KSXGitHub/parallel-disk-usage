use clap::CommandFactory;
use parallel_disk_usage::{args::Args, usage_md::render};

fn main() {
    let help = Args::command().render_long_help().to_string();
    println!("{}", render(&help).trim_end());
}

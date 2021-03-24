pub mod args;

pub use args::Args;

use structopt_utilities::StructOptUtils;

pub const LICENSE: &str = include_str!("LICENSE");

pub fn main() {
    let Args { license, files } = Args::strict_from_args();
    if license {
        println!("{}", LICENSE);
        return;
    }
    dbg!(files);
}

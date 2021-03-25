pub mod args;

pub use args::Args;

use structopt_utilities::StructOptUtils;

pub fn main() {
    let Args { copyright, files } = Args::strict_from_args();
    if copyright {
        println!("Apache-2.0 © 2021 Hoàng Văn Khải");
        return;
    }
    dbg!(files);
}

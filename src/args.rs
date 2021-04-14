use std::path::PathBuf;
use structopt::StructOpt;
use text_block_macros::text_block;

/// The CLI arguments.
#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "dirt",

    long_about = text_block! {
        "Summarize disk usage of the set of files, recursively for directories."
        ""
        "Copyright: Apache-2.0 © 2021 Hoàng Văn Khải <https://ksxgithub.github.io/>"
        "Donation: https://patreon.com/khai96_"
    }
)]
pub struct Args {
    /// List of files and/or directories.
    #[structopt(name = "files", about = "List of files and/or directories.")]
    pub files: Vec<PathBuf>,
}

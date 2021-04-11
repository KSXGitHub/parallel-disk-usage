use std::path::PathBuf;
use structopt::StructOpt;

/// The CLI arguments.
#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "dirt",
    about = "Summarize disk usage of the set of files, recursively for directories."
)]
pub struct Args {
    /// Whether to print author information.
    #[structopt(long, about = "Print author information to stdout and exit.")]
    pub copyright: bool,

    /// List of files and/or directories.
    #[structopt(name = "files", about = "List of files and/or directories.")]
    pub files: Vec<PathBuf>,
}

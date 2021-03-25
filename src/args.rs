use std::path::PathBuf;
use structopt::StructOpt;

/// Summarize disk usage of the set of files, recursively for directories
#[derive(Debug, Clone, StructOpt)]
#[structopt(name = "dirt")]
pub struct Args {
    /// Print license to stdout and exit
    #[structopt(long)]
    pub copyright: bool,

    /// List of files and/or directories
    #[structopt(name = "files")]
    pub files: Vec<PathBuf>,
}

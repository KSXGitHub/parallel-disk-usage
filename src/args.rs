use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "dirt",
    about = "Summarize disk usage of the set of files, recursively for directories."
)]
pub struct Args {
    #[structopt(long, about = "Print author information to stdout and exit.")]
    pub copyright: bool,

    #[structopt(name = "files", about = "List of files and/or directories.")]
    pub files: Vec<PathBuf>,
}

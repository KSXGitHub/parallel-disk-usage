use clap::CommandFactory;
use clap_mangen::Man;
use parallel_disk_usage::args::Args;
use std::{io::stdout, process::ExitCode};

fn main() -> ExitCode {
    let command = Args::command();
    let man = Man::new(command);
    match man.render(&mut stdout()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

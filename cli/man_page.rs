use parallel_disk_usage::man_page::render_man_page;
use std::process::ExitCode;

fn main() -> ExitCode {
    match render_man_page() {
        Ok(content) => {
            print!("{content}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

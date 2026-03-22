use parallel_disk_usage::usage_md::render_usage_md;
use std::process::ExitCode;

fn main() -> ExitCode {
    match render_usage_md() {
        Ok(content) => {
            println!("{}", content.trim_end());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

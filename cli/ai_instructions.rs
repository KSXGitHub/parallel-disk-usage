use parallel_disk_usage::ai_instructions::{generate, AiInstructionFile};
use std::{fs, process::ExitCode};

fn main() -> ExitCode {
    let should_generate = std::env::args().any(|arg| arg == "--generate");
    let files = generate();

    if should_generate {
        write_files(&files)
    } else {
        check_files(&files)
    }
}

fn write_files(files: &[AiInstructionFile]) -> ExitCode {
    for file in files {
        if let Some(parent) = std::path::Path::new(file.path).parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                eprintln!(
                    "error: failed to create directory for {}: {error}",
                    file.path
                );
                return ExitCode::FAILURE;
            }
        }
        if let Err(error) = fs::write(file.path, &file.content) {
            eprintln!("error: failed to write {}: {error}", file.path);
            return ExitCode::FAILURE;
        }
        eprintln!("wrote {}", file.path);
    }
    ExitCode::SUCCESS
}

fn check_files(files: &[AiInstructionFile]) -> ExitCode {
    let mut all_ok = true;
    for file in files {
        match fs::read_to_string(file.path) {
            Ok(ref actual) if actual == &file.content => {
                eprintln!("ok: {}", file.path);
            }
            Ok(_) => {
                eprintln!("outdated: {}", file.path);
                all_ok = false;
            }
            Err(error) => {
                eprintln!("error: failed to read {}: {error}", file.path);
                all_ok = false;
            }
        }
    }
    if !all_ok {
        eprintln!();
        eprintln!("Run `./run.sh pdu-ai-instructions --generate` to update.");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

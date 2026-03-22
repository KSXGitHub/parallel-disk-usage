use clap::Parser;
use std::{fs, path::Path, process::ExitCode};

/// Content shared across all AI instruction files.
const SHARED: &str = include_str!("../template/ai-instructions/shared.md");

/// Content specific to Claude (`CLAUDE.md`).
const CLAUDE: &str = include_str!("../template/ai-instructions/claude.md");

/// Content specific to GitHub Copilot (`.github/copilot-instructions.md`).
const COPILOT: &str = include_str!("../template/ai-instructions/copilot.md");

/// Content specific to agents (`AGENTS.md`).
const AGENTS: &str = include_str!("../template/ai-instructions/agents.md");

/// A generated AI instruction file.
struct AiInstructionFile {
    /// Path relative to the repository root.
    path: &'static str,
    /// Generated content.
    content: String,
}

/// Generate all AI instruction files.
fn generate() -> Vec<AiInstructionFile> {
    vec![
        AiInstructionFile {
            path: "CLAUDE.md",
            content: format!("{SHARED}{CLAUDE}"),
        },
        AiInstructionFile {
            path: ".github/copilot-instructions.md",
            content: format!("{SHARED}{COPILOT}"),
        },
        AiInstructionFile {
            path: "AGENTS.md",
            content: format!("{SHARED}{AGENTS}"),
        },
    ]
}

/// Check or generate AI instruction files from templates.
#[derive(Debug, Parser)]
struct Args {
    /// Generate the AI instruction files instead of checking them.
    #[clap(long)]
    generate: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let files = generate();

    if args.generate {
        write_files(&files)
    } else {
        check_files(&files)
    }
}

fn write_files(files: &[AiInstructionFile]) -> ExitCode {
    for file in files {
        if let Some(parent) = Path::new(file.path).parent() {
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

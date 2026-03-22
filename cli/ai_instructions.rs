use clap::Parser;
use derive_more::Display;
use std::{fs, path::Path, process::ExitCode};

const SHARED: &str = include_str!("../template/ai-instructions/shared.md");
const CLAUDE: &str = include_str!("../template/ai-instructions/claude.md");
const COPILOT: &str = include_str!("../template/ai-instructions/copilot.md");
const AGENTS: &str = include_str!("../template/ai-instructions/agents.md");

struct AiInstructionFile {
    path: &'static str,
    fragments: &'static [&'static str],
}

const FILES: [AiInstructionFile; 3] = [
    AiInstructionFile {
        path: "CLAUDE.md",
        fragments: &[SHARED, CLAUDE],
    },
    AiInstructionFile {
        path: ".github/copilot-instructions.md",
        fragments: &[SHARED, COPILOT],
    },
    AiInstructionFile {
        path: "AGENTS.md",
        fragments: &[SHARED, AGENTS],
    },
];

impl AiInstructionFile {
    fn content(&self) -> String {
        self.fragments.concat()
    }
}

#[derive(Debug, Display)]
enum RuntimeError {
    #[display("failed to create directory for {path}: {error}")]
    CreateDir {
        path: &'static str,
        error: std::io::Error,
    },
    #[display("failed to write {path}: {error}")]
    WriteFile {
        path: &'static str,
        error: std::io::Error,
    },
    #[display("failed to read {path}: {error}")]
    ReadFile {
        path: &'static str,
        error: std::io::Error,
    },
    #[display("{}", display_outdated(outdated))]
    Outdated { outdated: Vec<&'static str> },
}

fn display_outdated(outdated: &[&str]) -> String {
    let mut message = String::from("outdated files:");
    for path in outdated {
        message.push_str(&format!("\n  {path}"));
    }
    message.push_str("\n\nRun `./run.sh pdu-ai-instructions --generate` to update.");
    message
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
    let result = if args.generate {
        write_files()
    } else {
        check_files()
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[error] {error}");
            ExitCode::FAILURE
        }
    }
}

fn write_files() -> Result<(), RuntimeError> {
    for file in &FILES {
        if let Some(parent) = Path::new(file.path).parent() {
            fs::create_dir_all(parent).map_err(|error| RuntimeError::CreateDir {
                path: file.path,
                error,
            })?;
        }
        fs::write(file.path, file.content()).map_err(|error| RuntimeError::WriteFile {
            path: file.path,
            error,
        })?;
        eprintln!("wrote {}", file.path);
    }
    Ok(())
}

fn check_files() -> Result<(), RuntimeError> {
    let mut outdated = Vec::new();
    for file in &FILES {
        let actual = fs::read_to_string(file.path).map_err(|error| RuntimeError::ReadFile {
            path: file.path,
            error,
        })?;
        if actual == file.content() {
            eprintln!("ok: {}", file.path);
        } else {
            eprintln!("outdated: {}", file.path);
            outdated.push(file.path);
        }
    }
    if outdated.is_empty() {
        Ok(())
    } else {
        Err(RuntimeError::Outdated { outdated })
    }
}

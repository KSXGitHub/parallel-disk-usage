use clap::Parser;
use derive_more::Display;
use std::{
    fmt, fs,
    io::{self, Write},
    path::Path,
    process::ExitCode,
};

const SHARED: &str = include_str!("../template/ai-instructions/shared.md");
const CLAUDE: &str = include_str!("../template/ai-instructions/claude.md");
const COPILOT: &str = include_str!("../template/ai-instructions/copilot.md");
const AGENTS: &str = include_str!("../template/ai-instructions/agents.md");

#[derive(Clone, Copy)]
struct Fragments(&'static [&'static str]);

impl fmt::Display for Fragments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Fragments(fragments) = self;
        for fragment in *fragments {
            f.write_str(fragment)?;
        }
        Ok(())
    }
}

impl Fragments {
    fn matches(&self, actual: &str) -> bool {
        let Fragments(fragments) = self;
        let mut remaining = actual;
        for fragment in *fragments {
            match remaining.strip_prefix(fragment) {
                Some(rest) => remaining = rest,
                None => return false,
            }
        }
        remaining.is_empty()
    }
}

const FILES: &[(&str, Fragments)] = &[
    ("CLAUDE.md", Fragments(&[SHARED, CLAUDE])),
    (
        ".github/copilot-instructions.md",
        Fragments(&[SHARED, COPILOT]),
    ),
    ("AGENTS.md", Fragments(&[SHARED, AGENTS])),
];

#[derive(Debug, Display)]
enum RuntimeError {
    #[display("failed to create directory for {path}: {error}")]
    CreateDir {
        path: &'static str,
        error: io::Error,
    },
    #[display("failed to write {path}: {error}")]
    WriteFile {
        path: &'static str,
        error: io::Error,
    },
    #[display("failed to read {path}: {error}")]
    ReadFile {
        path: &'static str,
        error: io::Error,
    },
    #[display("Run `./run.sh pdu-ai-instructions --generate` to update.")]
    Outdated,
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
    let result = match args.generate {
        true => write_files(),
        false => check_files(),
    };
    if let Err(error) = result {
        eprintln!("error: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn write_files() -> Result<(), RuntimeError> {
    for &(path, fragments) in FILES {
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).map_err(|error| RuntimeError::CreateDir { path, error })?;
        }
        let mut output =
            fs::File::create(path).map_err(|error| RuntimeError::WriteFile { path, error })?;
        write!(output, "{fragments}").map_err(|error| RuntimeError::WriteFile { path, error })?;
        eprintln!("info: Generated file {path}");
    }
    Ok(())
}

fn check_files() -> Result<(), RuntimeError> {
    let mut result: Result<(), RuntimeError> = Ok(());
    for &(path, fragments) in FILES {
        let actual =
            fs::read_to_string(path).map_err(|error| RuntimeError::ReadFile { path, error })?;
        if !fragments.matches(&actual) {
            eprintln!("error: File {path} is out-of-date");
            result = Err(RuntimeError::Outdated);
        }
    }
    result
}

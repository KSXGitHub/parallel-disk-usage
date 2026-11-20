use command_extra::CommandExtra;
use pipe_trait::Pipe;
use std::path::{Path, PathBuf};
use std::process::Command;

const PDU_AI_INSTRUCTIONS: &str = env!("CARGO_BIN_EXE_pdu-ai-instructions");

fn workspace_root() -> PathBuf {
    let output = env!("CARGO")
        .pipe(Command::new)
        .with_arg("locate-project")
        .with_arg("--workspace")
        .with_arg("--message-format=plain")
        .output()
        .expect("spawn cargo locate-project");
    assert!(
        output.status.success(),
        "cargo locate-project exits with non-zero status: {:?}",
        output.status.code(),
    );
    let manifest = output
        .stdout
        .pipe(String::from_utf8)
        .expect("parse stdout as UTF-8");
    manifest
        .trim()
        .pipe(Path::new)
        .parent()
        .expect("get the directory that holds the workspace manifest")
        .to_path_buf()
}

#[test]
fn ai_instructions_up_to_date() {
    let output = PDU_AI_INSTRUCTIONS
        .pipe(Command::new)
        .with_arg(workspace_root())
        .output()
        .expect("spawn pdu-ai-instructions");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout = stdout.trim();
    if !stdout.is_empty() {
        eprintln!("STDOUT:\n{stdout}\n");
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        eprintln!("STDERR:\n{stderr}\n");
    }
    assert!(
        output.status.success(),
        "AI instruction files are outdated. Run `./run.sh pdu-ai-instructions --generate .` to update.",
    );
}

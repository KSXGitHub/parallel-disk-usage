//! The following test checks whether the AI instruction files are outdated.
//!
//! If the test fails, run `./run.sh pdu-ai-instructions --generate .` on the
//! root of the repo to update the files.

use command_extra::CommandExtra;
use std::process::Command;

const PDU_AI_INSTRUCTIONS: &str = env!("CARGO_BIN_EXE_pdu-ai-instructions");

/// Path to the top-level directory of the repository, which holds both the
/// templates and the generated files, two levels above this crate.
const REPOSITORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

#[test]
fn ai_instructions_up_to_date() {
    let output = Command::new(PDU_AI_INSTRUCTIONS)
        .with_arg(REPOSITORY)
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

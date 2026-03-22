#![cfg(feature = "ai-instructions")]

use command_extra::CommandExtra;
use std::process::Command;

const PDU_AI_INSTRUCTIONS: &str = env!("CARGO_BIN_EXE_pdu-ai-instructions");

#[test]
fn ai_instructions_up_to_date() {
    let output = Command::new(PDU_AI_INSTRUCTIONS)
        .with_current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("spawn pdu-ai-instructions");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        eprintln!("STDERR:\n{stderr}\n");
    }
    assert!(
        output.status.success(),
        "AI instruction files are outdated. Run `./run.sh pdu-ai-instructions --generate` to update.",
    );
}

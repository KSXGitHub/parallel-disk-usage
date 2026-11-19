use command_extra::CommandExtra;
use std::path::Path;
use std::process::Command;

const PDU_AI_INSTRUCTIONS: &str = env!("CARGO_BIN_EXE_pdu-ai-instructions");

const CARGO_MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn ai_instructions_up_to_date() {
    let repository = Path::new(CARGO_MANIFEST_DIR)
        .ancestors()
        .nth(2)
        .expect("locate the repository root above tools/ai-instructions");
    let output = Command::new(PDU_AI_INSTRUCTIONS)
        .with_arg(repository)
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

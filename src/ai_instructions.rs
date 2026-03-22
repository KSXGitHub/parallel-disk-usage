//! AI instruction file generation.
//!
//! Each target AI instruction file is assembled from a shared template
//! plus an optional target-specific template fragment.

/// Content shared across all AI instruction files.
const SHARED: &str = include_str!("../template/ai-instructions/shared.md");

/// Content specific to Claude (`CLAUDE.md`).
const CLAUDE: &str = include_str!("../template/ai-instructions/claude.md");

/// Content specific to GitHub Copilot (`.github/copilot-instructions.md`).
const COPILOT: &str = include_str!("../template/ai-instructions/copilot.md");

/// Content specific to agents (`AGENTS.md`).
const AGENTS: &str = include_str!("../template/ai-instructions/agents.md");

/// A generated AI instruction file.
pub struct AiInstructionFile {
    /// Path relative to the repository root.
    pub path: &'static str,
    /// Generated content.
    pub content: String,
}

/// Generate all AI instruction files.
pub fn generate() -> Vec<AiInstructionFile> {
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

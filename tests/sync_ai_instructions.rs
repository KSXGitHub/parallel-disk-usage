//! The following tests check whether the AI instruction files are in sync.
//!
//! All three files (CLAUDE.md, AGENTS.md, .github/copilot-instructions.md) should be identical.

macro_rules! check {
    ($name:ident: $a_path:literal == $b_path:literal) => {
        #[test]
        fn $name() {
            let a = include_str!($a_path);
            let b = include_str!($b_path);
            assert!(
                a == b,
                concat!(
                    "AI instruction files are out of sync: ",
                    $a_path,
                    " != ",
                    $b_path,
                ),
            );
        }
    };
}

check!(claude_md_vs_agents_md: "../CLAUDE.md" == "../AGENTS.md");
check!(claude_md_vs_copilot_instructions: "../CLAUDE.md" == "../.github/copilot-instructions.md");

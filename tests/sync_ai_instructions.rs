//! The following tests check whether the AI instruction files are in sync.
//!
//! All three files (CLAUDE.md, AGENTS.md, .github/copilot-instructions.md) should have
//! identical content (except for the title line).

macro_rules! check {
    ($name:ident: $a_path:literal == $b_path:literal) => {
        #[test]
        fn $name() {
            let a = include_str!($a_path);
            let b = include_str!($b_path);
            let a_body = strip_title(a);
            let b_body = strip_title(b);
            assert!(
                a_body == b_body,
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

/// Strip the first line (title) and return the rest.
fn strip_title(text: &str) -> &str {
    text.split_once('\n').map_or("", |(_, rest)| rest)
}

check!(claude_md_vs_agents_md: "../CLAUDE.md" == "../AGENTS.md");
check!(claude_md_vs_copilot_instructions: "../CLAUDE.md" == "../.github/copilot-instructions.md");

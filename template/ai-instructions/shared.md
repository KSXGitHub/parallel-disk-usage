# AI Instructions

Read and follow the CONTRIBUTING.md file in this repository for all code style conventions, commit message format, and development guidelines.

## Quick Reference

- Commit format: Conventional Commits. Pattern: `type(scope): lowercase description`. The scope is optional.
- Version releases are the only exception. The commit message is just the version number, such as `0.21.1`.
- Write documentation, comments, and other prose for ease of understanding first. Prefer a formal tone when it does not hurt clarity, and use complete sentences. Avoid mid-sentence breaks introduced by em dashes or long parenthetical clauses. Em dashes are a reliable symptom of loose phrasing; when one appears, restructure the surrounding sentence so each clause stands on its own rather than swapping the em dash for another punctuation mark.
- Prefer merged imports.
- Use descriptive names for generics, such as `Size` and `Report`. Do not use single letters.
- Use descriptive names for variables and closure parameters. Single letters are permitted only in these cases: (1) conventional names like `n` for count or `f` for formatter; (2) comparison closures like `|a, b|`; (3) trivial single-expression closures; (4) fold accumulators; (5) index variables `i`/`j`/`k` in short closures or index-based loops; and (6) test fixtures with identical roles. Single letters are never permitted in multi-line functions or closures.
- Use `pipe-trait` to chain through unary functions such as constructors, `Some`, `Ok`, and free functions. Use it to flatten nested calls and to continue method chains. Do not use it for simple standalone calls; prefer `foo(value)` over `value.pipe(foo)`.
- Prefer `where` clauses when a type has multiple trait bounds.
- Derive order: standard traits, then comparison traits, then `Hash`, then `derive_more`, then feature-gated derives.
- For error types, only derive `Display` and `Error` from `derive_more` when each is actually needed. Not all displayable types are errors.
- Minimize `unwrap()` in non-test code. Use proper error handling instead.
- Prefer `#[cfg_attr(..., ignore = "reason")]` over `#[cfg(...)]` when skipping tests. Use `#[cfg]` on tests only when the code cannot compile under the condition, such as when it references types or functions that do not exist on other platforms.
- Install the toolchain before running tests: `rustup toolchain install "$(< rust-toolchain)" && rustup component add --toolchain "$(< rust-toolchain)" rustfmt clippy`.
- When you change CLI arguments, help text, or anything that affects command-line output, run `./generate-completions.sh` to regenerate the shell completion files, the help text files, and `USAGE.md`. **Do not attempt to regenerate these files manually.** Always use the script.
- Validate changes with `FMT=true LINT=true BUILD=true TEST=true DOC=true ./test.sh`. When a test fails with a hint about `TEST_SKIP`, follow the hint and rerun with the suggested variable. When a sync test fails, read its error message and run the exact command it reports.
- **Always run the full test suite** (`FMT=true LINT=true BUILD=true TEST=true DOC=true ./test.sh`) before every commit. This rule applies to all changes, including documentation changes, comment edits, config changes, and refactors. The test suite checks formatting, linting, building, tests, and docs across multiple feature combinations, and any kind of change can break any of these checks.
- When the user provides a diff to apply, run `git apply` rather than interpreting each hunk manually. When a diff is provided for context or discussion, respond accordingly.

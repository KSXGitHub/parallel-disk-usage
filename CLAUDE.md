# AI Instructions

Read and follow the CONTRIBUTING.md file in this repository for all code style conventions, commit message format, and development guidelines.

## Quick Reference

- Commit format: Conventional Commits — `type(scope): lowercase description`
- Version releases are the only exception: just the version number (e.g. `0.21.1`)
- Prefer merged imports
- Use descriptive generic names (`Size`, `Report`), not single letters
- Use descriptive variable and closure parameter names by default — single letters are only allowed in: conventional names (`n` for count, `f` for formatter), comparison closures (`|a, b|`), trivial single-expression closures, fold accumulators, index variables (`i`/`j`/`k` in short closures or index-based loops only), and test fixtures (identical roles only). Never use single letters in multi-line functions or closures
- Use `pipe-trait` for chaining through unary functions (constructors, `Some`, `Ok`, free functions, etc.), avoiding nested calls, and continuing method chains — but not for simple standalone calls (prefer `foo(value)` over `value.pipe(foo)`)
- Prefer `where` clauses for multiple trait bounds
- Derive order: std traits → comparison traits → `Hash` → derive_more → feature-gated
- Error types: only derive `Display` and `Error` from `derive_more` when each is actually needed — not all displayable types are errors
- Minimize `unwrap()` in non-test code — use proper error handling
- Prefer `#[cfg_attr(..., ignore = "reason")]` over `#[cfg(...)]` to skip tests — use `#[cfg]` on tests only when the code cannot compile under the condition (e.g., references types/functions that don't exist on other platforms)
- Install toolchain before running tests: `rustup toolchain install "$(< rust-toolchain)" && rustup component add --toolchain "$(< rust-toolchain)" rustfmt clippy`
- If you change CLI arguments, help text, or anything that affects command-line output, run `./generate-completions.sh` to regenerate the shell completion files, help text files, and `USAGE.md`. **Do not attempt to regenerate these files manually** — always use the script.
- Run `FMT=true LINT=true BUILD=true TEST=true DOC=true ./test.sh` to validate changes. If a test fails with a hint about `TEST_SKIP`, follow the hint and rerun with the suggested variable. If a sync test fails, read its error message carefully and run the exact command it tells you to run.
- **ALWAYS run the full test suite** (`FMT=true LINT=true BUILD=true TEST=true DOC=true ./test.sh`) before committing, regardless of how trivial the change seems — this includes documentation-only changes, comment edits, config changes, and refactors. The test suite checks formatting, linting, building, tests, and docs across multiple feature combinations; any type of change can break any of these checks.
- When the user provides a diff and you need to update the files, don't manually interpret each hunk (that'd be slow); apply it with `git apply` instead. If the user provides a diff for context or discussion rather than as a change to apply, respond accordingly instead.
- `gh` (GitHub CLI) is not installed — do not attempt to use it

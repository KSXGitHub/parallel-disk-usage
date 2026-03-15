# AI Instructions

Read and follow the CONTRIBUTING.md file in this repository for all code style conventions, commit message format, and development guidelines.

## Quick Reference

- Commit format: Conventional Commits — `type(scope): lowercase description`
- Version releases are the only exception: just the version number (e.g. `0.21.1`)
- Prefer merged imports
- Use descriptive generic names (`Size`, `Report`), not single letters
- Prefer `where` clauses for multiple trait bounds
- Derive order: std traits → comparison traits → `Hash` → derive_more → feature-gated
- Custom errors: `#[derive(Debug, Display, Error)]` + `#[non_exhaustive]`
- Minimize `unwrap()` in non-test code — use proper error handling
- `#![deny(warnings)]` is active — code must be warning-free
- Install toolchain before running tests: `rustup toolchain install "$(< rust-toolchain)" && rustup component add --toolchain "$(< rust-toolchain)" rustfmt clippy`
- If the AI agent is Claude Code, `gh` (GitHub CLI) is not installed — do not attempt to use it
- Run `FMT=true LINT=true BUILD=true TEST=true DOC=true ./test.sh` to validate changes

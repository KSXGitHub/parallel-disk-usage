# Claude Code Instructions

Read and follow the CONTRIBUTING.md file in this repository for all code style conventions, commit message format, and development guidelines.

## Quick Reference

- Commit format: Conventional Commits — `type(scope): lowercase description`
- Version releases are the only exception: just the version number (e.g. `0.21.1`)
- Import order: internal (`crate::`/`super::`) → external crates → `std::`
- Use descriptive generic names (`Size`, `Report`), not single letters
- Prefer `where` clauses for multiple trait bounds
- Derive order: std traits → comparison traits → derive_more → feature-gated
- Custom errors: `#[derive(Debug, Display, Error)]` + `#[non_exhaustive]`
- No `unwrap()` — use proper error handling
- `#![deny(warnings)]` is active — code must be warning-free
- Run `FMT=true LINT=true BUILD=true TEST=true ./test.sh` to validate changes

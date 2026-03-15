# Contributing to parallel-disk-usage

## Commit Message Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/).

### Format

```
type(scope): lowercase description
```

### Rules

- **Types:** `feat`, `fix`, `refactor`, `perf`, `docs`, `style`, `chore`, `ci`, `test`, `lint`
- **Scopes** (optional): `cli`, `api`, `deps`, `readme`, `benchmark`, `toolchain`, `test`, or other relevant area
- **Description:** always lowercase after the colon, no trailing period, brief (3-7 words preferred)
- **Breaking changes:** append `!` before the colon (e.g. `feat(cli)!: remove deprecated flag`)
- **Code identifiers** in descriptions should be wrapped in backticks (e.g. `` chore(deps): update `rand` ``)

### Exception: Version Releases

Version release commits use **only** the version number as the message — no type prefix:

```
0.21.1
```

## Code Style

Automated tools enforce formatting (`cargo fmt`) and linting (`cargo clippy`). The following conventions are **not** enforced by those tools and must be followed manually.

### Import Organization

Prefer **merged imports** — combine multiple items from the same crate or module into a single `use` statement with braces rather than separate `use` lines. Import ordering is enforced by `cargo fmt`. Platform-specific imports (`#[cfg(unix)]`) go in a separate block after the main imports.

```rust
use crate::{
    args::{Args, Quantity, Threads},
    bytes_format::BytesFormat,
    size,
};
use clap::Parser;
use pipe_trait::Pipe;
use std::{io::stdin, time::Duration};

#[cfg(unix)]
use crate::get_size::{GetBlockCount, GetBlockSize};
```

### Module Organization

- Use the flat file pattern (`module.rs`) rather than `module/mod.rs` for submodules.
- List `pub mod` declarations first, then `pub use` re-exports, then private imports and items.
- Use `pub use` to re-export key types at the module level for convenience.

```rust
pub mod error_only_reporter;
pub mod error_report;
pub mod event;

pub use error_only_reporter::ErrorOnlyReporter;
pub use error_report::ErrorReport;
pub use event::Event;
```

- Type aliases using `pub use ... as ...` are used to provide semantic alternative names:

```rust
pub use Reflection as DataTreeReflection;
```

### Derive Macro Ordering

When deriving multiple traits, use this order and split across multiple `#[derive(...)]` lines for readability:

1. **Standard traits:** `Debug`, `Default`, `Clone`, `Copy`
2. **Comparison traits:** `PartialEq`, `Eq`, `PartialOrd`, `Ord`
3. **Hash**
4. **`derive_more` traits:** `Display`, `From`, `Into`, `Add`, `AddAssign`, etc.
5. **Feature-gated derives** on a separate `#[cfg_attr(...)]` line

```rust
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(From, Into, Add, AddAssign, Sub, SubAssign, Sum)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct Bytes(u64);
```

### Generic Parameter Naming

Use **descriptive names** for type parameters, not single letters:

- `Size`, `Name`, `SizeGetter`, `HardlinksRecorder`, `Report`

Single-letter generics are acceptable only in very short, self-contained trait impls.

### Variable and Closure Parameter Naming

Use **descriptive names** for variables and closure parameters by default. Single-letter names are permitted only in the specific cases listed below.

#### When single-letter names are allowed

- **Comparison closures:** `|a, b|` in `sort_by`, `cmp`, or similar two-argument comparison callbacks — this is idiomatic Rust.

  ```rust
  sort_reflection_by(&mut tree, |a, b| a.name.cmp(&b.name));
  ```

- **Conventional single-letter names:** `n` for a natural number (unsigned integer / count), `f` for a `fmt::Formatter`, and similar well-established conventions from math or the Rust standard library. Note: for indices, use `index`, `idx`, or `*_index` (e.g., `row_index`) — not `n`. (For `i`/`j`/`k`, see the dedicated rule below.)

  ```rust
  fn with_capacity(n: usize) -> Self { todo!() }
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
  ```

- **Index variables (`i`, `j`, `k`):** These may only be used in two contexts: (1) short closures, and (2) index-based loops/iterations (rare in Rust). In all other cases, use `index`, `idx`, or `*_index`.

  ```rust
  // OK — short closure
  left_indices.zip(right_indices).map(|(i, j)| matrix[i][j])

  // OK — index-based loop (rare in Rust)
  for i in 0..len { /* ... */ }

  // Bad — use a descriptive name instead
  let i = items.iter().position(|item| item.is_active()).unwrap();
  ```

- **Trivial single-expression closures:** A closure whose body is a single field access, method call, or wrapper may use a single letter when the type and purpose are obvious from context.

  ```rust
  .pipe(|x| vec![x])
  ```

- **Fold accumulators:** `acc` for the accumulator and a single letter for the element in trivial folds.

  ```rust
  .fold(PathBuf::new(), |acc, x| acc.join(x))
  ```

- **Test fixtures:** `let a`, `let b`, `let c` for interchangeable specimens with identical roles in equality or comparison tests (e.g., testing commutativity). Do not use single letters when the variables have distinct roles — use `actual`/`expected` or similar descriptive names instead.

  ```rust
  let a = vec![3, 1, 2].into_iter().collect::<BTreeSet<_>>();
  let b = vec![2, 3, 1].into_iter().collect::<BTreeSet<_>>();
  assert_eq!(a, b);
  ```

#### When single-letter names are NOT allowed

- **Multi-line functions and closures:** If a function or closure body spans multiple lines (e.g., contains a `let` binding followed by another expression, or multiple chained operations), use a descriptive name.

  ```rust
  // Good
  .map(|tree_row| {
      let columns = build_columns(tree_row);
      format_row(&columns)
  })

  // Bad
  .map(|t| {
      let columns = build_columns(t);
      format_row(&columns)
  })
  ```

- **`let` bindings in non-test code:** Always use descriptive names.

  ```rust
  // Good
  let metadata = entry.metadata()?;
  // Bad
  let m = entry.metadata()?;
  ```

- **Function and method parameters:** Always use descriptive names, except for conventional single-letter names listed above (`n`, `f`, etc.).

- **Closures with non-obvious context:** If the type or purpose is not immediately clear from the surrounding method chain, use a descriptive name.

  ```rust
  // Good — not obvious what the closure receives
  .filter_map(|entry| match entry { _ => todo!() })
  .for_each(|child| child.par_sort_by(compare))

  // Bad — reader must look up what .filter receives
  .filter(|x| x.get_mount_point() == mount_point)
  ```

### Trait Bounds

Prefer `where` clauses over inline bounds when there are multiple constraints:

```rust
impl<Size, SizeGetter, HardlinksRecorder, Report>
    From<FsTreeBuilder<'a, Size, SizeGetter, HardlinksRecorder, Report>>
    for DataTree<OsStringDisplay, Size>
where
    Report: Reporter<Size> + Sync + ?Sized,
    Size: size::Size + Send + Sync,
    SizeGetter: GetSize<Size = Size> + Sync,
    HardlinksRecorder: RecordHardlinks<Size, Report> + Sync + ?Sized,
```

### Visibility

- Use `pub` for the public API surface.
- Use `pub(crate)` for items shared within the crate but not exposed externally.
- Default to private for everything else.

### Error Handling

- Define custom error enums with `#[derive(Debug, Display, Error)]` from `derive_more`.
- Mark error enums as `#[non_exhaustive]`.
- Minimize `unwrap()` in non-test code — use proper error propagation. `unwrap()` is acceptable in tests and for provably infallible operations (with a comment explaining why). When deliberately ignoring an error, use `.ok()` with a comment explaining why.

```rust
#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum RuntimeError {
    #[display("SerializationFailure: {_0}")]
    SerializationFailure(serde_json::Error),
}
```

### Documentation Comments

- Use `///` doc comments for all public types, traits, functions, and fields.
- Use `//!` module-level doc comments at the top of `lib.rs` and significant modules.
- Include usage examples with `/// ```no_run` blocks for key public APIs.
- Reference related types with `[`backtick links`](crate::path)` syntax.

### Feature Gating

- Use `#[cfg(feature = "...")]` for optional functionality (e.g., `json`, `cli`).
- Use `#[cfg(unix)]` for POSIX-specific code.
- Use `#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]` for conditional derives.

### Pattern Matching

Use exhaustive matching. When mapping enum variants to values, prefer the concise wrapping style:

```rust
ExitCode::from(match self {
    RuntimeError::SerializationFailure(_) => 2,
    RuntimeError::DeserializationFailure(_) => 3,
})
```

### Struct Field Ordering

Order fields logically by purpose, not alphabetically. Group related fields together. Document every public field with `///` comments.

### Macros

Use macros to reduce boilerplate for repetitive patterns (e.g. newtype wrappers, trait impls for multiple numeric types). Keep macros well-scoped and documented.

### Warnings Policy

The crate uses `#![deny(warnings)]` — all warnings are treated as errors. Code must compile warning-free.

## Setup

Install the required Rust toolchain and components before running any checks:

```sh
rustup toolchain install "$(< rust-toolchain)"
rustup component add --toolchain "$(< rust-toolchain)" rustfmt clippy
```

## Automated Checks

Before submitting, ensure:

- `cargo fmt -- --check` passes
- `cargo clippy` passes (on all feature combinations)
- `cargo test` passes
- The project builds with no default features, default features, and all features

The CI script `test.sh` runs all of these across 5 feature combinations. You can run it locally with:

```sh
FMT=true LINT=true BUILD=true TEST=true DOC=true ./test.sh
```

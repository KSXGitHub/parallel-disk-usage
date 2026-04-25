# Contributing to parallel-disk-usage

## Commit Message Convention

This project uses [Conventional Commits](https://www.conventionalcommits.org/).

### Format

```
type(scope): lowercase description
```

### Rules

- **Types:** `feat`, `fix`, `refactor`, `perf`, `docs`, `style`, `chore`, `ci`, `test`, `lint`.
- **Scopes** (optional): `cli`, `api`, `deps`, `readme`, `benchmark`, `toolchain`, `test`, or another relevant area.
- **Description:** always lowercase after the colon, no trailing period, brief (3-7 words preferred).
- **Breaking changes:** append `!` before the colon. For example: `feat(cli)!: remove deprecated flag`.
- **Code identifiers** in descriptions should be wrapped in backticks. For example: `` chore(deps): update `rand` ``.

### Exception: Version Releases

Version release commits use **only** the version number as the message, with no type prefix:

```
0.21.1
```

## Writing Style

Write documentation, comments, and other prose for ease of understanding first. Prefer a formal tone when it does not hurt clarity, and use complete sentences. Avoid mid-sentence breaks introduced by em dashes or long parenthetical clauses. Em dashes are a reliable symptom of loose phrasing; when one appears, restructure the surrounding sentence so each clause stands on its own rather than swapping the em dash for another punctuation mark.

## Code Style

Automated tools enforce formatting (`cargo fmt`) and linting (`cargo clippy`). The following conventions are **not** enforced by those tools and must be followed manually.

### Import Organization

Prefer **merged imports**. Combine multiple items from the same crate or module into a single `use` statement with braces rather than separate `use` lines. Import ordering is enforced by `cargo fmt`. Imports gated by a platform attribute such as `#[cfg(unix)]` go in a separate block after the main imports.

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

- **Comparison closures:** `|a, b|` in `sort_by`, `cmp`, or similar two-argument comparison callbacks. This is idiomatic Rust.

  ```rust
  sort_reflection_by(&mut tree, |a, b| a.name.cmp(&b.name));
  ```

- **Conventional single-letter names:** `n` for a natural number such as an unsigned integer or count, `f` for a `fmt::Formatter`, and similar well-established conventions from math or the Rust standard library. Note: for indices, use `index`, `idx`, or `*_index` such as `row_index`, not `n`. For `i`/`j`/`k`, see the dedicated rule below.

  ```rust
  fn with_capacity(n: usize) -> Self { todo!() }
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() }
  ```

- **Index variables (`i`, `j`, `k`):** These may only be used in two contexts: short closures, and index-based loops or iterations. The latter is rare in Rust. In all other cases, use `index`, `idx`, or `*_index`.

  ```rust
  // OK: short closure
  left_indices.zip(right_indices).map(|(i, j)| matrix[i][j])

  // OK: index-based loop (rare in Rust)
  for i in 0..len { /* ... */ }

  // Bad: use a descriptive name instead
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

- **Test fixtures:** `let a`, `let b`, `let c` for interchangeable specimens with identical roles in equality or comparison tests. Do not use single letters when the variables have distinct roles; use `actual`/`expected` or similar descriptive names instead.

  ```rust
  let a = vec![3, 1, 2].into_iter().collect::<BTreeSet<_>>();
  let b = vec![2, 3, 1].into_iter().collect::<BTreeSet<_>>();
  assert_eq!(a, b);
  ```

#### When single-letter names are NOT allowed

- **Multi-line functions and closures:** Use a descriptive name when a function or closure body spans multiple lines. Examples include a body that contains a `let` binding followed by another expression, or a body with multiple chained operations.

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

- **Function and method parameters:** Always use descriptive names, except for the conventional single-letter names listed above, such as `n` and `f`.

- **Closures with non-obvious context:** When the type or purpose is not immediately clear from the surrounding method chain, use a descriptive name.

  ```rust
  // Good: not obvious what the closure receives
  .filter_map(|entry| match entry { _ => todo!() })
  .for_each(|child| child.par_sort_by(compare))

  // Bad: reader must look up what .filter receives
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

### Error Handling

- Use `derive_more` for error types. Only derive the traits that are actually used:
  - `Display`: derive when the type needs to be displayed, such as when it is printed to stderr or used in format strings.
  - `Error`: derive when the type is used as a `std::error::Error`, such as the error type in `Result` or the source of another error. Not all types with `Display` need `Error`.
  - A type that only needs formatting and not error handling should derive `Display` without `Error`.
- Minimize `unwrap()` in non-test code; use proper error propagation. `unwrap()` is acceptable in tests, and is also acceptable for provably infallible operations when accompanied by a comment explaining why. When deliberately ignoring an error, use `.ok()` with a comment explaining why.

```rust
#[derive(Debug, Display, Error)]
#[non_exhaustive]
pub enum RuntimeError {
    #[display("SerializationFailure: {_0}")]
    SerializationFailure(serde_json::Error),
}
```

### Conditional Test Skipping: `#[cfg]` vs `#[cfg_attr(..., ignore)]`

When a test cannot run under certain conditions, such as on the wrong platform, prefer `#[cfg_attr(..., ignore)]` over `#[cfg(...)]` to skip it. The test still compiles on every configuration and is only skipped at runtime. This approach catches type errors and regressions that a `#[cfg]` skip would hide.

Use `#[cfg]` on tests **only** when the code cannot compile under the condition. An example is a test that references types, functions, or trait methods gated behind `#[cfg]` that do not exist on other platforms or feature sets.

Prefer including a reason string in the `ignore` attribute to explain why the test is skipped.

```rust
// Good: test compiles everywhere, skipped at runtime on non-unix
#[test]
#[cfg_attr(not(unix), ignore = "only one path separator style is tested")]
fn unix_path_logic() { /* uses hardcoded unix paths but no unix-only types */ }

// Good: test CANNOT compile on non-unix (uses unix-only types)
#[cfg(unix)]
#[test]
fn block_size() { /* uses GetBlockSize which only exists on unix */ }
```

### Using `pipe-trait`

This codebase uses the [`pipe-trait`](https://docs.rs/pipe-trait) crate extensively. The `Pipe` trait enables method-chaining through unary functions, keeping code in a natural left-to-right reading order. Import it as `use pipe_trait::Pipe;`.

Any callable that takes a single argument works with `.pipe()`. This includes free functions, closures, newtype constructors, enum variant constructors, `Some`, `Ok`, `Err`, and trait methods such as `From::from`. The guidance below applies equally to all of them.

#### When to use pipe

**Chaining through a unary function at the end of an expression chain:**

```rust
// Good: pipe keeps the chain flowing left-to-right
stats.ino().pipe(InodeNumber)
list.into_sorted_unstable_by_key(|entry| u64::from(entry.ino))
    .pipe(Reflection)
value.0.into_iter().collect::<HashSet<_>>().pipe(Reflection)
METRIC.parse_value(bytes).pipe(Output::Units)
```

**Avoiding deeply nested function calls:**

```rust
// Nested calls are harder to read
let data = serde_json::from_reader::<_, JsonData>(stdin());
let name = Some(OsStringDisplay::from(entry.file_name()));

// Prefer piping instead
let data = stdin().pipe(serde_json::from_reader::<_, JsonData>);
let name = entry.file_name().pipe(OsStringDisplay::from).pipe(Some);
```

**Chaining through multiple unary functions:**

```rust
// Good: clear wrapping pipeline
ino.pipe(ConversionError::DuplicatedInode).pipe(Err)
map.pipe(HardlinkList).pipe(Ok)

UnsupportedFeature::DeduplicateHardlink
    .pipe(RuntimeError::UnsupportedFeature)
    .pipe(Err)
```

**Continuing a method chain through a free function and back to methods:**

```rust
// Good: pipe bridges from methods to a free function and back
block_dev
    .pipe(validate_block_device::<Fs>)
    .map(Cow::Borrowed)

"/sys/block"
    .pipe(Path::new)
    .join(block_dev)
    .pipe_as_ref(Fs::path_exists)
    .then_some(block_dev)
```

**Using `.pipe_as_ref()` to pass a reference mid-chain.** This avoids introducing a temporary variable when a free function takes `&T`:

```rust
// Good: pipe_as_ref calls .as_ref() then passes to the function
path_buf.pipe_as_ref(Fs::path_exists)

// Without pipe, you'd need a temporary or nested call
Fs::path_exists(path_buf.as_ref())
```

#### When NOT to use pipe

**Simple standalone function calls.** Pipe adds noise with no readability benefit:

```rust
// Bad: unnecessary pipe
let result = value.pipe(foo);

// Good: just call the function directly
let result = foo(value);
```

This applies to any unary callable, such as `Some`, `Ok`, or constructors, when there is no preceding chain to continue:

```rust
// Bad: pipe adds nothing here
let result = value.pipe(Some);

// Good: direct call is clearer
let result = Some(value);
```

However, piping through any unary function **is** preferred when it continues an existing chain:

```rust
// Good: continues a chain
report.summarize().pipe(Some)
entry.file_name().pipe(OsStringDisplay::from).pipe(Some)
```

### Pattern Matching

When mapping enum variants to values, prefer the concise wrapping style:

```rust
ExitCode::from(match self {
    RuntimeError::SerializationFailure(_) => 2,
    RuntimeError::DeserializationFailure(_) => 3,
})
```

## Setup

Install the required Rust toolchain and components before running any checks:

```sh
rustup toolchain install "$(< rust-toolchain)"
rustup component add --toolchain "$(< rust-toolchain)" rustfmt clippy
```

## Optional External Dependencies

Some integration tests require external tools that are not managed by `Cargo.toml`. These tests panic when the tools are absent. CI installs them to get full coverage.

- `squashfs-tools` (provides `mksquashfs`): cross-device (`--one-file-system`) FUSE test.
- `squashfuse` (provides `squashfuse`): cross-device (`--one-file-system`) FUSE test.
- `fuse3` (provides `fusermount3` and `/dev/fuse`): cross-device (`--one-file-system`) FUSE test.

Tests that need these tools will panic with a diagnostic message if they are missing. The panic message includes the specific `TEST_SKIP` variable to skip the test via `./test.sh`.

## Automated Checks

Before submitting, ensure:

- `cargo fmt -- --check` passes.
- `cargo clippy` passes on all feature combinations.
- `cargo test` passes.
- The project builds with no default features, with default features, and with all features.

The CI script `test.sh` runs all of these across 5 feature combinations. You can run it locally with:

```sh
FMT=true LINT=true BUILD=true TEST=true DOC=true ./test.sh
```

> [!IMPORTANT]
> Always run the full test suite before every commit. This rule applies to all changes, including documentation edits, comment changes, and config updates. Any change can break formatting, linting, building, tests, or doc generation across the different feature combinations.

> [!NOTE]
> Some tests may fail with a hint about `TEST_SKIP`. Follow the hint and rerun with the suggested variable.

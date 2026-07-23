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

Automated tools enforce formatting (`cargo fmt`), linting (`cargo clippy`), and a curated set of project-specific rules via dylint (`cargo dylint --all`, run with `DYLINT=true ./test.sh`). The following conventions must be followed manually unless a subsection notes that a specific dylint rule enforces them.

### Import Organization

Two `perfectionist` rules govern imports automatically:

- `perfectionist::import_granularity_mismatch`, configured for the `module` style, controls how items are merged within each `use` statement. Items from the same module are merged into a single braced `use` statement, while each module keeps its own `use` statement rather than collapsing an entire crate into one nested-braces statement.
- `perfectionist::import_grouping_mismatch`, configured for the `single_block` style with `reexports = "split"`, controls how `use` statements are partitioned into blocks. Private imports sit in one contiguous block with no blank lines between them. `pub use` re-exports lead, split into two blank-separated sub-blocks: submodule re-exports (a multi-segment path such as `pub use child::Item;`) above alias re-exports (a single-segment path such as `pub use Item as Alias;`).

Import ordering within the block is enforced separately by `cargo fmt`.

Imports gated by a platform or feature attribute such as `#[cfg(unix)]` are kept in their own block after the main imports, separated by a blank line. Under `single_block` the rule recognizes this trailing `#[cfg]` block automatically through its default `cfg_block_handling = "trailing"`, so no manual exception is required.

```rust
pub use iter::Iter;
pub use reflection::Reflection;

pub use Reflection as ListReflection;

use crate::size;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
```

### Module Organization

The flat file pattern (`module.rs` rather than `module/mod.rs`) is enforced by `clippy::mod_module_files`, enabled in `Cargo.toml`. Earlier releases relied on a `perfectionist::flat_module_pattern` rule; `perfectionist` `0.0.0-rc.19` removed it in favor of the equivalent Clippy lint. In addition to that requirement, follow these conventions:

- List `pub mod` declarations first, then the `pub use` re-exports, then the private `use` block, then the remaining items. `perfectionist::import_grouping_mismatch` (`reexports = "split"`) keeps re-exports above the private imports, split into a submodule-re-export block and an alias-re-export block, and `cargo fmt` sorts within each block.
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

The order of trait names within each `#[derive(...)]` attribute is enforced automatically by the `perfectionist::unordered_derives` rule, configured for the `prefix_then_alphabetical` style. The configured `prefix` in `dylint.toml` lists the trait families in their project-preferred order: `Debug`, formatting / error derives (`Display`, `Error`), defaults (`Default`, `SmartDefault`), `Clone` / `Copy`, comparison and `Hash`, reference wrappers (`AsRef`, `AsMut`, `Deref`, `DerefMut`), conversions (`From`, `Into`, `TryFrom`, `TryInto`, `FromStr`), iteration, arithmetic operator pairs and folds, and integer-format derives. Any trait that is not in the `prefix` (project-specific derives such as `Setters` and `Parser`) falls in ASCII-case-insensitive alphabetical order after the prefix entries.

The remaining conventions are not enforced by the rule and must be applied by hand. When a type derives many traits, split them across multiple `#[derive(...)]` lines for readability, and place feature-gated derives on a separate `#[cfg_attr(...)]` line.

```rust
#[derive(Debug, Display, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(From, Into, Add, AddAssign, Sub, SubAssign, Sum)]
#[cfg_attr(feature = "json", derive(Deserialize, Serialize))]
pub struct Bytes(u64);
```

### Generic Parameter Naming

Use **descriptive names** for type parameters, not single letters:

- `Size`, `Name`, `SizeGetter`, `HardlinksRecorder`, `Report`

Single-letter type parameters are flagged by `perfectionist::single_letter_generic`.

### Variable and Closure Parameter Naming

Use **descriptive names** for variables and closure parameters by default. Single-letter names are permitted only in the specific cases listed below. Enforced by `perfectionist::single_letter_let_binding`, `perfectionist::single_letter_function_param`, and `perfectionist::single_letter_closure_param`. The exact exemptions differ by binding kind, as the cases below describe. The `extra_allowed_idents` and `extra_trivial_callback_methods` knobs in `dylint.toml` extend the built-in exempt sets, though the project currently relies on the defaults aside from the `sort_reflection_by` callback.

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

- **Index variables (`i`, `j`, `k`):** These are exempt as function and closure parameters, and they read naturally in index-based loops or iterations, which are rare in Rust. They are not exempt as `let` bindings, where only `n` is allowed, so a `let` that holds an index must use `index`, `idx`, or `*_index` instead.

  ```rust
  // OK: closure parameter
  left_indices.zip(right_indices).map(|(i, j)| matrix[i][j])

  // OK: index-based loop
  for i in 0..len { /* ... */ }

  // Bad: a `let` binding allows only `n`, never `i`
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

- **`let` bindings:** Always use descriptive names.

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
- Minimize `unwrap()` in non-test code; use proper error propagation. `unwrap()` is acceptable in tests, and is also acceptable for provably infallible operations when accompanied by a comment explaining the invariant. When deliberately ignoring an error, use `.ok()` and document the rationale.

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

### Using `command-extra`

The integration tests build `std::process::Command` values with the [`command-extra`](https://docs.rs/command-extra) crate, which offers a chainable, owned style. Import it as `use command_extra::CommandExtra;`. Production code does not spawn subprocesses, so this convention applies only to the tests.

The standard `Command` builder methods, such as `arg`, `env`, and `current_dir`, take `&mut self` and return `&mut Command`. This makes them unsuitable for method chains that end in an owned value. The `CommandExtra` extension trait provides owned counterparts that take ownership and return an owned `Command`, enabling fluent one-expression construction:

```rust
// Good: fully chainable, owned style
let output = Command::new(PDU)
    .with_current_dir(&workspace)
    .with_arg("--json-output")
    .with_stdin(Stdio::null())
    .with_stdout(Stdio::piped())
    .output()
    .expect("spawn pdu");

// Bad: the borrowing builders force a separate mutable binding
let mut command = Command::new(PDU);
command.current_dir(&workspace);
command.arg("--json-output");
let output = command.output().expect("spawn pdu");
```

The trait provides an owned counterpart for each standard builder method, and the names follow three prefix patterns. Methods that add or set a value keep the `with_*` prefix: `with_arg`, `with_args`, `with_current_dir`, `with_env`, `with_stdin`, `with_stdout`, and `with_stderr`. Removal takes the `without_*` prefix, so `without_env` is the counterpart of `env_remove`. Clearing takes the `with_no_*` prefix, so `with_no_env` is the counterpart of `env_clear`.

### Pattern Matching

When mapping enum variants to values, prefer the concise wrapping style:

```rust
ExitCode::from(match self {
    RuntimeError::SerializationFailure(_) => 2,
    RuntimeError::DeserializationFailure(_) => 3,
})
```

## Dependency Injection for Tests

Some code paths cannot be reached by a real fixture. Reading a `sysinfo::Disk`, which has no public constructor, probing sysfs under `/sys/block`, resolving symbolic links, and canonicalizing paths against the live filesystem are all examples. For these paths this project uses a dependency-injection-for-tests pattern: the side effects a function needs are expressed as capability traits, production supplies a real provider, and each test supplies a fake.

### When to reach for it

The default remains real fixtures, such as a temporary directory or an integration test that runs the built binary. Reach for a dependency-injection seam only for paths that a real fixture cannot reach portably or deterministically.

- Values the standard library and third-party crates will not let a test construct, such as a `sysinfo::Disk`.
- Host state that a test cannot stage on demand, such as the sysfs block-device tree or a specific arrangement of symbolic links.
- Filesystem error branches that the host will not reproduce on request.

The opposite is also a smell: a function that carries a `Sys` generic but is only ever exercised through real fixtures is over-designed. If no test substitutes a fake for the seam, remove the generic and call the real operation directly.

### Naming

- The generic parameter is named `Sys`.
- The single production provider is named `Host`. It delegates every capability to the real operating system.
- A capability trait is named for the action it performs, such as `GetDiskKind`, `Canonicalize`, or `IsRealDir`.
- A fake is named for its behavior, such as a fake filesystem that resolves a fixed table of symbolic links, or for what it stands in for, such as a fake through which disk detection is tested.

### One trait per capability

Each capability is its own trait with a single method. A function then binds only the capabilities it actually consumes. Carry those capabilities as a single `Sys` generic with several bounds rather than one generic per capability, so call sites stay short and a fake only implements the methods the function under test exercises.

```rust
pub trait Canonicalize {
    fn canonicalize(path: &Path) -> io::Result<PathBuf>;
}

pub trait PathExists {
    fn path_exists(path: &Path) -> bool;
}

pub trait ReadLink {
    fn read_link(path: &Path) -> io::Result<PathBuf>;
}
```

Keep capabilities at the level of leaf primitives, each mirroring a single standard-library function, and compose higher-level behavior as ordinary free functions on top of them. A pure computation such as a path-prefix check is a plain method call inside the algorithm rather than a capability, because it touches no side effect.

### Self-less methods and a stateless provider

The provider holds no state of its own, so every capability method is an associated function that takes no `&self`. The disk value the disk-reading capabilities operate on is exposed as an associated type, so production reads a real `sysinfo::Disk` while a test substitutes a lightweight stand-in that the provider chooses.

```rust
pub trait DiskSource {
    type Disk;
}

pub trait GetMountPoint: DiskSource {
    fn get_mount_point(disk: &Self::Disk) -> &Path;
}

impl DiskSource for Host {
    type Disk = sysinfo::Disk;
}

impl GetMountPoint for Host {
    fn get_mount_point(disk: &Self::Disk) -> &Path {
        disk.mount_point()
    }
}
```

Production call sites name the provider explicitly through a turbofish, such as `some_operation::<Host>(&files)`, so the production choice is visible at the call site rather than left to inference.

### Fakes and their state are function-scoped

Each test defines its own fake `struct`, and, when the fake needs state, its own `static` for that state, both inside the test body. Rust allows `static`, `struct`, `const`, and `impl` items inside a function, and a function-local `static` still has `'static` lifetime, so each test stays self-contained and shares nothing with the others. Do not hoist fixture state to a module-level or global `static` to share it across tests, and do not reach for `thread_local!` to paper over such sharing.

For example, a test that needs specific host state declares its fixture tables as `static` items inside the test function, with its fake alongside them:

```rust
#[test]
fn some_reclassification_case() {
    static DEVICES: &[&str] = &["/sys/block/vda"];
    static DRIVERS: &[(&str, &str)] = &[("/sys/block/vda/device/driver", "virtio_blk")];

    struct Fs;
    impl PathExists for Fs {
        fn path_exists(path: &Path) -> bool {
            DEVICES.iter().any(|dev| path == Path::new(*dev))
        }
    }
    // ... the remaining capabilities, then the assertion ...
}
```

A fake that holds no state is the one exception. Because it has nothing to race on, it may sit at module scope and be shared, in the manner of a frozen clock. A stateless fake that only reads a fixed table of module constants is such a case, so it may be declared once and reused across the tests in its module. Small stateless helpers, such as a symlink resolver, may likewise stay at module scope.

## Unit Tests

A unit-test module may either sit inline as `mod tests { ... }` in its parent or live in a dedicated external `tests` submodule. Use the inline form for short test modules. Once the block becomes long enough to obscure the surrounding module, move the tests into an external file.

### When the inline form is acceptable

The inline form `mod tests { ... }` is acceptable on its own. Reserve it for modules whose entire test suite fits in a small number of lines, so the block does not noticeably extend the parent. Use the number of lines as the deciding factor.

### Where the external file sits

When the tests live externally, the parent declares them at the end of the file with the standard declaration:

```rust
#[cfg(test)]
mod tests;
```

The external file itself sits in a directory named after the parent, using the same path regardless of whether the parent has any other submodules. Concretely:

- For `src/foo.rs`, the tests file is `src/foo/tests.rs`.
- For `src/foo/bar.rs`, the tests file is `src/foo/bar/tests.rs`.

Do not flatten the tests into a sibling file such as `src/foo_tests.rs`, and do not skip the intermediate directory when the parent currently has no other submodules. This mirrors the flat file pattern (`module.rs` rather than `module/mod.rs`) described under [Module Organization](#module-organization).

## Setup

Install the required Rust toolchain and components before running any checks:

```sh
rustup toolchain install "$(< rust-toolchain)"
rustup component add --toolchain "$(< rust-toolchain)" rustfmt clippy
```

To run the dylint checks locally, also install `cargo-dylint` and `dylint-link`:

```sh
cargo install cargo-dylint dylint-link
```

These are only required when running with `DYLINT=true`. The dylint libraries declared in `dylint.toml` are built against their own pinned nightly toolchain, which `cargo-dylint` fetches automatically on first run.

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
- `cargo dylint --all` passes (requires `cargo-dylint` and `dylint-link`).

The CI script `test.sh` runs all of these across every supported feature combination. You can run it locally with:

```sh
FMT=true LINT=true BUILD=true TEST=true DOC=true DYLINT=true ./test.sh
```

`DYLINT` defaults to `false` because it requires extra tooling and a separate nightly toolchain. Enable it once `cargo-dylint` and `dylint-link` are installed.

> [!IMPORTANT]
> Always run the full test suite before every commit. This rule applies to all changes, including documentation edits, comment changes, and config updates. Any change can break formatting, linting, building, tests, or doc generation across the different feature combinations.

> [!NOTE]
> Some tests may fail with a hint about `TEST_SKIP`. Follow the hint and rerun with the suggested variable.

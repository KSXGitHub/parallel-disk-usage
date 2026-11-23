# Getting Started

## Installation

The default feature is `cli`, which builds the `pdu` binary and pulls in `clap`. Library consumers rarely want that.

```toml
[dependencies]
parallel-disk-usage = { version = "0.24", default-features = false }
```

Add `json` for serialization:

```toml
[dependencies]
parallel-disk-usage = { version = "0.24", default-features = false, features = ["json"] }
```

See [Feature Flags](Library-Feature-Flags.md) for the full list.

The crate name is hyphenated; the library is imported with underscores:

```rust
use parallel_disk_usage::data_tree::DataTree;
```

## First measurement

```rust
use parallel_disk_usage::data_tree::DataTree;
use parallel_disk_usage::device::DeviceBoundary;
use parallel_disk_usage::fs_tree_builder::FsTreeBuilder;
use parallel_disk_usage::get_size::GetApparentSize;
use parallel_disk_usage::hardlink::HardlinkIgnorant;
use parallel_disk_usage::os_string_display::OsStringDisplay;
use parallel_disk_usage::reporter::{ErrorOnlyReporter, ErrorReport};
use parallel_disk_usage::size::Bytes;

fn main() {
	let builder = FsTreeBuilder {
		root: std::env::current_dir().expect("get the current directory"),
		size_getter: GetApparentSize,
		hardlinks_recorder: &HardlinkIgnorant,
		reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
		device_boundary: DeviceBoundary::Cross,
		max_depth: u64::MAX,
	};

	let data_tree: DataTree<OsStringDisplay, Bytes> = builder.into();

	println!("{} bytes", data_tree.size().inner());
}
```

`FsTreeBuilder` is a struct of parameters; the traversal happens in its `From` implementation, so
`builder.into()` is what triggers the work. The type annotation on `data_tree` is what selects
`Bytes`. `reporter` and `hardlinks_recorder` are borrowed because they are shared across worker threads.

`max_depth` limits how deep the tree is *stored*, not how deep it is *measured*. Sizes below the cutoff still count toward their ancestors' totals.

## Adding a chart

Sort before rendering; the visualizer does not sort.

```rust
use parallel_disk_usage::bytes_format::BytesFormat;
use parallel_disk_usage::visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer};

let data_tree = data_tree.into_par_sorted( | left, right| left.size().cmp( & right.size()).reverse());

let visualizer = Visualizer {
data_tree: & data_tree,
bytes_format: BytesFormat::MetricUnits,
direction: Direction::BottomUp,
bar_alignment: BarAlignment::Left,
column_width_distribution: ColumnWidthDistribution::total(100),
};

println!("{visualizer}");
```

## Reporting errors

`ErrorReport::SILENT` discards every error. `ErrorReport::TEXT` prints each one to stderr through the library's status board:

```rust
use parallel_disk_usage::reporter::{ErrorOnlyReporter, ErrorReport};

let reporter = ErrorOnlyReporter::new(ErrorReport::TEXT);
```

Both are `fn(ErrorReport)` values. Supply any `Fn(ErrorReport)` to handle reports yourself:

```rust
let reporter = ErrorOnlyReporter::new( | report: ErrorReport| {
eprintln ! (
"[error] {operation} {path:?}: {error}",
operation = report.operation.name(),
path = report.path,
error = report.error,
);
});
```

For live progress counters, use `ProgressAndErrorReporter`. See [Reporters](Library-Reporters.md).

## Next

* [Building Trees](Library-Building-Trees.md) for every `FsTreeBuilder` field and the generic
  `TreeBuilder`.
* [DataTree](Library-DataTree.md) for inspection, sorting, filtering, and serialization.
* [Recipes](Library-Recipes.md) for end-to-end examples.

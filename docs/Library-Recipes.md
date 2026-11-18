# Recipes

Task-oriented examples, each assuming the dependency configuration from
[Getting Started](Library-Getting-Started.md).

## Total size of a directory

```rust
use parallel_disk_usage::data_tree::DataTree;
use parallel_disk_usage::device::DeviceBoundary;
use parallel_disk_usage::fs_tree_builder::FsTreeBuilder;
use parallel_disk_usage::get_size::GetApparentSize;
use parallel_disk_usage::hardlink::HardlinkIgnorant;
use parallel_disk_usage::os_string_display::OsStringDisplay;
use parallel_disk_usage::reporter::{ErrorOnlyReporter, ErrorReport};
use parallel_disk_usage::size::Bytes;
use std::path::Path;

fn total_size(root: &Path) -> Bytes {
	let data_tree: DataTree<OsStringDisplay, Bytes> = FsTreeBuilder {
		root: root.to_path_buf(),
		size_getter: GetApparentSize,
		hardlinks_recorder: &HardlinkIgnorant,
		reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
		device_boundary: DeviceBoundary::Cross,
		max_depth: 1,
	}
		.into();

	data_tree.size()
}
```

`max_depth: 1` keeps only the root node. The traversal still visits every file, so the total is complete.

## Largest entries directly under a directory

```rust
use parallel_disk_usage::size::Size;

let data_tree: DataTree<OsStringDisplay, Bytes> = FsTreeBuilder {
root: root.to_path_buf(),
size_getter: GetApparentSize,
hardlinks_recorder: & HardlinkIgnorant,
reporter: & ErrorOnlyReporter::new(ErrorReport::SILENT),
device_boundary: DeviceBoundary::Cross,
max_depth: 2,
}
.into();

let data_tree = data_tree.into_par_sorted( | left, right| left.size().cmp( & right.size()).reverse());

for child in data_tree.children().iter().take(10) {
println ! ("{:>10}  {}", child.size().display(BytesFormat::MetricUnits), child.name());
}
```

`max_depth: 2` retains the root and its immediate children.

## A chart sized to the terminal

```rust
use parallel_disk_usage::bytes_format::BytesFormat;
use parallel_disk_usage::visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer};

let width = terminal_size::terminal_size()
.map_or(80, | (terminal_size::Width(width), _) | usize::from(width));

let visualizer = Visualizer {
data_tree: & data_tree,
bytes_format: BytesFormat::MetricUnits,
direction: Direction::BottomUp,
bar_alignment: BarAlignment::Left,
column_width_distribution: ColumnWidthDistribution::total(width),
};

print!("{visualizer}"); // the chart already ends with a newline
```

## Live progress during a long scan

```rust
use parallel_disk_usage::reporter::{
	ErrorReport, ParallelReporter, ProgressAndErrorReporter, ProgressReport,
};
use parallel_disk_usage::status_board::GLOBAL_STATUS_BOARD;
use std::time::Duration;

let reporter = ProgressAndErrorReporter::<Bytes, _ >::new(
ProgressReport::TEXT,
Duration::from_millis(100),
ErrorReport::TEXT,
);

let data_tree: DataTree<OsStringDisplay, Bytes> = FsTreeBuilder {
root: root.to_path_buf(),
size_getter: GetApparentSize,
hardlinks_recorder: & HardlinkIgnorant,
reporter: & reporter,
device_boundary: DeviceBoundary::Cross,
max_depth: u64::MAX,
}
.into();

if reporter.destroy().is_err() {
eprintln ! ("[warning] the progress reporting thread panicked");
}

GLOBAL_STATUS_BOARD.clear_line(0);
```

Destroying the reporter before rendering keeps the progress line out of the chart; `clear_line(0)`
erases the last one it printed.

## Several roots at once

`pdu` puts every root under a synthetic parent of zero size, so the total is the sum.

```rust
use parallel_disk_usage::data_tree::DataTree;

let children: Vec<DataTree<OsStringDisplay, Bytes> > = roots
.iter()
.map( | root| {
FsTreeBuilder {
root: root.to_path_buf(),
size_getter: GetApparentSize,
hardlinks_recorder: & HardlinkIgnorant,
reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
device_boundary: DeviceBoundary::Cross,
max_depth: u64::MAX,
}
.into()
})
.collect();

let total = DataTree::dir(
OsStringDisplay::os_string_from("(total)"),
Bytes::new(0),
children,
);
```

If you also deduplicate hardlinks, name the synthetic root with an empty string until after
`deduplicate` has run, then rename it with `name_mut`. Path prefix stripping treats the empty string as a prefix of every path; any other name prevents matches. This is why `pdu` renames late.

## Filtering out the noise

Keep entries accounting for at least one percent of the total:

```rust
let minimal = data_tree.size().inner() as f32 * 0.01;
let data_tree = data_tree.into_par_retained( | node, _ | node.size().inner() as f32 > = minimal);
```

Limit display depth without limiting measurement:

```rust
let data_tree = data_tree.into_par_retained( | _, depth| depth < 3);
```

Both leave ancestor sizes untouched, so a directory keeps its true size after its contents are dropped.

## Apparent size against on-disk size

On Unix, measuring twice with different size getters shows the space lost to block rounding or saved by sparse files.

```rust
#[cfg(unix)]
use parallel_disk_usage::get_size::{GetApparentSize, GetBlockSize};

#[cfg(unix)]
fn compare(root: &std::path::Path) {
	let apparent: DataTree<OsStringDisplay, Bytes> = FsTreeBuilder {
		root: root.to_path_buf(),
		size_getter: GetApparentSize,
		hardlinks_recorder: &HardlinkIgnorant,
		reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
		device_boundary: DeviceBoundary::Cross,
		max_depth: 1,
	}
		.into();

	let on_disk: DataTree<OsStringDisplay, Bytes> = FsTreeBuilder {
		root: root.to_path_buf(),
		size_getter: GetBlockSize,
		hardlinks_recorder: &HardlinkIgnorant,
		reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
		device_boundary: DeviceBoundary::Cross,
		max_depth: 1,
	}
		.into();

	println!("apparent {:?}, on disk {:?}", apparent.size(), on_disk.size());
}
```

## Staying on one filesystem

```rust
use parallel_disk_usage::device::DeviceBoundary;

let builder = FsTreeBuilder {
root: "/".into(),
device_boundary: DeviceBoundary::Stay,
// ...
};
```

The equivalent of `--one-file-system`, which keeps a scan of `/` out of network mounts and removable media.

## A tree from something other than a filesystem

Anything with a parent-child structure and a size works with `TreeBuilder`. This walks an in-memory map.

```rust
use parallel_disk_usage::data_tree::DataTree;
use parallel_disk_usage::size::Bytes;
use parallel_disk_usage::tree_builder::{Info, TreeBuilder};
use std::collections::HashMap;

fn build(entries: &HashMap<String, (u64, Vec<String>)>) -> DataTree<String, Bytes> {
	TreeBuilder::<String, String, Bytes, _, _> {
		path: "/".to_string(),
		name: "/".to_string(),
		get_info: |path| match entries.get(path) {
			Some((size, children)) => Info {
				size: Bytes::new(*size),
				children: children.clone(),
			},
			None => Info {
				size: Bytes::new(0),
				children: Vec::new(),
			},
		},
		join_path: |prefix, name| format!("{prefix}/{name}"),
		max_depth: u64::MAX,
	}
		.into()
}
```

Both closures capture `entries` by shared reference, satisfying the `Copy + Sync` bound. See
[Building Trees](Library-Building-Trees.md#treebuilder).

## Round-tripping through JSON

*(feature: `json`)*

```rust
use parallel_disk_usage::json_data::{BinaryVersion, JsonData, JsonDataBody, JsonShared, JsonTree, SchemaVersion};

let tree = data_tree
.into_reflection()
.par_convert_names_to_utf8()
.expect("all names are valid UTF-8");

let json_data = JsonData {
schema_version: SchemaVersion,
binary_version: Some(BinaryVersion::current()),
body: JsonTree { tree, shared: JsonShared::default () }.into(),
};

let text = serde_json::to_string( & json_data).expect("serialize");
let parsed: JsonData = serde_json::from_str( & text).expect("deserialize");

let JsonDataBody::Bytes(json_tree) = parsed.body else {
panic ! ("expected a byte tree");
};
let data_tree = json_tree.tree.par_try_into_tree().expect("valid tree");
```

The output is byte-compatible with `pdu --json-output` and can be piped into `pdu --json-input`. See [JSON Data](Library-JSON-Data.md).

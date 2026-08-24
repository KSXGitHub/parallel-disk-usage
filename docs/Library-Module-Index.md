# Module Index

A map of the `parallel_disk_usage` crate. Items marked *(cli)*, *(json)*, or *(unix)* are conditional; see [Feature Flags](Library-Feature-Flags.md).

Generated API documentation lives at
[docs.rs/parallel-disk-usage](https://docs.rs/parallel-disk-usage). This page tells you which module to open.

## Measurement

**`fs_tree_builder`** — `FsTreeBuilder`, which walks a real directory tree. See
[Building Trees](Library-Building-Trees.md#fstreebuilder).

**`tree_builder`** — `TreeBuilder` and `Info`, the generic parallel recursion underneath
`FsTreeBuilder`. See [Building Trees](Library-Building-Trees.md#treebuilder).

**`get_size`** — `GetSize`, plus `GetApparentSize`, `GetBlockSize` *(unix)*, and
`GetBlockCount` *(unix)*. See
[Sizes and Formatting](Library-Sizes-And-Formatting.md#getsize).

## The tree

**`data_tree`** — `DataTree` with its constructors, getters, and the `par_sort_by` and `par_retain`
transformations. `data_tree::reflection` holds `Reflection`, also exported as
`DataTreeReflection`, and `ConversionError`. See [DataTree](Library-DataTree.md).

**`size`** — the `Size` trait and the units `Bytes` and `Blocks`. See
[Sizes and Formatting](Library-Sizes-And-Formatting.md#the-size-trait).

**`os_string_display`** — `OsStringDisplay`, the name type `FsTreeBuilder` produces. It wraps an
`OsString`, displays valid UTF-8 as text, and falls back to the `Debug` form otherwise.
`os_string_from` constructs one; `as_os_str` and `inner` read it back. It derefs to its inner value and derives `Serialize` and `Deserialize` *(json)*.

## Presentation

**`visualizer`** — `Visualizer` with `Direction`, `BarAlignment`, and `ColumnWidthDistribution`, plus the rendering components `ProportionBar`, `ProportionBarBlock`, `TreeSkeletalComponent`,
`TreeHorizontalSlice`, `ChildPosition`, and `Parenthood`. See [Visualizer](Library-Visualizer.md).

**`bytes_format`** — `BytesFormat`, `Formatter`, `ParsedValue`, `Output`, and the `scale_base`
constants. See [Sizes and Formatting](Library-Sizes-And-Formatting.md#formatting-bytes).

**`status_board`** — `StatusBoard` and the `GLOBAL_STATUS_BOARD` static, which arbitrate between transient and permanent messages on stderr. See
[Reporters](Library-Reporters.md#the-status-board).

## Progress and errors

**`reporter`** — `Reporter`, `ParallelReporter`, `Event`, the implementations `ErrorOnlyReporter`
and `ProgressAndErrorReporter`, and `ErrorReport`, `error_report::Operation`, and `ProgressReport`. See [Reporters](Library-Reporters.md).

## Hardlinks and filesystem identity

**`hardlink`** — `RecordHardlinks` and `DeduplicateSharedSize`, the implementations
`HardlinkIgnorant` and `HardlinkAware` *(unix)*, and the storage types `HardlinkList`,
`LinkPathList`, their reflections, and `SharedLinkSummary`. See [Hardlinks](Library-Hardlinks.md).

**`inode`** — `InodeNumber`, a newtype over `u64`. `InodeNumber::get(&metadata)` *(unix)* reads one from `Metadata`.

**`device`** — `DeviceBoundary`, the `Cross` or `Stay` choice passed to `FsTreeBuilder`, and
`DeviceNumber`, whose `get(&metadata)` *(unix)* reads a filesystem's device number. Both newtypes implement `Display` and the hexadecimal and octal formatting traits.

## Interchange

**`json_data`** — `JsonData`, `JsonDataBody`, `JsonTree`, `JsonShared`, `SchemaVersion`, and
`BinaryVersion`, plus the `SCHEMA_VERSION` and `CURRENT_VERSION` constants. See
[JSON Data](Library-JSON-Data.md).

## Command-line program

These modules implement `pdu`. They are public so the binary can use them, but are not a stable library surface. `app::Sub::run` is the reference example of the full pipeline.

| Module                  | Contents                                                           |
|-------------------------|--------------------------------------------------------------------|
| `app` *(cli)*           | `App::from_env` and `App::run`, and the generic `app::Sub`.        |
| `args` *(cli)*          | The `clap` argument definitions, including `Depth` and `Fraction`. |
| `runtime_error` *(cli)* | `RuntimeError` and its exit codes.                                 |
| `man_page` *(cli)*      | Man page generation.                                               |
| `usage_md` *(cli)*      | `USAGE.md` generation.                                             |

The crate also provides `parallel_disk_usage::main`, the entry point of the `pdu` binary.

## Re-exports

| Re-export                                 | Condition |
|-------------------------------------------|-----------|
| `zero_copy_pads`                          | Always    |
| `serde`, `serde_json`                     | *(json)*  |
| `clap`, `clap_complete`, `clap_utilities` | *(cli)*   |

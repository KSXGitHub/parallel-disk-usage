# Building Trees

A [`DataTree`](Library-DataTree.md) comes from one of three places: `FsTreeBuilder` for the real filesystem, `TreeBuilder` for any other hierarchy, or a
[`Reflection`](Library-DataTree.md#reflection). This page covers the first two.

## `FsTreeBuilder`

`fs_tree_builder::FsTreeBuilder` walks a real directory tree. It is a struct of parameters, and the traversal is performed by its `From` implementation, so measurement starts at `.into()`.

```rust
use parallel_disk_usage::data_tree::DataTree;
use parallel_disk_usage::device::DeviceBoundary;
use parallel_disk_usage::fs_tree_builder::FsTreeBuilder;
use parallel_disk_usage::get_size::GetApparentSize;
use parallel_disk_usage::hardlink::HardlinkIgnorant;
use parallel_disk_usage::os_string_display::OsStringDisplay;
use parallel_disk_usage::reporter::{ErrorOnlyReporter, ErrorReport};
use parallel_disk_usage::size::Bytes;

let builder = FsTreeBuilder {
root: "/usr/share".into(),
size_getter: GetApparentSize,
hardlinks_recorder: & HardlinkIgnorant,
reporter: & ErrorOnlyReporter::new(ErrorReport::SILENT),
device_boundary: DeviceBoundary::Cross,
max_depth: 10,
};

let data_tree: DataTree<OsStringDisplay, Bytes> = builder.into();
```

### Fields

| Field                | Type                                  | Meaning                                                                                                                       |
|----------------------|---------------------------------------|-------------------------------------------------------------------------------------------------------------------------------|
| `root`               | `PathBuf`                             | The directory or file at the top of the walk. Becomes the name of the root node.                                              |
| `size_getter`        | `impl GetSize<Size = Size> + Sync`    | Decides what a file's size is. See [Sizes and Formatting](Library-Sizes-And-Formatting.md#getsize).                              |
| `hardlinks_recorder` | `&impl RecordHardlinks<Size, Report>` | Detects and records hardlinks during the walk. Use `&HardlinkIgnorant` to skip detection. See [Hardlinks](Library-Hardlinks.md). |
| `reporter`           | `&impl Reporter<Size> + Sync`         | Receives progress and error events. See [Reporters](Library-Reporters.md).                                                       |
| `device_boundary`    | `DeviceBoundary`                      | `Cross` descends into other filesystems, `Stay` does not.                                                                     |
| `max_depth`          | `u64`                                 | Deepest level retained as nodes.                                                                                              |

`From<FsTreeBuilder<..>>` is implemented for `DataTree<OsStringDisplay, Size>`, where `Size` comes from the `size_getter`. The name type is always `OsStringDisplay`. Annotate the destination binding explicitly to pin both down.

### `max_depth`

`max_depth` limits the depth of the *stored* tree, not the depth of the *walk*. Everything below the cutoff is measured and contributes to its ancestors' totals, but is not kept as a separate node. A
`max_depth` of `1` yields a childless root whose size is the total of the whole tree. Use
`u64::MAX` for no limit.

Lowering `max_depth` therefore reduces memory usage but not traversal time.

### `device_boundary`

`DeviceBoundary::Stay` reproduces `--one-file-system`. It reads the device ID of `root` once, then skips any directory whose device ID differs. If `root` cannot be stated, the builder reports a
`SymlinkMetadata` error and returns a single zero-sized file node.

`DeviceBoundary::Cross` performs no device check.

### Errors

`FsTreeBuilder` never fails as a whole. Every I/O problem becomes an `Event::EncounterError` handed to the reporter, and the affected subtree is recorded with a size of zero and no children. The three failing operations are `SymlinkMetadata`, `ReadDirectory`, and `AccessEntry`. See
[Reporters](Library-Reporters.md#error-reports).

The walk uses `symlink_metadata`, so symbolic links are measured as links and never followed.

## `TreeBuilder`

`tree_builder::TreeBuilder` is the generic engine underneath `FsTreeBuilder`. It knows nothing about the filesystem: supply two closures and it performs the parallel recursion. Use it for a hierarchy that is not a directory tree, such as an archive index, an object store listing, or a test fixture.

```rust
use parallel_disk_usage::data_tree::DataTree;
use parallel_disk_usage::size::Bytes;
use parallel_disk_usage::tree_builder::{Info, TreeBuilder};

let builder = TreeBuilder::<String, String, Bytes, _, _ > {
path: "root".to_string(),
name: "root".to_string(),
get_info: | path| Info {
size: Bytes::new(path.len() as u64),
children: Vec::new(),
},
join_path: | prefix, name | format !("{prefix}/{name}"),
max_depth: 10,
};

let data_tree: DataTree<String, Bytes> = builder.into();
```

### Fields

| Field       | Meaning                                                                                     |
|-------------|---------------------------------------------------------------------------------------------|
| `path`      | The address of the root in whatever address space you are walking.                          |
| `name`      | The label of the root node. `Path` and `Name` may be different types.                       |
| `get_info`  | `Fn(&Path) -> Info<Name, Size>`. Returns the node's own size and the names of its children. |
| `join_path` | `Fn(&Path, &Name) -> Path`. Combines a parent path with a child name.                       |
| `max_depth` | Same meaning as in `FsTreeBuilder`.                                                         |

### `Info`

```rust
pub struct Info<Name, Size> {
	pub size: Size,
	pub children: Vec<Name>,
}
```

`size` is the node's own size, excluding children. `TreeBuilder` recurses into each child and
`DataTree::dir` adds their totals on top.

### Closure requirements

Both closures are `Copy + Send + Sync`, since they are cloned into every Rayon task. That rules out closures capturing by mutable reference or owning non-`Sync` state. For shared mutable state, capture a shared reference to something internally synchronized, as `FsTreeBuilder` does with its reporter and hardlink recorder.

`get_info` has no error channel and must not panic. Report failures out of band and return
`Info { size: default, children: vec![] }`, again following `FsTreeBuilder`.

## Choosing between the two

Use `FsTreeBuilder` for real directories. Reach for `TreeBuilder` when the source is not a filesystem, or when you need name and path types that `FsTreeBuilder` fixes for you.

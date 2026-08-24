# Hardlinks

A file with several hardlinks occupies its bytes once, but a naive walk counts them once per link. The `hardlink` module detects such files during measurement and subtracts the surplus afterwards. It is the library side of `--deduplicate-hardlinks`.

## Two phases

Deduplication is split in two: the first phase runs inside the parallel walk, the second only once the tree is complete.

1. **Record.** `RecordHardlinks::record_hardlinks` is called by `FsTreeBuilder` for every item it stats. An implementation notes files whose link count exceeds one and the paths where they were seen.
2. **Deduplicate.** `DeduplicateSharedSize::deduplicate` consumes the recorder, walks the finished tree, and reduces the size of every directory holding more than one link to the same inode.

One type usually implements both traits.

## The traits

```rust
pub trait RecordHardlinks<Size, Reporter: ?Sized> {
	type Error;
	fn record_hardlinks(&self, argument: RecordHardlinksArgument<Size, Reporter>) -> Result<(), Self::Error>;
}

pub trait DeduplicateSharedSize<Size: size::Size>: Sized {
	type Report;
	type Error;
	fn deduplicate(self, data_tree: &mut DataTree<OsStringDisplay, Size>) -> Result<Self::Report, Self::Error>;
}
```

`RecordHardlinksArgument` carries the `path`, the `stats`, the computed `size`, and the `reporter`, so an implementation can emit `Event::DetectHardlink` as it goes.

`record_hardlinks` takes `&self` and runs concurrently on every worker thread. `FsTreeBuilder`
discards its returned error, so an implementation that must surface a failure has to route it through the reporter.

`deduplicate` takes `self` by value, guaranteeing that recording has finished before correction begins. Its `Report` is the accumulated record.

## Implementations

### `HardlinkIgnorant`

Available everywhere. Both trait implementations are no-ops with `Error = Infallible`, and
`Report` is `()`. Hardlinks are treated as ordinary files. This is what `pdu` does without
`--deduplicate-hardlinks`.

```rust
use parallel_disk_usage::hardlink::HardlinkIgnorant;

let recorder = HardlinkIgnorant;
```

### `HardlinkAware`

*(Unix only)* Wraps a [`HardlinkList`](#hardlinklist). For every non-directory whose `nlink`
exceeds one, it emits `Event::DetectHardlink` and records the inode number, device number, size, link count, and path.

```rust
#[cfg(unix)]
use parallel_disk_usage::hardlink::HardlinkAware;
#[cfg(unix)]
use parallel_disk_usage::size::Bytes;

#[cfg(unix)]
let recorder = HardlinkAware::<Bytes>::new();
```

`deduplicate` returns the accumulated `HardlinkList`, with `Error = Infallible`. Recording can fail with `ReportHardlinksError::AddToRecord` when the same inode is observed with two different sizes or link counts, which means the filesystem changed mid-scan.

Directories are skipped, since their link counts reflect subdirectory entries rather than shared content.

## Putting it together

The recorder is borrowed during measurement and consumed afterwards, so bind it to a variable.

```rust
use parallel_disk_usage::data_tree::DataTree;
use parallel_disk_usage::device::DeviceBoundary;
use parallel_disk_usage::fs_tree_builder::FsTreeBuilder;
use parallel_disk_usage::get_size::GetApparentSize;
use parallel_disk_usage::hardlink::{DeduplicateSharedSize, HardlinkAware};
use parallel_disk_usage::os_string_display::OsStringDisplay;
use parallel_disk_usage::reporter::{ErrorOnlyReporter, ErrorReport};
use parallel_disk_usage::size::Bytes;

let recorder = HardlinkAware::<Bytes>::new();

let mut data_tree: DataTree<OsStringDisplay, Bytes> = FsTreeBuilder {
root: "/var/cache".into(),
size_getter: GetApparentSize,
hardlinks_recorder: & recorder,
reporter: & ErrorOnlyReporter::new(ErrorReport::SILENT),
device_boundary: DeviceBoundary::Cross,
max_depth: u64::MAX,
}
.into();

let record = recorder
.deduplicate( & mut data_tree)
.expect("deduplication is infallible");

println!("{} shared inodes", record.len());
```

### Ordering constraints

`deduplicate` matches recorded paths against node names by stripping the root's name as a prefix, so it must run after anything that renames the root and while the tree still contains the nodes holding the links. `pdu` runs build, cull, sort, deduplicate, then renames the synthetic root to
`(total)`. Follow the same order.

Deduplication only subtracts where at least two links to the same inode sit under a given directory. A link whose sibling lives outside the measured tree stays counted in full, since as far as the scan can tell the space is genuinely attributable to that directory.

## `HardlinkList`

`HardlinkList<Size>` is the storage behind `HardlinkAware`: a concurrent map from an inode key, being an inode number paired with a device number, to that file's size, total link count, and the paths where it was seen.

| Method                | Description                                                                                      |
|-----------------------|--------------------------------------------------------------------------------------------------|
| `new()`               | An empty list.                                                                                   |
| `len()`, `is_empty()` | Number of distinct shared inodes.                                                                |
| `iter()`              | Iterates the entries. Items expose `ino()`, `dev()`, `size()`, `links()`, and `paths()`.         |
| `into_reflection()`   | Converts into a comparable and serializable [`HardlinkListReflection`](#reflection-and-summary). |

The paths of one inode are held in a `LinkPathList`, offering `len()`, `is_empty()`, and
`into_reflection()`.

## Reflection and summary

As with `DataTree`, the concurrent structures implement neither `PartialEq` nor the `serde` traits. Convert them first.

* `HardlinkListReflection<Size>` is a `Vec` of `ReflectionEntry<Size>`, sorted by inode and device number, every pair unique. Each entry has public `ino`, `dev`, `size`, `links`, and `paths`
  fields. It offers `len()`, `is_empty()`, and `iter()`.
* `LinkPathListReflection` is the serializable form of a path list. Converting is O (n), since it turns a `Vec` into a `HashSet`.
* `SharedLinkSummary<Size>`, produced by the `SummarizeHardlinks` trait, aggregates a whole list.

`SharedLinkSummary` is `#[non_exhaustive]` and carries:

| Field                   | Meaning                                                    |
|-------------------------|------------------------------------------------------------|
| `inodes`                | Number of inodes with more than one link.                  |
| `exclusive_inodes`      | How many of those have no links outside the measured tree. |
| `all_links`             | Total link count across all shared inodes.                 |
| `detected_links`        | How many of those links were found inside the tree.        |
| `exclusive_links`       | Links belonging to exclusive inodes.                       |
| `shared_size`           | Total size of all shared inodes.                           |
| `exclusive_shared_size` | Total size of the exclusive ones.                          |

The "all" against "exclusive" distinction tells you whether deleting a directory would free the space, or whether another link elsewhere would keep it alive.

## Platform support

Hardlink detection needs `std::os::unix::fs::MetadataExt::nlink`, stable only on Unix. The Windows equivalent requires nightly, so `HardlinkAware` does not exist there.

The module provides aliases that resolve to the right type:

* `hardlink::record::Do<Size>` and `hardlink::deduplicate::Do<Size>` are `HardlinkAware<Size>`, Unix only.
* `hardlink::record::DoNot` and `hardlink::deduplicate::DoNot` are `HardlinkIgnorant`, available everywhere.

Code that must compile on Windows should use `DoNot` unconditionally or gate the aware path behind
`#[cfg(unix)]`.

# Library Overview

`parallel-disk-usage` ships both the `pdu` command-line program and a reusable Rust library named
`parallel_disk_usage`. This page describes the shape of the library. For the command-line program, see [CLI Usage](CLI-Usage.md).

The library measures disk usage of a filesystem tree in parallel, stores the result in an in-memory tree, and renders that tree as an ASCII chart or as JSON.

## The pipeline

Every use of the library follows the same three stages. Each stage is independent, so you may stop after any of them or substitute your own implementation.

```text
   ┌─────────────────────────────────────────────────────────────────────────┐
   │ 1. Measure                                                              │
   │    FsTreeBuilder  (real filesystem)                                     │
   │    TreeBuilder    (any hierarchical source)                             │
   └────────────────────────────────┬────────────────────────────────────────┘
                                    │  DataTree<Name, Size>
   ┌────────────────────────────────┴────────────────────────────────────────┐
   │ 2. Transform                                                            │
   │    par_sort_by, par_retain, par_cull_insignificant_data,                │
   │    DeduplicateSharedSize::deduplicate                                   │
   └────────────────────────────────┬────────────────────────────────────────┘
                                    │  DataTree<Name, Size>
   ┌────────────────────────────────┴────────────────────────────────────────┐
   │ 3. Render                                                               │
   │    Visualizer (ASCII chart)      DataTree::into_reflection (JSON)       │
   └─────────────────────────────────────────────────────────────────────────┘
```

Progress and errors are delivered during stage 1 by a [`Reporter`](Library-Reporters.md), so measurement never stops to print a message.

## Central types

| Type                                                  | Module                  | Role                                                                                     |
|-------------------------------------------------------|-------------------------|------------------------------------------------------------------------------------------|
| [`FsTreeBuilder`](Library-Building-Trees.md)             | `fs_tree_builder`       | Walks a real directory tree and produces a `DataTree`.                                   |
| [`TreeBuilder`](Library-Building-Trees.md)               | `tree_builder`          | Generic parallel tree construction from any source.                                      |
| [`DataTree`](Library-DataTree.md)                        | `data_tree`             | The measured tree.                                                                       |
| [`Reflection`](Library-DataTree.md#reflection)           | `data_tree::reflection` | Public-field mirror of `DataTree`, used for serialization and for construction in tests. |
| [`Visualizer`](Library-Visualizer.md)                    | `visualizer`            | Renders a `DataTree` into an ASCII chart.                                                |
| [`Reporter`](Library-Reporters.md)                       | `reporter`              | Receives progress and error events during measurement.                                   |
| [`GetSize`](Library-Sizes-And-Formatting.md#getsize)     | `get_size`              | Decides what "size" means for a given file.                                              |
| [`Size`](Library-Sizes-And-Formatting.md#the-size-trait) | `size`                  | Trait implemented by `Bytes` and `Blocks`.                                               |
| [`RecordHardlinks`](Library-Hardlinks.md)                | `hardlink`              | Detects hardlinks so their size is not counted twice.                                    |
| [`JsonData`](Library-JSON-Data.md)                       | `json_data`             | The schema of `--json-output` and `--json-input`.                                        |

## Generic parameters

The same three names appear throughout the source.

* `Name` is a node's label. `FsTreeBuilder` always produces
  [`OsStringDisplay`](Library-Module-Index.md#the-tree), which displays a valid UTF-8 path as text and falls back to the `Debug` form otherwise.
* `Size` is the unit of measurement, either `Bytes` or `Blocks`.
* `Report` is the progress sink, almost always taken by reference so it can be shared across Rayon worker threads.

## Parallelism

Measurement and most tree transformations use [Rayon](https://docs.rs/rayon). Parallel methods carry a `par_` prefix. Because their closures are handed to multiple threads, those closures must generally be `Copy + Sync`.

The library uses the ambient Rayon global pool and creates no pool of its own. To bound concurrency, install your own with
[`rayon::ThreadPoolBuilder`](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html) and run the measurement inside it.

## Next

* [Getting Started](Library-Getting-Started.md) for installation and a first program.
* [Feature Flags](Library-Feature-Flags.md) to trim the dependency tree.
* [Recipes](Library-Recipes.md) for task-oriented examples.

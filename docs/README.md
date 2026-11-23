# Documentation

`parallel-disk-usage` is a highly parallelized directory tree analyzer. It ships as the `pdu`
command-line program and as a reusable Rust library.

## Command-line program

* [CLI Usage](CLI-Usage.md) documents every flag and argument of `pdu`.

## Library

* [Library Overview](Library-Overview.md): the measure, transform, render pipeline and the central types.
* [Getting Started](Library-Getting-Started.md): installation and a first program.
* [Feature Flags](Library-Feature-Flags.md): what to enable, and what is platform-conditional.

### Core API

* [Building Trees](Library-Building-Trees.md): `FsTreeBuilder` and `TreeBuilder`.
* [DataTree](Library-DataTree.md): inspecting, sorting, filtering, and reflecting the measured tree.
* [Sizes and Formatting](Library-Sizes-And-Formatting.md): `Size`, `Bytes`, `Blocks`, `GetSize`, and
  `BytesFormat`.
* [Reporters](Library-Reporters.md): progress and error reporting during a scan.
* [Visualizer](Library-Visualizer.md): rendering the ASCII chart.
* [Hardlinks](Library-Hardlinks.md): detecting shared inodes and correcting for them.
* [JSON Data](Library-JSON-Data.md): the interchange format shared with `--json-output`.

### Reference

* [Recipes](Library-Recipes.md): examples for common tasks.
* [Module Index](Library-Module-Index.md): a map of the crate.

Generated API documentation is at
[docs.rs/parallel-disk-usage](https://docs.rs/parallel-disk-usage).

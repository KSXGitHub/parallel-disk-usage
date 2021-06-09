# Parallel Disk Usage (pdu)

[![Test](https://github.com/KSXGitHub/parallel-disk-usage/workflows/Test/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions?query=workflow%3ATest)
[![Benchmark](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/benchmark.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/benchmark.yaml)
[![Clippy](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/clippy.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/clippy.yaml)
[![Code formatting](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/fmt.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/fmt.yaml)
[![Lint dependency graph](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/cargo-deny.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/cargo-deny.yaml)
[![Crates.io Version](https://img.shields.io/crates/v/parallel-disk-usage?logo=rust)](https://crates.io/crates/parallel-disk-usage)

Highly parallelized, blazing fast directory tree analyzer.

## Description

`pdu` is a CLI program that renders a graphical chart for disk usages of files and directories, it is a replacement of [`dust`](https://github.com/bootandy/dust) and [`dutree`](https://github.com/nachoparker/dutree).

Both `dust` (v0.5.4) and `dutree` (v0.12.5) do not utilize the parallel computing capability of Rust to improve performance.

Furthermore, when providing multiple file names to `dust`, it shows bars of the exact same length regardless of size differences. This is the final push for the author of `pdu` to create this tool.

## Benchmark

The benchmark was generated by [a GitHub Workflow](https://github.com/KSXGitHub/parallel-disk-usage/blob/0.2.4/.github/workflows/deploy.yaml#L476-L658) and uploaded to the release page.

<details><summary>Programs</summary>

* `pdu` v0.4.0
* [`dust`](https://github.com/bootandy/dust) v0.5.4
* [`dutree`](https://github.com/nachoparker/dutree) v0.12.5
* [`dua`](https://github.com/Byron/dua-cli) v2.12.1
* [`ncdu`](https://dev.yorhel.nl/ncdu)
* [`gdu`](https://github.com/dundee/gdu) v5.0.0
* `du`

</details>

<figure>
  <img src="https://ksxgithub.github.io/parallel-disk-usage-0.4.0-benchmarks/tmp.benchmark-report.competing.blksize.svg">
  <figcaption align="center">
    benchmark results
    <em>(lower is better)</em>
  </figcaption>
</figure>

[_(See more)_](https://github.com/KSXGitHub/parallel-disk-usage-0.4.0-benchmarks/blob/master/tmp.benchmark-report.CHARTS.md)

## Demo

![screenshot](https://user-images.githubusercontent.com/11488886/120894518-04c41580-c643-11eb-9089-71822b543a2c.png)

[![asciicast of pdu command](https://asciinema.org/a/416663.svg)](https://asciinema.org/a/416663)

[![asciicast of pdu command on /usr](https://asciinema.org/a/416664.svg)](https://asciinema.org/a/416664)

## Features

* Fast.
* Relative comparison of separate files.
* Extensible via the library crate or JSON interface.
* Optional progress report.

## Limitations

* Ignorant of hard links: All hard links are counted as real files regardless of how many times it appear in the tree.
* Do not follow symbolic links.
* Do not differentiate filesystem: Mounted folders are counted as normal folders.
* The runtime is optimized at the expense of binary size.

## Development

### Prerequisites

* [`cargo`](github.com/rust-lang/cargo)

### Test

```sh
./test.sh && ./test.sh --release
```

<details><summary>
Environment Variables
</summary>

| name          | type              | default value | description                                     |
|---------------|-------------------|---------------|-------------------------------------------------|
| `FMT`         | `true` or `false` | `true`        | Whether to run `cargo fmt`                      |
| `LINT`        | `true` or `false` | `true`        | Whether to run `cargo clippy`                   |
| `DOC`         | `true` or `false` | `false`       | Whether to run `cargo doc`                      |
| `BUILD`       | `true` or `false` | `true`        | Whether to run `cargo build`                    |
| `TEST`        | `true` or `false` | `true`        | Whether to run `cargo test`                     |
| `BUILD_FLAGS` | string            | _(empty)_     | Space-separated list of flags for `cargo build` |
| `TEST_FLAGS`  | string            | _(empty)_     | Space-separated list of flags for `cargo test`  |

</details>

### Run

```sh
./run pdu "${arguments[@]}"
```

* `"${arguments[@]}"`: List of arguments to pass to `pdu`.

### Build

#### Debug build

```sh
cargo build --bin pdu
```

The resulting executable is located at `target/debug/pdu`.

#### Release build

```sh
cargo build --bin pdu --release
```

The resulting executable is located at `target/release/pdu`.

### Update shell completion files

```sh
./generate-completions.sh
```

## Extending `parallel-disk-usage`

The [parallel-disk-usage crate](https://crates.io/crates/parallel-disk-usage) is both a binary crate and a library crate. If you desire features that `pdu` itself lacks (that is, after you have asked the maintainer(s) of `pdu` for the features but they refused), you may use the library crate to build a tool of your own. The documentation for the library crate can be found in [docs.rs](https://docs.rs/parallel-disk-usage).

Alternatively, the `pdu` command provides `--json-input` flag and `--json-output` flag. The `--json-output` flag converts disk usage data into JSON and the `--json-input` flag turns said JSON into visualization. These 2 flags allow integration with other CLI tools (via pipe, as per the UNIX philosophy).

Beware that the structure of the JSON tree differs depends on the number of file/directory names that were provided (as CLI arguments):
* If there are only 0 or 1 file/directory names, the name of the tree root would be a real path (either `.` or the provided name).
* If there are 2 or more file/directory names, the name of the tree root would be `(total)` (which is not a real path), and the provided names would correspond to the children of the tree root.

## Installation

### Any Desktop OS

#### From GitHub

Go to the [GitHub Release Page](https://github.com/KSXGitHub/parallel-disk-usage/releases) and download a binary.

#### From [crates.io](https://crates.io)

**Prerequisites:**
  * [`cargo`](https://github.com/rust-lang/cargo)

```sh
cargo install parallel-disk-usage --bin pdu
```

### Arch Linux

#### From the [Arch User Repository](https://aur.archlinux.org)

**Prerequisites:**
  * An AUR helper, such as [`paru`](https://github.com/Morganamilo/paru)

```sh
paru -S parallel-disk-usage-bin
```

```sh
paru -S parallel-disk-usage
```

#### From [Khải's Pacman Repository](https://github.com/KSXGitHub/pacman-repo)

Follow the [installation instruction](https://github.com/KSXGitHub/pacman-repo#installation) then run the following command:

```sh
sudo pacman -S parallel-disk-usage
```

## Distributions

[![Packaging Status](https://repology.org/badge/vertical-allrepos/parallel-disk-usage.svg)](https://repology.org/project/parallel-disk-usage/versions)

## Similar programs

* **CLI:**
  * `du`
  * [`dust`](https://github.com/bootandy/dust)
  * [`dutree`](https://github.com/nachoparker/dutree)
  * [`dua`](https://github.com/byron/dua-cli)
* **TUI:**
  * [`ncdu`](https://dev.yorhel.nl/ncdu)
  * [`gdu`](https://github.com/dundee/gdu)
  * [`godu`](https://github.com/viktomas/godu)
* **GUI:**
  * [GNOME's Disk Usage Analyzer, a.k.a. `baobab`](https://wiki.gnome.org/action/show/Apps/DiskUsageAnalyzer)
  * Filelight

## License

[Apache 2.0](https://git.io/JGIAt) © [Hoàng Văn Khải](https://ksxgithub.github.io/).

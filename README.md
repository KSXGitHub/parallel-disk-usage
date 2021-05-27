# Parallel Disk Usage (pdu)

[![Test](https://github.com/KSXGitHub/parallel-disk-usage/workflows/Test/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions?query=workflow%3ATest)
[![Benchmark](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/benchmark.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/benchmark.yaml)
[![Clippy](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/clippy.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/clippy.yaml)
[![Code formatting](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/fmt.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/fmt.yaml)
[![Lint dependency graph](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/cargo-deny.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/cargo-deny.yaml)
[![Crates.io Version](https://img.shields.io/crates/v/parallel-disk-usage?logo=rust)](https://crates.io/crates/parallel-disk-usage)

Summarize disk usage of the set of files, recursively for directories.

## Description

`pdu` is a CLI program that renders a graphical chart for disk usages of files and directories, it is intended to replace [`dust`](https://github.com/bootandy/dust) and [`dutree`](https://github.com/nachoparker/dutree).

Both `dust` (v0.5.4) and `dutree` (v0.12.5) do not utilize the parallel computing capability of Rust to improve performance.

## Installation

### From GitHub

Go to the [GitHub Release Page](https://github.com/KSXGitHub/parallel-disk-usage/releases) and download a binary.

### From [crates.io](https://crates.io)

**Prerequisites:**
  * [`cargo`](https://github.com/rust-lang/cargo)

```sh
cargo install parallel-disk-usage
```

### From the [Arch User Repository](https://aur.archlinux.org)

**Prerequisites:**
  * An AUR helper, such as [`paru`](https://github.com/Morganamilo/paru)

```sh
paru -S parallel-disk-usage-bin
```

```sh
paru -S parallel-disk-usage
```

## Distributions

[![Packaging Status](https://repology.org/badge/vertical-allrepos/parallel-disk-usage.svg)](https://repology.org/project/parallel-disk-usage/versions)

## Similar programs

* **CLI:**
  * `du`
  * [`dust`](https://github.com/bootandy/dust)
  * [`dutree`](https://github.com/nachoparker/dutree)
* **TUI:**
  * [`ncdu`](https://dev.yorhel.nl/ncdu)
  * [`gdu`](https://github.com/dundee/gdu)
  * [`godu`](https://github.com/viktomas/godu)
* **GUI:**
  * [GNOME's Disk Usage Analyzer, a.k.a. `baobab`](https://wiki.gnome.org/action/show/Apps/DiskUsageAnalyzer)
  * Filelight

## License

[Apache 2.0](https://git.io/JGIAt) © [Hoàng Văn Khải](https://ksxgithub.github.io/).

# Parallel Disk Usage (pdu)

[![Test](https://github.com/KSXGitHub/parallel-disk-usage/workflows/Test/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions?query=workflow%3ATest)
[![Benchmark](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/benchmark.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/benchmark.yaml)
[![Clippy](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/clippy.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/clippy.yaml)
[![Code formatting](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/fmt.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/fmt.yaml)
[![Lint dependency graph](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/cargo-deny.yaml/badge.svg)](https://github.com/KSXGitHub/parallel-disk-usage/actions/workflows/cargo-deny.yaml)
[![Crates.io Version](https://img.shields.io/crates/v/parallel-disk-usage?logo=rust)](https://crates.io/crates/parallel-disk-usage)

Summarize disk usage of the set of files, recursively for directories.

## Description

`pdu` is a CLI program that renders a graphical chart for disk usages of files and directories, it is a replacement of [`dust`](https://github.com/bootandy/dust) and [`dutree`](https://github.com/nachoparker/dutree).

Both `dust` (v0.5.4) and `dutree` (v0.12.5) do not utilize the parallel computing capability of Rust to improve performance.

_Below is a benchmark sample of `pdu` (v0.0.0) against `dust`, `dutree`, and `du` (lower is better):_

<a href="https://github.com/KSXGitHub/parallel-disk-usage-0.0.0-benchmarks/blob/master/tmp.benchmark-report.CHARTS.md">
  <img
    alt="benchmark pdu v0.0.0 against dust, dutree, and du"
    src="https://ksxgithub.github.io/parallel-disk-usage-0.0.0-benchmarks/tmp.benchmark-report.competing.blksize.svg"
  >
</a>

## Demo

[![asciicast of pdu command](https://asciinema.org/a/416663.svg)](https://asciinema.org/a/416663)

[![asciicast of pdu command on /usr](https://asciinema.org/a/416664.svg)](https://asciinema.org/a/416664)

## Installation

### Any Desktop OS

#### From GitHub

Go to the [GitHub Release Page](https://github.com/KSXGitHub/parallel-disk-usage/releases) and download a binary.

#### From [crates.io](https://crates.io)

**Prerequisites:**
  * [`cargo`](https://github.com/rust-lang/cargo)

```sh
cargo install parallel-disk-usage
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
* **TUI:**
  * [`ncdu`](https://dev.yorhel.nl/ncdu)
  * [`gdu`](https://github.com/dundee/gdu)
  * [`godu`](https://github.com/viktomas/godu)
* **GUI:**
  * [GNOME's Disk Usage Analyzer, a.k.a. `baobab`](https://wiki.gnome.org/action/show/Apps/DiskUsageAnalyzer)
  * Filelight

## License

[Apache 2.0](https://git.io/JGIAt) © [Hoàng Văn Khải](https://ksxgithub.github.io/).

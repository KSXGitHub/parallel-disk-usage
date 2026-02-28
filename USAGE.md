# Usage

```sh
pdu [OPTIONS] [FILES]...
```

## Arguments

* `[FILES]...`: List of files and/or directories.

## Options

### `--json-input`

Read JSON data from stdin.

### `--json-output`

Print JSON data instead of an ASCII chart.

### `--bytes-format`

* _Aliases:_ `-b`.
* _Default:_ `metric`.
* _Choices:_
  - `plain`: Display plain number of bytes without units
  - `metric`: Use metric scale, i.e. 1K = 1000B, 1M = 1000K, and so on
  - `binary`: Use binary scale, i.e. 1K = 1024B, 1M = 1024K, and so on

How to display the numbers of bytes.

### `--deduplicate-hardlinks`

* _Aliases:_ `-H`, `--detect-links`, `--dedupe-links`.

Detect and subtract the sizes of hardlinks from their parent directory totals.

### `--top-down`

Print the tree top-down instead of bottom-up.

### `--align-right`

Set the root of the bars to the right.

### `--quantity`

* _Aliases:_ `-q`.
* _Default:_ `block-size`.
* _Choices:_
  - `apparent-size`: Measure apparent sizes
  - `block-size`: Measure block sizes (block-count * 512B)
  - `block-count`: Count numbers of blocks

Aspect of the files/directories to be measured.

### `--max-depth`

* _Aliases:_ `-d`, `--depth`.
* _Default:_ `10`.

Maximum depth to display the data. Could be either "inf" or a positive integer.

### `--total-width`

* _Aliases:_ `-w`, `--width`.

Width of the visualization.

### `--column-width`

Maximum widths of the tree column and width of the bar column.

### `--min-ratio`

* _Aliases:_ `-m`.
* _Default:_ `0.01`.

Minimal size proportion required to appear.

### `--no-sort`

Do not sort the branches in the tree.

### `--silent-errors`

* _Aliases:_ `-s`, `--no-errors`.

Prevent filesystem error messages from appearing in stderr.

### `--progress`

* _Aliases:_ `-p`.

Report progress being made at the expense of performance.

### `--threads`

* _Default:_ `auto`.

Set the maximum number of threads to spawn. Could be either "auto", "max", or a positive integer.

### `--omit-json-shared-details`

Do not output `.shared.details` in the JSON output.

### `--omit-json-shared-summary`

Do not output `.shared.summary` in the JSON output.

### `--help`

* _Aliases:_ `-h`.

Print help (see a summary with '-h').

### `--version`

* _Aliases:_ `-V`.

Print version.

## Examples

### Show disk usage chart of current working directory

```sh
pdu
```

### Show disk usage chart of a single file or directory

```sh
pdu path/to/file/or/directory
```

### Compare disk usages of multiple files and/or directories

```sh
pdu file.txt dir/
```

### Show chart in apparent sizes instead of block sizes

```sh
pdu --quantity=apparent-size
```

### Detect and subtract the sizes of hardlinks from their parent nodes

```sh
pdu --deduplicate-hardlinks
```

### Show sizes in plain numbers instead of metric units

```sh
pdu --bytes-format=plain
```

### Show sizes in base 2¹⁰ units (binary) instead of base 10³ units (metric)

```sh
pdu --bytes-format=binary
```

### Show disk usage chart of all entries regardless of size

```sh
pdu --min-ratio=0
```

### Only show disk usage chart of entries whose size is at least 5% of total

```sh
pdu --min-ratio=0.05
```

### Show disk usage data as JSON instead of chart

```sh
pdu --min-ratio=0 --max-depth=inf --json-output | jq
```

### Visualize existing JSON representation of disk usage data

```sh
pdu --json-input < disk-usage.json
```

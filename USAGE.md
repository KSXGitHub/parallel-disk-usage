# Usage

```sh
pdu [OPTIONS] [FILES]...
```

## Arguments

* `[FILES]...`: List of files and/or directories.

## Options

<a id="json-input" name="json-input"></a>
### `--json-input`

Read JSON data from stdin.

<a id="json-output" name="json-output"></a>
### `--json-output`

Print JSON data instead of an ASCII chart.

<a id="option-b" name="option-b"></a><a id="bytes-format" name="bytes-format"></a>
### `--bytes-format`

* _Aliases:_ `-b`.
* _Default:_ `metric`.
* _Choices:_
  - `plain`: Display plain number of bytes without units
  - `metric`: Use metric scale, i.e. 1K = 1000B, 1M = 1000K, and so on
  - `binary`: Use binary scale, i.e. 1K = 1024B, 1M = 1024K, and so on

How to display the numbers of bytes.

<a id="option-H" name="option-H"></a><a id="deduplicate-hardlinks" name="deduplicate-hardlinks"></a><a id="detect-links" name="detect-links"></a><a id="dedupe-links" name="dedupe-links"></a>
### `--deduplicate-hardlinks`

* _Aliases:_ `-H`, `--detect-links`, `--dedupe-links`.

Detect and subtract the sizes of hardlinks from their parent directory totals.

<a id="top-down" name="top-down"></a>
### `--top-down`

Print the tree top-down instead of bottom-up.

<a id="align-right" name="align-right"></a>
### `--align-right`

Set the root of the bars to the right.

<a id="option-q" name="option-q"></a><a id="quantity" name="quantity"></a>
### `--quantity`

* _Aliases:_ `-q`.
* _Default:_ `block-size`.
* _Choices:_
  - `apparent-size`: Measure apparent sizes
  - `block-size`: Measure block sizes (block-count * 512B)
  - `block-count`: Count numbers of blocks

Aspect of the files/directories to be measured.

<a id="option-d" name="option-d"></a><a id="max-depth" name="max-depth"></a><a id="depth" name="depth"></a>
### `--max-depth`

* _Aliases:_ `-d`, `--depth`.
* _Default:_ `10`.

Maximum depth to display the data. Could be either "inf" or a positive integer.

<a id="option-w" name="option-w"></a><a id="total-width" name="total-width"></a><a id="width" name="width"></a>
### `--total-width`

* _Aliases:_ `-w`, `--width`.

Width of the visualization.

<a id="column-width" name="column-width"></a>
### `--column-width`

Maximum widths of the tree column and width of the bar column.

<a id="option-m" name="option-m"></a><a id="min-ratio" name="min-ratio"></a>
### `--min-ratio`

* _Aliases:_ `-m`.
* _Default:_ `0.01`.

Minimal size proportion required to appear.

<a id="no-sort" name="no-sort"></a>
### `--no-sort`

Do not sort the branches in the tree.

<a id="option-s" name="option-s"></a><a id="silent-errors" name="silent-errors"></a><a id="no-errors" name="no-errors"></a>
### `--silent-errors`

* _Aliases:_ `-s`, `--no-errors`.

Prevent filesystem error messages from appearing in stderr.

<a id="option-p" name="option-p"></a><a id="progress" name="progress"></a>
### `--progress`

* _Aliases:_ `-p`.

Report progress being made at the expense of performance.

<a id="threads" name="threads"></a>
### `--threads`

* _Default:_ `auto`.

Set the maximum number of threads to spawn. Could be either "auto", "max", or a positive integer.

<a id="omit-json-shared-details" name="omit-json-shared-details"></a>
### `--omit-json-shared-details`

Do not output `.shared.details` in the JSON output.

<a id="omit-json-shared-summary" name="omit-json-shared-summary"></a>
### `--omit-json-shared-summary`

Do not output `.shared.summary` in the JSON output.

<a id="option-h" name="option-h"></a><a id="help" name="help"></a>
### `--help`

* _Aliases:_ `-h`.

Print help (see a summary with '-h').

<a id="option-V" name="option-V"></a><a id="version" name="version"></a>
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

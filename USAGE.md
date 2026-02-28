# Usage

```sh
pdu [OPTIONS] [FILES]...
```

## Arguments

* `[FILES]...`: List of files and/or directories

## Options

* `--json-input`: Read JSON data from stdin
* `--json-output`: Print JSON data instead of an ASCII chart
* `-b <BYTES_FORMAT>`, `--bytes-format <BYTES_FORMAT>`: How to display the numbers of bytes (default: `metric`)
  * `plain`: Display plain number of bytes without units
  * `metric`: Use metric scale, i.e. 1K = 1000B, 1M = 1000K, and so on
  * `binary`: Use binary scale, i.e. 1K = 1024B, 1M = 1024K, and so on
* `-H`, `--deduplicate-hardlinks`, `--detect-links`, `--dedupe-links`: Detect and subtract the sizes of hardlinks from their parent directory totals
* `--top-down`: Print the tree top-down instead of bottom-up
* `--align-right`: Set the root of the bars to the right
* `-q <QUANTITY>`, `--quantity <QUANTITY>`: Aspect of the files/directories to be measured (default: `block-size`)
  * `apparent-size`: Measure apparent sizes
  * `block-size`: Measure block sizes (block-count * 512B)
  * `block-count`: Count numbers of blocks
* `-d <MAX_DEPTH>`, `--max-depth <MAX_DEPTH>`, `--depth <MAX_DEPTH>`: Maximum depth to display the data. Could be either "inf" or a positive integer (default: `10`)
* `-w <TOTAL_WIDTH>`, `--total-width <TOTAL_WIDTH>`, `--width <TOTAL_WIDTH>`: Width of the visualization
* `--column-width <TREE_WIDTH> <BAR_WIDTH>`: Maximum widths of the tree column and width of the bar column
* `-m <MIN_RATIO>`, `--min-ratio <MIN_RATIO>`: Minimal size proportion required to appear (default: `0.01`)
* `--no-sort`: Do not sort the branches in the tree
* `-s`, `--silent-errors`, `--no-errors`: Prevent filesystem error messages from appearing in stderr
* `-p`, `--progress`: Report progress being made at the expense of performance
* `--threads <THREADS>`: Set the maximum number of threads to spawn. Could be either "auto", "max", or a positive integer (default: `auto`)
* `--omit-json-shared-details`: Do not output `.shared.details` in the JSON output
* `--omit-json-shared-summary`: Do not output `.shared.summary` in the JSON output
* `-h`, `--help`: Print help (see a summary with '-h')
* `-V`, `--version`: Print version

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

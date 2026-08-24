# Visualizer

`visualizer::Visualizer` renders a [`DataTree`](Library-DataTree.md) as the ASCII chart `pdu` prints. It is a struct of parameters whose work is performed by `Display`.

```rust
use parallel_disk_usage::bytes_format::BytesFormat;
use parallel_disk_usage::visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer};

let visualizer = Visualizer {
data_tree: & data_tree,
bytes_format: BytesFormat::MetricUnits,
direction: Direction::BottomUp,
bar_alignment: BarAlignment::Right,
column_width_distribution: ColumnWidthDistribution::total(100),
};

println!("{visualizer}");
```

## Fields

| Field                       | Type                      | Meaning                                                               |
|-----------------------------|---------------------------|-----------------------------------------------------------------------|
| `data_tree`                 | `&DataTree<Name, Size>`   | The tree to render.                                                   |
| `bytes_format`              | `Size::DisplayFormat`     | How sizes are rendered. `BytesFormat` for `Bytes`, `()` for `Blocks`. |
| `direction`                 | `Direction`               | Whether the root sits at the bottom or the top.                       |
| `bar_alignment`             | `BarAlignment`            | Which side the proportion bars fill from.                             |
| `column_width_distribution` | `ColumnWidthDistribution` | How horizontal space is allocated.                                    |

`Name` must implement `Display` and `Size` must implement `Into<u64>`, which both built-in units do. `Visualizer` is `Copy`.

## Two ways to render

* `Display`, that is `to_string()` or `{}`, produces the whole chart with a trailing newline on each line, ordered by `direction`.
* `rows()` returns `Vec<String>`, one entry per line, always top-down regardless of `direction`. Use it to post-process, paginate, or colourize.

```rust
for row in visualizer.rows() {
println ! ("{row}");
}
```

`rows()` may lay the chart out more than once while converging on a column allocation that fits. Render once and reuse the result.

## `Direction`

| Variant    | Effect                                                                                                |
|------------|-------------------------------------------------------------------------------------------------------|
| `BottomUp` | The root is the last line. The default of `pdu`, chosen so the root ends up next to the shell prompt. |
| `TopDown`  | The root is the first line, matching `--top-down`.                                                    |

## `BarAlignment`

| Variant | Effect                                              |
|---------|-----------------------------------------------------|
| `Left`  | Bars fill from the left.                            |
| `Right` | Bars fill from the right, matching `--align-right`. |

## `ColumnWidthDistribution`

Controls how many characters the chart may occupy and how they are split between the tree column and the bar column. Both constructors are `const fn`.

```rust
use parallel_disk_usage::visualizer::ColumnWidthDistribution;

// Total budget. The visualizer decides the split.
let width = ColumnWidthDistribution::total(100);

// Explicit split: at most 60 columns of tree, exactly 30 columns of bar.
let width = ColumnWidthDistribution::components(60, 30);
```

`total(width)` corresponds to `--total-width` and suits a terminal of known width. The visualizer works out the split and falls back to a minimum layout when the budget is too small for the names it must print, so a small total does not truncate the chart into unreadability.

`components(tree_column_max_width, bar_column_width)` corresponds to `--column-width` and pins both sides. The first value is a maximum; the second is exact.

To match the terminal, read its width with [`terminal_size`](https://docs.rs/terminal_size), which this crate already depends on for the CLI.

## Rendering components

The submodules that build the chart are public and available for custom renderers.

* `proportion_bar::ProportionBar` holds five counts, `level0` through `level4`, the number of blocks at each shading level. `display(alignment)` turns it into a `Display` value. The block characters are the `ProportionBarBlock` constants `LEVEL0_BLOCK` (`█`) through `LEVEL4_BLOCK`
  (a space).
* `tree::TreeSkeletalComponent` renders one cell of the branch drawing;
  `tree::TreeHorizontalSlice` is a horizontal span of it.
* `child_position::ChildPosition` distinguishes the last child from the rest, deciding whether the branch character is a corner or a tee. Build it with `ChildPosition::from_index(index, count)`.
* `parenthood::Parenthood` distinguishes a node with children from a leaf. Build it with
  `Parenthood::from_children_count(count)`.

For most purposes, walking the `DataTree` yourself and formatting sizes with
[`Size::display`](Library-Sizes-And-Formatting.md) is simpler than reusing these pieces.

## Ordering

The visualizer renders children in tree order and performs no sorting. Sort the tree first, as in
[DataTree](Library-DataTree.md#sorting), or the chart shows entries in filesystem order.

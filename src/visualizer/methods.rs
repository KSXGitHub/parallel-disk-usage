mod bar_table;
mod constants;
mod initial_table;
mod node_info;
mod table;
mod tree_table;

use bar_table::*;
use constants::*;
use initial_table::*;
use node_info::*;
use table::*;
use tree_table::*;

use super::{ChildPosition, Color, ColumnWidthDistribution, TreeHorizontalSlice, Visualizer};
use crate::size;
use lscolors::{Indicator, LsColors};
use std::{
    cmp::min,
    fmt::{self, Display},
    hash::Hash,
    sync::LazyLock,
};
use zero_copy_pads::{align_left, align_right, Width};

struct AnsiPrefixes {
    directory: String,
    normal: String,
    executable: String,
    symlink: String,
}

static ANSI_PREFIXES: LazyLock<AnsiPrefixes> = LazyLock::new(|| {
    let ls_colors = LsColors::from_env().unwrap_or_default();
    let compute = |indicator: Indicator| {
        ls_colors
            .style_for_indicator(indicator)
            .map(|s| s.to_nu_ansi_term_style().prefix().to_string())
            .unwrap_or_default()
    };
    AnsiPrefixes {
        directory: compute(Indicator::Directory),
        normal: compute(Indicator::RegularFile),
        executable: compute(Indicator::ExecutableFile),
        symlink: compute(Indicator::SymbolicLink),
    }
});

fn color_ansi_prefix(color: Color) -> &'static str {
    match color {
        Color::Directory => &ANSI_PREFIXES.directory,
        Color::Normal => &ANSI_PREFIXES.normal,
        Color::Executable => &ANSI_PREFIXES.executable,
        Color::Symlink => &ANSI_PREFIXES.symlink,
    }
}

struct ColoredSlice<'a> {
    slice: &'a TreeHorizontalSlice<String>,
    color: Color,
}

impl fmt::Display for ColoredSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let TreeHorizontalSlice {
            ancestor_relative_positions,
            skeletal_component,
            name,
        } = self.slice;
        for pos in ancestor_relative_positions {
            let connector = match pos {
                ChildPosition::Init => "│ ",
                ChildPosition::Last => "  ",
            };
            write!(f, "{connector}")?;
        }
        let prefix = color_ansi_prefix(self.color);
        let suffix = if prefix.is_empty() { "" } else { "\x1b[0m" };
        write!(f, "{skeletal_component}{prefix}{name}{suffix}")
    }
}

impl Width for ColoredSlice<'_> {
    fn width(&self) -> usize {
        self.slice.width()
    }
}

impl<'a, Name, Size> Visualizer<'a, Name, Size>
where
    Name: Display + Hash + Eq,
    Size: size::Size + Into<u64>,
{
    /// Create ASCII rows that visualize the [tree](crate::data_tree::DataTree), such rows
    /// are meant to be printed to a terminal screen.
    pub fn rows(mut self) -> Vec<String> {
        let initial_table = render_initial(self);
        let min_width = initial_table.column_width.total_max_width();

        let (tree_table, bar_width) = match self.column_width_distribution {
            ColumnWidthDistribution::Total { width } => {
                let extra_cols = 3; // make space for tree_column to minimize second-time re-rendering.

                if width <= min_width {
                    self.column_width_distribution
                        .set_components(min_width, extra_cols);
                    return self.rows();
                }

                if width <= MIN_OVERALL_WIDTH {
                    self.column_width_distribution
                        .set_components(min_width, MIN_OVERALL_WIDTH + extra_cols);
                    return self.rows();
                }

                let tree_max_width = min(width - min_width, width - MIN_OVERALL_WIDTH);
                let tree_table = render_tree(self, initial_table, tree_max_width);

                let min_width = tree_table.column_width.total_max_width();
                if width <= min_width {
                    self.column_width_distribution.set_components(min_width, 1);
                    return self.rows();
                }

                let bar_width = width - min_width;

                (tree_table, bar_width)
            }

            ColumnWidthDistribution::Components {
                tree_column_max_width,
                bar_column_width,
            } => {
                if bar_column_width < 1 {
                    self.column_width_distribution
                        .set_components(tree_column_max_width, 1);
                    return self.rows();
                }

                let tree_table = render_tree(self, initial_table, tree_column_max_width);

                (tree_table, bar_column_width)
            }
        };

        let size_width = tree_table.column_width.size_column_width;
        let tree_width = tree_table.column_width.tree_column_width;

        let bar_table = render_bars(tree_table, self.data_tree.size().into(), bar_width);

        bar_table
            .into_iter()
            .map(|row| {
                let BarRow { tree_row, proportion_bar } = row;
                let TreeRow { initial_row, tree_horizontal_slice: slice } = tree_row;

                let node_color = self.coloring.and_then(|coloring| {
                    if initial_row.node_info.children_count > 0 {
                        Some(Color::Directory)
                    } else {
                        coloring.get(initial_row.node_info.name).copied()
                    }
                });

                let aligned_colored;
                let aligned_normal;
                let tree = if let Some(color) = node_color {
                    aligned_colored = align_left(ColoredSlice { slice: &slice, color }, tree_width);
                    format_args!("{aligned_colored}")
                } else {
                    aligned_normal = align_left(&slice, tree_width);
                    format_args!("{aligned_normal}")
                };

                format!(
                    "{size} {tree}│{bar}│{ratio}",
                    size = align_right(&initial_row.size, size_width),
                    bar = proportion_bar.display(self.bar_alignment),
                    ratio = align_right(&initial_row.percentage, PERCENTAGE_COLUMN_MAX_WIDTH),
                )
            })
            .collect()
    }
}

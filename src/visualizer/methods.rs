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

use super::{ColumnWidthDistribution, Visualizer};
use crate::size::Size;
use std::{cmp::min, fmt::Display};
use zero_copy_pads::{align_left, align_right};

impl<'a, Name, Data> Visualizer<'a, Name, Data>
where
    Name: Display,
    Data: Size + Into<u64>,
{
    /// Create ASCII visualization of the [tree](Tree), such visualization is meant to be
    /// printed to a terminal screen.
    pub fn visualize(mut self) -> Vec<String> {
        let initial_table = render_initial(self);
        let min_width = initial_table.column_width.total_max_width();

        let (tree_table, bar_width) = match self.column_width_distribution {
            ColumnWidthDistribution::Total { max_width } => {
                let extra_cols = 3; // make space for tree_column to minimize second-time re-rendering.

                if max_width <= min_width {
                    self.column_width_distribution
                        .set_components(min_width, extra_cols);
                    return self.visualize();
                }

                if max_width <= MIN_OVERALL_WIDTH {
                    self.column_width_distribution
                        .set_components(min_width, MIN_OVERALL_WIDTH + extra_cols);
                    return self.visualize();
                }

                let tree_max_width = min(max_width - min_width, max_width - MIN_OVERALL_WIDTH);
                let tree_table = render_tree(self, initial_table, tree_max_width);

                let min_width = tree_table.column_width.total_max_width();
                if max_width <= min_width {
                    self.column_width_distribution.set_components(min_width, 1);
                    return self.visualize();
                }

                let bar_width = max_width - min_width;

                (tree_table, bar_width)
            }

            ColumnWidthDistribution::Components {
                tree_column_max_width,
                bar_column_width,
            } => {
                if bar_column_width < 1 {
                    self.column_width_distribution
                        .set_components(tree_column_max_width, 1);
                    return self.visualize();
                }

                let tree_table = render_tree(self, initial_table, tree_column_max_width);

                (tree_table, bar_column_width)
            }
        };

        let size_width = tree_table.column_width.size_column_width;
        let tree_width = tree_table.column_width.tree_column_width;

        let bar_table = render_bars(tree_table, self.tree.data().into(), bar_width);

        bar_table
            .into_iter()
            .map(|row| {
                format!(
                    "{size} {tree}│{bar}│{ratio}",
                    size = align_right(&row.size, size_width),
                    tree = align_left(&row.tree_horizontal_slice, tree_width),
                    bar = &row.proportion_bar,
                    ratio = align_right(&row.percentage, PERCENTAGE_COLUMN_MAX_WIDTH),
                )
            })
            .collect()
    }
}

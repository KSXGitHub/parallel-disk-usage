/// Specify distribution and maximum number of characters/blocks can be placed
/// in a line.
#[derive(Debug, Clone, Copy)]
pub enum ColumnWidthDistribution {
    /// Specify maximum number of characters/blocks can be placed in a line.
    Total {
        /// Maximum number of characters/blocks can be placed in a line.
        max_width: usize,
    },
    /// Specify maximum number of characters/blocks can be placed in a line
    /// for each individual component of the visualization.
    Components {
        /// Maximum number of characters/blocks can be placed in a line
        /// for the filesystem tree visualization.
        tree_column_max_width: usize,
        /// Number of characters/blocks can be placed in a line
        /// for the proportion bar.
        bar_column_width: usize,
    },
}

pub use ColumnWidthDistribution::*;

impl ColumnWidthDistribution {
    /// Specify maximum number of characters/blocks can be placed in a line.
    #[inline]
    pub const fn total(max_width: usize) -> Self {
        Total { max_width }
    }

    /// Specify maximum number of characters/blocks can be placed in a line
    /// for each individual component of the visualization.
    #[inline]
    pub const fn components(tree_column_max_width: usize, bar_column_width: usize) -> Self {
        Components {
            tree_column_max_width,
            bar_column_width,
        }
    }

    /// Deduce maximum number of characters/blocks can be placed in a line.
    pub const fn max_width(self) -> usize {
        match self {
            Total { max_width } => max_width,
            Components {
                tree_column_max_width,
                bar_column_width,
            } => tree_column_max_width + bar_column_width,
        }
    }

    pub(super) fn set(&mut self, new_tree_column_max_width: usize, new_bar_column_width: usize) {
        match self {
            ColumnWidthDistribution::Total { max_width } => {
                *max_width = new_tree_column_max_width + new_bar_column_width;
            }
            ColumnWidthDistribution::Components {
                tree_column_max_width,
                bar_column_width: bar_column_max_width,
            } => {
                *tree_column_max_width = new_tree_column_max_width;
                *bar_column_max_width = new_bar_column_width;
            }
        }
    }
}

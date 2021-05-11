/// Specify distribution and total number of characters/blocks can be placed
/// in a line.
#[derive(Debug, Clone, Copy)]
pub enum ColumnWidthDistribution {
    /// Specify total number of characters/blocks can be placed in a line.
    Total {
        /// Total number of characters/blocks can be placed in a line.
        width: usize,
    },
    /// Specify (maximum) number of characters/blocks can be placed in a line
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
    /// Specify total number of characters/blocks can be placed in a line.
    #[inline]
    pub const fn total(width: usize) -> Self {
        Total { width }
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

    pub(super) fn set_components(&mut self, tree_column_max_width: usize, bar_column_width: usize) {
        *self = Self::components(tree_column_max_width, bar_column_width);
    }
}

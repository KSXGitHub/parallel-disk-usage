use strum::{AsRefStr, EnumString, EnumVariantNames};

/// The direction of the visualization of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, EnumString, EnumVariantNames)]
#[strum(serialize_all = "kebab-case")]
pub enum Direction {
    /// The root of the tree is placed at the bottom of the visualization.
    BottomUp,
    /// The root of the tree is placed at the top of the visualization.
    TopDown,
}

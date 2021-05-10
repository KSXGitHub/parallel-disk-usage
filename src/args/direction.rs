use crate::visualizer::Direction as Inner;
use derive_more::{FromStr, Into};
use strum::VariantNames;

/// The direction of the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromStr, Into)]
pub struct Direction(Inner);

/// Possible CLI values of [`Direction`].
pub const DIRECTION_VALUES: &[&str] = Inner::VARIANTS;

impl Direction {
    pub(super) fn default_value() -> &'static str {
        Inner::BottomUp.as_ref()
    }
}

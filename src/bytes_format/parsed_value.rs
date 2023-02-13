use derive_more::{Display, Error};

/// Return value of [`Formatter::parse_value`](super::Formatter::parse_value).
#[derive(Debug, Display, Clone, Copy, Error)]
pub enum ParsedValue {
    /// When input value is less than `scale_base`.
    #[display(fmt = "{value}   ")]
    Small {
        /// Input value that is less than `scale_base`.
        value: u16,
    },
    /// When input value is greater than `scale_base`.
    #[display(fmt = "{coefficient:.1}{unit}")]
    Big {
        /// The visible part of the number.
        coefficient: f32,
        /// The unit that follows `coefficient`.
        unit: char,
        /// The divisor that was used upon the original number to get `coefficient`.
        scale: u64,
        /// The exponent that was used upon base scale to get `scale`.
        exponent: usize,
    },
}

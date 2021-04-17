use derive_more::Display;

/// Return value of [`UnitPrefix::parse`](UnitPrefix).
#[derive(Debug, Display, Clone, Copy)]
#[display(fmt = "{}{}", coefficient, unit)]
pub struct ParsedValue {
    pub(super) coefficient: u64,
    pub(super) unit: char,
    pub(super) scale: u64,
    pub(super) exponent: usize,
}

macro_rules! parsed_value_getter {
    ($(#[$attributes:meta])* $field:ident: $result:ty) => {
        $(#[$attributes])*
        pub const fn $field(self) -> $result {
            self.$field
        }
    };
}

impl ParsedValue {
    parsed_value_getter!(
        #[doc = "The visible part of the number."]
        coefficient: u64
    );
    parsed_value_getter!(
        #[doc = "The unit that follows `coefficient`."]
        unit: char
    );
    parsed_value_getter!(
        #[doc = "The divisor that was used upon the original number to get `coefficient`."]
        scale: u64
    );
    parsed_value_getter!(
        #[doc = "The exponent that was used upon base scale to get `scale`."]
        exponent: usize
    );
}

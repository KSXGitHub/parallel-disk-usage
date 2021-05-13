use super::{scale_base, ParsedValue};
use std::fmt::Debug;

/// Format a quantity of bytes.
#[derive(Debug, Clone, Copy)]
pub struct Formatter {
    scale_base: u64,
}

impl Formatter {
    /// Create a new formatter.
    pub const fn new(scale_base: u64) -> Self {
        Formatter { scale_base }
    }

    /// Multiplication factor.
    pub const fn scale_base(self) -> u64 {
        self.scale_base
    }

    /// Get scale in number.
    pub const fn scale(self, exp: u32) -> u64 {
        self.scale_base().pow(exp)
    }

    /// Parse a value according to the prefixing rule.
    pub const fn parse_value(self, value: u64) -> ParsedValue {
        macro_rules! check {
            ($exp:literal => $unit:literal) => {{
                let scale = self.scale($exp);
                if value >= scale {
                    return ParsedValue {
                        coefficient: rounded_div::u64(value, scale),
                        unit: $unit,
                        exponent: $exp,
                        scale,
                    };
                }
            }};
        }

        check!(5 => 'P');
        check!(4 => 'T');
        check!(3 => 'G');
        check!(2 => 'M');
        check!(1 => 'K');
        ParsedValue {
            coefficient: value,
            unit: 'B',
            scale: 1,
            exponent: 0,
        }
    }
}

macro_rules! variant {
    ($(#[$attributes:meta])* $name:ident) => {
        $(#[$attributes])*
        pub const $name: Formatter = Formatter::new(scale_base::$name);
    };
}

variant! {
    #[doc = "Format a quantity of bytes in [metric system](scale_base::METRIC)."]
    METRIC
}

variant! {
    #[doc = "Format a quantity of bytes in [binary system](scale_base::BINARY)."]
    BINARY
}

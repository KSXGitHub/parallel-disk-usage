use std::fmt::Debug;

pub mod parsed_value;
pub mod scale_base;

pub use parsed_value::ParsedValue;
pub use scale_base::{BINARY as BINARY_SCALE_BASE, METRIC as METRIC_SCALE_BASE};

/// Unit prefix to count bytes.
pub trait MeasurementSystem: Debug + Default + Clone + Copy {
    /// Multiplication factor of metric system
    const SCALE_BASE: u64;

    /// Get scale in number.
    fn scale(exp: u32) -> u64 {
        Self::SCALE_BASE.pow(exp)
    }

    /// Parse a value according to the prefixing rule.
    fn parse_value(value: u64) -> ParsedValue {
        macro_rules! check {
            ($exp:literal => $unit:literal) => {{
                let scale = Self::scale($exp);
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
    (
        $(#[$attributes:meta])*
        $name:ident = $scale_base:expr;
    ) => {
        $(#[$attributes])*
        #[derive(Debug, Default, Clone, Copy)]
        pub struct $name;

        impl MeasurementSystem for $name {
            const SCALE_BASE: u64 = $scale_base;
        }
    };
}

variant! {
    #[doc = "Use the metric system"]
    Metric = METRIC_SCALE_BASE;
}

variant! {
    #[doc = "Use the binary system"]
    Binary = BINARY_SCALE_BASE;
}

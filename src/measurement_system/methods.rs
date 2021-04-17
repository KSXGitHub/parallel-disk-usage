use super::{MeasurementSystem, ParsedValue, BINARY_SCALE_BASE, METRIC_SCALE_BASE};

impl MeasurementSystem {
    /// Get multiplication factor in number.
    pub const fn scale_base(self) -> u64 {
        match self {
            MeasurementSystem::Metric => METRIC_SCALE_BASE,
            MeasurementSystem::Binary => BINARY_SCALE_BASE,
        }
    }

    /// Get scale in number.
    pub const fn scale(self, exp: usize) -> u64 {
        self.scale_base().pow(exp as u32)
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

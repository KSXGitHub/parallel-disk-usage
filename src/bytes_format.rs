pub mod formatter;
pub mod output;
pub mod parsed_value;
pub mod scale_base;

pub use formatter::Formatter;
pub use output::Output;
pub use parsed_value::ParsedValue;

use pipe_trait::Pipe;

#[cfg(feature = "cli")]
use clap::ValueEnum;

/// The [`DisplayFormat`](crate::size::Size::DisplayFormat) type of [`Bytes`](crate::size::Bytes).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum BytesFormat {
    /// Display the value as-is.
    #[cfg_attr(
        feature = "cli",
        clap(name = "plain", help = "Display plain number of bytes without units")
    )]
    PlainNumber,
    /// Display the value with a unit suffix in [metric scale](formatter::METRIC).
    #[cfg_attr(
        feature = "cli",
        clap(
            name = "metric",
            help = "Use metric scale, i.e. 1K = 1000B, 1M = 1000K, and so on",
        )
    )]
    MetricUnits,
    /// Display the value with a unit suffix in [binary scale](formatter::BINARY).
    #[cfg_attr(
        feature = "cli",
        clap(
            name = "binary",
            help = "Use binary scale, i.e. 1K = 1024B, 1M = 1024K, and so on",
        )
    )]
    BinaryUnits,
}

impl BytesFormat {
    /// Format a quantity of bytes according to the settings.
    pub fn format(self, bytes: u64) -> Output {
        use formatter::{BINARY, METRIC};
        use BytesFormat::*;
        match self {
            PlainNumber => Output::PlainNumber(bytes),
            MetricUnits => METRIC.parse_value(bytes).pipe(Output::Units),
            BinaryUnits => BINARY.parse_value(bytes).pipe(Output::Units),
        }
    }
}

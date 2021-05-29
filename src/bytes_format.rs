pub mod formatter;
pub mod output;
pub mod parsed_value;
pub mod scale_base;

pub use formatter::Formatter;
pub use output::Output;
pub use parsed_value::ParsedValue;

use pipe_trait::Pipe;
use strum::{AsRefStr, EnumString, EnumVariantNames};

/// The [`DisplayFormat`](crate::size::Size::DisplayFormat) type of [`Bytes`](crate::size::Bytes).
#[derive(Debug, Clone, Copy, AsRefStr, EnumString, EnumVariantNames)]
pub enum BytesFormat {
    /// Display the value as-is.
    #[strum(serialize = "plain")]
    PlainNumber,
    /// Display the value with a unit suffix in [metric scale](formatter::METRIC).
    #[strum(serialize = "metric")]
    MetricUnits,
    /// Display the value with a unit suffix in [binary scale](formatter::BINARY).
    #[strum(serialize = "binary")]
    BinaryUnits,
}

impl BytesFormat {
    #[cfg(feature = "cli")]
    pub(crate) fn default_value() -> &'static str {
        BytesFormat::MetricUnits.as_ref()
    }

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

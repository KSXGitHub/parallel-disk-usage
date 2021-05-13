use strum::{AsRefStr, EnumString, EnumVariantNames};

/// The [`DisplayFormat`](Size::DisplayFormat) type of [`Bytes`].
#[derive(Debug, Clone, Copy, AsRefStr, EnumString, EnumVariantNames)]
pub enum BytesFormat {
    /// Display the value as-is.
    #[strum(serialize = "plain")]
    PlainNumber,
    /// Display the value with a unit suffix in [metric scale](Metric).
    #[strum(serialize = "metric")]
    MetricUnits,
    /// Display the value with a unit suffix in [binary scale](Binary).
    #[strum(serialize = "binary")]
    BinaryUnits,
}

impl BytesFormat {
    pub(crate) fn default_value() -> &'static str {
        BytesFormat::MetricUnits.as_ref()
    }
}

/// The [`DisplayFormat`](Size::DisplayFormat) type of [`Bytes`].
#[derive(Debug, Clone, Copy)]
pub enum BytesDisplayFormat {
    /// Display the value as-is.
    PlainNumber,
    /// Display the value with a unit suffix in [metric scale](Metric).
    MetricUnits,
    /// Display the value with a unit suffix in [binary scale](Binary).
    BinaryUnits,
}

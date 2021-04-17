pub mod parsed_value;
pub mod scale_base;

pub use parsed_value::ParsedValue;
pub use scale_base::{BINARY as BINARY_SCALE_BASE, METRIC as METRIC_SCALE_BASE};

/// Unit prefix to count bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementSystem {
    /// Use the metric system.
    Metric,
    /// Use the binary system.
    Binary,
}

mod methods;

#[cfg(test)]
mod test;

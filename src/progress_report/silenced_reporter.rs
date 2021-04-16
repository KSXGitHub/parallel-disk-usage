use super::{Event, ProgressReport, Size};
use crate::error_report::ErrorReport;

/// Only report errors.
#[derive(Debug)]
pub struct SilencedReporter<ReportError: Fn(ErrorReport)> {
    /// Report encountered errors.
    pub report_error: ReportError,
}

impl<ReportError: Fn(ErrorReport)> SilencedReporter<ReportError> {
    /// Create a new [`SilencedReporter`].
    pub fn new(report_error: ReportError) -> Self {
        SilencedReporter { report_error }
    }
}

impl<Data, ReportError> ProgressReport<Data> for SilencedReporter<ReportError>
where
    Data: Size,
    ReportError: Fn(ErrorReport),
{
    fn report(&self, event: Event<Data>) {
        let SilencedReporter { report_error } = self;
        if let Event::EncounterError(error_report) = event {
            report_error(error_report);
        }
    }
}

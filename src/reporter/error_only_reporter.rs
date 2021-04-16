use super::{ErrorReport, Event, Reporter, Size};

/// Only report errors.
#[derive(Debug)]
pub struct ErrorOnlyReporter<ReportError: Fn(ErrorReport)> {
    /// Report encountered errors.
    pub report_error: ReportError,
}

impl<ReportError: Fn(ErrorReport)> ErrorOnlyReporter<ReportError> {
    /// Create a new [`ErrorOnlyReporter`].
    pub fn new(report_error: ReportError) -> Self {
        ErrorOnlyReporter { report_error }
    }
}

impl<Data, ReportError> Reporter<Data> for ErrorOnlyReporter<ReportError>
where
    Data: Size,
    ReportError: Fn(ErrorReport),
{
    fn report(&self, event: Event<Data>) {
        let ErrorOnlyReporter { report_error } = self;
        if let Event::EncounterError(error_report) = event {
            report_error(error_report);
        }
    }
}

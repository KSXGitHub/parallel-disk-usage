use super::{ErrorReport, Event, ParallelReporter, Reporter};
use crate::size;

/// Only report errors.
#[derive(Debug)]
pub struct ErrorOnlyReporter<ReportError: Fn(ErrorReport)> {
    /// Report encountered errors.
    report_error: ReportError,
}

impl<ReportError: Fn(ErrorReport)> ErrorOnlyReporter<ReportError> {
    /// Create a new [`ErrorOnlyReporter`].
    pub fn new(report_error: ReportError) -> Self {
        ErrorOnlyReporter { report_error }
    }
}

impl<Size, ReportError> Reporter<Size> for ErrorOnlyReporter<ReportError>
where
    Size: size::Size,
    ReportError: Fn(ErrorReport),
{
    fn report(&self, event: Event<Size>) {
        let ErrorOnlyReporter { report_error } = self;
        if let Event::EncounterError(error_report) = event {
            report_error(error_report);
        }
    }
}

impl<Size, ReportError> ParallelReporter<Size> for ErrorOnlyReporter<ReportError>
where
    Size: size::Size,
    ReportError: Fn(ErrorReport),
{
    type DestructionError = (); // TODO: change this to `!` once it is stable.
    fn destroy(self) -> Result<(), Self::DestructionError> {
        Ok(())
    }
}

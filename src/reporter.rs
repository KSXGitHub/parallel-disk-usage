pub mod error_only_reporter;
pub mod error_report;
pub mod event;
pub mod progress_and_error_reporter;
pub mod progress_report;

pub use error_only_reporter::ErrorOnlyReporter;
pub use error_report::ErrorReport;
pub use event::Event;
pub use progress_and_error_reporter::ProgressAndErrorReporter;
pub use progress_report::ProgressReport;

use crate::size::Size;

/// Report progress.
pub trait Reporter<Data: Size> {
    /// Handle report event.
    fn report(&self, event: Event<Data>);
}

/// Utilize threads to report progress.
pub trait ParallelReporter<Data: Size>: Reporter<Data> {
    /// Error type of the [`destroy`](Self::destroy) method.
    type DestructionError;
    /// Stop all threads.
    fn destroy(self) -> Result<(), Self::DestructionError>;
}

impl<Data, Target> Reporter<Data> for &Target
where
    Data: Size,
    Target: Reporter<Data>,
{
    fn report(&self, event: Event<Data>) {
        Target::report(*self, event)
    }
}

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

use crate::size;

/// Report progress.
pub trait Reporter<Size: size::Size> {
    /// Handle report event.
    fn report(&self, event: Event<Size>);
}

/// Utilize threads to report progress.
pub trait ParallelReporter<Size: size::Size>: Reporter<Size> {
    /// Error type of the [`destroy`](Self::destroy) method.
    type DestructionError;
    /// Stop all threads.
    fn destroy(self) -> Result<(), Self::DestructionError>;
}

impl<Size, Target> Reporter<Size> for &Target
where
    Size: size::Size,
    Target: Reporter<Size>,
{
    fn report(&self, event: Event<Size>) {
        Target::report(*self, event)
    }
}

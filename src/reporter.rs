pub mod error_only_reporter;
pub mod error_report;
pub mod event;
pub mod progress;
pub mod progress_and_error_reporter;

pub use error_only_reporter::ErrorOnlyReporter;
pub use error_report::ErrorReport;
pub use event::Event;
pub use progress::Progress;
pub use progress_and_error_reporter::ProgressAndErrorReporter;

use crate::size::Size;

/// Report progress.
pub trait Reporter<Data: Size> {
    /// Handle report event.
    fn report(&self, event: Event<Data>);
}

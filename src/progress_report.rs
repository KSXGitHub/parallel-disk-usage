pub mod effectual_reporter;
pub mod event;
pub mod progress;
pub mod silenced_reporter;

pub use effectual_reporter::EffectualReporter;
pub use event::Event;
pub use progress::Progress;
pub use silenced_reporter::SilencedReporter;

use crate::size::Size;

/// Report progress.
pub trait ProgressReport<Data: Size> {
    /// Handle report event.
    fn report(&self, event: Event<Data>);
}

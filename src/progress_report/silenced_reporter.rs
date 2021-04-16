use super::{Event, ProgressReport, Size};

/// Do nothing.
#[derive(Debug)]
pub struct SilencedReporter;

impl<Data: Size> ProgressReport<Data> for SilencedReporter {
    fn report(&self, _: Event<Data>) {}
}

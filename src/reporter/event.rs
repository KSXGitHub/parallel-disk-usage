use super::ErrorReport;
use crate::size;

/// Report trigger event.
#[derive(Debug)]
pub enum Event<'a, Size: size::Size> {
    ReceiveData(Size),
    EncounterError(ErrorReport<'a>),
}

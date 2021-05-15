use super::ErrorReport;
use crate::size::Size;

/// Report trigger event.
#[derive(Debug)]
pub enum Event<'a, Data: Size> {
    ReceiveData(Data),
    EncounterError(ErrorReport<'a>),
}

use crate::{error_report::ErrorReport, size::Size};

/// Report trigger event
#[derive(Debug)]
pub enum Event<'a, Data: Size> {
    BeginScanning,
    FinishScanning,
    ReceiveData(Data),
    EncounterError(ErrorReport<'a>),
}

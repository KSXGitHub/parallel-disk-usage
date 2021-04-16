use crate::size::Size;

/// Report trigger event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event<Data: Size> {
    BeginScanning,
    FinishScanning,
    ReceiveData(Data),
    EncounterError,
}

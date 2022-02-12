use crate::{reporter::ProgressReport, size::Size};
use std::{
    ops::ControlFlow,
    sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
};

/// Like [`ProgressReport`] but mutable.
#[derive(Debug, Default)]
pub struct ProgressReportState {
    /// Whether the progress has stopped.
    pub stopped: AtomicBool,
    /// Number of scanned items.
    pub items: AtomicU64,
    /// Total size of scanned items.
    pub total: AtomicU64,
    /// Number of occurred errors.
    pub errors: AtomicU64,
}

impl ProgressReportState {
    /// Yield [`ProgressReport`] if it is running.
    /// Return `Break` otherwise.
    pub fn to_progress_report<Data>(&self) -> ControlFlow<(), ProgressReport<Data>>
    where
        Data: Size,
        u64: Into<Data>,
    {
        macro_rules! load {
            ($field:ident) => {
                self.$field.load(Relaxed)
            };
        }

        if load!(stopped) {
            return ControlFlow::Break(());
        }

        let items = load!(items);
        let total = load!(total).into();
        let errors = load!(errors);
        ControlFlow::Continue(ProgressReport {
            items,
            total,
            errors,
        })
    }
}

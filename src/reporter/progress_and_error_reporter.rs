use super::{ErrorReport, Event, ParallelReporter, ProgressReport, Reporter};
use crate::size;
use progress_report_state::ProgressReportState;
use std::{
    any::Any,
    marker::PhantomData,
    ops::ControlFlow,
    sync::{atomic::Ordering::Relaxed, Arc},
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

/// Store progress information and call report function on said information.
///
/// **NOTE:** If an error occurred, `report_error` would be called before `report_progress`.
#[derive(Debug)]
pub struct ProgressAndErrorReporter<Size, ReportError>
where
    Size: size::Size + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
    u64: Into<Size>,
{
    /// Progress information.
    progress: Arc<ProgressReportState>,
    /// Report encountered error.
    report_error: ReportError,
    /// Join handle of progress reporting thread.
    progress_reporter_handle: JoinHandle<()>,
    /// Keep generic parameters.
    _phantom: PhantomData<Size>,
}

impl<Size, ReportError> ProgressAndErrorReporter<Size, ReportError>
where
    Size: size::Size + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
    u64: Into<Size>,
{
    /// Create a new [`ProgressAndErrorReporter`] from a report function.
    pub fn new<ReportProgress>(
        report_progress: ReportProgress,
        progress_report_interval: Duration,
        report_error: ReportError,
    ) -> Self
    where
        ProgressReport<Size>: Default + 'static,
        ReportProgress: Fn(ProgressReport<Size>) + Send + Sync + 'static,
    {
        let progress = Arc::new(ProgressReportState::default());
        let progress_thread = progress.clone();
        let progress_reporter_handle = spawn(move || loop {
            sleep(progress_report_interval);
            match progress_thread.to_progress_report() {
                ControlFlow::Continue(progress) => report_progress(progress),
                ControlFlow::Break(()) => break,
            };
        });
        ProgressAndErrorReporter {
            progress,
            report_error,
            progress_reporter_handle,
            _phantom: PhantomData,
        }
    }

    /// Stop the thread that reports progress.
    ///
    /// This function would be automatically invoked once the value is [dropped](Drop).
    pub fn stop_progress_reporter(&self) {
        self.progress.stopped.store(true, Relaxed);
    }
}

impl<Size, ReportError> Reporter<Size> for ProgressAndErrorReporter<Size, ReportError>
where
    Size: size::Size + Into<u64> + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
    u64: Into<Size>,
{
    fn report(&self, event: Event<Size>) {
        use Event::*;
        let ProgressAndErrorReporter {
            progress,
            report_error,
            ..
        } = self;
        macro_rules! bump {
            ($field:ident += $delta:expr) => {
                progress.$field.fetch_add($delta, Relaxed)
            };
        }
        match event {
            ReceiveData(size) => {
                bump!(items += 1);
                bump!(total += size.into());
            }
            EncounterError(error_report) => {
                report_error(error_report);
                bump!(errors += 1);
            }
            DetectHardlink(info) => {
                bump!(linked += info.links);
                bump!(shared += info.size.into());
            }
        }
    }
}

impl<Size, ReportError> ParallelReporter<Size> for ProgressAndErrorReporter<Size, ReportError>
where
    Size: size::Size + Into<u64> + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
    u64: Into<Size>,
{
    type DestructionError = Box<dyn Any + Send + 'static>;
    fn destroy(self) -> Result<(), Self::DestructionError> {
        self.stop_progress_reporter();
        self.progress_reporter_handle.join()
    }
}

mod progress_report_state;

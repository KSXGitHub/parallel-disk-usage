use super::{ErrorReport, Event, ParallelReporter, ProgressReport, Reporter, Size};
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
pub struct ProgressAndErrorReporter<Data, ReportError>
where
    Data: Size + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
    u64: Into<Data>,
{
    /// Progress information.
    progress: Arc<ProgressReportState>,
    /// Report encountered error.
    report_error: ReportError,
    /// Join handle of progress reporting thread.
    progress_reporter_handle: JoinHandle<()>,
    /// Keep generic parameters.
    _phantom: PhantomData<Data>,
}

impl<Data, ReportError> ProgressAndErrorReporter<Data, ReportError>
where
    Data: Size + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
    u64: Into<Data>,
{
    /// Create a new [`ProgressAndErrorReporter`] from a report function.
    pub fn new<ReportProgress>(
        report_progress: ReportProgress,
        progress_report_interval: Duration,
        report_error: ReportError,
    ) -> Self
    where
        ProgressReport<Data>: Default + 'static,
        ReportProgress: Fn(ProgressReport<Data>) + Send + Sync + 'static,
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

impl<Data, ReportError> Reporter<Data> for ProgressAndErrorReporter<Data, ReportError>
where
    Data: Size + Into<u64> + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
    u64: Into<Data>,
{
    fn report(&self, event: Event<Data>) {
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
            ReceiveData(data) => {
                bump!(items += 1);
                bump!(total += data.into());
            }
            EncounterError(error_report) => {
                report_error(error_report);
                bump!(errors += 1);
            }
        }
    }
}

impl<Data, ReportError> ParallelReporter<Data> for ProgressAndErrorReporter<Data, ReportError>
where
    Data: Size + Into<u64> + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
    u64: Into<Data>,
{
    type DestructionError = Box<dyn Any + Send + 'static>;
    fn destroy(self) -> Result<(), Self::DestructionError> {
        self.stop_progress_reporter();
        self.progress_reporter_handle.join()
    }
}

mod progress_report_state;

use super::{ErrorReport, Event, ParallelReporter, ProgressReport, Reporter, Size};
use pipe_trait::Pipe;
use std::{
    any::Any,
    sync::{Arc, RwLock},
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
{
    /// Progress information.
    progress: Arc<RwLock<Option<ProgressReport<Data>>>>,
    /// Report encountered error.
    report_error: ReportError,
    /// Join handle of progress reporting thread.
    progress_reporter_handle: JoinHandle<()>,
}

impl<Data, ReportError> ProgressAndErrorReporter<Data, ReportError>
where
    Data: Size + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
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
        let progress = ProgressReport::default()
            .pipe(Some)
            .pipe(RwLock::new)
            .pipe(Arc::new);
        let progress_thread = progress.clone();
        let progress_reporter_handle = spawn(move || loop {
            sleep(progress_report_interval);
            if let Ok(progress) = progress_thread.read().as_deref() {
                if let Some(progress) = *progress {
                    report_progress(progress);
                } else {
                    break;
                }
            }
        });
        ProgressAndErrorReporter {
            progress,
            report_error,
            progress_reporter_handle,
        }
    }

    /// Stop the thread that reports progress.
    ///
    /// This function would be automatically invoked once the value is [dropped](Drop).
    pub fn stop_progress_reporter(&self) {
        let mut progress = self.progress.write().expect("lock progress to stop");
        *progress = None;
    }
}

impl<Data, ReportError> Reporter<Data> for ProgressAndErrorReporter<Data, ReportError>
where
    Data: Size + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
{
    fn report(&self, event: Event<Data>) {
        use Event::*;
        let ProgressAndErrorReporter {
            progress,
            report_error,
            ..
        } = self;
        macro_rules! handle_field {
            ($($field:ident $operator:tt $addend:expr;)+) => {
                if let Some(progress) = progress.write().ok().as_mut().and_then(|x| x.as_mut()) {
                    $(progress.$field $operator $addend;)+
                }
            };
        }
        match event {
            ReceiveData(data) => {
                handle_field! {
                    items += 1;
                    total += data;
                }
            }
            EncounterError(error_report) => {
                report_error(error_report);
                handle_field! {
                    errors += 1;
                }
            }
        }
    }
}

impl<Data, ReportError> ParallelReporter<Data> for ProgressAndErrorReporter<Data, ReportError>
where
    Data: Size + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
{
    type DestructionError = Box<dyn Any + Send + 'static>;
    fn destroy(self) -> Result<(), Self::DestructionError> {
        self.stop_progress_reporter();
        self.progress_reporter_handle.join()
    }
}

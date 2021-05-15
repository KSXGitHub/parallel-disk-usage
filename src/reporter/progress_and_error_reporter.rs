use super::{ErrorReport, Event, ProgressReport, Reporter, Size};
use pipe_trait::Pipe;
use std::{
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
            if let Some(progress) = *progress_thread.read().expect("lock progress to report") {
                report_progress(progress);
            } else {
                break;
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
            ($field:ident $operator:tt $addend:expr) => {{
                let expect_message = concat!("lock progress to mutate", stringify!($field));
                let mut progress = progress.write().expect(expect_message);
                if let Some(progress) = progress.as_mut() {
                    progress.$field $operator $addend;
                }
            }};

            ($field:ident) => {
                handle_field!($field += 1);
            };
        }
        match event {
            BeginScanning => handle_field!(known_items),
            FinishScanning => handle_field!(scanned_items),
            ReceiveData(data) => handle_field!(scanned_total += data),
            EncounterError(error_report) => {
                report_error(error_report);
                handle_field!(errors)
            }
        }
    }
}

impl<Data, ReportError> Drop for ProgressAndErrorReporter<Data, ReportError>
where
    Data: Size + Send + Sync,
    ReportError: Fn(ErrorReport) + Sync,
{
    fn drop(&mut self) {
        self.stop_progress_reporter();
    }
}

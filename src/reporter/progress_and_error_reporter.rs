use super::{ErrorReport, Event, ProgressReport, Reporter, Size};
use pipe_trait::Pipe;
use std::sync::{Arc, RwLock};

/// Store progress information and call report function on said information.
///
/// **NOTE:** If an error occurred, `report_error` would be called before `report_progress`.
#[derive(Debug)]
pub struct ProgressAndErrorReporter<Data, ReportProgress, ReportError>
where
    Data: Size,
    ReportProgress: Fn(&ProgressReport<Data>) + Sync,
    ReportError: Fn(ErrorReport) + Sync,
{
    /// Progress information.
    progress: Arc<RwLock<ProgressReport<Data>>>,
    /// Report progress information.
    report_progress: ReportProgress,
    /// Report encountered error.
    report_error: ReportError,
}

impl<Data, ReportProgress, ReportError> ProgressAndErrorReporter<Data, ReportProgress, ReportError>
where
    Data: Size,
    ReportProgress: Fn(&ProgressReport<Data>) + Sync,
    ReportError: Fn(ErrorReport) + Sync,
{
    /// Create a new [`ProgressAndErrorReporter`] from a report function.
    pub fn new(report_progress: ReportProgress, report_error: ReportError) -> Self
    where
        ProgressReport<Data>: Default,
    {
        let progress = ProgressReport::default().pipe(RwLock::new).pipe(Arc::new);
        ProgressAndErrorReporter {
            progress,
            report_progress,
            report_error,
        }
    }
}

impl<Data, ReportProgress, ReportError> Reporter<Data>
    for ProgressAndErrorReporter<Data, ReportProgress, ReportError>
where
    Data: Size,
    ReportProgress: Fn(&ProgressReport<Data>) + Sync,
    ReportError: Fn(ErrorReport) + Sync,
{
    fn report(&self, event: Event<Data>) {
        use Event::*;
        let ProgressAndErrorReporter {
            progress,
            report_progress,
            report_error,
        } = self;
        macro_rules! handle_field {
            ($field:ident $operator:tt $addend:expr) => {{
                {
                    let expect_message = concat!("lock progress to mutate", stringify!($field));
                    let mut progress = progress.write().expect(expect_message);
                    progress.$field $operator $addend;
                }
                {
                    let progress = progress.read().expect("lock progress to report");
                    report_progress(&progress);
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

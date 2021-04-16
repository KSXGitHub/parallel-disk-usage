use super::{Event, Progress, ProgressReport, Size};
use pipe_trait::Pipe;
use std::sync::{Arc, RwLock};

/// Store progress information and call report function on said information.
#[derive(Debug)]
pub struct EffectualReporter<Data, Report>
where
    Data: Size,
    Report: Fn(&Progress<Data>) + Sync,
{
    /// Progress information.
    pub progress: Arc<RwLock<Progress<Data>>>,
    /// The report function.
    pub report: Report,
}

impl<Data, Report> EffectualReporter<Data, Report>
where
    Data: Size,
    Report: Fn(&Progress<Data>) + Sync,
{
    /// Create a new [`EffectualReporter`] from a report function.
    pub fn new(report: Report) -> Self
    where
        Progress<Data>: Default,
    {
        let progress = Progress::default().pipe(RwLock::new).pipe(Arc::new);
        EffectualReporter { progress, report }
    }
}

impl<Data, Report> ProgressReport<Data> for EffectualReporter<Data, Report>
where
    Data: Size,
    Report: Fn(&Progress<Data>) + Sync,
{
    fn report(&self, event: Event<Data>) {
        use Event::*;
        let EffectualReporter { progress, report } = self;
        macro_rules! handle_field {
            ($field:ident $operator:tt $addend:expr) => {{
                {
                    let expect_message = concat!("lock progress to mutate", stringify!($field));
                    let mut progress = progress.write().expect(expect_message);
                    progress.$field $operator $addend;
                }
                {
                    let progress = progress.read().expect("lock progress to report");
                    report(&progress);
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
            EncounterError => handle_field!(errors),
        }
    }
}

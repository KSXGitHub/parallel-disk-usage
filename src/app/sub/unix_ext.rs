use super::HardlinkSubroutines;
use crate::data_tree::DataTree;
use crate::hardlink::HardlinkAware;
use crate::json_data::JsonShared;
use crate::os_string_display::OsStringDisplay;
use crate::runtime_error::RuntimeError;
use crate::size;
use pipe_trait::Pipe;

impl<Size> HardlinkSubroutines<Size> for HardlinkAware<Size>
where
    DataTree<OsStringDisplay, Size>: Send,
    Size: size::Size + Sync,
{
    fn convert_error(error: Self::Error) -> RuntimeError {
        match error {}
    }

    fn print_report(
        report: Self::Report,
        bytes_format: Size::DisplayFormat,
    ) -> Result<(), RuntimeError> {
        let summary = report.summarize();
        if summary.inodes > 0 {
            print!("{}", summary.display(bytes_format)); // the summary already ends with "\n", println! isn't needed here.
        }
        Ok(())
    }

    fn json_report(report: Self::Report) -> Result<Option<JsonShared<Size>>, RuntimeError> {
        let summary = report.summarize().pipe(Some);
        let details = report.into_reflection().pipe(Some);
        Ok(Some(JsonShared { details, summary }))
    }
}

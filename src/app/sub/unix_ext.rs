use super::HardlinkSubroutines;
use crate::{
    data_tree::DataTree, hardlink::HardlinkListReflection, os_string_display::OsStringDisplay,
    runtime_error::RuntimeError, size,
};
use pipe_trait::Pipe;

impl<Size> HardlinkSubroutines<Size> for crate::hardlink::HardlinkAware<Size>
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
            println!("{}", summary.display(bytes_format));
        }
        Ok(())
    }

    fn serializable_report(
        report: Self::Report,
    ) -> Result<Option<HardlinkListReflection<Size>>, RuntimeError> {
        report.into_reflection().pipe(Some).pipe(Ok)
    }
}

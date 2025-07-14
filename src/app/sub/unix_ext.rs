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
        let (inodes, links, size): (usize, usize, Size) = report
            .iter()
            .filter_map(|values| {
                let size = values.size();
                let links = values.links().len();
                (links > 1).then_some(())?;
                Some((*size, links))
            })
            .fold(
                (0, 0, Size::default()),
                |(inodes, total_links, total_size), (size, links)| {
                    (inodes + 1, total_links + links, total_size + size)
                },
            );
        if inodes > 0 {
            let size = size.display(bytes_format);
            println!("Detected {links} hardlinks for {inodes} unique files (total: {size})");
        }
        Ok(())
    }

    fn serializable_report(
        report: Self::Report,
    ) -> Result<Option<HardlinkListReflection<Size>>, RuntimeError> {
        report.into_reflection().pipe(Some).pipe(Ok)
    }
}

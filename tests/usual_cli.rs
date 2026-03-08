#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use parallel_disk_usage::{
    bytes_format::BytesFormat,
    data_tree::DataTree,
    fs_tree_builder::FsTreeBuilder,
    get_size::GetApparentSize,
    hardlink::HardlinkIgnorant,
    os_string_display::OsStringDisplay,
    reporter::{ErrorOnlyReporter, ErrorReport},
    visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer},
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::process::{Command, Stdio};

#[cfg(unix)]
use parallel_disk_usage::get_size::{GetBlockCount, GetBlockSize};

fn stdio(command: Command) -> Command {
    command
        .with_stdin(Stdio::null())
        .with_stdout(Stdio::piped())
        .with_stderr(Stdio::piped())
}

#[test]
fn total_width() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: DEFAULT_GET_SIZE,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn column_width() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--column-width")
        .with_arg("10")
        .with_arg("90")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: DEFAULT_GET_SIZE,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::components(10, 90),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn min_ratio_0() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--total-width=100")
        .with_arg("--min-ratio=0")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: GetApparentSize,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn min_ratio() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--total-width=100")
        .with_arg("--min-ratio=0.1")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: GetApparentSize,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.1);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn max_depth_2() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--total-width=100")
        .with_arg("--max-depth=2")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: GetApparentSize,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 2,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn max_depth_1() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--total-width=100")
        .with_arg("--max-depth=1")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: GetApparentSize,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 1,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn top_down() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg("--top-down")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: DEFAULT_GET_SIZE,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::TopDown,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn align_right() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg("--align-right")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: DEFAULT_GET_SIZE,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Right,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn quantity_apparent_size() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg("--quantity=apparent-size")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: GetApparentSize,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[cfg(unix)]
#[test]
fn quantity_block_size() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg("--quantity=block-size")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: GetBlockSize,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[cfg(unix)]
#[test]
fn quantity_block_count() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg("--quantity=block-count")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: GetBlockCount,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: (),
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn bytes_format_plain() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg("--quantity=apparent-size")
        .with_arg("--bytes-format=plain")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: GetApparentSize,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::PlainNumber,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn bytes_format_metric() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg("--quantity=apparent-size")
        .with_arg("--bytes-format=metric")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: GetApparentSize,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn bytes_format_binary() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg("--quantity=apparent-size")
        .with_arg("--bytes-format=binary")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: GetApparentSize,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::BinaryUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn path_to_workspace() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg(&workspace)
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        size_getter: DEFAULT_GET_SIZE,
        hardlinks_recorder: &HardlinkIgnorant,
        reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
        max_depth: 10,
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

#[test]
fn multiple_names() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--total-width=100")
        .with_arg("nested")
        .with_arg("flat")
        .with_arg("empty-dir")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let mut data_tree = ["nested", "flat", "empty-dir"]
        .iter()
        .map(|name| {
            let builder = FsTreeBuilder {
                root: workspace.to_path_buf().join(name),
                size_getter: GetApparentSize,
                hardlinks_recorder: &HardlinkIgnorant,
                reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
                max_depth: 10,
            };
            let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
            *data_tree.name_mut() = OsStringDisplay::os_string_from(name);
            data_tree
        })
        .pipe(|children| {
            DataTree::dir(
                OsStringDisplay::os_string_from("(total)"),
                0.into(),
                children.collect(),
            )
        })
        .into_par_sorted(|left, right| left.size().cmp(&right.size()).reverse());
    data_tree.par_cull_insignificant_data(0.01);
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);

    let mut lines = actual.lines();
    assert!(lines.next().unwrap().contains("┌──1"));
    assert!(lines.next().unwrap().contains("┌─┴0"));
    assert!(lines.next().unwrap().contains("┌─┴nested"));
    assert!(lines.next().unwrap().contains("│ ┌──1"));
    assert!(lines.next().unwrap().contains("│ ├──2"));
    assert!(lines.next().unwrap().contains("│ ├──3"));
    assert!(lines.next().unwrap().contains("├─┴flat"));
    assert!(lines.next().unwrap().contains("┌─┴(total)"));
    assert_eq!(lines.next(), None);
}

#[test]
fn multiple_names_max_depth_2() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--total-width=100")
        .with_arg("--max-depth=2")
        .with_arg("nested")
        .with_arg("flat")
        .with_arg("empty-dir")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let mut data_tree = ["nested", "flat", "empty-dir"]
        .iter()
        .map(|name| {
            let builder = FsTreeBuilder {
                root: workspace.to_path_buf().join(name),
                size_getter: GetApparentSize,
                hardlinks_recorder: &HardlinkIgnorant,
                reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
                max_depth: 1,
            };
            let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
            *data_tree.name_mut() = OsStringDisplay::os_string_from(name);
            data_tree
        })
        .pipe(|children| {
            DataTree::dir(
                OsStringDisplay::os_string_from("(total)"),
                0.into(),
                children.collect(),
            )
        })
        .into_par_sorted(|left, right| left.size().cmp(&right.size()).reverse());
    data_tree.par_cull_insignificant_data(0.01);
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);

    let mut lines = actual.lines();
    assert!(lines.next().unwrap().contains("┌──nested"));
    assert!(lines.next().unwrap().contains("├──flat"));
    assert!(lines.next().unwrap().contains("┌─┴(total)"));
    assert_eq!(lines.next(), None);
}

#[test]
fn multiple_names_max_depth_1() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--quantity=apparent-size")
        .with_arg("--total-width=100")
        .with_arg("--max-depth=1")
        .with_arg("nested")
        .with_arg("flat")
        .with_arg("empty-dir")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let mut data_tree = ["nested", "flat", "empty-dir"]
        .iter()
        .map(|name| {
            let builder = FsTreeBuilder {
                root: workspace.to_path_buf().join(name),
                size_getter: GetApparentSize,
                hardlinks_recorder: &HardlinkIgnorant,
                reporter: &ErrorOnlyReporter::new(ErrorReport::SILENT),
                max_depth: 10,
            };
            let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
            *data_tree.name_mut() = OsStringDisplay::os_string_from(name);
            data_tree
        })
        .pipe(|children| {
            DataTree::dir(
                OsStringDisplay::os_string_from("(total)"),
                0.into(),
                children.collect(),
            )
        })
        .into_par_retained(|_, _| false)
        .into_par_sorted(|left, right| left.size().cmp(&right.size()).reverse());
    data_tree.par_cull_insignificant_data(0.01);
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: None,
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);

    let mut lines = actual.lines();
    assert!(lines.next().unwrap().contains("┌──(total)"));
    assert_eq!(lines.next(), None);
}

/// Test that `--color=always` with multiple arguments correctly colors leaf nodes.
///
/// When multiple path arguments are provided, a synthetic `(total)` root is created.
/// The coloring logic must still correctly resolve filesystem types (symlink, directory, etc.)
/// for the original paths — not for non-existent paths prefixed with `(total)/...`.
///
/// We verify this by comparing the `--color=always` output (with ANSI stripped) against
/// `--color=never` output, AND checking that symlinks and empty directories actually
/// receive their expected ANSI color codes from LS_COLORS.
#[cfg(unix)]
#[test]
fn color_always_multiple_args() {
    // LS_COLORS: di=01;34 (bold blue for dirs), ln=01;36 (bold cyan for symlinks)
    let ls_colors = "rs=0:di=01;34:ln=01;36:ex=01;32:fi=00";
    let workspace = SampleWorkspace::simple_tree_with_diverse_kinds();

    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--color=always")
        .with_arg("--quantity=apparent-size")
        .with_arg("--total-width=100")
        .with_arg("--min-ratio=0")
        .with_arg("dir-a")
        .with_arg("dir-b")
        .with_arg("empty-dir-1")
        .with_arg("link-dir")
        .with_arg("link-file.txt")
        .with_env("LS_COLORS", ls_colors)
        .pipe(stdio)
        .output()
        .expect("spawn command with --color=always and multiple args")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{actual}\n");

    let colorless = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--color=never")
        .with_arg("--quantity=apparent-size")
        .with_arg("--total-width=100")
        .with_arg("--min-ratio=0")
        .with_arg("dir-a")
        .with_arg("dir-b")
        .with_arg("empty-dir-1")
        .with_arg("link-dir")
        .with_arg("link-file.txt")
        .with_env("LS_COLORS", ls_colors)
        .pipe(stdio)
        .output()
        .expect("spawn command with --color=never and multiple args")
        .pipe(stdout_text);
    eprintln!("COLORLESS:\n{colorless}\n");

    // Stripping ANSI from the colorful output should match the colorless output.
    let stripped = strip_ansi_escapes::strip_str(&actual);
    let stripped = stripped.trim_end();
    assert_eq!(stripped, colorless, "stripped colorful output must match colorless output");

    // Verify that symlinks receive the symlink color (bold cyan = \x1b[1;36m).
    // With the `(total)` bug, these would have no ANSI codes at all because
    // the path `(total)/link-dir` doesn't exist on the filesystem.
    let symlink_prefix = "\x1b[1;36m";
    let has_colored_symlink = actual.lines().any(|line| {
        line.contains(symlink_prefix) && (line.contains("link-dir") || line.contains("link-file.txt"))
    });
    assert!(
        has_colored_symlink,
        "symlinks (link-dir, link-file.txt) should be colored with the symlink prefix ({symlink_prefix}), \
         but no line contains it. This likely means the `(total)` synthetic root caused \
         filesystem type detection to fail for leaf nodes."
    );

    // Verify that the empty directory leaf receives the directory color (bold blue = \x1b[1;34m).
    let dir_prefix = "\x1b[1;34m";
    let has_colored_empty_dir = actual.lines().any(|line| {
        line.contains(dir_prefix) && line.contains("empty-dir-1")
    });
    assert!(
        has_colored_empty_dir,
        "empty-dir-1 should be colored with the directory prefix ({dir_prefix}), \
         but the matching line doesn't contain it."
    );
}

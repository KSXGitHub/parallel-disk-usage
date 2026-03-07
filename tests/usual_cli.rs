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

/// Predefined `LS_COLORS` value used in color tests to ensure deterministic output.
const LS_COLORS: &str = "rs=0:di=01;34:ln=01;36:ex=01;32:fi=00";

#[cfg(unix)]
use parallel_disk_usage::{
    get_size::{GetBlockCount, GetBlockSize},
    ls_colors::LsColors,
    visualizer::{Color, Coloring},
};
#[cfg(unix)]
use std::{collections::HashMap, ffi::OsString};

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

#[test]
fn colorful_equals_colorless() {
    let workspace = SampleWorkspace::default();

    let colorful = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--color=always")
        .with_arg("--total-width=100")
        .with_env("LS_COLORS", LS_COLORS)
        .pipe(stdio)
        .output()
        .expect("spawn command with --color=always");
    inspect_stderr(&colorful.stderr);
    assert!(colorful.status.success(), "pdu exited with non-zero status");
    let colorful_stripped = colorful
        .stdout
        .pipe(String::from_utf8)
        .expect("UTF-8")
        .pipe(strip_ansi_escapes::strip_str)
        .trim_end()
        .to_string();

    let colorless = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--color=never")
        .with_arg("--total-width=100")
        .with_env("LS_COLORS", LS_COLORS)
        .pipe(stdio)
        .output()
        .expect("spawn command with --color=never")
        .pipe(stdout_text);

    assert_eq!(colorful_stripped, colorless);
}

#[test]
fn different_ls_colors() {
    let workspace = SampleWorkspace::default();

    let with_ls_colors = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--color=always")
        .with_arg("--total-width=100")
        .with_env("LS_COLORS", LS_COLORS)
        .pipe(stdio)
        .output()
        .expect("spawn command with --color=always and LS_COLORS");
    inspect_stderr(&with_ls_colors.stderr);
    assert!(
        with_ls_colors.status.success(),
        "pdu exited with non-zero status"
    );
    let with_ls_colors_stripped = with_ls_colors
        .stdout
        .pipe(String::from_utf8)
        .expect("UTF-8")
        .pipe(strip_ansi_escapes::strip_str)
        .trim_end()
        .to_string();

    let without_ls_colors = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--color=always")
        .with_arg("--total-width=100")
        .without_env("LS_COLORS")
        .pipe(stdio)
        .output()
        .expect("spawn command with --color=always and without LS_COLORS");
    inspect_stderr(&without_ls_colors.stderr);
    assert!(
        without_ls_colors.status.success(),
        "pdu exited with non-zero status"
    );
    let without_ls_colors_stripped = without_ls_colors
        .stdout
        .pipe(String::from_utf8)
        .expect("UTF-8")
        .pipe(strip_ansi_escapes::strip_str)
        .trim_end()
        .to_string();

    assert_eq!(with_ls_colors_stripped, without_ls_colors_stripped);
}

#[cfg(unix)]
#[test]
fn color_always() {
    let workspace = SampleWorkspace::simple_tree_with_diverse_kinds();

    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--color=always")
        .with_arg("--total-width=100")
        .with_arg("--min-ratio=0")
        .with_env("LS_COLORS", LS_COLORS)
        .pipe(stdio)
        .output()
        .expect("spawn command with --color=always")
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
    data_tree.par_sort_by(|left, right| left.size().cmp(&right.size()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");

    let ls_colors = LsColors::from_str(LS_COLORS);
    let map = HashMap::from([
        (
            vec![
                OsString::from("."),
                OsString::from("dir-a"),
                OsString::from("file-a1.txt"),
            ],
            Color::Normal,
        ),
        (
            vec![
                OsString::from("."),
                OsString::from("dir-a"),
                OsString::from("file-a2.txt"),
            ],
            Color::Normal,
        ),
        (
            vec![
                OsString::from("."),
                OsString::from("dir-a"),
                OsString::from("subdir-a"),
                OsString::from("file-a3.txt"),
            ],
            Color::Normal,
        ),
        (
            vec![
                OsString::from("."),
                OsString::from("dir-b"),
                OsString::from("file-b1.txt"),
            ],
            Color::Normal,
        ),
        (
            vec![OsString::from("."), OsString::from("file-root.txt")],
            Color::Normal,
        ),
        (
            vec![OsString::from("."), OsString::from("link-dir")],
            Color::Symlink,
        ),
        (
            vec![OsString::from("."), OsString::from("link-file.txt")],
            Color::Symlink,
        ),
        (
            vec![OsString::from("."), OsString::from("empty-dir-1")],
            Color::Directory,
        ),
        (
            vec![OsString::from("."), OsString::from("empty-dir-2")],
            Color::Directory,
        ),
    ]);
    let coloring = Coloring::new(ls_colors, map);

    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        coloring: Some(&coloring),
    };
    let expected = format!("{visualizer}");
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{expected}\n");

    assert_eq!(actual, expected);
}

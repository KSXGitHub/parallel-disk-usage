#![cfg(feature = "cli")]

pub mod _utils;
pub use _utils::*;

use command_extra::CommandExtra;
use parallel_disk_usage::{
    bytes_format::BytesFormat,
    data_tree::DataTree,
    fs_tree_builder::FsTreeBuilder,
    os_string_display::OsStringDisplay,
    reporter::{ErrorOnlyReporter, ErrorReport},
    size_getters::{GET_APPARENT_SIZE, GET_BLOCK_COUNT, GET_BLOCK_SIZE},
    visualizer::{BarAlignment, ColumnWidthDistribution, Direction, Visualizer},
};
use pipe_trait::Pipe;
use pretty_assertions::assert_eq;
use std::{
    convert::TryInto,
    process::{Command, Stdio},
};

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: DEFAULT_GET_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: DEFAULT_GET_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::components(10, 90),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_APPARENT_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_APPARENT_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.1);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_APPARENT_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 2.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_APPARENT_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 1.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: DEFAULT_GET_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::TopDown,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: DEFAULT_GET_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Right,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_APPARENT_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_BLOCK_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_BLOCK_COUNT,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: (),
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

    assert_eq!(actual, expected);
}

#[cfg(unix)]
#[test]
fn bytes_format_plain() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg("--quantity=block-size")
        .with_arg("--bytes-format=plain")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_BLOCK_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::PlainNumber,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

    assert_eq!(actual, expected);
}

#[cfg(unix)]
#[test]
fn bytes_format_metric() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg("--quantity=block-size")
        .with_arg("--bytes-format=metric")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_BLOCK_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

    assert_eq!(actual, expected);
}

#[cfg(unix)]
#[test]
fn bytes_format_binary() {
    let workspace = SampleWorkspace::default();
    let actual = Command::new(PDU)
        .with_current_dir(&workspace)
        .with_arg("--total-width=100")
        .with_arg("--quantity=block-size")
        .with_arg("--bytes-format=binary")
        .pipe(stdio)
        .output()
        .expect("spawn command")
        .pipe(stdout_text);
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: GET_BLOCK_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    *data_tree.name_mut() = OsStringDisplay::os_string_from(".");
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::BinaryUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let builder = FsTreeBuilder {
        root: workspace.to_path_buf(),
        get_data: DEFAULT_GET_SIZE,
        reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
    };
    let mut data_tree: DataTree<OsStringDisplay, _> = builder.into();
    data_tree.par_cull_insignificant_data(0.01);
    data_tree.par_sort_by(|left, right| left.data().cmp(&right.data()).reverse());
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

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
    eprintln!("ACTUAL:\n{}\n", &actual);

    let mut data_tree = ["nested", "flat", "empty-dir"]
        .iter()
        .map(|name| {
            let builder = FsTreeBuilder {
                root: workspace.to_path_buf().join(name),
                get_data: GET_APPARENT_SIZE,
                reporter: ErrorOnlyReporter::new(ErrorReport::SILENT),
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
        .pipe(|tree| tree.into_par_sorted(|left, right| left.data().cmp(&right.data()).reverse()));
    data_tree.par_cull_insignificant_data(0.01);
    let visualizer = Visualizer::<OsStringDisplay, _> {
        data_tree: &data_tree,
        bytes_format: BytesFormat::MetricUnits,
        direction: Direction::BottomUp,
        bar_alignment: BarAlignment::Left,
        column_width_distribution: ColumnWidthDistribution::total(100),
        max_depth: 10.try_into().unwrap(),
    };
    let expected = format!("{}", visualizer);
    let expected = expected.trim_end();
    eprintln!("EXPECTED:\n{}\n", expected);

    assert_eq!(actual, expected);
}

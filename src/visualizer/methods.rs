use super::{
    ChildPosition, Parenthood, ProportionBar, TreeHorizontalSlice, TreeSkeletalComponent,
    Visualizer,
};
use crate::{size::Size, tree::Tree};
use assert_cmp::{debug_assert_op, debug_assert_op_expr};
use derive_more::{Deref, DerefMut};
use smart_default::SmartDefault;
use std::{
    cmp::{max, min},
    collections::{HashSet, LinkedList},
    fmt::Display,
    num::NonZeroUsize,
    ops::{Index, IndexMut},
};
use zero_copy_pads::{align_left, align_right, Width};

const PERCENTAGE_COLUMN_MAX_WIDTH: usize = "100%".len();
const BORDER_COLUMNS: usize = 3; // 4 columns, 3 borders, each border has a width of 1.

#[derive(SmartDefault, Deref, DerefMut)]
struct Table<Row, ColumnWidth: Default> {
    #[deref]
    #[deref_mut]
    data: LinkedList<Row>,
    column_width: ColumnWidth,
}

#[derive(Clone, Copy)]
struct NodeInfo<Name, NodeData> {
    name: Name,
    node_data: NodeData,
    row_index: usize,
    sibling_count: NonZeroUsize,
    index_as_child: usize,
    children_count: usize,
    remaining_depth: usize,
}

#[derive(Deref, DerefMut)]
struct InitialRow<Name, NodeData> {
    #[deref]
    #[deref_mut]
    node_info: NodeInfo<Name, NodeData>,
    ancestors: Vec<NodeInfo<Name, NodeData>>,
    preceding_sibling: Option<NodeInfo<Name, NodeData>>,
    size: String,
    percentage: String,
}

impl<Name, NodeData> InitialRow<Name, NodeData> {
    #[inline]
    fn parent(&self) -> Option<&'_ NodeInfo<Name, NodeData>> {
        self.ancestors.last()
    }
}

#[derive(Default, Clone, Copy)]
struct InitialColumnWidth {
    size_column_width: usize,
}

impl InitialColumnWidth {
    #[inline]
    const fn total_max_width(self) -> usize {
        self.size_column_width + PERCENTAGE_COLUMN_MAX_WIDTH + BORDER_COLUMNS
    }
}

type InitialTable<Name, NodeData> = Table<InitialRow<Name, NodeData>, InitialColumnWidth>;
fn render_initial<Name, Data>(visualizer: Visualizer<Name, Data>) -> InitialTable<&'_ Name, Data>
where
    Name: Display,
    Data: Size + Into<u64>,
{
    #[derive(Clone)]
    struct Param<Name, NodeData> {
        remaining_depth: usize,
        index_as_child: usize,
        ancestors: Vec<NodeInfo<Name, NodeData>>,
        preceding_sibling: Option<NodeInfo<Name, NodeData>>,
    }

    struct ActResult<Name, NodeData> {
        node_info: NodeInfo<Name, NodeData>,
    }

    struct TraverseResult<Name, NodeData> {
        node_info: NodeInfo<Name, NodeData>,
    }

    fn traverse<'a, Name, Data, Act>(
        tree: &'a Tree<Name, Data>,
        act: &mut Act,
        param: Param<&'a Name, Data>,
    ) -> Option<TraverseResult<&'a Name, Data>>
    where
        Data: Size,
        Act: FnMut(&'a Tree<Name, Data>, Param<&'a Name, Data>) -> ActResult<&'a Name, Data>,
    {
        if param.remaining_depth == 0 {
            return None;
        }
        let ActResult { node_info } = act(tree, param.clone());
        let remaining_depth = param.remaining_depth - 1;
        let mut preceding_sibling = None;
        for (index_as_child, child) in tree.children().iter().enumerate() {
            let mut ancestors = Vec::with_capacity(param.ancestors.len() + 1);
            ancestors.clone_from(&param.ancestors);
            ancestors.push(node_info);
            let traverse_result = traverse(
                child,
                act,
                Param {
                    remaining_depth,
                    index_as_child,
                    ancestors,
                    preceding_sibling,
                },
            );
            preceding_sibling = traverse_result.map(|x| x.node_info);
        }
        Some(TraverseResult { node_info })
    }

    let mut initial_table = InitialTable::default();
    let total_fs_size = visualizer.tree.data().into();

    traverse(
        visualizer.tree,
        &mut |node, param| {
            let Param {
                index_as_child,
                ancestors,
                remaining_depth,
                preceding_sibling,
            } = param;
            let name = node.name();
            let node_data = node.data();
            let row_index = initial_table.len();
            let children_count = node.children().len();
            let fs_size = node.data().into();
            let percentage = rounded_div::u64(fs_size * 100, total_fs_size);
            let percentage = format!("{}%", percentage);
            let size = node
                .data()
                .display(visualizer.measurement_system)
                .to_string();
            let sibling_count = ancestors
                .last()
                .map(|parent| parent.children_count)
                .unwrap_or(1);
            debug_assert_op!(sibling_count != 0);
            debug_assert_op!(index_as_child < sibling_count);
            let sibling_count = unsafe { NonZeroUsize::new_unchecked(sibling_count) };
            let node_info = NodeInfo {
                name,
                node_data,
                row_index,
                sibling_count,
                index_as_child,
                children_count,
                remaining_depth,
            };

            initial_table.column_width.size_column_width =
                max(initial_table.column_width.size_column_width, size.len());

            initial_table.push_back(InitialRow {
                node_info,
                ancestors,
                preceding_sibling,
                percentage,
                size,
            });

            ActResult { node_info }
        },
        Param {
            remaining_depth: visualizer.max_depth,
            index_as_child: 0,
            ancestors: Vec::with_capacity(0),
            preceding_sibling: None,
        },
    );

    initial_table
}

#[derive(Deref, DerefMut)]
struct TreeRow<Name, NodeData> {
    #[deref]
    #[deref_mut]
    initial_row: InitialRow<Name, NodeData>,
    tree_horizontal_slice: TreeHorizontalSlice<String>,
}

#[derive(Default, Clone, Copy, Deref, DerefMut)]
struct TreeColumnWidth {
    #[deref]
    #[deref_mut]
    initial_column_width: InitialColumnWidth,
    tree_column_width: usize,
}

impl TreeColumnWidth {
    #[inline]
    const fn total_max_width(self) -> usize {
        self.initial_column_width.total_max_width() + self.tree_column_width
    }
}

type TreeTable<Name, NodeData> = Table<TreeRow<Name, NodeData>, TreeColumnWidth>;
fn render_tree<'a, Name, Data>(
    visualizer: Visualizer<'a, Name, Data>,
    initial_table: InitialTable<&'a Name, Data>,
    max_width: usize,
) -> TreeTable<&'a Name, Data>
where
    Name: Display,
    Data: Size + Into<u64>,
{
    let InitialTable {
        data: initial_data,
        column_width: initial_column_width,
    } = initial_table;
    let initial_data_len = initial_data.len();
    let mut tree_column_width = TreeColumnWidth {
        initial_column_width,
        tree_column_width: 0,
    };
    let mut excluded_row_indices = HashSet::with_capacity(initial_data_len);
    let mut intermediate_table: Vec<_> = initial_data
        .into_iter()
        .map(|initial_row| {
            let child_position =
                ChildPosition::from_index(initial_row.index_as_child, initial_row.sibling_count);
            let parenthood = if initial_row.remaining_depth == 0 {
                Parenthood::Childless
            } else {
                Parenthood::from_children_count(initial_row.children_count)
            };
            let skeletal_component = TreeSkeletalComponent {
                child_position,
                parenthood,
                direction: visualizer.direction,
            };
            let ancestor_relative_positions = initial_row
                .ancestors
                .iter()
                .map(|node_info| {
                    ChildPosition::from_index(node_info.index_as_child, node_info.sibling_count)
                })
                .collect();
            let mut tree_horizontal_slice = TreeHorizontalSlice {
                ancestor_relative_positions,
                skeletal_component,
                name: initial_row.name.to_string(),
            };
            if let Ok(()) = tree_horizontal_slice.truncate(max_width) {
                tree_column_width.tree_column_width = max(
                    tree_column_width.tree_column_width,
                    tree_horizontal_slice.width(),
                );
            } else {
                excluded_row_indices.insert(initial_row.row_index);
            }
            TreeRow {
                tree_horizontal_slice,
                initial_row,
            }
        })
        .collect();

    debug_assert_op_expr!(intermediate_table.len(), ==, initial_data_len);
    if cfg!(debug_assertions) {
        intermediate_table
            .iter()
            .map(|row| row.row_index)
            .enumerate()
            .for_each(|(expected_row_index, actual_row_index)| {
                debug_assert_op!(actual_row_index == expected_row_index)
            });
    }

    for excluded_row_index in excluded_row_indices.iter().copied() {
        // mark more nodes as childless
        let parent_row_index = intermediate_table
            .index(excluded_row_index)
            .parent()
            .map(|parent_info| parent_info.row_index);
        if let Some(parent_row_index) = parent_row_index {
            let parent_row = &mut intermediate_table[parent_row_index];
            if parent_row.children_count == 0 || parent_row.children_count == 1 {
                parent_row.children_count = 0;
                parent_row
                    .tree_horizontal_slice
                    .skeletal_component
                    .parenthood = Parenthood::Childless;
            } else {
                parent_row.children_count -= 1;
            }
        }

        // mark more nodes as last amongst siblings
        let preceding_sibling_row_index = intermediate_table
            .index(excluded_row_index)
            .preceding_sibling
            .map(|node_info| node_info.row_index);
        if let (Some(preceding_sibling_row_index), Some(parent_row_index)) =
            (preceding_sibling_row_index, parent_row_index)
        {
            let is_sibling = |row: &&TreeRow<&Name, Data>| {
                row.parent()
                    .map(|parent| parent.row_index == parent_row_index)
                    .unwrap_or(false)
            };
            let is_excluded =
                |row: &TreeRow<&Name, Data>| excluded_row_indices.contains(&row.row_index);
            let following_siblings_are_all_excluded = intermediate_table
                .index(excluded_row_index..)
                .iter()
                .filter(is_sibling)
                .all(is_excluded);
            if following_siblings_are_all_excluded {
                let target = &mut intermediate_table
                    .index_mut(preceding_sibling_row_index)
                    .tree_horizontal_slice
                    .skeletal_component
                    .child_position;
                *target = ChildPosition::Last;
            }
        }
    }

    let tree_data: LinkedList<_> = intermediate_table
        .into_iter()
        .filter(|row| !excluded_row_indices.contains(&row.row_index))
        .collect();

    TreeTable {
        data: tree_data,
        column_width: tree_column_width,
    }
}

#[derive(Deref, DerefMut)]
struct BarRow<Name, NodeData> {
    #[deref]
    #[deref_mut]
    tree_row: TreeRow<Name, NodeData>,
    proportion_bar: ProportionBar,
}

#[derive(Default, Clone, Copy, Deref, DerefMut)]
struct BarColumnWidth {
    #[deref]
    #[deref_mut]
    tree_column_width: TreeColumnWidth,
}

fn render_bars<'a, Name, Data>(
    tree_table: TreeTable<&'a Name, Data>,
    total: u64,
    width: usize,
) -> LinkedList<BarRow<&'a Name, Data>>
where
    Name: Display,
    Data: Size + Into<u64> + 'a,
{
    tree_table
        .data
        .into_iter()
        .map(|tree_row| {
            let get_value = |node_info: &NodeInfo<&Name, Data>| {
                let node_data = node_info.node_data.into();
                rounded_div::u64(node_data * (width as u64), total) as usize
            };

            macro_rules! ancestor_value {
                ($index:expr, $fallback:expr) => {
                    tree_row
                        .ancestors
                        .get($index)
                        .map(get_value)
                        .unwrap_or($fallback)
                };
            }

            let lv0_value = get_value(&tree_row.node_info);
            let lv1_value = ancestor_value!(3, lv0_value);
            let lv2_value = ancestor_value!(2, lv1_value);
            let lv3_value = ancestor_value!(1, lv2_value);
            let lv4_value = ancestor_value!(0, lv3_value);
            debug_assert_op!(lv0_value <= lv1_value);
            debug_assert_op!(lv1_value <= lv2_value);
            debug_assert_op!(lv2_value <= lv3_value);
            debug_assert_op!(lv3_value <= lv4_value);
            debug_assert_op!(lv4_value == width);

            let lv0_visible = lv0_value;
            let lv1_visible = lv1_value - lv0_value;
            let lv2_visible = lv2_value - lv1_value;
            let lv3_visible = lv3_value - lv2_value;
            let lv4_visible = lv4_value - lv3_value;
            debug_assert_op_expr!(
                lv0_visible + lv1_visible + lv2_visible + lv3_visible + lv4_visible,
                ==,
                width
            );

            let proportion_bar = ProportionBar {
                level0: lv0_visible,
                level1: lv1_visible,
                level2: lv2_visible,
                level3: lv3_visible,
                level4: lv4_visible,
            };
            BarRow {
                tree_row,
                proportion_bar,
            }
        })
        .collect()
}

impl<'a, Name, Data> Visualizer<'a, Name, Data>
where
    Name: Display,
    Data: Size + Into<u64>,
{
    /// Create ASCII visualization of the [tree](Tree), such visualization is meant to be
    /// printed to a terminal screen.
    pub fn visualize(mut self) -> Vec<String> {
        let initial_table = render_initial(self);

        let min_width = initial_table.column_width.total_max_width();
        if self.max_width <= min_width {
            let extra_cols = 3; // make space for tree_column to minimize second-time re-rendering.
            self.max_width = min_width + extra_cols;
            return self.visualize();
        }

        let tree_max_width = min(self.max_width - min_width, self.max_width / 3);
        let tree_table = render_tree(self, initial_table, tree_max_width);

        let min_width = tree_table.column_width.total_max_width();
        if self.max_width <= min_width {
            self.max_width = min_width + 1;
            return self.visualize();
        }

        let size_width = tree_table.column_width.size_column_width;
        let tree_width = tree_table.column_width.tree_column_width;

        let bar_width = self.max_width - min_width;
        let bar_table = render_bars(tree_table, self.tree.data().into(), bar_width);

        bar_table
            .into_iter()
            .map(|row| {
                format!(
                    "{size} {tree}│{bar}│{ratio}",
                    size = align_right(&row.size, size_width),
                    tree = align_left(&row.tree_horizontal_slice, tree_width),
                    bar = &row.proportion_bar,
                    ratio = align_right(&row.percentage, PERCENTAGE_COLUMN_MAX_WIDTH),
                )
            })
            .collect()
    }
}

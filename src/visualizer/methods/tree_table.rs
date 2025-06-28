use super::{InitialColumnWidth, InitialRow, InitialTable, Table};
use crate::{
    size,
    visualizer::{
        ChildPosition, Parenthood, TreeHorizontalSlice, TreeSkeletalComponent, Visualizer,
    },
};
use assert_cmp::{debug_assert_op, debug_assert_op_expr};
use derive_more::{Deref, DerefMut};
use pipe_trait::Pipe;
use std::{
    cmp::max,
    collections::{HashSet, LinkedList},
    fmt::Display,
    ops::{Index, IndexMut},
};
use zero_copy_pads::Width;

#[derive(Deref, DerefMut)]
pub(super) struct TreeRow<Name, NodeData> {
    #[deref]
    #[deref_mut]
    pub(super) initial_row: InitialRow<Name, NodeData>,
    pub(super) tree_horizontal_slice: TreeHorizontalSlice<String>,
}

#[derive(Default, Clone, Copy, Deref, DerefMut)]
pub(super) struct TreeColumnWidth {
    #[deref]
    #[deref_mut]
    pub(super) initial_column_width: InitialColumnWidth,
    pub(super) tree_column_width: usize,
}

impl TreeColumnWidth {
    #[inline]
    pub(super) const fn total_max_width(self) -> usize {
        self.initial_column_width.total_max_width() + self.tree_column_width
    }
}

pub(super) type TreeTable<Name, NodeData> = Table<TreeRow<Name, NodeData>, TreeColumnWidth>;

pub(super) fn render_tree<'a, Name, Size>(
    visualizer: Visualizer<'a, Name, Size>,
    initial_table: InitialTable<&'a Name, Size>,
    max_width: usize,
) -> TreeTable<&'a Name, Size>
where
    Name: Display,
    Size: size::Size + Into<u64>,
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
            let parenthood = Parenthood::from_children_count(initial_row.children_count);
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
                initial_row,
                tree_horizontal_slice,
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

    // mark children of excluded nodes as excluded
    loop {
        let excluded_count = excluded_row_indices.len();
        let mut children_of_excluded = LinkedList::<usize>::new();

        for excluded_row_index in excluded_row_indices.iter().copied() {
            let is_child = |row: &&TreeRow<&Name, Size>| {
                row.parent()
                    .is_some_and(|node_info| node_info.row_index == excluded_row_index)
            };
            intermediate_table
                .index(excluded_row_index..)
                .iter()
                .filter(is_child)
                .map(|row| row.row_index)
                .pipe(|iter| children_of_excluded.extend(iter));
        }

        excluded_row_indices.extend(children_of_excluded);
        if excluded_row_indices.len() == excluded_count {
            break;
        }
    }

    for excluded_row_index in excluded_row_indices.iter().copied() {
        // mark more nodes as childless
        let parent_row_index = intermediate_table
            .index(excluded_row_index)
            .parent()
            .map(|parent_info| parent_info.row_index);
        if let Some(parent_row_index) = parent_row_index {
            let parent_row = &mut intermediate_table[parent_row_index];
            debug_assert_op_expr!(parent_row.children_count, >, 0);
            if parent_row.children_count == 1 {
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
            let is_sibling = |row: &&TreeRow<&Name, Size>| {
                row.parent()
                    .is_some_and(|parent| parent.row_index == parent_row_index)
            };
            let is_excluded =
                |row: &TreeRow<&Name, Size>| excluded_row_indices.contains(&row.row_index);
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

    let is_included = |row: &TreeRow<&Name, Size>| !excluded_row_indices.contains(&row.row_index);
    let tree_data: LinkedList<_> = intermediate_table.into_iter().filter(is_included).collect();

    TreeTable {
        data: tree_data,
        column_width: tree_column_width,
    }
}

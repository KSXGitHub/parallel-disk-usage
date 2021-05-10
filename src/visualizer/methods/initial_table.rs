use super::{NodeInfo, Table, BORDER_COLUMNS, PERCENTAGE_COLUMN_MAX_WIDTH};
use crate::{size::Size, tree::Tree, visualizer::Visualizer};
use assert_cmp::debug_assert_op;
use derive_more::{Deref, DerefMut};
use std::{cmp::max, fmt::Display, num::NonZeroUsize};

#[derive(Deref, DerefMut)]
pub(super) struct InitialRow<Name, NodeData> {
    #[deref]
    #[deref_mut]
    pub(super) node_info: NodeInfo<Name, NodeData>,
    pub(super) ancestors: Vec<NodeInfo<Name, NodeData>>,
    pub(super) preceding_sibling: Option<NodeInfo<Name, NodeData>>,
    pub(super) size: String,
    pub(super) percentage: String,
}

impl<Name, NodeData> InitialRow<Name, NodeData> {
    #[inline]
    pub(super) fn parent(&self) -> Option<&'_ NodeInfo<Name, NodeData>> {
        self.ancestors.last()
    }
}

#[derive(Default, Clone, Copy)]
pub(super) struct InitialColumnWidth {
    pub(super) size_column_width: usize,
}

impl InitialColumnWidth {
    #[inline]
    pub(super) const fn total_max_width(self) -> usize {
        self.size_column_width + PERCENTAGE_COLUMN_MAX_WIDTH + BORDER_COLUMNS
    }
}

pub(super) type InitialTable<Name, NodeData> =
    Table<InitialRow<Name, NodeData>, InitialColumnWidth>;

pub(super) fn render_initial<Name, Data>(
    visualizer: Visualizer<Name, Data>,
) -> InitialTable<&'_ Name, Data>
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
            debug_assert_op!(remaining_depth > 0);
            let children_count = if remaining_depth != 1 {
                node.children().len()
            } else {
                0
            };
            let fs_size = node.data().into();
            let percentage = rounded_div::u64(fs_size * 100, total_fs_size);
            let percentage = format!("{}%", percentage);
            let size = node.data().to_string();
            let sibling_count = ancestors.last().map_or(1, |parent| parent.children_count);
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
            remaining_depth: visualizer.max_depth.get(),
            index_as_child: 0,
            ancestors: Vec::with_capacity(0),
            preceding_sibling: None,
        },
    );

    initial_table
}

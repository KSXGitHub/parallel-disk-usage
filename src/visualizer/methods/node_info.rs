use std::num::NonZeroUsize;

#[derive(Clone, Copy)]
pub struct NodeInfo<Name, NodeData> {
    pub name: Name,
    pub node_data: NodeData,
    pub row_index: usize,
    pub sibling_count: NonZeroUsize,
    pub index_as_child: usize,
    pub children_count: usize,
    pub remaining_depth: usize,
}

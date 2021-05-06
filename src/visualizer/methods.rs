use super::{
    ChildPosition, Parenthood, ProportionBar, TreeHorizontalSlice, TreeSkeletalComponent,
    Visualizer,
};
use crate::{size::Size, tree::Tree};
use assert_cmp::{debug_assert_op, debug_assert_op_expr};
use itertools::izip;
use std::{
    cmp::{max, min},
    fmt::Display,
};
use zero_copy_pads::{align_left, align_right, AlignRight, PaddedColumnIter, Width};

#[derive(Default)]
struct TreeColumn {
    list: Vec<Option<TreeHorizontalSlice<String>>>,
    max_width: usize,
}

impl TreeColumn {
    #[inline]
    fn len(&self) -> usize {
        self.list.len()
    }
}

// NOTE: The 4 methods below, despite sharing the same structure, cannot be unified due to
//       them relying on each other's `PaddedColumnIter::total_width`.

impl<'a, Name, Data> Visualizer<'a, Name, Data>
where
    Name: Display,
    Data: Size + Into<u64>,
{
    fn visualize_sizes(self) -> PaddedColumnIter<String, char, AlignRight> {
        fn traverse<Name, Data, Act>(tree: &Tree<Name, Data>, act: &mut Act, remaining_depth: usize)
        where
            Data: Size,
            Act: FnMut(&Tree<Name, Data>),
        {
            act(tree);
            if remaining_depth == 0 {
                return;
            }
            let next_remaining_depth = remaining_depth - 1;
            for child in tree.children() {
                traverse(child, act, next_remaining_depth);
            }
        }

        let mut iter = PaddedColumnIter::new(' ', AlignRight);

        traverse(
            self.tree,
            &mut |node| {
                let value = node.data().display(self.measurement_system).to_string();
                iter.push_back(value);
            },
            self.max_depth,
        );

        iter
    }

    fn visualize_percentage(self) -> Vec<String> {
        fn traverse<Name, Data, Act>(tree: &Tree<Name, Data>, act: &mut Act, remaining_depth: usize)
        where
            Data: Size,
            Act: FnMut(&Tree<Name, Data>),
        {
            act(tree);
            if remaining_depth == 0 {
                return;
            }
            let next_remaining_depth = remaining_depth - 1;
            for child in tree.children() {
                traverse(child, act, next_remaining_depth);
            }
        }

        let total = self.tree.data().into();
        let mut result = Vec::new();

        traverse(
            self.tree,
            &mut |node| {
                let current = node.data().into();
                debug_assert_op!(current <= total);
                let percentage = rounded_div::u64(current * 100, total);
                let percentage = format!("{}%", percentage);
                result.push(percentage);
            },
            self.max_depth,
        );

        result
    }

    fn visualize_tree(self, max_width: usize) -> TreeColumn {
        #[derive(Clone)]
        struct Param {
            node_index: usize,
            parent_column_index: Option<usize>,
            sibling_count: usize,
            remaining_depth: usize,
            ancestor_relative_positions: Vec<ChildPosition>,
        }

        struct ActResult {
            relative_position: ChildPosition,
            column_index: usize,
        }

        fn traverse<Name, Data, Act>(tree: &Tree<Name, Data>, act: &mut Act, param: Param)
        where
            Data: Size,
            Act: FnMut(&Tree<Name, Data>, Param) -> ActResult,
        {
            let ActResult {
                relative_position,
                column_index,
            } = act(tree, param.clone());
            if param.remaining_depth == 0 {
                return;
            }
            let parent_column_index = Some(column_index);
            let sibling_count = tree.children().len();
            let remaining_depth = param.remaining_depth - 1;
            let ancestor_relative_positions = || {
                let mut result = param.ancestor_relative_positions.clone();
                result.push(relative_position);
                result
            };
            for (node_index, child) in tree.children().iter().enumerate() {
                traverse(
                    child,
                    act,
                    Param {
                        node_index,
                        parent_column_index,
                        sibling_count,
                        remaining_depth,
                        ancestor_relative_positions: ancestor_relative_positions(),
                    },
                );
            }
        }

        let mut tree_column = TreeColumn::default();

        traverse(
            self.tree,
            &mut |tree, param| {
                let Param {
                    node_index,
                    parent_column_index,
                    sibling_count,
                    remaining_depth,
                    ancestor_relative_positions,
                } = param;
                debug_assert_op!(sibling_count > node_index);
                let child_position = ChildPosition::from_index(node_index, sibling_count);
                let parenthood = if remaining_depth == 0 {
                    Parenthood::Childless
                } else {
                    Parenthood::from_node(tree)
                };
                let skeletal_component = TreeSkeletalComponent {
                    direction: self.direction,
                    child_position,
                    parenthood,
                };
                let name = tree.name().to_string();
                let mut tree_horizontal_slice = TreeHorizontalSlice {
                    ancestor_relative_positions,
                    skeletal_component,
                    name,
                };
                let tree_horizontal_slice =
                    if let Ok(()) = tree_horizontal_slice.truncate(max_width) {
                        Some(tree_horizontal_slice)
                    } else {
                        // if the name of the node was truncated so hard, it disappeared completely,
                        // then the parent of the truncated node should be childless.
                        if let Some(parent) = parent_column_index.and_then(|parent_column_index| {
                            tree_column.list[parent_column_index].as_mut()
                        }) {
                            parent.skeletal_component.parenthood = Parenthood::Childless;
                        }

                        // the `visualize` method would filter out `None` values, i.e. disappear.
                        None
                    };
                if let Some(tree_horizontal_slice) = &tree_horizontal_slice {
                    tree_column.max_width =
                        max(tree_column.max_width, tree_horizontal_slice.width());
                }
                let column_index = tree_column.list.len();
                tree_column.list.push(tree_horizontal_slice);
                ActResult {
                    relative_position: child_position,
                    column_index,
                }
            },
            Param {
                node_index: 0,
                parent_column_index: None,
                sibling_count: 1,
                remaining_depth: self.max_depth,
                ancestor_relative_positions: Vec::new(),
            },
        );

        tree_column
    }

    fn visualize_bars(self, width: usize) -> Vec<ProportionBar> {
        #[derive(Debug, Clone, Copy)]
        enum Values {
            Level0,
            Level1(usize),
            Level2(usize, usize),
            Level3(usize, usize, usize),
            Level4(usize, usize, usize, usize),
        }

        impl Values {
            fn add(self, x: usize) -> Self {
                use Values::*;
                match self {
                    Level0 => Level1(x),
                    Level1(a) => Level2(a, x),
                    Level2(a, b) => Level3(a, b, x),
                    Level3(a, b, c) => Level4(a, b, c, x),
                    _ => self,
                }
            }

            fn vec4(self, z: usize) -> (usize, usize, usize, usize) {
                #![allow(clippy::many_single_char_names)]
                use Values::*;
                match self {
                    Level0 => (z, z, z, z),
                    Level1(a) => (a, z, z, z),
                    Level2(a, b) => (a, b, z, z),
                    Level3(a, b, c) => (a, b, c, z),
                    Level4(a, b, c, d) => (a, b, c, d),
                }
            }
        }

        fn traverse<Name, Data, Act>(
            tree: &Tree<Name, Data>,
            act: &mut Act,
            values: Values,
            remaining_depth: usize,
        ) where
            Data: Size,
            Act: FnMut(&Tree<Name, Data>, Values) -> usize,
        {
            let next_value = act(tree, values);
            let next_values = values.add(next_value);
            if remaining_depth == 0 {
                return;
            }
            let next_remaining_depth = remaining_depth - 1;
            for child in tree.children() {
                traverse(child, act, next_values, next_remaining_depth);
            }
        }

        let total = self.tree.data().into();
        let mut bars = Vec::new();

        traverse(
            self.tree,
            &mut |tree, values| {
                let current = tree.data().into();
                debug_assert_op!(current <= total);
                let lv0_value = rounded_div::u64(current * (width as u64), total) as usize;
                let (lv4_value, lv3_value, lv2_value, lv1_value) = values.vec4(lv0_value);
                debug_assert_op!(lv0_value <= lv1_value);
                debug_assert_op!(lv1_value <= lv2_value);
                debug_assert_op!(lv2_value <= lv3_value);
                debug_assert_op!(lv3_value <= lv4_value);
                debug_assert_op!(lv4_value <= width);
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
                bars.push(ProportionBar {
                    level0: lv0_visible,
                    level1: lv1_visible,
                    level2: lv2_visible,
                    level3: lv3_visible,
                    level4: lv4_visible,
                });
                lv0_value
            },
            Values::Level0,
            self.max_depth,
        );

        bars
    }

    /// Create ASCII visualization of the [tree](Tree), such visualization is meant to be
    /// printed to a terminal screen.
    pub fn visualize(mut self) -> Vec<String> {
        let size_column = self.visualize_sizes();
        let percentage_column = self.visualize_percentage();
        let percentage_column_max_width = "100%".len();
        let border_cols = 3; // 4 columns, 3 borders, each border has a width of 1.
        let min_width = size_column.total_width() + percentage_column_max_width + border_cols;
        if self.max_width <= min_width {
            let extra_cols = 3; // make space for tree_column to minimize second-time re-rendering.
            self.max_width = min_width + extra_cols;
            return self.visualize();
        }
        let tree_max_width = min(self.max_width - min_width, self.max_width / 3);
        let tree_column = self.visualize_tree(tree_max_width);
        let min_width = min_width + tree_column.max_width;
        if self.max_width <= min_width {
            self.max_width = min_width + 1;
            return self.visualize();
        }
        let bar_width = self.max_width - min_width;
        let bars = self.visualize_bars(bar_width);
        debug_assert_op_expr!(bars.len(), ==, size_column.len());
        debug_assert_op_expr!(bars.len(), ==, percentage_column.len());
        debug_assert_op_expr!(bars.len(), ==, tree_column.len());
        let tree_column_max_width = tree_column.max_width;
        izip!(
            size_column,
            percentage_column.into_iter(),
            tree_column.list.into_iter(),
            bars.into_iter(),
        )
        .filter_map(|(size, percentage, tree_horizontal_slice, bar)| {
            if let Some(tree_horizontal_slice) = tree_horizontal_slice {
                Some((size, percentage, tree_horizontal_slice, bar))
            } else {
                None
            }
        })
        .map(|(size, percentage, tree_horizontal_slice, bar)| {
            format!(
                "{size} {tree}│{bar}│{ratio}",
                size = size,
                tree = align_left(tree_horizontal_slice, tree_column_max_width),
                bar = bar,
                ratio = align_right(percentage, percentage_column_max_width),
            )
        })
        .collect()
    }
}

use super::{
    ChildPosition, MaybeTreeHorizontalSlice, Parenthood, ProportionBar, TreeHorizontalSlice,
    TreeSkeletalComponent, Visualizer,
};
use crate::{size::Size, tree::Tree};
use assert_cmp::{debug_assert_op, debug_assert_op_expr};
use itertools::izip;
use std::fmt::Display;
use zero_copy_pads::{align_right, AlignLeft, AlignRight, PaddedColumnIter};

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
            for child in &tree.children {
                traverse(child, act, next_remaining_depth);
            }
        }

        let mut iter = PaddedColumnIter::new(' ', AlignRight);

        traverse(
            self.tree,
            &mut |node| {
                let value = node.data.display(self.measurement_system).to_string();
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
            for child in &tree.children {
                traverse(child, act, next_remaining_depth);
            }
        }

        let total = self.tree.data.into();
        let mut result = Vec::new();

        traverse(
            self.tree,
            &mut |node| {
                let current = node.data.into();
                debug_assert_op!(current <= total);
                let percentage = rounded_div::u64(current * 100, total);
                let percentage = format!("{}%", percentage);
                result.push(percentage);
            },
            self.max_depth,
        );

        result
    }

    fn visualize_tree(
        self,
        max_width: usize,
    ) -> PaddedColumnIter<MaybeTreeHorizontalSlice<String>, char, AlignLeft> {
        #[derive(Clone)]
        struct Param {
            node_index: usize,
            sibling_count: usize,
            remaining_depth: usize,
            ancestor_relative_positions: Vec<ChildPosition>,
        }

        fn traverse<Name, Data, Act>(tree: &Tree<Name, Data>, act: &mut Act, param: Param)
        where
            Data: Size,
            Act: FnMut(&Tree<Name, Data>, Param) -> ChildPosition,
        {
            let parent_relative_position = act(tree, param.clone());
            if param.remaining_depth == 0 {
                return;
            }
            let sibling_count = tree.children.len();
            let remaining_depth = param.remaining_depth - 1;
            let ancestor_relative_positions = || {
                let mut result = param.ancestor_relative_positions.clone();
                result.push(parent_relative_position);
                result
            };
            for (node_index, child) in tree.children.iter().enumerate() {
                traverse(
                    child,
                    act,
                    Param {
                        node_index,
                        sibling_count,
                        remaining_depth,
                        ancestor_relative_positions: ancestor_relative_positions(),
                    },
                );
            }
        }

        let mut padded_column_iter = PaddedColumnIter::new(' ', AlignLeft);

        traverse(
            self.tree,
            &mut |tree, param| {
                let Param {
                    node_index,
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
                let skeletal_component_visualization = TreeSkeletalComponent {
                    direction: self.direction,
                    child_position,
                    parenthood,
                }
                .visualize();
                let name = tree.name.to_string();
                let mut tree_horizontal_slice = TreeHorizontalSlice {
                    ancestor_relative_positions,
                    skeletal_component_visualization,
                    name,
                };
                let tree_horizontal_slice = MaybeTreeHorizontalSlice::from(
                    if let Ok(()) = tree_horizontal_slice.truncate(max_width) {
                        Some(tree_horizontal_slice)
                    } else {
                        None
                    },
                );
                padded_column_iter.push_back(tree_horizontal_slice);
                child_position
            },
            Param {
                node_index: 0,
                sibling_count: 1,
                remaining_depth: self.max_depth,
                ancestor_relative_positions: Vec::new(),
            },
        );

        padded_column_iter
    }

    fn visualize_bars(self, width: u64) -> Vec<ProportionBar> {
        #[derive(Debug, Clone, Copy)]
        enum Values {
            Level0,
            Level1(u64),
            Level2(u64, u64),
            Level3(u64, u64, u64),
            Level4(u64, u64, u64, u64),
        }

        impl Values {
            fn add(self, x: u64) -> Self {
                use Values::*;
                match self {
                    Level0 => Level1(x),
                    Level1(a) => Level2(a, x),
                    Level2(a, b) => Level3(a, b, x),
                    Level3(a, b, c) => Level4(a, b, c, x),
                    _ => self,
                }
            }

            fn vec4(self, z: u64) -> (u64, u64, u64, u64) {
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
            Act: FnMut(&Tree<Name, Data>, Values) -> u64,
        {
            let next_value = act(tree, values);
            let next_values = values.add(next_value);
            if remaining_depth == 0 {
                return;
            }
            let next_remaining_depth = remaining_depth - 1;
            for child in &tree.children {
                traverse(child, act, next_values, next_remaining_depth);
            }
        }

        let total = self.tree.data.into();
        let mut bars = Vec::new();

        traverse(
            self.tree,
            &mut |tree, values| {
                let current = tree.data.into();
                debug_assert_op!(current <= total);
                let lv0_value = rounded_div::u64(current * width, total);
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
                    level0: lv0_visible as usize,
                    level1: lv1_visible as usize,
                    level2: lv2_visible as usize,
                    level3: lv3_visible as usize,
                    level4: lv4_visible as usize,
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
        let tree_max_width = self.max_width - min_width;
        let tree_column = self.visualize_tree(tree_max_width);
        let min_width = min_width + tree_column.total_width();
        if self.max_width <= min_width {
            self.max_width = min_width + 1;
            return self.visualize();
        }
        let bar_width = self.max_width - min_width;
        let bars = self.visualize_bars(bar_width as u64);
        debug_assert_op_expr!(bars.len(), ==, size_column.len());
        debug_assert_op_expr!(bars.len(), ==, percentage_column.len());
        debug_assert_op_expr!(bars.len(), ==, tree_column.len());
        izip!(
            size_column,
            percentage_column.into_iter(),
            tree_column.into_iter(),
            bars.into_iter(),
        )
        .filter_map(|(size, percentage, tree_horizontal_slice, bar)| {
            if let Some(tree_horizontal_slice) =
                TreeHorizontalSlice::resolve_padded_maybe(tree_horizontal_slice)
            {
                Some((size, percentage, tree_horizontal_slice, bar))
            } else {
                None
            }
        })
        .map(|(size, percentage, tree_horizontal_slice, bar)| {
            format!(
                "{size} {tree}│{bar}│{ratio}",
                size = size,
                tree = tree_horizontal_slice,
                bar = bar,
                ratio = align_right(percentage, percentage_column_max_width),
            )
        })
        .collect()
    }
}

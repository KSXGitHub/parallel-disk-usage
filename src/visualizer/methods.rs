use super::{
    ChildPosition, Parenthood, ProportionBarBlock, TreeHorizontalSlice, TreeSkeletalComponent,
    Visualizer,
};
use crate::{size::Size, tree::Tree};
use assert_cmp::{debug_assert_op, debug_assert_op_expr};
use fmt_iter::repeat;
use itertools::izip;
use pipe_trait::Pipe;
use std::fmt::Display;
use zero_copy_pads::{align_column_right, align_right, AlignLeft, AlignRight, PaddedColumnIter};

impl<Name, Data> Visualizer<Name, Data>
where
    Name: Display,
    Data: Size + Into<u64>,
{
    fn visualize_sizes(&self) -> PaddedColumnIter<String, char, AlignRight> {
        let measurement_system = self.measurement_system;
        self.tree
            .iter_node()
            .map(|node| node.data.display(measurement_system).to_string())
            .pipe(align_column_right)
    }

    fn visualize_percentage(&self) -> Vec<String> {
        let total = self.tree.data.into();
        self.tree
            .iter_node()
            .map(|node| {
                let current = node.data.into();
                debug_assert_op!(current <= total);
                let percentage = rounded_div::u64(current * 100, total);
                format!("{}%", percentage)
            })
            .collect()
    }

    fn visualize_tree(&self) -> PaddedColumnIter<TreeHorizontalSlice<String>, char, AlignLeft> {
        #[derive(Clone, Copy)]
        struct Param {
            index: usize,
            count: usize,
            depth: usize,
        }

        fn traverse<Name, Data, Act>(tree: &Tree<Name, Data>, act: &mut Act, param: Param)
        where
            Data: Size,
            Act: FnMut(&Tree<Name, Data>, Param),
        {
            act(tree, param);
            let count = tree.children.len();
            let depth = param.depth + 1;
            for (index, child) in tree.children.iter().enumerate() {
                traverse(
                    child,
                    act,
                    Param {
                        index,
                        count,
                        depth,
                    },
                );
            }
        }

        let mut padded_column_iter = PaddedColumnIter::new(' ', AlignLeft);

        traverse(
            &self.tree,
            &mut |tree, param| {
                let Param {
                    index,
                    count,
                    depth,
                } = param;
                debug_assert_op!(count > index);
                let skeleton = TreeSkeletalComponent {
                    child_position: ChildPosition::from_index(index, count),
                    direction: self.direction,
                    parenthood: Parenthood::from_node(tree),
                }
                .visualize();
                let name = tree.name.to_string();
                padded_column_iter.push_back(TreeHorizontalSlice {
                    depth,
                    skeleton,
                    name,
                });
            },
            Param {
                index: 0,
                count: 1,
                depth: 0,
            },
        );

        padded_column_iter
    }

    fn visualize_bars(&self, width: u64) -> Vec<String> {
        fn traverse<Name, Data, Act>(
            tree: &Tree<Name, Data>,
            act: &mut Act,
            level: usize,
            lv1_value: u64,
            lv2_value: u64,
            lv3_value: u64,
        ) where
            Data: Size,
            Act: FnMut(&Tree<Name, Data>, usize, u64, u64, u64) -> u64,
        {
            let next_lv1_value = act(tree, level, lv1_value, lv2_value, lv3_value);
            let next_lv2_value = lv1_value;
            let next_lv3_value = lv2_value;
            for child in &tree.children {
                traverse(
                    child,
                    act,
                    level + 1,
                    next_lv1_value,
                    next_lv2_value,
                    next_lv3_value,
                );
            }
        }
        let mut bars = Vec::new();
        let total = self.tree.data.into();
        let space_block = ProportionBarBlock::new(4);
        let lv3_block = ProportionBarBlock::new(3);
        let lv2_block = ProportionBarBlock::new(2);
        let lv1_block = ProportionBarBlock::new(1);
        let lv0_block = ProportionBarBlock::new(0);
        traverse(
            &self.tree,
            &mut |tree, level, lv1_value, lv2_value, lv3_value| {
                let _ = level; // level can be used to limit depth, but it isn't implemented for now.
                let current = tree.data.into();
                debug_assert_op!(current <= total);
                let lv0_value = rounded_div::u64(current * width, total);
                debug_assert_op!(lv0_value <= lv1_value);
                debug_assert_op!(lv1_value <= lv2_value);
                debug_assert_op!(lv2_value <= lv3_value);
                debug_assert_op!(lv3_value <= width);
                let lv0_visible = lv0_value;
                let lv1_visible = lv1_value - lv0_value;
                let lv2_visible = lv2_value - lv1_value;
                let lv3_visible = lv3_value - lv2_value;
                let empty_spaces = width - lv3_value;
                debug_assert_op_expr!(
                    lv0_visible + lv1_visible + lv2_visible + lv3_visible + empty_spaces,
                    ==,
                    width
                );
                bars.push(format!(
                    "{space}{lv3}{lv2}{lv1}{lv0}",
                    space = repeat(space_block, empty_spaces as usize),
                    lv3 = repeat(lv3_block, lv3_visible as usize),
                    lv2 = repeat(lv2_block, lv2_visible as usize),
                    lv1 = repeat(lv1_block, lv1_visible as usize),
                    lv0 = repeat(lv0_block, lv0_visible as usize),
                ));
                lv0_value
            },
            0,
            width,
            width,
            width,
        );
        bars
    }

    pub fn visualize(&self, width: usize) -> Vec<String> {
        let size_column = self.visualize_sizes();
        let percentage_column = self.visualize_percentage();
        let tree_column = self.visualize_tree();
        // TODO: handle case where the total max_width is greater than given width
        let percentage_column_max_width = "100%".len();
        let bar_width = width
            - size_column.total_width()
            - percentage_column_max_width
            - tree_column.total_width();
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
        .map(|(size, percentage, tree_horizontal_slice, bar)| {
            format!(
                "{}{}{}{}",
                size,
                tree_horizontal_slice,
                bar,
                align_right(percentage, percentage_column_max_width),
            )
        })
        .collect()
    }
}

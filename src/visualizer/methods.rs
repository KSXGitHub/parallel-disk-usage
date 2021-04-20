use super::{
    ChildPosition, Parenthood, ProportionBarBlock, TreeSkeletalComponent,
    TreeSkeletalComponentVisualization, Visualizer,
};
use crate::{size::Size, tree::Tree};
use fmt_iter::repeat;
use itertools::izip;
use std::{cmp::max, fmt::Display};

#[derive(Debug)]
struct Column<Item> {
    max_width: usize,
    content: Vec<Item>,
}

fn traverse<Name, Data, Act>(tree: &Tree<Name, Data>, act: &mut Act)
where
    Data: Size,
    Act: FnMut(&Tree<Name, Data>),
{
    act(tree);
    for child in &tree.children {
        act(child);
        traverse(child, act);
    }
}

#[inline]
fn make_column<Name, Data, MakeItem>(tree: &Tree<Name, Data>, make_item: MakeItem) -> Column<String>
where
    Data: Size,
    MakeItem: Fn(&Tree<Name, Data>) -> String,
{
    let mut max_width = 0;
    let mut content = Vec::new();
    traverse(tree, &mut |tree| {
        let item = make_item(tree);
        max_width = max(max_width, item.len());
        content.push(item);
    });
    Column { max_width, content }
}

macro_rules! debug_assert_cmp {
    ($left:ident $op:tt $right:ident) => {
        debug_assert_cmp!(($left) $op ($right));
    };

    (($left:expr) $op:tt ($right:expr)) => {
        match ($left, $right) {
            (left, right) => {
                debug_assert!(
                    left $op right,
                    "{left_expr} {op} {right_expr} ⇒ {left_value} {op} {right_value} ⇒ false",
                    op = stringify!(op),
                    left_expr = stringify!($left),
                    right_expr = stringify!($right),
                    left_value = left,
                    right_value = right,
                );
            }
        }
    };
}

impl<Name, Data> Visualizer<Name, Data>
where
    Name: Display,
    Data: Size + Into<u64>,
{
    fn visualize_sizes(&self) -> Column<String> {
        make_column(&self.tree, |tree| {
            tree.data.display(self.measurement_system).to_string()
        })
    }

    fn visualize_percentage(&self) -> Column<String> {
        let total = self.tree.data.into();
        make_column(&self.tree, |tree| {
            let current = tree.data.into();
            debug_assert_cmp!(current <= total);
            let percentage = rounded_div::u64(current * 100, total);
            format!("{}%", percentage)
        })
    }

    fn visualize_tree(&self) -> Column<(TreeSkeletalComponentVisualization, String)> {
        fn traverse<Name, Data, Act>(tree: &Tree<Name, Data>, act: &mut Act)
        where
            Data: Size,
            Act: FnMut(&Tree<Name, Data>, usize, usize),
        {
            act(tree, 0, 1);
            let count = tree.children.len();
            for (index, child) in tree.children.iter().enumerate() {
                act(child, index, count);
                traverse(child, act);
            }
        }

        let mut max_width = 0;
        let mut content = Vec::new();

        traverse(&self.tree, &mut |tree, index, count| {
            debug_assert_cmp!(count > index);
            let skeleton = TreeSkeletalComponent {
                child_position: ChildPosition::from_index(index, count),
                direction: self.direction,
                parenthood: Parenthood::from_node(tree),
            }
            .visualize();
            let name = tree.name.to_string();
            max_width = max(max_width, skeleton.len() + name.len());
            content.push((skeleton, name))
        });

        Column { max_width, content }
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
                act(
                    child,
                    level + 1,
                    next_lv1_value,
                    next_lv2_value,
                    next_lv3_value,
                );
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
                debug_assert_cmp!(current <= total);
                let lv0_value = rounded_div::u64(current * width, total);
                debug_assert_cmp!(lv0_value <= lv1_value);
                debug_assert_cmp!(lv1_value <= lv2_value);
                debug_assert_cmp!(lv2_value <= lv3_value);
                debug_assert_cmp!(lv3_value <= width);
                let lv0_visible = lv0_value;
                let lv1_visible = lv1_value - lv0_value;
                let lv2_visible = lv2_value - lv1_value;
                let lv3_visible = lv3_value - lv2_value;
                let empty_spaces = width - lv3_value;
                debug_assert_cmp!(
                    (lv0_visible + lv1_visible + lv2_visible + lv3_visible + empty_spaces)
                        == (width)
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
        let bar_width =
            width - size_column.max_width - percentage_column.max_width - tree_column.max_width;
        let bars = self.visualize_bars(bar_width as u64);
        debug_assert_cmp!((bars.len()) == (size_column.content.len()));
        debug_assert_cmp!((bars.len()) == (percentage_column.content.len()));
        debug_assert_cmp!((bars.len()) == (tree_column.content.len()));
        izip!(
            size_column.content.into_iter(),
            percentage_column.content.into_iter(),
            tree_column.content.into_iter(),
            bars.into_iter(),
        )
        .map(|(size, percentage, (skeleton, name), bar)| {
            // TODO: proper padding
            format!("{}{}{}{}{}", size, skeleton, name, bar, percentage)
        })
        .collect()
    }
}

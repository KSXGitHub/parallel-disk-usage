use super::{NodeInfo, TreeRow, TreeTable};
use crate::{size, visualizer::ProportionBar};

use derive_more::{Deref, DerefMut};
use std::{collections::LinkedList, fmt::Display};

#[derive(Deref, DerefMut)]
pub(super) struct BarRow<Name, NodeData> {
    #[deref]
    #[deref_mut]
    pub(super) tree_row: TreeRow<Name, NodeData>,
    pub(super) proportion_bar: ProportionBar,
}

pub(super) fn render_bars<'a, Name, Size>(
    tree_table: TreeTable<&'a Name, Size>,
    total: u64,
    width: usize,
) -> LinkedList<BarRow<&'a Name, Size>>
where
    Name: Display,
    Size: size::Size + Into<u64> + 'a,
{
    tree_table
        .data
        .into_iter()
        .map(|tree_row| {
            let get_value = |node_info: &NodeInfo<&Name, Size>| {
                let node_data = node_info.node_data.into();
                if total == 0 {
                    return 0;
                }
                rounded_div::u64(node_data * (width as u64), total) as usize
            };

            macro_rules! ancestor_value {
                ($index:expr, $fallback:expr) => {
                    tree_row.ancestors.get($index).map_or($fallback, get_value)
                };
            }

            let lv0_value = get_value(&tree_row.node_info);
            let lv1_value = ancestor_value!(3, lv0_value);
            let lv2_value = ancestor_value!(2, lv1_value);
            let lv3_value = ancestor_value!(1, lv2_value);
            let lv4_value = width;
            debug_assert!(lv0_value <= lv1_value);
            debug_assert!(lv1_value <= lv2_value);
            debug_assert!(lv2_value <= lv3_value);
            debug_assert!(lv3_value <= lv4_value);

            let lv0_visible = lv0_value;
            let lv1_visible = lv1_value - lv0_value;
            let lv2_visible = lv2_value - lv1_value;
            let lv3_visible = lv3_value - lv2_value;
            let lv4_visible = lv4_value - lv3_value;

            #[cfg(debug_assertions)]
            {
                let actual_lv4_value = ancestor_value!(0, lv3_value);
                if actual_lv4_value != 0 {
                    debug_assert!(actual_lv4_value == width);
                    debug_assert!(
                        lv0_visible + lv1_visible + lv2_visible + lv3_visible + lv4_visible
                        ==
                        width
                    );
                }
            }

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

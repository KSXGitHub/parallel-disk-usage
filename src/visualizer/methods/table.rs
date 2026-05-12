use derive_more::{Deref, DerefMut};
use smart_default::SmartDefault;
use std::collections::LinkedList;

#[derive(Deref, DerefMut, SmartDefault)]
pub struct Table<Row, ColumnWidth: Default> {
    #[deref]
    #[deref_mut]
    pub data: LinkedList<Row>,
    pub column_width: ColumnWidth,
}

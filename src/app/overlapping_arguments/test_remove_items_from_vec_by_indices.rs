use super::remove_items_from_vec_by_indices;
use maplit::hashset;
use pretty_assertions::assert_eq;
use std::collections::HashSet;

#[test]
fn remove_nothing() {
    let original = vec![31, 54, 22, 81, 67, 45, 52, 20, 85, 66, 27, 84];
    let mut modified = original.clone();
    remove_items_from_vec_by_indices(&mut modified, &HashSet::new());
    assert_eq!(modified, original);
}

#[test]
fn remove_single() {
    let original = vec![31, 54, 22, 81, 67, 45, 52, 20, 85, 66, 27, 84];
    let mut modified = original.clone();
    remove_items_from_vec_by_indices(&mut modified, &hashset! { 3 });
    assert_eq!(&modified[..3], &original[..3]);
    assert_eq!(&modified[3..], &original[4..]);
}

#[test]
fn remove_multiple() {
    let original = vec![31, 54, 22, 81, 67, 45, 52, 20, 85, 66, 27, 84];
    let mut modified = original.clone();
    remove_items_from_vec_by_indices(&mut modified, &hashset! { 3, 4, 5, 7 });
    assert_eq!(&modified[..3], &original[..3]);
    assert_eq!(&modified[3..4], &original[6..7]);
    assert_eq!(&modified[4..], &original[8..]);
}

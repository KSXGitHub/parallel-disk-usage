use super::LinkPathList;
use pipe_trait::Pipe;
use pretty_assertions::{assert_eq, assert_ne};

#[test]
fn item_order_is_irrelevant_to_equality() {
    let first_order = ["3", "4", "0", "2", "1"]
        .pipe(LinkPathList::many)
        .into_reflection();
    let second_order = ["4", "0", "3", "2", "1"]
        .pipe(LinkPathList::many)
        .into_reflection();
    let sorted_order = ["0", "1", "2", "3", "4"]
        .pipe(LinkPathList::many)
        .into_reflection();
    assert_eq!(first_order, second_order);
    assert_eq!(second_order, sorted_order);
    assert_eq!(first_order, sorted_order);
}

#[test]
fn item_absent_cause_inequality() {
    let without_last = ["0", "1", "2", "3"]
        .pipe(LinkPathList::many)
        .into_reflection();
    let with_last = ["0", "1", "2", "3", "4"]
        .pipe(LinkPathList::many)
        .into_reflection();
    assert_ne!(without_last, with_last);
    assert_ne!(with_last, without_last);
}

#[test]
fn item_difference_cause_inequality() {
    let with_five = ["0", "1", "2", "3", "5"]
        .pipe(LinkPathList::many)
        .into_reflection();
    let with_four = ["0", "1", "2", "3", "4"]
        .pipe(LinkPathList::many)
        .into_reflection();
    assert_ne!(with_five, with_four);
    assert_ne!(with_four, with_five);
}

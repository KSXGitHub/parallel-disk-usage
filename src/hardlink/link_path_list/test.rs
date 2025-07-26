use super::LinkPathList;
use pipe_trait::Pipe;
use pretty_assertions::{assert_eq, assert_ne};

#[test]
fn item_order_is_irrelevant_to_equality() {
    let a = ["3", "4", "0", "2", "1"]
        .pipe(LinkPathList::many)
        .into_reflection();
    let b = ["4", "0", "3", "2", "1"]
        .pipe(LinkPathList::many)
        .into_reflection();
    let c = ["0", "1", "2", "3", "4"]
        .pipe(LinkPathList::many)
        .into_reflection();
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a, c);
}

#[test]
fn item_absent_cause_inequality() {
    let a = ["0", "1", "2", "3"]
        .pipe(LinkPathList::many)
        .into_reflection();
    let b = ["0", "1", "2", "3", "4"]
        .pipe(LinkPathList::many)
        .into_reflection();
    assert_ne!(a, b);
    assert_ne!(b, a);
}

#[test]
fn item_difference_cause_inequality() {
    let a = ["0", "1", "2", "3", "5"]
        .pipe(LinkPathList::many)
        .into_reflection();
    let b = ["0", "1", "2", "3", "4"]
        .pipe(LinkPathList::many)
        .into_reflection();
    assert_ne!(a, b);
    assert_ne!(b, a);
}

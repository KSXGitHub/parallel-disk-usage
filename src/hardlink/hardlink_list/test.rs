use super::{AddError, HardlinkList, NumberOfLinksConflictError, SizeConflictError};
use crate::size::Bytes;
use pipe_trait::Pipe;
use pretty_assertions::{assert_eq, assert_ne};

const TABLE: &[(u64, u64, u64, u64, &str)] = &[
    // dev, ino, size, links, path
    (0, 241, 3652, 1, "a"),
    (0, 569, 2210, 1, "b"),
    (0, 110, 2350, 3, "c"),
    (0, 110, 2350, 3, "c1"),
    (0, 778, 1110, 1, "d"),
    (0, 274, 6060, 2, "e"),
    (0, 274, 6060, 2, "e1"),
    (0, 883, 4530, 1, "f"),
];

fn add<const ROW: usize>(list: HardlinkList<Bytes>) -> HardlinkList<Bytes> {
    let values = TABLE[ROW];
    let (dev, ino, size, links, path) = values;
    if let Err(error) = list.add(ino.into(), dev, size.into(), links, path.as_ref()) {
        panic!("Failed to add {values:?} (index: {ROW}) to the list: {error}");
    }
    list
}

#[test]
fn insertion_order_is_irrelevant_to_equality() {
    let a = HardlinkList::new()
        .pipe(add::<3>)
        .pipe(add::<1>)
        .pipe(add::<4>)
        .pipe(add::<6>)
        .pipe(add::<5>)
        .pipe(add::<0>)
        .pipe(add::<7>)
        .pipe(add::<2>)
        .into_reflection();

    let b = HardlinkList::new()
        .pipe(add::<5>)
        .pipe(add::<6>)
        .pipe(add::<2>)
        .pipe(add::<0>)
        .pipe(add::<1>)
        .pipe(add::<3>)
        .pipe(add::<7>)
        .pipe(add::<4>)
        .into_reflection();

    let c = HardlinkList::new()
        .pipe(add::<0>)
        .pipe(add::<1>)
        .pipe(add::<2>)
        .pipe(add::<3>)
        .pipe(add::<4>)
        .pipe(add::<5>)
        .pipe(add::<6>)
        .pipe(add::<7>)
        .into_reflection();

    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a, c);
}

#[test]
fn omitting_insertion_cause_inequality() {
    let a = HardlinkList::new()
        .pipe(add::<0>)
        .pipe(add::<1>)
        .pipe(add::<2>)
        .pipe(add::<3>)
        .pipe(add::<4>)
        .pipe(add::<5>)
        .pipe(add::<6>)
        .pipe(add::<7>)
        .into_reflection();

    let b = HardlinkList::new()
        .pipe(add::<0>)
        .pipe(add::<1>)
        .pipe(add::<2>)
        .pipe(add::<3>)
        .pipe(add::<4>)
        .pipe(add::<5>)
        .pipe(add::<7>)
        .into_reflection();

    assert_ne!(a, b);
    assert_ne!(b, a);
}

#[test]
fn insertion_difference_cause_inequality() {
    let a = HardlinkList::new()
        .pipe(add::<0>)
        .pipe(add::<1>)
        .pipe(add::<2>)
        .pipe(add::<3>)
        .pipe(add::<4>)
        .pipe(add::<5>)
        .pipe(add::<6>)
        .into_reflection();

    let b = HardlinkList::new()
        .pipe(add::<0>)
        .pipe(add::<1>)
        .pipe(add::<2>)
        .pipe(add::<3>)
        .pipe(add::<4>)
        .pipe(add::<5>)
        .pipe(add::<7>)
        .into_reflection();

    assert_ne!(a, b);
    assert_ne!(b, a);
}

#[test]
fn detect_size_change() {
    let list = HardlinkList::<Bytes>::new();
    list.add(123.into(), 0, 100.into(), 1, "a".as_ref())
        .expect("add the first path");
    let actual = list
        .add(123.into(), 0, 110.into(), 1, "b".as_ref())
        .expect_err("add the second path");
    let expected = AddError::SizeConflict(SizeConflictError {
        ino: 123.into(),
        recorded: 100.into(),
        detected: 110.into(),
    });
    assert_eq!(actual, expected);
}

#[test]
fn detect_number_of_links_change() {
    let list = HardlinkList::<Bytes>::new();
    list.add(123.into(), 0, 100.into(), 1, "a".as_ref())
        .expect("add the first path");
    let actual = list
        .add(123.into(), 0, 100.into(), 2, "b".as_ref())
        .expect_err("add the second path");
    let expected = AddError::NumberOfLinksConflict(NumberOfLinksConflictError {
        ino: 123.into(),
        recorded: 1,
        detected: 2,
    });
    assert_eq!(actual, expected);
}

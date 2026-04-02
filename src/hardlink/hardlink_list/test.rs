use super::{AddError, HardlinkList, NumberOfLinksConflictError, SizeConflictError};
use crate::size::Bytes;
use pipe_trait::Pipe;
use pretty_assertions::{assert_eq, assert_ne};

const TABLE: &[(u64, u64, u64, u64, &str)] = &[
    // ino, dev, size, links, path
    (241, 0, 3652, 1, "a"),
    (569, 0, 2210, 1, "b"),
    (110, 0, 2350, 3, "c"),
    (110, 0, 2350, 3, "c1"),
    (778, 0, 1110, 1, "d"),
    (274, 0, 6060, 2, "e"),
    (274, 0, 6060, 2, "e1"),
    (883, 0, 4530, 1, "f"),
];

fn add<const ROW: usize>(list: HardlinkList<Bytes>) -> HardlinkList<Bytes> {
    let values = TABLE[ROW];
    let (ino, dev, size, links, path) = values;
    if let Err(error) = list.add(ino.into(), dev.into(), size.into(), links, path.as_ref()) {
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
    list.add(123.into(), 0.into(), 100.into(), 1, "a".as_ref())
        .expect("add the first path");
    let actual = list
        .add(123.into(), 0.into(), 110.into(), 1, "b".as_ref())
        .expect_err("add the second path");
    let expected = AddError::SizeConflict(SizeConflictError {
        ino: 123.into(),
        dev: 0.into(),
        recorded: 100.into(),
        detected: 110.into(),
    });
    assert_eq!(actual, expected);
}

#[test]
fn detect_number_of_links_change() {
    let list = HardlinkList::<Bytes>::new();
    list.add(123.into(), 0.into(), 100.into(), 1, "a".as_ref())
        .expect("add the first path");
    let actual = list
        .add(123.into(), 0.into(), 100.into(), 2, "b".as_ref())
        .expect_err("add the second path");
    let expected = AddError::NumberOfLinksConflict(NumberOfLinksConflictError {
        ino: 123.into(),
        dev: 0.into(),
        recorded: 1,
        detected: 2,
    });
    assert_eq!(actual, expected);
}

#[test]
fn same_ino_on_different_devices_are_treated_separately() {
    let list = HardlinkList::<Bytes>::new();

    // dev=1, ino=100 — first filesystem
    list.add(100.into(), 1.into(), 50.into(), 2, "dev1/file_a".as_ref())
        .expect("add dev1/file_a");
    list.add(100.into(), 1.into(), 50.into(), 2, "dev1/file_b".as_ref())
        .expect("add dev1/file_b (same dev+ino → same inode group)");

    // dev=2, ino=100 — second filesystem, coincidentally same inode number
    list.add(100.into(), 2.into(), 80.into(), 2, "dev2/file_c".as_ref())
        .expect("add dev2/file_c (different dev → separate inode group)");
    list.add(100.into(), 2.into(), 80.into(), 2, "dev2/file_d".as_ref())
        .expect("add dev2/file_d (same dev+ino → same inode group as file_c)");

    // Each device should produce its own entry, so the list should have 2 entries.
    assert_eq!(list.len(), 2, "expected one entry per (ino, dev) pair");

    let reflection = list.into_reflection();
    assert_eq!(reflection.len(), 2);

    // Sorted by (ino, dev), so dev=1 comes first.
    let entries: Vec<_> = reflection.iter().collect();
    assert_eq!(entries[0].dev, 1.into());
    assert_eq!(entries[0].ino, 100.into());
    assert_eq!(entries[0].paths.len(), 2);
    assert_eq!(entries[1].dev, 2.into());
    assert_eq!(entries[1].ino, 100.into());
    assert_eq!(entries[1].paths.len(), 2);
}

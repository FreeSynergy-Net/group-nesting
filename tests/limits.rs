//! Depth limit: 16 edges pass, 17 do not, in every mode.
//!
//! Pattern: none (test file).
#![allow(clippy::assert_is_empty)]

mod common;
use common::tree;
use group_nesting::{Group, MAX_NESTING_DEPTH, Mode, NestingError};

const NAMES: [&str; 20] = [
    "g0", "g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8", "g9", "g10", "g11", "g12", "g13", "g14",
    "g15", "g16", "g17", "g18", "g19",
];

fn chain(mode: Mode, edges: usize) -> Result<common::T, NestingError<&'static str>> {
    let mut t = tree(&NAMES[..=edges]);
    for i in 0..edges {
        t.link(&NAMES[i], &NAMES[i + 1], mode)?;
    }
    Ok(t)
}

#[test]
fn limit_is_sixteen_edges() {
    assert_eq!(MAX_NESTING_DEPTH, 16);
    for mode in [Mode::BottomUp, Mode::TopDown, Mode::Single] {
        assert!(chain(mode, 16).is_ok(), "16 edges in {mode:?}");
        assert_eq!(
            chain(mode, 17).err(),
            Some(NestingError::TooDeep { max: 16 }),
            "17 edges in {mode:?}"
        );
    }
}

/// Edge `i` joins `NAMES[i]` and `NAMES[i + 1]`; even edges are `BottomUp`, odd ones `TopDown`, so the
/// membership graph is one long chain `g0 -> g1 -> ...` while the tree is only one level deep.
fn zigzag(i: usize) -> (&'static str, &'static str, Mode) {
    if i.is_multiple_of(2) {
        (NAMES[i], NAMES[i + 1], Mode::BottomUp)
    } else {
        (NAMES[i + 1], NAMES[i], Mode::TopDown)
    }
}

#[test]
fn tree_chain_counts_even_when_modes_alternate() {
    let mut t = tree(&NAMES[..=17]);
    let mut result = Ok(());
    for i in 0..17_usize {
        let mode = if i.is_multiple_of(2) {
            Mode::BottomUp
        } else {
            Mode::TopDown
        };
        result = t.link(&NAMES[i], &NAMES[i + 1], mode).map(|_| ());
    }
    assert_eq!(result, Err(NestingError::TooDeep { max: 16 }));
}

#[test]
fn membership_chain_is_limited_although_the_tree_is_flat() {
    let mut t = tree(&NAMES[..=17]);
    for i in 0..16_usize {
        let (lower, upper, mode) = zigzag(i);
        t.link(&lower, &upper, mode).unwrap();
    }
    let (lower, upper, mode) = zigzag(16);
    assert_eq!(
        t.link(&lower, &upper, mode),
        Err(NestingError::TooDeep { max: 16 })
    );
}

#[test]
fn change_mode_is_checked_against_depth() {
    let mut t = tree(&NAMES[..=17]);
    for i in 0..17_usize {
        let (lower, upper, mode) = zigzag(i);
        let mode = if i == 8 { Mode::Single } else { mode };
        t.link(&lower, &upper, mode).unwrap();
    }
    let (lower, upper, mode) = zigzag(8);
    assert_eq!(
        t.change_mode(&lower, &upper, mode),
        Err(NestingError::TooDeep { max: 16 })
    );
    assert_eq!(t.edges().len(), 17);
}

#[test]
fn adding_a_parent_above_the_top_is_too_deep() {
    let mut t = chain(Mode::Single, 16).unwrap();
    t.add_group(Group::new("extra")).unwrap();
    assert_eq!(
        t.link(&"g16", &"extra", Mode::Single),
        Err(NestingError::TooDeep { max: 16 })
    );
    t.link(&"extra", &"g16", Mode::Single).unwrap();
}

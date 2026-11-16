//! `nesting_effect`: numbers for the consent dialog.
//!
//! Pattern: none (test file).
#![allow(clippy::assert_is_empty)]

mod common;
use common::{human, tree};
use group_nesting::{Mode, NestingEffect, NestingError};

#[test]
fn counts_groups_and_people_over_the_whole_closure() {
    let mut t = tree(&["top", "mid", "low", "other"]);
    human(&mut t, "low", "a");
    human(&mut t, "low", "b");
    human(&mut t, "other", "c");
    t.link(&"mid", &"top", Mode::BottomUp).unwrap();
    let e = t.nesting_effect(&"low", &"mid", Mode::BottomUp).unwrap();
    // mid and top gain a and b.
    assert_eq!(
        e,
        NestingEffect {
            mode: Mode::BottomUp,
            groups_affected: 2,
            persons_gained: 2,
            persons_lost: 0
        }
    );
    assert!(t.edges().len() == 1, "the tree itself is untouched");
}

#[test]
fn mode_change_can_lose_people() {
    let mut t = tree(&["up", "down"]);
    human(&mut t, "down", "a");
    t.link(&"down", &"up", Mode::BottomUp).unwrap();
    let e = t.nesting_effect(&"down", &"up", Mode::Single).unwrap();
    assert_eq!(
        (e.groups_affected, e.persons_gained, e.persons_lost),
        (1, 0, 1)
    );
    let e = t.nesting_effect(&"down", &"up", Mode::TopDown).unwrap();
    assert_eq!(
        (e.groups_affected, e.persons_gained, e.persons_lost),
        (1, 0, 1)
    );
}

#[test]
fn refused_changes_have_no_effect() {
    let mut t = tree(&["a", "b"]);
    t.link(&"b", &"a", Mode::BottomUp).unwrap();
    assert!(matches!(
        t.nesting_effect(&"a", &"b", Mode::Single),
        Err(NestingError::Cycle { .. })
    ));
    assert!(matches!(
        t.nesting_effect(&"b", &"a", Mode::BottomUp),
        Err(NestingError::AlreadyThere { .. })
    ));
}

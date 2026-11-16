//! Chefs and deputies: eight levels of uneven depth, in both directions.
//!
//! Pattern: none (test file).
#![allow(clippy::assert_is_empty)]

mod common;
use common::{directory_members, human, people, tree};
use group_nesting::Mode;

const LEVELS: [&str; 8] = ["l1", "l2", "l3", "l4", "l5", "l6", "l7", "chefs"];

/// l1 < l2 < ... < chefs (chefs on top), plus uneven side branches.
fn org(mode: Mode) -> common::T {
    let mut t = tree(&LEVELS);
    for g in ["side5a", "side5b", "side2", "side2x"] {
        t.add_group(group_nesting::Group::new(g)).unwrap();
    }
    for pair in LEVELS.windows(2) {
        t.link(&pair[0], &pair[1], mode).unwrap();
    }
    // Uneven: a branch two levels deep under l5, one level under l2.
    t.link(&"side5a", &"l5", mode).unwrap();
    t.link(&"side5b", &"side5a", mode).unwrap();
    t.link(&"side2", &"l2", mode).unwrap();
    t.link(&"side2x", &"side2", mode).unwrap();
    for g in LEVELS
        .iter()
        .chain(&["side5a", "side5b", "side2", "side2x"])
    {
        let p: &'static str = Box::leak(format!("p-{g}").into_boxed_str());
        human(&mut t, g, p);
    }
    t
}

#[test]
fn top_down_chefs_are_members_of_everything_below() {
    let t = org(Mode::TopDown);
    // Chefs (top) flow down: chefs' member shows up in every level below and in the side branches.
    for g in ["l7", "l1", "side5a", "side5b", "side2", "side2x"] {
        assert!(people(&t, g).contains(&"p-chefs"), "chefs missing in {g}");
    }
    // Nothing flows up: chefs only have their own member.
    assert_eq!(people(&t, "chefs"), ["p-chefs"]);
    // l1 sees everybody above it on its own line, not the side branches of other lines.
    let l1 = people(&t, "l1");
    assert!(l1.contains(&"p-l7") && l1.contains(&"p-l2"));
    assert!(!l1.contains(&"p-side5a") && !l1.contains(&"p-side2"));
    assert_eq!(t.edges().len(), 11);
    for g in ["l1", "l4", "side5b", "chefs"] {
        let want = directory_members(&t, g);
        assert_eq!(want.into_iter().collect::<Vec<_>>(), people(&t, g));
    }
}

#[test]
fn bottom_up_the_same_tree_counts_upward() {
    let t = org(Mode::BottomUp);
    let chefs = people(&t, "chefs");
    assert_eq!(chefs.len(), 12, "everybody counts for the top group");
    assert_eq!(people(&t, "l1"), ["p-l1"]);
    let l5 = people(&t, "l5");
    assert!(l5.contains(&"p-side5b") && l5.contains(&"p-side2x") && !l5.contains(&"p-l6"));
    for g in ["l3", "side5a", "chefs"] {
        let want = directory_members(&t, g);
        assert_eq!(want.into_iter().collect::<Vec<_>>(), people(&t, g));
    }
}

#[test]
fn rights_direction_is_the_opposite_of_membership() {
    // "A right granted to l7 applies to l7's effective members": Chefs are in there under TopDown.
    let t = org(Mode::TopDown);
    assert!(people(&t, "l7").contains(&"p-chefs"));
    // Under BottomUp the right of the upper group applies to everybody below: l1 is in chefs.
    let u = org(Mode::BottomUp);
    assert!(people(&u, "chefs").contains(&"p-l1"));
}

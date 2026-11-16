//! Cycles, self edges, double edges, diamonds.
//!
//! Pattern: none (test file).
#![allow(clippy::assert_is_empty)]

mod common;
use common::{human, memberof, people, tree};
use group_nesting::{MemberOf, Mode, NestingError};

#[test]
fn cycle_over_mixed_directions_is_found_after_the_reversal() {
    // No cycle in the tree, but a cycle in the membership graph:
    // c -> p1 (BottomUp), p1 -> c2 (c2 TopDown under p1), c2 -> p2 (BottomUp), then c TopDown under p2.
    let mut t = tree(&["p1", "p2", "c", "c2"]);
    t.link(&"c", &"p1", Mode::BottomUp).unwrap();
    t.link(&"c2", &"p1", Mode::TopDown).unwrap();
    t.link(&"c2", &"p2", Mode::BottomUp).unwrap();
    let err = t.link(&"c", &"p2", Mode::TopDown).unwrap_err();
    assert_eq!(
        err,
        NestingError::Cycle {
            path: vec!["p2", "c", "p1", "c2", "p2"]
        }
    );
    assert_eq!(t.edges().len(), 3, "nothing written on error");
    // The same pair is fine as Single and as BottomUp.
    t.link(&"c", &"p2", Mode::Single).unwrap();
}

#[test]
fn tree_cycle_in_every_mode_including_single() {
    for first in [Mode::BottomUp, Mode::TopDown, Mode::Single] {
        for second in [Mode::BottomUp, Mode::TopDown, Mode::Single] {
            let mut t = tree(&["a", "b"]);
            t.link(&"a", &"b", first).unwrap();
            assert_eq!(
                t.link(&"b", &"a", second),
                Err(NestingError::Cycle {
                    path: vec!["b", "a", "b"]
                }),
                "{first:?} then reversed {second:?}"
            );
        }
    }
}

#[test]
fn self_edge_is_a_cycle_in_every_mode() {
    for mode in [Mode::BottomUp, Mode::TopDown, Mode::Single] {
        let mut t = tree(&["a"]);
        assert_eq!(
            t.link(&"a", &"a", mode),
            Err(NestingError::Cycle {
                path: vec!["a", "a"]
            })
        );
    }
}

#[test]
fn longer_tree_cycle_names_the_path() {
    let mut t = tree(&["a", "b", "c"]);
    t.link(&"a", &"b", Mode::Single).unwrap();
    t.link(&"b", &"c", Mode::Single).unwrap();
    assert_eq!(
        t.link(&"c", &"a", Mode::Single),
        Err(NestingError::Cycle {
            path: vec!["c", "a", "b", "c"]
        })
    );
}

#[test]
fn double_edge_is_already_there_in_same_or_other_mode() {
    let mut t = tree(&["a", "b"]);
    t.link(&"b", &"a", Mode::BottomUp).unwrap();
    for mode in [Mode::BottomUp, Mode::TopDown, Mode::Single] {
        assert_eq!(
            t.link(&"b", &"a", mode),
            Err(NestingError::AlreadyThere {
                upper: "a",
                lower: "b"
            })
        );
    }
    assert_eq!(t.edges().len(), 1);
}

#[test]
fn diamond_is_allowed_and_counts_each_person_once() {
    let mut t = tree(&["top", "left", "right", "bottom"]);
    human(&mut t, "bottom", "p");
    for m in [Mode::BottomUp, Mode::TopDown] {
        let mut d = t.clone();
        d.link(&"left", &"top", m).unwrap();
        d.link(&"right", &"top", m).unwrap();
        d.link(&"bottom", &"left", m).unwrap();
        d.link(&"bottom", &"right", m).unwrap();
        let top = d.effective_members(&"top");
        match m {
            Mode::BottomUp => {
                assert_eq!(top.len(), 1);
                assert_eq!(top[0].inherited.len(), 1, "one source group, shortest path");
                assert_eq!(top[0].inherited[0].via.len(), 3);
            }
            _ => assert!(top.is_empty()),
        }
    }
    t.link(&"left", &"top", Mode::BottomUp).unwrap();
}

#[test]
fn kanidm_itself_accepts_cycles_which_is_why_we_refuse_them() {
    let cyc = [
        MemberOf {
            member: "a",
            of: "b",
        },
        MemberOf {
            member: "b",
            of: "c",
        },
        MemberOf {
            member: "c",
            of: "a",
        },
    ];
    let closure = memberof(&cyc);
    // Every group of the cycle becomes a member of every group, itself included.
    for g in ["a", "b", "c"] {
        assert_eq!(closure[g].len(), 3, "{g}");
    }
    let mut t = tree(&["a", "b", "c"]);
    t.link(&"a", &"b", Mode::BottomUp).unwrap();
    t.link(&"b", &"c", Mode::BottomUp).unwrap();
    assert!(t.link(&"c", &"a", Mode::BottomUp).is_err());
    assert!(people(&t, "a").is_empty());
}

#[test]
fn old_data_with_a_cycle_terminates_and_is_reported() {
    use group_nesting::{Edge, Finding, Group, Kind};
    let t: group_nesting::Tree<&str, &str> = group_nesting::Tree::from_parts(
        Mode::BottomUp,
        vec![Group::new("a"), Group::new("b")],
        vec![
            Edge::new("a", "b", Mode::BottomUp),
            Edge::new("b", "a", Mode::BottomUp),
        ],
        vec![("x", Kind::Human)],
        vec![("a", "x")],
    );
    assert_eq!(people(&t, "b"), ["x"]);
    assert_eq!(people(&t, "a"), ["x"]);
    assert_eq!(t.audit().len(), 2);
    assert!(
        t.audit()
            .iter()
            .all(|f| matches!(f, Finding::EdgeInvalid { .. }))
    );
}

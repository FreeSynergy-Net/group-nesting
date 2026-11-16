//! The projection onto Kanidm `memberof`, the dry-run plan, and the closure cross-check.
//!
//! Pattern: none (test file).
#![allow(clippy::assert_is_empty)]

mod common;
use common::{directory_members, human, memberof, people, tree};
use group_nesting::{Group, MemberOf, MirrorFinding, Mode};

fn m(member: &'static str, of: &'static str) -> MemberOf<&'static str> {
    MemberOf { member, of }
}

#[test]
fn native_per_mode() {
    let mut t = tree(&["a", "b", "c", "d"]);
    t.link(&"b", &"a", Mode::BottomUp).unwrap();
    t.link(&"c", &"a", Mode::TopDown).unwrap();
    t.link(&"d", &"a", Mode::Single).unwrap();
    assert_eq!(t.kanidm_projection(), [m("a", "c"), m("b", "a")]);
}

#[test]
fn plan_adds_removes_and_reports_without_silent_removal() {
    let mut t = tree(&["a", "b", "c", "x"]);
    t.link(&"b", &"a", Mode::BottomUp).unwrap();
    t.link(&"c", &"b", Mode::Single).unwrap();
    // The directory has b in a (right), c in b (we wrote it earlier, now Single), x in a (by hand),
    // and a group the tree does not know.
    let current = [m("b", "a"), m("c", "b"), m("x", "a"), m("ghost", "a")];
    let managed = [m("b", "a"), m("c", "b")];
    let plan = t.mirror_plan(&current, &managed);
    assert!(plan.add.is_empty());
    assert_eq!(plan.remove, [m("c", "b")]);
    assert_eq!(
        plan.findings,
        [MirrorFinding::ForeignNesting {
            outer: "a",
            inner: "x"
        }]
    );
    // Empty directory: everything wanted is added.
    assert_eq!(t.mirror_plan(&[], &[]).add, [m("b", "a")]);
}

#[test]
fn unmirrored_group_leaves_a_finding_not_a_silent_gap() {
    let mut t = tree(&["a"]);
    t.add_group(Group::new("b").unmirrored()).unwrap();
    t.link(&"b", &"a", Mode::BottomUp).unwrap();
    let plan = t.mirror_plan(&[], &[]);
    assert!(plan.add.is_empty());
    assert_eq!(
        plan.findings,
        [MirrorFinding::NestedNotMirrored {
            outer: "a",
            inner: "b"
        }]
    );
}

#[test]
fn directory_closure_equals_effective_members_on_nine_plus_levels() {
    for mode in [Mode::BottomUp, Mode::TopDown] {
        let ids = [
            "n0", "n1", "n2", "n3", "n4", "n5", "n6", "n7", "n8", "n9", "n10",
        ];
        let mut t = tree(&ids);
        for (i, id) in ids.iter().enumerate() {
            let p: &'static str = Box::leak(format!("person{i}").into_boxed_str());
            human(&mut t, id, p);
        }
        for pair in ids.windows(2) {
            t.link(&pair[0], &pair[1], mode).unwrap();
        }
        for g in ids {
            let want: Vec<_> = directory_members(&t, g).into_iter().collect();
            assert_eq!(want, people(&t, g), "{mode:?} {g}");
        }
        let top = people(&t, "n10").len();
        assert_eq!(top, if mode == Mode::BottomUp { 11 } else { 1 });
    }
}

#[test]
fn closure_reaches_a_fixed_point_on_a_diamond_with_mixed_modes() {
    let mut t = tree(&["top", "l", "r", "bottom", "side"]);
    for (g, p) in [("bottom", "p1"), ("l", "p2"), ("side", "p3")] {
        human(&mut t, g, p);
    }
    t.link(&"l", &"top", Mode::BottomUp).unwrap();
    t.link(&"r", &"top", Mode::BottomUp).unwrap();
    t.link(&"bottom", &"l", Mode::BottomUp).unwrap();
    t.link(&"bottom", &"r", Mode::BottomUp).unwrap();
    t.link(&"side", &"top", Mode::TopDown).unwrap();
    let closure = memberof(&t.kanidm_projection());
    assert_eq!(closure["bottom"].len(), 4, "l, r, top, side");
    for g in ["top", "l", "r", "bottom", "side"] {
        let want: Vec<_> = directory_members(&t, g).into_iter().collect();
        assert_eq!(want, people(&t, g), "{g}");
    }
}

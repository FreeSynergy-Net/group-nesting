//! The three modes: who is an effective member where, and what the directory gets.
//!
//! Pattern: none (test file).
#![allow(clippy::assert_is_empty)]

mod common;
use common::{human, people, tree};
use group_nesting::{MemberOf, Mode, NestingError};

#[test]
fn bottom_up_members_count_upward() {
    let mut t = tree(&["region", "team"]);
    human(&mut t, "team", "anna");
    human(&mut t, "region", "ben");
    t.link(&"team", &"region", Mode::BottomUp).unwrap();
    assert_eq!(people(&t, "region"), ["anna", "ben"]);
    assert_eq!(people(&t, "team"), ["anna"]);
    assert_eq!(
        t.kanidm_projection(),
        [MemberOf {
            member: "team",
            of: "region"
        }]
    );
}

#[test]
fn top_down_membership_flows_down() {
    let mut t = tree(&["chefs", "vize"]);
    human(&mut t, "chefs", "carla");
    human(&mut t, "vize", "dirk");
    t.link(&"vize", &"chefs", Mode::TopDown).unwrap();
    // The upper group stands in `member` of the lower: Chefs are effective members of Vize.
    assert_eq!(people(&t, "vize"), ["carla", "dirk"]);
    assert_eq!(people(&t, "chefs"), ["carla"]);
    assert_eq!(
        t.kanidm_projection(),
        [MemberOf {
            member: "chefs",
            of: "vize"
        }]
    );
}

#[test]
fn single_is_display_only() {
    let mut t = tree(&["a", "b"]);
    human(&mut t, "a", "x");
    human(&mut t, "b", "y");
    t.link(&"b", &"a", Mode::Single).unwrap();
    assert_eq!(people(&t, "a"), ["x"]);
    assert_eq!(people(&t, "b"), ["y"]);
    assert!(t.kanidm_projection().is_empty());
    assert_eq!(t.edges().len(), 1);
}

#[test]
fn origin_is_direct_inherited_or_both() {
    let mut t = tree(&["top", "mid", "low"]);
    human(&mut t, "low", "p");
    human(&mut t, "mid", "p");
    t.link(&"low", &"mid", Mode::BottomUp).unwrap();
    t.link(&"mid", &"top", Mode::BottomUp).unwrap();
    let m = t.effective_members(&"top");
    assert_eq!(m.len(), 1, "each person once");
    assert!(!m[0].direct);
    assert_eq!(m[0].inherited[0].from, Some("mid"));
    assert_eq!(m[0].inherited[0].via, ["mid", "top"]);
    assert_eq!(m[0].inherited[1].via, ["low", "mid", "top"]);
    let own = t.effective_members(&"mid");
    assert!(own[0].direct);
    assert_eq!(own[0].inherited.len(), 1);
}

#[test]
fn concealed_origin_hides_source_and_path() {
    let mut t = tree(&["top", "mid", "low"]);
    human(&mut t, "low", "p");
    t.link(&"low", &"mid", Mode::BottomUp).unwrap();
    t.link(&"mid", &"top", Mode::BottomUp).unwrap();
    let origin = &t.effective_members(&"top")[0].inherited[0];
    let all = origin.concealed(|_| true);
    assert_eq!(all, *origin);
    let no_low = origin.concealed(|g| *g != "low");
    assert_eq!((no_low.from, no_low.via.len()), (None, 0));
    let no_mid = origin.concealed(|g| *g != "mid");
    assert_eq!((no_mid.from, no_mid.via.len()), (Some("low"), 0));
}

#[test]
fn mixed_paths_give_sibling_effect_like_kanidm() {
    // P has a BottomUp child C1 and a TopDown child C2: C1 -> P -> C2.
    let mut t = tree(&["p", "c1", "c2"]);
    human(&mut t, "c1", "x");
    t.link(&"c1", &"p", Mode::BottomUp).unwrap();
    t.link(&"c2", &"p", Mode::TopDown).unwrap();
    assert_eq!(people(&t, "c2"), ["x"]);
    assert_eq!(people(&t, "p"), ["x"]);
}

#[test]
fn default_mode_and_mode_change() {
    let mut t = tree(&["a", "b"]);
    assert_eq!(t.default_mode(), Mode::BottomUp);
    assert_eq!(t.link_default(&"b", &"a").unwrap().mode, Mode::BottomUp);
    assert_eq!(
        t.change_mode(&"b", &"a", Mode::BottomUp),
        Err(NestingError::AlreadyThere {
            upper: "a",
            lower: "b"
        })
    );
    t.change_mode(&"b", &"a", Mode::TopDown).unwrap();
    assert_eq!(
        t.kanidm_projection(),
        [MemberOf {
            member: "a",
            of: "b"
        }]
    );
    t.change_mode(&"b", &"a", Mode::Single).unwrap();
    assert!(t.kanidm_projection().is_empty());
    assert_eq!(
        t.change_mode(&"a", &"b", Mode::Single),
        Err(NestingError::NotLinked {
            upper: "b",
            lower: "a"
        })
    );
}

#[test]
fn unlink_and_unknown_groups() {
    let mut t = tree(&["a", "b"]);
    t.link(&"b", &"a", Mode::BottomUp).unwrap();
    t.unlink(&"b", &"a").unwrap();
    assert!(t.edges().is_empty());
    assert!(t.unlink(&"b", &"a").is_err());
    assert_eq!(
        t.link(&"z", &"a", Mode::Single),
        Err(NestingError::UnknownGroup("z"))
    );
    assert!(t.effective_members(&"z").is_empty());
    assert_eq!(
        t.add_group(group_nesting::Group::new("a")),
        Err(NestingError::DuplicateGroup("a"))
    );
}

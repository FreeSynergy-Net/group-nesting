//! The AI rule: AI accounts only in AI groups, AI groups only as `Single` leaves.
//!
//! Pattern: none (test file).
#![allow(clippy::assert_is_empty)]

mod common;
use common::{human, people, tree};
use group_nesting::{Edge, Finding, Group, InForce, Kind, MirrorFinding, Mode, NestingError, Tree};

fn with_ai() -> common::T {
    let mut t = tree(&["team", "all"]);
    t.add_group(Group::new("bots").ai()).unwrap();
    t.set_kind("bot", Kind::Ai);
    t
}

#[test]
fn accounts_must_match_the_group_kind() {
    let mut t = with_ai();
    t.set_kind("anna", Kind::Human);
    assert_eq!(
        t.add_member(&"team", "bot"),
        Err(NestingError::AiNesting {
            group: "team",
            path: vec!["team"]
        })
    );
    assert_eq!(
        t.add_member(&"bots", "anna"),
        Err(NestingError::AiNesting {
            group: "bots",
            path: vec!["bots"]
        })
    );
    assert_eq!(
        t.add_member(&"bots", "who"),
        Err(NestingError::AiStatusUnknown { group: "bots" })
    );
    t.add_member(&"bots", "bot").unwrap();
    t.add_member(&"team", "who").unwrap();
    assert!(
        t.direct_members(&"team").contains(&"who"),
        "unknown stays allowed in human groups"
    );
}

#[test]
fn ai_group_is_never_part_of_an_effective_edge() {
    let mut t = with_ai();
    let bad = |g, path: Vec<&'static str>| Err(NestingError::AiNesting { group: g, path });
    assert_eq!(
        t.link(&"bots", &"all", Mode::BottomUp).map(|_| ()),
        bad("bots", vec!["bots", "all"])
    );
    assert_eq!(
        t.link(&"bots", &"all", Mode::TopDown).map(|_| ()),
        bad("bots", vec!["bots", "all"])
    );
    // As a parent: never, not even Single.
    for m in [Mode::BottomUp, Mode::TopDown, Mode::Single] {
        assert_eq!(
            t.link(&"team", &"bots", m).map(|_| ()),
            bad("bots", vec!["team", "bots"])
        );
    }
    assert!(t.edges().is_empty());
    t.link(&"bots", &"all", Mode::Single).unwrap();
}

#[test]
fn ai_never_inherits_and_never_passes_rights_on() {
    let mut t = with_ai();
    human(&mut t, "all", "anna");
    t.add_member(&"bots", "bot").unwrap();
    t.link(&"bots", &"all", Mode::Single).unwrap();
    assert_eq!(people(&t, "all"), ["anna"]);
    assert_eq!(people(&t, "bots"), ["bot"]);
    assert!(t.kanidm_projection().is_empty());
}

#[test]
fn flag_changes_cannot_bypass_the_rule() {
    let mut t = with_ai();
    human(&mut t, "team", "anna");
    assert!(t.set_ai(&"team", true).is_err(), "a human is in there");
    t.add_member(&"bots", "bot").unwrap();
    assert_eq!(
        t.set_ai(&"bots", false),
        Err(NestingError::AiNesting {
            group: "bots",
            path: vec!["bots"]
        })
    );
    t.link(&"all", &"team", Mode::Single).unwrap();
    t.add_group(Group::new("empty")).unwrap();
    t.link(&"empty", &"all", Mode::BottomUp).unwrap();
    assert!(
        t.set_ai(&"empty", true).is_err(),
        "it has an effective edge"
    );
    assert!(t.set_ai(&"all", true).is_err(), "it has children");
    t.add_group(Group::new("lonely")).unwrap();
    t.set_ai(&"lonely", true).unwrap();
}

#[test]
fn old_data_edge_acts_like_single_and_is_reported() {
    let t: common::T = Tree::from_parts(
        Mode::BottomUp,
        vec![Group::new("all"), Group::new("bots").ai()],
        vec![Edge::new("all", "bots", Mode::BottomUp)],
        vec![("bot", Kind::Ai), ("anna", Kind::Human)],
        vec![("bots", "bot"), ("all", "anna")],
    );
    assert_eq!(
        people(&t, "all"),
        ["anna"],
        "the AI does not reach the human group"
    );
    assert!(t.kanidm_projection().is_empty());
    assert_eq!(
        t.audit(),
        [Finding::HeldBack {
            edge: Edge::new("all", "bots", Mode::BottomUp)
        }]
    );
    let plan = t.mirror_plan(&[], &[]);
    assert!(plan.findings.contains(&MirrorFinding::HeldBack {
        outer: "all",
        inner: "bots"
    }));
    assert!(
        plan.findings
            .contains(&MirrorFinding::AiGroupExcluded("bots"))
    );
}

#[test]
fn re_marking_after_joining_does_not_count() {
    let mut t = tree(&["team", "top"]);
    human(&mut t, "team", "anna");
    t.link(&"team", &"top", Mode::BottomUp).unwrap();
    assert_eq!(people(&t, "top"), ["anna"]);
    t.set_kind("anna", Kind::Ai); // marked as AI afterwards
    assert_eq!(t.in_force(&"team", &"anna"), InForce::KindMismatch);
    assert!(people(&t, "team").is_empty() && people(&t, "top").is_empty());
    assert_eq!(t.direct_members(&"team"), ["anna"], "stays stored");
    assert_eq!(
        t.audit(),
        [Finding::KindMismatch {
            group: "team",
            person: "anna"
        }]
    );
    let plan = t.mirror_plan(&[], &[]);
    assert!(plan.findings.contains(&MirrorFinding::LeftOut {
        group: "team",
        person: "anna"
    }));
    // The other direction: an AI re-marked as human in an AI group.
    let mut u = with_ai();
    u.add_member(&"bots", "bot").unwrap();
    u.set_kind("bot", Kind::Human);
    assert_eq!(u.in_force(&"bots", &"bot"), InForce::KindMismatch);
    assert!(people(&u, "bots").is_empty());
}

#[test]
fn restore_of_wrong_data_counts_nothing_and_does_not_fail() {
    let t: common::T = Tree::from_parts(
        Mode::BottomUp,
        vec![Group::new("team"), Group::new("bots").ai()],
        vec![],
        vec![
            ("bot", Kind::Ai),
            ("anna", Kind::Human),
            ("who", Kind::Unknown),
        ],
        vec![
            ("team", "bot"),
            ("bots", "anna"),
            ("bots", "who"),
            ("team", "anna"),
        ],
    );
    assert_eq!(people(&t, "team"), ["anna"]);
    assert!(people(&t, "bots").is_empty());
    assert_eq!(t.audit().len(), 3);
    assert_eq!(
        t.mirror_plan(&[], &[]).findings.len(),
        4,
        "AiGroupExcluded plus three LeftOut"
    );
}

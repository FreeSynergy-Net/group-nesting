//! Sensitive groups (GDPR Art. 9): no effective edge to a non-sensitive group, either direction.
//!
//! Pattern: none (test file).
#![allow(clippy::assert_is_empty)]

mod common;
use group_nesting::{Conflict, Group, Mode, NestingError, Tree};

fn pair() -> Tree<&'static str, &'static str> {
    let mut t = Tree::new(Mode::BottomUp);
    t.add_group(Group::new("health").sensitive()).unwrap();
    t.add_group(Group::new("open")).unwrap();
    t.add_group(Group::new("health2").sensitive()).unwrap();
    t
}

#[test]
fn both_directions_and_both_modes_are_refused() {
    let flow = |from, to| NestingError::ModeConflict(Conflict::SensitiveFlow { from, to });
    let mut t = pair();
    // BottomUp: lower is a member of upper.
    assert_eq!(
        t.link(&"health", &"open", Mode::BottomUp),
        Err(flow("health", "open"))
    );
    assert_eq!(
        t.link(&"open", &"health", Mode::BottomUp),
        Err(flow("open", "health"))
    );
    // TopDown: upper is a member of lower.
    assert_eq!(
        t.link(&"open", &"health", Mode::TopDown),
        Err(flow("health", "open"))
    );
    assert_eq!(
        t.link(&"health", &"open", Mode::TopDown),
        Err(flow("open", "health"))
    );
    assert!(t.edges().is_empty());
}

#[test]
fn single_is_allowed_and_sensitive_with_sensitive_too() {
    let mut t = pair();
    t.link(&"health", &"open", Mode::Single).unwrap();
    t.link(&"health2", &"health", Mode::BottomUp).unwrap();
    assert!(
        t.kanidm_projection().is_empty(),
        "sensitive groups are never mirrored"
    );
}

#[test]
fn flag_change_and_mode_change_are_checked_too() {
    let mut t = pair();
    t.link(&"health", &"open", Mode::Single).unwrap();
    assert!(t.change_mode(&"health", &"open", Mode::BottomUp).is_err());
    t.link(&"health2", &"health", Mode::BottomUp).unwrap();
    assert!(
        t.set_sensitive(&"health2", false).is_err(),
        "would open a flow"
    );
    assert!(t.group(&"health2").unwrap().sensitive, "unchanged on error");
    t.unlink(&"health2", &"health").unwrap();
    t.set_sensitive(&"health2", false).unwrap();
    assert!(t.link(&"health2", &"health", Mode::BottomUp).is_err());
}

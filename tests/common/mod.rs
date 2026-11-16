//! Shared helpers for the integration tests (not a test itself).
//!
//! Pattern: none (free helper functions).
#![allow(dead_code)]

use group_nesting::{Group, Kind, MemberOf, Mode, Tree};
use std::collections::{BTreeMap, BTreeSet};

pub type T = Tree<&'static str, &'static str>;

/// A tree of ordinary groups with the given ids.
pub fn tree(ids: &[&'static str]) -> T {
    let mut t = Tree::new(Mode::BottomUp);
    for id in ids {
        t.add_group(Group::new(*id)).expect("new group");
    }
    t
}

/// A human who is a direct member.
pub fn human(t: &mut T, group: &'static str, person: &'static str) {
    t.set_kind(person, Kind::Human);
    t.add_member(&group, person).expect("human joins");
}

/// Effective persons of a group, sorted.
pub fn people(t: &T, group: &'static str) -> Vec<&'static str> {
    t.effective_members(&group)
        .into_iter()
        .map(|m| m.person)
        .collect()
}

/// Kanidm's `memberof` closure: for every group, all groups it is (transitively) a member of. Runs
/// to a fixed point like `memberof.rs` does, so cycles terminate.
pub fn memberof(
    edges: &[MemberOf<&'static str>],
) -> BTreeMap<&'static str, BTreeSet<&'static str>> {
    let mut map: BTreeMap<&'static str, BTreeSet<&'static str>> = BTreeMap::new();
    for e in edges {
        map.entry(e.member).or_default().insert(e.of);
    }
    loop {
        let mut changed = false;
        let snapshot = map.clone();
        for (member, of) in &snapshot {
            for o in of {
                for up in snapshot.get(o).into_iter().flatten() {
                    changed |= map.entry(member).or_default().insert(up);
                }
            }
        }
        if !changed {
            return map;
        }
    }
}

/// The persons the directory would show as members of `group`: direct members of the group and of
/// every group that has `group` in its `memberof` closure.
pub fn directory_members(t: &T, group: &'static str) -> BTreeSet<&'static str> {
    let closure = memberof(&t.kanidm_projection());
    let mut out = BTreeSet::new();
    for g in t.groups() {
        let in_group = g.id == group || closure.get(g.id).is_some_and(|s| s.contains(group));
        if in_group {
            out.extend(t.direct_members(&g.id));
        }
    }
    out
}

/// A tiny deterministic generator (xorshift), so the property tests need no outside crate.
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % n as u64).expect("small")
    }
}

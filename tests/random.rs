//! Property tests with a fixed seed and our own tiny generator (no outside crate): random edges
//! and modes are tried; whatever is accepted must keep every invariant, whatever is refused must
//! change nothing.
//!
//! Pattern: none (test file).
#![allow(clippy::assert_is_empty)]

mod common;
use common::{Rng, T, directory_members, people};
use group_nesting::{Group, Kind, MAX_NESTING_DEPTH, Mode, NestingError};
use std::collections::BTreeSet;

const IDS: [&str; 14] = [
    "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n",
];
const PEOPLE: [&str; 8] = ["p0", "p1", "p2", "p3", "p4", "p5", "p6", "p7"];

fn random_tree(rng: &mut Rng) -> T {
    let mut t: T = group_nesting::Tree::new(Mode::BottomUp);
    let n = 4 + rng.below(IDS.len() - 3);
    for id in &IDS[..n] {
        t.add_group(Group::new(*id)).unwrap();
    }
    for p in PEOPLE {
        t.set_kind(p, Kind::Human);
        t.add_member(&IDS[rng.below(n)], p).unwrap();
    }
    t
}

/// Naive Warshall closure over the arcs, independent of the crate's breadth-first search.
fn naive(t: &T, group: &'static str) -> BTreeSet<&'static str> {
    let ids: Vec<&'static str> = t.groups().map(|g| g.id).collect();
    let mut reach: BTreeSet<(&str, &str)> = t
        .edges()
        .iter()
        .filter_map(group_nesting::Edge::arc)
        .collect();
    for k in &ids {
        for i in &ids {
            for j in &ids {
                if reach.contains(&(*i, *k)) && reach.contains(&(*k, *j)) {
                    reach.insert((*i, *j));
                }
            }
        }
    }
    let mut out = BTreeSet::new();
    for g in &ids {
        if *g == group || reach.contains(&(*g, group)) {
            out.extend(t.direct_members(g));
        }
    }
    out
}

#[test]
fn accepted_trees_keep_every_invariant() {
    let mut accepted = 0;
    let mut refused = 0;
    for seed in 1..=150_u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let mut t = random_tree(&mut rng);
        let n = t.groups().count();
        for _ in 0..60 {
            let (lower, upper) = (IDS[rng.below(n)], IDS[rng.below(n)]);
            let mode = [Mode::BottomUp, Mode::TopDown, Mode::Single][rng.below(3)];
            let before = t.edges().to_vec();
            let r = if rng.below(4) == 0 {
                t.change_mode(&lower, &upper, mode)
            } else {
                t.link(&lower, &upper, mode)
            };
            match r {
                Ok(_) => accepted += 1,
                Err(e) => {
                    refused += 1;
                    assert_eq!(t.edges(), before, "seed {seed}: {e} must change nothing");
                    assert!(!matches!(e, NestingError::UnknownGroup(_)));
                }
            }
        }
        assert!(t.audit().is_empty(), "seed {seed}: {:?}", t.audit());
        for g in t.groups().map(|g| g.id).collect::<Vec<_>>() {
            let got: BTreeSet<_> = people(&t, g).into_iter().collect();
            assert_eq!(
                got,
                naive(&t, g),
                "seed {seed} group {g}: breadth-first vs naive"
            );
            assert_eq!(
                got,
                directory_members(&t, g),
                "seed {seed} group {g}: vs memberof"
            );
        }
        assert!(depth_ok(&t), "seed {seed}: depth");
    }
    assert!(
        accepted > 500 && refused > 500,
        "{accepted} accepted, {refused} refused"
    );
}

/// Longest chain of tree edges and of arcs is within the limit (checked by plain recursion on an
/// acyclic graph, which `audit` already proved).
fn depth_ok(t: &T) -> bool {
    fn deepest(t: &T, at: &'static str, arcs: bool, left: u32) -> bool {
        let next: Vec<&'static str> = t
            .edges()
            .iter()
            .filter_map(|e| {
                if arcs {
                    e.arc().filter(|a| a.0 == at).map(|a| a.1)
                } else {
                    (e.lower == at).then_some(e.upper)
                }
            })
            .collect();
        next.iter()
            .all(|n| left > 0 && deepest(t, n, arcs, left - 1))
    }
    t.groups().all(|g| {
        deepest(t, g.id, false, MAX_NESTING_DEPTH) && deepest(t, g.id, true, MAX_NESTING_DEPTH)
    })
}

#[test]
fn effect_matches_the_real_change() {
    for seed in 1..=60_u64 {
        let mut rng = Rng(seed.wrapping_mul(0xD1B5_4A32_D192_ED03));
        let mut t = random_tree(&mut rng);
        let n = t.groups().count();
        for _ in 0..30 {
            let (lower, upper) = (IDS[rng.below(n)], IDS[rng.below(n)]);
            let mode = [Mode::BottomUp, Mode::TopDown, Mode::Single][rng.below(3)];
            let Ok(effect) = t.nesting_effect(&lower, &upper, mode) else {
                continue;
            };
            let before: Vec<_> = IDS[..n].iter().map(|g| people(&t, g)).collect();
            let exists = t
                .edges()
                .iter()
                .any(|e| e.lower == lower && e.upper == upper);
            if exists {
                t.change_mode(&lower, &upper, mode).unwrap();
            } else {
                t.link(&lower, &upper, mode).unwrap();
            }
            let after: Vec<_> = IDS[..n].iter().map(|g| people(&t, g)).collect();
            let changed = before.iter().zip(&after).filter(|(a, b)| a != b).count();
            assert_eq!(effect.groups_affected as usize, changed, "seed {seed}");
        }
    }
}

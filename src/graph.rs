//! Small graph helpers: shortest path, longest chain, adjacency. Safe on cyclic input.
//!
//! Pattern: none (free helper functions over plain pair lists).

use crate::group::GroupId;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Successor lists of a pair list `(from, to)`, in deterministic order.
pub(crate) fn adjacency<G: GroupId>(pairs: &[(G, G)]) -> BTreeMap<G, Vec<G>> {
    let mut map: BTreeMap<G, Vec<G>> = BTreeMap::new();
    for (a, b) in pairs {
        map.entry(a.clone()).or_default().push(b.clone());
    }
    for next in map.values_mut() {
        next.sort();
        next.dedup();
    }
    map
}

/// Shortest path `from` to ... to `to` (both included) over `pairs`, if there is one. A path from a
/// node to itself is just that node.
pub(crate) fn path<G: GroupId>(pairs: &[(G, G)], from: &G, to: &G) -> Option<Vec<G>> {
    let next = adjacency(pairs);
    let mut back: BTreeMap<G, G> = BTreeMap::new();
    let mut seen: BTreeSet<G> = BTreeSet::from([from.clone()]);
    let mut queue = VecDeque::from([from.clone()]);
    while let Some(at) = queue.pop_front() {
        if &at == to {
            let mut way = vec![at.clone()];
            let mut cur = at;
            while let Some(prev) = back.get(&cur) {
                way.push(prev.clone());
                cur = prev.clone();
            }
            way.reverse();
            return Some(way);
        }
        for n in next.get(&at).into_iter().flatten() {
            if seen.insert(n.clone()) {
                back.insert(n.clone(), at.clone());
                queue.push_back(n.clone());
            }
        }
    }
    None
}

/// Number of edges in the longest chain of an acyclic pair list. Nodes on a cycle are skipped (the
/// result never loops); cycles are reported by the checks, not here.
pub(crate) fn longest<G: GroupId>(pairs: &[(G, G)]) -> u32 {
    let next = adjacency(pairs);
    let mut indegree: BTreeMap<&G, usize> = BTreeMap::new();
    for (a, targets) in &next {
        indegree.entry(a).or_insert(0);
        for t in targets {
            *indegree.entry(t).or_insert(0) += 1;
        }
    }
    let mut dist: BTreeMap<&G, u32> = BTreeMap::new();
    let mut queue: VecDeque<&G> = indegree
        .iter()
        .filter(|(_, d)| **d == 0)
        .map(|(g, _)| *g)
        .collect();
    let mut best = 0;
    while let Some(at) = queue.pop_front() {
        let here = dist.get(at).copied().unwrap_or(0);
        best = best.max(here);
        for t in next.get(at).into_iter().flatten() {
            let d = dist.entry(t).or_insert(0);
            *d = (*d).max(here + 1);
            if let Some(n) = indegree.get_mut(t) {
                *n -= 1;
                if *n == 0 {
                    queue.push_back(t);
                }
            }
        }
    }
    best
}

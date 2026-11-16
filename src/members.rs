//! Effective members with their origin.
//!
//! Pattern: Specification (one rule, one computation: a backward breadth-first search over the
//! membership graph) + Value Object (the answer).

use crate::graph::adjacency;
use crate::group::{GroupId, InForce};
use crate::tree::Tree;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// "Inherited from `from`", by the shortest path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inherited<G> {
    /// The group the person is a direct member of; `None` once concealed.
    pub from: Option<G>,
    /// Shortest path in the membership graph, from `from` to the asked group, both included. Empty
    /// once concealed.
    pub via: Vec<G>,
}

impl<G: GroupId> Inherited<G> {
    /// Applies the "hidden source" rule for a viewer who may read only some groups: if the source
    /// is unreadable, `from` is `None`; if any group on the path is unreadable, `via` is empty (the
    /// shape of the tree would otherwise show). The viewer then only sees "inherited".
    #[must_use]
    pub fn concealed(&self, readable: impl Fn(&G) -> bool) -> Self {
        let from = self.from.clone().filter(&readable);
        let via = if self.via.iter().all(&readable) {
            self.via.clone()
        } else {
            Vec::new()
        };
        Self { from, via }
    }
}

/// An effective member: each person once, with the origin(s).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveMember<G, P> {
    /// The person.
    pub person: P,
    /// Whether the person is a direct member of the asked group.
    pub direct: bool,
    /// One entry per source group, shortest path first. Empty for a purely direct member.
    pub inherited: Vec<Inherited<G>>,
}

impl<G: GroupId, P: Clone + Ord> Tree<G, P> {
    /// Effective members of `group`: its direct members and the direct members of every group `X`
    /// with a directed path `X -> ... -> group` in the membership graph (all arcs, `BottomUp` and
    /// `TopDown` may mix, like Kanidm's `memberof`). Memberships of the wrong account kind do not
    /// count. Cost: O(groups + edges + memberships). Unknown group: empty.
    #[must_use]
    pub fn effective_members(&self, group: &G) -> Vec<EffectiveMember<G, P>> {
        if !self.groups.contains_key(group) {
            return Vec::new();
        }
        let reverse: Vec<(G, G)> = self.arcs().into_iter().map(|(a, b)| (b, a)).collect();
        let before = adjacency(&reverse);
        // next[x] = the group after x on the shortest way to `group`.
        let mut next: BTreeMap<G, G> = BTreeMap::new();
        let mut order = vec![group.clone()];
        let mut seen = BTreeSet::from([group.clone()]);
        let mut queue = VecDeque::from([group.clone()]);
        while let Some(at) = queue.pop_front() {
            for x in before.get(&at).into_iter().flatten() {
                if seen.insert(x.clone()) {
                    next.insert(x.clone(), at.clone());
                    order.push(x.clone());
                    queue.push_back(x.clone());
                }
            }
        }
        let mut out: BTreeMap<P, EffectiveMember<G, P>> = BTreeMap::new();
        for source in order {
            let mut via = vec![source.clone()];
            while let Some(n) = next.get(via.last().unwrap_or(&source)) {
                via.push(n.clone());
            }
            for person in self.direct.get(&source).into_iter().flatten() {
                if self.in_force(&source, person) != InForce::Yes {
                    continue;
                }
                let entry = out
                    .entry(person.clone())
                    .or_insert_with(|| EffectiveMember {
                        person: person.clone(),
                        direct: false,
                        inherited: Vec::new(),
                    });
                if source == *group {
                    entry.direct = true;
                } else {
                    entry.inherited.push(Inherited {
                        from: Some(source.clone()),
                        via: via.clone(),
                    });
                }
            }
        }
        out.into_values().collect()
    }
}

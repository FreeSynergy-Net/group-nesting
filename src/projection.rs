//! Projection onto Kanidm `memberof` and the dry-run plan.
//!
//! Pattern: Adapter (tree to directory edges) + Template Method (`mirror_plan`: wanted, compare,
//! report, always in this order). Nothing is removed silently: an edge that somebody made by hand
//! in the directory is a finding, not a step.

use crate::group::GroupId;
use crate::tree::Tree;
use std::collections::BTreeSet;

/// "`member` is a member of `of`" in the directory.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemberOf<G> {
    /// The group that is added to `of` as a member.
    pub member: G,
    /// The group that gets the member.
    pub of: G,
}

/// Something the mirror reports instead of acting silently.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MirrorFinding<G, P> {
    /// The directory has `inner` in `outer` (both known to the tree), but the tree has no such
    /// effective edge. Not removed until a person decides (it may be a hand-made edge).
    ForeignNesting {
        /// The group with the member.
        outer: G,
        /// The nested group.
        inner: G,
    },
    /// The tree has this effective arc, but one end is not mirrored, so the directory lacks it and
    /// its closure is smaller than `effective_members`.
    NestedNotMirrored {
        /// The group with the member.
        outer: G,
        /// The nested group.
        inner: G,
    },
    /// An AI group is never mirrored (the directory holds humans only).
    AiGroupExcluded(G),
    /// An effective edge at an AI group (old data) is not written; it acts like `Single`.
    HeldBack {
        /// The group with the member.
        outer: G,
        /// The nested group.
        inner: G,
    },
    /// A membership of the wrong account kind: left out, it does not count.
    LeftOut {
        /// The group.
        group: G,
        /// The person.
        person: P,
    },
}

/// The dry run: what to add, what to remove, what to look at. Group-in-group edges only; person
/// memberships are the caller's business (`effective_members`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MirrorPlan<G, P> {
    /// Edges to add in the directory.
    pub add: Vec<MemberOf<G>>,
    /// Edges to remove: only those the caller says it wrote itself (`managed`) that are not wanted
    /// any more (mode changed to `Single`, edge unlinked, ...).
    pub remove: Vec<MemberOf<G>>,
    /// Everything that was not done silently.
    pub findings: Vec<MirrorFinding<G, P>>,
}

impl<G: GroupId, P: Clone + Ord> Tree<G, P> {
    fn mirrored(&self, g: &G) -> bool {
        self.groups.get(g).is_some_and(crate::Group::is_mirrored)
    }

    /// The native Kanidm edges for the tree: `BottomUp` = lower in upper, `TopDown` = upper in
    /// lower, `Single` = none. Only arcs between two mirrored groups; sorted, no duplicates.
    #[must_use]
    pub fn kanidm_projection(&self) -> Vec<MemberOf<G>> {
        let set: BTreeSet<MemberOf<G>> = self
            .arcs()
            .into_iter()
            .filter(|(a, b)| self.mirrored(a) && self.mirrored(b))
            .map(|(member, of)| MemberOf { member, of })
            .collect();
        set.into_iter().collect()
    }

    /// Compares the wanted edges with the directory. `current` = the group-in-group edges the
    /// directory has now; `managed` = the edges the caller wrote earlier. Edges with a group the
    /// tree does not know are not looked at (not ours).
    #[must_use]
    pub fn mirror_plan(
        &self,
        current: &[MemberOf<G>],
        managed: &[MemberOf<G>],
    ) -> MirrorPlan<G, P> {
        let wanted: BTreeSet<MemberOf<G>> = self.kanidm_projection().into_iter().collect();
        let have: BTreeSet<&MemberOf<G>> = current.iter().collect();
        let add = wanted
            .iter()
            .filter(|w| !have.contains(w))
            .cloned()
            .collect();
        let (mut remove, mut findings) = (BTreeSet::new(), Vec::new());
        for c in have.into_iter().filter(|c| !wanted.contains(*c)) {
            if managed.contains(c) {
                remove.insert(c.clone());
            } else if self.groups.contains_key(&c.member) && self.groups.contains_key(&c.of) {
                findings.push(MirrorFinding::ForeignNesting {
                    outer: c.of.clone(),
                    inner: c.member.clone(),
                });
            }
        }
        self.report(&mut findings);
        MirrorPlan {
            add,
            remove: remove.into_iter().collect(),
            findings,
        }
    }

    /// Findings that do not depend on the directory.
    fn report(&self, findings: &mut Vec<MirrorFinding<G, P>>) {
        for g in self.groups.values().filter(|g| g.ai) {
            findings.push(MirrorFinding::AiGroupExcluded(g.id.clone()));
        }
        for e in &self.edges {
            let Some((inner, outer)) = e.arc() else {
                continue;
            };
            if self.held_back(e) {
                findings.push(MirrorFinding::HeldBack { outer, inner });
            } else if self.mirrored(&inner) != self.mirrored(&outer) {
                findings.push(MirrorFinding::NestedNotMirrored { outer, inner });
            }
        }
        for (group, people) in &self.direct {
            for person in people {
                if self.in_force(group, person) == crate::InForce::KindMismatch {
                    findings.push(MirrorFinding::LeftOut {
                        group: group.clone(),
                        person: person.clone(),
                    });
                }
            }
        }
    }
}

//! What an edge would change, as numbers (for a consent dialog that shows no names).
//!
//! Pattern: Value Object (the numbers) + Memento-like probe (the change is tried on a copy of the
//! tree, so the real tree is never touched).

use crate::error::NestingError;
use crate::group::GroupId;
use crate::mode::Mode;
use crate::tree::Tree;
use std::collections::{BTreeMap, BTreeSet};

/// The effect of adding an edge or changing its mode, over the whole closure: also groups far
/// above or below and siblings reached over mixed paths. Numbers only: the other side learns
/// nothing about members it may not read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NestingEffect {
    /// The mode asked for.
    pub mode: Mode,
    /// How many groups would get different effective members.
    pub groups_affected: u32,
    /// Persons who would become an effective member of at least one more group.
    pub persons_gained: u32,
    /// Persons who would lose the effective membership of at least one group (mode change).
    pub persons_lost: u32,
}

impl<G: GroupId, P: Clone + Ord> Tree<G, P> {
    fn member_sets(&self) -> BTreeMap<G, BTreeSet<P>> {
        self.groups
            .keys()
            .map(|g| {
                let people = self.effective_members(g).into_iter().map(|m| m.person);
                (g.clone(), people.collect())
            })
            .collect()
    }

    /// What `link` (or, if the pair already has an edge, `change_mode`) would change. The tree
    /// itself stays as it is. The effect is only given for a change that would be accepted.
    ///
    /// # Errors
    /// The error `link` or `change_mode` would give.
    pub fn nesting_effect(
        &self,
        lower: &G,
        upper: &G,
        mode: Mode,
    ) -> Result<NestingEffect, NestingError<G>> {
        let mut probe = self.clone();
        let exists = self
            .edges
            .iter()
            .any(|e| e.lower == *lower && e.upper == *upper);
        if exists {
            probe.change_mode(lower, upper, mode)?;
        } else {
            probe.link(lower, upper, mode)?;
        }
        let (before, after) = (self.member_sets(), probe.member_sets());
        let (mut gained, mut lost) = (BTreeSet::new(), BTreeSet::new());
        let mut affected = 0_u32;
        for (g, old) in &before {
            let new = &after[g];
            if old != new {
                affected += 1;
                gained.extend(new.difference(old).cloned());
                lost.extend(old.difference(new).cloned());
            }
        }
        let count = |s: &BTreeSet<P>| u32::try_from(s.len()).unwrap_or(u32::MAX);
        Ok(NestingEffect {
            mode,
            groups_affected: affected,
            persons_gained: count(&gained),
            persons_lost: count(&lost),
        })
    }
}

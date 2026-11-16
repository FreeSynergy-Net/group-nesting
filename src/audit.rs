//! Audit of loaded data: what is wrong in a tree that was not built through the checked methods.
//!
//! Pattern: Visitor-like report (walks the state once, collects findings; changes nothing).

use crate::check::check_edge;
use crate::error::NestingError;
use crate::group::{GroupId, InForce};
use crate::mode::Edge;
use crate::tree::Tree;

/// Something wrong in stored data. The data stays as it is; the caller decides what to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding<G, P> {
    /// A stored membership of the wrong account kind: it does not count.
    KindMismatch {
        /// The group.
        group: G,
        /// The person.
        person: P,
    },
    /// An effective edge at an AI group (old data): held back, it acts like `Single`.
    HeldBack {
        /// The offending edge.
        edge: Edge<G>,
    },
    /// An edge that would be refused today (cycle, too deep, sensitive flow).
    EdgeInvalid {
        /// The edge.
        edge: Edge<G>,
        /// Why it would be refused.
        error: NestingError<G>,
    },
}

impl<G: GroupId, P: Clone + Ord> Tree<G, P> {
    /// Checks every edge against the others and every membership against its group. Findings come
    /// in a fixed order: edges as stored, then memberships by group and person.
    #[must_use]
    pub fn audit(&self) -> Vec<Finding<G, P>> {
        let mut out = Vec::new();
        for (i, edge) in self.edges.iter().enumerate() {
            if self.held_back(edge) {
                out.push(Finding::HeldBack { edge: edge.clone() });
                continue;
            }
            let mut rest = self.edges.clone();
            rest.remove(i);
            if let Err(error) = check_edge(&self.groups, &rest, edge) {
                out.push(Finding::EdgeInvalid {
                    edge: edge.clone(),
                    error,
                });
            }
        }
        for (group, people) in &self.direct {
            for person in people {
                if self.in_force(group, person) == InForce::KindMismatch {
                    out.push(Finding::KindMismatch {
                        group: group.clone(),
                        person: person.clone(),
                    });
                }
            }
        }
        out
    }
}

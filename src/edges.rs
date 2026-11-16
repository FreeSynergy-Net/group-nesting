//! Changing edges (`link`, `change_mode`, `unlink`) and the effective arcs of the membership graph.
//!
//! Pattern: Command-style methods on the aggregate (check first, then change; on error nothing
//! changes) + Specification (the arc rule lives in `Edge::arc`, the held-back rule in `arcs`).

use crate::check::check_edge;
use crate::error::NestingError;
use crate::group::GroupId;
use crate::mode::{Edge, Mode};
use crate::tree::Tree;

impl<G: GroupId, P: Clone + Ord> Tree<G, P> {
    /// Links `lower` under `upper` with `mode`.
    ///
    /// # Errors
    /// `UnknownGroup`; `AlreadyThere` (the pair has an edge in any mode); `AiNesting`; `Cycle`
    /// (tree or membership graph, self edge = `[A, A]`); `TooDeep`; `ModeConflict`.
    pub fn link(&mut self, lower: &G, upper: &G, mode: Mode) -> Result<Edge<G>, NestingError<G>> {
        self.known(lower)?;
        self.known(upper)?;
        if self
            .edges
            .iter()
            .any(|e| e.lower == *lower && e.upper == *upper)
        {
            return Err(NestingError::AlreadyThere {
                upper: upper.clone(),
                lower: lower.clone(),
            });
        }
        let edge = Edge::new(upper.clone(), lower.clone(), mode);
        check_edge(&self.groups, &self.edges, &edge)?;
        self.edges.push(edge.clone());
        Ok(edge)
    }

    /// Like `link`, with the default mode of the tree.
    ///
    /// # Errors
    /// As `link`.
    pub fn link_default(&mut self, lower: &G, upper: &G) -> Result<Edge<G>, NestingError<G>> {
        self.link(lower, upper, self.default_mode)
    }

    /// Changes the mode of an edge; the checks run without the old edge.
    ///
    /// # Errors
    /// `NotLinked`; `AlreadyThere` if the mode is already this one; otherwise as `link`.
    pub fn change_mode(
        &mut self,
        lower: &G,
        upper: &G,
        mode: Mode,
    ) -> Result<Edge<G>, NestingError<G>> {
        let at = self
            .edges
            .iter()
            .position(|e| e.lower == *lower && e.upper == *upper)
            .ok_or_else(|| NestingError::NotLinked {
                upper: upper.clone(),
                lower: lower.clone(),
            })?;
        if self.edges[at].mode == mode {
            return Err(NestingError::AlreadyThere {
                upper: upper.clone(),
                lower: lower.clone(),
            });
        }
        let mut rest = self.edges.clone();
        rest.remove(at);
        let new = Edge::new(upper.clone(), lower.clone(), mode);
        check_edge(&self.groups, &rest, &new)?;
        self.edges[at] = new.clone();
        Ok(new)
    }

    /// Removes an edge.
    ///
    /// # Errors
    /// `NotLinked`.
    pub fn unlink(&mut self, lower: &G, upper: &G) -> Result<Edge<G>, NestingError<G>> {
        let at = self
            .edges
            .iter()
            .position(|e| e.lower == *lower && e.upper == *upper)
            .ok_or_else(|| NestingError::NotLinked {
                upper: upper.clone(),
                lower: lower.clone(),
            })?;
        Ok(self.edges.remove(at))
    }

    /// Whether an edge touches an AI group with an effective mode. Such an edge can only come from
    /// old data (writing refuses it); it is held back and acts like `Single`.
    pub(crate) fn held_back(&self, e: &Edge<G>) -> bool {
        e.mode != Mode::Single
            && [&e.upper, &e.lower]
                .iter()
                .any(|g| self.groups.get(*g).is_some_and(|x| x.ai))
    }

    /// The arcs of the membership graph `(member, of)` that are in force: no `Single`, no edge held
    /// back by the AI rule.
    pub(crate) fn arcs(&self) -> Vec<(G, G)> {
        self.edges
            .iter()
            .filter(|e| !self.held_back(e))
            .filter_map(Edge::arc)
            .collect()
    }
}

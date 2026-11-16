//! The checks for one edge against the other edges: AI rule, cycles, depth, sensitive flow.
//!
//! Pattern: Specification (each rule is a small function; `check_edge` composes them in a fixed
//! order, the same for every caller).

use crate::MAX_NESTING_DEPTH;
use crate::error::{Conflict, NestingError};
use crate::graph::{longest, path};
use crate::group::{Group, GroupId};
use crate::mode::{Edge, Mode};
use std::collections::BTreeMap;

pub(crate) type Groups<G> = BTreeMap<G, Group<G>>;

/// AI rule for an edge: an AI group is only a `Single` leaf (a child, never a parent).
pub(crate) fn check_ai_edge<G: GroupId>(
    groups: &Groups<G>,
    new: &Edge<G>,
) -> Result<(), NestingError<G>> {
    let ai = |g: &G| groups.get(g).is_some_and(|x| x.ai);
    let (ai_upper, ai_lower) = (ai(&new.upper), ai(&new.lower));
    if ai_upper || (ai_lower && new.mode != Mode::Single) {
        let group = if ai_upper { &new.upper } else { &new.lower };
        return Err(NestingError::AiNesting {
            group: group.clone(),
            path: vec![new.lower.clone(), new.upper.clone()],
        });
    }
    Ok(())
}

/// Cycles: first in the tree (all modes), then in the membership graph after the reversal.
fn check_cycles<G: GroupId>(others: &[Edge<G>], new: &Edge<G>) -> Result<(), NestingError<G>> {
    let tree: Vec<(G, G)> = others
        .iter()
        .map(|e| (e.lower.clone(), e.upper.clone()))
        .collect();
    if let Some(mut way) = path(&tree, &new.upper, &new.lower) {
        way.insert(0, new.lower.clone());
        return Err(NestingError::Cycle { path: way });
    }
    if let Some((u, v)) = new.arc() {
        let arcs: Vec<(G, G)> = others.iter().filter_map(Edge::arc).collect();
        if let Some(mut way) = path(&arcs, &v, &u) {
            way.insert(0, u);
            return Err(NestingError::Cycle { path: way });
        }
    }
    Ok(())
}

/// Depth of the tree and of the membership graph, counted in edges.
fn check_depth<G: GroupId>(others: &[Edge<G>], new: &Edge<G>) -> Result<(), NestingError<G>> {
    let mut tree: Vec<(G, G)> = others
        .iter()
        .map(|e| (e.lower.clone(), e.upper.clone()))
        .collect();
    tree.push((new.lower.clone(), new.upper.clone()));
    let mut arcs: Vec<(G, G)> = others.iter().filter_map(Edge::arc).collect();
    arcs.extend(new.arc());
    if longest(&tree) > MAX_NESTING_DEPTH || longest(&arcs) > MAX_NESTING_DEPTH {
        return Err(NestingError::TooDeep {
            max: MAX_NESTING_DEPTH,
        });
    }
    Ok(())
}

/// Sensitive flow: an arc whose ends differ in `sensitive`, in either direction.
pub(crate) fn check_sensitive<G: GroupId>(
    groups: &Groups<G>,
    arcs: &[(G, G)],
) -> Result<(), NestingError<G>> {
    let sensitive = |g: &G| groups.get(g).is_some_and(|x| x.sensitive);
    for (u, v) in arcs {
        if sensitive(u) != sensitive(v) {
            return Err(NestingError::ModeConflict(Conflict::SensitiveFlow {
                from: u.clone(),
                to: v.clone(),
            }));
        }
    }
    Ok(())
}

/// Checks `new` against `others` (the edges without `new`): AI rule, cycles, depth, sensitive.
pub(crate) fn check_edge<G: GroupId>(
    groups: &Groups<G>,
    others: &[Edge<G>],
    new: &Edge<G>,
) -> Result<(), NestingError<G>> {
    check_ai_edge(groups, new)?;
    check_cycles(others, new)?;
    check_depth(others, new)?;
    let arc: Vec<(G, G)> = new.arc().into_iter().collect();
    check_sensitive(groups, &arc)
}

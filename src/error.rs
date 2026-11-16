//! Typed errors of the checks.
//!
//! Pattern: Value Object (error enum, one variant per rule).

use std::fmt;

/// Which pair of modes or flags cannot go together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Conflict<G> {
    /// An effective edge between a sensitive and a non-sensitive group, in either direction. People
    /// would become effective members of a sensitive group without joining it, or sensitive
    /// membership would flow into an open group. `Single` has no arc and stays allowed.
    SensitiveFlow {
        /// The member side of the offending arc.
        from: G,
        /// The group side of the offending arc.
        to: G,
    },
}

/// Why a change was refused. Nothing is changed when an error is returned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NestingError<G> {
    /// A group id that the tree does not know.
    UnknownGroup(G),
    /// A group id that is already in the tree.
    DuplicateGroup(G),
    /// This pair of groups already has an edge, in any mode (use `change_mode`).
    AlreadyThere {
        /// The upper group of the existing edge.
        upper: G,
        /// The lower group of the existing edge.
        lower: G,
    },
    /// There is no such edge.
    NotLinked {
        /// The upper group asked for.
        upper: G,
        /// The lower group asked for.
        lower: G,
    },
    /// The edge would close a cycle in the tree (all modes) or in the membership graph (after the
    /// `TopDown` edges are reversed). `path` names it; first group = last group. A self edge is
    /// `[A, A]`.
    Cycle {
        /// The cycle, closed.
        path: Vec<G>,
    },
    /// A chain would be longer than `MAX_NESTING_DEPTH` edges.
    TooDeep {
        /// The limit.
        max: u32,
    },
    /// Modes or flags do not fit together.
    ModeConflict(Conflict<G>),
    /// The AI rule is broken: an AI account in a human group, a human in an AI group, an AI group
    /// with an effective edge or with children. Never dropped silently.
    AiNesting {
        /// The group where it fails.
        group: G,
        /// The offending edge `[lower, upper]`, or just `[group]` for a membership.
        path: Vec<G>,
    },
    /// An account of unknown kind was to join an AI group.
    AiStatusUnknown {
        /// The AI group.
        group: G,
    },
}

impl<G: fmt::Debug> fmt::Display for NestingError<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownGroup(g) => write!(f, "unknown group {g:?}"),
            Self::DuplicateGroup(g) => write!(f, "group {g:?} already exists"),
            Self::AlreadyThere { upper, lower } => {
                write!(f, "{lower:?} already belongs to {upper:?}")
            }
            Self::NotLinked { upper, lower } => {
                write!(f, "{lower:?} does not belong to {upper:?}")
            }
            Self::Cycle { path } => write!(f, "would close a cycle: {path:?}"),
            Self::TooDeep { max } => write!(f, "nesting deeper than {max} edges"),
            Self::ModeConflict(Conflict::SensitiveFlow { from, to }) => {
                write!(
                    f,
                    "sensitive and open group may not be linked: {from:?} -> {to:?}"
                )
            }
            Self::AiNesting { group, path } => {
                write!(f, "AI rule broken at {group:?} (path {path:?})")
            }
            Self::AiStatusUnknown { group } => {
                write!(f, "account of unknown kind may not join AI group {group:?}")
            }
        }
    }
}

impl<G: fmt::Debug> std::error::Error for NestingError<G> {}

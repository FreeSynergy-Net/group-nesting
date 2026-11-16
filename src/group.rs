//! Groups, account kinds and whether a membership counts.
//!
//! Pattern: Value Object (group flags, kind marker).

use std::fmt::Debug;

/// What a group id must be: cheap to clone, ordered (for deterministic output), hashable and
/// printable in errors. Implemented for every such type; nothing to do for callers.
pub trait GroupId: Clone + Ord + std::hash::Hash + Debug {}
impl<T: Clone + Ord + std::hash::Hash + Debug> GroupId for T {}

/// A group and the few flags the rules need.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group<G> {
    /// The group's id.
    pub id: G,
    /// Special-category group (GDPR Art. 9, for example health or religion): no effective edge to
    /// or from a non-sensitive group, never mirrored.
    pub sensitive: bool,
    /// AI group: takes only AI accounts and takes part in no effective edge.
    pub ai: bool,
    /// Whether the group is to be mirrored into a directory (ignored for sensitive and AI groups).
    pub mirror: bool,
}

impl<G> Group<G> {
    /// An ordinary group that is mirrored.
    #[must_use]
    pub fn new(id: G) -> Self {
        Self {
            id,
            sensitive: false,
            ai: false,
            mirror: true,
        }
    }

    /// Marks the group as sensitive.
    #[must_use]
    pub fn sensitive(mut self) -> Self {
        self.sensitive = true;
        self
    }

    /// Marks the group as an AI group.
    #[must_use]
    pub fn ai(mut self) -> Self {
        self.ai = true;
        self
    }

    /// Keeps the group out of the directory mirror.
    #[must_use]
    pub fn unmirrored(mut self) -> Self {
        self.mirror = false;
        self
    }

    /// Whether the group is written to a directory: wanted, not sensitive, not an AI group.
    #[must_use]
    pub fn is_mirrored(&self) -> bool {
        self.mirror && !self.sensitive && !self.ai
    }
}

/// What kind of account a person is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum Kind {
    /// A human.
    Human,
    /// An AI service account.
    Ai,
    /// Nobody knows. Allowed in human groups, never in AI groups.
    #[default]
    Unknown,
}

/// Whether a stored membership counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InForce {
    /// The membership counts.
    Yes,
    /// The account kind does not fit the group (an AI in a human group, a human or unknown in an
    /// AI group): stored, but it does not count.
    KindMismatch,
}

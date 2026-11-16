//! The mode of an edge and the edge itself.
//!
//! Pattern: Value Object (immutable pair of groups plus mode).

/// How an edge between two groups acts. The membership graph reads "X is a member of Y", exactly
/// like Kanidm's `member`/`memberof`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Mode {
    /// The lower group is a member of the upper group (Kanidm-native). Members count upward; the
    /// rights of the upper group apply to everyone below (example: Helfa).
    BottomUp,
    /// The upper group is a member of the lower group (the edge reversed). Membership flows down,
    /// rights flow up: "Chefs" stand in `member` of "Vizechefs"; Chefs may do what the Vizechefs
    /// may do and are also effective members of the Vizechefs.
    TopDown,
    /// Display only: the group belongs to the tree for orientation. Nothing flows; a directory
    /// sees flat groups, the edge is not mirrored.
    Single,
}

/// A directed edge of the tree: `lower` belongs to `upper`, with a mode.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Edge<G> {
    /// The group above (the parent in the tree).
    pub upper: G,
    /// The group below (the child in the tree).
    pub lower: G,
    /// How the edge acts.
    pub mode: Mode,
}

impl<G: Clone> Edge<G> {
    /// Creates an edge.
    #[must_use]
    pub fn new(upper: G, lower: G, mode: Mode) -> Self {
        Self { upper, lower, mode }
    }

    /// The arc of the membership graph as `(who becomes a member, of what)`, or `None` for
    /// `Single`. This is the reversal: `TopDown` turns the tree edge around.
    #[must_use]
    pub fn arc(&self) -> Option<(G, G)> {
        match self.mode {
            Mode::BottomUp => Some((self.lower.clone(), self.upper.clone())),
            Mode::TopDown => Some((self.upper.clone(), self.lower.clone())),
            Mode::Single => None,
        }
    }
}

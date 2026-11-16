//! The tree: groups, edges, account kinds and direct members, with the checked ways to change them.
//!
//! Pattern: Aggregate (one object owns all state; every change goes through a method that checks
//! the rules first and changes nothing on error) + Builder-style loaders (`from_parts` for stored
//! data that is NOT checked, see `audit`).

use crate::check::{Groups, check_sensitive};
use crate::error::NestingError;
use crate::group::{Group, GroupId, InForce, Kind};
use crate::mode::{Edge, Mode};
use std::collections::{BTreeMap, BTreeSet};

/// A forest of groups with edges, account kinds and direct members. `G` = group id, `P` = person id.
#[derive(Clone, Debug)]
pub struct Tree<G, P> {
    pub(crate) default_mode: Mode,
    pub(crate) groups: Groups<G>,
    pub(crate) edges: Vec<Edge<G>>,
    pub(crate) kinds: BTreeMap<P, Kind>,
    pub(crate) direct: BTreeMap<G, BTreeSet<P>>,
}

impl<G: GroupId, P: Clone + Ord> Tree<G, P> {
    /// An empty tree. `default_mode` is the mode `link_default` uses (one default per tree).
    #[must_use]
    pub fn new(default_mode: Mode) -> Self {
        Self {
            default_mode,
            groups: BTreeMap::new(),
            edges: Vec::new(),
            kinds: BTreeMap::new(),
            direct: BTreeMap::new(),
        }
    }

    /// Loads stored state WITHOUT checking it (old data, restore). Use `audit` to see what is
    /// wrong. The effective computation stays safe on such data: an AI group with an effective
    /// edge acts like `Single`, a membership of the wrong kind does not count, cycles terminate.
    #[must_use]
    pub fn from_parts(
        default_mode: Mode,
        groups: Vec<Group<G>>,
        edges: Vec<Edge<G>>,
        kinds: Vec<(P, Kind)>,
        members: Vec<(G, P)>,
    ) -> Self {
        let mut tree = Self::new(default_mode);
        tree.groups = groups.into_iter().map(|g| (g.id.clone(), g)).collect();
        tree.edges = edges;
        tree.kinds = kinds.into_iter().collect();
        for (g, p) in members {
            tree.direct.entry(g).or_default().insert(p);
        }
        tree
    }

    /// The default mode of this tree.
    #[must_use]
    pub fn default_mode(&self) -> Mode {
        self.default_mode
    }

    /// All groups, ordered by id.
    pub fn groups(&self) -> impl Iterator<Item = &Group<G>> {
        self.groups.values()
    }

    /// One group.
    #[must_use]
    pub fn group(&self, id: &G) -> Option<&Group<G>> {
        self.groups.get(id)
    }

    /// All edges in the order they were added.
    #[must_use]
    pub fn edges(&self) -> &[Edge<G>] {
        &self.edges
    }

    /// Adds a group.
    ///
    /// # Errors
    /// `DuplicateGroup` if the id exists.
    pub fn add_group(&mut self, group: Group<G>) -> Result<(), NestingError<G>> {
        if self.groups.contains_key(&group.id) {
            return Err(NestingError::DuplicateGroup(group.id));
        }
        self.groups.insert(group.id.clone(), group);
        Ok(())
    }

    pub(crate) fn known(&self, id: &G) -> Result<(), NestingError<G>> {
        if self.groups.contains_key(id) {
            Ok(())
        } else {
            Err(NestingError::UnknownGroup(id.clone()))
        }
    }

    /// The kind of an account; `Unknown` if never set.
    #[must_use]
    pub fn kind(&self, person: &P) -> Kind {
        self.kinds.get(person).copied().unwrap_or_default()
    }

    /// Sets (or re-marks) the kind of an account. NOT checked on purpose: a person marked as AI
    /// after joining a human group stays a member that no longer counts (`InForce::KindMismatch`).
    pub fn set_kind(&mut self, person: P, kind: Kind) {
        self.kinds.insert(person, kind);
    }

    /// Adds a direct member. The kind comes from `set_kind` (default `Unknown`).
    ///
    /// # Errors
    /// `UnknownGroup`; `AiNesting` for an AI account in a human group or a human in an AI group;
    /// `AiStatusUnknown` for an unknown account in an AI group.
    pub fn add_member(&mut self, group: &G, person: P) -> Result<(), NestingError<G>> {
        self.known(group)?;
        let ai_group = self.groups[group].ai;
        match (ai_group, self.kind(&person)) {
            (false, Kind::Ai) | (true, Kind::Human) => {
                return Err(NestingError::AiNesting {
                    group: group.clone(),
                    path: vec![group.clone()],
                });
            }
            (true, Kind::Unknown) => {
                return Err(NestingError::AiStatusUnknown {
                    group: group.clone(),
                });
            }
            _ => {}
        }
        self.direct.entry(group.clone()).or_default().insert(person);
        Ok(())
    }

    /// Removes a direct member; `true` if there was one.
    pub fn remove_member(&mut self, group: &G, person: &P) -> bool {
        self.direct
            .get_mut(group)
            .is_some_and(|set| set.remove(person))
    }

    /// Direct members of a group, whether or not they count.
    #[must_use]
    pub fn direct_members(&self, group: &G) -> Vec<P> {
        self.direct
            .get(group)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Whether the membership of `person` in `group` counts.
    #[must_use]
    pub fn in_force(&self, group: &G, person: &P) -> InForce {
        let ai_group = self.groups.get(group).is_some_and(|g| g.ai);
        match (ai_group, self.kind(person)) {
            (true, Kind::Ai) | (false, Kind::Human | Kind::Unknown) => InForce::Yes,
            _ => InForce::KindMismatch,
        }
    }

    /// Changes the sensitive flag.
    ///
    /// # Errors
    /// `UnknownGroup`; `ModeConflict` if an effective edge would then join a sensitive and a
    /// non-sensitive group.
    pub fn set_sensitive(&mut self, group: &G, sensitive: bool) -> Result<(), NestingError<G>> {
        self.known(group)?;
        let mut probe = self.groups.clone();
        if let Some(g) = probe.get_mut(group) {
            g.sensitive = sensitive;
        }
        let arcs: Vec<(G, G)> = self.edges.iter().filter_map(Edge::arc).collect();
        check_sensitive(&probe, &arcs)?;
        self.groups = probe;
        Ok(())
    }

    /// Changes the AI flag.
    ///
    /// # Errors
    /// `UnknownGroup`; `AiNesting` when `true` for a group with humans, an effective edge or
    /// children, or `false` for a group with AI members (otherwise the flag would be a way around
    /// the checks); `AiStatusUnknown` when `true` for a group with unknown accounts.
    pub fn set_ai(&mut self, group: &G, ai: bool) -> Result<(), NestingError<G>> {
        self.known(group)?;
        let kinds: Vec<Kind> = self
            .direct_members(group)
            .iter()
            .map(|p| self.kind(p))
            .collect();
        let broken = |path: Vec<G>| NestingError::AiNesting {
            group: group.clone(),
            path,
        };
        if ai {
            if kinds.contains(&Kind::Human) {
                return Err(broken(vec![group.clone()]));
            }
            if kinds.contains(&Kind::Unknown) {
                return Err(NestingError::AiStatusUnknown {
                    group: group.clone(),
                });
            }
            let bad = self
                .edges
                .iter()
                .find(|e| (e.upper == *group) || (e.lower == *group && e.mode != Mode::Single));
            if let Some(e) = bad {
                return Err(broken(vec![e.lower.clone(), e.upper.clone()]));
            }
        } else if kinds.contains(&Kind::Ai) {
            return Err(broken(vec![group.clone()]));
        }
        if let Some(g) = self.groups.get_mut(group) {
            g.ai = ai;
        }
        Ok(())
    }
}

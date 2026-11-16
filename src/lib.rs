//! Group nesting modes on top of a flat or Kanidm-style directory.
//!
//! Each edge between two groups has a [`Mode`]: `BottomUp` (members count upward, Kanidm's native
//! behaviour), `TopDown` (the reversed edge: membership flows down, rights flow up) or `Single`
//! (display only). The crate checks a tree before anything is written (cycles after reversing the
//! edges, depth, sensitive groups, AI accounts), computes effective members with their origin and
//! projects the tree onto Kanidm `memberof` as a dry-run plan.
//!
//! It has no dependencies, no async runtime and no knowledge of any product: group and person ids
//! are generic.
//!
//! Pattern: Facade (the public surface re-exports the parts; no logic here).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod audit;
mod check;
mod edges;
mod effect;
mod error;
mod graph;
mod group;
mod members;
mod mode;
mod projection;
mod tree;

pub use audit::Finding;
pub use effect::NestingEffect;
pub use error::{Conflict, NestingError};
pub use group::{Group, GroupId, InForce, Kind};
pub use members::{EffectiveMember, Inherited};
pub use mode::{Edge, Mode};
pub use projection::{MemberOf, MirrorFinding, MirrorPlan};
pub use tree::Tree;

/// Longest allowed chain of edges, in the tree and in the membership graph. Counted in EDGES:
/// 16 edges are 17 groups. Kanidm has no limit of its own; this keeps display, computation and the
/// "inherited from ... via ..." explanation manageable.
pub const MAX_NESTING_DEPTH: u32 = 16;

/// Makes `cargo test` run the code example of the README.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

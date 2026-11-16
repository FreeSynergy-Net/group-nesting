# group-nesting: design

A small, standalone rule engine for nested groups. It owns the hierarchy logic (modes, checks, effective members, projection) and nothing else. It is written so that others, Kanidm included, can take it over.

## Aims

- **Standalone**: no FreeSynergy dependency, no async runtime, no dependencies at all (std only). `thiserror` was left out on purpose: the one error enum has a hand-written `Display`, so there is nothing to audit.
- **Generic ids**: `Tree<G, P>` with `G: GroupId` (`Clone + Ord + Hash + Debug`, blanket-implemented) and `P: Clone + Ord`. The caller maps its own ids (a `From` or a lookup table).
- **Deterministic**: `BTreeMap`/`BTreeSet` everywhere, so results and findings come in a fixed order.
- **Safe on bad data**: nothing panics, `unsafe` is forbidden, searches track visited nodes, so even a stored cycle terminates.

## Structure

| File | Content | Pattern |
|---|---|---|
| `mode.rs` | `Mode`, `Edge`, the reversal (`Edge::arc`) | Value Object |
| `group.rs` | `Group`, `Kind`, `InForce`, `GroupId` | Value Object |
| `error.rs` | `NestingError`, `Conflict` | Value Object |
| `graph.rs` | shortest path, longest chain, adjacency | helper functions |
| `check.rs` | the per-edge rules in a fixed order | Specification |
| `tree.rs` | `Tree`: groups, kinds, members; checked mutations; `from_parts` | Aggregate |
| `edges.rs` | `link`, `change_mode`, `unlink`; the arcs in force | Command-style methods |
| `members.rs` | effective members with origin; concealed origin | Specification |
| `effect.rs` | `nesting_effect` (numbers, tried on a copy) | Value Object, probe |
| `audit.rs` | findings on loaded data | Report |
| `projection.rs` | `kanidm_projection`, `mirror_plan` | Adapter, Template Method |

Files stay under 300 lines; each has a `Pattern:` line.

## Rules

- **Edge** = `(upper, lower, mode)`. The tree edge always points lower to upper. The **arc** of the membership graph reads "X is a member of Y": `BottomUp` = lower in upper, `TopDown` = upper in lower, `Single` has none.
- **Order of checks** for a new edge: pair known, `AlreadyThere`, AI rule, tree cycle, membership cycle (after the reversal), depth, sensitive flow. Nothing is changed on error.
- **Depth** counts edges (16 edges = 17 groups) and is checked on the tree and on the membership graph. A zigzag of alternating modes can make a long membership chain in a flat tree, so both are needed.
- **Effective members** of G: direct members of G plus direct members of every X with a directed path X to G in the membership graph (modes may mix, like `memberof`). One backward breadth-first search from G, O(groups + edges + memberships); the first find per source is the shortest path, which is the origin (`Inherited::via`). A viewer who may not read a group sees a concealed origin (`Inherited::concealed`).
- **Sensitive**: an arc whose ends differ in `sensitive` is refused in both directions. Reason: into a sensitive group people would become effective members without joining; out of it, special-category membership would show up in an open group.
- **AI rule**: see README. Writing refuses; loaded data is tolerated and reported: wrong-kind memberships do not count, an AI group's effective edge acts like `Single`.
- **Projection** only for mirrored groups (flag, not sensitive, not AI). An arc with one end not mirrored is a finding (`NestedNotMirrored`), not a gap.
- **`mirror_plan(current, managed)`**: `add` = wanted minus current. A current edge that is not wanted is removed only if the caller wrote it (`managed`); otherwise, if both groups are known to the tree, it is `ForeignNesting`. Edges with unknown groups are not looked at. No silent removal.

## Decisions

- **Reversed edge for TopDown (not flattened member lists).** The directory's own closure does the work; the tree stays the single source of truth and is checked first. Risks (cycles over mixed directions, sibling effects over mixed paths) are covered by the checks and by `nesting_effect`, which counts over the whole closure.
- **One edge per pair, any mode.** `AlreadyThere` rather than silent idempotence, because the caller learns the pair is taken; the way to another mode is `change_mode`.
- **`set_kind` is unchecked, `add_member` is checked.** Re-marking an account after it joined is a fact of life; it must not fail, it must stop counting (`KindMismatch`).
- **Direct unit tests are in `tests/`** (integration tests through the public API): all three modes, an 8-level chefs/deputies tree with uneven branches, mixed-direction cycles, depth 16/17, sensitive in both directions, the AI cases, the closure cross-check against a Kanidm-style `memberof` fixed point, and property tests with a fixed seed and a tiny generator of our own (accepted trees keep all invariants, refused changes change nothing, breadth-first result equals a naive closure).

## What the caller does

- Maps its ids to `G` and `P`.
- Asks consent of both groups' managers before `link` (an AI never gives it) and checks rights.
- Stores the tree; loads it with `from_parts` and runs `audit()`.
- Fetches the directory's current group-in-group edges, calls `mirror_plan`, shows the plan, applies it, and remembers what it wrote (`managed`).

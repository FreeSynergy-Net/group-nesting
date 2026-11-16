# group-nesting

Group nesting modes for directories that nest groups in one direction only (Kanidm and similar).

**Standalone.** No dependencies, no async runtime, no knowledge of any product. Group and person ids are generic. Licence: MIT OR Apache-2.0.

## The problem

Kanidm nests groups in one direction: if group **A** is a member of group **B**, everyone in A is also a member of B (via `memberof`) and receives B's rights. This suits organisations where members "count upward": the German volunteer network Helfa works like this, a helper in a local group is automatically part of the regional and the national level.

Many organisations also need the **opposite** direction, and groups that appear in a tree only for orientation. This crate adds both, on top of the native mechanism, without flattening or computing member lists.

## The three modes

Each edge between two groups carries a mode. A tree can have a default mode.

| Mode | Meaning | In Kanidm |
|---|---|---|
| `BottomUp` | Members count upward; the rights of the upper group apply to everyone below (example: Helfa). Kanidm-native. | lower group is in `member` of the upper group |
| `TopDown` | Membership flows down, rights flow up: "Chefs" stand in `member` of "Vizechefs"; Chefs may do what the Vizechefs may do and are also effective members of the Vizechefs (they see the member list and are invited with them). | upper group is in `member` of the lower group (the edge reversed) |
| `Single` | Display only: the group belongs to the tree for orientation. Nothing flows. | not mirrored; Kanidm sees flat groups |

Kanidm's own `memberof` closure does the work (checked against `server/lib/src/plugins/memberof.rs`, v1.12.1: it runs to a fixed point and has no depth limit). Chains of 8 or more levels with uneven depth are covered by this crate's tests; the upstream tests we found go to 4 levels.

## What it checks before anything reaches the directory

1. **No cycles.** Mixing `BottomUp` and `TopDown` can create A in B and B in A. The directed membership graph is built AFTER reversing the `TopDown` edges; a cycle is `NestingError::Cycle { path }`. The tree itself must be acyclic too (all modes, a self edge is `[A, A]`). Kanidm would accept a cycle and make every group in it a member of every other.
2. **Depth limit**: `MAX_NESTING_DEPTH` = 16 edges, in the tree and in the membership graph (`TooDeep`).
3. **Sensitive groups stay apart**: no effective edge between a sensitive group (GDPR Art. 9) and a non-sensitive one, in either direction (`ModeConflict(SensitiveFlow)`). `Single` is allowed.
4. **One edge per pair**: a second edge for the same pair, in any mode, is `AlreadyThere`; the way to another mode is `change_mode`. Diamonds are fine; every person counts once per group.

## The AI rule

AI service accounts may only be members of dedicated AI groups, and AI groups take part in no effective edge: they never inherit and never pass on rights. An AI group can only be a `Single` leaf (a child, never a parent), shown in the tree. The reason: damage done by an automated agent spreads faster than people can react. The rule runs on the whole closure, not only on direct members, because a nested edge could otherwise give an AI account an admin group through `memberof`.

- Writing a violation is a typed error, never a silent omission: `AiNesting { group, path }`, or `AiStatusUnknown` for an account of unknown kind joining an AI group (unknown accounts stay allowed in human groups).
- If the state went wrong without a write (an account re-marked as AI after joining, old data, a restore), the membership stays stored but does not count (`InForce::KindMismatch`); `audit()` and the mirror plan report it. An old effective edge at an AI group acts like `Single` and is reported (`HeldBack`).
- AI groups are never mirrored (`AiGroupExcluded`).
- Rights for AI groups, and keeping AI accounts away from admin groups, belong to the caller; this crate gives it the closure and the findings.

## Example

```rust
use group_nesting::{Group, Kind, Mode, Tree};

let mut tree: Tree<&str, &str> = Tree::new(Mode::BottomUp);
for id in ["chefs", "vize"] {
    tree.add_group(Group::new(id))?;
}
tree.set_kind("carla", Kind::Human);
tree.add_member(&"chefs", "carla")?;

// "Chefs" stand in `member` of "Vizechefs": TopDown, the edge is reversed.
tree.link(&"vize", &"chefs", Mode::TopDown)?;
assert_eq!(tree.effective_members(&"vize")[0].person, "carla");

// What would Kanidm get? (a dry run against the current directory state)
let plan = tree.mirror_plan(&[], &[]);
assert_eq!(plan.add[0].member, "chefs");
assert_eq!(plan.add[0].of, "vize");
# Ok::<(), group_nesting::NestingError<&str>>(())
```

`mirror_plan(current, managed)` compares the wanted edges with the directory: it lists edges to add, edges to remove (only those the caller wrote itself, `managed`) and findings. An edge somebody made by hand in the directory is `ForeignNesting`, never removed silently.

## Limits

- It is a rule engine over small in-memory trees (hundreds of groups). It stores nothing and talks to no directory.
- It projects group-in-group edges only. Person memberships come from `effective_members`.
- Rights are not modelled: the crate says who is an effective member where; what a group may do is up to the caller.
- Consent of both groups' managers for an effective edge, and the rule that an AI can never give it, are enforced by the caller before `link`.
- The AI marking protects only against honest AI accounts.

## Made with AI help, said openly

Claude (Anthropic) took part in writing this crate. The commits carry a `Co-Authored-By` footer. All decisions were made and reviewed by people.

## Relation to FreeSynergy

[FreeSynergy](https://github.com/FreeSynergy) is a family of free programs that keep your data yours. Its groups program (`fs-groups`) uses this crate for nesting; the crate itself knows nothing about FreeSynergy. Anyone who wants top-down or display-only trees on top of Kanidm can use it.

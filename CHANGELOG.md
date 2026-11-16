# Changelog

## 0.0.1

First version.

- Modes `BottomUp`, `TopDown`, `Single`; edges `(upper, lower, mode)`; default mode per tree; reversal into the membership graph.
- Checks: cycles in tree and membership graph (`Cycle { path }`), depth (`MAX_NESTING_DEPTH` = 16 edges, `TooDeep`), sensitive flow in both directions, `AlreadyThere`.
- Effective members with origin (direct / inherited from X via a path / concealed source).
- AI rule: AI accounts only in AI groups, AI groups only as `Single` leaves (`AiNesting`, `AiStatusUnknown`); `InForce::KindMismatch` for stored data that no longer fits; `audit()`.
- Projection onto Kanidm `memberof` and a dry-run mirror plan (`add`, `remove`, findings such as `ForeignNesting`, `AiGroupExcluded`).
- `nesting_effect`: numbers for a consent dialog.
- No dependencies.

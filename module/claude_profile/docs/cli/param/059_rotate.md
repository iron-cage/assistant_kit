# Parameter :: `rotate::`

Execute account rotation after quota fetch — switch to the account marked `→` by the active `next::` strategy.

### Summary

| Attribute | Value |
|-----------|-------|
| Type | `bool` (0 or 1) |
| Default | `0` |
| Commands | [`.usage`](../command/006_usage.md) |
| Mutual exclusion | `live::1` (exits 1) |
| Ownership gate | G5 — non-owned accounts skipped; `force::1` bypasses |

### Semantics

When `rotate::1`, after the quota table is rendered, `.usage` executes a switch to the account selected by `find_next_for_strategy()` (the same account that receives `→` in the table body).

The rotation target is the winner of the active `next::` strategy (default `next::renew`). Combine `rotate::1 next::endurance` or `rotate::1 next::drain` to switch using alternative strategies.

**Ownership gate (G5):** Only owned accounts are eligible rotation targets. A non-owned account is excluded from `find_first_eligible` when `rotate::1` is active (without `force::1`). This mirrors the ownership gate on `.account.use`.

**`dry::1` interaction:** When both `rotate::1 dry::1`, the table is rendered normally and the output ends with `[dry-run] would switch to '{name}'`. No credentials are written.

**`force::1` interaction:** Bypasses the G5 ownership gate. Non-owned accounts become eligible rotation targets — the same bypass semantics as `.account.use force::1`.

**Post-switch touch:** Touch is applied using the winner's `AccountQuota` already fetched during the main pipeline — no additional API call for the switch.

**No eligible account:** When no account passes `find_first_eligible` (all are current, active, occupied, h-exhausted, or non-owned), exits 1 with `"no eligible account to rotate to"`. The table is still rendered before the error.

**Exit codes:** 0 (switched or dry-run) | 1 (no eligible candidate, ownership violation, or `live::1` conflict) | 2 (switch I/O failure)

### Examples

```bash
clp .usage rotate::1                        # switch using default next::renew strategy
clp .usage rotate::1 next::endurance        # switch to account with most 5h quota
clp .usage rotate::1 next::drain            # switch to account closest to exhaustion
clp .usage rotate::1 dry::1                 # preview without switching
clp .usage rotate::1 force::1               # bypass G5 ownership gate
clp .usage rotate::1 trace::1               # emit trace output during rotation
```

### Valid Values

| Value | Meaning |
|-------|---------|
| `0` (default) | Display-only — no account switch executed |
| `1` | Execute switch to `→` winner after rendering the table |

### Cross-References

| File | Relationship |
|------|--------------|
| [feature/038_usage_strategy_rotate.md](../../feature/038_usage_strategy_rotate.md) | Full behavioral specification for rotation |
| [param/032_next.md](032_next.md) | `next::` selects the rotation target strategy |
| [param/058_force.md](058_force.md) | `force::1` bypasses G5 gate |
| [param/004_dry.md](004_dry.md) | `dry::1` previews rotation target |

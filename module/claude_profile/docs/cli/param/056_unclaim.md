# Parameter :: 56. `unclaim::`

Mutation param on `.accounts` and `.usage` that clears the `owner` field in `{name}.json` for one or all accounts. Re-activated in Feature 037 as part of the unified parameter set; the original `unclaim::` on `.account.save` was removed and the standalone `.account.unclaim` command was deregistered.

- **Default:** `0` — no unclaim when absent
- **Constraints:** `Kind::Bool` (`0` or `1`). Mutually exclusive with `owner::` (Feature 063).
- **Purpose:** Release account ownership — calls `write_owner(name, store, "")` directly without touching credentials or the active marker.

**Behavior:**

```text
unclaim::1 name::X          → clear owner of single account (G8 gate)
unclaim::1                   → batch clear all owned accounts in filtered set
unclaim::1 name::X dry::1   → preview without writing
unclaim::1 name::X force::1 → bypass G8 even if owned by another
unclaim::1 owner::VALUE     → exit 1 (mutual exclusion)
```

**G8 ownership gate:** Account must be unowned or owned by the caller; otherwise exit 1 with `"ownership violation"`. Bypassed by `force::1`. Batch mode skips non-owned accounts with a `"skip"` message instead of exiting 1.

### Referenced Type

- **Fundamental Type:** `bool`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | `.accounts` | Primary host — mutation param (unified param set, Feature 037) |
| 2 | `.usage` | Shared unified param set (Feature 037) |

### See Also

- [cli/param/062_owner.md](062_owner.md) — `owner::` — ownership assignment (complementary; mutually exclusive)
- [cli/param/058_force.md](058_force.md) — `force::` — bypass G8 ownership gate
- [feature/036_account_ownership.md](../../feature/036_account_ownership.md) — full ownership model; G1–G8 enforcement gates
- [feature/037_accounts_usage_param_unification.md](../../feature/037_accounts_usage_param_unification.md) — re-activation of `unclaim::` as unified mutation param

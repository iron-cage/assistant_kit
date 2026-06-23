# Parameter :: 62. `owner::`

Mutation param on `.accounts` and `.usage` that explicitly sets the `owner` field in `{name}.json` for a named account. Complementary to `unclaim::1` (which clears the owner field).

- **Default:** *(omit)* — no owner write when absent
- **Constraints:** Non-empty string required; empty value rejected (use `unclaim::1` to clear). Mutually exclusive with `unclaim::1`.
- **Purpose:** Assign account ownership to a specific `USER@MACHINE` identity without requiring direct JSON file editing. Enables cross-machine ownership management via the CLI.

**Behavior:**

```text
owner::user1@w003 name::X          → write_owner(X, store, "user1@w003")
owner::user1@w003 name::X dry::1   → preview without writing
owner::user1@w003 name::X force::1 → bypass G8 even if owned by another
owner::user1@w003 unclaim::1       → exit 1 (mutual exclusion)
owner::user1@w003                   → exit 1 (name:: required)
```

**G8 ownership gate:** Same gate as `unclaim::1`. Account must be unowned or owned by the caller; otherwise exit 1 with `"ownership violation"`. Bypassed by `force::1`.

**Value format:** The value is written as-is to `{name}.json`. Convention is `USER@HOSTNAME` (matching `current_identity()` output), but any non-empty string is accepted.

**Notes:**
- No batch mode — `name::` is always required (unlike `unclaim::1` which supports batch).
- Does not touch credentials, active marker, or any file other than `{name}.json`.
- `force::1` + `dry::1`: G8 bypassed, write previewed but not executed.

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | `.accounts` | Primary host — mutation param (unified param set, Feature 037) |
| 2 | `.usage` | Shared unified param set (Feature 037) |

### See Also

- [cli/param/056_unclaim.md](056_unclaim.md) — `unclaim::` — ownership release (complementary; mutually exclusive)
- [cli/param/058_force.md](058_force.md) — `force::` — bypass G8 ownership gate
- [feature/063_explicit_ownership_claim.md](../../feature/063_explicit_ownership_claim.md) — full feature design
- [feature/036_account_ownership.md](../../feature/036_account_ownership.md) — ownership model, G1–G8 gates

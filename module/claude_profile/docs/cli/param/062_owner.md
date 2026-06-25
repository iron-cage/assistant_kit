# Parameter :: 62. `owner::`

Mutation param on `.accounts` and `.usage` that sets or clears the `owner` field in `{name}.json`. The value `"0"` is a sentinel meaning "clear ownership" — identical to the former `unclaim::1` path. Any other non-empty value is assigned as owner. Supports comma-list `name::X,Y,Z` for batch operations.

- **Default:** *(omit)* — no owner write when absent
- **Constraints:** Non-empty string required; empty value (`owner::`) rejected (use `owner::0` to clear). Value `"0"` triggers the ownership-release path; all other non-empty values trigger the ownership-set path. `name::` supports comma-list (e.g., `name::X,Y,Z`) for batch; absent `name::` with `owner::0` batch-clears all owned accounts in filtered set.
- **Purpose:** Set or release account ownership via a single unified param. Replaces both `unclaim::1` (release) and the former no-CLI-path-to-set limitation.

**Behavior:**

```text
owner::user1@w003 name::X          → write_owner(X, store, "user1@w003")
owner::0 name::X                    → write_owner(X, store, "")     (release)
owner::0                             → batch-clear all owned accounts in filter
owner::user1@w003 name::X,Y,Z      → batch set owner for X, Y, Z
owner::0 name::X,Y,Z               → batch clear owner for X, Y, Z
owner::user1@w003 name::X dry::1   → preview without writing
owner::user1@w003 name::X force::1 → bypass G8 even if owned by another
owner::0 name::X force::1          → bypass G8 for ownership release
```

**`owner::0` sentinel:** The literal string `"0"` releases ownership — calls `write_owner(name, store, "")` to clear the `owner` field in `{name}.json`. This replaces the former `unclaim::1` param (removed in Feature 064). Batch: when `owner::0` is present and `name::` is absent, ownership is cleared for all accounts in the current filtered set; non-owned accounts are skipped with a `"skip"` message rather than exiting 1.

**Batch via comma-list:** `name::X,Y,Z` applies the operation to multiple accounts in one invocation. Each account is resolved and its G8 gate evaluated independently. Non-batch operations (single `name::X`) behave identically to before.

**G8 ownership gate:** Same gate for both set and clear paths. Account must be unowned or owned by the caller; otherwise exit 1 with `"ownership violation"`. Bypassed by `force::1`. In batch mode: per-account G8 check; non-owned accounts exit 1 per-account (when `force::1` absent) unless using the `owner::0` batch-clear (which skips, not exits, for non-owned accounts).

**Value format:** Non-`"0"` value is written as-is to `{name}.json`. Convention is `USER@HOSTNAME` (matching `current_identity()` output), but any non-empty string other than `"0"` is accepted.

**Notes:**
- Empty value (`owner::`) exits 1 with error directing to `owner::0`.
- `force::1` + `dry::1`: G8 bypassed, write previewed but not executed.
- Does not touch credentials, active marker, or any file other than `{name}.json`.
- `owner::0` is not a valid identity — it cannot be used to set ownership to the string `"0"`.

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | `.accounts` | Primary host — mutation param (unified param set, Feature 037) |
| 2 | `.usage` | Shared unified param set (Feature 037) |

### See Also

- [cli/param/056_unclaim.md](056_unclaim.md) — `unclaim::` — REMOVED; replaced by `owner::0`
- [cli/param/058_force.md](058_force.md) — `force::` — bypass G8 ownership gate
- [feature/064_active_marker_and_owner_redesign.md](../../feature/064_active_marker_and_owner_redesign.md) — `owner::0` sentinel + batch redesign
- [feature/063_explicit_ownership_claim.md](../../feature/063_explicit_ownership_claim.md) — original `owner::` design (set path)
- [feature/036_account_ownership.md](../../feature/036_account_ownership.md) — ownership model, G1–G8 gates

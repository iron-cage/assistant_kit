# Parameter :: 13. `active::`

> **Repurposed (Feature 064):** `active::` was formerly a `bool` field-presence toggle for `.credentials.status` (already removed from `.accounts` in Feature 037 AC-13). It is now a `Kind::String` mutation param — the value is the target `USER@MACHINE` identity for an active-account marker write or clear.
>
> **Migration from `assign::1` + `for::` (Feature 064):**
> - `assign::1 name::X` → `active::$USER@$HOSTNAME name::X` (current machine)
> - `assign::1 name::X for::bob@laptop` → `active::bob@laptop name::X`

Mutation param on `.accounts` and `.usage` that writes or clears the per-machine active-account marker (`_active_{machine}_{user}`) for any host+user pair. When `name::X` is provided, assigns X as the active account for the target identity. When `name::` is absent, clears (unassigns) the marker for the target identity.

- **Default:** *(omit)* — no marker write when absent
- **Constraints:** `Kind::String`; value format `USER@MACHINE` — split on first `@`; both parts required; each component sanitized per `active_marker_filename()` char-filter (alphanumeric, `-`, `.` kept; all others become `_`). The value `"0"` is NOT accepted (to clear ownership, use `owner::0`; to clear a marker, use `active::USER@MACHINE` without `name::`).
- **Purpose:** Assign or unassign which account a machine should use — without credential rotation. Replaces the former two-param `assign::1` + `for::` combination with a single unified param.

**Behavior:**

```text
active::user1@w003 name::X          → write _active_w003_user1 = X (assign)
active::user1@w003                   → clear _active_w003_user1    (unassign)
active::user1@w003 name::X dry::1   → preview without writing
active::user1@w003 name::X trace::1 → emit [trace] accounts active  assign: OK
```

**Value format and sanitization:**

Split on the **first** `@`:
- Left of `@` → user component → sanitized → second segment of `_active_{machine}_{user}`
- Right of `@` → machine component → sanitized → first segment of `_active_{machine}_{user}`

Sanitization: alphanumeric, `-`, `.` kept; all other characters become `_`. Identical to the former `for::` parameter rules.

**Examples:**

| `active::` value | Written filename |
|-----------------|-----------------|
| `user1@w003` | `_active_w003_user1` |
| `alice@my laptop` | `_active_my_laptop_alice` (space → `_`) |
| `alice@w003.local` | `_active_w003.local_alice` (dot preserved) |

**No ownership side effects:** `active::` does NOT modify the `owner` field in `{name}.json` — marker-only write. Ownership is managed by `owner::` and `owner::0`.

**No batch mode:** Unlike `owner::`, `active::` does not accept comma-list `name::X,Y,Z` — each machine/user has exactly one active account.

**Error cases:**
- `active::badvalue` (no `@`) → exit 1
- `active::@machine` or `active::user@` (empty component) → exit 1
- Account `name::` not found in credential store → exit 1

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | `.accounts` | Primary host — mutation param (Feature 064) |
| 2 | `.usage` | Shared unified param set (Feature 037) |

### See Also

- [cli/param/001_name.md](001_name.md) — `name::` — account identifier (required for assign; absent = unassign)
- [cli/param/004_dry.md](004_dry.md) — `dry::` — dry-run preview
- [cli/param/057_assign.md](057_assign.md) — `assign::` — REMOVED; replaced by this param
- [cli/param/053_for.md](053_for.md) — `for::` — REMOVED; functionality absorbed into `active::` value
- [feature/064_active_marker_and_owner_redesign.md](../../feature/064_active_marker_and_owner_redesign.md) — full redesign scope
- [feature/025_per_machine_active_marker.md](../../feature/025_per_machine_active_marker.md) — marker filename derivation and sanitization rules
- [feature/032_account_assign.md](../../feature/032_account_assign.md) — original assign behavior now implemented here

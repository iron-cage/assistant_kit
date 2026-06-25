# Parameter :: 13. `active::`

> **REMOVED (Feature 065):** The `active::` parameter has been removed from the unified parameter set on `.accounts` and `.usage`. Marker assignment is now performed via `assignee::USER@MACHINE name::X` (or `assignee::0 name::X` for current machine).
>
> **Migration:**
> - `active::user1@w003 name::X` → `assignee::user1@w003 name::X`
> - `active::user1@w003` (unassign) → `assignee::user1@w003`
> - `active::$USER@$HOSTNAME name::X` → `assignee::0 name::X` (current machine via sentinel)
>
> Using `active::` now exits 1 with a migration message.
>
> See [feature/065_assignee_param_redesign.md](../../feature/065_assignee_param_redesign.md) for full context.

[Historical specification retained below for reference.]

---

~~Mutation param on `.accounts` and `.usage` that writes or clears the per-machine active-account marker (`_active_{machine}_{user}`) for any host+user pair. When `name::X` is provided, assigns X as the active account for the target identity. When `name::` is absent, clears (unassigns) the marker for the target identity.~~

- **Default:** *(omit)* — no marker write when absent
- **Constraints:** `Kind::String`; value format `USER@MACHINE` — split on first `@`; both parts required; each component sanitized per `active_marker_filename()` char-filter (alphanumeric, `-`, `.` kept; all others become `_`). The value `"0"` is NOT accepted (to clear ownership, use `owner::0`; to clear a marker, use `active::USER@MACHINE` without `name::`).
- **Purpose:** Assign or unassign which account a machine should use — without credential rotation. Replaces the former two-param `assign::1` + `for::` combination with a single unified param.

**Behavior:**

```text
active::user1@w003 name::X          → write _active_w003_user1 = X (assign)
active::user1@w003                   → clear _active_w003_user1    (unassign)
active::user1@w003 name::X dry::1   → preview without writing
active::user1@w003 name::X trace::1 → emit ... · accounts active  assign: OK
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

- [cli/param/063_assignee.md](063_assignee.md) — `assignee::` — replacement param (Feature 065)
- [feature/065_assignee_param_redesign.md](../../feature/065_assignee_param_redesign.md) — removal context
- [feature/064_active_marker_and_owner_redesign.md](../../feature/064_active_marker_and_owner_redesign.md) — Feature 064 where `active::` was introduced as `Kind::String`
- [feature/025_per_machine_active_marker.md](../../feature/025_per_machine_active_marker.md) — marker filename derivation and sanitization rules

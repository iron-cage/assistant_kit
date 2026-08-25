# Parameter: 63. `assignee::`

Mutation param on `.accounts` and `.usage` that writes or clears the per-machine active-account marker (`_active_{machine}_{user}`) for any host+user pair. The value is either a `USER@MACHINE` identity or the sentinel `"0"` (meaning: current machine, expanded to `$USER@$HOSTNAME` before processing). When `name::X` is provided, assigns X as the active account for the target identity. When `name::` is absent, clears (unassigns) the marker for the target identity.

Renamed from `active::` (Feature 065). `active::` is now a REMOVED_TOGGLE.

- **Default:** *(omit)* — no marker write when absent
- **Constraints:** `Kind::String`; value is either `"0"` (current-machine sentinel) or `USER@MACHINE` format — split on first `@`; both parts required; each component sanitized per `active_marker_filename()` char-filter (alphanumeric, `-`, `.` kept; all others become `_`). Empty value (`assignee::`) rejected. Value must contain `@` or be exactly `"0"`.
- **Purpose:** Assign or unassign which account a machine should use — without credential rotation. Provides a canonical `assignee::` param symmetric with `owner::`, plus a `"0"` sentinel to target the current machine without requiring `$USER@$HOSTNAME` shell expansion at call sites.

**Behavior:**

```text
assignee::devuser@devbox name::X        → write _active_devbox_devuser = X (assign explicit)
assignee::0 name::X                     → expand 0→$USER@$HOSTNAME; write _active_{host}_{user} = X (assign current)
assignee::devuser@devbox                → clear _active_devbox_devuser    (unassign explicit)
assignee::0                             → expand 0→$USER@$HOSTNAME; clear _active_{host}_{user} (unassign current)
assignee::devuser@devbox name::X dry::1 → preview without writing
```

**`assignee::0` sentinel:** The literal string `"0"` expands to the current machine identity (`$USER@$HOSTNAME`) before all other processing. After expansion, processing is identical to `assignee::USER@MACHINE`. This is a **"current machine" shortcut** — the `"0"` does NOT mean "clear" (contrast: in `owner::`, `"0"` releases ownership).

**Value format and sanitization:**

Split on the **first** `@`:
- Left of `@` → user component → sanitized → second segment of `_active_{machine}_{user}`
- Right of `@` → machine component → sanitized → first segment of `_active_{machine}_{user}`

Sanitization: alphanumeric, `-`, `.` kept; all other characters become `_`. Identical to the former `active::` and `for::` parameter rules.

**Examples:**

| `assignee::` value | Resolves to | Written filename |
|-------------------|------------|-----------------|
| `devuser@devbox` | `devuser@devbox` | `_active_devbox_devuser` |
| `0` | `$USER@$HOSTNAME` | `_active_{hostname}_{user}` |
| `alice@my laptop` | `alice@my laptop` | `_active_my_laptop_alice` (space → `_`) |
| `alice@devbox.local` | `alice@devbox.local` | `_active_devbox.local_alice` (dot preserved) |

**No ownership side effects:** `assignee::` does NOT modify the `owner` field in `{name}.json` — marker-only write. Ownership is managed by `owner::` and `owner::0`.

**No batch mode:** Unlike `owner::`, `assignee::` does not accept comma-list `name::X,Y,Z` — each machine/user has exactly one active account.

**`force::` ignored:** `assignee::` has no ownership gate — `force::1` is silently ignored.

**Error cases:**
- `assignee::badvalue` (no `@`, not `"0"`) → exit 1 (invalid format)
- `assignee::@machine` or `assignee::user@` (empty component) → exit 1
- Account `name::` not found in credential store → exit 1, message names the source
  param explicitly (`account 'X' (from name::) not found in credential store`) — the
  looked-up value always derives from `name::`, never from `assignee::`'s own value,
  even though both parameters are present on the same command line (BUG-342 fix)
- `active::` → exit 1 with REMOVED_TOGGLE migration message

### Referenced Type

- **Fundamental Type:** `string`

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | `.accounts` | Primary host — mutation param (Feature 065) |
| 2 | `.usage` | Shared unified param set (Feature 037) |

### Referenced User Stories

| # | User Story | Persona |
|---|------------|---------|
| 1 | [Account Onboarding and Lifecycle Management](../user_story/002_onboarding.md) | Write per-machine active-account marker via `assignee::USER@MACHINE name::X` |

### See Also

- [cli/param/001_name.md](001_name.md) — `name::` — account identifier (required for assign; absent = unassign)
- [cli/param/004_dry.md](004_dry.md) — `dry::` — dry-run preview
- [cli/param/013_active.md](013_active.md) — `active::` — REMOVED; replaced by this param
- [feature/065_assignee_param_redesign.md](../../feature/065_assignee_param_redesign.md) — full redesign scope
- [feature/025_per_machine_active_marker.md](../../feature/025_per_machine_active_marker.md) — marker filename derivation and sanitization rules
- [feature/032_account_assign.md](../../feature/032_account_assign.md) — original assign behavior

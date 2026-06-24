# Feature: Active Marker and Owner Param Redesign

### Scope

- **Purpose**: Replace the two-param `assign::1` + `for::` combination with a single `active::USER@MACHINE` param; add `owner::0` sentinel to `owner::` for ownership release; support batch comma-list in `name::` for `owner::` operations; remove `unclaim::1` as an active param (REMOVED_TOGGLE → `owner::0`).
- **Responsibility**: Documents the repurposed `active::` param (`Kind::String`, value = `USER@MACHINE`) for marker assign/unassign; `owner::0` sentinel for ownership release; comma-list batch in `name::` for `owner::`; REMOVED_TOGGLE stubs for `assign`, `for`, and `unclaim` with migration messages.
- **In Scope**: `active::USER@MACHINE` param — assign when `name::` present, unassign when absent; `owner::0` sentinel (value string `"0"`) replacing `unclaim::1` ownership release path; `owner::USER@MACHINE name::X,Y,Z` batch ownership set; `owner::0` alone batch-clears all owned accounts; REMOVED_TOGGLE entries for `assign`, `for`, `unclaim` emitting migration messages; all of `.accounts` and `.usage` (unified param set per Feature 037).
- **Out of Scope**: Ownership model design (→ [036_account_ownership.md](036_account_ownership.md)); active marker filename derivation (→ [025_per_machine_active_marker.md](025_per_machine_active_marker.md)); credential rotation (→ [004_account_use.md](004_account_use.md)).

### Design

**Background:** Feature 037 introduced `assign::1` + `for::USER@MACHINE` as a two-param combination for marker assignment, and `unclaim::1` for ownership release. This design has two friction points: (1) two params are required for a single logical operation (assign a marker for a specific machine), and (2) `unclaim::1` is a separate param from `owner::` despite performing the inverse of what `owner::` does. Feature 064 unifies both.

**`active::` repurposed:** The `active::` param changes from `Kind::Bool` (a field-presence toggle that was already removed from `.accounts` in Feature 037 AC-13) to `Kind::String` (a mutation param). The value is the target `USER@MACHINE` identity whose marker is written or cleared.

```
active::USER@MACHINE name::X   → write _active_{machine}_{user} = X (assign)
active::USER@MACHINE           → clear _active_{machine}_{user}  (unassign)
```

The value format, sanitization rules, and split semantics (on first `@`) are identical to the former `for::` parameter. The current machine default is no longer implicit — `active::` always requires an explicit `USER@MACHINE` value. To target the current machine, pass `active::$USER@$HOSTNAME name::X`.

**`owner::0` sentinel:** The string value `"0"` is a sentinel meaning "clear ownership" — identical to the former `unclaim::1` path (`write_owner(name, store, "")`). Any non-empty, non-`"0"` value is the identity to assign as owner. The sentinel avoids requiring a separate boolean param for the inverse operation.

```
owner::0 name::X           → write_owner(X, store, "")   (release ownership)
owner::USER@MACHINE name::X → write_owner(X, store, identity)  (assign ownership)
```

**Batch via comma-list in `name::`:** Both `owner::0` and `owner::USER@MACHINE` accept a comma-separated list in `name::` to operate on multiple accounts in one invocation. Each account is resolved independently; G8 is evaluated per account.

```
owner::0 name::X,Y,Z               → clear ownership for X, Y, and Z
owner::user1@w003 name::X,Y,Z      → set ownership for X, Y, and Z
```

**`owner::0` batch-clear (no `name::`):** When `owner::0` is present and `name::` is absent, ownership is cleared for all owned accounts in the current filtered set (same behavior as the former `unclaim::1` no-`name::` batch path). G8 is evaluated per account; non-owned accounts are skipped with a `"skip"` message rather than exiting 1.

**REMOVED_TOGGLE stubs:** `assign`, `for`, and `unclaim` are registered as `bfd()` (REMOVED_TOGGLE / dead-flag) entries in `registry.rs`. Any invocation emits a migration message and exits 1.

| Removed param | Migration message |
|---------------|-------------------|
| `assign::` | "REMOVED — use `active::USER@MACHINE name::X` instead" |
| `for::` | "REMOVED — functionality absorbed into `active::` value: `active::USER@MACHINE name::X`" |
| `unclaim::` | "REMOVED — use `owner::0 name::X` instead (or `owner::0` alone to batch-clear)" |

**`force::` bypass:** The `force::1` bypass for G8 applies to both `owner::0` and `owner::USER@MACHINE` paths (same as it applied to `unclaim::1` and `owner::USER@MACHINE` in prior design). The `active::` param has no ownership gate — `force::1` has no effect on `active::`.

**Value format — `active::` sanitization:** The `active::` value is split on the first `@`: left → user component, right → machine component. Each component is sanitized per the same char-filter as `active_marker_filename()`: alphanumeric, `-`, `.` kept; all other characters become `_`. Examples mirror the former `for::` examples.

### Acceptance Criteria

- **AC-01**: `clp .accounts active::user1@w003 name::alice@corp.com` writes `{credential_store}/_active_w003_user1` = `alice@corp.com`; exits 0; stdout contains `assigned alice@corp.com for user1@w003  →  _active_w003_user1`. No credential files modified.
- **AC-02**: `clp .accounts active::user1@w003` (no `name::`) clears (empties or deletes) `{credential_store}/_active_w003_user1`; exits 0; stdout contains `unassigned user1@w003  →  _active_w003_user1 cleared`. No credential files modified.
- **AC-03**: `clp .accounts active::user1@w003 name::alice@corp.com dry::1` exits 0; stdout contains `[dry-run] would assign alice@corp.com for user1@w003  →  _active_w003_user1`; no files written.
- **AC-04**: `clp .accounts active::user1@w003 name::ghost@example.com` when account not in credential store exits 1 with account-not-found error; no marker file written.
- **AC-05**: `clp .accounts assign::1 name::X` exits 1 with migration message: "REMOVED — use `active::USER@MACHINE name::X` instead". No files modified.
- **AC-06**: `clp .accounts assign::1 name::X for::bob@laptop` exits 1 (both `assign::1` and `for::` trigger REMOVED_TOGGLE messages). No files modified.
- **AC-07**: `clp .accounts unclaim::1 name::X` exits 1 with migration message: "REMOVED — use `owner::0 name::X` instead (or `owner::0` alone to batch-clear)". No files modified.
- **AC-08**: `clp .accounts owner::0 name::alice@corp.com` writes `owner: ""` to `{name}.json` via `write_owner()`; exits 0; stdout contains `unclaimed alice@corp.com`; G8 gate evaluated before write; credentials NOT touched.
- **AC-09**: `clp .accounts owner::0` (no `name::`) clears ownership for all owned accounts in the credential store (pre-filter — display-filter params such as `only_valid::`, `exclude_exhausted::`, and `min_5h::` do NOT scope the batch-clear); per-account G8 check; non-owned accounts (unowned or owned by another identity) are skipped with a `"skip"` message rather than exiting 1; exits 0.
- **AC-10**: `clp .accounts owner::0 name::X,Y,Z` clears ownership for X, Y, and Z; each evaluated against G8 independently; when an account is owned by another identity and `force::1` is absent, that account exits 1 per-account (unlike the no-`name::` batch-clear which skips non-owned accounts); exits 0 when all succeed.
- **AC-11**: `clp .accounts owner::user1@w003 name::X,Y,Z` sets ownership for X, Y, and Z; each evaluated against G8 independently; exits 0.
- **AC-12**: `clp .accounts owner::0 name::X force::1` bypasses G8 for X even when owned by a different identity; exits 0.
- **AC-13**: `active::user1@w003` sanitizes value correctly — e.g., `active::alice@my laptop` writes `_active_my_laptop_alice` (space → `_`). Dot and hyphen in machine component preserved verbatim.
- **AC-14**: `active::` does NOT modify the `owner` field in `{name}.json` — marker-only write, identical to the former `assign::1` ownership-neutral behavior.
- **AC-15**: `owner::` with empty string value (`owner::`) still exits 1 with message directing user to `owner::0` for ownership release (empty string ≠ sentinel `"0"`).
- **AC-16**: `clp .accounts owner::0 name::X dry::1` prints `[dry-run] would clear owner of X`; exits 0; no files written. G8 gate still runs before dry-run check.
- **AC-17**: `clp .accounts owner::0 name::X force::1 dry::1` bypasses G8 AND respects dry-run — prints `[dry-run] would clear owner of X`; no files written.
- **AC-18**: `clp .accounts active::0 name::alice@corp.com` exits 1; value `"0"` contains no `@` and is therefore rejected by the `USER@MACHINE` format validation (same path as AC error cases); error message indicates invalid format and directs user to `owner::0` for ownership release or `active::USER@MACHINE` without `name::` for marker unassign. No files modified.
- **AC-19**: `clp .accounts active::user1@w003 dry::1` (no `name::`) exits 0; stdout contains `[dry-run] would unassign user1@w003  →  _active_w003_user1 cleared`; no `_active_*` file modified or deleted.

### Features

| File | Relationship |
|------|--------------|
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | Marker filename derivation; `active_marker_filename()` sanitization rules used by `active::` value |
| [032_account_assign.md](032_account_assign.md) | Former `.accounts assign::1` + `for::` behavior now implemented via `active::` |
| [036_account_ownership.md](036_account_ownership.md) | `owner` field, G8 gate, `write_owner()` — `owner::0` sentinel triggers `write_owner(name, store, "")` |
| [037_accounts_usage_param_unification.md](037_accounts_usage_param_unification.md) | Param set context — `assign::1`/`for::` removed; `unclaim::1` removed; `active::` added to mutation set |
| [063_explicit_ownership_claim.md](063_explicit_ownership_claim.md) | `owner::USER@MACHINE` set path; `owner::` param; G8 gate shared with `owner::0` |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/013_active.md](../cli/param/013_active.md) | `active::` — repurposed mutation param (String); assign/unassign marker |
| [cli/param/053_for.md](../cli/param/053_for.md) | `for::` — REMOVED; functionality absorbed into `active::` value |
| [cli/param/056_unclaim.md](../cli/param/056_unclaim.md) | `unclaim::` — REMOVED; replaced by `owner::0` |
| [cli/param/057_assign.md](../cli/param/057_assign.md) | `assign::` — REMOVED; replaced by `active::USER@MACHINE name::X` |
| [cli/param/062_owner.md](../cli/param/062_owner.md) | `owner::` — extended with `owner::0` sentinel; batch via comma-list `name::` |
| [cli/param/058_force.md](../cli/param/058_force.md) | `force::` — bypass G8 for `owner::0` and `owner::USER@MACHINE`; no effect on `active::` |
| [cli/param/001_name.md](../cli/param/001_name.md) | `name::` — now supports comma-list `X,Y,Z` for batch `owner::` operations |
| [cli/param/004_dry.md](../cli/param/004_dry.md) | `dry::` — preview without writing; applies to both `active::` and `owner::0` |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.accounts` — Command 3; primary host for new params |
| [cli/command/006_usage.md](../cli/command/006_usage.md) | `.usage` — Command 9; shares unified param set |

### Sources

| File | Relationship |
|------|--------------|
| `src/registry.rs` | `bfd("assign", ...)`, `bfd("for", ...)`, `bfd("unclaim", ...)` → REMOVED_TOGGLEs; `reg_arg_opt("active", Kind::String)` added to both `.accounts` and `.usage` |
| `src/commands/accounts.rs` | `active::` dispatch (assign/unassign); `owner::` dispatch updated for `owner::0` sentinel + comma-list batch |
| `src/usage/api.rs` | Same `active::` and `owner::` dispatch changes for `.usage` side |
| `claude_profile_core/src/account.rs` | `write_owner()` — called by `owner::0` path; `active_marker_filename()` sanitization re-used by `active::` value parser |

### Tests

| File | Relationship |
|------|--------------|
| [tests/docs/feature/64_active_marker_and_owner_redesign.md](../../tests/docs/feature/64_active_marker_and_owner_redesign.md) | FT spec mapping ACs to test cases |
| [tests/docs/cli/param/14_active.md](../../tests/docs/cli/param/14_active.md) | EC edge cases for repurposed `active::` param |
| [tests/docs/cli/param/63_owner.md](../../tests/docs/cli/param/63_owner.md) | EC edge cases for `owner::0` sentinel + batch |

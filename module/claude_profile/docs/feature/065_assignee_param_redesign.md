# Feature: Assignee Param Redesign

### Scope

- **Purpose**: Rename `active::` → `assignee::` to establish a canonical term for the machine/user identity that holds an account as currently active; introduce `assignee::0` as a "current machine" sentinel (symmetric with `owner::0` = release) to eliminate the need for `$USER@$HOSTNAME` shell expansion at call sites; register `active::` as a REMOVED_TOGGLE with a migration message.
- **Responsibility**: Documents the renamed `assignee::` param (`Kind::String`, value = `USER@MACHINE` or sentinel `"0"`); the `assignee::0` expansion rule; the REMOVED_TOGGLE stub for `active::`.
- **In Scope**: `assignee::USER@MACHINE` param — assign when `name::` present, unassign when absent; `assignee::0` sentinel expanding to `$USER@$HOSTNAME` before processing; REMOVED_TOGGLE stub for `active::` emitting a migration message; all of `.accounts` and `.usage` (unified param set per Feature 037).
- **Out of Scope**: Marker filename derivation and sanitization rules (→ [025_per_machine_active_marker.md](025_per_machine_active_marker.md)); ownership model (→ [036_account_ownership.md](036_account_ownership.md)); `owner::` param design (→ [063_explicit_ownership_claim.md](063_explicit_ownership_claim.md), [064_active_marker_and_owner_redesign.md](064_active_marker_and_owner_redesign.md)).

### Design

**Background:** Feature 064 introduced `active::USER@MACHINE` as the marker assign/unassign param, replacing the former `assign::1` + `for::` two-param combination. However, `active::` carries historical baggage (it was formerly a `bool` field-presence toggle) and, more importantly, lacks a canonical vocabulary counterpart to "owner" / `owner::`. Feature 065 establishes **assignee** as the canonical term for the USER@MACHINE identity holding an account as currently active via `_active_{machine}_{user}` marker file, and renames the param accordingly.

**`assignee::` param:** Direct rename of `active::` (`Kind::String`). All value format, sanitization, assign/unassign semantics, dry-run behavior, and isolation guarantees remain identical. The param name change is the only behavioral change to the positive path.

```
assignee::USER@MACHINE name::X   → write _active_{machine}_{user} = X (assign)
assignee::USER@MACHINE           → clear _active_{machine}_{user}      (unassign)
```

**`assignee::0` sentinel:** The string value `"0"` expands to the current machine identity (`$USER@$HOSTNAME`) before all other processing. This eliminates the need for shell expansion at call sites and establishes symmetry with `owner::` (where `"0"` is also a special sentinel, though with a different meaning: release ownership). After expansion, processing continues identically to `assignee::USER@MACHINE`.

```
assignee::0 name::X    → expand to assignee::$USER@$HOSTNAME name::X (assign current machine)
assignee::0            → expand to assignee::$USER@$HOSTNAME          (unassign current machine)
```

The `"0"` sentinel in `assignee::` is a **"current machine" shortcut** — it does NOT mean "clear" (contrast: in `owner::`, `"0"` = release ownership). Both `assignee::0` and `assignee::USER@MACHINE` follow the same code path after expansion.

**REMOVED_TOGGLE for `active::`:** `active::` is registered as a `bfs()` (REMOVED_TOGGLE / dead-flag, `Kind::String` — `bfs` is required so the framework accepts `USER@MACHINE`-format values at the parser level and routes to the routine, which then emits the migration message) entry in `registry.rs`. Any invocation emits a migration message and exits 1.

| Removed param | Migration message |
|---------------|-------------------|
| `active::` | "REMOVED — use `assignee::USER@MACHINE name::X` (or `assignee::0 name::X` for current machine)" |

**Vocabulary alignment:**

| Concept | Param | Sentinel `"0"` meaning |
|---------|-------|------------------------|
| Ownership (persistent, in `{name}.json`) | `owner::` | Release/clear ownership |
| Assignment (transient, in `_active_*` file) | `assignee::` | Current machine ($USER@$HOSTNAME) |

**Symmetry illustration:**

```
# Set ownership for alice:
owner::user1@w003 name::alice@corp.com    # explicit identity
owner::0 name::alice@corp.com             # 0 = RELEASE (clear owner field)

# Assign marker for alice:
assignee::user1@w003 name::alice@corp.com # explicit identity
assignee::0 name::alice@corp.com          # 0 = CURRENT MACHINE ($USER@$HOSTNAME)

# Unassign marker:
assignee::user1@w003                      # clear explicit identity's marker
assignee::0                               # clear current machine's marker
```

**`force::` bypass:** `assignee::` has no ownership gate — `force::1` is silently ignored when combined with `assignee::`. (Same behavior as `active::` in Feature 064.)

### Acceptance Criteria

- **AC-01**: `clp .accounts assignee::user1@w003 name::alice@corp.com` writes `{credential_store}/_active_w003_user1` = `alice@corp.com`; exits 0; stdout contains `assigned alice@corp.com for user1@w003  →  _active_w003_user1`. No credential files modified.
- **AC-02**: `clp .accounts assignee::0 name::alice@corp.com` expands `$USER@$HOSTNAME`, writes `{credential_store}/_active_{hostname}_{user}` = `alice@corp.com`; exits 0; stdout contains `assigned alice@corp.com for {user}@{hostname}  →  _active_{hostname}_{user}`.
- **AC-03**: `clp .accounts assignee::user1@w003` (no `name::`) clears `{credential_store}/_active_w003_user1`; exits 0; stdout contains `unassigned user1@w003  →  _active_w003_user1 cleared`. No credential files modified.
- **AC-04**: `clp .accounts assignee::0` (no `name::`) expands `$USER@$HOSTNAME`, clears `{credential_store}/_active_{hostname}_{user}`; exits 0; stdout contains `unassigned {user}@{hostname}  →  _active_{hostname}_{user} cleared`.
- **AC-05**: `clp .accounts assignee::user1@w003 name::alice@corp.com dry::1` exits 0; stdout contains `[dry-run] would assign alice@corp.com for user1@w003  →  _active_w003_user1`; no files written.
- **AC-06**: `clp .accounts assignee::0 name::alice@corp.com dry::1` exits 0; stdout contains `[dry-run] would assign alice@corp.com for {user}@{hostname}  →  _active_{hostname}_{user}`; no files written.
- **AC-07**: `clp .accounts assignee::0 dry::1` (no `name::`) exits 0; stdout contains `[dry-run] would unassign {user}@{hostname}  →  _active_{hostname}_{user} cleared`; no `_active_*` file modified or deleted.
- **AC-08**: `clp .accounts assignee::user1@w003 name::ghost@example.com` when account not in credential store exits 1 with account-not-found error; no marker file written.
- **AC-09**: `clp .accounts assignee::badvalue` (no `@` and value ≠ `"0"`) exits 1 with invalid `USER@MACHINE` format error; no `_active_*` file written.
- **AC-10**: `clp .accounts active::user1@w003 name::alice@corp.com` exits 1; stderr contains migration message: "REMOVED — use `assignee::USER@MACHINE name::X` (or `assignee::0 name::X` for current machine)". No files modified.
- **AC-11**: `assignee::` does NOT modify the `owner` field in `{name}.json` — marker-only write, ownership-neutral.
- **AC-12**: `assignee::` value sanitization: spaces → `_`; dots and hyphens preserved verbatim. Example: `assignee::alice@my laptop` → `_active_my_laptop_alice`.
- **AC-13**: `clp .accounts assignee::user1@w003 name::alice@corp.com force::1` — `force::1` silently ignored; marker written identically to without `force::1`; exits 0.

### Bugs

| ID | Summary | Status |
|----|---------|--------|
| *(none)* | | |

### Features

| File | Relationship |
|------|--------------|
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | Marker filename derivation; `active_marker_filename()` sanitization rules used by `assignee::` value |
| [032_account_assign.md](032_account_assign.md) | Former `.accounts assign::1` + `for::` behavior; original marker write behavior |
| [036_account_ownership.md](036_account_ownership.md) | Ownership model; `owner` field; `write_owner()`; nine enforcement gates — `assignee::` bypasses all (no gate) |
| [063_explicit_ownership_claim.md](063_explicit_ownership_claim.md) | `owner::USER@MACHINE` set path; G8 gate shared with `owner::0` |
| [064_active_marker_and_owner_redesign.md](064_active_marker_and_owner_redesign.md) | Predecessor: introduced `active::KIND::String`; this feature renames `active::` → `assignee::` and adds `assignee::0` sentinel |
| [070_account_claim_and_reservation_control.md](070_account_claim_and_reservation_control.md) | `assignee::` is the structural precedent for `lock::`/`reserve::`'s ungated write path (`force::` without effect); G9 also gates `assignee::` target-side |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/063_assignee.md](../cli/param/063_assignee.md) | `assignee::` — renamed mutation param; assign/unassign marker; `0` sentinel = current machine |
| [cli/param/013_active.md](../cli/param/013_active.md) | `active::` — REMOVED (this feature); replaced by `assignee::` |
| [cli/param/001_name.md](../cli/param/001_name.md) | `name::` — account identifier (required for assign; absent = unassign) |
| [cli/param/004_dry.md](../cli/param/004_dry.md) | `dry::` — preview without writing |
| [cli/param/058_force.md](../cli/param/058_force.md) | `force::` — no effect on `assignee::`; silently ignored |

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md) | `.accounts` — Command 3; primary host for `assignee::` |
| [cli/command/006_usage.md](../cli/command/006_usage.md) | `.usage` — Command 9; shares unified param set |

### Sources

| File | Relationship |
|------|--------------|
| `src/registry.rs` | `bfs("active", ...)` → REMOVED_TOGGLE (Kind::String — accepts USER@MACHINE values at parser level so routine can show migration message); `bfs("for", ...)` updated to point to `assignee::`; `reg_arg_opt("assignee", Kind::String)` added to both `.accounts` and `.usage` |
| `src/commands/accounts.rs` | `assignee::` dispatch (assign/unassign); `assignee::0` sentinel expansion before processing |
| `src/usage/api.rs` | Same `assignee::` dispatch and `assignee::0` expansion for `.usage` side |
| `claude_profile_core/src/account.rs` | `active_marker_filename()` sanitization re-used by `assignee::` value parser; `current_identity()` used for `assignee::0` expansion |

### Tests

| File | Relationship |
|------|--------------|
| [tests/docs/feature/065_assignee_param_redesign.md](../../tests/docs/feature/065_assignee_param_redesign.md) | FT spec mapping ACs to test cases |
| [tests/docs/cli/param/64_assignee.md](../../tests/docs/cli/param/64_assignee.md) | EC edge cases for `assignee::` param |

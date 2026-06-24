# Feature: Explicit Ownership Claim

### Scope

- **Purpose**: Provide a CLI-exposed write path for the `owner` field in `{name}.json`, allowing any identity to be explicitly assigned as account owner.
- **Responsibility**: Documents the `owner::` mutation parameter on `.accounts` and `.usage` — explicit ownership assignment via `write_owner(name, store, identity)`, G8 gate enforcement, mutual exclusion with `unclaim::1`, and interaction with `dry::1`, `force::1`, `trace::1`.
- **In Scope**: `owner::USER@MACHINE` parameter (set path); G8 ownership gate on the write path; `force::1` bypass; `dry::1` preview; `trace::1` diagnostic; `name::` required for set path (single-account; batch set via comma-list introduced in Feature 064); prefix resolution via `resolve_account_name()`. (`owner::0` release sentinel and batch support → Feature 064.)
- **Out of Scope**: Ownership model design (→ [036_account_ownership.md](036_account_ownership.md)); identity resolution mechanics (→ [025_per_machine_active_marker.md](025_per_machine_active_marker.md)); active marker assignment (→ [032_account_assign.md](032_account_assign.md)).

### Design

**Background:** Prior to this feature, the `owner` field could only be CLEARED via `.accounts unclaim::1` (calls `write_owner(name, store, "")`). There was no CLI-exposed path to SET the `owner` field to a non-empty value — direct JSON editing was the only option. (Feature 064 subsequently replaced `unclaim::1` with `owner::0` sentinel; the release path is now `owner::0 name::X`. The `owner::USER@MACHINE` set path from this feature is unchanged.)

**`owner::` parameter:** A `Kind::String` mutation parameter registered on both `.accounts` and `.usage` (unified param set, Feature 037). When `owner::VALUE` is provided alongside `name::X`:

1. Resolve `name::X` via `resolve_account_name()` (prefix resolution)
2. Evaluate G8 ownership gate: `read_owner()` → `is_owned()`. If account is owned by a different identity → exit 1 with `"ownership violation: this account is owned by {owner}"` (unless `force::1`)
3. If `dry::1`: print `[dry-run] would set owner of {name} to {value}` → exit 0; no file writes
4. Call `write_owner(name, credential_store, value)` — writes `"owner": "{value}"` to `{name}.json`
5. Print: `owned {name} by {value}`
6. If `trace::1`: emit `[trace] accounts owner  write_owner: OK  name={name} identity={value}`

**Value format:** The `owner::` value is an opaque string written as-is to the `owner` field. The conventional format is `USER@HOSTNAME` (matching `current_identity()` output), but the field accepts any non-empty string. An empty `owner::` value is rejected (use `unclaim::1` to clear).

**Mutual exclusion (post-Feature 064):** Since `unclaim::1` has been removed (Feature 064), the former mutual exclusion with `unclaim::1` no longer applies. `owner::USER@MACHINE` and `owner::0` are values of the same param — they cannot co-exist in a single invocation (one value replaces the other). The only remaining constraint: `owner::` (empty value) is rejected; use `owner::0` to clear.

**No batch mode:** Unlike `unclaim::1` (which supports batch when `name::` is absent), `owner::VALUE` requires `name::X`. Batch ownership assignment is out of scope for the initial implementation.

**G8 gate behavior for `owner::`:**

| Account state | `owner::VALUE` | `owner::VALUE force::1` |
|---------------|----------------|-------------------------|
| Unowned (owner="" or absent) | ✅ Gate passes → write | ✅ Gate passes → write |
| Owned by caller | ✅ Gate passes → write | ✅ Gate passes → write |
| Owned by different identity | ❌ Exit 1 ownership violation | ✅ Force bypass → write |

This matches the G8 pattern used by `unclaim::1`.

### Acceptance Criteria

- **AC-01**: `clp .accounts owner::user1@w003 name::illia` writes `"owner": "user1@w003"` to `{illia}.json` via `write_owner()`; exits 0; stdout contains `owned {name} by user1@w003`.
- **AC-02**: `name::` is required — `clp .accounts owner::user1@w003` without `name::` exits 1 with an error message.
- **AC-03**: G8 gate: when `{name}.json` has `owner: "other@host"` and caller is not `other@host`, `clp .accounts owner::me@here name::X` exits 1 with `"ownership violation: this account is owned by other@host"`.
- **AC-04**: Unowned account (owner="" or absent): gate passes — `owner::VALUE` writes successfully.
- **AC-05**: `clp .accounts owner::` (empty value) exits 1 with error directing user to `owner::0` for ownership release. (Former `unclaim::1` mutual exclusion no longer applies — `unclaim::` is now REMOVED; `owner::0` is the release sentinel.)
- **AC-06**: `clp .accounts owner::user1@w003 name::X dry::1` prints `[dry-run] would set owner of {name} to user1@w003`; exits 0; no files written.
- **AC-07**: `clp .accounts owner::user1@w003 name::X force::1` when account is owned by a different identity → G8 bypassed → `write_owner()` succeeds; exits 0.
- **AC-08**: `clp .accounts owner::user1@w003 name::X trace::1` emits `[trace] accounts owner  write_owner: OK  name={name} identity=user1@w003` to stderr.
- **AC-09**: Prefix resolution: `owner::user1@w003 name::ill` resolves `ill` to the full account email if unambiguous.
- **AC-10**: `owner::` with empty value exits 1 with error directing user to use `owner::0` to clear ownership.
- **AC-11**: After setting owner, all G1–G8 gates respect the new owner on subsequent operations from any identity.
- **AC-12**: `.usage owner::user1@w003 name::X` works identically to `.accounts owner::user1@w003 name::X` — same write path, same gates, same output (mutation executes before table render, same as `unclaim::1` and `assign::1`).

### Features

| File | Relationship |
|------|--------------|
| [036_account_ownership.md](036_account_ownership.md) | Core ownership model — `owner` field, `write_owner()`, G1–G8 gates, `is_owned()` predicate |
| [032_account_assign.md](032_account_assign.md) | `.accounts assign::1` is marker-only — does NOT set owner (contrast point) |
| [037_accounts_usage_param_unification.md](037_accounts_usage_param_unification.md) | Unified param set — `owner::` registered on both `.accounts` and `.usage` |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/062_owner.md](../cli/param/062_owner.md) | `owner::` — explicit ownership assignment parameter |
| [cli/param/056_unclaim.md](../cli/param/056_unclaim.md) | `unclaim::` — REMOVED (Feature 064); replaced by `owner::0` sentinel |
| [cli/param/058_force.md](../cli/param/058_force.md) | `force::` — bypass G8 ownership gate |
| [cli/param/001_name.md](../cli/param/001_name.md) | `name::` — required account identifier with prefix resolution |
| [cli/param/004_dry.md](../cli/param/004_dry.md) | `dry::` — dry-run preview |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/accounts.rs` | `accounts_routine()` — `owner::` write path alongside existing `unclaim::1` and `assign::1` |
| `src/usage/api.rs` | `usage_routine()` — `owner::` write path (same logic, shared unified param set) |
| `claude_profile_core/src/account.rs` | `write_owner()` — the underlying write API called by both paths |

### Tests

| File | Relationship |
|------|--------------|
| [tests/docs/feature/63_explicit_ownership_claim.md](../../tests/docs/feature/63_explicit_ownership_claim.md) | FT spec mapping ACs to test cases |
| [tests/docs/cli/param/63_owner.md](../../tests/docs/cli/param/63_owner.md) | EC edge case spec for `owner::` parameter |

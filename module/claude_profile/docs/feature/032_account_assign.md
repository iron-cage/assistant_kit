# Feature: Account Marker Assignment

### Scope

- **Purpose**: Allow writing the per-machine active-account marker for any host+user pair without credential rotation.
- **Responsibility**: Documents the `.account.assign` CLI command — marker-only write, `for::USER@MACHINE` parameter, dry-run, live usage block when called without `name::`.
- **In Scope**: `.account.assign` command; `for::USER@MACHINE` parameter; sanitization rules for marker filename; dry-run; live usage block output when `name::` absent; marker write only (no credential copy, no `~/.claude.*` side effects, no `owner` field changes).
- **Out of Scope**: Full credential rotation (→ 004_account_use.md); per-machine marker filename derivation (→ 025_per_machine_active_marker.md); account-save host display label (→ 029_account_host_metadata.md); credential file access control (enforcement is logical, not filesystem-level); ownership stamp (→ 036_account_ownership.md, managed by `.account.save`).

> **CLI surface migration (Feature 037 — shipped, then Feature 064 — shipped):** This feature's behavior was absorbed into `.accounts` as `assign::1` + `for::` parameters in Feature 037. Those params were replaced by `active::USER@MACHINE` in Feature 064. The standalone `.account.assign` command **has been removed** (deregistered from the command registry — produces generic 'unknown command' error). All acceptance criteria below apply via `clp .accounts active::USER@MACHINE name::X`. See [037_accounts_usage_param_unification.md](037_accounts_usage_param_unification.md) and [064_active_marker_and_owner_redesign.md](064_active_marker_and_owner_redesign.md).

### Design

`.account.assign` writes (or overwrites) the per-machine active-account marker file for any host+user pair in the credential store, without performing a credential rotation or touching any `~/.claude.*` file.

**Primary use case:** pre-seeding which account a machine should use when accessing a shared or synced credential store — without being logged into that machine at the time.

#### Marker file

The written file is `{credential_store}/_active_{machine}_{user}` containing the account name as plain text. This is the same filename format produced by `active_marker_filename()` in Feature 025. The same sanitization rules apply to the components parsed from `for::`:

- Keep alphanumeric, `-`, `.`; replace all other characters with `_`
- Split `for::USER@MACHINE` on the first `@`: left part → user component, right part → machine component

Examples:

| `for::` value | Written filename | Written machine | Written user |
|---------------|-----------------|-----------------|--------------|
| `alice@laptop` | `_active_laptop_alice` | `laptop` | `alice` |
| `user1@w003.local` | `_active_w003.local_user1` | `w003.local` | `user1` |
| `alice@my laptop` | `_active_my_laptop_alice` | `my_laptop` (space→`_`) | `alice` |

When `for::` is omitted, the target is the current machine — same resolved values as `active_marker_filename()` (`$USER`/`$USERNAME`/`"user"` and `resolve_hostname()`).

#### Live usage block

When called with no `name::` argument, the command emits a context-aware usage block (to stdout) and exits 0:

```
.account.assign — write the active-account marker for any machine without credential rotation.

  name::   account to assign (required)
  for::    USER@MACHINE to target  (default: current machine)
  dry::1   preview without writing

Current machine:  {user}@{machine}  (→ _active_{machine}_{user})
Active account:   {active}

Ready to copy:
  clp .account.assign name::{active}
  clp .account.assign name::{active} for::{user}@{machine}
  clp .account.assign name::{active} for::otheruser@othermachine dry::1
```

Where `{machine}` and `{user}` are the current machine's resolved values (same sources as `active_marker_filename()`), and `{active}` is the content of the own `_active_{machine}_{user}` marker file. When no active account is set, `{active}` shows `(none)` and the `Ready to copy:` section is omitted.

#### Parameters

| Param | Kind | Required | Default | Notes |
|-------|------|----------|---------|-------|
| `name::` | String | no¹ | — | Account to assign; prefix resolution via `resolve_account_name()` |
| `for::` | String | no | `$USER@resolve_hostname()` | Target identity as `USER@MACHINE`; both parts required when provided |
| `dry::` | Int | no | 0 | Print what would be written without writing |

¹ When `name::` is absent the command emits the live usage block and exits 0.

#### Execution steps

1. If `name::` absent → emit live usage block; exit 0
2. Resolve `name::` via `resolve_account_name()` (validates existence and safe characters)
3. If `for::` provided: split on first `@`; require both parts non-empty; sanitize each component (alphanumeric, `-`, `.`; others → `_`)
4. If `for::` absent: user = `$USER`/`$USERNAME`/`"user"` fallback; machine = `resolve_hostname()`
5. Construct `_active_{machine}_{user}` filename
6. If `dry::1`: print `[dry-run] would assign {name} for {user}@{machine}  →  _active_{machine}_{user}`; exit 0
7. Write `{name}` to `{credential_store}/_active_{machine}_{user}`
8. Print: `Assigned {name} for {user}@{machine}  →  _active_{machine}_{user}`

**No credential side effects.** `~/.claude/.credentials.json`, `~/.claude.json`, `~/.claude/settings.json`, and `{name}.json` are never touched by `.account.assign`. Only the marker file is written.

#### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success; or live usage block (no `name::`) |
| 1 | Invalid chars in `name::`; `for::` missing `@`; or empty `for::` component |
| 2 | Account not found in credential store |

### Acceptance Criteria

- **AC-01**: `clp .account.assign name::alice@corp.com` writes `{credential_store}/_active_{hostname}_{user}` = `alice@corp.com`; exits 0; stdout contains `Assigned alice@corp.com for {user}@{machine}  →  _active_{machine}_{user}`.
- **AC-02**: `clp .account.assign name::alice@corp.com for::bob@laptop` writes `{credential_store}/_active_laptop_bob` = `alice@corp.com`; exits 0; `~/.claude/.credentials.json`, `~/.claude.json`, and `~/.claude/settings.json` are untouched.
- **AC-03**: `clp .account.assign name::alice@corp.com dry::1` exits 0; stdout is `[dry-run] would assign alice@corp.com for {user}@{machine}  →  _active_{machine}_{user}`; no files written.
- **AC-04**: `clp .account.assign` (no `name::`) exits 0 and emits a live usage block containing: current machine identity (`{user}@{machine}`), active account name from own marker (or `(none)` if absent), and copy-paste ready examples with the actual active account name substituted.
- **AC-05**: `clp .account.assign name::ghost@example.com` where the account does not exist in the credential store exits 2 with an actionable error message.
- **AC-06**: `clp .account.assign name::alice@corp.com for::badvalue` (no `@` in `for::` value) exits 1 with an error message explaining the `USER@MACHINE` format requirement.
- **AC-07**: `clp .account.assign name::alice@corp.com for::@laptop` or `for::bob@` (empty component on either side of `@`) exits 1.
- **AC-08**: Characters in `for::` user and machine components are sanitized per the same char-filter as `active_marker_filename()` — alphanumeric, `-`, `.` kept; others become `_`. Example: `for::alice@my laptop` writes `_active_my_laptop_alice`.
- **AC-09**: Prefix resolution: `clp .account.assign name::alice` where `alice` is a unique local-part prefix in the credential store resolves to the full account name and writes the marker for that account.
- **AC-10**: Overwriting an existing marker: if `_active_laptop_bob` already contains `old@corp.com`, `.account.assign name::new@corp.com for::bob@laptop` overwrites it; exits 0; file now contains `new@corp.com`.
- **AC-11**: The command does not invoke `switch_account()` — `~/.claude/.credentials.json` and `~/.claude.json` are left unchanged by a successful assign.
- **AC-12**: `clp .account.assign name::alice@corp.com for::bob@laptop dry::1` includes `_active_laptop_bob` in the dry-run stdout.
- **AC-13**: `.accounts active::USER@MACHINE` does NOT modify the `owner` field in `{name}.json` — marker-only write. The `active::` path in `accounts_routine()` does not call `write_owner()`. Ownership is released via `.accounts owner::0` and set via `.accounts owner::USER@MACHINE` (see [036_account_ownership.md](036_account_ownership.md)).

### Commands

| File | Relationship |
|------|--------------|
| [cli/command/001_account.md](../cli/command/001_account.md#command--16-accountassign) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [004_account_use.md](004_account_use.md) | Full credential rotation (contrast: `.account.assign` is marker-only) |
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | Marker filename derivation, `active_marker_filename()`, `resolve_hostname()` |
| [029_account_host_metadata.md](029_account_host_metadata.md) | `host::` on `.account.save` is a display label; `for::` on `.account.assign` is a marker target identity |
| [036_account_ownership.md](036_account_ownership.md) | Ownership model — ownership stamped by `.account.save`, not `.account.assign` |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/001_name.md](../cli/param/001_name.md) | `name::` — account identifier with prefix resolution |
| [cli/param/004_dry.md](../cli/param/004_dry.md) | `dry::` — dry-run flag |
| [cli/param/053_for.md](../cli/param/053_for.md) | `for::` — `USER@MACHINE` target identity |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | `.accounts assign::1` | CLI surface for this feature (`.account.assign` removed — see Feature 037) |

### Sources

| File | Relationship |
|------|--------------|
| `src/commands/accounts.rs` | `accounts_routine()` assign path — marker write only; does not call `write_owner()` |
| `claude_profile_core/src/account.rs` | `active_marker_filename()`, `resolve_hostname()` |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/account_assign_test.rs` | Integration tests for `.account.assign` |
| [tests/docs/feature/032_account_assign.md](../../tests/docs/feature/032_account_assign.md) | FT spec mapping ACs to test cases |

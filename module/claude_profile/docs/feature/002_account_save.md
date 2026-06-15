# Feature: Save Account

### Scope

- **Purpose**: Snapshot the current active credentials as a named account profile for later restoration.
- **Responsibility**: Documents the `account::save()` API and the `.account.save` CLI command (FR-7).
- **In Scope**: Credential copy, `oauthAccount` snapshot from `~/.claude.json`, org identity snapshot (`{name}.json` via endpoint 005), name validation, directory init, CLI dry-run behaviour, idempotent re-save as metadata refresh. Ownership stamp: `account_save_routine()` always passes `Some(&current_identity())` to `save()` — writes `current_identity()` as owner on every interactive save. Background refresh callers pass `owner: None`. To release ownership, use `.account.unclaim` (see Feature 036).
- **Out of Scope**: Account switching (→ 004_account_use.md), store initialization (→ 001_account_store_init.md).

### Design

`claude_profile` must copy `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json`, creating or overwriting the named entry. It also extracts the `oauthAccount` subtree from `~/.claude.json` and writes it into `{name}.json` — containing `{"oauthAccount": {...}}` alongside other per-account metadata — to preserve account identity for later enumeration by `.accounts`. Machine-global state (`commands.*`, `mcpServers`, `projects`) is never captured in the per-account snapshot. The credential store path is resolved per FR-6 (see `001_account_store_init.md`).

**Name resolution** — when `name::` is omitted, `oauthAccount.emailAddress` from `~/.claude.json` is the primary inference source (this field is updated by both clp account switches and external OAuth login). If absent or empty, the per-machine active marker file (`_active_{hostname}_{user}` — see Feature 025) is the fallback. If neither source provides a name, exits 1 with a clear message directing the user to pass `name::` explicitly.

**Name validation** — account names must be valid email addresses:
- Non-empty
- Must contain `@` with non-empty local part and domain

**Operation steps:**
1. Resolve name: use explicit `name::` if provided; otherwise read `oauthAccount.emailAddress` from `~/.claude.json` as primary inference source; if absent or empty, fall back to the per-machine active marker file (`{credential_store}/_active_{hostname}_{user}` via `active_marker_filename()` — see Feature 025); if neither source provides a non-empty name, exit 1.
2. Validate `name` against the rules above (exit 1 on violation).
3. Resolve credential store directory and ensure it exists (`create_dir_all` — see FR-6).
4. Read `~/.claude/.credentials.json`.
5. Write contents to `{credential_store}/{name}.credentials.json` (creates or overwrites).
6. Extract `oauthAccount` subtree from `~/.claude.json`. If `{credential_store}/{name}.json` already exists, read its current content and merge: update the `oauthAccount` key while preserving all other existing keys (notably `_renewal_at` written by `.account.renewal`). Write the merged result to `{credential_store}/{name}.json` (best-effort: skip silently if source absent or `oauthAccount` key missing; merge failure silently falls back to overwrite).
7. Capture model preference: read `model` field from `~/.claude/settings.json`; if present, write `{"model": "<value>"}` into `{credential_store}/{name}.json` (merged with existing content); if `~/.claude/settings.json` is absent or lacks a `model` field, skip silently — save still succeeds. This enables `switch_account()` to restore the per-account model preference on next switch.
8. Call `claude_quota::fetch_claude_cli_roles(&access_token)` (feature-gated, best-effort): on success, write response into `{credential_store}/{name}.json`; on any failure, skip silently — save still succeeds.
9. Write `{credential_store}/_active_{hostname}_{user}` = `{name}` via `active_marker_filename()` — mark the saved account as the current active account (per-machine marker, see feature 025) (when invoked from `.account.save` or `.account.relogin`; background refresh callers pass `update_marker=false` and do not write the marker).

**Idempotency as metadata refresh:** `save()` overwrites all snapshot data on every invocation. Re-running `clp .account.save` for an existing account name re-fetches endpoint 005 and overwrites org identity fields in `{name}.json` alongside all other metadata. This is the canonical mechanism for refreshing cached metadata when org membership or role changes — no separate command required.

**Model preference capture (BUG-222 fix):** `save()` reads the `model` field from `~/.claude/settings.json` and writes it into `{credential_store}/{name}.json`. This snapshot enables `switch_account()` to restore the per-account model preference on the next switch. If `~/.claude/settings.json` is absent or lacks a `model` field, no model field is written and save still succeeds.

**Ownership stamp:** `account_save_routine()` always passes `Some(&current_identity())` to `save()`, which writes `current_identity()` as the `owner` field in `{name}.json`. To release ownership, use `clp .account.unclaim name::EMAIL` — a dedicated command that calls `write_owner(name, store, "")` directly without touching credentials. Background refresh callers pass `owner: None` (preserves existing owner via read-merge). `.account.assign` is ownership-neutral — marker-only write; does NOT call `write_owner()`. See Feature 036.

**Dry-run mode** (`dry::1`): Print `[dry-run] would save current credentials as '{name}'` without modifying any files.

### Acceptance Criteria

- **AC-01**: `clp .account.save name::alice@acme.com` exits 0 and creates `{credential_store}/alice@acme.com.credentials.json`.
- **AC-02**: `clp .account.save name::` (empty) exits 1 with `account name must not be empty`.
- **AC-03**: `clp .account.save name::notanemail` exits 1 with `must be an email address`.
- **AC-04**: `clp .account.save name::alice@acme.com dry::1` prints `[dry-run] would save current credentials as 'alice@acme.com'` and creates no files.
- **AC-05**: When `~/.claude.json` contains an `oauthAccount` key, `{credential_store}/{name}.json` is created alongside the credential file, containing `{"oauthAccount": {...}}`.
- **AC-07**: When `~/.claude.json` is absent or lacks an `oauthAccount` key, no `oauthAccount` snapshot is written to `{name}.json` — save still succeeds.
- **AC-08**: `clp .account.save` (no `name::`) reads `oauthAccount.emailAddress` from `~/.claude.json` as the primary name source; if absent, falls back to the per-machine active marker file; saves normally with output `saved current credentials as '{name}'`.
- **AC-09**: `clp .account.save` (no `name::`) when both `oauthAccount.emailAddress` in `~/.claude.json` and the per-machine active marker file are absent or empty exits 1 with `cannot infer account name: no active account set — pass name:: explicitly`.
- **AC-10**: After a successful `clp .account.save`, `{credential_store}/_active_{hostname}_{user}` (the per-machine active marker) contains the saved account name; `clp .credentials.status` shows `Account: {name}` immediately.
- **AC-11**: `clp .account.save name::a/b@c.com` exits 1 — path-unsafe characters (`/`, `\`, `*`) in the email local part are rejected by `validate_name()` before any filesystem operation.
- **AC-12**: When endpoint 005 responds successfully, org identity fields are written to `{credential_store}/{name}.json` alongside the credential file.
- **AC-13**: When endpoint 005 fails (network error, scope issue, or feature not enabled), no org identity fields are written to `{name}.json` and save still exits 0.
- **AC-14**: Re-running `clp .account.save` for an existing account name overwrites org identity fields in `{name}.json` with a fresh endpoint 005 response; this is the metadata refresh mechanism.
- **AC-15**: Background refresh calls to `save()` (via `refresh_account_token`) pass `update_marker=false`; the per-machine active marker is not written; any concurrent `.account.use` switch is not disturbed.
- **AC-16**: `clp .account.save` (no `name::`) when `~/.claude.json` contains `oauthAccount.emailAddress = "i5@wbox.pro"` and the per-machine active marker contains `"i2@wbox.pro"` (stale from a prior clp session) saves credentials as `i5@wbox.pro`; `i2@wbox.pro.credentials.json` is NOT created or modified. (BUG-212 regression guard.)
- **AC-17**: When `{credential_store}/{name}.json` already exists and contains a `_renewal_at` key, re-running `clp .account.save` preserves the `_renewal_at` value in the updated snapshot (read-merge, not full overwrite); the `oauthAccount` key is updated and all other keys are retained.
- **AC-18**: `clp .account.save name::alice@acme.com` when `~/.claude/settings.json` contains `{"model": "sonnet"}` writes `{"model": "sonnet"}` into `{credential_store}/alice@acme.com.json`. When `~/.claude/settings.json` is absent or has no `model` field, no model field is written to `alice@acme.com.json` and exit code is still 0.
- **AC-19**: `clp .account.save name::alice@acme.com` stamps `current_identity()` as `owner` in `alice@acme.com.json` via `account_save_routine()` passing `Some(&current_identity())`. To release ownership, use `clp .account.unclaim name::EMAIL` — calls `write_owner(name, store, "")` directly without touching credentials. Background refresh callers pass `owner: None` (preserves existing `owner`). See Feature 036 AC-01/AC-02/AC-14.

### Bugs

| File | Relationship |
|------|--------------|
| `task/claude_profile/bug/222_switch_account_model_preference_not_restored.md` | BUG-222 ✅ Fixed (TSK-234): `save()` now writes model to `{name}.json` (step 7); `switch_account()` restores model on switch |

### Related Commands

| File | Relationship |
|------|--------------|
| [036_account_ownership.md](036_account_ownership.md) | ownership model — `.account.unclaim` clears ownership; G1–G8 enforcement gates |

### Commands

| File | Relationship |
|------|--------------|
| [command/001_account.md](../cli/command/001_account.md#command--4-accountsave) | CLI command specification |

### Features

| File | Relationship |
|------|--------------|
| [001_account_store_init.md](001_account_store_init.md) | Directory initialization triggered by save |
| [014_rich_account_metadata.md](014_rich_account_metadata.md) | Metadata fields snapshotted by `save()` |
| [022_org_identity_snapshot.md](022_org_identity_snapshot.md) | Org identity metadata lifecycle and org fields in `{name}.json` |
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | Per-machine marker naming convention used in step 8 |
| [030_account_renewal_override.md](030_account_renewal_override.md) | `_renewal_at` field written by `.account.renewal`; preserved by `save()` read-merge (AC-17) |
| [036_account_ownership.md](036_account_ownership.md) | Ownership model — `.account.save` stamps `current_identity()` as owner; `.account.unclaim` clears ownership; `.account.assign` is marker-only |
| [032_account_assign.md](032_account_assign.md) | `.account.assign` — marker-only write; does NOT call `write_owner()`; does NOT modify `owner` field in `{name}.json` |

### Referenced Commands

| # | Command | Role |
|---|---------|------|
| 1 | [`.account.save`](../cli/command/001_account.md#command--4-accountsave) | CLI surface for this feature |

### Sources

| File | Relationship |
|------|--------------|
| `src/account.rs` | `save()` implementation — validate, init dir, copy credentials |
| `src/commands/account_ops.rs` | `account_save_routine()` — CLI handler |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/accounts_test.rs` | Verifies credential file and metadata snapshots created with correct content |
| `claude_profile_core/tests/account_test.rs` | `as_save_writes_active_marker` — unit test: active marker written after `save()` |
| `tests/cli/credentials_test.rs` | `cred14` — CLI: `.credentials.status` shows `Account: {name}` after `.account.save` |
| `tests/cli/account_mutations_test.rs` | `as15` — name inferred from active marker (AC-08); `as16` — active marker written after save (AC-10); `as17`/`as18` — path-unsafe chars in local part exit 1 (AC-11); `mre_bug_209_account_save_uses_active_marker_not_stale_email` — BUG-209 regression: stale `emailAddress` ignored (AC-08); `mre_bug_212_account_save_stale_marker_uses_oauth_email` — BUG-212 regression: stale `_active` overridden by `oauthAccount.emailAddress` (AC-16) |
| [tests/docs/feature/002_account_save.md](../../tests/docs/feature/002_account_save.md) | FT-01…FT-13 test case planning for Feature 002 |

# Feature: Save Account

### Scope

- **Purpose**: Snapshot the current active credentials as a named account profile for later restoration.
- **Responsibility**: Documents the `account::save()` API and the `.account.save` CLI command (FR-7).
- **In Scope**: Credential copy, metadata snapshot (`~/.claude.json`, `settings.json`), name validation, directory init, CLI dry-run behaviour.
- **Out of Scope**: Account switching (→ 004_account_switch.md), store initialization (→ 001_account_store_init.md).

### Design

`claude_profile` must copy `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json`, creating or overwriting the named entry. It also snapshots `~/.claude.json` and `~/.claude/settings.json` as `{name}.claude.json` and `{name}.settings.json` to preserve rich account metadata for later enumeration by `.accounts`. The credential store path is resolved per FR-6 (see `001_account_store_init.md`).

**Name resolution** — when `name::` is omitted, the account name is inferred from `emailAddress` in `~/.claude.json`. If that field is absent, exits 1 with a clear message directing the user to pass `name::` explicitly.

**Name validation** — account names must be valid email addresses:
- Non-empty
- Must contain `@` with non-empty local part and domain

**Operation steps:**
1. Resolve name: use explicit `name::` if provided; otherwise read `emailAddress` from `~/.claude.json` via `parse_string_field`; if absent, exit 1.
2. Validate `name` against the rules above (exit 1 on violation).
3. Resolve credential store directory and ensure it exists (`create_dir_all` — see FR-6).
4. Read `~/.claude/.credentials.json`.
5. Write contents to `{credential_store}/{name}.credentials.json` (creates or overwrites).
6. Copy `~/.claude.json` → `{credential_store}/{name}.claude.json` (best-effort: skip silently if source absent).
7. Copy `~/.claude/settings.json` → `{credential_store}/{name}.settings.json` (best-effort: skip silently if source absent).
8. Write `{credential_store}/_active` = `{name}` — mark the saved account as the current active account.

**Dry-run mode** (`dry::1`): Print `[dry-run] would save current credentials as '{name}'` without modifying any files.

### Acceptance Criteria

- **AC-01**: `clp .account.save name::alice@acme.com` exits 0 and creates `{credential_store}/alice@acme.com.credentials.json`.
- **AC-02**: `clp .account.save name::` (empty) exits 1 with `account name must not be empty`.
- **AC-03**: `clp .account.save name::notanemail` exits 1 with `must be an email address`.
- **AC-04**: `clp .account.save name::alice@acme.com dry::1` prints `[dry-run] would save current credentials as 'alice@acme.com'` and creates no files.
- **AC-05**: When `~/.claude.json` exists, `{credential_store}/{name}.claude.json` is created alongside the credential file.
- **AC-06**: When `~/.claude/settings.json` exists, `{credential_store}/{name}.settings.json` is created alongside the credential file.
- **AC-07**: When `~/.claude.json` is absent, no `.claude.json` snapshot is created — save still succeeds.
- **AC-08**: `clp .account.save` (no `name::`) with `emailAddress` present in `~/.claude.json` infers the account name from that field and saves normally; output reads `saved current credentials as '{email}'`.
- **AC-09**: `clp .account.save` (no `name::`) when `~/.claude.json` has no `emailAddress` exits 1 with `cannot infer account name: emailAddress absent from ~/.claude.json — pass name:: explicitly`.
- **AC-10**: After a successful `clp .account.save`, `{credential_store}/_active` contains the saved account name; `clp .credentials.status` shows `Account: {name}` immediately.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `save()` implementation — validate, init dir, copy credentials |
| source | `src/commands.rs` | `account_save_routine()` — CLI handler |
| test | `tests/cli/accounts_test.rs` | Verifies credential file and metadata snapshots created with correct content |
| test | `claude_profile_core/tests/account_test.rs` | `as_save_writes_active_marker` — unit test: `_active` written after `save()` |
| test | `tests/cli/credentials_test.rs` | `cred14` — CLI: `.credentials.status` shows `Account: {name}` after `.account.save` |
| test | `tests/cli/account_mutations_test.rs` | `as16` — CLI: `_active` file contains saved name after `.account.save` |
| doc | [001_account_store_init.md](001_account_store_init.md) | Directory initialization triggered by save |
| doc | [cli/commands.md](../cli/commands.md#command--4-accountsave) | CLI command specification |
| doc | [014_rich_account_metadata.md](014_rich_account_metadata.md) | Metadata fields snapshotted by `save()` |

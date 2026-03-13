# Feature: Save Account

### Scope

- **Purpose**: Snapshot the current active credentials as a named account profile for later restoration.
- **Responsibility**: Documents the `account::save()` API and the `.account.save` CLI command (FR-7).
- **In Scope**: Credential copy, name validation, directory init, CLI dry-run behaviour.
- **Out of Scope**: Account switching (→ 004_account_switch.md), store initialization (→ 001_account_store_init.md).

### Design

`claude_profile` must copy `~/.claude/.credentials.json` to `~/.claude/accounts/{name}.credentials.json`, creating or overwriting the named entry.

**Name validation** — account names must be:
- Non-empty
- Free of filesystem-forbidden characters: `/\:*?"<>|` and null bytes

**Operation steps:**
1. Validate `name` against the rules above (exit 1 on violation).
2. Ensure `~/.claude/accounts/` exists (`create_dir_all` — see FR-6).
3. Read `~/.claude/.credentials.json`.
4. Write contents to `~/.claude/accounts/{name}.credentials.json` (creates or overwrites).

**Dry-run mode** (`dry::1`): Print `[dry-run] would save current credentials as '{name}'` without modifying any files.

### Acceptance Criteria

- **AC-01**: `clp .account.save name::work` exits 0 and creates `~/.claude/accounts/work.credentials.json`.
- **AC-02**: `clp .account.save name::` (empty) exits 1 with `account name must not be empty`.
- **AC-03**: `clp .account.save name::foo/bar` exits 1 with `contains invalid characters`.
- **AC-04**: `clp .account.save name::work dry::1` prints `[dry-run] would save current credentials as 'work'` and creates no files.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `save()` implementation — validate, init dir, copy credentials |
| source | `src/commands.rs` | `account_save_routine()` — CLI handler |
| test | `tests/account_tests.rs::save_copies_credentials_to_named_file` | Verifies credential file created with correct content |
| doc | [001_account_store_init.md](001_account_store_init.md) | Directory initialization triggered by save |
| doc | [cli/commands.md](../cli/commands.md#command--5-accountsave) | CLI command specification |

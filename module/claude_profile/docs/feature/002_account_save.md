# Feature: Save Account

### Scope

- **Purpose**: Snapshot the current active credentials as a named account profile for later restoration.
- **Responsibility**: Documents the `account::save()` API and the `.account.save` CLI command (FR-7).
- **In Scope**: Credential copy, name validation, directory init, CLI dry-run behaviour.
- **Out of Scope**: Account switching (→ 004_account_switch.md), store initialization (→ 001_account_store_init.md).

### Design

`claude_profile` must copy `~/.claude/.credentials.json` to `{credential_store}/{name}.credentials.json`, creating or overwriting the named entry. The credential store path is resolved per FR-6 (see `001_account_store_init.md`).

**Name validation** — account names must be valid email addresses:
- Non-empty
- Must contain `@` with non-empty local part and domain

**Operation steps:**
1. Validate `name` against the rules above (exit 1 on violation).
2. Resolve credential store directory and ensure it exists (`create_dir_all` — see FR-6).
3. Read `~/.claude/.credentials.json`.
4. Write contents to `{credential_store}/{name}.credentials.json` (creates or overwrites).

**Dry-run mode** (`dry::1`): Print `[dry-run] would save current credentials as '{name}'` without modifying any files.

### Acceptance Criteria

- **AC-01**: `clp .account.save name::alice@acme.com` exits 0 and creates `{credential_store}/alice@acme.com.credentials.json`.
- **AC-02**: `clp .account.save name::` (empty) exits 1 with `account name must not be empty`.
- **AC-03**: `clp .account.save name::notanemail` exits 1 with `must be an email address`.
- **AC-04**: `clp .account.save name::alice@acme.com dry::1` prints `[dry-run] would save current credentials as 'alice@acme.com'` and creates no files.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `save()` implementation — validate, init dir, copy credentials |
| source | `src/commands.rs` | `account_save_routine()` — CLI handler |
| test | `tests/account_tests.rs::save_copies_credentials_to_named_file` | Verifies credential file created with correct content |
| doc | [001_account_store_init.md](001_account_store_init.md) | Directory initialization triggered by save |
| doc | [cli/commands.md](../cli/commands.md#command--5-accountsave) | CLI command specification |

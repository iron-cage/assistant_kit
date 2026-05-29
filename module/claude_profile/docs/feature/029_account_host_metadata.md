# Feature: Account Host and Role Metadata

### Scope

- **Purpose**: Allow accounts to carry host and role labels that identify which machine and workspace context each account belongs to, displayed in `.usage` via opt-in columns.
- **Responsibility**: Documents the `host::` and `role::` parameters for `.account.save`, auto-capture of `$USER@$HOSTNAME` at save time, storage in the account profile, and the `cols::+host` / `cols::+role` display columns.
- **In Scope**: `host::` and `role::` params on `.account.save`, auto-capture from `$USER@$HOSTNAME` when `host::` is omitted, storage in `{name}.profile.json`, `host` and `role` columns in the `cols::` registry (off by default).
- **Out of Scope**: Account switching (→ 004_account_use.md), column visibility mechanism (→ 033_cols.md), `.usage` row filtering (→ 028_usage_row_filtering.md).

### Design

When `.account.save` runs, it captures host metadata for the account being saved:

- **`host::` param**: explicit machine/host label for this account. If omitted, auto-captured as `$USER@$HOSTNAME`. The auto-captured value records which user on which machine was active when the account was saved — useful for identifying where an account is primarily used.
- **`role::` param**: user-defined role label for this account (e.g., `work`, `personal`, `dev`, `staging`). If omitted, stored as empty string.

Both values are written to `{credential_store}/{name}.profile.json` as a JSON object:

```json
{ "host": "alice@workstation", "role": "work" }
```

This file is created or overwritten on every `save()` invocation (same idempotency semantics as other snapshot files). If `host::` is omitted and `$HOSTNAME` / `$USER` env vars are absent, the host field is stored as an empty string — save still succeeds.

**Display via `cols::`:** The `host` and `role` column IDs are off by default in the `cols::` registry. Enable via `cols::+host,+role` in `.usage`. The columns show the values from `{name}.profile.json` if present; empty string if the file is absent or the field is missing.

**`.accounts` display:** Host and role fields are also surfaced in `.accounts` output when `host::1` or `role::1` field toggles are active (separate opt-in toggle params, analogous to `uuid::`, `display_name::`, etc.).

### Acceptance Criteria

- **AC-01**: `clp .account.save host::mybox role::work` writes `{name}.profile.json` containing `{"host": "mybox", "role": "work"}` alongside the credential file.
- **AC-02**: `clp .account.save` (no `host::`) auto-captures `$USER@$HOSTNAME` as the host value in `{name}.profile.json`.
- **AC-03**: When `$USER` and `$HOSTNAME` are both unset, `clp .account.save` stores `host: ""` — save succeeds without error.
- **AC-04**: `clp .account.save host::newbox role::dev` on an existing account overwrites `{name}.profile.json` with the new values.
- **AC-05**: `clp .usage cols::+host` shows the `Host` column populated from `{name}.profile.json`; accounts with no profile file show an empty cell.
- **AC-06**: `clp .usage cols::+role` shows the `Role` column populated from `{name}.profile.json`; accounts with no profile file show an empty cell.
- **AC-07**: `clp .usage cols::+host,+role get::host` outputs the host label for the first row as a bare string (format::value).
- **AC-08**: `clp .accounts host::1 role::1` shows `Host:` and `Role:` fields in each account's output block.
- **AC-09**: `{name}.profile.json` absence does not cause any command to exit non-zero — the file is treated as optional metadata.
- **AC-10**: Re-running `clp .account.save` with `host::newbox` updates the host label without affecting credential or `.claude.json` snapshot files.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/account.rs` | `save()` — profile.json write; host/role capture |
| source | `src/commands/account_ops.rs` | `account_save_routine()` — host/role param extraction |
| source | `src/usage/render.rs` | host/role column rendering in quota table |
| param | [cli/param/048_host.md](../cli/param/048_host.md) | `host::` parameter specification for `.account.save` |
| param | [cli/param/033_cols.md](../cli/param/033_cols.md) | `cols::` registry — `host` and `role` column IDs |
| doc | [002_account_save.md](002_account_save.md) | Account save operation this feature extends |
| doc | [009_token_usage.md](009_token_usage.md) | Base `.usage` rendering that gains `host`/`role` columns |
| doc | [028_usage_row_filtering.md](028_usage_row_filtering.md) | `get::host` and `get::role` field extraction |

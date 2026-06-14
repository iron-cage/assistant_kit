# Feature: Account Host and Role Metadata

### Scope

- **Purpose**: Allow accounts to carry host and role labels that identify which machine and workspace context each account belongs to, displayed in `.usage` via opt-in columns.
- **Responsibility**: Documents the `host::` and `role::` parameters for `.account.save`, auto-capture of `$USER@<hostname>` at save time (hostname via syscall fallback chain), storage in the account profile, and the `cols::+host` / `cols::+role` display columns.
- **In Scope**: `host::` and `role::` params on `.account.save`, auto-capture from `$USER@<hostname>` when `host::` is omitted (hostname resolved via `resolve_hostname()` — same fallback chain as `active_marker_filename()`), storage in `{name}.json`, `host` and `role` columns in the `cols::` registry (off by default).
- **Out of Scope**: Account switching (→ 004_account_use.md), column visibility mechanism (→ 033_cols.md), `.usage` row filtering (→ 028_usage_row_filtering.md), account ownership and access enforcement (→ 036_account_ownership.md — `owner` field is separate from `host` display label).

### Design

When `.account.save` runs, it captures host metadata for the account being saved:

- **`host::` param**: explicit machine/host label for this account. If omitted, auto-captured as `$USER@<hostname>` where hostname is resolved via `resolve_hostname()`: `$HOSTNAME` env var → `/etc/hostname` file → `"local"` fallback (same fallback chain as `active_marker_filename()` in Feature 025). The auto-captured value records which user on which machine was active when the account was saved — useful for identifying where an account is primarily used.
- **`role::` param**: user-defined role label for this account (e.g., `work`, `personal`, `dev`, `staging`). If omitted, stored as empty string.

Both values are written to `{credential_store}/{name}.json` as a JSON object:

```json
{ "host": "alice@workstation", "role": "work" }
```

This file is created or overwritten on every `save()` invocation (same idempotency semantics as other snapshot files). If `host::` is omitted and all hostname fallbacks resolve (env, file, `"local"` default), the host field is always populated. If `$USER` is also unset, the host field is stored as `"@<hostname>"` — save still succeeds.

**Display via `cols::`:** The `host` and `role` column IDs are off by default in the `cols::` registry. Enable via `cols::+host,+role` in `.usage`. The columns show the values from `{name}.json` if present; empty string if the file is absent or the field is missing.

**`host::` vs `owner`:** The `host::` value is a user-customizable display label that can be any string (e.g., `"workstation"`, `"laptop"`). The `owner` field (Feature 036) is always auto-captured as `$USER@<hostname>` at save time and is never user-specified via CLI parameter. These are separate fields with different semantics: `host::` is for display/identification; `owner` is for access enforcement.

**`.accounts` display:** Host and role fields are also surfaced in `.accounts` output when `host::1` or `role::1` field toggles are active (separate opt-in toggle params, analogous to `uuid::`, `display_name::`, etc.).

### Acceptance Criteria

- **AC-01**: `clp .account.save host::mybox role::work` writes `{name}.json` containing `{"host": "mybox", "role": "work"}` alongside the credential file.
- **AC-02**: `clp .account.save` (no `host::`) auto-captures `$USER@<hostname>` as the host value in `{name}.json`, where hostname is resolved via `resolve_hostname()` (`$HOSTNAME` → `/etc/hostname` → `"local"`).
- **AC-03**: When `$USER` is unset, `clp .account.save` stores `host: "@<hostname>"` (hostname always resolves via fallback chain) — save succeeds without error.
- **AC-04**: `clp .account.save host::newbox role::dev` on an existing account overwrites `{name}.json` with the new values.
- **AC-05**: `clp .usage cols::+host` shows the `Host` column populated from `{name}.json`; accounts with no profile file show an empty cell.
- **AC-06**: `clp .usage cols::+role` shows the `Role` column populated from `{name}.json`; accounts with no profile file show an empty cell.
- **AC-07**: `clp .usage cols::+host,+role get::host` outputs the host label for the first row as a bare string (format::value).
- **AC-08**: `clp .accounts host::1 role::1` shows `Host:` and `Role:` fields in each account's output block.
- **AC-09**: `{name}.json` absence does not cause any command to exit non-zero — the file is treated as optional metadata.
- **AC-10**: Re-running `clp .account.save` with `host::newbox` updates the host label in `{name}.json` without affecting credential files.

### Bugs

| File | Relationship |
|------|--------------|
| `task/claude_profile/bug/239_account_save_hostname_empty_env_var.md` | BUG-239 ✅ Fixed: `resolve_hostname()` fallback chain (`$HOSTNAME` → `/etc/hostname` → `"local"`) extracted and shared with `active_marker_filename()` |

### Features

| File | Relationship |
|------|--------------|
| [002_account_save.md](002_account_save.md) | Account save operation this feature extends |
| [009_token_usage.md](009_token_usage.md) | Base `.usage` rendering that gains `host`/`role` columns |
| [025_per_machine_active_marker.md](025_per_machine_active_marker.md) | `resolve_hostname()` fallback chain shared with `active_marker_filename()` |
| [028_usage_row_filtering.md](028_usage_row_filtering.md) | `get::host` and `get::role` field extraction |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/033_cols.md](../cli/param/033_cols.md) | `cols::` registry — `host` and `role` column IDs |
| [cli/param/048_host.md](../cli/param/048_host.md) | `host::` parameter specification for `.account.save` |
| [036_account_ownership.md](036_account_ownership.md) | `owner` field — access enforcement; separate from `host::` display label |

### Sources

| File | Relationship |
|------|--------------|
| `src/account.rs` | `save()` — `{name}.json` host/role write; host/role capture |
| `src/commands/account_ops.rs` | `account_save_routine()` — host/role param extraction |
| `src/usage/render.rs` | host/role column rendering in quota table |

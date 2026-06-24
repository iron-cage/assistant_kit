# Feature: Org Identity Snapshot

### Scope

- **Purpose**: Persist org identity fields from endpoint 005 (`GET /api/oauth/claude_cli/roles`) into `{name}.json` at save-time so that `organization_uuid`, `organization_name`, and workspace fields are available without a live API call.
- **Responsibility**: Documents the org identity metadata lifecycle (write on save, delete on delete), the five org fields added to the `Account` struct, the `org_uuid::` and `org_name::` params, and the endpoint 005 transport in `claude_quota` (FR-22).
- **In Scope**: `fetch_claude_cli_roles()` transport in `claude_quota`; org identity metadata in `{name}.json` created by `save()`, deleted by `delete()`; org fields on `Account`; `org_uuid::` and `org_name::` params on `.credentials.status` and `.accounts`; JSON output; `.account.save` idempotency as the metadata refresh mechanism.
- **Out of Scope**: Live org identity lookup (no on-demand API calls from display commands); workspace-level params beyond `org_uuid::` and `org_name::` (workspace fields stored but not yet exposed via params); `workspace_role` field from API response (present in `{name}.json` raw JSON but not added to `Account` struct — no display use case; deferred per YAGNI); mutations to org membership; user identity fields from `{name}.json` (→ 021_extended_snapshot_fields.md).

### Design

#### Org identity metadata

`save()` calls `claude_quota::fetch_claude_cli_roles(&access_token)` (feature-gated under `dep:claude_quota`) and writes the response into `{credential_store}/{name}.json`. This is best-effort: if the fetch fails (network error, scope insufficient, token expired), org identity metadata is not written and `save()` still succeeds. Existing org identity fields in `{name}.json` are overwritten on re-save.

`delete()` removes `{credential_store}/{name}.json` best-effort. Missing file is silently skipped.

`switch_account()` patches `organizationName` and `organizationUuid` inside `oauthAccount` from the saved org identity metadata in `{name}.json` when switching accounts (BUG-219 fix). This is best-effort: silently skipped when `{name}.json` is absent or org fields are empty. The org identity metadata in `{name}.json` is read directly from the credential store by `list()`; it does not need to be restored to any `~/.claude/` path.

**Metadata refresh via `.account.save` idempotency:** Because `save()` overwrites all snapshot files on every invocation, re-running `.account.save` (with the same name or inferring the name from `~/.claude.json`) acts as a full metadata refresh — re-fetches endpoint 005 and overwrites org identity fields in `{name}.json` alongside all other snapshots. No separate refresh command is needed.

#### Org identity fields

Written as the raw JSON response from endpoint 005 into `{name}.json`:

```json
{
  "organization_uuid": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
  "organization_name": "alice@example.com's Organization",
  "organization_role": "admin",
  "workspace_uuid": null,
  "workspace_name": null,
  "workspace_role": null
}
```

Personal accounts have `null` workspace fields. Enterprise accounts have non-null workspace fields.

#### New `Account` fields

| Field | Source | `Account` field | Semantics |
|-------|--------|----------------|-----------|
| `organization_uuid` | `{name}.json` | `organization_uuid: String` | Org UUID |
| `organization_name` | `{name}.json` | `organization_name: String` | Org display name |
| `organization_role` | `{name}.json` | `org_role: String` | User's role in org (struct field renamed by TSK-324; JSON key `"organization_role"` retained) |
| `workspace_uuid` | `{name}.json` | `workspace_uuid: String` | Workspace UUID (empty for personal) |
| `workspace_name` | `{name}.json` | `workspace_name: String` | Workspace name (empty for personal) |

All fields empty string when `{name}.json` absent or field missing.

#### `fetch_claude_cli_roles()` transport

New function in `claude_quota::lib`:

```
fetch_claude_cli_roles(token: &str) -> Result<ClaudeCliRolesData, ...>
```

Issues `GET https://api.anthropic.com/api/oauth/claude_cli/roles` with the Bearer token. Parses the JSON response into `ClaudeCliRolesData`. See endpoint contract in `contract/claude_code/docs/endpoint/005_claude_cli_roles.md`.

#### New field-presence params (opt-in, default `0`)

| Param | Default | Source | Output Line |
|-------|---------|--------|-------------|
| `org_uuid::` | `0` | `{name}.json` → `organization_uuid` | `Org ID:   {uuid_or_N/A}` |
| `org_name::` | `0` | `{name}.json` → `organization_name` | `Org:      {name_or_N/A}` |

Applied to `.credentials.status` (reads from live `~/.claude.json` — org fields only available if `{name}.json` was saved for the active account and is re-read from the credential store) and `.accounts` (reads from `{name}.json` snapshot).

**`.credentials.status` note:** `.credentials.status` reads live credential data without requiring the credential store. For `org_uuid::` and `org_name::`, it reads from `{credential_store}/{active_account}.json` using the active account name from the per-machine active marker. If no active account or no `{name}.json` — shows `N/A`.

**`format::json`:** Always includes `organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`, `workspace_name` regardless of params.

### Acceptance Criteria

- **AC-01**: `clp .account.save` writes org identity metadata to `{name}.json` in the credential store when endpoint 005 returns a valid response.
- **AC-02**: If endpoint 005 call fails during `save()`, org identity metadata is not written to `{name}.json`; `save()` still exits 0.
- **AC-03**: Re-running `clp .account.save` (same name) overwrites org identity fields in `{name}.json` with fresh data — this is the metadata refresh mechanism.
- **AC-04**: `clp .account.delete name::alice@acme.com` removes `{credential_store}/alice@acme.com.json` if it exists; absent file causes no error.
- **AC-05**: `clp .accounts cols::+org_uuid` shows `Org ID:` line per account from saved `{name}.json`; `N/A` when absent.
- **AC-06**: `clp .accounts cols::+org_name` shows `Org:` line per account from saved `{name}.json`; `N/A` when absent.
- **AC-07**: `clp .credentials.status org_uuid::1` shows `Org ID:` from the active account's `{name}.json`; `N/A` when absent.
- **AC-08**: `clp .credentials.status org_name::1` shows `Org:` from the active account's `{name}.json`; `N/A` when absent.
- **AC-09**: `format::json` always includes `organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`, `workspace_name` on both commands.
- **AC-10**: `fetch_claude_cli_roles()` is feature-gated; `claude_profile_core` with `default-features = false` does not require `claude_quota` dep.
- **AC-11**: Personal accounts (null workspace fields in API response) → `workspace_uuid`, `workspace_name` are stored as empty string in `Account`, shown as `N/A` in text output, and included as `""` in JSON output (per AC-09).

### Contracts

| File | Relationship |
|------|--------------|
| `contract/claude_code/docs/endpoint/005_claude_cli_roles.md` | Endpoint 005 wire contract |

### Features

| File | Relationship |
|------|--------------|
| [003_account_list.md](003_account_list.md) | `.accounts` — `cols::+org_uuid`/`cols::+org_name` opt-in columns and org fields in `format::json` |
| [002_account_save.md](002_account_save.md) | `.account.save` — org identity written to `{name}.json` and idempotency |
| [005_account_delete.md](005_account_delete.md) | `.account.delete` — `{name}.json` removed best-effort |
| [014_rich_account_metadata.md](014_rich_account_metadata.md) | Base rich metadata feature (FR-20); this feature extends it |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/030_org_uuid.md](../cli/param/030_org_uuid.md) | `org_uuid::` param specification |
| [cli/param/031_org_name.md](../cli/param/031_org_name.md) | `org_name::` param specification |
| [cli/param_group/002_field_presence.md](../cli/param_group/002_field_presence.md) | Field presence group — `org_uuid::` and `org_name::` are members |

### Sources

| File | Relationship |
|------|--------------|
| `claude_profile_core/src/account.rs` | `Account` struct org fields; `save()` writes org identity to `{name}.json` (feature-gated); `delete()` removes `{name}.json` best-effort; `list()` reads org identity from `{name}.json` |
| `claude_quota/src/lib.rs` | `fetch_claude_cli_roles()` transport; `ClaudeCliRolesData` struct |
| `src/commands/credentials.rs`, `src/commands/accounts.rs` | `credentials_status_routine()` — reads active account org identity from `{name}.json`; `accounts_routine()` — renders org fields |
| `src/lib.rs` | Registration of `org_uuid::` and `org_name::` params |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/accounts_test.rs` | org field rendering from `{name}.json` snapshot |
| `tests/cli/account_mutations_test.rs` | org identity in `{name}.json` created on save; removed on delete |
| `tests/cli/credentials_test.rs` | `org_uuid::` and `org_name::` on `.credentials.status` |

### Schema

| File | Relationship |
|------|-------------|
| [schema/002_account_json.md](../schema/002_account_json.md) | Unified `{name}.json` field table — `org_uuid`, `org_name` rows owned by this feature |

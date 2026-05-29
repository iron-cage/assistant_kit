# Feature: Org Identity Snapshot

### Scope

- **Purpose**: Persist org identity fields from endpoint 005 (`GET /api/oauth/claude_cli/roles`) as `{name}.roles.json` at save-time so that `organization_uuid`, `organization_name`, and workspace fields are available without a live API call.
- **Responsibility**: Documents the `{name}.roles.json` snapshot lifecycle (write on save, delete on delete), the five org fields added to the `Account` struct, the `org_uuid::` and `org_name::` params, and the endpoint 005 transport in `claude_quota` (FR-22).
- **In Scope**: `fetch_claude_cli_roles()` transport in `claude_quota`; `{name}.roles.json` created by `save()`, deleted by `delete()`; org fields on `Account`; `org_uuid::` and `org_name::` params on `.credentials.status` and `.accounts`; JSON output; `.account.save` idempotency as the metadata refresh mechanism.
- **Out of Scope**: Live org identity lookup (no on-demand API calls from display commands); workspace-level params beyond `org_uuid::` and `org_name::` (workspace fields stored but not yet exposed via params); `workspace_role` field from API response (present in `{name}.roles.json` raw JSON but not added to `Account` struct — no display use case; deferred per YAGNI); mutations to org membership; user identity fields from `{name}.claude.json` (→ 021_extended_snapshot_fields.md).

### Design

#### `{name}.roles.json` lifecycle

`save()` calls `claude_quota::fetch_claude_cli_roles(&access_token)` (feature-gated under `dep:claude_quota`) and writes the response to `{credential_store}/{name}.roles.json`. This is best-effort: if the fetch fails (network error, scope insufficient, token expired), the roles.json is not written and `save()` still succeeds. Existing `{name}.roles.json` is overwritten on re-save.

`delete()` removes `{credential_store}/{name}.roles.json` best-effort alongside `{name}.claude.json` and `{name}.settings.json`. Missing roles.json is silently skipped.

`switch_account()` requires no change — `{name}.roles.json` is read directly from the credential store by `list()`; it does not need to be restored to any `~/.claude/` path.

**Metadata refresh via `.account.save` idempotency:** Because `save()` overwrites all snapshot files on every invocation, re-running `.account.save` (with the same name or inferring the name from `~/.claude.json`) acts as a full metadata refresh — re-fetches endpoint 005 and overwrites `{name}.roles.json` alongside all other snapshots. No separate refresh command is needed.

#### `{name}.roles.json` structure

Written as the raw JSON response from endpoint 005:

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
| `organization_uuid` | `{name}.roles.json` | `organization_uuid: String` | Org UUID |
| `organization_name` | `{name}.roles.json` | `organization_name: String` | Org display name |
| `organization_role` | `{name}.roles.json` | `organization_role: String` | User's role in org |
| `workspace_uuid` | `{name}.roles.json` | `workspace_uuid: String` | Workspace UUID (empty for personal) |
| `workspace_name` | `{name}.roles.json` | `workspace_name: String` | Workspace name (empty for personal) |

All fields empty string when `{name}.roles.json` absent or field missing.

#### `fetch_claude_cli_roles()` transport

New function in `claude_quota::lib`:

```
fetch_claude_cli_roles(token: &str) -> Result<ClaudeCliRolesData, ...>
```

Issues `GET https://api.anthropic.com/api/oauth/claude_cli/roles` with the Bearer token. Parses the JSON response into `ClaudeCliRolesData`. See endpoint contract in `contract/claude_code/docs/endpoint/005_claude_cli_roles.md`.

#### New field-presence params (opt-in, default `0`)

| Param | Default | Source | Output Line |
|-------|---------|--------|-------------|
| `org_uuid::` | `0` | `{name}.roles.json` → `organization_uuid` | `Org ID:   {uuid_or_N/A}` |
| `org_name::` | `0` | `{name}.roles.json` → `organization_name` | `Org:      {name_or_N/A}` |

Applied to `.credentials.status` (reads from live `~/.claude.json` — org fields only available if `{name}.roles.json` was saved for the active account and is re-read from the credential store) and `.accounts` (reads from `{name}.roles.json` snapshot).

**`.credentials.status` note:** `.credentials.status` reads live credential data without requiring the credential store. For `org_uuid::` and `org_name::`, it reads from `{credential_store}/{active_account}.roles.json` using the active account name from the per-machine active marker. If no active account or no roles.json — shows `N/A`.

**`format::json`:** Always includes `organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`, `workspace_name` regardless of params.

### Acceptance Criteria

- **AC-01**: `clp .account.save` writes `{name}.roles.json` to the credential store when endpoint 005 returns a valid response.
- **AC-02**: If endpoint 005 call fails during `save()`, `{name}.roles.json` is not written; `save()` still exits 0.
- **AC-03**: Re-running `clp .account.save` (same name) overwrites `{name}.roles.json` with fresh data — this is the metadata refresh mechanism.
- **AC-04**: `clp .account.delete name::alice@acme.com` removes `{credential_store}/alice@acme.com.roles.json` if it exists; absent file causes no error.
- **AC-05**: `clp .accounts org_uuid::1` shows `Org ID:` line per account from saved `{name}.roles.json`; `N/A` when roles.json absent.
- **AC-06**: `clp .accounts org_name::1` shows `Org:` line per account from saved `{name}.roles.json`; `N/A` when roles.json absent.
- **AC-07**: `clp .credentials.status org_uuid::1` shows `Org ID:` from the active account's roles.json; `N/A` when absent.
- **AC-08**: `clp .credentials.status org_name::1` shows `Org:` from the active account's roles.json; `N/A` when absent.
- **AC-09**: `format::json` always includes `organization_uuid`, `organization_name`, `organization_role`, `workspace_uuid`, `workspace_name` on both commands.
- **AC-10**: `fetch_claude_cli_roles()` is feature-gated; `claude_profile_core` with `default-features = false` does not require `claude_quota` dep.
- **AC-11**: Personal accounts (null workspace fields in API response) → `workspace_uuid`, `workspace_name` are stored as empty string in `Account`, shown as `N/A` in text output, and included as `""` in JSON output (per AC-09).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `claude_profile_core/src/account.rs` | `Account` struct org fields; `save()` writes roles.json (feature-gated); `delete()` removes roles.json best-effort; `list()` reads roles.json |
| source | `claude_quota/src/lib.rs` | `fetch_claude_cli_roles()` transport; `ClaudeCliRolesData` struct |
| source | `src/commands/credentials.rs`, `src/commands/accounts.rs` | `credentials_status_routine()` — reads active account roles.json; `accounts_routine()` — renders org fields |
| source | `src/lib.rs` | Registration of `org_uuid::` and `org_name::` params |
| test | `tests/cli/accounts_test.rs` | org field rendering from roles.json snapshot |
| test | `tests/cli/account_mutations_test.rs` | roles.json created on save; removed on delete |
| test | `tests/cli/credentials_test.rs` | `org_uuid::` and `org_name::` on `.credentials.status` |
| doc | [002_account_save.md](002_account_save.md) | `.account.save` — `{name}.roles.json` written and idempotency |
| doc | [005_account_delete.md](005_account_delete.md) | `.account.delete` — `{name}.roles.json` removed best-effort |
| doc | [014_rich_account_metadata.md](014_rich_account_metadata.md) | Base rich metadata feature (FR-20); this feature extends it |
| doc | [cli/param/030_org_uuid.md](../cli/param/030_org_uuid.md) | `org_uuid::` param specification |
| doc | [cli/param/031_org_name.md](../cli/param/031_org_name.md) | `org_name::` param specification |
| doc | `contract/claude_code/docs/endpoint/005_claude_cli_roles.md` | Endpoint 005 wire contract |
| doc | [cli/param_group/002_field_presence.md](../cli/param_group/002_field_presence.md) | Field presence group — `org_uuid::` and `org_name::` are members |

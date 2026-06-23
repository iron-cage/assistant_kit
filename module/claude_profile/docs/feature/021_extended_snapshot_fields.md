# Feature: Extended Snapshot Fields

### Scope

- **Purpose**: Expose additional user identity and capabilities fields already present in `{name}.json` snapshots through new opt-in field-presence params on `.credentials.status` and `.accounts`.
- **Responsibility**: Documents the `tagged_id`, `uuid`, and `capabilities` fields added to the `Account` struct and the `uuid::` and `capabilities::` params (FR-21).
- **In Scope**: Reading `taggedId`, `uuid`, `capabilities` from `{name}.json` via `account::list()`; reading same fields from live `~/.claude.json` via `read_live_cred_meta()`; opt-in field-presence params `uuid::` and `capabilities::` on `.credentials.status` and `.accounts`; JSON output always including these fields; `parse_string_array_field` helper for array extraction.
- **Out of Scope**: Mutations to `~/.claude.json` (read-only); OAuth API calls; org identity from endpoint 005 (→ 022_org_identity_snapshot.md).

### Design

The `{name}.json` snapshot (a copy of `~/.claude.json` taken at `save()` time) contains the full `oauthAccount` object written by Claude Code. The current `Account` struct reads four fields from it: `emailAddress`, `displayName`, `organizationRole`, `billingType`. Three additional fields are present in the same object and require no new API calls — see [schema/002_account_json.md](../schema/002_account_json.md) for the complete field table (`tagged_id`, `uuid`, `capabilities` rows).

**Extraction:** `tagged_id` and `uuid` use the existing `parse_string_field()` helper. `capabilities` is a JSON array — a new `parse_string_array_field(json, key) -> Vec<String>` helper is required. This helper scans for `"key":[` and parses quoted strings until `]`.

**Snapshot vs. live:** The snapshot in the credential store (`{name}.json`) is used for `.accounts`. The live `~/.claude.json` is used for `.credentials.status` via `read_live_cred_meta()`. Both paths read the same three keys with the same helpers.

**New field-presence params (all opt-in, default `0`):**

| Param | Default | Source | Output Line |
|-------|---------|--------|-------------|
| `uuid::` | `0` | `{name}.json` → `oauthAccount.taggedId` | `ID:           {tagged_id_or_N/A}` |
| `capabilities::` | `0` | `{name}.json` → `oauthAccount.capabilities[]` | `Capabilities: {cap1, cap2_or_N/A}` |

**Output format:** `capabilities` renders as comma-separated values (e.g. `max, chat`). Empty `Vec` → `N/A`.

**`format::json`:** `format::json` always includes `tagged_id` (string or `""`) and `capabilities` (array or `[]`) regardless of field-presence params. The `uuid` field (UUID form of user ID) is stored in `Account` for completeness but is not exposed in text output or JSON — text output shows `tagged_id` via the `uuid::` param, and JSON output serializes `tagged_id`, not raw `uuid`.

**Missing fields:** All new fields show `N/A` in text and empty string / empty array in JSON when absent from the snapshot. Never error on absent fields.

### Acceptance Criteria

- **AC-01**: `clp .credentials.status uuid::1` shows `ID: {tagged_id}` or `ID: N/A`.
- **AC-02**: `clp .credentials.status capabilities::1` shows `Capabilities: {list}` or `Capabilities: N/A`.
- **AC-03**: `clp .accounts uuid::1` shows `ID:` line per account from saved `{name}.json`.
- **AC-04**: `clp .accounts capabilities::1` shows `Capabilities:` line per account from saved `{name}.json`.
- **AC-05**: Both params default to `0` — absent from default output.
- **AC-06**: `format::json` on both commands always includes `tagged_id` (string) and `capabilities` (array) keys.
- **AC-07**: Absent snapshot → `N/A` / `[]` for all three fields without error.
- **AC-08**: `parse_string_array_field` correctly extracts `["claude_max","chat"]` → `vec!["claude_max", "chat"]`.
- **AC-09**: `capabilities` with empty array `[]` in snapshot → `N/A` in text output, `[]` in JSON.

### Features

| File | Relationship |
|------|--------------|
| [003_account_list.md](003_account_list.md) | `.accounts` command |
| [014_rich_account_metadata.md](014_rich_account_metadata.md) | Base rich metadata feature (FR-20); `uuid::` and `capabilities::` extend it |

### Parameters

| File | Relationship |
|------|--------------|
| [cli/param/028_uuid.md](../cli/param/028_uuid.md) | `uuid::` param specification |
| [cli/param/029_capabilities.md](../cli/param/029_capabilities.md) | `capabilities::` param specification |
| [cli/param_group/002_field_presence.md](../cli/param_group/002_field_presence.md) | Field presence group — `uuid::` and `capabilities::` are members |

### Sources

| File | Relationship |
|------|--------------|
| `claude_profile_core/src/account.rs` | `Account` struct new fields; `list()` reads them; `parse_string_array_field` helper |
| `src/commands/accounts.rs`, `src/commands/credentials.rs` | `read_live_cred_meta()` — reads new fields from live `~/.claude.json`; `accounts_routine()`, `credentials_status_routine()` — render params |
| `src/lib.rs` | Registration of `uuid::` and `capabilities::` params |

### Tests

| File | Relationship |
|------|--------------|
| `tests/cli/credentials_test.rs` | Test cases for `uuid::` and `capabilities::` on `.credentials.status` |
| `tests/cli/accounts_test.rs` | Test cases for `uuid::` and `capabilities::` on `.accounts` |

### Schema

| File | Relationship |
|------|-------------|
| [schema/002_account_json.md](../schema/002_account_json.md) | Unified `{name}.json` field table — `tagged_id`, `uuid`, `capabilities` rows owned by this feature |
| [schema/007_claude_json.md](../schema/007_claude_json.md) | `~/.claude.json` fields read (`oauthAccount.id`, `primaryEmailAddress`, `capabilities`) |

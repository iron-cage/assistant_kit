# Schema: Account Metadata — `{name}.json`

### Scope

- **Purpose**: Unified field table for the per-account supplementary metadata file stored alongside `{name}.credentials.json`.
- **Responsibility**: Documents all fields in the per-account supplementary metadata file `{name}.json` across all features.
- **In Scope**: All fields written or read by `clp` across all features — core identity, OAuth metadata, org identity, extended snapshot fields, host/role labels, renewal override, ownership, quota cache, and measurement history.
- **Out of Scope**: `{name}.credentials.json` (→ [schema/001](001_credentials_json.md)); HTTP API response shapes.

### File Location

```
{credential_store}/{name}.json
```

### Format

2-space pretty-printed JSON, trailing newline. See [invariant/007](../invariant/007_json_storage_format.md).

**Write semantics:** `account::save()` performs a read-merge — existing fields are preserved and only updated when the save operation explicitly provides a new value. Fields introduced by one feature are never clobbered by a save that doesn't know about them.

### Field Table

| Field | Type | Source | Written by | Read by | Feature |
|-------|------|--------|-----------|---------|---------|
| `oauthAccount` | object | `~/.claude.json` → `oauthAccount` subtree | `save()` at save time | `list()` for `.accounts`/`.usage` display | [014](../feature/014_rich_account_metadata.md) |
| `oauthAccount.displayName` | string | `~/.claude.json` | `save()` | `list()` → `display_name` field | [014](../feature/014_rich_account_metadata.md) |
| `oauthAccount.organizationRole` | string | `~/.claude.json` | `save()` | (not read by `list()` — `Account.role` reading `organizationRole` dropped by TSK-324; blob preserved in `{name}.json` via `oauthAccount` subtree) | [014](../feature/014_rich_account_metadata.md) |
| `oauthAccount.billingType` | string | `~/.claude.json` | `save()` | `list()` → `billing` field | [014](../feature/014_rich_account_metadata.md) |
| `model` | string | `~/.claude/settings.json` → `model` | `save()` at save time; `switch_account()` restores on switch | `list()` → `model` field; `switch_account()` restores model to settings.json | [014](../feature/014_rich_account_metadata.md) |
| `tagged_id` | string | `~/.claude.json` → `oauthAccount.primaryEmailAddress` or email | `save()` | `list()` → `tagged_id` field | [021](../feature/021_extended_snapshot_fields.md) |
| `uuid` | string | `~/.claude.json` → `oauthAccount.id` | `save()` | `list()` → `uuid` field (opt-in `uuid::1`) | [021](../feature/021_extended_snapshot_fields.md) |
| `capabilities` | array of strings | `~/.claude.json` → `oauthAccount.capabilities` | `save()` | `list()` → `capabilities` field (opt-in `capabilities::1`) | [021](../feature/021_extended_snapshot_fields.md) |
| `organization_uuid` | string | Endpoint 005 at save time | `save()` | `list()` → `organization_uuid` field (opt-in `org_uuid::1`) | [022](../feature/022_org_identity_snapshot.md) |
| `organization_name` | string | Endpoint 005 at save time | `save()` | `list()` → `organization_name` field (opt-in `org_name::1`) | [022](../feature/022_org_identity_snapshot.md) |
| `organization_role` | string | Endpoint 005 at save time | `save()` | `list()` → `Account.org_role` field; `format::json` as `"organization_role"` key | [022](../feature/022_org_identity_snapshot.md), [003](../feature/003_account_list.md) |
| `workspace_uuid` | string | Endpoint 005 at save time | `save()` | `list()` → `Account.workspace_uuid`; `format::json` as `"workspace_uuid"` key; empty for personal accounts | [022](../feature/022_org_identity_snapshot.md) |
| `workspace_name` | string | Endpoint 005 at save time | `save()` | `list()` → `Account.workspace_name`; `format::json` as `"workspace_name"` key; empty for personal accounts | [022](../feature/022_org_identity_snapshot.md) |
| `host` | string | `$HOSTNAME`/`/etc/hostname`/`"local"` | `save()` when `host::` param given or auto-captured | `list()` → `host` field (opt-in `cols::+host`) | [029](../feature/029_account_host_metadata.md) |
| `role` | string | `role::` CLI param at save time | `save()` when `role::` param given | `list()` → `role` metadata label (opt-in `cols::+role`) | [029](../feature/029_account_host_metadata.md) |
| `_renewal_at` | string (ISO 8601) | `at::` or `from_now::` CLI param | `.account.renewal` command | `list()` → `~Renews` / `→ Next` columns | [030](../feature/030_account_renewal_override.md) |
| `owner` | string | `owner::` CLI param or `current_identity()` | `.accounts owner::`, `.account.assign` (removed in F037) | `list()` → ownership gate checks; `current_identity()` comparison | [036](../feature/036_account_ownership.md), [063](../feature/063_explicit_ownership_claim.md) |
| `cache` | object | Live API response via `write_quota_cache()` | `apply_touch()` after touch; `apply_refresh()` after refresh; `fetch.rs` on successful fetch | `fetch.rs` cache fallback; `approximate_quota()` | [033](../feature/033_quota_cache.md) |
| `cache.fetched_at` | string (ISO 8601) | Current time at cache write | `write_quota_cache()` | Cache age staleness indicator | [033](../feature/033_quota_cache.md) |
| `cache.status` | string (always `"ok"`) | Hardcoded literal at cache write time | `write_quota_cache()` | Not read by any consumer — write-only | [033](../feature/033_quota_cache.md) |
| `cache.five_hour` | object (`left_pct`, `resets_at`) | API `five_hour` quota block | `write_quota_cache()` | Cache fallback reads | [033](../feature/033_quota_cache.md) |
| `cache.seven_day` | object (`left_pct`, `resets_at`) | API `seven_day` quota block | `write_quota_cache()` | Cache fallback reads | [033](../feature/033_quota_cache.md) |
| `cache.seven_day_sonnet` | object or null | API `seven_day_sonnet` quota block | `write_quota_cache()` | Cache fallback reads | [033](../feature/033_quota_cache.md) |
| `cache.model_override` | string | `apply_model_override()` decision | `write_cache_string()` | Model override restore on next run | [033](../feature/033_quota_cache.md) |
| `cache.last_touch_at` | string (ISO 8601) | Touch subprocess completion time | `write_cache_field()` after touch | Touch lifecycle state | [024](../feature/024_session_touch.md), [033](../feature/033_quota_cache.md) |
| `cache.touch_idle` | bool | Touch subprocess idle determination | `write_cache_field()` after touch | Touch lifecycle gating | [024](../feature/024_session_touch.md), [033](../feature/033_quota_cache.md) |
| `history` | array of objects | Successful API measurements | `fetch.rs` after every successful `fetch_oauth_usage()` call | `approximate_quota()` in `approx.rs` | [040](../feature/040_quota_measurement_history.md) |
| `history[*].ts` | number (unix secs) | Measurement timestamp | history append | Polynomial fitting | [040](../feature/040_quota_measurement_history.md) |
| `history[*].five_hour` | number (%) | `five_hour.utilization` at measurement time | history append | Polynomial fitting | [040](../feature/040_quota_measurement_history.md) |
| `history[*].seven_day` | number (%) | `seven_day.utilization` at measurement time | history append | Polynomial fitting | [040](../feature/040_quota_measurement_history.md) |
| `history[*].seven_day_sonnet` | number or null (%) | `seven_day_sonnet.utilization` at measurement time | history append | Polynomial fitting | [040](../feature/040_quota_measurement_history.md) |
| `history[*].five_h_resets_at` | string (ISO 8601) or null | `five_hour.resets_at` at measurement | history append | Reset boundary filter | [040](../feature/040_quota_measurement_history.md) |
| `history[*].seven_d_resets_at` | string (ISO 8601) or null | `seven_day.resets_at` at measurement | history append | Reset boundary filter | [040](../feature/040_quota_measurement_history.md) |

### Preserved-Only Fields

These fields are written by one caller and never touched by others (preserved via read-merge):

- `_renewal_at` — written only by `.account.renewal`; never overwritten by `.account.save`
- `owner` — written only by ownership operations; preserved by save
- `host`, `role` — written at save time with explicit params; preserved on re-save without those params

### Example

```json
{
  "oauthAccount": {
    "displayName": "alice",
    "organizationRole": "admin",
    "billingType": "stripe_subscription"
  },
  "model": "sonnet",
  "tagged_id": "alice@example.com",
  "uuid": "01234567-...",
  "capabilities": ["claude_max"],
  "organization_uuid": "org-abc123",
  "organization_name": "Example Corp",
  "organization_role": "admin",
  "workspace_uuid": null,
  "workspace_name": null,
  "host": "w003",
  "role": "work",
  "_renewal_at": "2026-07-01T00:00:00Z",
  "owner": "user1@w003",
  "cache": {
    "fetched_at": "2026-06-23T12:00:00Z",
    "status": "ok",
    "five_hour": { "left_pct": 42.5, "resets_at": "2026-06-23T16:00:00Z" },
    "seven_day": { "left_pct": 30.0, "resets_at": "2026-06-27T00:00:00Z" },
    "seven_day_sonnet": null,
    "model_override": "opus",
    "last_touch_at": "2026-06-23T06:30:00Z",
    "touch_idle": false
  },
  "history": [
    { "ts": 1749900000, "five_hour": 10.0, "seven_day": 20.0, "seven_day_sonnet": null,
      "five_h_resets_at": "2026-06-22T12:00:00Z", "seven_d_resets_at": "2026-06-27T00:00:00Z" }
  ]
}
```

### Schema

| File | Relationship |
|------|-------------|
| [001_credentials_json.md](001_credentials_json.md) | Companion credential file `{name}.credentials.json` |

### Features

| File | Relationship |
|------|-------------|
| [feature/002_account_save.md](../feature/002_account_save.md) | Save algorithm — read-merge semantics |
| [feature/014_rich_account_metadata.md](../feature/014_rich_account_metadata.md) | `oauthAccount` subtree, `model` field |
| [feature/021_extended_snapshot_fields.md](../feature/021_extended_snapshot_fields.md) | `tagged_id`, `uuid`, `capabilities` |
| [feature/022_org_identity_snapshot.md](../feature/022_org_identity_snapshot.md) | `org_uuid`, `org_name` |
| [feature/029_account_host_metadata.md](../feature/029_account_host_metadata.md) | `host`, `role` label fields |
| [feature/030_account_renewal_override.md](../feature/030_account_renewal_override.md) | `_renewal_at` field |
| [feature/033_quota_cache.md](../feature/033_quota_cache.md) | `cache` subtree |
| [feature/036_account_ownership.md](../feature/036_account_ownership.md) | `owner` field |
| [feature/040_quota_measurement_history.md](../feature/040_quota_measurement_history.md) | `history` array |
| [feature/063_explicit_ownership_claim.md](../feature/063_explicit_ownership_claim.md) | `owner::` param write path |

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/007](../invariant/007_json_storage_format.md) | 2-space pretty-print + trailing newline |

# Schema: Account Metadata — `{name}.json`

### Scope

- **Purpose**: Unified field table for the per-account supplementary metadata file stored alongside `{name}.credentials.json`.
- **Responsibility**: Documents all fields in the per-account supplementary metadata file `{name}.json` across all features.
- **In Scope**: All fields written or read by `clp` across all features — core identity, OAuth metadata, org identity, extended snapshot fields, host/role labels, renewal override, ownership, and low-churn quota metadata (top-level keys since TSK-500; the legacy `cache` subtree is documented for pre-migration files).
- **Out of Scope**: `{name}.credentials.json` (→ [schema/001](001_credentials_json.md)); the untracked local volatile cache `-cache/{name}.json` (→ [feature/033](../feature/033_quota_cache.md) — volatile quota fields and measurement history live there since TSK-500); HTTP API response shapes.

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
| `backend` | string (`"anthropic"` \| `"redirect"`) | `backend::` CLI param at save time | `save()` | `switch_account()` branch selection; `list()` → `backend` field; `apply_model_override()` bypass check | [071](../feature/071_redirect_backend_accounts.md) |
| `base_url` | string | `base_url::` CLI param at save time | `save()` (redirect accounts only) | `switch_account()` → writes `env.ANTHROPIC_BASE_URL` in `settings.json` | [071](../feature/071_redirect_backend_accounts.md) |
| `redirect_model` | string | `redirect_model::` CLI param at save time | `save()` (redirect accounts only) | `switch_account()` → writes `env.ANTHROPIC_MODEL` in `settings.json` | [071](../feature/071_redirect_backend_accounts.md) |
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
| `inference_provider` | string | `inference_provider::` CLI param at save time | `save()` when `inference_provider::` param given | `list()` → `inference_provider` field (default identity column, `cols::-inference_provider` to hide); Gate 10 (eligibility) compares against the active `provider` config value | [072](../feature/072_inference_provider_selection.md) |
| `_renewal_at` | string (ISO 8601) | `at::` or `from_now::` CLI param | `.account.renewal` command | `list()` → `~Renews` / `→ Next` columns | [030](../feature/030_account_renewal_override.md) |
| `owner` | string | `owner::` CLI param or `current_identity()` | `.accounts owner::`, `.account.assign` (removed in F037) | `list()` → ownership gate checks; `current_identity()` comparison | [036](../feature/036_account_ownership.md), [063](../feature/063_explicit_ownership_claim.md) |
| `claim_lock` | bool | `lock::` CLI param | `.accounts lock::` | Gate 9 (eligibility); G9 (explicit-command on `.account.use`/`assignee::`) | [070](../feature/070_account_claim_and_reservation_control.md) |
| `reserve` | bool | `reserve::` CLI param | `.accounts reserve::` | `find_next_for_strategy()` leading sort key | [070](../feature/070_account_claim_and_reservation_control.md) |
| `model_override` | string | `apply_model_override()` decision | `write_cache_string()` (top-level since TSK-500) | Model override restore on next run (top-level first, legacy `cache.model_override` fallback) | [033](../feature/033_quota_cache.md) |
| `last_touch_at` | string (ISO 8601) | Touch subprocess completion time | `write_cache_field()` after touch (top-level since TSK-500) | Touch lifecycle state (top-level first, legacy fallback) | [024](../feature/024_session_touch.md), [033](../feature/033_quota_cache.md) |
| `touch_idle` | bool | Touch subprocess idle determination | `write_cache_field()` after touch (top-level since TSK-500) | Touch lifecycle gating (top-level first, legacy fallback) | [024](../feature/024_session_touch.md), [033](../feature/033_quota_cache.md) |
| `org_created_at` | string (ISO 8601) | Live `fetch_oauth_account` response `org_created_at` field | `write_cache_string_if_changed()` after successful account fetch (top-level since TSK-500; write skipped when value unchanged) | Non-live-fetch branches → `AccountQuota.org_created_at` → `~Renews` Estimate (033 AC-15) | [033](../feature/033_quota_cache.md) |
| `cache` | object — **LEGACY (pre-TSK-500)** | Old single-tier layout: volatile quota fields + low-churn metadata + `history[]` in one tracked subtree | No current writer — dissolved by the one-time migration inside `write_quota_cache()` (033 AC-18): low-churn keys relocated to top level, volatile fields and history moved to the untracked local `-cache/{name}.json`, `cache` key removed | Merged-read fallback until migrated: `read_quota_cache()` / `read_history()` read it when the local cache file is absent | [033](../feature/033_quota_cache.md), [040](../feature/040_quota_measurement_history.md) |

### Preserved-Only Fields

These fields are written by one caller and never touched by others (preserved via read-merge):

- `_renewal_at` — written only by `.account.renewal`; never overwritten by `.account.save`
- `owner` — written only by ownership operations; preserved by save
- `claim_lock`, `reserve` — written only by `.accounts lock::`/`reserve::`; preserved by save; both default to `false` when absent
- `host`, `role` — written at save time with explicit params; preserved on re-save without those params
- `inference_provider` — written at save time with explicit `inference_provider::` param; preserved on re-save without that param; defaults to `"anthropic"` when absent (pre-existing accounts, and all accounts created before Feature 072)
- `backend` — written only at `.account.save` creation time; preserved on re-save; defaults to `"anthropic"` when absent (pre-existing accounts, and all accounts created before Feature 071)
- `base_url`, `redirect_model` — written only at `.account.save` creation time for `backend::redirect` accounts; preserved on re-save; absent entirely for `anthropic` accounts
- `model_override`, `last_touch_at`, `touch_idle`, `org_created_at` — written only by the quota/touch/override side-effect paths (top-level since TSK-500); preserved by save

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
  "inference_provider": "anthropic",
  "_renewal_at": "2026-07-01T00:00:00Z",
  "owner": "user1@w003",
  "claim_lock": false,
  "reserve": false,
  "model_override": "opus",
  "last_touch_at": "2026-06-23T06:30:00Z",
  "touch_idle": false,
  "org_created_at": "2026-01-01T00:00:00Z"
}
```

Volatile quota fields (`fetched_at`, `status`, `five_hour`, `seven_day`, `seven_day_sonnet`, `history`) live in the untracked local `-cache/{name}.json` since TSK-500 — see [feature/033](../feature/033_quota_cache.md) for its structure. A pre-migration file instead carries all of these plus the four low-churn keys nested in a legacy `cache{}` subtree.

### Redirect Backend Example

A `backend::redirect` account (see [feature/071](../feature/071_redirect_backend_accounts.md)) omits every OAuth-quota-specific field (the low-churn keys `model_override`/`last_touch_at`/`touch_idle`/`org_created_at`, any legacy `cache` subtree, and `_renewal_at`) — there is no Claude quota to track — and carries `base_url`/`redirect_model` instead:

```json
{
  "backend": "redirect",
  "base_url": "https://api.moonshot.ai/anthropic",
  "redirect_model": "kimi-k3",
  "host": "w003",
  "role": "kimi"
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
| [feature/033_quota_cache.md](../feature/033_quota_cache.md) | Low-churn top-level keys (`model_override`, `last_touch_at`, `touch_idle`, `org_created_at`) + legacy `cache` subtree + the local `-cache/{name}.json` companion |
| [feature/036_account_ownership.md](../feature/036_account_ownership.md) | `owner` field |
| [feature/040_quota_measurement_history.md](../feature/040_quota_measurement_history.md) | History array — in local `-cache/{name}.json` since TSK-500; legacy `cache.history` readable pre-migration |
| [feature/063_explicit_ownership_claim.md](../feature/063_explicit_ownership_claim.md) | `owner::` param write path |
| [feature/070_account_claim_and_reservation_control.md](../feature/070_account_claim_and_reservation_control.md) | `claim_lock`, `reserve` fields |
| [feature/071_redirect_backend_accounts.md](../feature/071_redirect_backend_accounts.md) | `backend`, `base_url`, `redirect_model` fields |
| [feature/072_inference_provider_selection.md](../feature/072_inference_provider_selection.md) | `inference_provider` field |

### Invariants

| File | Relationship |
|------|-------------|
| [invariant/007](../invariant/007_json_storage_format.md) | 2-space pretty-print + trailing newline |

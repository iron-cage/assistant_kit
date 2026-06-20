# Endpoint: GET /api/oauth/usage

### Scope

- **Purpose**: Per-period quota utilization for Claude Max accounts — primary data source for the `.usage` command.
- **Responsibility**: Complete wire contract for `GET /api/oauth/usage`: URL, auth, full response schema including all known active and inactive fields, field semantics, and error codes.
- **In Scope**: Request headers, response JSON schema, `utilization`/`resets_at` semantics, inactive bucket inventory, `extra_usage` shape, error behavior.
- **Out of Scope**: Rate-limit header data (→ `003_v1_messages.md`); account identity/billing info (→ `002_oauth_account.md`); Rust parsing implementation (→ `src/lib.rs::parse_oauth_usage`).

### Request

```
GET https://api.anthropic.com/api/oauth/usage
Authorization: Bearer {access_token}
```

No `anthropic-version` or `anthropic-beta` headers required. Simple GET — no request body.

Workspace constant: `claude_quota::OAUTH_USAGE_URL`.

### Response

HTTP 200 on success. Body is a JSON object.

**Active period buckets** (present for Claude Max accounts):

| Field | Type | Range | Semantics |
|-------|------|-------|-----------|
| `five_hour.utilization` | `f64` | 0.0–100.0 | Consumed % of 5-hour session quota |
| `five_hour.resets_at` | `string\|null` | ISO-8601 UTC | When the 5-hour window resets |
| `seven_day.utilization` | `f64` | 0.0–100.0 | Consumed % of 7-day all-model quota |
| `seven_day.resets_at` | `string\|null` | ISO-8601 UTC | When the 7-day window resets |
| `seven_day_sonnet.utilization` | `f64` | 0.0–100.0 | Consumed % of 7-day Sonnet-only quota |
| `seven_day_sonnet.resets_at` | `string\|null` | ISO-8601 UTC | When the Sonnet 7-day window resets |

**Inactive buckets** (null or zeroed as of 2026-05-23; reserved for future features):

| Field | Observed value |
|-------|----------------|
| `seven_day_oauth_apps` | `null` |
| `seven_day_opus` | `null` |
| `seven_day_cowork` | `null` |
| `seven_day_omelette` | `{"utilization": 0.0, "resets_at": null}` |
| `tangelo` | `null` |
| `iguana_necktie` | `null` |
| `omelette_promotional` | `null` |

**Pay-as-you-go overage** (`extra_usage` — disabled for all observed accounts):

| Field | Type | Observed |
|-------|------|---------|
| `is_enabled` | bool | `false` |
| `monthly_limit` | number\|null | `null` |
| `used_credits` | number\|null | `null` |
| `utilization` | f64\|null | `null` |
| `currency` | string\|null | `null` |
| `disabled_reason` | string\|null | `null` |

**Example response:**

```json
{
  "five_hour":             {"utilization": 34.0, "resets_at": "2026-05-23T09:50:00.363135+00:00"},
  "seven_day":             {"utilization": 32.0, "resets_at": "2026-05-25T01:00:00.363161+00:00"},
  "seven_day_sonnet":      {"utilization": 53.0, "resets_at": "2026-05-25T01:00:00.363173+00:00"},
  "seven_day_oauth_apps":  null,
  "seven_day_opus":        null,
  "seven_day_cowork":      null,
  "seven_day_omelette":    {"utilization": 0.0, "resets_at": null},
  "tangelo":               null,
  "iguana_necktie":        null,
  "omelette_promotional":  null,
  "extra_usage": {
    "is_enabled": false,
    "monthly_limit": null,
    "used_credits": null,
    "utilization": null,
    "currency": null,
    "disabled_reason": null
  }
}
```

### Field Semantics

**`utilization`**: 0.0 = nothing consumed this period, 100.0 = quota fully consumed. Remaining quota = `100.0 - utilization`.

**`resets_at`**: ISO-8601 UTC timestamp marking when the rolling window clears. `null` when the bucket is inactive (no active data for that feature).

**Period independence**: Each period's `resets_at` clock is independent. `five_hour.resets_at` and `seven_day.resets_at` always differ and advance independently based on usage.

**Non-Max accounts**: Accounts with `billing_type: none` still return HTTP 200 with quota fields populated. The data reflects historical rolling-window activity and may show non-zero utilization even after subscription cancellation.

### Error Codes

| HTTP | Meaning | Typical action |
|------|---------|----------------|
| 401 | Token invalid or missing | Refresh token via OAuth lifecycle |
| 403 | Token lacks required scope | Re-authenticate |
| 429 | Rate limited | Back off; conditionally refresh if local token is expired |

### Since

pre-v1.0 (unverified)

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../../../module/claude_quota/src/lib.rs` | `fetch_oauth_usage`, `parse_oauth_usage`, `OAUTH_USAGE_URL` |
| doc | `../../../../module/claude_profile/docs/feature/009_token_usage.md` | `.usage` command — primary consumer of this endpoint |
| doc | `../../../../module/claude_profile/docs/feature/017_token_refresh.md` | Auth-error retry and refresh trigger |
| doc | [002_oauth_account.md](002_oauth_account.md) | Account identity and billing type endpoint |
| doc | [003_v1_messages.md](003_v1_messages.md) | Alternative quota source via POST response headers |

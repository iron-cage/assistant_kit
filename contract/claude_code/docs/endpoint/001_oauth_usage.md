# Endpoint: GET /api/oauth/usage

### Scope

- **Purpose**: Per-period quota utilization for Claude Max accounts — primary data source for the `.usage` command.
- **Responsibility**: Complete wire contract for `GET /api/oauth/usage`: URL, auth, full response schema including all known active and inactive fields, field semantics, and error codes.
- **In Scope**: Request headers, response JSON schema, `utilization`/`resets_at` semantics, inactive bucket inventory, `extra_usage` shape, `limits[]` quota boundary array, `spend` object, error behavior.
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
| `five_hour.limit_dollars` | `number\|null` | — | Reserved — `null` in every response observed (as of 2026-07-28) |
| `five_hour.used_dollars` | `number\|null` | — | Reserved — `null` in every response observed (as of 2026-07-28) |
| `five_hour.remaining_dollars` | `number\|null` | — | Reserved — `null` in every response observed (as of 2026-07-28) |
| `seven_day.utilization` | `f64` | 0.0–100.0 | Consumed % of 7-day all-model quota |
| `seven_day.resets_at` | `string\|null` | ISO-8601 UTC | When the 7-day window resets |
| `seven_day.limit_dollars` | `number\|null` | — | Reserved — `null` in every response observed (as of 2026-07-28) |
| `seven_day.used_dollars` | `number\|null` | — | Reserved — `null` in every response observed (as of 2026-07-28) |
| `seven_day.remaining_dollars` | `number\|null` | — | Reserved — `null` in every response observed (as of 2026-07-28) |
| `seven_day_sonnet.utilization` | `f64` | 0.0–100.0 | Consumed % of 7-day Sonnet-only quota |
| `seven_day_sonnet.resets_at` | `string\|null` | ISO-8601 UTC | When the Sonnet 7-day window resets |

`seven_day_sonnet` has returned the bare value `null` (never a populated object) in every response observed since 2026-06-25 — see [algorithm/009_oauth_usage_response_migration.md](../../../../module/claude_profile/docs/algorithm/009_oauth_usage_response_migration.md). Whether a populated `seven_day_sonnet` object would also carry `limit_dollars`/`used_dollars`/`remaining_dollars` is unconfirmed — only `five_hour` and `seven_day` have been directly observed as non-null objects carrying these 3 fields.

**Quota boundary entries (`limits[]`)** — array introduced 2026-06-25, extended with a 3rd entry kind since. Present in every response observed:

| Field | Type | Semantics |
|-------|------|-----------|
| `kind` | `string` | Boundary type. Observed values: `"session"` (5h window), `"weekly_all"` (7d all-model window), `"weekly_scoped"` (7d window scoped to a specific model — new as of this investigation, 2026-07-28) |
| `group` | `string` | Display grouping — `"session"` or `"weekly"` |
| `percent` | `integer` | Consumed %, 0–100. Semantically identical to `utilization` in the named-field format: `utilization = percent as f64` |
| `severity` | `string` | `"normal"` observed exclusively so far; `"warning"`/`"critical"` presumed from naming, not yet observed |
| `resets_at` | `string` | ISO-8601 UTC reset timestamp |
| `scope` | `object\|null` | `null` for `"session"`/`"weekly_all"`. For `"weekly_scoped"`, an **object** (not a string): `{"model": {"id": string\|null, "display_name": string}, "surface": string\|null}` — `id` observed `null`, `display_name` observed `"Fable"`, `surface` observed `null`, in every sample so far |
| `is_active` | `bool` | Observed to vary independently of raw `percent` in a way not fully characterized — e.g. one account with 5h=74%/7d=71% showed `session.is_active=true, weekly_all.is_active=false`, while a near-idle account (5h=0%/7d=1%) showed the reverse. Likely indicates "which window is currently the binding/governing constraint" rather than raw window-open state, but this is an open/tentative reading from 2 samples, not a confirmed semantic |

**Structural note:** `scope` being an object for `"weekly_scoped"` (rather than a flat string) is significant for parser forward-compatibility — see the "Known Limitation" entry in [algorithm/009_oauth_usage_response_migration.md](../../../../module/claude_profile/docs/algorithm/009_oauth_usage_response_migration.md).

**Inactive buckets** (null or zeroed as of 2026-07-28; reserved for future features):

| Field | Observed value |
|-------|----------------|
| `seven_day_oauth_apps` | `null` |
| `seven_day_opus` | `null` |
| `seven_day_cowork` | `null` |
| `seven_day_omelette` | `null` |
| `tangelo` | `null` |
| `iguana_necktie` | `null` |
| `omelette_promotional` | `null` |
| `cinder_cove` | `null` |
| `amber_ladder` | `null` |
| `nimbus_quill` | `null` |

**Pay-as-you-go overage** (`extra_usage` — disabled for all observed accounts):

| Field | Type | Observed |
|-------|------|---------|
| `is_enabled` | bool | `false` |
| `monthly_limit` | number\|null | `null` |
| `used_credits` | number\|null | `null` |
| `utilization` | f64\|null | `null` |
| `currency` | string\|null | `null` |
| `decimal_places` | number\|null | `null` |
| `disabled_reason` | string\|null | `null` |
| `user_disabled` | bool | real per-account value — observed both `false` and `true` across accounts |
| `spend_limit_reached` | bool | `false` observed |
| `credits_ever_enabled` | bool | real per-account value — observed both `false` and `true` across accounts |
| `daily` | object\|null | `null` observed |
| `weekly` | object\|null | `null` observed |

**Spend (`spend`)** — new top-level object, not present in the pre-2026-06-25 or 2026-06-25 response shapes:

| Field | Type | Observed |
|-------|------|---------|
| `used.amount_minor` | integer | `0` |
| `used.currency` | string | `"USD"` |
| `used.exponent` | integer | `2` |
| `limit` | —\|null | `null` |
| `percent` | integer | `0` |
| `severity` | string | `"normal"` |
| `enabled` | bool | `false` |
| `disabled_reason` | string\|null | `null` |
| `cap` | —\|null | `null` |
| `balance` | —\|null | `null` |
| `auto_reload` | —\|null | `null` |
| `disclaimer` | string | user-facing credits explainer text |
| `can_purchase_credits` | bool | `false` |
| `can_toggle` | bool | `false` |

**Other top-level fields:**

| Field | Type | Observed |
|-------|------|---------|
| `member_dashboard_available` | bool | `false` — new top-level field, not present in the pre-2026-06-25 or 2026-06-25 response shapes |

**Example response** (live-verified 2026-07-28 against 2 independent accounts; values below are illustrative, not a literal capture):

```json
{
  "five_hour": {
    "utilization": 34.0, "resets_at": "2026-07-28T09:50:00.363135+00:00",
    "limit_dollars": null, "used_dollars": null, "remaining_dollars": null
  },
  "seven_day": {
    "utilization": 32.0, "resets_at": "2026-07-28T19:00:00.363161+00:00",
    "limit_dollars": null, "used_dollars": null, "remaining_dollars": null
  },
  "seven_day_oauth_apps":  null,
  "seven_day_opus":        null,
  "seven_day_sonnet":      null,
  "seven_day_cowork":      null,
  "seven_day_omelette":    null,
  "tangelo":               null,
  "iguana_necktie":        null,
  "omelette_promotional":  null,
  "nimbus_quill":          null,
  "cinder_cove":           null,
  "amber_ladder":          null,
  "extra_usage": {
    "is_enabled": false, "monthly_limit": null, "used_credits": null, "utilization": null,
    "currency": null, "decimal_places": null, "disabled_reason": null, "user_disabled": false,
    "spend_limit_reached": false, "credits_ever_enabled": false, "daily": null, "weekly": null
  },
  "limits": [
    {"kind": "session", "group": "session", "percent": 34, "severity": "normal",
     "resets_at": "2026-07-28T09:50:00.363135+00:00", "scope": null, "is_active": true},
    {"kind": "weekly_all", "group": "weekly", "percent": 32, "severity": "normal",
     "resets_at": "2026-07-28T19:00:00.363161+00:00", "scope": null, "is_active": false},
    {"kind": "weekly_scoped", "group": "weekly", "percent": 8, "severity": "normal",
     "resets_at": "2026-07-28T19:00:00.516422+00:00",
     "scope": {"model": {"id": null, "display_name": "Fable"}, "surface": null}, "is_active": false}
  ],
  "spend": {
    "used": {"amount_minor": 0, "currency": "USD", "exponent": 2},
    "limit": null, "percent": 0, "severity": "normal", "enabled": false, "disabled_reason": null,
    "cap": null, "balance": null, "auto_reload": null,
    "disclaimer": "Usage credits cover you when you hit your plan limits. [Learn more](https://support.claude.com/articles/12429409)",
    "can_purchase_credits": false, "can_toggle": false
  },
  "member_dashboard_available": false
}
```

### Field Semantics

**`utilization`**: 0.0 = nothing consumed this period, 100.0 = quota fully consumed. Remaining quota = `100.0 - utilization`.

**`resets_at`**: ISO-8601 UTC timestamp marking when the rolling window clears. `null` when the bucket is inactive (no active data for that feature).

**Period independence**: Each period's `resets_at` clock is independent. `five_hour.resets_at` and `seven_day.resets_at` always differ and advance independently based on usage.

**Non-Max accounts**: Accounts with `billing_type: none` still return HTTP 200 with quota fields populated. The data reflects historical rolling-window activity and may show non-zero utilization even after subscription cancellation.

**`limits[].percent`**: same 0–100 consumed-percentage semantics as `utilization` — direct cast, no scale conversion (`utilization = percent as f64`).

**`limits[].scope`**: a JSON *object* for `"weekly_scoped"` entries, not a string — `{"model": {"id", "display_name"}, "surface"}`. Any parser treating `scope` as a plain string (matching the pre-2026-06-25 assumption that a re-enabled per-model bucket would carry a flat `"sonnet"`/`"opus"` string) will silently fail to read it. `null` for `"session"`/`"weekly_all"` entries.

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
| doc | [004_oauth_token.md](004_oauth_token.md) | Token refresh endpoint — produces the access token consumed by this endpoint |
| doc | `../../../../module/claude_profile/docs/algorithm/009_oauth_usage_response_migration.md` | Dual-source parsing algorithm, `limits[]` field semantics, and the `scan_limits_for_kind()` known limitation against nested `scope` objects |

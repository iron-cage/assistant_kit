# Account Field Index

Cross-endpoint inventory of every field returned by account-related Anthropic API endpoints, organized by concept domain.

### Scope

- **Purpose**: Single-reference field dictionary across all observed account endpoints — what data is available, from which source, in what format.
- **Responsibility**: Field name, source endpoint, type, observed values, and semantic notes for every known account field. Overlap analysis between endpoints serving similar concepts.
- **In Scope**: All fields from endpoints 001–005 that carry account or session state. Header fields from endpoint 003.
- **Out of Scope**: Request body fields (→ individual endpoint files); endpoints 006–010 (blocked or domain-only); Anthropic public inference API body fields.

---

## User Identity

Source: **[002](002_oauth_account.md)** only.

| Field | Type | Example | Semantics |
|-------|------|---------|-----------|
| `tagged_id` | string | `"user_01BDCWiki5PxAn3hFN1Whvrx"` | Stable user ID with type prefix |
| `uuid` | string | `"52af6817-f911-408f-b033-5e1d977af315"` | UUID form of user ID |
| `email_address` | string | `"i11@wbox.pro"` | Primary account email |
| `full_name` | string | `"i11"` | Full display name |
| `display_name` | string | `"i11"` | Short display name |
| `is_verified` | bool | `true` | Email address verified |
| `age_is_verified` | bool | `true` | Age gate passed |
| `is_anonymous` | bool | `false` | Anonymous session flag |
| `created_at` (user-level) | string | ISO-8601 UTC | User account creation date |
| `updated_at` (user-level) | string | ISO-8601 UTC | Last profile change |
| `completed_verification_at` | string | ISO-8601 UTC | Email/age verification timestamp |
| `verified_phone_number_last4` | string\|null | `null` | Last 4 digits of verified phone |
| `settings` (user-level) | object | `{}` | ~60-key UI preferences; all fields `null` for standard accounts |

---

## Subscription & Billing

Source: **[001](001_oauth_usage.md)** (quota JSON), **[002](002_oauth_account.md)** (billing state), **[003](003_v1_messages.md)** (rate-limit headers).

### From 002 — `memberships[0].organization`

| Field | Type | Observed values | Semantics |
|-------|------|----------------|-----------|
| `billing_type` | string | `"stripe_subscription"`, `"none"` | Active subscription status |
| `capabilities` | string[] | `["claude_max","chat"]`, `["chat"]` | Enabled product features |
| `rate_limit_tier` | string | `"default_claude_max_20x"`, `"default_claude_ai"` | Quota tier identifier |
| `rate_limit_upsell` | string | `"get_more_usage"`, `"upgrade_to_pro"` | UI upsell prompt |
| `free_credits_status` | string | `"available"`, `"granted_manually"` | Free credit grant state |
| `created_at` (org-level) | string | ISO-8601 UTC | **Billing cycle anchor** — day-of-month used for Stripe renewal estimation |
| `billable_usage_paused_until` | string\|null | `null` | Usage billing pause end |
| `api_disabled_reason` | string\|null | `null` | Reason API access is disabled |
| `api_disabled_until` | string\|null | `null` | Until-timestamp for API disable |
| `merchant_of_record` | string | `"anthropic"` | Payment processor entity |
| `data_retention` | string | `"default"` | Data retention policy |
| `parent_organization_uuid` | string\|null | `null` | Parent org UUID (enterprise only) |
| `settings` (org-level) | object | `{}` | ~50-key feature flags; all `null` for personal orgs |

**Subscription identity:**

```
Max active:   billing_type == "stripe_subscription"  AND  "claude_max" IN capabilities
No sub:       billing_type == "none"                 AND  capabilities == ["chat"]
Pro (no Max): billing_type == "stripe_subscription"  AND  "claude_max" NOT IN capabilities
```

### From 001 — `GET /api/oauth/usage` JSON body

| Field | Type | Range | Semantics |
|-------|------|-------|-----------|
| `five_hour.utilization` | f64 | 0.0–100.0 | 5-hour session quota consumed % |
| `five_hour.resets_at` | string\|null | ISO-8601 UTC | 5-hour window reset timestamp |
| `seven_day.utilization` | f64 | 0.0–100.0 | 7-day all-model quota consumed % |
| `seven_day.resets_at` | string\|null | ISO-8601 UTC | 7-day window reset timestamp |
| `seven_day_sonnet.utilization` | f64 | 0.0–100.0 | 7-day Sonnet-only quota consumed % |
| `seven_day_sonnet.resets_at` | string\|null | ISO-8601 UTC | Sonnet 7-day reset timestamp |
| `extra_usage.is_enabled` | bool | `false` observed | Pay-as-you-go overage enabled |
| `extra_usage.monthly_limit` | number\|null | `null` | Monthly overage cap |
| `extra_usage.used_credits` | number\|null | `null` | Credits used this month |
| `extra_usage.utilization` | f64\|null | `null` | Overage utilization fraction |
| `extra_usage.currency` | string\|null | `null` | Billing currency |
| `extra_usage.disabled_reason` | string\|null | `null` | Overage disable reason |

**Inactive buckets** (all `null` or zeroed as of 2026-05-23):
`seven_day_oauth_apps`, `seven_day_opus`, `seven_day_cowork`, `tangelo`, `iguana_necktie`, `omelette_promotional`; `seven_day_omelette` returns `{"utilization":0.0,"resets_at":null}`.

### From 003 — `POST /v1/messages` response headers

| Header | Type | Range | Semantics |
|--------|------|-------|-----------|
| `anthropic-ratelimit-unified-5h-utilization` | f64 string | 0.0–1.0 | 5-hour quota consumed (fraction, NOT percent) |
| `anthropic-ratelimit-unified-5h-reset` | u64 string | Unix seconds | 5-hour window reset timestamp |
| `anthropic-ratelimit-unified-7d-utilization` | f64 string | 0.0–1.0 | 7-day all-model quota consumed (fraction) |
| `anthropic-ratelimit-unified-7d-reset` | u64 string | Unix seconds | 7-day window reset timestamp |
| `anthropic-ratelimit-unified-status` | string | — | `"allowed"` \| `"allowed_warning"` \| `"rejected"` |

---

## Organization Membership

Source: **[002](002_oauth_account.md)** (`memberships[]`), **[005](005_claude_cli_roles.md)**.

### From 002 — `memberships[0]`

| Field | Type | Observed | Semantics |
|-------|------|----------|-----------|
| `role` | string | `"admin"` | User's role in the organization |
| `seat_tier` | string\|null | `null` | Seat tier (enterprise billing) |
| `created_at` (membership) | string | ISO-8601 UTC | When user joined the organization |
| `updated_at` (membership) | string | ISO-8601 UTC | Last membership update |

### From 005 — `GET /api/oauth/claude_cli/roles`

| Field | Type | Observed | Semantics |
|-------|------|----------|-----------|
| `organization_uuid` | string | UUID | Organization UUID |
| `organization_name` | string | `"i11@wbox.pro's Organization"` | Organization display name |
| `organization_role` | string | `"admin"` | User's role in the organization |
| `workspace_uuid` | string\|null | `null` | Workspace UUID (enterprise feature) |
| `workspace_name` | string\|null | `null` | Workspace display name |
| `workspace_role` | string\|null | `null` | User's role in the workspace |

---

## Token Lifecycle

Source: **[004](004_oauth_token.md)** only.

| Field | Type | Semantics |
|-------|------|-----------|
| `access_token` | string | New Bearer token for API calls |
| `refresh_token` | string | New refresh token — **invalidates** the one sent in the request |
| `expires_in` | u64 | Access token TTL in seconds (observed: 3600) |

---

## Overlap Analysis

### Quota data — 001 vs 003

Both endpoints expose 5-hour and 7-day quota utilization. They differ substantially:

| Aspect | 001 — `GET /api/oauth/usage` | 003 — `POST /v1/messages` headers |
|--------|------------------------------|-----------------------------------|
| Request cost | GET, no inference | POST inference (1 token billed) |
| Response format | JSON body | HTTP response headers |
| Scale | 0.0–100.0 % | 0.0–1.0 fraction |
| Buckets exposed | 5h, 7d, 7d\_sonnet + inactive | 5h, 7d only |
| Quota status field | not present | `unified-status` (`"allowed"` / `"rejected"`) |
| `anthropic-beta` required | no | yes (`oauth-2025-04-20`) |
| Headers on 4xx responses | n/a | yes — headers present even on auth errors |
| Implemented in workspace | `fetch_oauth_usage` | `fetch_rate_limits` |

**Use 001** when you only need quota data — no inference token cost. **Use 003** when you must make an inference call anyway (already paying) or when `unified-status` is needed.

### Organization identity — 002 vs 005

| Concept | 002 — `GET /api/oauth/account` | 005 — `GET /api/oauth/claude_cli/roles` |
|---------|--------------------------------|-----------------------------------------|
| Organization UUID | not exposed | `organization_uuid` ✓ |
| Organization name | not exposed | `organization_name` ✓ |
| User org role | `memberships[0].role` | `organization_role` |
| Workspace UUID/name/role | not present | `workspace_uuid/name/role` (null = personal) |
| Billing type | `org.billing_type` ✓ | not present |
| Capabilities / tier | `org.capabilities`, `rate_limit_tier` ✓ | not present |
| Billing cycle anchor | `org.created_at` ✓ | not present |
| Full user identity | all user fields ✓ | not present |
| Response size | large (~5 KB) | small (~200 B) |

**Use 005** for lightweight org UUID / role lookups. **Use 002** when billing type, capabilities, or `org.created_at` (billing cycle anchor for renewal estimation) is needed.

### Utilization scale mismatch — 001 vs 003

The same quota window is expressed at different scales across endpoints:

| Endpoint | 5-hour field | Value meaning |
|----------|-------------|---------------|
| 001 JSON | `five_hour.utilization` = `34.0` | 34% consumed → 66% remaining |
| 003 header | `5h-utilization` = `"0.34"` | 0.34 fraction consumed → 0.66 remaining |

**Conversion:** `remaining_pct = (1.0 - header_utilization) * 100`

---

## Field Coverage Matrix

Which endpoint exposes each concept domain:

| Concept | 001 | 002 | 003 | 004 | 005 |
|---------|:---:|:---:|:---:|:---:|:---:|
| User identity (email, name, UUID) | — | ✓ | — | — | — |
| Subscription / billing type | — | ✓ | — | — | — |
| Capabilities (Max/Pro) | — | ✓ | — | — | — |
| Billing cycle anchor | — | ✓ | — | — | — |
| 5-hour quota utilization | ✓ | — | ✓ | — | — |
| 7-day quota utilization | ✓ | — | ✓ | — | — |
| 7-day Sonnet quota | ✓ | — | — | — | — |
| Quota status (allowed/rejected) | — | — | ✓ | — | — |
| Org UUID | — | — | — | — | ✓ |
| Org name | — | — | — | — | ✓ |
| Org role | — | ✓ (via membership) | — | — | ✓ |
| Workspace fields | — | — | — | — | ✓ |
| New access token | — | — | — | ✓ | — |
| New refresh token | — | — | — | ✓ | — |
| Token TTL | — | — | — | ✓ | — |

---

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [001_oauth_usage.md](001_oauth_usage.md) | Full 001 quota response schema |
| doc | [002_oauth_account.md](002_oauth_account.md) | Full 002 account response schema |
| doc | [003_v1_messages.md](003_v1_messages.md) | Full 003 rate-limit header schema |
| doc | [004_oauth_token.md](004_oauth_token.md) | Full 004 token refresh schema |
| doc | [005_claude_cli_roles.md](005_claude_cli_roles.md) | Full 005 roles response schema |
| source | `../../../../module/claude_quota/src/lib.rs` | `fetch_oauth_usage`, `fetch_oauth_account`, `fetch_rate_limits` — transport implementations |

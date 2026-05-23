# Endpoint: GET /api/oauth/account

### Scope

- **Purpose**: Per-account identity, organization membership, billing type, subscription capabilities, and billing cycle anchor date — the authoritative live source for account state.
- **Responsibility**: Complete wire contract for `GET /api/oauth/account`: URL, auth, full response schema, billing semantics, credential-staleness detection, billing-cycle estimation, and known limitations.
- **In Scope**: User identity fields, organization membership schema (billing_type, capabilities, rate_limit_tier, created_at), credential file staleness detection, monthly renewal estimation from `org.created_at`.
- **Out of Scope**: Quota utilization (→ `001_oauth_usage.md`); Stripe billing portal (browser session required, not reachable via OAuth token); subscription end date (not exposed by this API — see Limitations).

### Request

```
GET https://api.anthropic.com/api/oauth/account
Authorization: Bearer {access_token}
anthropic-version: 2023-06-01
```

No request body.

### Response

HTTP 200 on success. Body is a JSON object.

**User-level fields:**

| Field | Type | Example |
|-------|------|---------|
| `tagged_id` | string | `"user_01BDCWiki5PxAn3hFN1Whvrx"` |
| `uuid` | string | `"52af6817-f911-408f-b033-5e1d977af315"` |
| `email_address` | string | `"i11@wbox.pro"` |
| `full_name` | string | `"i11"` |
| `display_name` | string | `"i11"` |
| `is_verified` | bool | `true` |
| `age_is_verified` | bool | `true` |
| `is_anonymous` | bool | `false` |
| `created_at` | string | ISO-8601 UTC — user account creation |
| `updated_at` | string | ISO-8601 UTC — last profile change |
| `completed_verification_at` | string | ISO-8601 UTC — email/age verification |
| `verified_phone_number_last4` | string\|null | last 4 digits of verified phone |

**Organization membership** (`memberships[0].organization`):

| Field | Type | Values observed | Semantics |
|-------|------|----------------|-----------|
| `billing_type` | string | `"stripe_subscription"`, `"none"` | Current subscription status |
| `capabilities` | string[] | `["claude_max","chat"]`, `["chat"]` | Enabled product features |
| `rate_limit_tier` | string | `"default_claude_max_20x"`, `"default_claude_ai"` | Quota tier |
| `rate_limit_upsell` | string | `"get_more_usage"`, `"upgrade_to_pro"` | UI upsell prompt |
| `free_credits_status` | string | `"available"`, `"granted_manually"` | Free credit grant |
| `created_at` | string | ISO-8601 UTC | **Billing cycle anchor** — see Billing Estimation |
| `billable_usage_paused_until` | string\|null | `null` | Usage billing pause end |
| `api_disabled_reason` | string\|null | `null` | Reason API access is disabled |
| `api_disabled_until` | string\|null | `null` | Until-timestamp for API disable |
| `merchant_of_record` | string | `"anthropic"` | Payment processor entity |
| `data_retention` | string | `"default"` | Data retention policy |
| `parent_organization_uuid` | string\|null | `null` | Parent org (enterprise only) |

**Membership-level fields** (`memberships[0]`):

| Field | Type | Observed |
|-------|------|----------|
| `role` | string | `"admin"` |
| `seat_tier` | string\|null | `null` |
| `created_at` | string | ISO-8601 UTC — when user joined org |
| `updated_at` | string | ISO-8601 UTC |

**`organization.settings`**: Large object (~50 keys) controlling org-level feature flags and Claude Code settings. All fields `null` in single-user personal organizations.

**`settings`** (user-level): Large object (~60 keys) for UI preferences, feature toggles, and onboarding state. All `internal_tier_*` and `internal_*_trial_*` fields observed as `null` for standard Max accounts.

### Billing Semantics

**Active Max subscription:**
```
billing_type == "stripe_subscription"
AND "claude_max" in capabilities
```

**No active subscription:**
```
billing_type == "none"
AND capabilities == ["chat"]
AND rate_limit_upsell == "upgrade_to_pro"
```

**`free_credits_status: "granted_manually"`** indicates Anthropic manually granted free credits to that account (distinct from the standard `"available"` grant).

### Credential Staleness Detection

The `subscriptionType` field in per-account `*.credentials.json` files is written at OAuth-token-creation time and is not updated when the subscription changes. The `billing_type` field from this endpoint is the authoritative current state.

| `credentials.json` subscriptionType | `org.billing_type` | True state |
|---|---|---|
| `"max"` | `"stripe_subscription"` | Active Max — consistent |
| `"max"` | `"none"` | Subscription cancelled after last login — **credential stale** |

### Billing Cycle Estimation

No subscription expiry or next-renewal date is returned. Stripe anchors monthly billing to the day-of-month from the subscription start date. `org.created_at` is the subscription start date.

```
next_renewal ≈ next_occurrence_of( day(org.created_at) ) after today
```

Example (observed, sampled 2026-05-23):

| Account | `org.created_at` | Billing day | Next renewal |
|---------|-----------------|-------------|--------------|
| i3@wbox.pro | 2025-11-11 | 11th | Jun 11, 2026 |
| i6@wbox.pro | 2025-12-06 | 6th | Jun 6, 2026 |
| i7@wbox.pro | 2026-03-03 | 3rd | Jun 3, 2026 |
| i8@wbox.pro | 2026-03-03 | 3rd | Jun 3, 2026 |
| illia.t@obox.systems | 2026-04-03 | 3rd | Jun 3, 2026 |
| i11@wbox.pro | 2026-05-05 | 5th | Jun 5, 2026 |
| i12@wbox.pro | 2026-05-05 | 5th | Jun 5, 2026 |

This is an estimate. Annual plans, mid-cycle prorations, or Stripe billing anchor adjustments are not accounted for.

### Known Limitations

**No subscription expiry date**: Neither `next_renewal_at` nor `subscription_expires_at` exists anywhere in this response. Billing period end is managed by Stripe and not exposed via the OAuth API.

**`/api/bootstrap` returns `account: null`** when called with an OAuth bearer token — the `account` field is only populated for web-session auth. Use this endpoint directly instead.

**`/api/account` returns 403** with `"account_session_invalid"` for OAuth bearer tokens — requires web-session cookies.

**`/api/organizations` returns 403** for OAuth bearer tokens for the same reason.

### Example Response

HTTP 200 (personal Max account, i11@wbox.pro, sampled 2026-05-23). `settings` and `organization.settings` are large objects (~50–60 keys each) that are all `null` for personal organizations — shown as `{}` here for brevity.

```json
{
  "tagged_id": "user_01BDCWiki5PxAn3hFN1Whvrx",
  "uuid": "52af6817-f911-408f-b033-5e1d977af315",
  "email_address": "i11@wbox.pro",
  "full_name": "i11",
  "display_name": "i11",
  "is_verified": true,
  "age_is_verified": true,
  "is_anonymous": false,
  "created_at": "2026-05-05T12:00:00.000Z",
  "updated_at": "2026-05-20T08:00:00.000Z",
  "completed_verification_at": "2026-05-05T12:05:00.000Z",
  "verified_phone_number_last4": null,
  "memberships": [
    {
      "role": "admin",
      "seat_tier": null,
      "created_at": "2026-05-05T12:00:00.000Z",
      "updated_at": "2026-05-05T12:00:00.000Z",
      "organization": {
        "billing_type": "stripe_subscription",
        "capabilities": ["claude_max", "chat"],
        "rate_limit_tier": "default_claude_max_20x",
        "rate_limit_upsell": "get_more_usage",
        "free_credits_status": "available",
        "created_at": "2026-05-05T12:00:00.000Z",
        "billable_usage_paused_until": null,
        "api_disabled_reason": null,
        "api_disabled_until": null,
        "merchant_of_record": "anthropic",
        "data_retention": "default",
        "parent_organization_uuid": null,
        "settings": {}
      }
    }
  ],
  "settings": {}
}
```

### Error Codes

| HTTP | Meaning |
|------|---------|
| 200 | Success |
| 401 | Token invalid or expired |
| 403 | Token lacks required scope, or endpoint requires web-session auth |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../../../module/claude_quota/src/lib.rs` | `fetch_oauth_account`, `parse_oauth_account`, `OAUTH_ACCOUNT_URL` |
| doc | `../../../../module/claude_profile/docs/feature/009_token_usage.md` | `.usage` command — consumer of this endpoint (Sub + ~Renews columns) |
| doc | `../../../../module/claude_profile/docs/feature/014_rich_account_metadata.md` | Rich metadata fields (reads local snapshot, not this API) |
| doc | [001_oauth_usage.md](001_oauth_usage.md) | Quota utilization endpoint |
| doc | [003_v1_messages.md](003_v1_messages.md) | Rate-limit header endpoint |

# Command :: 15. `.account.inspect` — Integration Tests

Unified live account diagnostic — identity, subscription, org, and quota utilization for one account via endpoints 002 (account), 005 (roles), and 001 (usage).

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Default (active account) produces expected output fields | Active account set; `clp .account.inspect`; output contains `Account:`, `Status:`, `Tagged ID:`, `Memberships:`, `Billing:` | 0 |
| IT-2 | `name::alice` inspects named account | Account `alice@acme.com` exists in credential store; `name::alice`; output shows `Account: alice@acme.com` | 0 |
| IT-3 | `format::json` outputs valid JSON with required keys | Account exists; `format::json`; output is valid JSON containing `account`, `status`, `memberships` fields | 0 |
| IT-4 | `trace::1` emits `[trace]` lines to stderr | Account exists; `trace::1`; stderr contains `[trace]` endpoint call lines; stdout contains normal output | 0 |
| IT-5 | `refresh::0` skips token refresh on expired credentials | Account with expired `expiresAt`; `refresh::0`; no subprocess spawned; command proceeds with local credentials | 0 |
| IT-6 | Unknown account name exits 2 | `name::nobody@example.com` not in credential store | 2 |
| IT-7 | Unknown parameter rejected → exit 1 | `clp .account.inspect unknown::x` | 1 |
| IT-8 | Multi-membership display marks `← selected` | Account with two memberships (billing_type=none and billing_type=stripe_subscription); `Memberships: 2` in output; second entry ends with `← selected` | 0 |
| IT-9 | Single membership display has no selection marker | Account with one membership; output contains no `← selected` text | 0 |
| IT-10 | Prefix resolves to full account name | Account `alice@acme.com` in credential store; `name::alice`; output shows `Account: alice@acme.com` | 0 |
| IT-11 | `format::json` membership array length matches text output count | Account with two memberships; `format::json | jq '.memberships | length'` returns `2` | 0 |
| IT-12 | `refresh::1` (default) attempts token refresh when locally expired | Account with expired `expiresAt`; `refresh::1` (default); refresh subprocess spawned; output present | 0 |
| IT-13 | Output includes `Name:` and `Email:` from endpoint 002 | Active account; endpoint 002 returns `full_name`, `display_name`, `email_address`; output contains `Name:` and `Email:` | 0 |
| IT-14 | Output includes `Capabilities:` and `Tier:` from selected membership | Active account; endpoint 002 returns membership with capabilities and rate_limit_tier; output contains `Capabilities:` and `Tier:` | 0 |
| IT-15 | Output includes usage data from endpoint 001 | Active account; endpoint 001 returns utilization; output contains `Session (5h):`, `Weekly (7d):`, `Sonnet (7d):` | 0 |
| IT-16 | `format::json` includes extended fields | Active account; `format::json`; JSON contains `email_address`, `full_name`, `capabilities`, `rate_limit_tier`, `session_5h_pct`, `weekly_7d_pct`, `sonnet_7d_pct` | 0 |

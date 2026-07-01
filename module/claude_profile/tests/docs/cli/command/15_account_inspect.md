# Command :: 15. `.account.inspect` — Integration Tests

Unified live account diagnostic — identity, subscription, org, and quota utilization for one account via endpoints 002 (account), 005 (roles), and 001 (usage).

| # | Test | Conditions | Exit |
|---|------|-----------|------|
| IT-1 | Default (active account) produces expected output fields | Active account set; `clp .account.inspect`; output contains `Account:`, `Status:`, `Tagged ID:`, `Memberships:`, `Billing:` | 0 |
| IT-2 | `name::alice` inspects named account | Account `alice@acme.com` exists in credential store; `name::alice`; output shows `Account: alice@acme.com` | 0 |
| IT-3 | `format::json` outputs valid JSON with required keys | Account exists; `format::json`; output is valid JSON containing `account`, `status`, `memberships` fields | 0 |
| IT-4 | `trace::1` emits timestamped diagnostic lines to stderr | Account exists; `trace::1`; stderr contains timestamped diagnostic endpoint call lines; stdout contains normal output | 0 |
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

---

### IT-1: Default (active account) produces expected output fields

- **Given:** Active account set in credential store
- **When:** `clp .account.inspect`
- **Then:** Exits 0. stdout contains `Account:`, `Status:`, `Tagged ID:`, `Memberships:`, `Billing:`.
- **Exit:** 0
- **Source:** [command/001_account.md](../../../../docs/cli/command/001_account.md)

---

### IT-2: `name::alice` inspects named account

- **Given:** Account `alice@acme.com` exists in credential store
- **When:** `clp .account.inspect name::alice@acme.com`
- **Then:** Exits 0. stdout contains `Account: alice@acme.com`.
- **Exit:** 0
- **Source:** [param/001_name.md](../../../../docs/cli/param/001_name.md)

---

### IT-3: `format::json` outputs valid JSON with required keys

- **Given:** Active account exists
- **When:** `clp .account.inspect format::json`
- **Then:** Exits 0. stdout is valid JSON containing `account`, `status`, and `memberships` fields.
- **Exit:** 0
- **Source:** [param/002_format.md](../../../../docs/cli/param/002_format.md)

---

### IT-4: `trace::1` emits timestamped diagnostic lines to stderr

- **Given:** Active account exists
- **When:** `clp .account.inspect trace::1`
- **Then:** Exits 0. stderr contains timestamped diagnostic lines from endpoint calls. stdout contains normal output.
- **Exit:** 0
- **Source:** [param/023_trace.md](../../../../docs/cli/param/023_trace.md)

---

### IT-5: `refresh::0` skips token refresh on expired credentials

- **Given:** Active account with expired `expiresAt` timestamp
- **When:** `clp .account.inspect refresh::0`
- **Then:** Exits 0. No subprocess spawned for token refresh. Command proceeds with locally cached credentials.
- **Exit:** 0
- **Source:** [param/019_refresh.md](../../../../docs/cli/param/019_refresh.md)

---

### IT-6: Unknown account name exits 2

- **Given:** `nobody@example.com` is not in the credential store
- **When:** `clp .account.inspect name::nobody@example.com`
- **Then:** Exits 2. stderr indicates account not found.
- **Exit:** 2
- **Source:** [param/001_name.md](../../../../docs/cli/param/001_name.md)

---

### IT-7: Unknown parameter rejected → exit 1

- **Given:** Any account state
- **When:** `clp .account.inspect unknown::x`
- **Then:** Exits 1. stderr indicates unrecognized parameter.
- **Exit:** 1
- **Source:** [invariant/006_param_defaults.md](../../../../docs/invariant/006_param_defaults.md)

---

### IT-8: Multi-membership display marks `← selected`

- **Given:** Active account has two memberships: one with `billing_type=none`, one with `billing_type=stripe_subscription`
- **When:** `clp .account.inspect`
- **Then:** Exits 0. stdout contains `Memberships: 2`. The selected membership entry ends with `← selected`. The other entry does not.
- **Exit:** 0
- **Source:** [command/001_account.md](../../../../docs/cli/command/001_account.md)

---

### IT-9: Single membership display has no selection marker

- **Given:** Active account has exactly one membership
- **When:** `clp .account.inspect`
- **Then:** Exits 0. stdout does not contain the string `← selected`.
- **Exit:** 0
- **Source:** [command/001_account.md](../../../../docs/cli/command/001_account.md)

---

### IT-10: Prefix resolves to full account name

- **Given:** Account `alice@acme.com` in credential store
- **When:** `clp .account.inspect name::alice`
- **Then:** Exits 0. stdout contains `Account: alice@acme.com` (full email resolved from prefix).
- **Exit:** 0
- **Source:** [param/001_name.md](../../../../docs/cli/param/001_name.md)

---

### IT-11: `format::json` membership array length matches text output count

- **Given:** Active account with two memberships
- **When:** `clp .account.inspect format::json | jq '.memberships | length'`
- **Then:** Exits 0. Output is `2`.
- **Exit:** 0
- **Source:** [param/002_format.md](../../../../docs/cli/param/002_format.md)

---

### IT-12: `refresh::1` (default) attempts token refresh when locally expired

- **Given:** Active account with expired `expiresAt` timestamp
- **When:** `clp .account.inspect refresh::1`
- **Then:** Exits 0. Refresh subprocess is spawned. Output is present (uses refreshed credentials).
- **Exit:** 0
- **Source:** [param/019_refresh.md](../../../../docs/cli/param/019_refresh.md)

---

### IT-13: Output includes `Name:` and `Email:` from endpoint 002

- **Given:** Active account; endpoint 002 returns `full_name`, `display_name`, `email_address`
- **When:** `clp .account.inspect`
- **Then:** Exits 0. stdout contains `Name:` and `Email:` fields populated from endpoint 002 response.
- **Exit:** 0
- **Source:** [command/001_account.md](../../../../docs/cli/command/001_account.md)

---

### IT-14: Output includes `Capabilities:` and `Tier:` from selected membership

- **Given:** Active account; endpoint 002 returns membership with capabilities and `rate_limit_tier`
- **When:** `clp .account.inspect`
- **Then:** Exits 0. stdout contains `Capabilities:` and `Tier:` fields from the selected membership.
- **Exit:** 0
- **Source:** [command/001_account.md](../../../../docs/cli/command/001_account.md)

---

### IT-15: Output includes usage data from endpoint 001

- **Given:** Active account; endpoint 001 returns utilization data
- **When:** `clp .account.inspect`
- **Then:** Exits 0. stdout contains `Session (5h):`, `Weekly (7d):`, `Sonnet (7d):` fields from endpoint 001.
- **Exit:** 0
- **Source:** [command/001_account.md](../../../../docs/cli/command/001_account.md)

---

### IT-16: `format::json` includes extended fields

- **Given:** Active account; endpoints return full data
- **When:** `clp .account.inspect format::json`
- **Then:** Exits 0. JSON output contains `email_address`, `full_name`, `capabilities`, `rate_limit_tier`, `session_5h_pct`, `weekly_7d_pct`, `sonnet_7d_pct` fields.
- **Exit:** 0
- **Source:** [param/002_format.md](../../../../docs/cli/param/002_format.md)

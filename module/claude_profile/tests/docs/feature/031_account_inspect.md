# Test: Feature 031 — Account Inspect

### Scope

- **Purpose**: Test cases for `.account.inspect` identity, membership selection, org fields, and endpoint fallback behavior.
- **Source**: `docs/feature/031_account_inspect.md`
- **Covers**: AC-01 through AC-25

Feature behavioral requirement test cases for `docs/feature/031_account_inspect.md` (FR-31). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Active account shows identity fields from endpoint 002 | AC-01 |
| FT-02 | All memberships shown with index, billing_type, has_max, capabilities | AC-02 |
| FT-03 | Multi-membership: selected marker on highest-priority membership | AC-03 |
| FT-04 | Single-membership: no selected marker | AC-04 |
| FT-05 | Org fields shown from endpoint 005 | AC-05 |
| FT-06 | Billing and Has Max taken from selected membership (not index 0) | AC-06 |
| FT-07 | Endpoint 002 failure falls back to snapshot for Billing/Has Max | AC-07 |
| FT-08 | Endpoint 002 failure falls back to snapshot for Tagged ID/UUID | AC-08 |
| FT-09 | Endpoint 005 failure falls back to snapshot for org fields | AC-09 |
| FT-10 | refresh::1 (default): locally-expired token triggers refresh attempt | AC-10 |
| FT-11 | refresh::0: locally-expired token NOT refreshed; all endpoints get stale token | AC-11 |
| FT-12 | name:: resolved by AccountSelector; invalid name exits 2 | AC-12 |
| FT-13 | format::json includes all required fields | AC-13 |
| FT-14 | trace::1 emits timestamped diagnostic lines per endpoint | AC-14 |
| FT-15 | No credential store exits 2 | AC-15 |
| FT-16 | Priority 2 selection: stripe_subscription (no claude_max) preferred over none | AC-03, AC-06 |
| FT-17 | Priority 3 fallback: all none memberships → memberships[0] selected | AC-03, AC-06 |
| FT-18 | Credential file absent exits 2 | AC-16 |
| FT-19 | Enterprise workspace fields shown | AC-17 |
| FT-20 | Unicode account name (IDN email) resolves via full email lookup | AC-12 |
| FT-21 | Empty credentials file (0 bytes) shows unknown status, exits 0 | AC-18 |
| FT-22 | Malformed credentials JSON (missing `oauthAccount`) shows unknown status, exits 0 | AC-19 |
| FT-23 | `format` parameter is case-sensitive — uppercase `JSON` rejected, exits 1 | AC-13 |
| FT-24 | Token with `expiresAt=0` (Unix epoch) shows status "expired", exits 0 | AC-01 |
| FT-25 | Name and Email fields shown from endpoint 002 | AC-20 |
| FT-26 | Name field omitted when full_name and display_name are empty | AC-20 |
| FT-27 | Capabilities and Tier fields from selected membership | AC-21 |
| FT-28 | Usage data shown when endpoint 001 available | AC-22 |
| FT-29 | Usage section omitted when endpoint 001 unavailable | AC-23 |
| FT-30 | JSON output includes usage and identity extension fields | AC-24 |
| FT-31 | Identity fields sourced from endpoint 002, not fabricated userinfo (BUG-295) | AC-25 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Active account shows Account, Status, Tagged ID, UUID from endpoint 002 | AC-01 | Identity |
| FT-02 | All memberships shown with index, billing_type, has_max, capabilities | AC-02 | Memberships |
| FT-03 | Multi-membership selected marker on stripe_subscription+claude_max membership | AC-03 | Memberships |
| FT-04 | Single-membership shows no selected marker | AC-04 | Memberships |
| FT-05 | Org, Org UUID, Org Role, Workspace fields from endpoint 005 | AC-05 | Org Identity |
| FT-06 | Billing and Has Max from priority-selected membership, not memberships[0] | AC-06 | Selection Priority |
| FT-07 | Endpoint 002 failure: Memberships shows error; Billing falls back with (snapshot) | AC-07 | Endpoint Fallback |
| FT-08 | Endpoint 002 failure: Tagged ID and UUID fall back with (snapshot) | AC-08 | Endpoint Fallback |
| FT-09 | Endpoint 005 failure: org fields fall back with (snapshot) | AC-09 | Endpoint Fallback |
| FT-10 | Locally-expired token with refresh::1 triggers refresh_account_token() | AC-10 | Token Refresh |
| FT-11 | Locally-expired token with refresh::0: all endpoints fail; full snapshot fallback | AC-11 | Token Refresh |
| FT-12 | name::prefix resolves to account; unknown name exits 2 | AC-12 | Name Resolution |
| FT-13 | format::json includes memberships array with selected field | AC-13 | JSON Format |
| FT-14 | trace::1 emits timestamped diagnostic endpoint lines to stderr | AC-14 | Trace |
| FT-15 | No credential store exits 2 | AC-15 | Error Handling |
| FT-16 | Priority 2 selection: stripe_subscription without claude_max preferred over none | AC-03, AC-06 | Selection Priority |
| FT-17 | Priority 3 fallback: all none memberships selects memberships[0] | AC-03, AC-06 | Selection Priority |
| FT-18 | Credential file absent exits 2 | AC-16 | Error Handling |
| FT-19 | Enterprise workspace fields shown | AC-17 | Org Identity |
| FT-20 | Unicode account name (IDN email) resolves via full email lookup | AC-12 | Name Resolution |
| FT-21 | Empty credentials file (0 bytes) shows unknown status, exits 0 | AC-18 | Error Handling |
| FT-22 | Malformed credentials JSON (missing `oauthAccount`) shows unknown status, exits 0 | AC-19 | Error Handling |
| FT-23 | `format` parameter is case-sensitive — uppercase `JSON` rejected, exits 1 | AC-13 | Format |
| FT-24 | Token with `expiresAt=0` (Unix epoch) shows status "expired", exits 0 | AC-01 | Status |
| FT-25 | Name and Email from endpoint 002 with differing full_name/display_name | AC-20 | Identity |
| FT-26 | Name field omitted when full_name and display_name empty | AC-20 | Identity |
| FT-27 | Capabilities and Tier from selected membership | AC-21 | Subscription |
| FT-28 | Usage data (5h/7d/Sonnet) shown when endpoint 001 available | AC-22 | Usage |
| FT-29 | Usage section omitted when endpoint 001 unavailable | AC-23 | Usage |
| FT-30 | JSON includes email, name, capabilities, tier, usage fields | AC-24 | JSON Format |
| FT-31 | Identity from endpoint 002, not fabricated userinfo (BUG-295) | AC-25 | Identity |

**Total:** 31 FT cases

---

### FT-01: Active account shows Account, Status, Tagged ID, UUID from endpoint 002

- **Given:** An active account `live@test.com` with a real live access token (via `live_active_token()`).
- **When:** `clp .account.inspect` (no name:: — uses active account)
- **Then:** Output contains the `Account:`, `Status:`, `Tagged ID:`, and `UUID:` labels; `Tagged ID:` is not `N/A` (endpoint 002 succeeded).
- **Exit:** 0
- **Live:** yes
- **Note:** The cited test hits the real Anthropic API, so it cannot assert specific fabricated values like `tagged_id: "user_01abc"` or `Account: alice@acme.com` — it only checks label presence and that `Tagged ID` isn't the `N/A` fallback. Runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `lim_it_ai14_identity_fields_from_endpoint_002`
- **Source:** [031_account_inspect.md AC-01](../../../docs/feature/031_account_inspect.md)

---

### FT-02: All memberships shown with index, billing_type, has_max, capabilities

- **Given:** An account `live@test.com` with a real live access token; whatever membership data the live account actually has.
- **When:** `clp .account.inspect`
- **Then:** Output shows a `Memberships:` line whose value parses as a number.
- **Exit:** 0
- **Live:** yes
- **Note:** The cited test cannot control or predict the live account's real membership count or per-membership fields — it only asserts the `Memberships:` label is present and its value is numeric, not the specific `[0]`/`[1]` index/billing_type/has_max/capabilities values this FT case originally claimed. Runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `lim_it_ai15_memberships_shown_with_count`
- **Source:** [031_account_inspect.md AC-02](../../../docs/feature/031_account_inspect.md)

---

### FT-03: Multi-membership selected marker on stripe_subscription+claude_max membership

- **Given:** Account `live@test.com` with a real live access token and whatever real membership data it has.
- **When:** `clp .account.inspect`
- **Then:** If the parsed `Memberships:` count is greater than 1, output contains exactly one `← selected` marker.
- **Exit:** 0
- **Live:** yes
- **Note:** The cited test branches on the live account's actual membership count at runtime — it cannot force a specific two-membership `[0]=none`/`[1]=stripe_subscription+claude_max` scenario, and does not check which specific index carries the marker or its `billing_type`/`has_max` values. The controlled priority-selection scenario this FT case describes is verified precisely by the offline unit test `mre_bug237_multi_membership_selects_stripe_max_over_none` in `claude_quota/src/lib.rs` instead (`billing_type="stripe_subscription"`, `has_max=true` selected over `billing_type="none"`). Runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `lim_it_ai16_selected_marker_multi_membership` (live, structural marker-count check); `mre_bug237_multi_membership_selects_stripe_max_over_none` (offline unit test, in `claude_quota/src/lib.rs`, exact priority scenario)
- **Source:** [031_account_inspect.md AC-03, AC-06](../../../docs/feature/031_account_inspect.md)

---

### FT-04: Single-membership shows no selected marker

- **Given:** Account `live@test.com` with a real live access token and whatever real membership data it has.
- **When:** `clp .account.inspect`
- **Then:** If the parsed `Memberships:` count is 1 or fewer, output contains zero `← selected` markers.
- **Exit:** 0
- **Live:** yes
- **Note:** `lim_it_ai16_selected_marker_multi_membership` is the same single test function cited for FT-03 — it branches at runtime on whichever count the live account actually has, rather than running FT-03's and FT-04's scenarios as two independently-controlled fixtures. The controlled single-membership scenario this FT case describes is verified precisely by the offline unit test `mre_bug237_single_membership_fallback_unchanged` in `claude_quota/src/lib.rs` instead. Runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `lim_it_ai16_selected_marker_multi_membership` (live, structural marker-count check); `mre_bug237_single_membership_fallback_unchanged` (offline unit test, in `claude_quota/src/lib.rs`)
- **Source:** [031_account_inspect.md AC-04](../../../docs/feature/031_account_inspect.md)

---

### FT-05: Org, Org UUID, Org Role, Workspace fields from endpoint 005

- **Given:** Account `live@test.com` with a real live access token; whatever real org/workspace data endpoint 005 returns for it.
- **When:** `clp .account.inspect`
- **Then:** Output contains the `Org:`, `Org UUID:`, `Org Role:`, `Workspace UUID:`, and `Workspace:` labels.
- **Exit:** 0
- **Live:** yes
- **Note:** The cited test only checks that all five labels are present — it does not assert specific values (`alice's Org`, `admin`, etc.) or the `(none)` rendering for null workspace fields, since it cannot control what the real live org/workspace data is. Runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `lim_it_ai17_org_fields_from_endpoint_005`
- **Source:** [031_account_inspect.md AC-05](../../../docs/feature/031_account_inspect.md)

---

### FT-06: Billing and Has Max from priority-selected membership, not memberships[0]

- **Given:** Account `live@test.com` with a real live access token.
- **When:** `clp .account.inspect`
- **Then:** Output contains the `Billing:` and `Has Max:` labels; `Billing:` is not the `N/A`-padded placeholder (`"Billing:         N/A"`).
- **Exit:** 0
- **Live:** yes
- **Note:** The cited test cannot control the live account's real billing type, so it does not assert `Billing: stripe_subscription` or `Has Max: yes` specifically — only label presence and non-`N/A`. The controlled priority-selection scenario (index 1's `stripe_subscription+claude_max` beating index 0's `none`) is verified precisely by the offline unit test `mre_bug237_multi_membership_selects_stripe_max_over_none` in `claude_quota/src/lib.rs`. Runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `lim_it_ai18_billing_from_selected_membership` (live, label presence); `mre_bug237_multi_membership_selects_stripe_max_over_none` (offline unit test, in `claude_quota/src/lib.rs`, exact priority scenario)
- **Source:** [031_account_inspect.md AC-06](../../../docs/feature/031_account_inspect.md)

---

### FT-07: Endpoint 002 failure: Memberships shows error; Billing falls back with (snapshot)

- **Given:** An account with NO `accessToken` at all (offline fixture — all three endpoints fail uniformly with "no token", not a live token hitting a network error specifically on endpoint 002); `{name}.json` snapshot exists with `billing_type: "stripe_subscription"`.
- **When:** `clp .account.inspect`
- **Then:** `Memberships:` line contains `endpoint unavailable`; `Billing: stripe_subscription (snapshot)`; `Has Max: yes (snapshot)`. Exit 0.
- **Exit:** 0
- **Source fn:** `ai10_memberships_endpoint_unavailable_message`, `ai09_snapshot_all_fields_when_no_token`
- **Source:** [031_account_inspect.md AC-07](../../../docs/feature/031_account_inspect.md)

---

### FT-08: Endpoint 002 failure: Tagged ID and UUID fall back with (snapshot)

- **Given:** An account with NO `accessToken` at all (offline fixture — all three endpoints fail uniformly with "no token", not a live token hitting HTTP 500 specifically on endpoint 002); `{name}.json` snapshot contains `tagged_id: "user_01abc"` and `uuid: "aaaa-bbbb"`.
- **When:** `clp .account.inspect`
- **Then:** `Tagged ID: user_01abc (snapshot)`; `UUID: aaaa-bbbb (snapshot)`. Exit 0.
- **Exit:** 0
- **Note:** Because no `accessToken` is present, endpoints 005 and 001 also fail — they do NOT show live data as this FT case originally claimed; the cited test only asserts the identity/billing/org fields shown, all with `(snapshot)` suffix.
- **Source fn:** `ai09_snapshot_all_fields_when_no_token`
- **Source:** [031_account_inspect.md AC-08](../../../docs/feature/031_account_inspect.md)

---

### FT-09: Endpoint 005 failure: org fields fall back with (snapshot)

- **Given:** An account with NO `accessToken` at all (offline fixture — all three endpoints fail uniformly with "no token", not a live token hitting HTTP 403 specifically on endpoint 005); `{name}.json` snapshot contains `organization_name: "alice's Org"`, `organization_uuid: "org-uuid-1"`, `organization_role: "admin"`.
- **When:** `clp .account.inspect`
- **Then:** `Org: alice's Org (snapshot)`; `Org UUID: org-uuid-1 (snapshot)`; `Org Role: admin (snapshot)`. Exit 0.
- **Exit:** 0
- **Note:** Because no `accessToken` is present, endpoints 001 and 002 also fail — they do NOT show live data as this FT case originally claimed; the cited test asserts identity, billing, and org fields all fall back to `(snapshot)` together.
- **Source fn:** `ai09_snapshot_all_fields_when_no_token`
- **Source:** [031_account_inspect.md AC-09](../../../docs/feature/031_account_inspect.md)

---

### FT-10: Locally-expired token with refresh::1 triggers a refresh attempt

- **Given:** An account whose `expiresAt` in `{name}.credentials.json` is in the past, with a deliberately fake `accessToken` (`"fake_refresh_token"`) so the refresh attempt fails against the real OAuth endpoint (returns HTTP 400).
- **When:** `clp .account.inspect` (default: `refresh::1`)
- **Then:** `attempt_expired_token_refresh()` is attempted (per the `is_expired && refresh != 0` guard in `src/commands/account_inspect.rs`); it fails; the command does NOT panic or exit non-zero — `Status:` still shows `expired`. Exit 0.
- **Exit:** 0
- **Live:** yes
- **Note:** This FT case previously claimed the refresh *succeeds* and `Status:` becomes `valid` — the opposite of what the cited test actually verifies. The cited test deliberately uses a fake token specifically so the refresh **fails**, to confirm graceful degradation (no panic, exit 0, status stays `expired`) rather than a successful refresh round-trip. Runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `lim_it_ai20_refresh_attempted_on_expired_token`
- **Source:** [031_account_inspect.md AC-10](../../../docs/feature/031_account_inspect.md)

---

### FT-11: Locally-expired token with refresh::0: full snapshot fallback

- **Given:** An account whose `expiresAt` is in the past; `refresh::0` is specified.
- **When:** `clp .account.inspect refresh::0`
- **Then:** `Status: 🔴 expired (Xh Ym ago)`; `Memberships: endpoint unavailable (auth error)`; all fields show `(snapshot)` suffix or `N/A` if no snapshot. No `refresh_account_token()` call. Exit 0.
- **Exit:** 0
- **Source fn:** `ai08_expired_token_shows_expired_status`, `ai09_snapshot_all_fields_when_no_token`, `ai10_memberships_endpoint_unavailable_message`
- **Source:** [031_account_inspect.md AC-11](../../../docs/feature/031_account_inspect.md)

---

### FT-12: name::prefix resolves to account; unknown name exits 2

- **Given:** Credential store contains `alice@acme.com.credentials.json`; `name::alice` resolves by prefix.
- **When:** `clp .account.inspect name::alice`
- **Then:** Output shows `Account: alice@acme.com`. Exit 0.
- **And When:** `clp .account.inspect name::nobody` (bare prefix, no domain — matches no saved account)
- **Then:** Exit 2 with stderr containing `not found`.
- **Exit:** 0 / 2
- **Note:** The unknown-account case originally cited `name::nobody@acme.com` (a full email); the actual test `ai02_account_not_found_exits_2` uses the bare prefix `name::nobody` instead, and checks stderr for the generic substring `not found` rather than the exact message `account not found: nobody@acme.com`. `ai03`/`ai04`/`ai06` cover three further AC-12 sub-scenarios (empty `name::` → exit 1; no `name::` and no active marker → exit 2; no `name::` with an active marker → resolves via the marker) not individually narrated above.
- **Source fn:** `ai07_prefix_name_resolves`, `ai02_account_not_found_exits_2`, `ai03_empty_name_exits_1`, `ai04_no_active_account_exits_2`, `ai06_active_marker_used_when_no_name`
- **Source:** [031_account_inspect.md AC-12](../../../docs/feature/031_account_inspect.md)

---

### FT-13: format::json includes memberships array with selected field

- **Given:** An account with no `accessToken` (offline fixture, `ai11`/`ai12`); an invalid `format::` value (`ai05`); an uppercase `format::JSON` value (`ai30`); separately, a `live@test.com` account with a real live token (`lim_it_ai19`).
- **When:** `clp .account.inspect format::json` (`ai11`, `ai12`, `lim_it_ai19`); `clp .account.inspect format::csv` (`ai05`, exit 1); `clp .account.inspect format::JSON` (`ai30`, exit 1).
- **Then:** `ai11` — JSON output contains the top-level field names `account`, `status`, `expires_in_secs`, `tagged_id`, `uuid`, `email_address`, `full_name`, `display_name`, `memberships`, `billing_type`, `has_max`, `capabilities`, `rate_limit_tier`, `session_5h_pct`/`_reset_ts`, `weekly_7d_pct`/`_reset_ts`, `sonnet_7d_pct`/`_reset_ts`, `organization_name`, `organization_uuid`, `organization_role`, `workspace_uuid`, `workspace_name`, `data_source`. `ai12` — `data_source` is `"snapshot"` when all endpoints fail. `lim_it_ai19` — `data_source` is `"live"` or `"partial_snapshot"` and `status` is `"valid"` with a real token. Exit 0 for all three; exit 1 for `ai05`/`ai30`.
- **Exit:** 0 / 1
- **Live:** partial (`lim_it_ai19` only)
- **Note:** This FT case previously claimed a controlled two-membership scenario with per-element `index`/`billing_type`/`has_max`/`capabilities`/`selected` keys and `"selected": true`/`false` values — no cited test verifies the `memberships` array's internal per-element structure or a specific `selected` boolean; `ai11` only confirms the top-level field NAMES are present in the JSON schema (the account here has no memberships data at all, since it has no token). `lim_it_ai19` runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `ai11_json_all_required_fields`, `ai12_json_data_source_snapshot_when_all_fail`, `ai05_format_invalid_exits_1`, `ai30_format_case_sensitive_uppercase_exits_1`, `lim_it_ai19_valid_token_live_data_source_json`
- **Source:** [031_account_inspect.md AC-13](../../../docs/feature/031_account_inspect.md)

---

### FT-14: trace::1 emits timestamped diagnostic endpoint lines to stderr

- **Given:** An account with no `accessToken` (offline, `ai13`); separately, a `live@test.com` account with a real live token (`lim_it_ai21`).
- **When:** `clp .account.inspect trace::1` (stderr captured)
- **Then:** `ai13` — stderr contains at least one ` · `-formatted trace line, including a status line. `lim_it_ai21` — stderr contains at least three ` · `-formatted trace lines (one per endpoint).
- **Exit:** 0
- **Live:** partial (`lim_it_ai21` only)
- **Note:** The "at least three, one per endpoint, each showing URL and HTTP status" claim is verified only by the live test `lim_it_ai21` (which counts `>= 3` lines containing ` · `, without checking each shows a URL/HTTP-status pair specifically); the offline `ai13` only checks for a single generic trace-format line plus the word "status". `lim_it_ai21` runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `ai13_trace_emits_lines_to_stderr`, `lim_it_ai21_trace_endpoint_lines_on_live_account`
- **Source:** [031_account_inspect.md AC-14](../../../docs/feature/031_account_inspect.md)

---

### FT-15: No credential store exits 2

- **Given:** No credential store directory exists (fresh `$HOME`, no `.persistent/claude/credential/`).
- **When:** `clp .account.inspect name::alice@acme.com` (full email bypasses store lookup)
- **Then:** Exit 2 with `credential file not found: {path}`. The absent store is treated identically to an absent credential file — no distinct store-not-found branch exists.
- **Exit:** 2
- **Source fn:** `ai22_credential_store_absent_exits_2`
- **Source:** [031_account_inspect.md AC-15](../../../docs/feature/031_account_inspect.md)

---

### FT-16: Priority 2 selection: stripe_subscription without claude_max preferred over none

- **Given:** An account whose endpoint 002 response contains two memberships: `[0] billing_type=none, has_max=false, capabilities=[chat]` and `[1] billing_type=stripe_subscription, has_max=false, capabilities=[chat]` (no `claude_max` in either).
- **When:** `clp .account.inspect`
- **Then:** Output shows `Memberships: 2`; membership [1] is marked `← selected` (stripe_subscription beats none even without claude_max); `Billing: stripe_subscription`; membership [0] is unmarked.
- **Exit:** 0
- **Source fn:** `lim_it_ai16_selected_marker_multi_membership` (marker count and single-vs-multi branching); priority rule verified by `mre_bug237_multi_membership_selects_stripe_over_none_no_max` in `claude_quota` crate
- **Source:** [031_account_inspect.md AC-03, AC-06](../../../docs/feature/031_account_inspect.md)

---

### FT-17: Priority 3 fallback: all none memberships selects memberships[0]

- **Given:** An account whose endpoint 002 response contains two memberships: `[0] billing_type=none, capabilities=[chat]` and `[1] billing_type=none, capabilities=[chat]`.
- **When:** `clp .account.inspect`
- **Then:** Output shows `Memberships: 2`; membership [0] is marked `← selected` (Priority 3 fallback applies — no stripe_subscription in either; `memberships[0]` is the fallback); membership [1] is unmarked; `Billing: none`.
- **Exit:** 0
- **Source fn:** `lim_it_ai16_selected_marker_multi_membership` (marker count and single-vs-multi branching); fallback rule verified by `mre_bug237_single_membership_fallback_unchanged` in `claude_quota` crate
- **Source:** [031_account_inspect.md AC-03, AC-06](../../../docs/feature/031_account_inspect.md)

---

### FT-18: Credential file absent exits 2

- **Given:** Credential store directory exists (`{credential_store}/`) but `alice@acme.com.credentials.json` is absent.
- **When:** `clp .account.inspect name::alice@acme.com`
- **Then:** Exit 2 with `credential file not found: {path}`. Unlike AC-15, the store directory is present; the credential file for the specific account is simply missing.
- **Exit:** 2
- **Source fn:** `ai01_credential_file_absent_exits_2`
- **Source:** [031_account_inspect.md AC-16](../../../docs/feature/031_account_inspect.md)

---

### FT-19: Enterprise workspace fields show non-none values

- **Given:** An account whose endpoint 005 response includes non-null `workspace_uuid` and `workspace_name` (enterprise account with a named workspace).
- **When:** `clp .account.inspect`
- **Then:** `Workspace UUID:` shows the UUID string (not `(none)`); `Workspace:` shows the workspace name string (not `(none)`). In `format::json`, `workspace_uuid` and `workspace_name` contain the raw string values.
- **Exit:** 0
- **Source fn:** `ai23_workspace_fields_show_values_when_non_null`
- **Source:** [031_account_inspect.md AC-17](../../../docs/feature/031_account_inspect.md)

---

### FT-20: Unicode account name (IDN email) resolves via full email lookup

- **Given:** An account named `alice@münchen.de` is registered via `write_account()` — the credentials file `alice@münchen.de.credentials.json` is present in the credential store (UTF-8 Linux filesystem).
- **When:** `clp .account.inspect name::alice@münchen.de refresh::0`
- **Then:** Exit 0; output contains `alice@münchen.de`. The AccountSelector performs a full email match; the unicode byte sequence in the filename survives the round-trip unchanged on a UTF-8 filesystem.
- **Exit:** 0
- **Source fn:** `ai27_unicode_account_name_resolves`
- **Source:** [031_account_inspect.md AC-12](../../../docs/feature/031_account_inspect.md)

---

### FT-21: Empty credentials file (0 bytes) shows unknown status, exits 0

- **Given:** The credentials file `u@test.com.credentials.json` exists in the credential store but contains 0 bytes (empty file — simulates a truncated write or disk error).
- **When (text):** `clp .account.inspect name::u@test.com refresh::0`
- **When (JSON):** `clp .account.inspect name::u@test.com refresh::0 format::json`
- **Then (text):** Exit 0; output contains `unknown`.
- **Then (JSON):** Exit 0; JSON output contains `"status":"unknown"`.
- **Note:** Distinct from FT-18 (absent file → exits 2). An existing-but-empty file passes the file-existence check; the JSON parse failure produces an unknown status rather than a hard error exit.
- **Exit:** 0
- **Source fn:** `ai28_empty_credentials_file_shows_unknown_status`
- **Source:** [031_account_inspect.md AC-18](../../../docs/feature/031_account_inspect.md)

---

### FT-22: Malformed credentials JSON (missing `oauthAccount`) shows unknown status, exits 0

- **Given:** The credentials file `u@test.com.credentials.json` contains valid JSON `{"version":"2","data":{}}` — parseable but lacks the `oauthAccount` key, so `expiresAt` is absent.
- **When (text):** `clp .account.inspect name::u@test.com refresh::0`
- **When (JSON):** `clp .account.inspect name::u@test.com refresh::0 format::json`
- **Then (text):** Exit 0; output contains `unknown`.
- **Then (JSON):** Exit 0; JSON output contains `"status":"unknown"`.
- **Note:** Simulates a version-mismatch schema written by an older tool. Graceful degradation (unknown status, exit 0) is required; panicking or exiting non-zero is a regression.
- **Exit:** 0
- **Source fn:** `ai29_malformed_credentials_json_shows_unknown_status`
- **Source:** [031_account_inspect.md AC-19](../../../docs/feature/031_account_inspect.md)

---

### FT-23: `format` parameter is case-sensitive — uppercase `JSON` rejected, exits 1

- **Given:** A valid account `alice@acme.com` exists in the credential store.
- **When:** `clp .account.inspect name::alice@acme.com format::JSON`
- **Then:** Exit 1; stderr contains the invalid value or a reference to the `format` parameter.
- **Note:** The parameter validator only accepts lowercase `"text"` and `"json"`. Case variants (`"JSON"`, `"Text"`) must be rejected, not silently accepted or mapped. This prevents silent fallback masking a user typo.
- **Exit:** 1
- **Source fn:** `ai30_format_case_sensitive_uppercase_exits_1`
- **Source:** [031_account_inspect.md AC-13](../../../docs/feature/031_account_inspect.md)

---

### FT-24: Token with `expiresAt=0` (Unix epoch) shows status "expired", exits 0

- **Given:** `u@test.com.credentials.json` contains `{"oauthAccount":{"expiresAt":0,...}}` — `expiresAt` is present and zero (Unix epoch, 1970-01-01).
- **When:** `clp .account.inspect name::u@test.com refresh::0`
- **Then:** Exit 0; output contains `expired`. Output must NOT contain `unknown`.
- **Note:** `expiresAt=0` is a valid timestamp (parseable integer). It must produce `"expired"`, not `"unknown"` (which is reserved for missing or unparseable `expiresAt`). This is the lower boundary of the expiry parser.
- **Exit:** 0
- **Source fn:** `ai31_expires_at_zero_shows_expired_status`
- **Source:** [031_account_inspect.md AC-01](../../../docs/feature/031_account_inspect.md)

---

### FT-25: Name and Email fields shown from endpoint 002 (live) or snapshot (offline)

- **Given (live, `lim_it_ai22`):** Account `live@test.com` with a real live access token.
- **Given (offline snapshot, `ai33`):** Account `alice@acme.com` with no `accessToken`; `{name}.json` snapshot has `full_name: "Alice Smith"`, `display_name: "Alice"`.
- **Given (offline snapshot, `ai34`):** Account `bob@test.com` with no `accessToken`; `{name}.json` snapshot has `full_name: "Robert Smith"`, `display_name: "Bob"` (different from each other, to exercise the `"FullName (DisplayName)"` format).
- **When:** `clp .account.inspect`
- **Then (`lim_it_ai22`):** `Email:` label present, without `(snapshot)` suffix; if a `Name:` line is present, it also lacks `(snapshot)`.
- **Then (`ai33`):** `Name:` line contains `Alice Smith` and `(snapshot)`; `Email:` line contains `alice@acme.com` and `(snapshot)`.
- **Then (`ai34`):** Output contains exactly `Robert Smith (Bob)`.
- **Exit:** 0
- **Live:** partial (`lim_it_ai22` only)
- **Note:** This FT case originally described a single live-endpoint-002-success scenario with one fixed set of values (`Alice Smith`/`Alice`/`alice@acme.com`) verified by all three cited functions together. In fact `ai33` and `ai34` exercise the offline **snapshot fallback** path (endpoint 002 unreachable, no token) — the opposite premise — and `ai34` uses different literal values (`Robert Smith`/`Bob`/`bob@test.com`) to isolate the `"FullName (DisplayName)"` format specifically. `lim_it_ai22` is the only citation that actually reaches live endpoint 002, and it only checks label presence and absence of `(snapshot)`, not the specific `Alice Smith (Alice)` value. `lim_it_ai22` runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `lim_it_ai22_name_and_email_from_endpoint_002`, `ai33_name_email_from_snapshot`, `ai34_name_shows_display_name_when_different`
- **Source:** [031_account_inspect.md AC-20](../../../docs/feature/031_account_inspect.md)

---

### FT-26: Name field omitted when full_name and display_name are empty

- **Given:** An active account whose endpoint 002 response has `full_name: ""`, `display_name: ""`, `email_address: "bob@test.com"`.
- **When:** `clp .account.inspect`
- **Then:** Output does NOT contain `Name:` line. `Email: bob@test.com` is shown. Exit 0.
- **Exit:** 0
- **Source fn:** `ai36_name_omitted_when_names_empty`
- **Source:** [031_account_inspect.md AC-20](../../../docs/feature/031_account_inspect.md)

---

### FT-27: Capabilities and Tier fields from selected membership

- **Given:** Account `live@test.com` with a real live access token.
- **When:** `clp .account.inspect`
- **Then:** Output contains `Capabilities:` with a `[`-bracketed value, and the `Tier:` label.
- **Exit:** 0
- **Live:** yes
- **Note:** The cited test cannot control the live account's real capabilities/tier, so it does not assert the specific `[claude_max, chat]` / `default_claude_max_20x` values this FT case originally claimed — only that `Capabilities:` uses array-bracket formatting and `Tier:` is present. Runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `lim_it_ai23_capabilities_and_tier_from_membership`
- **Source:** [031_account_inspect.md AC-21](../../../docs/feature/031_account_inspect.md)

---

### FT-28: Usage data shown when endpoint 001 available

- **Given:** Account `live@test.com` with a real live access token.
- **When:** `clp .account.inspect`
- **Then:** Output contains a `Session (5h):` line containing at least one digit.
- **Exit:** 0
- **Live:** yes
- **Note:** The cited test cannot control the live account's real usage figures, so it does not assert the specific `45%`/`33%`/`53%` values this FT case originally claimed, and only checks the `Session (5h):` line — it does not check `Weekly (7d):` or `Sonnet (7d):` at all. Runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `lim_it_ai24_usage_data_from_endpoint_001`
- **Source:** [031_account_inspect.md AC-22](../../../docs/feature/031_account_inspect.md)

---

### FT-29: Usage section omitted when endpoint 001 unavailable

- **Given:** An account with NO `accessToken` at all (offline fixture — all three endpoints fail uniformly with "no token", not a live token hitting HTTP 500/timeout specifically on endpoint 001).
- **When:** `clp .account.inspect`
- **Then:** Output does NOT contain `Session (5h):`, `Weekly (7d):`, or `Sonnet (7d):` lines. Exit 0.
- **Exit:** 0
- **Source fn:** `ai32_usage_absent_when_offline`
- **Source:** [031_account_inspect.md AC-23](../../../docs/feature/031_account_inspect.md)

---

### FT-30: JSON output includes usage and identity extension fields

- **Given:** An account with no `accessToken` at all (offline fixture — the cited test verifies the JSON schema always includes these field names, independent of whether any endpoint actually succeeds).
- **When:** `clp .account.inspect format::json`
- **Then:** JSON output contains the field names `email_address`, `full_name`, `display_name`, `capabilities`, `rate_limit_tier`, `session_5h_pct`, `session_5h_reset_ts`, `weekly_7d_pct`, `weekly_7d_reset_ts`, `sonnet_7d_pct`, `sonnet_7d_reset_ts`. Exit 0.
- **Exit:** 0
- **Note:** This FT case originally claimed "all three endpoints succeed" — the cited test actually uses an account with no token at all (all endpoints fail), and verifies only that the JSON schema's field keys are present, not that they carry live-populated values.
- **Source fn:** `ai11_json_all_required_fields`
- **Source:** [031_account_inspect.md AC-24](../../../docs/feature/031_account_inspect.md)

---

### FT-31: Identity fields sourced from endpoint 002, not fabricated userinfo endpoint (BUG-295)

- **Given (structural, `ai35`):** The `src/commands/account_inspect.rs` source file.
- **Given (live, `lim_it_ai14`):** Account `live@test.com` with a real live access token.
- **When:** `clp .account.inspect` (`lim_it_ai14`); source-file content check (`ai35`, no CLI invocation).
- **Then:** `ai35` — `account_inspect.rs` does not contain the substring `"userinfo"` anywhere, i.e. the removed `/api/oauth/userinfo` endpoint cannot be referenced at all. `lim_it_ai14` — `Tagged ID:`/`UUID:` labels present and `Tagged ID:` is not `N/A` (endpoint 002 succeeded), but the test does not compare the displayed values against a separately-captured endpoint 002 response.
- **Exit:** 0
- **Live:** partial (`lim_it_ai14` only)
- **Note:** The `ai35` structural check is a stronger guarantee than a runtime "no HTTP request was made" assertion — the code literally cannot reference the removed endpoint string. `lim_it_ai14`'s "values match endpoint 002 response" is not literally verified (there is no independent second capture of endpoint 002's response to compare against); it only confirms the fields are populated from *some* live source, not synthesized. `lim_it_ai14` runs in-container against the live API (`~/.claude` is plugin-mounted); nextest puts it in the `live-api-serial` test group with exponential retries, and it fails loudly rather than skipping when the token is missing or the API is unreachable.
- **Source fn:** `ai35_no_userinfo_endpoint_reference`, `lim_it_ai14_identity_fields_from_endpoint_002`
- **Source:** [031_account_inspect.md AC-25](../../../docs/feature/031_account_inspect.md)

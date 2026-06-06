# Test: Feature 031 — Account Inspect

Feature behavioral requirement test cases for `docs/feature/031_account_inspect.md` (FR-31). Each FT case maps to one or more acceptance criteria.

### AC Coverage Index

| FT | Criterion | AC |
|----|-----------|-----|
| FT-01 | Active account shows identity fields from endpoint 001 | AC-01 |
| FT-02 | All memberships shown with index, billing_type, has_max, capabilities | AC-02 |
| FT-03 | Multi-membership: selected marker on highest-priority membership | AC-03 |
| FT-04 | Single-membership: no selected marker | AC-04 |
| FT-05 | Org fields shown from endpoint 005 | AC-05 |
| FT-06 | Billing and Has Max taken from selected membership (not index 0) | AC-06 |
| FT-07 | Endpoint 002 failure falls back to snapshot for Billing/Has Max | AC-07 |
| FT-08 | Endpoint 001 failure falls back to snapshot for Tagged ID/UUID | AC-08 |
| FT-09 | Endpoint 005 failure falls back to snapshot for org fields | AC-09 |
| FT-10 | refresh::1 (default): locally-expired token triggers refresh attempt | AC-10 |
| FT-11 | refresh::0: locally-expired token NOT refreshed; all endpoints get stale token | AC-11 |
| FT-12 | name:: resolved by AccountSelector; invalid name exits 2 | AC-12 |
| FT-13 | format::json includes all required fields | AC-13 |
| FT-14 | trace::1 emits [trace] lines per endpoint | AC-14 |
| FT-15 | No credential store exits 2 | AC-15 |
| FT-16 | Priority 2 selection: stripe_subscription (no claude_max) preferred over none | AC-03, AC-06 |
| FT-17 | Priority 3 fallback: all none memberships → memberships[0] selected | AC-03, AC-06 |
| FT-18 | Credential file absent exits 2 | AC-16 |
| FT-19 | Enterprise workspace fields shown | AC-17 |
| FT-20 | Unicode account name (IDN email) resolves via full email lookup | AC-12 |
| FT-21 | Empty credentials file (0 bytes) shows unknown status, exits 0 | AC-18 |
| FT-22 | Malformed credentials JSON (missing `oauthAccount`) shows unknown status, exits 0 | AC-19 |

### Test Case Index

| ID | Test Name | AC | Category |
|----|-----------|-----|----------|
| FT-01 | Active account shows Account, Status, Tagged ID, UUID from endpoint 001 | AC-01 | Identity |
| FT-02 | All memberships shown with index, billing_type, has_max, capabilities | AC-02 | Memberships |
| FT-03 | Multi-membership selected marker on stripe_subscription+claude_max membership | AC-03 | Memberships |
| FT-04 | Single-membership shows no selected marker | AC-04 | Memberships |
| FT-05 | Org, Org UUID, Org Role, Workspace fields from endpoint 005 | AC-05 | Org Identity |
| FT-06 | Billing and Has Max from priority-selected membership, not memberships[0] | AC-06 | Selection Priority |
| FT-07 | Endpoint 002 failure: Memberships shows error; Billing falls back with (snapshot) | AC-07 | Endpoint Fallback |
| FT-08 | Endpoint 001 failure: Tagged ID and UUID fall back with (snapshot) | AC-08 | Endpoint Fallback |
| FT-09 | Endpoint 005 failure: org fields fall back with (snapshot) | AC-09 | Endpoint Fallback |
| FT-10 | Locally-expired token with refresh::1 triggers refresh_account_token() | AC-10 | Token Refresh |
| FT-11 | Locally-expired token with refresh::0: all endpoints fail; full snapshot fallback | AC-11 | Token Refresh |
| FT-12 | name::prefix resolves to account; unknown name exits 2 | AC-12 | Name Resolution |
| FT-13 | format::json includes memberships array with selected field | AC-13 | JSON Format |
| FT-14 | trace::1 emits [trace] endpoint lines to stderr | AC-14 | Trace |
| FT-15 | No credential store exits 2 | AC-15 | Error Handling |
| FT-16 | Priority 2 selection: stripe_subscription without claude_max preferred over none | AC-03, AC-06 | Selection Priority |
| FT-17 | Priority 3 fallback: all none memberships selects memberships[0] | AC-03, AC-06 | Selection Priority |
| FT-18 | Credential file absent exits 2 | AC-16 | Error Handling |
| FT-19 | Enterprise workspace fields shown | AC-17 | Org Identity |
| FT-20 | Unicode account name (IDN email) resolves via full email lookup | AC-12 | Name Resolution |
| FT-21 | Empty credentials file (0 bytes) shows unknown status, exits 0 | AC-18 | Error Handling |
| FT-22 | Malformed credentials JSON (missing `oauthAccount`) shows unknown status, exits 0 | AC-19 | Error Handling |

**Total:** 22 FT cases

---

### FT-01: Active account shows Account, Status, Tagged ID, UUID from endpoint 001

- **Given:** An active account `alice@acme.com` with a valid access token (not locally expired); endpoint 001 returns `taggedId: "user_01abc"`, `uuid: "aaaa-bbbb"`.
- **When:** `clp .account.inspect` (no name:: — uses active account)
- **Then:** Output contains `Account: alice@acme.com`, `Status: 🟢 valid (expires in ...)`; `Tagged ID: user_01abc`; `UUID: aaaa-bbbb`. Exit 0.
- **Exit:** 0
- **Source fn:** `lim_it_ai14_identity_fields_from_endpoint_001`
- **Source:** [031_account_inspect.md AC-01](../../../../docs/feature/031_account_inspect.md)

---

### FT-02: All memberships shown with index, billing_type, has_max, capabilities

- **Given:** An account whose endpoint 002 response contains two membership objects: `[0] billing_type=none, has_max=false, capabilities=[chat]` and `[1] billing_type=stripe_subscription, has_max=true, capabilities=[claude_max, chat]`.
- **When:** `clp .account.inspect`
- **Then:** Output shows `Memberships: 2`; both membership lines appear with correct `[index]`, `billing_type`, `has_max`, and `capabilities` values. Exit 0.
- **Exit:** 0
- **Source fn:** `lim_it_ai15_memberships_shown_with_count`
- **Source:** [031_account_inspect.md AC-02](../../../../docs/feature/031_account_inspect.md)

---

### FT-03: Multi-membership selected marker on stripe_subscription+claude_max membership

- **Given:** Same two-membership account as FT-02.
- **When:** `clp .account.inspect`
- **Then:** The line for `[1]` (stripe_subscription + claude_max) ends with `← selected`. The line for `[0]` has no marker. `Billing: stripe_subscription` and `Has Max: yes` reflect membership [1].
- **Exit:** 0
- **Source fn:** `lim_it_ai16_selected_marker_multi_membership`
- **Source:** [031_account_inspect.md AC-03, AC-06](../../../../docs/feature/031_account_inspect.md)

---

### FT-04: Single-membership shows no selected marker

- **Given:** An account whose endpoint 002 response contains one membership: `billing_type=stripe_subscription, has_max=true, capabilities=[claude_max, chat]`.
- **When:** `clp .account.inspect`
- **Then:** Output shows `Memberships: 1`; one membership line without `← selected` marker.
- **Exit:** 0
- **Source fn:** `lim_it_ai16_selected_marker_multi_membership`
- **Source:** [031_account_inspect.md AC-04](../../../../docs/feature/031_account_inspect.md)

---

### FT-05: Org, Org UUID, Org Role, Workspace fields from endpoint 005

- **Given:** Endpoint 005 returns `organization_name: "alice's Org"`, `organization_uuid: "aaaa"`, `organization_role: "admin"`, `workspace_uuid: null`, `workspace_name: null`.
- **When:** `clp .account.inspect`
- **Then:** Output shows `Org: alice's Org`; `Org UUID: aaaa`; `Org Role: admin`; `Workspace UUID: (none)`; `Workspace: (none)`. Exit 0.
- **Exit:** 0
- **Source fn:** `lim_it_ai17_org_fields_from_endpoint_005`
- **Source:** [031_account_inspect.md AC-05](../../../../docs/feature/031_account_inspect.md)

---

### FT-06: Billing and Has Max from priority-selected membership, not memberships[0]

- **Given:** Same two-membership account as FT-02 (index 0 is `none`, index 1 is `stripe_subscription+claude_max`).
- **When:** `clp .account.inspect`
- **Then:** `Billing: stripe_subscription` (from [1], not [0]); `Has Max: yes` (from [1]). The `Billing:` field does NOT show `none`.
- **Exit:** 0
- **Source fn:** `lim_it_ai18_billing_from_selected_membership`
- **Source:** [031_account_inspect.md AC-06](../../../../docs/feature/031_account_inspect.md)

---

### FT-07: Endpoint 002 failure: Memberships shows error; Billing falls back with (snapshot)

- **Given:** An account with a valid token; endpoint 002 returns a network error; `{name}.claude.json` snapshot exists with `billing_type: "stripe_subscription"`.
- **When:** `clp .account.inspect`
- **Then:** `Memberships: endpoint unavailable (network error)` (or similar); `Billing: stripe_subscription (snapshot)`; `Has Max: yes (snapshot)`. Exit 0.
- **Exit:** 0
- **Source fn:** `ai10_memberships_endpoint_unavailable_message`, `ai09_snapshot_all_fields_when_no_token`
- **Source:** [031_account_inspect.md AC-07](../../../../docs/feature/031_account_inspect.md)

---

### FT-08: Endpoint 001 failure: Tagged ID and UUID fall back with (snapshot)

- **Given:** An account with a valid token; endpoint 001 returns HTTP 500; `{name}.claude.json` snapshot contains `taggedId: "user_01abc"` and `uuid: "aaaa"`.
- **When:** `clp .account.inspect`
- **Then:** `Tagged ID: user_01abc (snapshot)`; `UUID: aaaa (snapshot)`. Other fields (from endpoints 002 and 005) show live data. Exit 0.
- **Exit:** 0
- **Source fn:** `ai09_snapshot_all_fields_when_no_token`
- **Source:** [031_account_inspect.md AC-08](../../../../docs/feature/031_account_inspect.md)

---

### FT-09: Endpoint 005 failure: org fields fall back with (snapshot)

- **Given:** An account with a valid token; endpoint 005 returns HTTP 403; `{name}.roles.json` snapshot contains `organization_name: "alice's Org"`, etc.
- **When:** `clp .account.inspect`
- **Then:** `Org: alice's Org (snapshot)`; `Org UUID: aaaa (snapshot)`; etc. Fields from endpoints 001 and 002 show live data. Exit 0.
- **Exit:** 0
- **Source fn:** `ai09_snapshot_all_fields_when_no_token`
- **Source:** [031_account_inspect.md AC-09](../../../../docs/feature/031_account_inspect.md)

---

### FT-10: Locally-expired token with refresh::1 triggers refresh_account_token()

- **Given:** An account whose `expiresAt` in `{name}.credentials.json` is in the past; `refresh_account_token()` succeeds and updates the token to a valid one; endpoints 001/002/005 then succeed.
- **When:** `clp .account.inspect` (default: `refresh::1`)
- **Then:** `Status: 🟢 valid (expires in Xh Ym)` — the output reflects the refreshed token; live data shown for all endpoints. `refresh_account_token()` was called once. Exit 0.
- **Exit:** 0
- **Source fn:** `lim_it_ai20_refresh_attempted_on_expired_token`
- **Source:** [031_account_inspect.md AC-10](../../../../docs/feature/031_account_inspect.md)

---

### FT-11: Locally-expired token with refresh::0: full snapshot fallback

- **Given:** An account whose `expiresAt` is in the past; `refresh::0` is specified.
- **When:** `clp .account.inspect refresh::0`
- **Then:** `Status: 🔴 expired (Xh Ym ago)`; `Memberships: endpoint unavailable (auth error)`; all fields show `(snapshot)` suffix or `N/A` if no snapshot. No `refresh_account_token()` call. Exit 0.
- **Exit:** 0
- **Source fn:** `ai08_expired_token_shows_expired_status`, `ai09_snapshot_all_fields_when_no_token`, `ai10_memberships_endpoint_unavailable_message`
- **Source:** [031_account_inspect.md AC-11](../../../../docs/feature/031_account_inspect.md)

---

### FT-12: name::prefix resolves to account; unknown name exits 2

- **Given:** Credential store contains `alice@acme.com.credentials.json`; `name::alice` resolves by prefix.
- **When:** `clp .account.inspect name::alice`
- **Then:** Output shows `Account: alice@acme.com`. Exit 0.
- **And When:** `clp .account.inspect name::nobody@acme.com`
- **Then:** Exit 2 with `account not found: nobody@acme.com`.
- **Exit:** 0 / 2
- **Source fn:** `ai07_prefix_name_resolves`, `ai02_account_not_found_exits_2`, `ai03_empty_name_exits_1`, `ai04_no_active_account_exits_2`, `ai06_active_marker_used_when_no_name`
- **Source:** [031_account_inspect.md AC-12](../../../../docs/feature/031_account_inspect.md)

---

### FT-13: format::json includes memberships array with selected field

- **Given:** A two-membership account.
- **When:** `clp .account.inspect format::json`
- **Then:** JSON output is valid; contains `memberships` array where each element has `index`, `billing_type`, `has_max`, `capabilities`, `selected`; the high-priority membership has `"selected": true`; the other has `"selected": false`. Also contains `account`, `status`, `expires_in_secs`, `tagged_id`, `uuid`, `billing_type`, `has_max`, `organization_name`, `organization_uuid`, `organization_role`, `workspace_uuid`, `workspace_name`, `data_source`.
- **Exit:** 0
- **Source fn:** `ai11_json_all_required_fields`, `ai12_json_data_source_snapshot_when_all_fail`, `ai05_format_invalid_exits_1`, `lim_it_ai19_valid_token_live_data_source_json`
- **Source:** [031_account_inspect.md AC-13](../../../../docs/feature/031_account_inspect.md)

---

### FT-14: trace::1 emits [trace] endpoint lines to stderr

- **Given:** An account with a valid token.
- **When:** `clp .account.inspect trace::1` (stderr captured)
- **Then:** Stderr contains at least three `[trace]` lines, one per endpoint (001, 002, 005), each showing the URL and HTTP status.
- **Exit:** 0
- **Source fn:** `ai13_trace_emits_lines_to_stderr`, `lim_it_ai21_trace_endpoint_lines_on_live_account`
- **Source:** [031_account_inspect.md AC-14](../../../../docs/feature/031_account_inspect.md)

---

### FT-15: No credential store exits 2

- **Given:** No credential store directory exists (fresh `$HOME`, no `.persistent/claude/credential/`).
- **When:** `clp .account.inspect name::alice@acme.com` (full email bypasses store lookup)
- **Then:** Exit 2 with `credential file not found: {path}`. The absent store is treated identically to an absent credential file — no distinct store-not-found branch exists.
- **Exit:** 2
- **Source fn:** `ai22_credential_store_absent_exits_2`
- **Source:** [031_account_inspect.md AC-15](../../../../docs/feature/031_account_inspect.md)

---

### FT-16: Priority 2 selection: stripe_subscription without claude_max preferred over none

- **Given:** An account whose endpoint 002 response contains two memberships: `[0] billing_type=none, has_max=false, capabilities=[chat]` and `[1] billing_type=stripe_subscription, has_max=false, capabilities=[chat]` (no `claude_max` in either).
- **When:** `clp .account.inspect`
- **Then:** Output shows `Memberships: 2`; membership [1] is marked `← selected` (stripe_subscription beats none even without claude_max); `Billing: stripe_subscription`; membership [0] is unmarked.
- **Exit:** 0
- **Source fn:** `lim_it_ai16_selected_marker_multi_membership` (marker count and single-vs-multi branching); priority rule verified by `mre_bug237_multi_membership_selects_stripe_over_none_no_max` in `claude_quota` crate
- **Source:** [031_account_inspect.md AC-03, AC-06](../../../../docs/feature/031_account_inspect.md)

---

### FT-17: Priority 3 fallback: all none memberships selects memberships[0]

- **Given:** An account whose endpoint 002 response contains two memberships: `[0] billing_type=none, capabilities=[chat]` and `[1] billing_type=none, capabilities=[chat]`.
- **When:** `clp .account.inspect`
- **Then:** Output shows `Memberships: 2`; membership [0] is marked `← selected` (Priority 3 fallback applies — no stripe_subscription in either; `memberships[0]` is the fallback); membership [1] is unmarked; `Billing: none`.
- **Exit:** 0
- **Source fn:** `lim_it_ai16_selected_marker_multi_membership` (marker count and single-vs-multi branching); fallback rule verified by `mre_bug237_single_membership_fallback_unchanged` in `claude_quota` crate
- **Source:** [031_account_inspect.md AC-03, AC-06](../../../../docs/feature/031_account_inspect.md)

---

### FT-18: Credential file absent exits 2

- **Given:** Credential store directory exists (`{credential_store}/`) but `alice@acme.com.credentials.json` is absent.
- **When:** `clp .account.inspect name::alice@acme.com`
- **Then:** Exit 2 with `credential file not found: {path}`. Unlike AC-15, the store directory is present; the credential file for the specific account is simply missing.
- **Exit:** 2
- **Source fn:** `ai01_credential_file_absent_exits_2`
- **Source:** [031_account_inspect.md AC-16](../../../../docs/feature/031_account_inspect.md)

---

### FT-19: Enterprise workspace fields show non-none values

- **Given:** An account whose endpoint 005 response includes non-null `workspace_uuid` and `workspace_name` (enterprise account with a named workspace).
- **When:** `clp .account.inspect`
- **Then:** `Workspace UUID:` shows the UUID string (not `(none)`); `Workspace:` shows the workspace name string (not `(none)`). In `format::json`, `workspace_uuid` and `workspace_name` contain the raw string values.
- **Exit:** 0
- **Source fn:** `ai23_workspace_fields_show_values_when_non_null`
- **Source:** [031_account_inspect.md AC-17](../../../../docs/feature/031_account_inspect.md)

---

### FT-20: Unicode account name (IDN email) resolves via full email lookup

- **Given:** An account named `alice@münchen.de` is registered via `write_account()` — the credentials file `alice@münchen.de.credentials.json` is present in the credential store (UTF-8 Linux filesystem).
- **When:** `clp .account.inspect name::alice@münchen.de refresh::0`
- **Then:** Exit 0; output contains `alice@münchen.de`. The AccountSelector performs a full email match; the unicode byte sequence in the filename survives the round-trip unchanged on a UTF-8 filesystem.
- **Exit:** 0
- **Source fn:** `ai27_unicode_account_name_resolves`
- **Source:** [031_account_inspect.md AC-12](../../../../docs/feature/031_account_inspect.md)

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
- **Source:** [031_account_inspect.md AC-18](../../../../docs/feature/031_account_inspect.md)

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
- **Source:** [031_account_inspect.md AC-19](../../../../docs/feature/031_account_inspect.md)

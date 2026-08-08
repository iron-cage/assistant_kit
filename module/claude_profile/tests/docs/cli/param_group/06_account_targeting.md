# Test: Account Targeting Parameter Group

Interaction tests for Group 6 (Account Targeting: `host::`, `role::`, `inference_provider::` on `.account.save`).
See [param_group/006_account_targeting.md](../../../../docs/cli/param_group/006_account_targeting.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | Both `host::` and `role::` written to same `{name}.json` | Behavioral Divergence |
| CC-2 | Combined safe default — no `host::` or `role::` → auto-captured host, empty role | Behavioral Divergence |
| CC-3 | Re-save with new `host::` overwrites `{name}.json` (idempotent) | Update Semantics |
| CC-4 | `cols::+host,+role` shows both columns populated from `{name}.json` | Cross-Command Display |
| CC-5 | `host::`, `role::`, `inference_provider::` all written together; `inference_provider` visible by default (no `cols::` needed) | Cross-Command Display |

---

### CC-1: Both `host::` and `role::` written to same `{name}.json`

- **Behavioral Divergence:** Providing both `host::` and `role::` produces a `{name}.json` with both fields; providing neither (CC-2) auto-captures host with empty role — proving both params govern independent metadata fields.
- **Given:** No pre-existing account for `test@example.com`.
- **When:** `clp .account.save name::test@example.com host::workbox role::dev`
- **Then:** Exits 0. `{credential_store}/test@example.com.json` exists and contains both `"host": "workbox"` and `"role": "dev"`.
- **Exit:** 0
- **Source fn:** `as_save_writes_profile_json` (in `account_renewal_test_b.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md)

---

### CC-2: Combined safe default — omitting both `host::` and `role::` auto-captures host

- **Behavioral Divergence:** Same `.account.save` invocation without `host::` or `role::` produces a `{name}.json` with auto-captured host and empty role — diverging from CC-1 where both were explicit.
- **Given:** `$USER=testuser`, `$HOSTNAME=testhost` in environment. No pre-existing account.
- **When:** `clp .account.save name::test@example.com` (neither `host::` nor `role::` provided)
- **Then:** Exits 0. `{name}.json` contains `"host": "testuser@testhost"` (auto-captured from `$USER@$HOSTNAME`). `"role"` field is empty string.
- **Exit:** 0
- **Source fn:** `as24_host_auto_capture_user_hostname` (in `account_renewal_test_b.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md)

---

### CC-3: Re-save with new `host::` overwrites `{name}.json`

- **Given:** Account `test@example.com` previously saved with `host::oldbox role::ops`.
- **When:** `clp .account.save name::test@example.com host::newbox role::dev`
- **Then:** Exits 0. `{name}.json` now contains `"host": "newbox"` and `"role": "dev"`. Previous `oldbox`/`ops` values overwritten; file is not accumulated.
- **Exit:** 0
- **Source fn:** `as26_host_resave_overwrites` (in `account_renewal_test_b.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md)

---

### CC-4: `cols::+host,+role` shows both columns populated from `{name}.json`

- **Given:** Account `test@example.com` saved with `host::mybox role::work`. `.usage` run against credential store.
- **When:** `clp .usage cols::+host,+role`
- **Then:** Exits 0. Table row for `test@example.com` shows `"mybox"` in the `Host` column and `"work"` in the `Role` column. Both columns appear in the header row.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it240_lim_it_cols_host_role_shows_profile_data` (in `usage_lim_it_test_b.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md)

---

### CC-5: `host::`, `role::`, and `inference_provider::` combine independently; `inference_provider` shown by default

- **Given:** No pre-existing account for `test@example.com`.
- **When:** `clp .account.save name::test@example.com host::workbox role::dev inference_provider::kimi`
- **Then:** Exits 0. `{name}.json` contains `"host": "workbox"`, `"role": "dev"`, and `"inference_provider": "kimi"` — three independent metadata fields, no interaction between them. A subsequent `clp .accounts name::test@example.com` (no `cols::`) shows the `Provider` column with `kimi` — unlike `host`/`role`, `inference_provider` is in the default identity set and needs no `cols::+` to appear.
- **Exit:** 0
- **Source fn:** *(planned — not yet implemented)*
- **Source:** [feature/072_inference_provider_selection.md AC-01, AC-05](../../../../docs/feature/072_inference_provider_selection.md), [param_group/006_account_targeting.md](../../../../docs/cli/param_group/006_account_targeting.md)

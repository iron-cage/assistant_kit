# Test: Account Targeting Parameter Group

Interaction tests for Group 6 (Account Targeting: `host::`, `role::` on `.account.save`).
See [param_group/006_account_targeting.md](../../../../docs/cli/param_group/006_account_targeting.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | Both `host::` and `role::` written to same `profile.json` | Behavioral Divergence |
| CC-2 | Combined safe default — no `host::` or `role::` → auto-captured host, empty role | Behavioral Divergence |
| CC-3 | Re-save with new `host::` overwrites `profile.json` (idempotent) | Update Semantics |
| CC-4 | `cols::+host,+role` shows both columns populated from `profile.json` | Cross-Command Display |

---

### CC-1: Both `host::` and `role::` written to same `profile.json`

- **Behavioral Divergence:** Providing both `host::` and `role::` produces a `profile.json` with both fields; providing neither (CC-2) auto-captures host with empty role — proving both params govern independent metadata fields.
- **Given:** No pre-existing account for `test@example.com`.
- **When:** `clp .account.save name::test@example.com host::workbox role::dev`
- **Then:** Exits 0. `{credential_store}/test@example.com.profile.json` exists and contains both `"host": "workbox"` and `"role": "dev"`.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md)

---

### CC-2: Combined safe default — omitting both `host::` and `role::` auto-captures host

- **Behavioral Divergence:** Same `.account.save` invocation without `host::` or `role::` produces a `profile.json` with auto-captured host and empty role — diverging from CC-1 where both were explicit.
- **Given:** `$USER=testuser`, `$HOSTNAME=testhost` in environment. No pre-existing account.
- **When:** `clp .account.save name::test@example.com` (neither `host::` nor `role::` provided)
- **Then:** Exits 0. `profile.json` contains `"host": "testuser@testhost"` (auto-captured from `$USER@$HOSTNAME`). `"role"` field is empty string.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md)

---

### CC-3: Re-save with new `host::` overwrites `profile.json`

- **Given:** Account `test@example.com` previously saved with `host::oldbox role::ops`.
- **When:** `clp .account.save name::test@example.com host::newbox role::dev`
- **Then:** Exits 0. `profile.json` now contains `"host": "newbox"` and `"role": "dev"`. Previous `oldbox`/`ops` values overwritten; file is not accumulated.
- **Exit:** 0
- **Source fn:** ⏳ (in `tests/cli/account_mutations_test.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md)

---

### CC-4: `cols::+host,+role` shows both columns populated from `profile.json`

- **Given:** Account `test@example.com` saved with `host::mybox role::work`. `.usage` run against credential store.
- **When:** `clp .usage cols::+host,+role`
- **Then:** Exits 0. Table row for `test@example.com` shows `"mybox"` in the `Host` column and `"work"` in the `Role` column. Both columns appear in the header row.
- **Exit:** 0
- **Live:** yes
- **Source fn:** ⏳ (in `tests/cli/usage_test.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md)

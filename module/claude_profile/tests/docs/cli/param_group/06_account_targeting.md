# Test: Account Targeting Parameter Group

Interaction tests for Group 6 (Account Targeting: `host::`, `tags::`, `inference_provider::` on `.account.save` — `role::` REMOVED by Feature 075).
See [param_group/006_account_targeting.md](../../../../docs/cli/param_group/006_account_targeting.md) for specification.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | Both `host::` and `tags::` written to same `{name}.json` | Behavioral Divergence |
| CC-2 | Combined safe default — no `host::` or `tags::` → auto-captured host, no `tags` key | Behavioral Divergence |
| CC-3 | Re-save with new `host::` overwrites `{name}.json` (idempotent) | Update Semantics |
| CC-4 | `cols::+host,+role` shows both columns populated from `{name}.json` (legacy `role` field) | Cross-Command Display |
| CC-5 | `host::`, `tags::`, `inference_provider::` all written together; `inference_provider` visible by default (no `cols::` needed) | Cross-Command Display |

---

### CC-1: Both `host::` and `tags::` written to same `{name}.json`

- **Behavioral Divergence:** Providing both `host::` and `tags::` produces a `{name}.json` with both fields; providing neither (CC-2) auto-captures host and writes no `tags` key — proving both params govern independent metadata fields.
- **Given:** No pre-existing account for `test@example.com`.
- **When:** `clp .account.save name::test@example.com host::testbox tags::dev`
- **Then:** Exits 0. `{credential_store}/test@example.com.json` exists and contains both `"host": "testbox"` and `"tags": ["dev"]`.
- **Exit:** 0
- **Source fn:** `as_save_writes_profile_json` (in `account_renewal_test_b.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md), [feature/075_account_tags.md AC-01](../../../../docs/feature/075_account_tags.md)

---

### CC-2: Combined safe default — omitting both `host::` and `tags::` auto-captures host

- **Behavioral Divergence:** Same `.account.save` invocation without `host::` or `tags::` produces a `{name}.json` with auto-captured host and no `tags` key (absent ≡ empty set, [feature/075 AC-02]) — diverging from CC-1 where both were explicit.
- **Given:** `$USER=alice`, `$HOSTNAME=workstation` in environment. No pre-existing account.
- **When:** `clp .account.save name::test@example.com` (neither `host::` nor `tags::` provided)
- **Then:** Exits 0. `{name}.json` contains `"host": "alice@workstation"` (auto-captured from `$USER@$HOSTNAME`). No `role` or `tags` key is written.
- **Exit:** 0
- **Source fn:** `as24_host_auto_capture_user_hostname` (in `account_renewal_test_b.rs`); tags-key absence: `account_tag_t02_save_without_tags_omits_field` (in `account_tag_test.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md), [feature/075_account_tags.md AC-02](../../../../docs/feature/075_account_tags.md)

---

### CC-3: Re-save with new `host::` overwrites `{name}.json`

- **Given:** Account `test@example.com` previously saved with `host::oldbox`.
- **When:** `clp .account.save name::test@example.com host::newbox`
- **Then:** Exits 0. `{name}.json` now contains `"host": "newbox"`; the previous `oldbox` value is overwritten and no longer present — file is not accumulated. (This test exercises `host::` overwrite only; it does not pass `role::` in either save call.)
- **Exit:** 0
- **Source fn:** `as26_host_resave_overwrites` (in `account_renewal_test_b.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md)

---

### CC-4: `cols::+host,+role` shows both columns populated from `{name}.json`

- **Given:** Account `test@example.com` whose `{name}.json` carries `"host": "mybox"` and a legacy `"role": "work"` field (written directly — `role::` is REMOVED from `.account.save` by Feature 075; the field survives on accounts never tag-written, and `cols::+role` remains a legacy display column until the lazy migration erases it). `.usage` run against credential store.
- **When:** `clp .usage cols::+host,+role`
- **Then:** Exits 0. Table row for `test@example.com` shows `"mybox"` in the `Host` column and `"work"` in the `Role` column. Both columns appear in the header row.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it240_lim_it_cols_host_role_shows_profile_data` (in `usage_lim_it_test_b.rs`)
- **Source:** [feature/029_account_host_metadata.md](../../../../docs/feature/029_account_host_metadata.md)

---

### CC-5: `host::`, `tags::`, and `inference_provider::` combine independently; `inference_provider` shown by default

- **Given:** No pre-existing account for `test@example.com`.
- **When:** `clp .account.save name::test@example.com host::workbox tags::dev inference_provider::kimi`
- **Then:** Exits 0. `{name}.json` contains `"host": "workbox"`, `"tags": ["dev"]`, and `"inference_provider": "kimi"` — three independent metadata fields, no interaction between them. A subsequent `clp .accounts name::test@example.com` (no `cols::`) shows the `Provider` column with `kimi` — unlike `host`/`tags`, `inference_provider` is in the default identity set and needs no `cols::+` to appear.
- **Exit:** 0
- **Source fn:** *(planned — not yet implemented)*
- **Source:** [feature/072_inference_provider_selection.md AC-01, AC-05](../../../../docs/feature/072_inference_provider_selection.md), [param_group/006_account_targeting.md](../../../../docs/cli/param_group/006_account_targeting.md)

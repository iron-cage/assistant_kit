# Schema 002: Account JSON — `{name}.json`

SC test cases for `docs/schema/002_account_json.md`. Verifies the supplementary account
metadata file: read-merge write semantics, field-specific preservation rules, encoding
format compliance, and append-only history behavior.

**Source:** [docs/schema/002_account_json.md](../../../../docs/schema/002_account_json.md)

### SC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| SC-1 | Re-save preserves unrelated fields via read-merge | Write Semantics | ✅ |
| SC-2 | `_renewal_at` preserved by save — not overwritten | Preserved-Only Fields | ✅ |
| SC-3 | `owner` field preserved by unrelated saves | Preserved-Only Fields | ✅ |
| SC-4 | JSON format: 2-space pretty-print, trailing newline | Encoding | ✅ |
| SC-5 | `history` array appended — never truncated by successful fetch | Append-Only | ✅ |
| SC-6 | `_quota_cache` updated atomically on successful API call | Cache Write | ✅ |

---

### SC-1: Re-save preserves unrelated fields via read-merge

- **Given:** `{name}.json` contains `_renewal_at`, `host`, and `role` fields not provided by the current save operation
- **When:** `.account.save` is invoked again for the same account (without specifying `_renewal_at`, `host::`, or `role::`)
- **Then:** All 3 preserved-only fields remain in `{name}.json` unchanged — `save()` performs a read-merge, not a clobber
- **Source fn:** `as29_resave_credentials_unchanged` (cli/account_renewal_test_b.rs)
- **Source:** [docs/schema/002_account_json.md §Format §Preserved-Only Fields](../../../../docs/schema/002_account_json.md)

---

### SC-2: `_renewal_at` is preserved by `.account.save` — not overwritten

- **Given:** `{name}.json` has `_renewal_at` set by a prior `.account.renewal` call
- **When:** `.account.save` is invoked without providing `at::` or `from_now::`
- **Then:** `_renewal_at` remains unchanged in `{name}.json` — `.account.save` never touches this field
- **Source fn:** `as29_resave_credentials_unchanged` (cli/account_renewal_test_b.rs)
- **Source:** [docs/schema/002_account_json.md §Preserved-Only Fields](../../../../docs/schema/002_account_json.md)

---

### SC-3: `owner` field preserved by saves that don't provide ownership params

- **Given:** `{name}.json` has `owner` set by a prior assign/claim operation
- **When:** `.account.save` is invoked without `owner::` parameter
- **Then:** `owner` field remains in `{name}.json` unchanged — ownership state survives re-save
- **Source fn:** `ft12_save_does_not_stamp_owner` (cli/account_ownership_test.rs)
- **Source:** [docs/schema/002_account_json.md §Preserved-Only Fields](../../../../docs/schema/002_account_json.md)

---

### SC-4: File is 2-space pretty-printed JSON with trailing newline

- **Given:** A valid `.account.save` operation completes
- **When:** `{name}.json` is read as raw bytes
- **Then:** The content is valid JSON with 2-space indentation and ends with a newline character — complies with [invariant/007](../../../../docs/invariant/007_json_storage_format.md)
- **Source fn:** `sc4_002_account_json_is_2space_pretty_with_trailing_newline` (account_tests.rs)
- **Source:** [docs/schema/002_account_json.md §Format](../../../../docs/schema/002_account_json.md)

---

### SC-5: `history` array is append-only — successful fetches add entries, never remove

- **Given:** `{name}.json` contains a `history` array with N existing measurement entries
- **When:** A successful `fetch_oauth_usage()` call completes
- **Then:** `history` contains N+1 entries — the new measurement is appended and the prior entries are preserved
- **Source fn:** `sc5_002_history_entry_appended_not_truncated` (account_tests.rs)
- **Source:** [docs/schema/002_account_json.md §Field Table (history)](../../../../docs/schema/002_account_json.md)

---

### SC-6: `_quota_cache` updated atomically on successful API call

- **Given:** A prior `_quota_cache` exists in `{name}.json`
- **When:** A successful `fetch_oauth_usage()` call completes and `write_quota_cache()` is invoked
- **Then:** All `_quota_cache` subfields (`five_hour`, `seven_day`, `seven_day_sonnet`, `cached_at`) are written as a single coherent object — no partial write leaves mismatched fields
- **Source fn:** `sc6_002_quota_cache_all_subfields_written_atomically` (account_tests.rs)
- **Source:** [docs/schema/002_account_json.md §Field Table (_quota_cache)](../../../../docs/schema/002_account_json.md)

# Schema 002: Account JSON — `{name}.json`

SC test cases for `docs/schema/002_account_json.md`. Verifies the supplementary account
metadata file: read-merge write semantics, field-specific preservation rules, encoding
format compliance, and append-only history behavior.

**Source:** [docs/schema/002_account_json.md](../../../docs/schema/002_account_json.md)

### SC Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| SC-1 | Re-save preserves unrelated fields via read-merge | Write Semantics | ✅ |
| SC-2 | `_renewal_at` preserved by save — not overwritten | Preserved-Only Fields | ✅ |
| SC-3 | `owner` field preserved by unrelated saves | Preserved-Only Fields | ✅ |
| SC-4 | JSON format: 2-space pretty-print, trailing newline | Encoding | ✅ |
| SC-5 | `history` array appended — never truncated by successful fetch | Append-Only | ✅ |
| SC-6 | `cache` updated atomically on successful API call | Cache Write | ✅ |
| SC-7 | `inference_provider` written only when explicitly given; preserved by unrelated saves | Preserved-Only Fields | 🔲 |
| SC-8 | `tags` written sorted/deduplicated when given; key absent when never given | Write Semantics | ✅ |

---

### SC-1: Re-save preserves unrelated fields via read-merge

- **Given:** `{name}.json` contains `_renewal_at`, `host`, and `role` fields not provided by the current save operation
- **When:** `.account.save` is invoked again for the same account (without specifying `_renewal_at`, `host::`, or `role::`)
- **Then:** All 3 preserved-only fields remain in `{name}.json` unchanged — `save()` performs a read-merge, not a clobber
- **Note:** Direct test evidence covers `_renewal_at` preservation only (shared with SC-2).
  `host` traverses the same `save()` read-merge code path but is independently tested only
  for its EXPLICIT-overwrite behavior (`as26_host_resave_overwrites` in
  `tests/cli/account_renewal_test_b.rs`), not omission-preservation specifically. `role`
  became a preserved-only legacy field with Feature 075 (`role::` removed; the CLI always
  passes `None`, so `save()` never writes it) — a stored legacy value survives re-saves
  until the first tag write migrates it to a tag
  (`account_tag_t09_first_tag_write_migrates_role` in `tests/cli/account_tag_test.rs`).
- **Source fn:** `as22_save_preserves_renewal_at` in `tests/cli/account_renewal_test.rs`
- **Source:** [docs/schema/002_account_json.md §Format §Preserved-Only Fields](../../../docs/schema/002_account_json.md)

---

### SC-2: `_renewal_at` is preserved by `.account.save` — not overwritten

- **Given:** `{name}.json` has `_renewal_at` set by a prior `.account.renewal` call
- **When:** `.account.save` is invoked without providing `at::` or `from_now::`
- **Then:** `_renewal_at` remains unchanged in `{name}.json` — `.account.save` never touches this field
- **Source fn:** `as22_save_preserves_renewal_at` in `tests/cli/account_renewal_test.rs`
- **Source:** [docs/schema/002_account_json.md §Preserved-Only Fields](../../../docs/schema/002_account_json.md)

---

### SC-3: `owner` field preserved by saves that don't provide ownership params

- **Given:** `{name}.json` has `owner` set by a prior assign/claim operation
- **When:** `.account.save` is invoked without `owner::` parameter
- **Then:** `owner` field remains in `{name}.json` unchanged — ownership state survives re-save
- **Source fn:** `ft12_save_does_not_stamp_owner` (cli/account_ownership_test.rs)
- **Source:** [docs/schema/002_account_json.md §Preserved-Only Fields](../../../docs/schema/002_account_json.md)

---

### SC-4: File is 2-space pretty-printed JSON with trailing newline

- **Given:** A valid `.account.save` operation completes
- **When:** `{name}.json` is read as raw bytes
- **Then:** The content is valid JSON with 2-space indentation and ends with a newline character — complies with [invariant/007](../../../docs/invariant/007_json_storage_format.md)
- **Source fn:** `sc4_002_account_json_is_2space_pretty_with_trailing_newline` (account_tests.rs)
- **Source:** [docs/schema/002_account_json.md §Format](../../../docs/schema/002_account_json.md)

---

### SC-5: `history` array is append-only — successful fetches add entries, never remove

- **Given:** `{name}.json` contains a `history` array with N existing measurement entries
- **When:** A successful `fetch_oauth_usage()` call completes
- **Then:** `history` contains N+1 entries — the new measurement is appended and the prior entries are preserved
- **Source fn:** `sc5_002_history_entry_appended_not_truncated` (account_tests.rs)
- **Source:** [docs/schema/002_account_json.md §Field Table (history)](../../../docs/schema/002_account_json.md)

---

### SC-6: `cache` updated atomically on successful API call

- **Given:** A prior `cache` exists in `{name}.json`
- **When:** A successful `fetch_oauth_usage()` call completes and `write_quota_cache()` is invoked
- **Then:** All `cache` subfields (`fetched_at`, `five_hour`, `seven_day`, `seven_day_sonnet`) are written as a single coherent object — no partial write leaves mismatched fields
- **Source fn:** `sc6_002_quota_cache_all_subfields_written_atomically` (account_tests.rs)
- **Source:** [docs/schema/002_account_json.md §Field Table (cache)](../../../docs/schema/002_account_json.md)

---

### SC-7: `inference_provider` written only when explicitly given; preserved by unrelated saves

- **Given:** `{name}.json` does not yet contain an `inference_provider` key.
- **When:** `.account.save` is invoked without `inference_provider::` — then, in a second scenario, `{name}.json` already has `"inference_provider": "kimi"` and `.account.save` is invoked again without `inference_provider::`.
- **Then:** First scenario: `{name}.json` still has no `inference_provider` key at all — the field is never written as the literal default `"anthropic"`. Second scenario: `"inference_provider": "kimi"` remains unchanged — `save()`'s read-merge preserves it exactly like `_renewal_at`, `owner`, `host`, and `role` (SC-1, SC-2, SC-3).
- **Source fn:** *(planned — not yet implemented)*
- **Source:** [docs/schema/002_account_json.md §Preserved-Only Fields](../../../docs/schema/002_account_json.md), [feature/072_inference_provider_selection.md AC-02](../../../docs/feature/072_inference_provider_selection.md)

---

### SC-8: `tags` written sorted/deduplicated when given; key absent when never given

- **Given:** No pre-existing account — then, second scenario, a fresh save that never passes `tags::`.
- **When:** `.account.save tags::kimi_pool,ci,ci` — then `.account.save` without `tags::`.
- **Then:** First scenario: `{name}.json` contains `"tags": ["ci", "kimi_pool"]` — lowercased, deduplicated, sorted at write. Second scenario: no `tags` key at all — absence ≡ empty set on every read path, never written as a literal `[]`.
- **Source fn:** `account_tag_t01_save_tags_writes_sorted_dedup`, `account_tag_t02_save_without_tags_omits_field` (in `tests/cli/account_tag_test.rs`)
- **Source:** [docs/schema/002_account_json.md §Field Table (tags)](../../../docs/schema/002_account_json.md), [feature/075_account_tags.md AC-01, AC-02](../../../docs/feature/075_account_tags.md)

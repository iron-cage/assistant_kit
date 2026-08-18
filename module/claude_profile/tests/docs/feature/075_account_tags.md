# FT — Feature 075: Account Tags

### Scope

- **Purpose**: Test cases for the account `tags` set — write paths (`.account.save tags::`, `.account.tag`), the `.tags` listing, `.accounts` integration, `role::` removal, and lazy `role`→tag migration.
- **Source**: `docs/feature/075_account_tags.md`
- **Covers**: AC-01 through AC-16

### Test Cases

| FT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| FT-01 | AC-01 | `.account.save tags::kimi_pool,ci` → sorted deduplicated array written | `t01_save_tags_writes_sorted_dedup` |
| FT-02 | AC-02 | `.account.save` without `tags::` → field absent; reads treat as empty set | `t02_save_without_tags_omits_field` |
| FT-03 | AC-03 | Invalid tag (charset/length/empty item) → exit 1 naming it, no write | `t03_invalid_tag_exits_1_no_write` |
| FT-04 | AC-04 | `.account.save role::x` → exit 1 with migration message naming `tags::` | `t04_role_param_exits_1_migration_message` |
| FT-05 | AC-05 | `.account.tag add::a,b` unions into existing set | `t05_tag_add_unions_set` |
| FT-06 | AC-06 | `.account.tag remove::a` removes; absent tag is no-op success | `t06_tag_remove_idempotent` |
| FT-07 | AC-07 | `.account.tag tags::a,b` replaces; combining ops exits 1 | `t07_tag_replace_and_mutual_exclusion` |
| FT-08 | AC-08 | `.account.tag` with no operation → exit 1 | `t08_tag_no_operation_exits_1` |
| FT-09 | AC-09 | First tag write converts non-empty `role` to tag, removes field | `t09_first_tag_write_migrates_role` |
| FT-10 | AC-10 | Ungated write; `name::X,Y` batch; `dry::1` touches nothing | `t10_tag_ungated_batch_dry` |
| FT-11 | AC-11 | `.tags` lists union of account + filter tags with counts; `(no tags)` exit 0 | `t11_tags_lists_union_sorted` |
| FT-12 | AC-12 | `.tags format::json` → array of `{"tag","accounts","filters"}` | `t12_tags_json_shape` |
| FT-13 | AC-13 | `.accounts tags::a,b` shows only accounts carrying all listed tags | `t13_accounts_tags_subset_filter` |
| FT-14 | AC-14 | `Tags:` line only for ≥1 tag; JSON always includes `tags` array | `t14_accounts_tags_line_and_json` |
| FT-15 | AC-15 | `cols::+tags` adds Tags column on `.accounts`/`.usage`; not in defaults | `t15_cols_plus_tags_column` |
| FT-16 | AC-16 | No tags anywhere → pre-existing commands byte-identical | `t16_untagged_store_byte_identical` |

### Notes

- **📋 Planned — implementation pending.** All Source fn names are *prescriptive* for the implementation task (`tests/cli/account_tag_test.rs`), not descriptive of existing tests; none exist yet. When implementation lands, correct any drifted names here rather than leaving stale citations (the staleness pattern documented in `072_inference_provider_selection.md`'s Notes).
- All FT cases must use a temporary isolated credential store; no real user environment.
- FT-01–FT-04 exercise the `.account.save` write path; FT-05–FT-10 exercise `.account.tag`; FT-11–FT-12 the `.tags` listing; FT-13–FT-15 `.accounts`/`.usage` rendering; FT-16 the zero-migration guarantee.
- FT-09 must cover both trigger paths: a `.account.save tags::` write and a `.account.tag` write (including `remove::`) each fire the migration.
- FT-11's union must include a tag that exists only in a `_filter_*` file (accounts count 0, filters count ≥1) — the typo-hazard row.
- FT-16 is the adoption-safety anchor: run representative pre-existing commands (`.accounts`, `.usage`-level list rendering) against a store never tag-written and diff output against pre-feature expectations.
- Tag value contract (charset, normalization, ordering): `docs/type/003_tag.md`; on-disk field: `docs/schema/002_account_json.md`.
- Command-level IT specs sharing these scenarios: `tests/docs/cli/command/25_account_tag.md` (FT-05–FT-10), `tests/docs/cli/command/22_tags.md` (FT-11–FT-12).

---

### FT-01: `.account.save tags::` writes sorted deduplicated set

- **Given:** Active credentials present; account `alice@test.com` not yet saved.
- **When:** `clp .account.save name::alice@test.com tags::kimi_pool,ci,kimi_pool`
- **Then:** `alice@test.com.json` contains `"tags": ["ci", "kimi_pool"]`. Exits 0.
- **Exit:** 0
- **Source fn:** `t01_save_tags_writes_sorted_dedup` *(planned)*
- **Source:** [075_account_tags.md AC-01](../../../docs/feature/075_account_tags.md)

---

### FT-02: Omitted `tags::` leaves field absent

- **Given:** Any state.
- **When:** `clp .account.save name::alice@test.com`
- **Then:** `alice@test.com.json` has no `tags` key; `.accounts`/`.tags` treat the account as untagged (empty set), no error.
- **Exit:** 0
- **Source fn:** `t02_save_without_tags_omits_field` *(planned)*
- **Source:** [075_account_tags.md AC-02](../../../docs/feature/075_account_tags.md)

---

### FT-03: Invalid tag exits 1 without writing

- **Given:** Any state.
- **When:** `clp .account.save name::alice@test.com tags::Bad!Tag` (also: 65-char tag; empty comma item `a,,b`)
- **Then:** Exits 1; stderr names the offending tag (post-lowercasing form); no file written.
- **Exit:** 1
- **Source fn:** `t03_invalid_tag_exits_1_no_write` *(planned)*
- **Source:** [075_account_tags.md AC-03](../../../docs/feature/075_account_tags.md)

---

### FT-04: `role::` exits 1 with migration message

- **Given:** Any state.
- **When:** `clp .account.save name::alice@test.com role::work`
- **Then:** Exits 1; stderr names `tags::` as the replacement; no file written.
- **Exit:** 1
- **Source fn:** `t04_role_param_exits_1_migration_message` *(planned)*
- **Source:** [075_account_tags.md AC-04](../../../docs/feature/075_account_tags.md)

---

### FT-05: `add::` unions into the existing set

- **Given:** `alice@test.com.json` has `"tags": ["ci"]`.
- **When:** `clp .account.tag name::alice@test.com add::kimi_pool,ci`
- **Then:** `alice@test.com.json` contains `"tags": ["ci", "kimi_pool"]` (dedup, sorted). Exits 0.
- **Exit:** 0
- **Source fn:** `t05_tag_add_unions_set` *(planned)*
- **Source:** [075_account_tags.md AC-05](../../../docs/feature/075_account_tags.md)

---

### FT-06: `remove::` is idempotent

- **Given:** `alice@test.com.json` has `"tags": ["ci", "kimi_pool"]`.
- **When:** `clp .account.tag name::alice@test.com remove::ci` then `clp .account.tag name::alice@test.com remove::nonexistent`
- **Then:** First call leaves `["kimi_pool"]`; second call is a no-op success leaving the set unchanged. Both exit 0.
- **Exit:** 0
- **Source fn:** `t06_tag_remove_idempotent` *(planned)*
- **Source:** [075_account_tags.md AC-06](../../../docs/feature/075_account_tags.md)

---

### FT-07: `tags::` replaces; combined operations exit 1

- **Given:** `alice@test.com.json` has `"tags": ["ci", "kimi_pool"]`.
- **When:** `clp .account.tag name::alice@test.com tags::personal`; then `clp .account.tag name::alice@test.com tags::a add::b`; then `clp .account.tag name::alice@test.com add::a remove::b`
- **Then:** First call replaces the whole set with `["personal"]` (exit 0); second and third exit 1 (one operation per invocation), set unchanged.
- **Exit:** 0 / 1 / 1
- **Source fn:** `t07_tag_replace_and_mutual_exclusion` *(planned)*
- **Source:** [075_account_tags.md AC-07](../../../docs/feature/075_account_tags.md)

---

### FT-08: No operation given exits 1

- **Given:** `alice@test.com` saved.
- **When:** `clp .account.tag name::alice@test.com`
- **Then:** Exits 1; stderr says no operation given, naming `add::`/`remove::`/`tags::`.
- **Exit:** 1
- **Source fn:** `t08_tag_no_operation_exits_1` *(planned)*
- **Source:** [075_account_tags.md AC-08](../../../docs/feature/075_account_tags.md)

---

### FT-09: First tag write migrates legacy `role`

- **Given:** `alice@test.com.json` has `"role": "Work"` and no `tags` key.
- **When:** `clp .account.tag name::alice@test.com add::ci` (variant: `.account.save tags::ci`; variant: `remove::x`)
- **Then:** `alice@test.com.json` contains `"tags": ["ci", "work"]` (role lowercased/sanitized, merged) and **no** `role` key. Exits 0.
- **Exit:** 0
- **Source fn:** `t09_first_tag_write_migrates_role` *(planned)*
- **Source:** [075_account_tags.md AC-09](../../../docs/feature/075_account_tags.md)

---

### FT-10: Ungated writes, comma-list batch, `dry::1`

- **Given:** `alice@test.com` owned by a *different* Identity; `bob@test.com` unowned.
- **When:** `clp .account.tag name::alice@test.com,bob@test.com add::ci dry::1` then the same without `dry::1`
- **Then:** No ownership error either time (ungated); `dry::1` run leaves both files byte-identical and prints previews; real run applies `ci` to both.
- **Exit:** 0
- **Source fn:** `t10_tag_ungated_batch_dry` *(planned)*
- **Source:** [075_account_tags.md AC-10](../../../docs/feature/075_account_tags.md)

---

### FT-11: `.tags` lists the union, sorted, with counts

- **Given:** Accounts carrying `ci` (2 accounts) and `kimi_pool` (1); a `_filter_*` file referencing `kimi_pool` and `typo_tag` (carried by no account).
- **When:** `clp .tags`
- **Then:** Rows sorted: `ci 2 0`, `kimi_pool 1 1`, `typo_tag 0 1`. With an empty store: prints `(no tags)`, exits 0.
- **Exit:** 0
- **Source fn:** `t11_tags_lists_union_sorted` *(planned)*
- **Source:** [075_account_tags.md AC-11](../../../docs/feature/075_account_tags.md)

---

### FT-12: `.tags format::json` shape

- **Given:** Same fixture as FT-11.
- **When:** `clp .tags format::json`
- **Then:** Stdout is a JSON array of `{"tag": …, "accounts": N, "filters": N}` objects, sorted by tag.
- **Exit:** 0
- **Source fn:** `t12_tags_json_shape` *(planned)*
- **Source:** [075_account_tags.md AC-12](../../../docs/feature/075_account_tags.md)

---

### FT-13: `.accounts tags::` subset filter

- **Given:** Accounts: `alice` tags `[ci, kimi_pool]`, `bob` tags `[ci]`, `carol` untagged.
- **When:** `clp .accounts tags::ci,kimi_pool`
- **Then:** Only `alice` listed (must carry **all** listed tags); `clp .accounts tags::ci` lists `alice` and `bob`.
- **Exit:** 0
- **Source fn:** `t13_accounts_tags_subset_filter` *(planned)*
- **Source:** [075_account_tags.md AC-13](../../../docs/feature/075_account_tags.md)

---

### FT-14: `Tags:` line rules and JSON array

- **Given:** `alice` tags `[ci]`; `carol` untagged.
- **When:** `clp .accounts` and `clp .accounts format::json`
- **Then:** Text mode shows `Tags: ci` for `alice` and no `Tags:` line for `carol`; JSON includes `"tags": ["ci"]` and `"tags": []` respectively (always present).
- **Exit:** 0
- **Source fn:** `t14_accounts_tags_line_and_json` *(planned)*
- **Source:** [075_account_tags.md AC-14](../../../docs/feature/075_account_tags.md)

---

### FT-15: `cols::+tags` opt-in column

- **Given:** Tagged accounts present.
- **When:** `clp .accounts cols::+tags format::table` and `clp .usage cols::+tags` (quota fetch may be stubbed/failed — column presence is the assertion); plus default invocations without `cols::`
- **Then:** `Tags` column appears (comma-joined) only when opted in; absent from both commands' default sets.
- **Exit:** 0
- **Source fn:** `t15_cols_plus_tags_column` *(planned)*
- **Source:** [075_account_tags.md AC-15](../../../docs/feature/075_account_tags.md)

---

### FT-16: Zero-migration adoption

- **Given:** A store where no tag was ever written (accounts may carry legacy `role` fields).
- **When:** Pre-existing commands run (`.accounts`, `.accounts format::json`, list rendering paths).
- **Then:** Output is byte-identical to pre-feature behavior; `role` fields remain untouched; no `tags` keys appear.
- **Exit:** 0
- **Source fn:** `t16_untagged_store_byte_identical` *(planned)*
- **Source:** [075_account_tags.md AC-16](../../../docs/feature/075_account_tags.md)

# Test: `role::` Parameter (Account Metadata Label) — REMOVED

The save-side `role::` free-text metadata parameter on `.account.save` was **removed by
Feature 075** (a role value is now just a tag — see
[075_account_tags.md](../../feature/075_account_tags.md)). Passing `role::` exits 1 with a
migration message naming `tags::`; the rejection contract is pinned by
`account_tag_t04_role_param_exits_1_migration` in `tests/cli/account_tag_test.rs`, and the
absence of a legacy `role` field on fresh saves by `as31_save_omits_role_field` in
`tests/cli/account_renewal_test_b.rs`.

Former cases EC-1, EC-2, EC-3, EC-5, EC-6 (write/overwrite/empty-string semantics of the
stored `role` field) tested the removed behavior and were deleted together with their
source fns (`as30`, `as31_role_omit_stores_empty`, `as32`, `as33`, `as34`). Stored `role`
fields in pre-075 profile JSON files remain readable and are migrated to a tag on the
first tag write (Feature 075 AC-09, `account_tag_t09_first_tag_write_migrates_role`).

Note: This is distinct from param 015 `role::` (boolean display toggle for
`.accounts`/`.credentials.status`), which is unaffected.

### Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-4 | Legacy `role` value appears in `clp .usage cols::+role` | Display |

---

### EC-4: Legacy `role` value appears in `clp .usage cols::+role`

- **Given:** Account `test@example.com` whose profile JSON carries a legacy `"role": "work"` field (pre-075 store; no tag write has migrated it yet).
- **When:** `clp .usage cols::+role`
- **Then:** Exits 0. Table row for `test@example.com` shows "work" in Role column.
- **Exit:** 0
- **Live:** yes
- **Source fn:** `it203_cols_role_shows_role_column` (in `usage_filter_test_b.rs`)
- **Source:** [param/052_role.md](../../../../docs/cli/param/052_role.md)

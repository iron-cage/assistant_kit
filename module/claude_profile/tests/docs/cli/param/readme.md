# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual clp parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: 54 active clp parameter edge case files (params 1–55 except param 2); `current::` (docs param 18) has no dedicated EC file — covered by `command/003_accounts.md` IT tests. Production param 56 (`unclaim::`) was REMOVED — file `57_unclaim.md` is retained as a tombstone with no active test cases.
- **Numbering note**: Test doc numbering is offset +1 from `docs/cli/param/` starting at position 2 (test `003_format.md` ↔ production `002_format.md`). Test file `002_` does not exist; this is intentional — the offset arose when `current::` was excluded from EC coverage, shifting subsequent test IDs by one.
- **Out of Scope**: Command-level tests (→ `command/`), parameter group interactions (→ `param_group/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 01_name.md | Edge cases for `name::` parameter |
| 03_format.md | Edge cases for `format::` parameter |
| 04_threshold.md | Edge cases for `threshold::` parameter |
| 05_dry.md | Edge cases for `dry::` parameter |
| 06_account.md | Edge cases for `account::` parameter |
| 07_sub.md | Edge cases for `sub::` parameter |
| 08_tier.md | Edge cases for `tier::` parameter |
| 09_token.md | Edge cases for `token::` parameter |
| 10_expires.md | Edge cases for `expires::` parameter |
| 11_email.md | Edge cases for `email::` parameter (`.credentials.status` and `.accounts`) |
| 12_file.md | Edge cases for `file::` parameter |
| 13_saved.md | Edge cases for `saved::` parameter |
| 14_active.md | Edge cases for `active::` parameter |
| 15_display_name.md | Edge cases for `display_name::` parameter |
| 16_role.md | Edge cases for `role::` parameter |
| 17_billing.md | Edge cases for `billing::` parameter |
| 18_model.md | Edge cases for `model::` parameter |
| 19_refresh.md | Edge cases for `refresh::` parameter |
| 20_live.md | Edge cases for `live::` parameter |
| 21_interval.md | Edge cases for `interval::` parameter |
| 22_jitter.md | Edge cases for `jitter::` parameter |
| 23_trace.md | Edge cases for `trace::` parameter |
| 24_field.md | Edge cases for `field::` parameter |
| 25_sort.md | Edge cases for `sort::` parameter |
| 26_desc.md | Edge cases for `desc::` parameter |
| 27_prefer.md | Edge cases for `prefer::` parameter |
| 28_uuid.md | Edge cases for `uuid::` parameter |
| 29_capabilities.md | Edge cases for `capabilities::` parameter |
| 30_org_uuid.md | Edge cases for `org_uuid::` parameter |
| 31_org_name.md | Edge cases for `org_name::` parameter |
| 32_next.md | Edge cases for `next::` parameter |
| 33_cols.md | Edge cases for `cols::` parameter |
| 34_touch.md | Edge cases for `touch::` parameter |
| 35_imodel.md | Edge cases for `imodel::` parameter |
| 36_effort.md | Edge cases for `effort::` parameter |
| 37_count.md | Edge cases for `count::` parameter |
| 38_offset.md | Edge cases for `offset::` parameter |
| 39_only_active.md | Edge cases for `only_active::` parameter |
| 40_only_next.md | Edge cases for `only_next::` parameter |
| 41_min_5h.md | Edge cases for `min_5h::` parameter |
| 42_min_7d.md | Edge cases for `min_7d::` parameter |
| 43_only_valid.md | Edge cases for `only_valid::` parameter |
| 44_exclude_exhausted.md | Edge cases for `exclude_exhausted::` parameter |
| 45_get.md | Edge cases for `get::` parameter |
| 46_abs.md | Edge cases for `abs::` parameter |
| 47_no_color.md | Edge cases for `no_color::` parameter |
| 48_host.md | Edge cases for `host::` parameter (`.account.save` metadata capture) |
| 49_at.md | Edge cases for `at::` parameter (`.account.renewal` absolute timestamp) |
| 50_from_now.md | Edge cases for `from_now::` parameter (`.account.renewal` relative delta) |
| 51_clear.md | Edge cases for `clear::` parameter (`.account.renewal` renewal removal) |
| 52_role.md | Edge cases for `role::` parameter (`.account.save` free-text metadata label) |
| 53_for.md | Edge cases for `for::` parameter (`USER@MACHINE` target identity for `.account.assign`) |
| 54_set_model.md | Edge cases for `set_model::` parameter (explicit session model override) |
| 55_set.md | Edge cases for `set::` parameter (`.model` mode selector: absent = get, present = set) |
| 57_unclaim.md | **REMOVED** — `unclaim::` removed from `.account.save`; see `command/18_account_unclaim.md` |

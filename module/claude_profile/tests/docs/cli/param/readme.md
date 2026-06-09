# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual clp parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: 53 active clp parameter edge case files (`name::` through `set_model::`, params 1–54 except param 2); all parameters covered.
- **Out of Scope**: Command-level tests (→ `command/`), parameter group interactions (→ `param_group/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| 001_name.md | Edge cases for `name::` parameter |
| 003_format.md | Edge cases for `format::` parameter |
| 004_threshold.md | Edge cases for `threshold::` parameter |
| 005_dry.md | Edge cases for `dry::` parameter |
| 006_account.md | Edge cases for `account::` parameter |
| 007_sub.md | Edge cases for `sub::` parameter |
| 008_tier.md | Edge cases for `tier::` parameter |
| 009_token.md | Edge cases for `token::` parameter |
| 010_expires.md | Edge cases for `expires::` parameter |
| 011_email.md | Edge cases for `email::` parameter (`.credentials.status` and `.accounts`) |
| 012_file.md | Edge cases for `file::` parameter |
| 013_saved.md | Edge cases for `saved::` parameter |
| 014_active.md | Edge cases for `active::` parameter |
| 015_display_name.md | Edge cases for `display_name::` parameter |
| 016_role.md | Edge cases for `role::` parameter |
| 017_billing.md | Edge cases for `billing::` parameter |
| 018_model.md | Edge cases for `model::` parameter |
| 019_refresh.md | Edge cases for `refresh::` parameter |
| 020_live.md | Edge cases for `live::` parameter |
| 021_interval.md | Edge cases for `interval::` parameter |
| 022_jitter.md | Edge cases for `jitter::` parameter |
| 023_trace.md | Edge cases for `trace::` parameter |
| 024_field.md | Edge cases for `field::` parameter |
| 025_sort.md | Edge cases for `sort::` parameter |
| 026_desc.md | Edge cases for `desc::` parameter |
| 027_prefer.md | Edge cases for `prefer::` parameter |
| 028_uuid.md | Edge cases for `uuid::` parameter |
| 029_capabilities.md | Edge cases for `capabilities::` parameter |
| 030_org_uuid.md | Edge cases for `org_uuid::` parameter |
| 031_org_name.md | Edge cases for `org_name::` parameter |
| 032_next.md | Edge cases for `next::` parameter |
| 033_cols.md | Edge cases for `cols::` parameter |
| 034_touch.md | Edge cases for `touch::` parameter |
| 035_imodel.md | Edge cases for `imodel::` parameter |
| 036_effort.md | Edge cases for `effort::` parameter |
| 037_count.md | Edge cases for `count::` parameter |
| 038_offset.md | Edge cases for `offset::` parameter |
| 039_only_active.md | Edge cases for `only_active::` parameter |
| 040_only_next.md | Edge cases for `only_next::` parameter |
| 041_min_5h.md | Edge cases for `min_5h::` parameter |
| 042_min_7d.md | Edge cases for `min_7d::` parameter |
| 043_only_valid.md | Edge cases for `only_valid::` parameter |
| 044_exclude_exhausted.md | Edge cases for `exclude_exhausted::` parameter |
| 045_get.md | Edge cases for `get::` parameter |
| 046_abs.md | Edge cases for `abs::` parameter |
| 047_no_color.md | Edge cases for `no_color::` parameter |
| 048_host.md | Edge cases for `host::` parameter (`.account.save` metadata capture) |
| 049_at.md | Edge cases for `at::` parameter (`.account.renewal` absolute timestamp) |
| 050_from_now.md | Edge cases for `from_now::` parameter (`.account.renewal` relative delta) |
| 051_clear.md | Edge cases for `clear::` parameter (`.account.renewal` renewal removal) |
| 052_role.md | Edge cases for `role::` parameter (`.account.save` free-text metadata label) |
| 053_for.md | Edge cases for `for::` parameter (`USER@MACHINE` target identity for `.account.assign`) |
| 054_set_model.md | Edge cases for `set_model::` parameter (explicit session model override) |

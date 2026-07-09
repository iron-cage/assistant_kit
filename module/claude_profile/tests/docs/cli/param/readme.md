# Parameter Tests

### Scope

- **Purpose**: Document edge case coverage for individual clp parameters.
- **Responsibility**: Index of per-parameter edge case test files covering parameter-level behavior.
- **In Scope**: 66 clp parameter edge case files — full coverage of all 66 production parameters in `docs/cli/param/` (61 active + 5 tombstones). `current::` (docs param 18) has its own dedicated EC file, `18_current.md`, per One Element One Spec — setup context is shared with `command/03_accounts.md` IT tests (IT-26, IT-27, IT-28) but the case list is complete and self-contained. Production param 56 (`unclaim::`) was previously REMOVED (Feature 036), re-activated on `.accounts`/`.usage` in Feature 037, and REMOVED AGAIN in Feature 064 (use `owner::0`) — file `57_unclaim.md` is a dual tombstone. File `53_for.md` is a TOMBSTONE — `for::` REMOVED (Feature 064; absorbed into `active::` value, which was then renamed to `assignee::` in Feature 065). File `58_assign.md` is a TOMBSTONE — `assign::` REMOVED (Feature 064; use `assignee::USER@MACHINE name::X`). File `14_active.md` is a TOMBSTONE — `active::` REMOVED (Feature 065; use `assignee::USER@MACHINE name::X`). Test file `64_assignee.md` covers param 063 (`assignee::`) introduced in Feature 065 with `assignee::0` sentinel. Test file `59_force.md` covers param 058 (`force::`) introduced in Feature 036/037. Test file `60_rotate.md` covers param 059 (`rotate::`) introduced in Feature 038. Test files `61_solo.md` and `62_who.md` cover params 060 (`solo::`) and 061 (`who::`) introduced in Feature 061 / Plan 022. Test file `63_owner.md` covers param 062 (`owner::`) introduced in Feature 063, extended with `owner::0` sentinel and batch comma-list in Feature 064. Test file `64_id.md` covers param 064 (`id::`) and `66_reset.md` covers param 066 (`reset::`), both introduced with `.model.select` (Feature 069). Test file `65_offline.md` covers param 065 (`offline::`) introduced with `.models` (Feature 068).
- **Numbering note**: Test doc numbering is offset +1 from `docs/cli/param/` from position 19 onward (test `19_refresh.md` ↔ production `019_refresh.md`), because production param 18 (`current::`) now has a same-numbered test file (`18_current.md`) sitting alongside `18_model.md` (production param 017) rather than absorbing its slot — the historical +1 shift for params 19–63 is preserved unchanged for file-identity stability; only the newly-filled `18_current.md` and the three appended `64_id.md`/`65_offline.md`/`66_reset.md` files use same-numbered production alignment.
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
| 14_active.md | **TOMBSTONE** — `active::` REMOVED (Feature 065); replaced by `assignee::USER@MACHINE name::X`; see [feature/065_assignee_param_redesign.md](../../../../docs/feature/065_assignee_param_redesign.md) |
| 15_display_name.md | Edge cases for `display_name::` parameter |
| 16_role.md | Edge cases for `role::` parameter |
| 17_billing.md | Edge cases for `billing::` parameter |
| 18_current.md | Edge cases for `current::` parameter (`.accounts` Current line, own case list per One Element One Spec) |
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
| 53_for.md | **TOMBSTONE** — `for::` REMOVED (Feature 064); functionality absorbed into `assignee::` value (via `active::` Feature 064, renamed Feature 065); see [feature/065_assignee_param_redesign.md](../../../../docs/feature/065_assignee_param_redesign.md) |
| 54_set_model.md | Edge cases for `set_model::` parameter (explicit session model override) |
| 55_set.md | Edge cases for `set::` parameter (`.model` mode selector: absent = get, present = set) |
| 57_unclaim.md | **TOMBSTONE** — `unclaim::` removed from `.account.save` (Feature 036); re-activated on `.accounts`/`.usage` in Feature 037; REMOVED AGAIN (Feature 064) — use `owner::0`; see [param/056_unclaim.md](../../../../docs/cli/param/056_unclaim.md) |
| 58_assign.md | **TOMBSTONE** — `assign::` REMOVED (Feature 064); use `assignee::USER@MACHINE name::X` (Feature 065); see [feature/065_assignee_param_redesign.md](../../../../docs/feature/065_assignee_param_redesign.md) |
| 59_force.md | Edge cases for `force::` parameter (G5–G8 ownership gate bypass — Feature 036/037) |
| 60_rotate.md | Edge cases for `rotate::` parameter (strategy-driven rotation on `.usage` — Feature 038) |
| 61_solo.md | Edge cases for `solo::` parameter (token conservation mode — current+owned only) |
| 62_who.md | Edge cases for `who::` parameter (sessions table visibility in `.usage`) |
| 63_owner.md | Edge cases for `owner::` parameter (Feature 063: explicit ownership set; Feature 064: `owner::0` sentinel + batch comma-list) |
| 64_assignee.md | Edge cases for `assignee::` parameter (Feature 065: renamed from `active::`; `assignee::0` sentinel = current machine) |
| 64_id.md | Edge cases for `id::` parameter (`.model.select` set-mode model pin — Feature 069) |
| 65_offline.md | Edge cases for `offline::` parameter (`.models` static catalog vs live API toggle — Feature 068) |
| 66_reset.md | Edge cases for `reset::` parameter (`.model.select` clears pinned subprocess model — Feature 069) |

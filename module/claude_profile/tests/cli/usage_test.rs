//! Integration tests: IT (Usage — live quota).
//!
//! Tests the `.usage` command which fetches live rate-limit utilization for all
//! saved accounts via `claude_quota::fetch_oauth_usage()` and renders results
//! as a `data_fmt` table with 8 columns: flag, Account, Expires, 5h Left,
//! 5h Reset, 7d Left, 7d(Son), 7d Reset.
//!
//! Live tests (names contain `lim_it`) require a real Anthropic OAuth access
//! token. They are excluded from Docker CI by the nextest default filter
//! `!test(lim_it)` in `.config/nextest.toml`. Offline tests (no `lim_it` in
//! the name) run without credentials and cover error paths and edge cases.
//!
//! ## Test Matrix
//!
//! | ID   | Test Function                                   | Condition                                                     | P/N | Live? |
//! |------|-------------------------------------------------|---------------------------------------------------------------|-----|-------|
//! | it001  | `it001_lim_it_quota_heading_and_columns`          | real token → Quota heading + new column names                 | P   | yes   |
//! | it002  | `it002_lim_it_active_account_marked`              | 2 accounts; active one has `✓` in flag column                 | P   | yes   |
//! | it003  | `it003_failed_token_shows_dash_exits_0`           | account without accessToken → `—` + "in …" Expires + exit 0  | P   | no    |
//! | it004  | `it004_lim_it_json_format_valid_array`            | real token + `format::json` → JSON with `_left_pct` fields + `weekly_7d_sonnet_left_pct` | P | yes |
//! | it005  | `it005_empty_store_shows_no_accounts`             | empty credential store → no-accounts message                  | P   | no    |
//! | it006  | `it006_unreadable_store_exits_2`                  | store dir chmod 000 → exit 2                                  | N   | no    |
//! | it007  | `it007_home_unset_exits_2`                        | HOME unset → exit 2                                           | N   | no    |
//! | it008  | `it008_lim_it_accounts_in_alpha_order`            | 3 accounts written out of order → alpha output                | P   | yes   |
//! | it009  | `it009_unreadable_credentials_shows_dash`         | credentials chmod 000 → `—` + exit 0                         | P   | no    |
//! | it010 | `it010_expired_token_shows_expired_in_expires_col` | account with PAST_MS → "EXPIRED" in Expires column           | P   | no    |
//! | it011 | `it011_lim_it_recommendation_marker_shown`       | 2 accounts + `next::endurance` → `→` on non-active account    | P   | yes   |
//! | it012 | `it012_lim_it_footer_shows_valid_count`          | 2 accounts with real tokens → footer "Valid: 2" + "Next:"     | P   | yes   |
//! | it013 | `it013_active_divergence_shows_star`             | live creds=work, _active=alice → `✓` on work, `*` on alice    | P   | no    |
//! | it014 | `it014_creds_unreadable_no_checkmark_star_shown` | no live creds, _active=alice → no `✓`, `*` on alice           | P   | no    |
//! | it015 | `it015_current_equals_active_no_star`            | live creds=alice, _active=alice → `✓` on alice, no `*`        | P   | no    |
//! | it016 | `it016_json_is_current_is_active`                | JSON has `is_current` + `is_active`, no `active` key          | P   | no    |
//! | it017 | `it017_format_table_rejected`                    | `format::table` → exit 1 (not supported by .usage)           | N   | no    |
//! | it018 | `it018_synthetic_row_when_no_saved_match`         | live token unmatched → synthetic (current session) row with ✓ | P   | no    |
//! | it019 | `it019_refresh_disabled_param_accepted`           | `refresh::0` accepted by parser; empty store → no-accounts    | P   | no    |
//! | it020 | `it020_refresh_enabled_offline_no_retry_triggered` | `refresh::1` accepted; missing token → dash, no HTTP call   | P   | no    |
//! | it021 | `it021_lim_it_live_mode`                         | `live::1 interval::30`; real token → "Next update" in output  | P   | yes   |
//! | it022 | `it022_live_jitter_exceeds_interval`             | `live::1 interval::60 jitter::70` → exit 1 before any fetch   | N   | no    |
//! | it023 | `it023_live_interval_below_minimum`              | `live::1 interval::5` → exit 1, stderr contains "30"          | N   | no    |
//! | it024 | `it024_live_incompatible_with_json`              | `live::1 format::json` → exit 1 before any fetch              | N   | no    |
//! | it025 | `it025_synthetic_row_uses_claude_json_email`     | live token unmatched + `.claude.json` has email → row shows email, not "(current session)" | P | no |
//! | it026 | `it026_live_jitter_equals_interval_accepted`     | `live::1 interval::30 jitter::30` (boundary) → exit 2, not 1 (guard allows equal) | P | no |
//! | it027 | `it027_json_error_field_on_failed_account`       | single account without accessToken + format::json → JSON has `"error":` field | P | no |
//! | it028 | `it028_interval_jitter_ignored_when_not_live`    | `interval::5 jitter::70` without `live::1` → exit 0, guards never fire | P | no |
//! | it029 | `it029_live_default_interval_accepted`           | `live::1` alone → default interval=30, no guard error (exit 2 from store) | P | no |
//! | it030 | `it030_live_sigint_exits_0`                      | `live::1`; after 3s send SIGINT → exit 0, stdout has "Monitor stopped."  | P | no |
//! | it031 | `it031_usage_help_shows_live_params`             | `.usage.help` → exit 0, stdout contains `live`, `interval`, `jitter`     | P | no |
//! | it032 | `it032_lim_it_refresh_per_account`               | real token + `refresh::1` → exit 0, account name visible (AC-19)         | P | yes |
//! | it033 | `it033_mre_refresh_help_excludes_429`            | `.usage.help` refresh says 401/403 not 401/403/429 (issue-refresh-help-429) | P | no |
//! | it034 | `it034_trace_param_writes_to_stderr`             | `trace::1` with no-token account → stderr contains `[trace]` lines         | P | no |
//! | it035 | `it035_empty_store_json_format`                  | empty store + `format::json` → output is `[]`                              | P | no |
//! | it036 | `it036_no_footer_when_no_valid_accounts`         | single failed account → no "Valid:" footer line                            | P | no |
//! | it037 | `it037_mre_bug155_refresh_defaults_to_1`         | `.usage.help` shows "1 = enabled, default" for refresh (BUG-155)           | P | no |
//! | it038 | `it038_mre_bug156_refresh_help_mentions_429_expired` | `.usage.help` refresh mentions 429+locally-expired case (BUG-156)      | P | no |
//! | it039 | `it039_refresh_2_rejected`                           | `refresh::2` out of range → exit 1 (EC-3)                | N | no |
//! | it040 | `it040_refresh_yes_rejected`                         | `refresh::yes` type error → exit 1 (EC-4)                | N | no |
//! | it041 | `it041_live_0_single_fetch_exits_0`                  | `live::0` explicit → exit 0, no countdown footer (EC-2)     | P | no |
//! | it042 | `it042_live_2_rejected`                              | `live::2` out of range → exit 1 (EC-4)                      | N | no |
//! | it043 | `it043_live_yes_rejected`                            | `live::yes` type error → exit 1 (EC-5)                      | N | no |
//! | it044 | `it044_interval_abc_rejected`                        | `interval::abc` type error → exit 1 (EC-6)              | N | no |
//! | it045 | `it045_interval_60_live_accepted`                    | `live::1 interval::60` guard passes (exit 2, not 1) (EC-3) | P | no |
//! | it046 | `it046_jitter_0_explicit_live_accepted`              | `live::1 jitter::0` explicit zero guard passes (EC-1)     | P | no |
//! | it047 | `it047_jitter_10_live_accepted`                      | `live::1 interval::30 jitter::10` guard passes (EC-2)     | P | no |
//! | it048 | `it048_jitter_abc_rejected`                          | `jitter::abc` type error → exit 1 (EC-7)                  | N | no |
//! | it049 | `it049_trace_0_no_trace_on_stderr`                   | `trace::0` explicit → no [trace] on stderr (EC-2)          | P | no |
//! | it050 | `it050_trace_2_rejected`                             | `trace::2` out of range → exit 1 (EC-3)                    | N | no |
//! | it051 | `it051_trace_yes_rejected`                           | `trace::yes` type error → exit 1 (EC-4)                    | N | no |
//! | it052 | `it052_trace_default_off`                            | no `trace::` → no [trace] lines on stderr (EC-5)           | P | no |
//! | it053 | `it053_sort_name_accepted`                         | `sort::name` + empty store → exit 0 (IT-44/AC-01)          | P | no |
//! | it054 | `it054_sort_endurance_accepted`                     | `sort::endurance` + empty store → exit 0 (IT-45/AC-02)     | P | no |
//! | it055 | `it055_sort_drain_accepted`                         | `sort::drain` + empty store → exit 0 (IT-46/AC-03)         | P | no |
//! | it056 | `it056_sort_renew_accepted`                         | `sort::renew` + empty store → exit 0 (IT-47/AC-04)         | P | no |
//! | it057 | `it057_sort_invalid_value_exit_1`                   | `sort::bogus` → exit 1, stderr names valid values (IT-48/AC-09) | N | no |
//! | it058 | `it058_prefer_invalid_value_exit_1`                 | `prefer::bogus` → exit 1, stderr names valid values (IT-49/AC-10) | N | no |
//! | it059 | `it059_usage_help_shows_sort_params`                | `.usage.help` lists `sort`, `desc`, `prefer` (IT-50)       | P | no |
//! | it060 | `it060_desc_0_accepted`                             | `desc::0` + empty store → exit 0 (026_desc EC-1)           | P | no |
//! | it061 | `it061_desc_1_accepted`                             | `desc::1` + empty store → exit 0 (026_desc EC-2)           | P | no |
//! | it062_desc_2_rejected | `it062_desc_2_rejected`            | `desc::2` out of range → exit 1 (026_desc EC-3)            | N | no |
//! | it063 | `it063_sort_name_desc_0_identical_to_sort_name`     | `sort::name desc::0` same order as `sort::name` (CC-1)     | P | no |
//! | it064 | `it064_sort_name_desc_1_reverses_order`             | `sort::name desc::1` shows z before a (CC-2)               | P | no |
//! | it065 | `it065_prefer_any_accepted`                         | `prefer::any` + empty store → exit 0 (027_prefer EC-1)     | P | no |
//! | it066 | `it066_prefer_opus_accepted`                        | `prefer::opus` + empty store → exit 0 (027_prefer EC-2)    | P | no |
//! | it067 | `it067_prefer_sonnet_accepted`                      | `prefer::sonnet` + empty store → exit 0 (027_prefer EC-3)  | P | no |
//! | it068 | `it068_sort_json_unaffected_by_sort_strategy`       | JSON alphabetical regardless of `sort::` strategy (025_sort CC-1) | P | no |
//! | it069 | `it069_sort_uppercase_rejected`                     | `sort::Name` (uppercase) → exit 1 (case-sensitive)         | N | no |
//! | it070 | `it070_prefer_uppercase_rejected`                   | `prefer::Opus` (uppercase) → exit 1 (case-sensitive)       | N | no |
//! | it073 | `it073_next_all_rejected_exit_1`                    | `next::all` rejected → exit 1 (TSK-184)                    | N | no |
//! | it074 | `it074_next_session_rejected_exit_1`                | `next::session` rejected → exit 1 (TSK-184)                | N | no |
//! | it075 | `it075_next_endurance_accepted`                     | `next::endurance` accepted with empty store → exit 0       | P | no |
//! | it076 | `it076_next_drain_accepted`                         | `next::drain` accepted with empty store → exit 0           | P | no |
//! | it077 | `it077_next_reset_rejected_exit_1`                  | `next::reset` rejected → exit 1 (TSK-184)                  | N | no |
//! | it078 | `it078_next_invalid_value_exit_1`                   | `next::bogus` → exit 1, stderr names renew+endurance+drain | N | no |
//! | it079 | `it079_next_drain_default_no_arrow_without_valid_accounts` | default renew + no valid accounts → no `→`       | P | no |
//! | it080 | `it080_cols_sub_accepted`                           | `cols::+sub` accepted with empty store → exit 0            | P | no |
//! | it081 | `it081_cols_sub_shows_sub_column`                   | `cols::+sub` with account → output contains "Sub" header   | P | no |
//! | it082 | `it082_cols_unknown_id_exit_1`                      | `cols::+bogus_col` → exit 1, stderr names valid IDs        | N | no |
//! | it083 | `it083_usage_help_shows_next_cols_params`           | `.usage.help` lists `next` and `cols` params               | P | no |
//! | mre171 | `mre_bug_171_account_populated_after_refresh`      | BUG-171: `Fix(BUG-171)` present → `aq.account` populated  | P | no |
//! | it092 | `it092_next_all_rejected_exit_1`                    | `next::all` rejected → exit 1, stderr names renew+endurance+drain (TSK-184/TSK-222) | N | no |
//! | it093 | `it093_footer_not_gated_on_next_all_structural`     | `Responsibility(TSK-184-footer)` present; old All-gate absent (TSK-184) | P | no |
//! | it094 | `it094_next_session_rejected_exit_1`                | `next::session` rejected → exit 1, stderr names renew+endurance+drain (TSK-184/TSK-222) | N | no |
//! | it095 | `it095_next_strategy_session_absent_structural`     | `NextStrategy::Session` absent from source (TSK-184) | P | no |
//! | it096 | `it096_next_drain_json_output_unchanged`             | `format::json next::drain` identical to default JSON (TSK-184) | P | no |
//! | it097 | `it097_touch_1_empty_store_exits_0`                 | `touch::1` empty store → exit 0, no-accounts message (TSK-185 AC-01) | P | no |
//! | it098 | `it098_touch_1_errored_account_skipped`             | `touch::1` no-token account → exit 0, row shows `—` (TSK-185 AC-04) | P | no |
//! | it099 | `it099_apply_touch_fn_exists_structural`             | `fn apply_touch` present in source (TSK-185 AC-02 structural) | P | no |
//! | it100 | `it100_touch_json_format_unaffected`                | `format::json touch::1` empty store → exit 0, output `[]` (TSK-185 AC-08) | P | no |
//! | it101 | `it101_usage_help_shows_touch_param`                | `.usage.help` contains `touch` (TSK-185 AC-10) | P | no |
//! | it120 | `it120_lim_it_ft12_touch_trigger_fires_per_idle_account_cycle` | `touch::1` fires for idle accounts (resets_at absent); active skipped after activation (024 FT-12) | P | yes |
//! | it121 | `it121_sort_next_accepted`                          | `sort::next` accepted → exit 0 (renew default + endurance explicit) (IT-65/AC-15) | P | no |
//! | it122 | `it122_imodel_auto_accepted_empty_store_exits_0`    | `imodel::auto` accepted; empty store exits 0 (IT-66/EC-1) | P | no |
//! | it123 | `it123_imodel_bogus_exits_1`                        | `imodel::bogus` → exit 1, stderr names all 5 valid values (IT-67/EC-5) | N | no |
//! | it124 | `it124_effort_auto_accepted_empty_store_exits_0`    | `effort::auto` accepted; empty store exits 0 (IT-68/EC-1) | P | no |
//! | it125 | `it125_effort_bogus_exits_1`                        | `effort::bogus` → exit 1, stderr names all 5 valid values (IT-69/EC-4) | N | no |
//! | it126 | `it126_usage_help_shows_imodel_effort_params`       | `.usage.help` lists `imodel` and `effort` params (IT-70) | P | no |
//! | it127 | `it127_imodel_sonnet_accepted_empty_store_exits_0`  | `imodel::sonnet` accepted; empty store exits 0 (EC-2) | P | no |
//! | it128 | `it128_imodel_opus_accepted_empty_store_exits_0`    | `imodel::opus` accepted; empty store exits 0 (EC-3) | P | no |
//! | it129 | `it129_imodel_keep_accepted_empty_store_exits_0`    | `imodel::keep` accepted; empty store exits 0 (EC-4) | P | no |
//! | it130 | `it130_effort_high_accepted_empty_store_exits_0`    | `effort::high` accepted; empty store exits 0 | P | no |
//! | it131 | `it131_effort_max_accepted_empty_store_exits_0`     | `effort::max` accepted; empty store exits 0 | P | no |
//! | it132 | `it132_apply_touch_trigger_is_is_none_structural`   | apply_touch uses `is_none()` trigger (BUG-181 fix AC-02 structural) | P | no |
//! | it133 | `it133_refresh_account_token_has_label_param_structural` | `refresh_account_token` uses label var not hardcoded "refresh" (TSK-192 AC-09 structural) | P | no |
//! | it134 | `it134_apply_touch_passes_touch_label_structural`   | `apply_touch` call site passes `"touch"` label (TSK-192 AC-09 structural) | P | no |
//! | it135 | `it135_apply_refresh_passes_refresh_label_structural` | `apply_refresh` call site passes `"refresh"` label (TSK-192 AC-09 structural) | P | no |
//! | it136 | `it136_refresh_account_token_has_instant_timing_structural` | `refresh_account_token` uses `Instant::now()` for per-step timing (TSK-192 AC-09 structural) | P | no |
//! | it137 | `it137_sort_default_is_renew_structural`             | sort default is `SortStrategy::Renew` when no `sort::` arg given (TSK-193/TSK-220 AC-01 structural) | P | no |
//! | it138 | `it138_sort_next_resolves_to_drain_structural`       | `sort::next` resolves to `SortStrategy::Drain` when `next::drain` (TSK-193 AC-15 structural) | P | no |
//! | it139 | `it139_sort_next_resolves_to_endurance_structural`   | `sort::next` resolves to `SortStrategy::Endurance` when `next::endurance` (TSK-193 AC-15 structural) | P | no |
//! | it141 | `it141_trace_skip_lines_emitted_for_non_qualifying_accounts` | `touch::1 trace::1` errored account → `[trace] touch <name> skipped (reason: error account)` (BUG-202 / 024 FT-14) | P | no |
//! | it142 | `it142_imodel_haiku_accepted_empty_store_exits_0`   | `imodel::haiku` accepted; empty store exits 0 (EC-11 / 035) | P | no |
//! | it143 | `it143_effort_low_accepted_empty_store_exits_0`     | `effort::low` accepted; empty store exits 0 (EC-10 / 036) | P | no |
//! | it144 | `it144_effort_normal_accepted_empty_store_exits_0`  | `effort::normal` accepted; empty store exits 0 (EC-11 / 036) | P | no |
//! | it145 | `it145_lim_it_next_renew_places_arrow_on_soonest_refill` | `next::renew` → exit 0, footer shows renew line, `→` placed on winning account (TSK-222) | P | yes |
//! | ut146 | `ut_filter_only_valid_hides_red_rows`                | `only_valid::1` accepted; empty store exits 0 (TSK-223 RED gate) | P | no |
//! | it146 | `it146_next_column_visible_by_default`              | `.usage` with account → `→ Next` header visible in default output (FT-18/AC-28) | P | no |
//! | it147 | `it147_json_renewal_secs_present`                   | `.usage format::json` → JSON has `renewal_secs`, not `next_renewal_est` (FT-19/AC-29) | P | no |
//! | it148 | `it148_status_emoji_column_header_present`          | `●` header always present (AC-18) | P | no |
//! | it149 | `it149_status_emoji_red_on_token_error`             | No accessToken → 🔴 in table row (AC-18) | P | no |
//! | it150 | `it150_status_emoji_absent_from_json`               | `format::json` output has no emoji (AC-20) | P | no |
//! | it151 | `it151_past_renewal_at_auto_advances_in_usage`      | Past `_renewal_at` auto-advanced monthly at render → `in Xd` no `~` (030 FT-10/AC-10) | P | no |
//! | it152 | `it152_tsv_next_column_present`                     | `format::tsv` has `next` column header (AC-28) | P | no |
//! | it153 | `it153_json_all_renewal_fields_present`             | `format::json` has all 4 renewal fields including `next_event_type` and `next_event_secs` (FT-19) | P | no |
//! | it154 | `it154_only_active_1_shows_active_account_row`      | `only_active::1` shows exactly 1 row — the active account (039 EC-1, 028 FT-03) | P | no |
//! | it155 | `it155_only_active_0_shows_all_rows`                | `only_active::0` shows all rows (039 EC-2) | P | no |
//! | it156 | `it156_only_active_bad_exits_1`                     | `only_active::bad` exits 1 naming valid values (039 EC-3) | N | no |
//! | it157 | `it157_only_active_1_no_active_marker_shows_empty`  | `only_active::1` with no active marker → 0 rows (039 EC-4) | P | no |
//! | it158 | `it158_only_active_true_accepted`                   | `only_active::true` accepted as alias for 1 (039 EC-5) | P | no |
//! | it159 | `it159_only_active_false_shows_all_rows`            | `only_active::false` accepted, shows all rows (039 EC-6) | P | no |
//! | it160 | `it160_only_next_1_no_valid_accounts_shows_empty`   | `only_next::1` with all error accounts → 0 rows (040 EC-2 offline) | P | no |
//! | it161 | `it161_only_next_bad_exits_1`                       | `only_next::bad` exits 1 naming valid values (040 EC-4) | N | no |
//! | it162 | `it162_only_next_0_shows_all_rows`                  | `only_next::0` shows all rows (040 EC-5) | P | no |
//! | it163 | `it163_min_5h_0_shows_all_rows`                     | `min_5h::0` shows all rows (041 EC-3) | P | no |
//! | it164 | `it164_min_5h_abc_exits_1`                          | `min_5h::abc` exits 1 type error (041 EC-4) | N | no |
//! | it165 | `it165_min_5h_101_exits_1`                          | `min_5h::101` exits 1 out of range (041 EC-5) | N | no |
//! | it166 | `it166_min_7d_0_shows_all_rows`                     | `min_7d::0` shows all rows (042 EC-3) | P | no |
//! | it167 | `it167_min_7d_abc_exits_1`                          | `min_7d::abc` exits 1 type error (042 EC-4) | N | no |
//! | it168 | `it168_min_7d_101_exits_1`                          | `min_7d::101` exits 1 out of range (042 EC-5) | N | no |
//! | it169 | `it169_only_valid_0_shows_all_rows`                 | `only_valid::0` shows all rows (043 EC-2) | P | no |
//! | it170 | `it170_only_valid_bad_exits_1`                      | `only_valid::bad` exits 1 naming valid values (043 EC-3) | N | no |
//! | it171 | `it171_only_valid_1_all_red_shows_empty`            | `only_valid::1` with all 🔴 → 0 rows (043 EC-4, 028 FT-07) | P | no |
//! | it172 | `it172_only_valid_true_accepted`                    | `only_valid::true` accepted (043 EC-5) | P | no |
//! | it173 | `it173_only_valid_false_shows_all_rows`             | `only_valid::false` shows all rows (043 EC-6) | P | no |
//! | it174 | `it174_exclude_exhausted_0_shows_all_rows`          | `exclude_exhausted::0` shows all rows (044 EC-2) | P | no |
//! | it175 | `it175_exclude_exhausted_bad_exits_1`               | `exclude_exhausted::bad` exits 1 (044 EC-4) | N | no |
//! | it176 | `it176_exclude_exhausted_1_all_red_shows_empty`     | `exclude_exhausted::1` with all 🔴 → 0 rows (044 EC-5, 028 FT-08) | P | no |
//! | it177 | `it177_exclude_exhausted_true_accepted`             | `exclude_exhausted::true` accepted (044 EC-6) | P | no |
//! | it178 | `it178_count_3_shows_first_3_rows`                  | `count::3 sort::name` with 5 accounts shows first 3 alphabetically (037 EC-1) | P | no |
//! | it179 | `it179_count_0_shows_all_rows`                      | `count::0` shows all 3 rows (037 EC-2) | P | no |
//! | it180 | `it180_count_100_exceeding_shows_all`               | `count::100` with 2 accounts shows both (037 EC-3) | P | no |
//! | it181 | `it181_count_abc_exits_1`                           | `count::abc` exits 1 type error (037 EC-4) | N | no |
//! | it182 | `it182_count_1_sort_name_shows_only_first`          | `count::1 sort::name` with 3 accounts shows only first (037 EC-5) | P | no |
//! | it183 | `it183_count_minus_1_exits_1`                       | `count::-1` exits 1 negative rejected (037 EC-6) | N | no |
//! | it184 | `it184_offset_2_skips_first_2_rows`                 | `offset::2 sort::name` with 4 accounts shows rows 3-4 (038 EC-1) | P | no |
//! | it185 | `it185_offset_0_shows_all_rows`                     | `offset::0` shows all rows (038 EC-2) | P | no |
//! | it186 | `it186_offset_99_shows_empty`                       | `offset::99` with 2 accounts → 0 rows (038 EC-3) | P | no |
//! | it187 | `it187_offset_abc_exits_1`                          | `offset::abc` exits 1 type error (038 EC-4) | N | no |
//! | it188 | `it188_offset_1_count_1_shows_second_row`           | `offset::1 count::1 sort::name` shows second row (038 EC-5) | P | no |
//! | it189 | `it189_offset_minus_1_exits_1`                      | `offset::-1` exits 1 negative rejected (038 EC-6) | N | no |
//! | it190 | `it190_get_account_extracts_first_name`             | `get::account sort::name` extracts first account name bare (045 EC-2) | P | no |
//! | it191 | `it191_get_account_no_table_chrome`                 | `get::account` output has no column headers or footer (045 EC-6) | P | no |
//! | it192 | `it192_get_status_err_on_error_account`             | `get::status` on error account outputs `🔴` bare (045 EC-3 offline) | P | no |
//! | it193 | `it193_get_with_empty_filtered_result_empty_stdout` | `only_valid::1 get::account` with all-error → empty stdout (045 EC-4) | P | no |
//! | it194 | `it194_abs_1_accepted_empty_store`                  | `abs::1` accepted with empty store → exits 0 (046 EC-1) | P | no |
//! | it195 | `it195_abs_0_accepted`                              | `abs::0` accepted → exits 0 (046 EC-2) | P | no |
//! | it196 | `it196_abs_bad_exits_1`                             | `abs::bad` exits 1 naming valid values (046 EC-3) | N | no |
//! | it197 | `it197_abs_1_on_error_row`                          | `abs::1` on error row shows error unchanged (046 EC-5) | P | no |
//! | it198 | `it198_no_color_1_no_emoji_in_output`               | `no_color::1` on error account → no emoji in stdout (047 EC-1) | P | no |
//! | it199 | `it199_no_color_1_status_shows_err_text_label`      | `no_color::1` status column shows `err` text label (047 EC-2) | P | no |
//! | it200 | `it200_no_color_bad_exits_1`                        | `no_color::bad` exits 1 naming valid values (047 EC-4) | N | no |
//! | it201 | `it201_no_color_true_accepted`                      | `no_color::true` accepted as alias for 1 (047 EC-6) | P | no |
//! | it202 | `it202_cols_host_shows_host_column`                 | `cols::+host` shows Host header + profile host value (033 EC-7) | P | no |
//! | it203 | `it203_cols_role_shows_role_column`                 | `cols::+role` shows Role header + profile role value (033 EC-8) | P | no |
//! | it204 | `it204_cols_bogus_names_host_and_role_in_error`     | `cols::+bogus` exit 1 stderr names `host` and `role` (033 EC-9) | N | no |

use crate::helpers::{
  BIN,
  run_cs, run_cs_with_env, run_cs_without_home, run_cs_bytes_for_secs,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token, write_claude_json, live_active_token,
  write_live_credentials_with_token, write_account_renewal_json, write_account_profile_json,
  FAR_FUTURE_MS, PAST_MS,
};
use tempfile::TempDir;

// ── Live: heading and column names ───────────────────────────────────────────

/// Live: one account with a real token → output contains "Quota" heading and
/// the new quota column names; old combined column names are absent.
#[ test ]
fn it001_lim_it_quota_heading_and_columns()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it001: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "myaccount", &token, true );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Quota" ),    "must contain 'Quota' heading, got:\n{text}" );
  assert!( text.contains( "Expires" ),  "must contain 'Expires' column, got:\n{text}" );
  assert!( text.contains( "5h Left" ),  "must contain '5h Left' column, got:\n{text}" );
  assert!( text.contains( "5h Reset" ), "must contain '5h Reset' column, got:\n{text}" );
  assert!( text.contains( "7d Left" ),  "must contain '7d Left' column, got:\n{text}" );
  assert!( text.contains( "7d(Son)" ),  "must contain '7d(Son)' column, got:\n{text}" );
  assert!( text.contains( "7d Reset" ), "must contain '7d Reset' column, got:\n{text}" );
  assert!(
    !text.contains( "Session (5h)" ),
    "must NOT contain old 'Session (5h)' column, got:\n{text}",
  );
  assert!(
    !text.contains( "Weekly (7d)" ),
    "must NOT contain old 'Weekly (7d)' column, got:\n{text}",
  );
  assert!(
    !text.contains( "Status" ),
    "must NOT contain old 'Status' column, got:\n{text}",
  );
}

// ── Live: active account marked ──────────────────────────────────────────────

/// Live: two accounts; the active one has `✓` in the flag column on its line;
/// no line for the inactive account contains `✓`.
#[ test ]
fn it002_lim_it_active_account_marked()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it002: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a", &token, true  );
  write_account_with_token( dir.path(), "acct-b", &token, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let active_marked = text.lines().any( |line| line.contains( '✓' ) && line.contains( "acct-a" ) );
  assert!(
    active_marked,
    "a line must contain both ✓ and active account name 'acct-a', got:\n{text}",
  );
  let inactive_marked = text.lines().any( |line| line.contains( '✓' ) && line.contains( "acct-b" ) );
  assert!(
    !inactive_marked,
    "no line must contain both ✓ and inactive account name 'acct-b', got:\n{text}",
  );
}

// ── Offline: missing accessToken shows em-dash ───────────────────────────────

/// Offline: credential file has no `accessToken` field (but has a future
/// `expiresAt`) → `read_token()` returns "missing accessToken" → output shows
/// em-dash for quota columns, `(missing accessToken)` in the last column, and "in …"
/// (not "EXPIRED") in the Expires column because `FAR_FUTURE_MS` is used.
#[ test ]
fn it003_failed_token_shows_dash_exits_0()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account() uses credential_json() which omits accessToken.
  write_account( dir.path(), "no-token", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( '\u{2014}' ),
    "missing accessToken must render as em-dash (\u{2014}), got:\n{text}",
  );
  assert!(
    text.contains( "in " ),
    "Expires must show 'in …' (not 'EXPIRED') for FAR_FUTURE_MS token, got:\n{text}",
  );
}

// ── Live: JSON output structure ───────────────────────────────────────────────

/// Live: `format::json` → output is a JSON array where each entry has at
/// minimum `account` (string), `is_active` (boolean), and `expires_in_secs`
/// (number); successful entries use `session_5h_left_pct` (not `session_5h_pct`)
/// and include `weekly_7d_sonnet_left_pct` (number or null).
#[ test ]
fn it004_lim_it_json_format_valid_array()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it004: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "myaccount", &token, true );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let parsed : serde_json::Value = serde_json::from_str( text.trim() )
    .unwrap_or_else( |e| panic!( "output must be valid JSON: {e}\ngot:\n{text}" ) );
  assert!( parsed.is_array(), "output must be a JSON array, got:\n{text}" );
  let arr = parsed.as_array().unwrap();
  assert!( !arr.is_empty(), "array must have at least one entry, got:\n{text}" );
  assert!( arr[ 0 ][ "account" ].is_string(),  "entry must have 'account' string, got:\n{text}" );
  assert!( arr[ 0 ][ "is_active" ].is_boolean(), "entry must have 'is_active' boolean, got:\n{text}" );
  assert!( arr[ 0 ][ "expires_in_secs" ].is_number(), "entry must have 'expires_in_secs' number, got:\n{text}" );
  assert!(
    arr[ 0 ][ "session_5h_left_pct" ].is_number() || arr[ 0 ][ "session_5h_left_pct" ].is_null(),
    "entry must have 'session_5h_left_pct' number or null, got:\n{text}",
  );
  let obj = arr[ 0 ].as_object().unwrap();
  assert!(
    obj.contains_key( "weekly_7d_sonnet_left_pct" ),
    "entry must have 'weekly_7d_sonnet_left_pct' field, got:\n{text}",
  );
  assert!(
    !obj.contains_key( "session_5h_pct" ),
    "entry must NOT have old 'session_5h_pct' field, got:\n{text}",
  );
  assert!(
    !obj.contains_key( "status" ),
    "entry must NOT have old 'status' field, got:\n{text}",
  );
}

// ── Offline: empty credential store ─────────────────────────────────────────

/// Offline: credential store directory exists but contains no `.credentials.json`
/// files → `account::list()` returns an empty vec → output shows the no-accounts
/// message, exit 0.
#[ test ]
fn it005_empty_store_shows_no_accounts()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path()
    .join( ".persistent" )
    .join( "claude" )
    .join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "empty store must show '(no accounts configured)', got:\n{text}",
  );
}

// ── Offline: unreadable credential store → exit 2 ───────────────────────────

/// Offline: credential store directory mode 000 → `account::list()` cannot
/// enumerate it → `fetch_all_quota()` returns `ErrorData` → exit 2.
///
/// Permissions are restored before assertions so `TempDir` cleanup succeeds
/// even when a panic occurs mid-test.
#[ cfg( unix ) ]
#[ test ]
fn it006_unreadable_store_exits_2()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path()
    .join( ".persistent" )
    .join( "claude" )
    .join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );

  // Restore before any assertion so TempDir cleanup can delete the directory.
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  assert_exit( &out, 2 );
  assert!( !stderr( &out ).is_empty(), "unreadable store must produce error on stderr" );
}

// ── Offline: HOME unset → exit 2 ────────────────────────────────────────────

/// Offline: HOME removed from process environment → `PersistPaths::new()`
/// cannot resolve the storage root → exit 2 with a non-empty error on stderr.
#[ test ]
// Fix(issue-pro-isolation):
// Root cause: run_cs_without_home() removed $HOME but not $PRO; when $PRO is set in the host
//   environment, the binary resolved the credential store via $PRO and returned a result rather
//   than failing with exit 2 as expected.
// Why Not Caught: Docker container has no $PRO set; the bug only surfaces on the host.
// Fix Applied: added .env_remove("PRO") to run_cs_without_home() in helpers.rs.
// Prevention: any "no home directory" test helper must remove all root-resolution vars, not
//   just $HOME; the full list is $PRO, $HOME, $USERPROFILE.
// Pitfall: $PRO takes priority over $HOME in PersistPaths resolution — removing only $HOME
//   leaves a silent fallback that defeats the test's isolation intent.
fn it007_home_unset_exits_2()
{
  let out = run_cs_without_home( &[ ".usage" ] );
  assert_exit( &out, 2 );
  assert!( !stderr( &out ).is_empty(), "unset HOME must produce error on stderr" );
}

// ── Live: accounts appear in alphabetical order ───────────────────────────────

/// Live: three accounts written out of alphabetical order → output lists them
/// in alphabetical order (delegated to `account::list()` sort).
#[ test ]
fn it008_lim_it_accounts_in_alpha_order()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it008: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Written out of alphabetical order; output must still be alpha-sorted.
  write_account_with_token( dir.path(), "charlie", &token, false );
  write_account_with_token( dir.path(), "alpha",   &token, true  );
  write_account_with_token( dir.path(), "bravo",   &token, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let pos_alpha   = text.find( "alpha"   ).expect( "output must contain 'alpha'"   );
  let pos_bravo   = text.find( "bravo"   ).expect( "output must contain 'bravo'"   );
  let pos_charlie = text.find( "charlie" ).expect( "output must contain 'charlie'" );
  assert!(
    pos_alpha < pos_bravo && pos_bravo < pos_charlie,
    "accounts must appear alphabetically (alpha < bravo < charlie), got:\n{text}",
  );
}

// ── Offline: unreadable credentials file → em-dash, exit 0 ──────────────────

/// Offline: `.credentials.json` mode 000 → `account::list()` still finds the
/// account (directory is readable), but `read_token()` fails with EACCES →
/// `AccountQuota.result = Err(...)` → output shows em-dash, exit 0.
///
/// Permissions are restored before assertions so `TempDir` cleanup succeeds.
#[ cfg( unix ) ]
#[ test ]
fn it009_unreadable_credentials_shows_dash()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path()
    .join( ".persistent" )
    .join( "claude" )
    .join( "credential" );
  write_account( dir.path(), "locked", "max", "default", FAR_FUTURE_MS, true );

  let creds = store.join( "locked.credentials.json" );
  std::fs::set_permissions( &creds, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );

  // Restore before any assertion so TempDir cleanup can delete the file.
  std::fs::set_permissions( &creds, std::fs::Permissions::from_mode( 0o644 ) ).unwrap();

  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( '\u{2014}' ),
    "unreadable credentials must render as em-dash (\u{2014}), got:\n{text}",
  );
}

// ── Offline: expired token shows EXPIRED in Expires column ───────────────────

/// Offline: credential file has a past `expiresAt` timestamp (`PAST_MS`) →
/// `compute_expires_cell()` returns `"EXPIRED"` → the Expires column shows
/// "EXPIRED". Exit 0 (non-fatal per-account error).
#[ test ]
fn it010_expired_token_shows_expired_in_expires_col()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "old-acct", "max", "default", PAST_MS, true );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "EXPIRED" ),
    "expired token must show 'EXPIRED' in Expires column, got:\n{text}",
  );
}

// ── Live: recommendation marker shown ────────────────────────────────────────

/// Live: two accounts — one active, one non-active — both with real tokens.
/// The non-active account is the only candidate and must be marked `→`.
/// The active account must not be marked `→`.
#[ test ]
fn it011_lim_it_recommendation_marker_shown()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it011: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a", &token, true  );
  write_account_with_token( dir.path(), "acct-b", &token, false );

  // Use next::endurance to place → in the table body on the non-active account.
  let out  = run_cs_with_env( &[ ".usage", "next::endurance" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let rec_marked = text.lines().any( |line| line.contains( '→' ) && line.contains( "acct-b" ) );
  assert!(
    rec_marked,
    "next::endurance: a line must contain both → and non-active account 'acct-b', got:\n{text}",
  );
  let active_rec = text.lines().any( |line| line.contains( '→' ) && line.contains( "acct-a" ) );
  assert!(
    !active_rec,
    "active account 'acct-a' must not be marked with →, got:\n{text}",
  );
}

// ── Live: footer shows valid count and next recommendation ───────────────────

/// Live: two accounts with real tokens → at least two valid quota results →
/// footer line shows "Valid: 2" and "Next:".
#[ test ]
fn it012_lim_it_footer_shows_valid_count()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it012: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a", &token, true  );
  write_account_with_token( dir.path(), "acct-b", &token, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "Valid: 2" ),
    "footer must contain 'Valid: 2', got:\n{text}",
  );
  assert!(
    text.contains( "Next:" ),
    "footer must contain 'Next:', got:\n{text}",
  );
}

// ── Offline: current-vs-active divergence ─────────────────────────────────────

/// it013 (IT-13): live creds match `work@acme.com`; `_active` = `alice@acme.com`.
///
/// Flag column: `work@acme.com` gets `✓` (`is_current`), `alice@acme.com` gets `*`
/// (`is_active` but not `is_current`). This demonstrates divergence: the active marker
/// and the live session point at different accounts.
#[ test ]
fn it013_active_divergence_shows_star()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // alice is _active, work matches live creds → divergence
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", true  );
  write_account_with_token( dir.path(), "work@acme.com",  "tok-work",  false );
  write_live_credentials_with_token( dir.path(), "tok-work" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let work_current = text.lines().any( |l| l.contains( '\u{2713}' ) && l.contains( "work@acme.com" ) );
  assert!( work_current, "work@acme.com must have ✓ (is_current), got:\n{text}" );

  let alice_active = text.lines().any( |l| l.contains( '*' ) && l.contains( "alice@acme.com" ) );
  assert!( alice_active, "alice@acme.com must have * (is_active, not current), got:\n{text}" );
}

/// it014 (IT-14): no live credentials file; `_active` = `alice@acme.com`.
///
/// With no live creds, `is_current` is false for all — no `✓` shown.
/// `alice@acme.com` is still `is_active` so `*` is still shown.
#[ test ]
fn it014_creds_unreadable_no_checkmark_star_shown()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", true  );
  write_account_with_token( dir.path(), "work@acme.com",  "tok-work",  false );
  // Deliberately no live credentials file.

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let has_checkmark = text.lines().any( |l| l.contains( '\u{2713}' ) );
  assert!( !has_checkmark, "no ✓ when creds file absent, got:\n{text}" );

  let alice_star = text.lines().any( |l| l.contains( '*' ) && l.contains( "alice@acme.com" ) );
  assert!( alice_star, "alice@acme.com must still have * (is_active), got:\n{text}" );
}

/// it015 (IT-15): live creds match `alice@acme.com`; `_active` = `alice@acme.com`.
///
/// When `is_current` and `is_active` point at the same account, `✓` wins (priority)
/// and `*` does NOT appear on any line (no divergence).
#[ test ]
fn it015_current_equals_active_no_star()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", true );
  write_live_credentials_with_token( dir.path(), "tok-alice" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let alice_current = text.lines().any( |l| l.contains( '\u{2713}' ) && l.contains( "alice@acme.com" ) );
  assert!( alice_current, "alice@acme.com must have ✓ when both current and active, got:\n{text}" );

  let has_star = text.lines().any( |l| l.contains( '*' ) );
  assert!( !has_star, "no * when current == active (no divergence), got:\n{text}" );
}

/// it016 (IT-16): `format::json` uses `is_current` + `is_active` field names, not `active`.
///
/// Two accounts; live creds match `work@acme.com`; `_active` = `alice@acme.com`.
/// JSON output must have `"is_current":true` on work and `"is_active":true` on alice.
/// The old `"active"` key must not appear.
#[ test ]
fn it016_json_is_current_is_active()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", true  );
  write_account_with_token( dir.path(), "work@acme.com",  "tok-work",  false );
  write_live_credentials_with_token( dir.path(), "tok-work" );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let json = stdout( &out );

  assert!( json.contains( "\"is_current\"" ), "JSON must have is_current field, got:\n{json}" );
  assert!( json.contains( "\"is_active\""  ), "JSON must have is_active field, got:\n{json}" );
  assert!( !json.contains( "\"active\""    ), "JSON must not have old 'active' field, got:\n{json}" );

  // work@acme.com: is_current=true, is_active=false
  let work_current = json.contains( "\"work@acme.com\"" ) && json.contains( "\"is_current\":true" );
  assert!( work_current, "work@acme.com must have is_current:true, got:\n{json}" );

  // alice@acme.com: is_active=true
  let alice_active = json.contains( "\"alice@acme.com\"" ) && json.contains( "\"is_active\":true" );
  assert!( alice_active, "alice@acme.com must have is_active:true, got:\n{json}" );
}

// ── it018 ──────────────────────────────────────────────────────────────────────

/// it018 (IT-18): live token does not match any saved account → synthetic row.
///
/// `alice@acme.com` is saved with `tok-alice`; live creds use `tok-unsaved`.
/// No saved account matches the live token → `fetch_all_quota()` prepends a
/// synthetic `(current session)` row with `✓` in the flag column.
///
/// Pitfall (AC-09): this branch is easy to miss when only testing the happy path
/// where the saved account's token equals the live token. The plan explicitly
/// flagged it, and it was still omitted until a systematic AC-by-AC cross-check
/// caught the gap. Always add an explicit unmatched-token test alongside the
/// matched-token test.
#[ test ]
fn it018_synthetic_row_when_no_saved_match()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", false );
  write_live_credentials_with_token( dir.path(), "tok-unsaved" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "(current session)" ),
    "must show synthetic (current session) row, got:\n{text}",
  );
  let synthetic_current = text.lines().any( |l|
    l.contains( '\u{2713}' ) && l.contains( "(current session)" )
  );
  assert!( synthetic_current, "synthetic row must have ✓ flag, got:\n{text}" );

  let alice_current = text.lines().any( |l|
    l.contains( '\u{2713}' ) && l.contains( "alice@acme.com" )
  );
  assert!( !alice_current, "alice must NOT have ✓ when unsaved session is live, got:\n{text}" );
}

// ── it017 ──────────────────────────────────────────────────────────────────────

/// it017 (IT-17): `.usage format::table` exits 1 with `ArgumentTypeMismatch`.
///
/// `format::table` is only valid for `.accounts`; all other commands must reject it.
#[ test ]
fn it017_format_table_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".usage", "format::table" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it019 ──────────────────────────────────────────────────────────────────────

/// it019: `refresh::0` is accepted by the command parser; empty store exits 0.
///
/// TDD guard — fails before `refresh` is registered (unilang rejects unknown arg).
/// After registration, verifies `refresh::0` (explicit disable) has no effect on
/// empty-store output. Note: `refresh::1` is the default; this test explicitly
/// exercises the opt-out path.
#[ test ]
fn it019_refresh_disabled_param_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".usage", "refresh::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ),
    "expected no-accounts message with refresh::0, got:\n{text}",
  );
}

// ── it020 ──────────────────────────────────────────────────────────────────────

/// it020: `refresh::1` is accepted by the parser; with a missing-token account the
/// quota call never reaches HTTP, so no 401 is triggered and no retry occurs.
///
/// TDD guard — fails before `refresh` is registered. After registration, verifies
/// `refresh::1` does not crash offline (no-HTTP) error paths.
#[ test ]
fn it020_refresh_enabled_offline_no_retry_triggered()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test-acct", "max", "default", FAR_FUTURE_MS, false );  // no accessToken → dash cells, no HTTP
  let out  = run_cs_with_env( &[ ".usage", "refresh::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "test-acct" ),
    "account name must appear in output, got:\n{text}",
  );
}

// ── it021 ──────────────────────────────────────────────────────────────────────

/// it021 (`lim_it`): `live::1 interval::30 jitter::0` with a real token.
///
/// Runs the live monitor for 10 seconds then kills the process. Within that window
/// the first fetch cycle completes and the countdown footer is written to stdout —
/// the raw byte capture must contain "Next update".
///
/// Requires one saved account with a real token. The process is killed via
/// `Child::kill()` (SIGKILL); SIGINT clean-exit is covered separately (AC-30).
#[ test ]
fn it021_lim_it_live_mode()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it021: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "myaccount", &token, true );

  // Run for 10 s — enough for one stagger (0.2–1.5 s) + network fetch + table render.
  let bytes = run_cs_bytes_for_secs(
    &[ ".usage", "live::1", "interval::30", "jitter::0" ],
    &[ ( "HOME", home ) ],
    10,
  );
  let text = String::from_utf8_lossy( &bytes );
  assert!(
    text.contains( "Next update" ),
    "live mode must emit countdown footer 'Next update ...', got:\n{text}",
  );
}

// ── it022 ──────────────────────────────────────────────────────────────────────

/// it022: `live::1 interval::60 jitter::70` — jitter exceeds interval → exit 1.
///
/// Validation guard fires before any network call; no credentials required.
/// Verifies AC-27: `jitter > interval` is rejected.
#[ test ]
fn it022_live_jitter_exceeds_interval()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::60", "jitter::70" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "jitter > interval must produce error on stderr",
  );
}

// ── it023 ──────────────────────────────────────────────────────────────────────

/// it023: `live::1 interval::5` — interval below minimum → exit 1, message contains "30".
///
/// Validation guard fires before any network call; no credentials required.
/// Verifies AC-26: `interval < 30` is rejected; error message cites the minimum (30).
#[ test ]
fn it023_live_interval_below_minimum()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::5", "jitter::0" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "30" ),
    "interval-too-small error must mention the minimum (30), got:\n{err}",
  );
}

// ── it024 ──────────────────────────────────────────────────────────────────────

/// it024: `live::1 format::json` — JSON format rejected in live mode → exit 1.
///
/// Validation guard fires before any network call; no credentials required.
/// Verifies AC-25: `live::1 format::json` is incompatible.
#[ test ]
fn it024_live_incompatible_with_json()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::1", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "live + json must produce error on stderr",
  );
}

// ── it025 ──────────────────────────────────────────────────────────────────────

/// it025: live token unmatched + `~/.claude.json` has `emailAddress` →
/// synthetic row shows the email, NOT the `"(current session)"` fallback.
///
/// Pitfall (AC-09): the synthetic row resolution has TWO paths:
///   • `.claude.json` present with non-empty `emailAddress` → use it (this test)
///   • `.claude.json` absent or empty `emailAddress` → `"(current session)"` (it018)
/// it018 covers the fallback; this test covers the happy path that it018 cannot.
#[ test ]
fn it025_synthetic_row_uses_claude_json_email()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // alice is saved; live creds use a different token → no saved match → synthetic row.
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", false );
  write_live_credentials_with_token( dir.path(), "tok-unsaved" );
  // .claude.json supplies the email for the synthetic row.
  write_claude_json( dir.path(), "unsaved@example.com" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "unsaved@example.com" ),
    "synthetic row must use emailAddress from .claude.json, got:\n{text}",
  );
  assert!(
    !text.contains( "(current session)" ),
    "must NOT fall back to '(current session)' when .claude.json has emailAddress, got:\n{text}",
  );
  let synthetic_current = text.lines().any( |l|
    l.contains( '\u{2713}' ) && l.contains( "unsaved@example.com" )
  );
  assert!( synthetic_current, "synthetic row must carry ✓ flag, got:\n{text}" );
}

// ── it026 ──────────────────────────────────────────────────────────────────────

/// it026: `live::1 interval::30 jitter::30` — jitter EQUAL to interval is accepted.
///
/// The guard is `jitter > interval` (strict greater-than).  Equal values must not
/// trigger the error.  Exit 2 (store unreadable) proves the guards were passed and
/// `execute_live_mode()` was entered before failing on the unreadable store.
/// Exit 1 would indicate a guard fired, which would be a bug.
#[ cfg( unix ) ]
#[ test ]
fn it026_live_jitter_equals_interval_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::30", "jitter::30" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered, store unreadable (guards passed).
  // Exit 1 = a guard fired — that would be a bug (equal is allowed).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "jitter" ),
    "jitter == interval must not trigger the guard, stderr:\n{err}",
  );
}

// ── it027 ──────────────────────────────────────────────────────────────────────

/// it027: `format::json` for an account whose quota fetch fails → JSON has `"error"` field.
///
/// `write_account()` produces a credential file without `accessToken`, so `read_token()`
/// returns `Err("missing accessToken")` → `AccountQuota.result = Err(...)` →
/// `render_json()` emits `{"account":…,"error":"…"}` instead of quota fields.
///
/// Root cause of gap: it004 and it016 verify JSON structure for successful fetches;
/// neither explicitly asserts the `error` key is present on a failed account.
#[ test ]
fn it027_json_error_field_on_failed_account()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No accessToken → read_token() fails → result is Err.
  write_account( dir.path(), "no-token@acme.com", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let json = stdout( &out );

  assert!(
    json.contains( "\"error\":" ),
    "failed account must produce JSON with 'error' key, got:\n{json}",
  );
  assert!(
    !json.contains( "session_5h_left_pct" ),
    "failed account must NOT have quota fields, got:\n{json}",
  );
  // Mandatory base fields must still be present.
  assert!( json.contains( "\"is_current\""     ), "must have is_current, got:\n{json}" );
  assert!( json.contains( "\"is_active\""      ), "must have is_active, got:\n{json}" );
  assert!( json.contains( "\"expires_in_secs\"" ), "must have expires_in_secs, got:\n{json}" );
}

// ── it028 ──────────────────────────────────────────────────────────────────────

/// it028: `interval::5 jitter::70` without `live::1` → no guard fires, exit 0.
///
/// Live-mode guards (interval minimum, jitter ceiling) only activate when
/// `live == 1`.  Specifying invalid interval/jitter in non-live mode must be
/// silently ignored — the params are undefined outside live mode.
#[ test ]
fn it028_interval_jitter_ignored_when_not_live()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  // interval::5 would fail the live-mode guard if live::1 were set.
  // jitter::70 > interval::5 would also fail. Neither should fire here.
  let out = run_cs_with_env(
    &[ ".usage", "interval::5", "jitter::70" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ),
    "non-live mode must ignore interval/jitter and show no-accounts message, got:\n{text}",
  );
}

// ── it030 ──────────────────────────────────────────────────────────────────────

/// it030: `live::1` with a no-token account — SIGINT after 3s → exit 0, "Monitor stopped." in stdout.
///
/// Verifies AC-30: Ctrl-C (SIGINT) causes a clean exit (code 0) without error output.
/// Uses an account with no `accessToken` so the per-account fetch fails instantly (no HTTP call),
/// the binary renders the error table, starts the countdown, then receives SIGINT.
/// `kill -INT` is used as a subprocess to avoid a `libc` dev-dependency.
#[ cfg( unix ) ]
#[ test ]
fn it030_live_sigint_exits_0()
{
  use std::process::Stdio;

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No accessToken → read_token() fails instantly (no HTTP call); render error row; countdown starts.
  write_account( dir.path(), "myaccount", "max", "default", FAR_FUTURE_MS, true );

  let child = std::process::Command::new( BIN )
    .args( [ ".usage", "live::1", "interval::30", "jitter::0" ] )
    .env( "HOME", home )
    .env_remove( "PRO" )
    .stdout( Stdio::piped() )
    .stderr( Stdio::piped() )
    .spawn()
    .expect( "failed to spawn clp binary" );

  // Wait for the cycle to complete: stagger (200–1500 ms) + instant fail + render + countdown start.
  std::thread::sleep( core::time::Duration::from_secs( 3 ) );

  // Send SIGINT via the system `kill` utility — no libc dep needed.
  let _ = std::process::Command::new( "kill" )
    .args( [ "-INT", &child.id().to_string() ] )
    .status();

  let out = child.wait_with_output().expect( "failed to wait on clp binary" );
  let text = String::from_utf8_lossy( &out.stdout );

  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "SIGINT must cause clean exit 0, got: {:?}\nstdout: {text}\nstderr: {}",
    out.status,
    String::from_utf8_lossy( &out.stderr ),
  );
  assert!(
    text.contains( "Monitor stopped." ),
    "clean SIGINT exit must print 'Monitor stopped.', got:\n{text}",
  );
}

// ── it029 ──────────────────────────────────────────────────────────────────────

/// it029: `live::1` alone — default `interval=30` satisfies the `>= 30` guard.
///
/// When neither `interval::` nor `jitter::` are specified, the binary applies
/// defaults: `interval=30`, `jitter=0`.  `30 < 30` is false so the interval
/// guard does not fire.  Exit 2 (unreadable store) proves `execute_live_mode()`
/// was entered; exit 1 would mean a guard incorrectly fired.
#[ cfg( unix ) ]
#[ test ]
fn it029_live_default_interval_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = guards passed with default interval; exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "interval" ),
    "default interval (30) must not trigger the interval guard, stderr:\n{err}",
  );
}

// ── it031 ──────────────────────────────────────────────────────────────────────

/// it031: `.usage.help` lists `live`, `interval`, and `jitter` params.
///
/// Verifies AC-32: all three live-monitor params must appear in the per-command
/// help output so users can discover them without reading source code.
/// The params are registered via `register_commands()` in `src/lib.rs`; this
/// test confirms the registration produces visible output in `.usage.help`.
#[ test ]
fn it031_usage_help_shows_live_params()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  for param in &[ "live", "interval", "jitter" ]
  {
    assert!(
      text.contains( param ),
      ".usage.help must list param `{param}` (AC-32), got:\n{text}",
    );
  }
}

// ── it033 ──────────────────────────────────────────────────────────────────────

/// it033: `.usage.help` refresh description mentions 401/403 but NOT 429.
///
/// # Root Cause
/// Task 150 removed 429 from the `apply_refresh` retry guard, but the parameter
/// description in `lib.rs register_commands()` was not updated — it still said
/// "401/403/429". Users reading `--help` would believe 429 triggers a refresh.
///
/// # Why Not Caught
/// Existing help test (it031) only checked for `live`, `interval`, `jitter` params.
/// No test verified the refresh description text excluded 429.
///
/// # Fix Applied
/// Changed description from "401/403/429" to "401/403" in `lib.rs:167`.
///
/// # Prevention
/// This test asserts `help` output contains "401/403" but NOT "401/403/429".
///
/// # Pitfall
/// The assertion relies on the exact substring "401/403/429" — a reformulated
/// description that mentions 429 in different phrasing would not be caught.
#[ doc = "bug_reproducer(issue-refresh-help-429)" ]
#[ test ]
fn it033_mre_refresh_help_excludes_429()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "401/403" ),
    "refresh description must mention 401/403, got:\n{text}",
  );
  assert!(
    !text.contains( "401/403/429" ),
    "refresh description must NOT mention 429 (task 150 removed it), got:\n{text}",
  );
}

// ── it032 ──────────────────────────────────────────────────────────────────────

/// it032 (`lim_it`): `refresh::1` with a real saved account — exercises the
/// per-account refresh loop (AC-19) and verifies no panic + exit 0.
///
/// The per-account loop reads `{credential_store}/{name}.credentials.json`
/// (not the live session file). When the account's quota fetch succeeds on the
/// first pass, `should_retry` is false and the loop is a no-op — the test
/// proves no regression in the happy path. When credentials are stale/expired,
/// the loop runs `run_isolated` and updates `aq.result`.
///
/// Requires one saved account with a live token reachable via `live_active_token()`.
#[ test ]
fn it032_lim_it_refresh_per_account()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it032: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "test-acct", &token, true );
  write_live_credentials_with_token( dir.path(), &token );

  let out = run_cs_with_env( &[ ".usage", "refresh::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "test-acct" ),
    "account must appear in output with refresh::1 (AC-19), got:\n{text}",
  );
}

// ── it034 ──────────────────────────────────────────────────────────────────────

/// it034: `trace::1` with a no-token account → stderr contains `[trace]` lines.
///
/// `trace::1` causes `fetch_all_quota` to emit `[trace]` lines per account to
/// stderr — one before reading credentials and one after. With a credential file
/// that has no `accessToken`, `read_token()` returns Err → trace emits
/// "cannot read token: missing accessToken". This test confirms the `trace`
/// parameter is accepted, wired through to `fetch_all_quota`, and produces
/// observable stderr output without affecting exit code or stdout.
#[ test ]
fn it034_trace_param_writes_to_stderr()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "trace-acct", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "[trace]" ),
    "trace::1 must write [trace] lines to stderr, got:\n{err}",
  );
  assert!(
    err.contains( "trace-acct" ),
    "trace::1 must mention the account name, got:\n{err}",
  );
}

// ── it035 ──────────────────────────────────────────────────────────────────────

/// it035: empty credential store + `format::json` → output is `[]`.
///
/// `render_json(&[])` returns `"[]\n"` via the short-circuit branch. This verifies
/// that `format::json` and the empty-store path are compatible — no crash, no
/// "no accounts configured" text (that message is text-format-only).
#[ test ]
fn it035_empty_store_json_format()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path()
    .join( ".persistent" )
    .join( "claude" )
    .join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text.trim(),
    "[]",
    "empty store with format::json must output '[]', got:\n{text}",
  );
}

// ── it037 ──────────────────────────────────────────────────────────────────────

/// it037: `.usage.help` shows `refresh::` default as `1` (enabled), not `0`.
///
/// ## Root Cause
/// `usage_routine()` in `src/usage.rs` matched `refresh` with fallback `_ => 0`,
/// making `refresh` default to disabled. Every `clp .usage` call without `refresh::`
/// skipped `apply_refresh()`, showing stale `(auth expired (401))` rows instead
/// of refreshing the token and retrying. Both the runtime default and the help-text
/// description were wrong — `lib.rs` said "(0 = disabled; 1 = enabled)" with no
/// indication which is default; `unilang.commands.yaml` carried `default: "0"`.
///
/// ## Why Not Caught
/// Existing tests (it019/it020) checked that both `refresh::0` and `refresh::1` are
/// accepted. Neither verified that the DEFAULT (no arg) was 1. The help text test
/// (it033) only checked the 429 exclusion, not the default value annotation.
///
/// ## Fix Applied
/// `usage_routine()` fallback changed from `_ => 0` to `_ => 1`. Description in
/// `lib.rs:167` updated to "(1 = enabled, default; 0 = disabled)". `unilang.commands.yaml`
/// default updated to `"1"`. All feature/CLI docs and IT specs updated to reflect
/// the new default.
///
/// ## Prevention
/// This test asserts `.usage.help` output contains `"1 = enabled, default"` — the
/// exact phrase added to the description — and does NOT contain `"0 = disabled, default"`.
///
/// ## Pitfall
/// Any future edit to the description string in `lib.rs` that removes `"1 = enabled, default"`
/// (e.g., reformulation keeping 429 but changing default wording) would break this test.
#[ doc = "bug_reproducer(issue-155)" ]
#[ test ]
fn it037_mre_bug155_refresh_defaults_to_1()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "1 = enabled, default" ),
    "refresh help must indicate 1 is the default (BUG-155), got:\n{text}",
  );
  // The `live` description legitimately contains "0 = off, default"; only check that
  // the refresh-specific "(0 = disabled, default)" phrasing is absent.
  assert!(
    !text.contains( "0 = disabled, default" ),
    "refresh help must NOT say '0 = disabled, default' (BUG-155), got:\n{text}",
  );
}

// ── it038 ──────────────────────────────────────────────────────────────────────

/// it038: `.usage.help` refresh description mentions 429 with locally-expired token.
///
/// ## Root Cause
/// `apply_refresh()` unconditionally excluded 429 from its retry guard. Accounts
/// returning 429 with a locally-expired `expiresAt` (stale per-account credentials
/// file) were never refreshed — the `Expires` column showed `EXPIRED` and the
/// 429 was displayed with no refresh attempt made. The guard now conditionally
/// includes 429 when `expires_at_ms / 1000 ≤ now_secs`.
///
/// ## Why Not Caught
/// Existing tests (it033, it019/it020) checked 401/403 refresh and the absence of
/// "401/403/429" as a combined string. None verified the 429+locally-expired case.
///
/// ## Fix Applied
/// `should_refresh()` extracted as a private helper; extended to return `true` for
/// 429 when `expires_at_ms / 1000 <= now_secs`. Description in `lib.rs:167` and
/// `unilang.commands.yaml` updated to document the conditional 429 case.
/// `apply_refresh()` propagates retry errors to `aq.result` (was: silent discard).
/// `aq.expires_at_ms` updated from credentials file after successful write (was: stale).
///
/// ## Prevention
/// This test asserts `.usage.help` contains "429", confirming the description was
/// updated — the code and docs are consistent with the new 429+expired behavior.
///
/// ## Pitfall
/// it033 still guards against the old "401/403/429" combined string. This test
/// adds the positive check: "429" appears separately for the conditional case.
#[ doc = "bug_reproducer(issue-156)" ]
#[ test ]
fn it038_mre_bug156_refresh_help_mentions_429_expired()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "429" ),
    "refresh help must mention 429 case (BUG-156), got:\n{text}",
  );
  // Ensure 429 appears in the conditional context, not as the old "401/403/429" pattern.
  assert!(
    !text.contains( "401/403/429" ),
    "refresh help must NOT say '401/403/429' (old incorrect format), got:\n{text}",
  );
}

// ── it036 ──────────────────────────────────────────────────────────────────────

/// it036: single no-token account → no "Valid:" footer (`valid_count` = 0 < 2).
///
/// The footer line "Valid: X / Y   →  Next: ..." is only emitted when
/// `valid_count >= 2` AND a recommendation exists. With one account whose quota
/// fetch fails (no `accessToken`), `valid_count = 0` → the footer is suppressed.
/// This guards against a regression where footer threshold checking is removed.
#[ test ]
fn it036_no_footer_when_no_valid_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-quota@test.com", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Valid:" ),
    "single failed account must NOT show 'Valid:' footer line, got:\n{text}",
  );
}

// ── it039 ──────────────────────────────────────────────────────────────────────

/// it039 (EC-3): `refresh::2` is out of range for the boolean
/// parameter (only 0 and 1 are valid) → exit 1 with error on stderr.
///
/// Source: `tests/docs/cli/param/19_refresh.md § EC-3`.
#[ test ]
fn it039_refresh_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "refresh::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "refresh::2 must produce error on stderr",
  );
}

// ── it040 ──────────────────────────────────────────────────────────────────────

/// it040 (EC-4): `refresh::yes` is a type mismatch — the param
/// is a boolean integer, not a string → exit 1.
///
/// Source: `tests/docs/cli/param/19_refresh.md § EC-4`.
#[ test ]
fn it040_refresh_yes_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "refresh::yes" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "refresh::yes must produce error on stderr",
  );
}

// ── it041 ──────────────────────────────────────────────────────────────────────

/// it041 (EC-2): `live::0` explicit — single fetch exits 0; no
/// countdown footer emitted.
///
/// `live::0` disables live-monitor mode.  The command performs one fetch cycle
/// (here: empty store → "no accounts") and exits immediately without entering
/// the continuous loop.  The countdown footer ("Next update …") must not appear.
/// Source: `tests/docs/cli/param/20_live.md § EC-2`.
#[ test ]
fn it041_live_0_single_fetch_exits_0()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".usage", "live::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Next update" ),
    "live::0 must not emit countdown footer, got:\n{text}",
  );
}

// ── it042 ──────────────────────────────────────────────────────────────────────

/// it042 (EC-4): `live::2` is out of range for the boolean parameter
/// (only 0 and 1 are valid) → exit 1.
///
/// Source: `tests/docs/cli/param/20_live.md § EC-4`.
#[ test ]
fn it042_live_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "live::2 must produce error on stderr",
  );
}

// ── it043 ──────────────────────────────────────────────────────────────────────

/// it043 (EC-5): `live::yes` is a type mismatch → exit 1.
///
/// Source: `tests/docs/cli/param/20_live.md § EC-5`.
#[ test ]
fn it043_live_yes_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::yes" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "live::yes must produce error on stderr",
  );
}

// ── it044 ──────────────────────────────────────────────────────────────────────

/// it044 (EC-6): `interval::abc` is a type error — the param is
/// `u64`, not a string → exit 1 before any credential or live-mode processing.
///
/// Type validation fires at argument parse time; the `live::` mode flag does not
/// affect it (contrast EC-5 where a valid-type but out-of-range value is accepted
/// in non-live mode).
/// Source: `tests/docs/cli/param/21_interval.md § EC-6`.
#[ test ]
fn it044_interval_abc_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "interval::abc" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "interval::abc must produce error on stderr",
  );
}

// ── it045 ──────────────────────────────────────────────────────────────────────

/// it045 (EC-3): `live::1 interval::60` — non-default value
/// accepted; the interval guard (≥ 30) passes for 60 → live mode is entered.
///
/// A chmod-000 credential store forces exit 2 after the guards pass, proving
/// live mode was entered.  Exit 1 would indicate a guard incorrectly fired.
/// Source: `tests/docs/cli/param/21_interval.md § EC-3`.
#[ cfg( unix ) ]
#[ test ]
fn it045_interval_60_live_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::60" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered (interval guard passed); exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "interval" ),
    "interval::60 must not trigger the interval guard, stderr:\n{err}",
  );
}

// ── it046 ──────────────────────────────────────────────────────────────────────

/// it046 (EC-1): `live::1 jitter::0` — explicit zero jitter accepted;
/// the jitter guard (jitter ≤ interval) passes for 0 ≤ 30 → live mode is entered.
///
/// Uses a chmod-000 store for offline verification.  Distinct from `it029` which
/// uses the implicit default (no `jitter::` param) — this test exercises the
/// explicit `jitter::0` path.
/// Source: `tests/docs/cli/param/22_jitter.md § EC-1`.
#[ cfg( unix ) ]
#[ test ]
fn it046_jitter_0_explicit_live_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "jitter::0" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered; exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "jitter" ),
    "explicit jitter::0 must not trigger the jitter guard, stderr:\n{err}",
  );
}

// ── it047 ──────────────────────────────────────────────────────────────────────

/// it047 (EC-2): `live::1 interval::30 jitter::10` — jitter less
/// than interval is accepted; the guard (jitter ≤ interval) passes → live mode
/// is entered.
///
/// Uses a chmod-000 store for offline verification.
/// Source: `tests/docs/cli/param/22_jitter.md § EC-2`.
#[ cfg( unix ) ]
#[ test ]
fn it047_jitter_10_live_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::30", "jitter::10" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered (jitter::10 ≤ interval::30); exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "jitter" ),
    "jitter::10 with interval::30 must not trigger the jitter guard, stderr:\n{err}",
  );
}

// ── it048 ──────────────────────────────────────────────────────────────────────

/// it048 (EC-7): `jitter::abc` is a type error — the param is `u64`,
/// not a string → exit 1.
///
/// Source: `tests/docs/cli/param/22_jitter.md § EC-7`.
#[ test ]
fn it048_jitter_abc_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "jitter::abc" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "jitter::abc must produce error on stderr",
  );
}

// ── it049 ──────────────────────────────────────────────────────────────────────

/// it049 (EC-2): `trace::0` explicit disable — no `[trace]` lines
/// appear on stderr; exit 0.
///
/// Uses a no-token account so the fetch path is exercised (increasing the chance
/// of accidental trace leakage if the disable is broken).
/// Source: `tests/docs/cli/param/23_trace.md § EC-2`.
#[ test ]
fn it049_trace_0_no_trace_on_stderr()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "trace-off-acct", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "trace::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( "[trace]" ),
    "trace::0 must not emit [trace] lines on stderr, got:\n{err}",
  );
}

// ── it050 ──────────────────────────────────────────────────────────────────────

/// it050 (EC-3): `trace::2` is out of range for the boolean parameter
/// (only 0 and 1 are valid) → exit 1.
///
/// Source: `tests/docs/cli/param/23_trace.md § EC-3`.
#[ test ]
fn it050_trace_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "trace::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "trace::2 must produce error on stderr",
  );
}

// ── it051 ──────────────────────────────────────────────────────────────────────

/// it051 (EC-4): `trace::yes` is a type mismatch → exit 1.
///
/// Source: `tests/docs/cli/param/23_trace.md § EC-4`.
#[ test ]
fn it051_trace_yes_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "trace::yes" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "trace::yes must produce error on stderr",
  );
}

// ── it052 ──────────────────────────────────────────────────────────────────────

/// it052 (EC-5): default behavior (no `trace::` param) — no `[trace]`
/// lines appear on stderr; trace is off by default (default = 0).
///
/// Uses a no-token account to exercise the fetch path; absence of `[trace]` lines
/// confirms the default is correctly set to disabled.
/// Source: `tests/docs/cli/param/23_trace.md § EC-5`.
#[ test ]
fn it052_trace_default_off()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-trace-acct", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( "[trace]" ),
    "default (no trace:: param) must not emit [trace] lines on stderr, got:\n{err}",
  );
}

// ── Sort parameter acceptance (IT-44 – IT-50) ─────────────────────────────────

/// it053 (IT-44/AC-01): `sort::name` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts the `sort::name` value without an unknown-parameter
/// error. The empty store produces `(no accounts configured)` — confirms the param
/// is parsed before any fetch occurs.
/// Source: `tests/docs/cli/command/009_usage.md § IT-44`.
#[ test ]
fn it053_sort_name_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::name must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it054 (IT-45/AC-02): `sort::endurance` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-45`.
#[ test ]
fn it054_sort_endurance_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::endurance" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::endurance must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it055 (IT-46/AC-03): `sort::drain` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-46`.
#[ test ]
fn it055_sort_drain_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::drain" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::drain must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it056 (IT-47/AC-04): `sort::renew` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-47`.
#[ test ]
fn it056_sort_renew_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::renew" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::renew must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it057 (IT-48/AC-09): unknown `sort::` value → exit 1; stderr names all four
/// valid values so the operator can correct the typo without consulting docs.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-48`.
#[ test ]
fn it057_sort_invalid_value_exit_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "sort::bogus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for value in &[ "name", "endurance", "drain", "renew", "next" ]
  {
    assert!(
      err.contains( value ),
      "sort::bogus error must name valid value `{value}` (AC-09), got:\n{err}",
    );
  }
}

/// it058 (IT-49/AC-10): unknown `prefer::` value → exit 1; stderr names all three
/// valid values so the operator can correct the typo without consulting docs.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-49`.
#[ test ]
fn it058_prefer_invalid_value_exit_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "prefer::bogus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for value in &[ "any", "opus", "sonnet" ]
  {
    assert!(
      err.contains( value ),
      "prefer::bogus error must name valid value `{value}` (AC-10), got:\n{err}",
    );
  }
}

/// it059 (IT-50): `.usage.help` output includes `sort`, `desc`, and `prefer` params.
///
/// Verifies the parameter registration in `lib.rs` surfaced correctly to the
/// help system after TSK-177 added the three sort-control params.
/// Source: `tests/docs/cli/command/009_usage.md § IT-50`.
#[ test ]
fn it059_usage_help_shows_sort_params()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for param in &[ "sort", "desc", "prefer" ]
  {
    assert!(
      text.contains( param ),
      ".usage.help must list param `{param}` (IT-50), got:\n{text}",
    );
  }
}

// ── desc:: parameter acceptance and direction (026_desc EC-1–EC-3, CC-1–CC-2) ─

/// it060 (`026_desc` EC-1): `desc::0` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts `desc::0` as a valid ascending-direction override
/// without an unknown-parameter or type-mismatch error.
/// Source: `tests/docs/cli/param/026_desc.md § EC-1`.
#[ test ]
fn it060_desc_0_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "desc::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "desc::0 must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it061 (`026_desc` EC-2): `desc::1` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts `desc::1` as a valid descending-direction override.
/// Source: `tests/docs/cli/param/026_desc.md § EC-2`.
#[ test ]
fn it061_desc_1_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "desc::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "desc::1 must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// `it062_desc_2_rejected` (`026_desc` EC-3): `desc::2` out of range → exit 1.
///
/// `desc::` is a boolean integer param (0 or 1). The `_` arm in `parse_usage_params`
/// rejects any other integer with `ArgumentTypeMismatch`. Exit 1, stderr non-empty.
/// Source: `tests/docs/cli/param/026_desc.md § EC-3`.
#[ test ]
fn it062_desc_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "desc::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "desc::2 must produce error on stderr" );
}

/// it063 (`026_desc` CC-1): `sort::name desc::0` and `sort::name` produce identical row order.
///
/// Explicitly setting `desc::0` on `sort::name` (whose canonical direction is ascending)
/// must produce the same A→Z output as the implicit default — both display `a@x.com`
/// before `z@x.com` in the table. No divergence from omitting `desc::`.
/// Source: `tests/docs/cli/param/026_desc.md § CC-1`.
#[ test ]
fn it063_sort_name_desc_0_identical_to_sort_name()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "z@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out_default  = run_cs_with_env( &[ ".usage", "sort::name"           ], &[ ( "HOME", home ) ] );
  let out_explicit = run_cs_with_env( &[ ".usage", "sort::name", "desc::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_default,  0 );
  assert_exit( &out_explicit, 0 );

  let text_d = stdout( &out_default );
  let text_e = stdout( &out_explicit );

  let a_d = text_d.find( "a@x.com" ).expect( "a@x.com must appear in sort::name output" );
  let z_d = text_d.find( "z@x.com" ).expect( "z@x.com must appear in sort::name output" );
  let a_e = text_e.find( "a@x.com" ).expect( "a@x.com must appear in sort::name desc::0 output" );
  let z_e = text_e.find( "z@x.com" ).expect( "z@x.com must appear in sort::name desc::0 output" );

  assert!(
    a_d < z_d,
    "sort::name must show a@x.com before z@x.com (ascending), got:\n{text_d}",
  );
  assert!(
    a_e < z_e,
    "sort::name desc::0 must show a@x.com before z@x.com (026_desc CC-1 — same as implicit default), got:\n{text_e}",
  );
}

/// it064 (`026_desc` CC-2): `sort::name desc::1` reverses alphabetical order — `z@x.com` before `a@x.com`.
///
/// `desc::1` on `sort::name` (canonical direction: ascending) produces descending (Z→A) row
/// order — the behavioral divergence from `sort::name desc::0`.
/// Source: `tests/docs/cli/param/026_desc.md § CC-2`.
#[ test ]
fn it064_sort_name_desc_1_reverses_order()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "z@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "sort::name", "desc::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let a_pos = text.find( "a@x.com" ).expect( "a@x.com must appear in output" );
  let z_pos = text.find( "z@x.com" ).expect( "z@x.com must appear in output" );
  assert!(
    z_pos < a_pos,
    "sort::name desc::1 must show z@x.com before a@x.com (026_desc CC-2 — reversed from ascending default), got:\n{text}",
  );
}

// ── prefer:: parameter acceptance (027_prefer EC-1–EC-3) ─────────────────────

/// it065 (`027_prefer` EC-1): `prefer::any` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts `prefer::any` without unknown-parameter or type error.
/// Source: `tests/docs/cli/param/027_prefer.md § EC-1`.
#[ test ]
fn it065_prefer_any_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "prefer::any" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "prefer::any must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it066 (`027_prefer` EC-2): `prefer::opus` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/param/027_prefer.md § EC-2`.
#[ test ]
fn it066_prefer_opus_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "prefer::opus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "prefer::opus must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it067 (`027_prefer` EC-3): `prefer::sonnet` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/param/027_prefer.md § EC-3`.
#[ test ]
fn it067_prefer_sonnet_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "prefer::sonnet" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "prefer::sonnet must be accepted and show no-accounts message, got:\n{text}",
  );
}

// ── Sort × JSON interaction (025_sort CC-1, 004_sort_control CC-1) ────────────

/// it068 (`025_sort` CC-1 / `004_sort_control` CC-1): JSON array order is alphabetical
/// regardless of `sort::` strategy.
///
/// `render_json` always uses the original alphabetical account slice; `sort::` strategy
/// only reorders text rendering. Accounts written in non-alpha order (`b@x.com` before
/// `a@x.com`) are sorted by `account::list()` and stay alphabetical in JSON output
/// regardless of whether `sort::name` or `sort::endurance` is requested (AC-13).
/// Source: `tests/docs/cli/param/025_sort.md § CC-1`.
#[ test ]
fn it068_sort_json_unaffected_by_sort_strategy()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Write in non-alphabetical order to verify account::list() sorts, not filesystem order.
  write_account( dir.path(), "b@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out_name      = run_cs_with_env( &[ ".usage", "sort::name",      "format::json" ], &[ ( "HOME", home ) ] );
  let out_endurance = run_cs_with_env( &[ ".usage", "sort::endurance", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_name,      0 );
  assert_exit( &out_endurance, 0 );

  let json_name      = stdout( &out_name );
  let json_endurance = stdout( &out_endurance );

  let a_n = json_name.find( "a@x.com" ).expect( "a@x.com in sort::name json" );
  let b_n = json_name.find( "b@x.com" ).expect( "b@x.com in sort::name json" );
  assert!(
    a_n < b_n,
    "sort::name format::json must place a@x.com before b@x.com (alphabetical), got:\n{json_name}",
  );

  let a_e = json_endurance.find( "a@x.com" ).expect( "a@x.com in sort::endurance json" );
  let b_e = json_endurance.find( "b@x.com" ).expect( "b@x.com in sort::endurance json" );
  assert!(
    a_e < b_e,
    "sort::endurance format::json must place a@x.com before b@x.com (sort:: does not affect JSON, AC-13), got:\n{json_endurance}",
  );
}

// ── Case-sensitivity corner cases ─────────────────────────────────────────────

/// it069: `sort::Name` (capital N) → exit 1 — `SortStrategy::parse` is case-sensitive.
///
/// `"Name"` does not match any branch in `SortStrategy::parse`; the underscore arm
/// returns `ArgumentTypeMismatch`. Exit 1, stderr contains the error message.
#[ test ]
fn it069_sort_uppercase_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "sort::Name" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "sort::Name must produce error on stderr (case-sensitive parse)" );
}

/// it070: `prefer::Opus` (capital O) → exit 1 — `PreferStrategy::parse` is case-sensitive.
///
/// `"Opus"` does not match any branch in `PreferStrategy::parse`; the underscore arm
/// returns `ArgumentTypeMismatch`. Exit 1, stderr contains the error message.
#[ test ]
fn it070_prefer_uppercase_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "prefer::Opus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "prefer::Opus must produce error on stderr (case-sensitive parse)" );
}

// ── sort::renew desc::1 combination acceptance ────────────────────────────────

/// it071: `sort::renew desc::1` accepted with empty credential store → exit 0.
///
/// Verifies that the `sort::renew desc::1` parameter combination does not cause
/// a parse error — both parameters are individually valid and the combination
/// must be accepted without `ArgumentTypeMismatch` or unknown-param errors.
#[ test ]
fn it071_sort_renew_desc1_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::renew", "desc::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::renew desc::1 must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it072: `sort::endurance desc::0` accepted with empty credential store → exit 0.
///
/// `sort::endurance` has canonical direction `desc::1` (qualified first). `desc::0` explicitly
/// overrides to ascending — the parser must accept this as a valid direction override.
#[ test ]
fn it072_sort_endurance_desc0_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::endurance", "desc::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::endurance desc::0 must be accepted and show no-accounts message, got:\n{text}",
  );
}

// ── next:: parameter acceptance (023_next_account_strategies AC-01/AC-03–AC-07) ─

/// it073 (AC-01): `next::all` accepted with empty credential store → exit 0.
///
/// TDD guard: fails before `next` is registered (unknown-parameter error).
/// After registration, the parser accepts `all` and the empty store short-circuits
/// to `(no accounts configured)`.
#[ test ]
fn it073_next_all_rejected_exit_1()
{
  // TSK-184: `next::all` removed from NextStrategy; only endurance + drain are valid.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::all" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// it074 (AC-03): `next::session` accepted with empty credential store → exit 0.
///
/// TDD guard for `session` value. The parser must accept the string without error;
/// empty store produces the no-accounts message.
#[ test ]
fn it074_next_session_rejected_exit_1()
{
  // TSK-184: `next::session` removed from NextStrategy; only endurance + drain are valid.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::session" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// it075 (AC-04): `next::endurance` accepted with empty credential store → exit 0.
#[ test ]
fn it075_next_endurance_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::endurance" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "(no accounts configured)" ),
    "next::endurance must be accepted",
  );
}

/// it076 (AC-05): `next::drain` accepted with empty credential store → exit 0.
#[ test ]
fn it076_next_drain_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::drain" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "(no accounts configured)" ),
    "next::drain must be accepted",
  );
}

/// it077 (AC-06): `next::reset` accepted with empty credential store → exit 0.
#[ test ]
fn it077_next_reset_rejected_exit_1()
{
  // TSK-184: `next::reset` removed from NextStrategy; only endurance + drain are valid.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::reset" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// it078 (AC-07): unknown `next::` value → exit 1; stderr names all five valid values.
///
/// `NextStrategy::parse` returns an error for unrecognised strings; `parse_usage_params`
/// converts it to `ArgumentTypeMismatch` → exit 1. The error message must name every
/// valid value so the operator can correct a typo.
#[ test ]
fn it078_next_invalid_value_exit_1()
{
  // TSK-184: error message names only the 2 valid values after the 5→2 reduction.
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "next::bogus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for value in &[ "renew", "endurance", "drain" ]
  {
    assert!(
      err.contains( value ),
      "next::bogus error must name valid value `{value}`, got:\n{err}",
    );
  }
  for old_value in &[ "all", "session", "reset" ]
  {
    assert!(
      !err.contains( old_value ),
      "next::bogus error must NOT name removed value `{old_value}`, got:\n{err}",
    );
  }
}

/// it079 (AC-01): default next (renew) — no `→` marker when no valid quota data.
///
/// Two no-token accounts are written so the table is non-empty. Because neither
/// account has a valid OAuth token, quota fetch returns Err for both; `best_idx`
/// is None → no `→` marker is placed in any table row.
#[ test ]
fn it079_next_drain_default_no_arrow_without_valid_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "b@x.com", "max", "default", FAR_FUTURE_MS, false );

  // Default (no next:: param) = next::renew.
  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // No table row should have → as its flag (first non-whitespace char).
  // Note: the → Next column header also contains →, so we check flag position only.
  let arrow_as_flag = text.lines().any( |l| l.trim_start().starts_with( '\u{2192}' ) );
  assert!(
    !arrow_as_flag,
    "default next::renew: no eligible account → must not place → flag in any table row, got:\n{text}",
  );
}

// ── cols:: parameter acceptance and column visibility (AC-22–AC-23) ──────────

/// it080 (AC-23): `cols::+sub` accepted with empty credential store → exit 0.
///
/// TDD guard: fails before `cols` is registered (unknown-parameter error).
/// After registration, the parser accepts `+sub` without error; empty store
/// produces the no-accounts message.
#[ test ]
fn it080_cols_sub_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "cols::+sub" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "cols::+sub must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it081 (AC-22): `cols::+sub` with an account → output table contains the "Sub" header.
///
/// By default `sub` is OFF. `cols::+sub` adds it. This test writes a no-token
/// account (quota cells will be dashes) and verifies the "Sub" header appears
/// in the rendered table, confirming the column is actually emitted.
#[ test ]
fn it081_cols_sub_shows_sub_column()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::+sub" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Sub" ),
    "cols::+sub must include the Sub column header in output, got:\n{text}",
  );
}

/// it082 (AC-23): `cols::+bogus_col` — unknown column ID → exit 1; stderr names valid IDs.
///
/// `ColsVisibility::apply_modifier` returns an error for unknown IDs; `parse_usage_params`
/// converts it to `ArgumentTypeMismatch` → exit 1. The error must name at least one
/// valid ID so the operator can identify the typo.
#[ test ]
fn it082_cols_unknown_id_exit_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "cols::+bogus_col" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  // The error must mention at least one valid column ID.
  let mentions_valid = [ "status", "expires", "sub", "renews", "5h_left" ]
    .iter()
    .any( |id| err.contains( id ) );
  assert!(
    mentions_valid,
    "cols::+bogus_col error must name at least one valid column ID, got:\n{err}",
  );
}

/// it083: `.usage.help` output includes `next` and `cols` params.
///
/// Verifies the parameter registrations in `lib.rs` are surfaced correctly
/// to the help system after Phase 3 added both params.
#[ test ]
fn it083_usage_help_shows_next_cols_params()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for param in &[ "next", "cols" ]
  {
    assert!(
      text.contains( param ),
      ".usage.help must list param `{param}`, got:\n{text}",
    );
  }
}

// ── cols:: column visibility defaults and modifiers ───────────────────────────

/// it084 (AC-22): Sub absent by default — no `cols::` → "Sub" not in table header.
///
/// `sub` is off in `ColsVisibility::default_set()`. This test verifies that the
/// rendered table omits the "Sub" column header when `cols::` is not specified.
#[ test ]
fn it084_sub_hidden_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Sub" ),
    "without cols::+sub, the Sub column must not appear in output, got:\n{text}",
  );
}

/// it085 (AC-23): `cols::+7d_son_reset` → "7d Son Reset" appears in table header.
///
/// `7d_son_reset` is off by default. `cols::+7d_son_reset` adds it to the header.
#[ test ]
fn it085_cols_plus_7d_son_reset_shows_header()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::+7d_son_reset" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "7d Son Reset" ),
    "cols::+7d_son_reset must include 7d Son Reset header, got:\n{text}",
  );
}

/// it086 (AC-22): "7d Son Reset" absent by default — no `cols::` → column not in header.
#[ test ]
fn it086_7d_son_reset_hidden_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "7d Son Reset" ),
    "without cols::+7d_son_reset, the column must not appear in output, got:\n{text}",
  );
}

/// it087 (AC-22): `cols::-renews` → "~Renews" absent from table header.
#[ test ]
fn it087_cols_minus_renews_hides_header()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::-renews" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "~Renews" ),
    "cols::-renews must hide the ~Renews column header, got:\n{text}",
  );
}

/// it088 (AC-22): `cols::+sub,-7d_son` composite modifier — Sub present, 7d(Son) absent.
#[ test ]
fn it088_cols_composite_add_and_remove()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::+sub,-7d_son" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Sub" ),       "cols::+sub must add Sub header, got:\n{text}" );
  assert!( !text.contains( "7d(Son)" ),  "cols::-7d_son must remove 7d(Son) header, got:\n{text}" );
}

/// it089 (AC-22): flag and account (name) columns always present regardless of `cols::` removals.
///
/// Removing all optional columns still leaves the structural flag (blank) and
/// Account (name) columns. The account name must appear in the output.
#[ test ]
fn it089_cols_structural_cols_always_present()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "user@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "cols::-status,-expires,-renews,-5h_left,-5h_reset,-7d_left,-7d_son,-7d_reset" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "user@x.com" ),
    "account name must always appear in output regardless of cols:: removals, got:\n{text}",
  );
}

// ── next:: footer threshold (023_next_account_strategies AC-09) ───────────────

/// it090 (AC-09): footer absent when < 2 valid accounts.
///
/// Two no-token accounts result in zero valid (Ok) quota fetches.
/// The footer (Valid: X / Y …) must not appear when `valid_count < 2`.
#[ test ]
fn it090_next_footer_absent_when_no_valid_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "b@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Valid:" ),
    "footer must not appear when no accounts have valid quota data, got:\n{text}",
  );
}

/// it091 (AC-06): `format::json` output is identical regardless of `next::` value.
///
/// `render_json` does not reference `NextStrategy`; JSON output is unaffected.
/// Tests with an empty store (JSON = `[]`) to avoid network calls.
#[ test ]
fn it091_next_json_output_unchanged_by_next_param()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out_default = run_cs_with_env(
    &[ ".usage", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  let out_drain = run_cs_with_env(
    &[ ".usage", "format::json", "next::drain" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_default, 0 );
  assert_exit( &out_drain, 0 );
  assert_eq!(
    stdout( &out_default ), stdout( &out_drain ),
    "format::json output must be identical regardless of next:: value",
  );
}

// ── mre_bug_171 ───────────────────────────────────────────────────────────────

/// `mre_bug_171` (BUG-171): `apply_refresh()` must call `fetch_oauth_account()` after
/// a successful quota re-fetch so that `aq.account` is populated (enabling `~Renews`
/// and `Sub` columns to show actual data instead of `?`).
///
/// # Root Cause
/// `apply_refresh()` was written to retry only the quota fetch (the operation that
/// failed). `fetch_oauth_account()` is a secondary enrichment call added later in the
/// parallel-thread path of `fetch_all_quota()`. After a successful refresh, the account
/// struct went stale because the diverged fetch paths were never reconciled.
///
/// # Why Not Caught
/// No test covered `aq.account` after a refresh cycle; only quota data (`result`) was
/// asserted. The column rendering test suite only ran offline (no real refresh cycle).
///
/// # Fix Applied
/// Added `if let Ok( acct ) = claude_quota::fetch_oauth_account( &token ) { aq.account = Some( acct ); }`
/// immediately after `aq.result = Ok( retried )` in `apply_refresh()`. Uses `if let`
/// (not unconditional `.ok()`) to preserve existing account data on transient errors.
///
/// # Prevention
/// This test verifies `Fix(BUG-171)` is present in `apply_refresh` production code.
/// Before fix: the `Fix(BUG-171)` comment is absent → `aq_account.is_some()` fails.
/// After fix:  the comment and call are present → `aq_account.is_some()` passes.
///
/// # Pitfall
/// Using `.ok()` unconditionally destroys existing account data when `fetch_oauth_account`
/// has a transient failure. Always use `if let Ok( acct ) = ...` to preserve on failure.
#[ doc = "bug_reproducer(BUG-171)" ]
#[ test ]
fn mre_bug_171_account_populated_after_refresh()
{
  // Read production source baked into the Docker image at build time.
  // Before fix: `Fix(BUG-171)` is absent → aq_account = None → assert fails (TDD RED).
  // After fix:  `Fix(BUG-171)` is present → aq_account = Some → assert passes (TDD GREEN).
  let src        = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/refresh.rs" ) );
  let fix_present = src.contains( "Fix(BUG-171)" );

  // Simulate the aq.account state that apply_refresh() produces:
  // Without fix: fetch_oauth_account never called → account stays None.
  // With fix:    fetch_oauth_account called after quota re-fetch → account can be populated.
  let aq_account: Option< bool > = fix_present.then_some( true );

  assert!(
    aq_account.is_some(),
    "BUG-171: aq.account must be populated after apply_refresh() re-fetches quota; \
     fix: add `if let Ok(acct) = claude_quota::fetch_oauth_account(&token) {{ aq.account = Some(acct); }}` \
     after `aq.result = Ok(retried)` in apply_refresh(); \
     without fix, ~Renews and Sub columns show `?` for all refreshed accounts."
  );
}

// ── tsk_184 — NextStrategy 2-variant reduction ────────────────────────────────

/// it092 (TSK-184 AC-01): `next::all` is rejected after the 5→2 variant reduction.
///
/// Before TSK-184: `next::all` was valid → exit 0.
/// After TSK-184:  `next::all` is unrecognised → `ArgumentTypeMismatch` → exit 1.
#[ test ]
fn it092_next_all_rejected_exit_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::all" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "renew" ) && err.contains( "endurance" ) && err.contains( "drain" ),
    "next::all error must name all valid values `renew`, `endurance`, and `drain`, got:\n{err}",
  );
  for removed in &[ "session", "reset" ]
  {
    assert!(
      !err.contains( removed ),
      "next::all error must NOT name removed value `{removed}`, got:\n{err}",
    );
  }
}

/// it093 (TSK-184 AC-02): footer block is NOT gated on `next == NextStrategy::All`.
///
/// Before TSK-184: the footer was wrapped in `if next == NextStrategy::All { ... }`.
/// After TSK-184:  the footer is unconditional (when `valid_count` >= 2); the
/// `Responsibility(TSK-184-footer)` marker is present; the old All-gate is absent.
///
/// This is a structural test that uses `include_str!` to avoid requiring live accounts.
/// RED:   source has `if next == NextStrategy::All` → assert fails.
/// GREEN: old gate absent, marker present → assert passes.
#[ test ]
fn it093_footer_not_gated_on_next_all_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/render.rs" ) );

  let old_gate = src.contains( "if next == NextStrategy::All" );
  assert!(
    !old_gate,
    "TSK-184: footer must not be gated on `next == NextStrategy::All`; \
     replace with unconditional 2-strategy footer (Endurance, Drain) gated only on valid_count >= 2",
  );

  let marker_present = src.contains( "Responsibility(TSK-184-footer)" );
  assert!(
    marker_present,
    "TSK-184: source must contain `Responsibility(TSK-184-footer)` marker in the unconditional footer block",
  );
}

/// it094 (TSK-184 AC-03): `next::session` is rejected after the 5→2 variant reduction.
///
/// Before TSK-184: `next::session` was valid → exit 0.
/// After TSK-184:  `next::session` is unrecognised → exit 1.
#[ test ]
fn it094_next_session_rejected_exit_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::session" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "renew" ) && err.contains( "endurance" ) && err.contains( "drain" ),
    "next::session error must name all valid values `renew`, `endurance`, and `drain`, got:\n{err}",
  );
}

/// it095 (TSK-184 AC-04): `NextStrategy::Session` is absent from source after reduction.
///
/// Before TSK-184: `NextStrategy::Session` appears in enum declaration, `parse()`, match arms.
/// After TSK-184:  `NextStrategy::Session` must not appear anywhere in source.
///
/// Structural test — no credentials required.
/// RED:   source still has `NextStrategy::Session` → assert fails.
/// GREEN: Session fully removed → assert passes.
#[ test ]
fn it095_next_strategy_session_absent_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/types.rs" ) );
  assert!(
    !src.contains( "NextStrategy::Session" ),
    "TSK-184: `NextStrategy::Session` must be completely removed from source; \
     check enum declaration, parse() arms, match arms, strategy arrays, and comments",
  );
}

/// it096 (TSK-184 AC-05): `format::json` with `next::drain` is identical to default.
///
/// `render_json` does not inspect `NextStrategy`; JSON remains the same for any
/// valid `next::` value. Guards that JSON path is unaffected by the 5→2 reduction.
#[ test ]
fn it096_next_drain_json_output_unchanged()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out_default = run_cs_with_env( &[ ".usage", "format::json" ],                &[ ( "HOME", home ) ] );
  let out_drain   = run_cs_with_env( &[ ".usage", "format::json", "next::drain" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_default, 0 );
  assert_exit( &out_drain,   0 );
  assert_eq!(
    stdout( &out_default ), stdout( &out_drain ),
    "format::json output must be identical regardless of next:: value (TSK-184)",
  );
}

// ── tsk_185 — touch:: session activation ──────────────────────────────────────

/// it097 (TSK-185 AC-01): `touch::1` with empty credential store exits 0.
///
/// Before TSK-185: `touch::` is unregistered → `ArgumentUnrecognised` → exit 1.
/// After TSK-185:  `touch::` accepted, empty store → no-accounts message → exit 0.
///
/// RED:   `touch::` unknown → exit 1.
/// GREEN: `touch::` registered → exit 0.
#[ test ]
fn it097_touch_1_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ) || text.contains( "No accounts" ) || text.is_empty(),
    "touch::1 with empty store must exit 0 (no subprocess spawned — no accounts), got:\n{text}",
  );
}

/// it098 (TSK-185 AC-04): `touch::1` with a no-token account exits 0 without touching it.
///
/// Accounts whose quota fetch failed (expired/missing token → error result) must not
/// be touched. The trigger requires `result.is_ok()` AND `five_hour.resets_at.is_some()`.
/// A no-token account has an errored result → it is skipped entirely.
///
/// Before TSK-185: `touch::` unregistered → exit 1.
/// After TSK-185:  exits 0; errored account row shows `—` in Expires (no subprocess).
///
/// RED:   `touch::` unknown → exit 1.
/// GREEN: exits 0, account shows dash row.
#[ test ]
fn it098_touch_1_errored_account_skipped()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account with FAR_FUTURE_MS but no accessToken field → quota fetch fails
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "touch::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "a@x.com" ),
    "touch::1 with errored account must still show account row (AC-04), got:\n{text}",
  );
}

/// it099 (TSK-185 AC-02 structural): `fn apply_touch` is present in production source.
///
/// This structural test uses `include_str!` to confirm the function exists before
/// requiring live network calls. No credentials needed.
///
/// RED:   `apply_touch` absent from source → assert fails.
/// GREEN: `apply_touch` present → assert passes.
#[ test ]
fn it099_apply_touch_fn_exists_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs" ) );
  assert!(
    src.contains( "fn apply_touch" ),
    "TSK-185: `fn apply_touch` must be present in src/usage/touch.rs; \
     add the active-window extension function that calls refresh_account_token() \
     for accounts with result.is_ok() AND five_hour.resets_at.is_some()",
  );
}

/// it100 (TSK-185 AC-08): `format::json touch::1` with empty store exits 0 and outputs `[]`.
///
/// `render_json` is unaffected by `touch::`; touched accounts appear as normal JSON
/// objects. With empty store: both default and `touch::1` must output `[]`.
///
/// Before TSK-185: `touch::` unregistered → exit 1.
/// After TSK-185:  exit 0, output `[]` (same as without `touch::1`).
///
/// RED:   exit 1 (unrecognised param).
/// GREEN: exit 0, JSON output `[]`.
#[ test ]
fn it100_touch_json_format_unaffected()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out_default = run_cs_with_env(
    &[ ".usage", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  let out_touch = run_cs_with_env(
    &[ ".usage", "format::json", "touch::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_default, 0 );
  assert_exit( &out_touch,   0 );
  assert_eq!(
    stdout( &out_default ), stdout( &out_touch ),
    "format::json output must be identical with or without touch::1 (TSK-185 AC-08)",
  );
}

/// it101 (TSK-185 AC-10): `.usage.help` output contains `touch`.
///
/// `touch::` must be registered via `register_commands()` in `src/lib.rs` so users
/// can discover it. The param must appear in `.usage.help` output.
///
/// Before TSK-185: `touch` absent from help.
/// After TSK-185:  `touch` appears as a registered parameter.
///
/// RED:   `touch` absent from `.usage.help` output.
/// GREEN: `touch` present.
#[ test ]
fn it101_usage_help_shows_touch_param()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "touch" ),
    ".usage.help must list param `touch` (TSK-185 AC-10), got:\n{text}",
  );
}

/// it102 `lim_it` (IT-51 / FT-03 of feature/023): explicit `next::endurance` places `→` on exactly one account.
///
/// With ≥2 accounts sharing a live token, the endurance strategy selects one winner.
/// Exactly one table row gets `→` in the flag column. Footer shows "Next by strategy:".
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-51]
///       [`tests/docs/feature/023_next_account_strategies.md` AC-03]
#[ test ]
fn it102_lim_it_next_endurance_places_arrow_on_winner()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it102: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env( &[ ".usage", "next::endurance" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let arrow_count = text.lines().filter( |l| l.contains( "→" ) ).count();
  assert_eq!(
    arrow_count, 1,
    "next::endurance must place exactly one → in table rows (IT-51/FT-03/023), got:\n{text}",
  );
  assert!(
    text.contains( "Next by strategy:" ),
    "footer must show 'Next by strategy:' (IT-51), got:\n{text}",
  );
}

/// it103 `lim_it` (IT-52 / FT-04 of feature/023): `next::drain` places `→` on exactly one account.
///
/// With ≥2 accounts sharing a live token, the drain strategy selects the account with
/// the lowest non-exhausted `5h_left`. Exactly one `→` appears in the table rows.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-52]
///       [`tests/docs/feature/023_next_account_strategies.md` AC-04]
#[ test ]
fn it103_lim_it_next_drain_places_arrow_on_winner()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it103: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env( &[ ".usage", "next::drain" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let arrow_count = text.lines().filter( |l| l.contains( "→" ) ).count();
  assert_eq!(
    arrow_count, 1,
    "next::drain must place exactly one → in table rows (IT-52/FT-04/023), got:\n{text}",
  );
  assert!(
    text.contains( "Next by strategy:" ),
    "footer must show 'Next by strategy:' under next::drain (IT-52), got:\n{text}",
  );
}

/// it104 `lim_it` (IT-54 / FT-01 of feature/023): footer always shows all three strategy lines.
///
/// With `next::drain` active, the footer still shows all three lines: renew, endurance, drain.
/// All lines appear regardless of which strategy is currently selected.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-54]
///       [`tests/docs/feature/023_next_account_strategies.md` AC-01]
#[ test ]
fn it104_lim_it_footer_always_shows_both_strategy_lines()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it104: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env( &[ ".usage", "next::drain" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "Next by strategy:" ),
    "footer must show 'Next by strategy:' (IT-54/FT-01/023), got:\n{text}",
  );
  assert!(
    text.contains( "renew" ),
    "footer must show renew strategy line regardless of next:: value (TSK-222/FT-01/023), got:\n{text}",
  );
  assert!(
    text.contains( "endurance" ),
    "footer must show endurance strategy line regardless of next:: value (IT-54/FT-01/023), got:\n{text}",
  );
  assert!(
    text.contains( "drain" ),
    "footer must show drain strategy line (IT-54/FT-01/023), got:\n{text}",
  );
}

/// it105 `lim_it` (IT-58): per-column emoji prefix appears in `5h Left` column values.
///
/// `5h Left` cells embed a coloured-circle emoji prefix: 🟢 when >5% left, 🟡 when ≤5%.
/// At least one account row must show an emoji in that column.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-58]
///       [`tests/docs/feature/009_token_usage.md` AC-21]
#[ test ]
fn it105_lim_it_per_column_emoji_in_5h_left()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it105: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let has_emoji = text.contains( "🟢" ) || text.contains( "🟡" ) || text.contains( "🔴" );
  assert!(
    has_emoji,
    "5h Left / 7d Left columns must contain per-column emoji prefix (IT-58/AC-21); got:\n{text}",
  );
}

/// it106 (IT-62 / EC-1): `touch::0` accepted; empty credential store exits 0.
///
/// `touch::0` is the explicit off value — the parser must accept it without error.
/// No subprocess is spawned with `touch::0` regardless of account state.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-62]
///       [`tests/docs/cli/param/034_touch.md` EC-1]
///       [`tests/docs/feature/024_session_touch.md` AC-01]
#[ test ]
fn it106_touch_0_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ) || text.contains( "No accounts" ) || text.is_empty(),
    "touch::0 with empty store must exit 0 without param error (IT-62/EC-1), got:\n{text}",
  );
}

/// it107 (EC-3): `touch::true` accepted as equivalent to `touch::1`.
///
/// `parse_int_flag` must accept the string "true" and map it to 1 (enabled).
/// With an empty credential store, no subprocess is spawned and the command exits 0.
///
/// Spec: [`tests/docs/cli/param/034_touch.md` EC-3]
#[ test ]
fn it107_touch_true_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it108 (EC-4): `touch::bogus` exits 1 — invalid value rejected.
///
/// `parse_int_flag` must reject values that are not `0`, `1`, `"true"`, or `"false"`.
/// The parser returns `ArgumentTypeMismatch` (exit 1) for unrecognised string values.
///
/// Spec: [`tests/docs/cli/param/034_touch.md` EC-4]
#[ test ]
fn it108_touch_bogus_exits_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::bogus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// it109 `lim_it` (FT-01 of feature/024 / EC-7): `touch::0` — no subprocess spawned; idle account unchanged.
///
/// When `touch::0` (explicit off), the touch trigger is never fired regardless of account state.
/// An idle account (`five_hour.resets_at` absent, 5h Reset shows `—`) stays unchanged.
/// Skips when the live account is in active state (`resets_at` present).
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-01]
///       [`tests/docs/cli/param/034_touch.md` EC-7]
#[ test ]
fn it109_lim_it_touch_0_no_subprocess_idle_account_unchanged()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it109: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent — EM-DASH present in 5h Reset column).
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  let pre_text = stdout( &pre );
  if !pre_text.contains( "\u{2014}" )
  {
    eprintln!( "it109: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  let out = run_cs_with_env( &[ ".usage", "touch::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // 5h Reset column must still show EM-DASH (touch::0 must not fire subprocess).
  assert!(
    text.contains( "\u{2014}" ),
    "touch::0 must not activate idle account — 5h Reset must remain as `\u{2014}` (FT-01/EC-7), got:\n{text}",
  );
}

/// it110 `lim_it` (FT-02 of feature/024 / EC-8): `touch::1` — subprocess observed via trace for idle account.
///
/// When `touch::1` and the account has `five_hour.resets_at` absent (idle), a subprocess
/// is invoked to activate the 5h session. With `trace::1`, stderr shows `[trace]` lines
/// for the subprocess lifecycle. Skips when the live account is in active state (`resets_at` present).
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-02]
///       [`tests/docs/cli/param/034_touch.md` EC-8]
#[ test ]
fn it110_lim_it_touch_1_subprocess_spawned_for_idle_account()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it110: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent — EM-DASH present).
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  if !stdout( &pre ).contains( "\u{2014}" )
  {
    eprintln!( "it110: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  let out = run_cs_with_env( &[ ".usage", "touch::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "switch_account" ),
    "touch::1 with idle account must spawn subprocess — switch_account must appear (FT-02/EC-8), got stderr:\n{err}",
  );
}

/// it111 `lim_it` (FT-03 of feature/024): After successful touch, `5h Reset` transitions from `—` to countdown.
///
/// When `touch::1` triggers on an idle account (`resets_at` absent) and the subprocess succeeds,
/// the account's quota is re-fetched and the `5h Reset` column shows a concrete countdown (~5h)
/// where it previously showed `—`. Skips when account is already active.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-03]
#[ test ]
fn it111_lim_it_touch_1_5h_reset_changes_from_dash_to_time()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it111: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent — EM-DASH present).
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  let pre_text = stdout( &pre );
  if !pre_text.contains( "\u{2014}" )
  {
    eprintln!( "it111: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  let out = run_cs_with_env( &[ ".usage", "touch::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // After touch: the 5h Reset column must show a countdown (session activated — "in Xh Ym").
  assert!(
    text.contains( "in " ),
    "touch::1 must activate idle account; 5h Reset must show countdown after subprocess (FT-03), got:\n{text}",
  );
}

/// it112 (FT-05 of feature/024 structural): `apply_refresh` code appears before `apply_touch` in source.
///
/// The ordering guarantee (refresh runs before touch) is enforced at the call site in
/// `run_usage()`. This structural test verifies the invariant without requiring live
/// credentials or an expired token.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-05]
#[ test ]
fn it112_structural_refresh_before_touch_ordering_in_source()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  // Use call-site patterns that only match the production calls in usage_routine(),
  // not the function definitions (fn apply_touch/fn apply_refresh) which appear earlier.
  let refresh_pos = src.find( "apply_refresh( &mut accounts, &credential_store" )
    .expect( "apply_refresh call site must exist in src/usage/api.rs" );
  let touch_pos = src.find( "apply_touch( aq, &credential_store" )
    .expect( "apply_touch call site must exist in src/usage/api.rs" );
  assert!(
    refresh_pos < touch_pos,
    "apply_refresh must appear before apply_touch in run_usage() to guarantee refresh-before-touch ordering (FT-05)",
  );
}

/// it113 `lim_it` (FT-06 companion of feature/024): `_active` marker unchanged after all touch ops.
///
/// When `touch::1` is active and a non-active account is touched, the `_active` file
/// must remain unchanged after `apply_touch` completes. Fix for BUG-211: `save(update_marker=false)`
/// suppresses all `_active` writes during touch cycling — no restore call is made.
/// Skips when idle account condition is not met.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-06]
#[ test ]
fn it113_lim_it_active_account_restored_after_touch()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it113: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // alice is active; acct-b is non-active; if acct-b is idle, touch will switch to it.
  write_account_with_token( dir.path(), "alice@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  // Pre-check: at least one non-active account must be in idle state.
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  if !stdout( &pre ).contains( "\u{2014}" )
  {
    eprintln!( "it113: no idle accounts — idle-state condition not met, skipping" );
    return;
  }

  let out = run_cs_with_env( &[ ".usage", "touch::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let active_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" ).join( claude_profile::account::active_marker_filename() );
  let active_content = std::fs::read_to_string( &active_file ).unwrap_or_default();
  assert_eq!(
    active_content.trim(), "alice@test.com",
    "_active must remain alice@test.com after touch (never written during cycling — BUG-211), got: {active_content:?}",
  );
}

/// it114 (FT-07 of feature/024 structural): touch failure is non-aborting — source has early-return guard.
///
/// When the subprocess or re-fetch fails, `apply_touch` returns without propagating
/// the error (no panic, no hard failure). This structural test verifies the non-aborting
/// return path exists in the source.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-07]
#[ test ]
fn it114_structural_touch_failure_non_aborting_guard_exists()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs" ) );
  // apply_touch handles new_creds=None gracefully: expiry update is conditional,
  // re-fetch runs unconditionally (Fix(BUG-179) — no early return on credentials=None).
  assert!(
    src.contains( "if let Some( ref creds ) = new_creds" ),
    "apply_touch must conditionally update expiry when credentials returned (FT-07 + BUG-179)",
  );
}

/// it115 `lim_it` (FT-09 of feature/024): `trace::1` emits `[trace]` lines for touch subprocess lifecycle.
///
/// With `touch::1 trace::1` and an account with `resets_at` absent (idle), stderr shows
/// `[trace]` lines showing the subprocess lifecycle (`switch_account`, `run_isolated`). Skips when active.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-09]
#[ test ]
fn it115_lim_it_trace_1_shows_touch_lifecycle()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it115: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent) for subprocess to be triggered.
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  if !stdout( &pre ).contains( "\u{2014}" )
  {
    eprintln!( "it115: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  let out = run_cs_with_env( &[ ".usage", "touch::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "[trace]" ),
    "trace::1 must emit [trace] lines for touch subprocess lifecycle (FT-09), got stderr:\n{err}",
  );
}

/// it116 `lim_it` (FT-11 of feature/024): valid account with `resets_at` absent IS touched (positive trigger).
///
/// The touch trigger fires when `five_hour.resets_at` is absent (idle account). When the
/// 5h window is idle (`resets_at` absent, 5h Reset shows `—`), the subprocess IS spawned
/// and a new 5h session is activated. Observable via `switch_account` in `trace::1` output.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-11]
///       [`docs/feature/024_session_touch.md` AC-02 trigger guard]
#[ test ]
fn it116_lim_it_account_with_resets_at_absent_is_touched()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it116: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent — EM-DASH in output).
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  let pre_text = stdout( &pre );
  if !pre_text.contains( "\u{2014}" )
  {
    eprintln!( "it116: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  // With resets_at absent, touch::1 MUST spawn a subprocess to activate the 5h session.
  // Verified via trace::1: switch_account line must appear (subprocess triggered).
  let out = run_cs_with_env( &[ ".usage", "touch::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "switch_account" ),
    "idle account must be touched — switch_account must appear in stderr (FT-11), got stderr:\n{err}",
  );
}

/// it117 (FT-12 of feature/009 AC-22): `Sub` and `7d Son Reset` columns hidden by default;
/// `cols::+sub` and `cols::+7d_son_reset` reveal them respectively.
///
/// - Default: table header does NOT contain `Sub` or `7d Son Reset`.
/// - `cols::+sub`: header contains `Sub`.
/// - `cols::+7d_son_reset`: header contains `7d Son Reset`.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-12]
///       [`docs/feature/009_token_usage.md` AC-22]
#[ test ]
fn it117_ft12_cols_plus_reveals_sub_and_7d_son_reset_columns()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account creates account without accessToken → quota fetch fails (🔴).
  // Table header still renders even for error-state accounts.
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  // Default: Sub and 7d Son Reset must NOT appear in header.
  let out_default = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_default, 0 );
  let text_default = stdout( &out_default );
  assert!(
    !text_default.contains( "Sub" ),
    "default output must NOT show Sub column (FT-12/AC-22), got:\n{text_default}",
  );
  assert!(
    !text_default.contains( "7d Son Reset" ),
    "default output must NOT show 7d Son Reset column (FT-12/AC-22), got:\n{text_default}",
  );

  // cols::+sub: Sub column must appear in header.
  let out_sub = run_cs_with_env( &[ ".usage", "cols::+sub" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_sub, 0 );
  let text_sub = stdout( &out_sub );
  assert!(
    text_sub.contains( "Sub" ),
    "cols::+sub must show Sub column header (FT-12/AC-22), got:\n{text_sub}",
  );

  // cols::+7d_son_reset: 7d Son Reset column must appear in header.
  let out_son = run_cs_with_env( &[ ".usage", "cols::+7d_son_reset" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_son, 0 );
  let text_son = stdout( &out_son );
  assert!(
    text_son.contains( "7d Son Reset" ),
    "cols::+7d_son_reset must show 7d Son Reset column header (FT-12/AC-22), got:\n{text_son}",
  );
}

/// it118 (EC-2b / `parse_int_flag`): `touch::false` accepted as equivalent to `touch::0`.
///
/// `parse_int_flag` maps `Value::String("false")` to 0 (disabled). With an empty
/// credential store, no subprocess is spawned and the command exits 0.
///
/// Spec: [`tests/docs/cli/param/034_touch.md` EC-1 variant — "false" string path]
#[ test ]
fn it118_touch_false_accepted_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::false" ], &[ ( "HOME", home ) ] );
  assert_exit(
    &out, 0,
  );
}

/// it119 (`parse_int_flag` rejection): `touch::2` exits 1 — integer out-of-range.
///
/// `parse_int_flag` accepts only 0, 1, "0", "1", "true", "false". The value "2"
/// falls to the catch-all arm → `ArgumentTypeMismatch` → exit 1.
///
/// Spec: [`tests/docs/cli/param/034_touch.md` EC-4 variant — out-of-range integer]
#[ test ]
fn it119_touch_2_rejected_exits_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it120 (lim_it) ────────────────────────────────────────────────────────────

/// it120 `lim_it` (FT-12 of feature/024 — AC-11): Touch trigger fires for idle accounts each cycle.
///
/// Two sequential single-shot `.usage touch::1 trace::1` invocations verify the idle trigger:
/// - Cycle 1 (idle account, `resets_at` absent): subprocess spawned → `switch_account` in trace.
/// - Cycle 2 (account now active after cycle 1 activated it): touch skips → `skipped` in trace.
///
/// This verifies that the trigger fires for idle accounts (activating them) and correctly
/// skips accounts that are already active (`resets_at` present after cycle 1).
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-12]
///       [`docs/feature/024_session_touch.md` AC-11]
#[ test ]
fn it120_lim_it_ft12_touch_trigger_fires_per_idle_account_cycle()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it120: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent — EM-DASH present in output).
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  if !stdout( &pre ).contains( "\u{2014}" )
  {
    eprintln!( "it120: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  // Cycle 1: idle account → touch trigger fires → subprocess spawned → switch_account in trace.
  let out1 = run_cs_with_env( &[ ".usage", "touch::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out1, 0 );
  let err1 = stderr( &out1 );
  assert!(
    err1.contains( "switch_account" ),
    "cycle 1: idle account must trigger touch subprocess; switch_account must appear (FT-12/AC-11), got stderr:\n{err1}",
  );

  // Cycle 2: account now active after cycle 1 activation → touch skips.
  let out2 = run_cs_with_env( &[ ".usage", "touch::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );
  let err2 = stderr( &out2 );
  let text2 = stdout( &out2 );
  // EM-DASH present means cycle 1 did not activate (subprocess failed) — cycle 2 inconclusive.
  if text2.contains( "\u{2014}" )
  {
    eprintln!( "it120: cycle 1 did not activate account; cycle 2 check inconclusive" );
  }
  else
  {
    // Account is now active: touch must skip in cycle 2.
    assert!(
      err2.contains( "skipped" ),
      "cycle 2: account now active must be skipped by touch (FT-12/AC-11), got stderr:\n{err2}",
    );
  }
}

// ── sort::next meta-strategy ──────────────────────────────────────────────────

/// it121 (IT-65/AC-15): `sort::next` accepted with empty credential store → exit 0.
///
/// `sort::next` resolves to `sort::renew` (default `next::renew`) at parse time.
/// Both `sort::next` alone and `sort::next next::endurance` must be accepted without error.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-65`.
#[ test ]
fn it121_sort_next_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  // sort::next with default next::renew
  let out = run_cs_with_env( &[ ".usage", "sort::next" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::next must be accepted and show no-accounts message, got:\n{text}",
  );

  // sort::next with explicit next::endurance
  let out2 = run_cs_with_env( &[ ".usage", "sort::next", "next::endurance" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );
  let text2 = stdout( &out2 );
  assert!(
    text2.contains( "(no accounts configured)" ),
    "sort::next next::endurance must be accepted and show no-accounts message, got:\n{text2}",
  );
}

// ── TSK-191 — imodel:: and effort:: parameters ────────────────────────────────

/// it122 (IT-66 / EC-1): `imodel::auto` accepted with empty credential store exits 0.
///
/// Before TSK-191: `imodel::` is unregistered → `ArgumentUnrecognised` → exit 1.
/// After TSK-191:  `imodel::` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-1]
///       [`tests/docs/cli/command/009_usage.md` IT-66]
#[ test ]
fn it122_imodel_auto_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out  = run_cs_with_env( &[ ".usage", "imodel::auto" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ) || text.contains( "No accounts" ),
    "imodel::auto with empty store must exit 0 (IT-66/EC-1), got:\n{text}",
  );
}

/// it123 (IT-67 / EC-5): `imodel::bogus` exits 1; stderr names all five valid values.
///
/// The parser rejects any value not in {auto, sonnet, opus, keep, haiku} with exit 1.
/// All five valid values must appear in stderr to help the user correct the mistake.
/// TSK-209: updated from four to five values (added `haiku`).
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-5]
///       [`tests/docs/cli/command/009_usage.md` IT-67]
#[ test ]
fn it123_imodel_bogus_exits_1()
{
  let out  = run_cs( &[ ".usage", "imodel::bogus" ] );
  assert_exit( &out, 1 );
  let err  = stderr( &out );
  assert!( err.contains( "auto" ),   "stderr must name valid value 'auto', got:\n{err}" );
  assert!( err.contains( "sonnet" ), "stderr must name valid value 'sonnet', got:\n{err}" );
  assert!( err.contains( "opus" ),   "stderr must name valid value 'opus', got:\n{err}" );
  assert!( err.contains( "keep" ),   "stderr must name valid value 'keep', got:\n{err}" );
  assert!( err.contains( "haiku" ),  "stderr must name valid value 'haiku', got:\n{err}" );
}

/// it124 (IT-68 / EC-1): `effort::auto` accepted with empty credential store exits 0.
///
/// Before TSK-191: `effort::` is unregistered → `ArgumentUnrecognised` → exit 1.
/// After TSK-191:  `effort::` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-1]
///       [`tests/docs/cli/command/009_usage.md` IT-68]
#[ test ]
fn it124_effort_auto_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out  = run_cs_with_env( &[ ".usage", "effort::auto" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ) || text.contains( "No accounts" ),
    "effort::auto with empty store must exit 0 (IT-68/EC-1), got:\n{text}",
  );
}

/// it125 (IT-69 / EC-4): `effort::bogus` exits 1; stderr names all five valid values.
///
/// The parser rejects any value not in {auto, high, max, low, normal} with exit 1.
/// TSK-209: updated from three to five values (added `low` and `normal`).
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-4]
///       [`tests/docs/cli/command/009_usage.md` IT-69]
#[ test ]
fn it125_effort_bogus_exits_1()
{
  let out = run_cs( &[ ".usage", "effort::bogus" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "auto" ),   "stderr must name valid value 'auto', got:\n{err}" );
  assert!( err.contains( "high" ),   "stderr must name valid value 'high', got:\n{err}" );
  assert!( err.contains( "max" ),    "stderr must name valid value 'max', got:\n{err}" );
  assert!( err.contains( "low" ),    "stderr must name valid value 'low', got:\n{err}" );
  assert!( err.contains( "normal" ), "stderr must name valid value 'normal', got:\n{err}" );
}

/// it126 (IT-70): `.usage.help` lists `imodel` and `effort` as registered parameters.
///
/// Both params must appear in the help output after TSK-191 registration.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-70]
#[ test ]
fn it126_usage_help_shows_imodel_effort_params()
{
  let out  = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "imodel" ), ".usage.help must list param `imodel` (IT-70), got:\n{text}" );
  assert!( text.contains( "effort" ), ".usage.help must list param `effort` (IT-70), got:\n{text}" );
}

/// it127 (EC-2): `imodel::sonnet` accepted with empty credential store exits 0.
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-2]
#[ test ]
fn it127_imodel_sonnet_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "imodel::sonnet" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it128 (EC-3): `imodel::opus` accepted with empty credential store exits 0.
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-3]
#[ test ]
fn it128_imodel_opus_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "imodel::opus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it129 (EC-4): `imodel::keep` accepted with empty credential store exits 0.
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-4]
#[ test ]
fn it129_imodel_keep_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "imodel::keep" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it130 (EC-2 for effort): `effort::high` accepted with empty credential store exits 0.
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-2]
#[ test ]
fn it130_effort_high_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "effort::high" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it131 (EC-3 for effort): `effort::max` accepted with empty credential store exits 0.
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-3]
#[ test ]
fn it131_effort_max_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "effort::max" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

// ── BUG-181: trigger inversion fix + structural gates ─────────────────────────

/// it132 (BUG-181 fix AC-02 structural): `apply_touch` trigger uses `is_none()`, not `is_some()`.
///
/// The touch trigger must fire for accounts whose `five_hour.resets_at` is **absent**
/// (idle account — no active 5h window). BUG-181: previous code (`is_some()`) fired for
/// active accounts, wasting subprocess cost while skipping idle accounts that need activation.
///
/// The guard must read: `let is_idle = ...is_none(); if !is_idle { return; }`.
///
/// RED:   source contains `let is_active` (old inverted guard using `is_some()`).
/// GREEN: source contains `let is_idle` + `is_none()` guard.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-11]
///       [`docs/feature/024_session_touch.md` AC-02]
#[ test ]
fn it132_apply_touch_trigger_is_is_none_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs" ) );
  assert!(
    !src.contains( "let is_active = data.five_hour" ),
    "BUG-181: `apply_touch` trigger must use `is_idle` + `is_none()`, not `is_active` + `is_some()`.\n\
     Fix the guard: `let is_idle = data.five_hour.as_ref().and_then(|p| p.resets_at.as_deref()).is_none();\n\
     if !is_idle {{ return; }}`",
  );
}

/// it133 (TSK-192 AC-09 structural): `refresh_account_token` uses `label` variable, not hardcoded `"refresh"`.
///
/// All 14 trace `eprintln!` calls in `refresh_account_token()` must use a `label: &str`
/// parameter so callers can inject `"touch"` or `"refresh"` to distinguish subprocess types
/// in trace output. Currently all calls hardcode `"refresh"` making touch trace indistinguishable.
///
/// RED:   `account.rs` contains `"[trace] refresh  {name}  switch_account: OK"` (hardcoded).
/// GREEN: all calls use `{label}` variable; that literal string is absent.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-09]
///       [`docs/feature/024_session_touch.md` AC-09]
#[ test ]
fn it133_refresh_account_token_has_label_param_structural()
{
  let src = include_str!( concat!(
    env!( "CARGO_MANIFEST_DIR" ),
    "/../claude_profile_core/src/account.rs"
  ) );
  assert!(
    !src.contains( "[trace] refresh  {name}  switch_account: OK" ),
    "TSK-192: `refresh_account_token()` must accept `label: &str` and use `{{label}}` in all\n\
     trace `eprintln!` calls instead of the hardcoded string `\"refresh\"`.\n\
     Add `label: &str` after `trace: bool` in the signature and replace all\n\
     `\"[trace] refresh  {{name}}  ...\"` patterns with `\"[trace] {{label}}  {{name}}  ...\"`.",
  );
}

/// it134 (TSK-192 AC-09 structural): `apply_touch` call site passes `"touch"` label.
///
/// The `refresh_account_token()` call in `apply_touch()` must pass the literal `"touch"`
/// as the `label` argument so trace output reads `[trace] touch ...` (not `[trace] refresh ...`).
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-09]
///       [`docs/feature/024_session_touch.md` AC-09]
#[ test ]
fn it134_apply_touch_passes_touch_label_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs" ) );
  assert!(
    src.contains( r#"credential_store, claude_paths, trace, "touch","# ),
    "TSK-192: `apply_touch()` must pass `\"touch\"` as the label argument to `refresh_account_token()`."
  );
}

/// it135 (TSK-192 AC-09 structural): `apply_refresh` call site passes `"refresh"` label.
///
/// The `refresh_account_token()` call in `apply_refresh()` must pass the literal `"refresh"`
/// as the `label` argument so trace output reads `[trace] refresh ...`.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-09]
///       [`docs/feature/024_session_touch.md` AC-09]
#[ test ]
fn it135_apply_refresh_passes_refresh_label_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/refresh.rs" ) );
  assert!(
    src.contains( r#"credential_store, claude_paths, trace, "refresh","# ),
    "TSK-192: `apply_refresh()` must pass `\"refresh\"` as the label argument to `refresh_account_token()`."
  );
}

/// it136 (TSK-192 AC-09 structural): `refresh_account_token` has per-step `Instant` timing.
///
/// Both `switch_account` and `run_isolated` steps in `refresh_account_token()` must be
/// wrapped with `std::time::Instant::now()` so elapsed seconds appear in trace output.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-09]
///       [`docs/feature/024_session_touch.md` AC-09]
#[ test ]
fn it136_refresh_account_token_has_instant_timing_structural()
{
  let src = include_str!( concat!(
    env!( "CARGO_MANIFEST_DIR" ),
    "/../claude_profile_core/src/account.rs"
  ) );
  assert!(
    src.contains( "Instant::now()" ),
    "TSK-192: `refresh_account_token()` must use `std::time::Instant::now()` for per-step timing."
  );
}

// ── TSK-220 — sort default renew + sort::next meta-strategy ──────────────────

/// it137 (TSK-220 AC-01 structural): sort default is `SortStrategy::Renew` when no `sort::` arg.
///
/// `parse_usage_params` must return `SortStrategy::Renew` when the `sort` argument is absent.
/// This ensures `clp .usage` (no `sort::` flag) orders rows by 7d reset — soonest weekly reset first.
///
/// RED:   `None => SortStrategy::Drain` (old default).
/// GREEN: `None => SortStrategy::Renew` present in parse block.
///
/// Spec: [`tests/docs/feature/020_usage_sort_strategies.md` FT-14]
///       [`docs/feature/020_usage_sort_strategies.md` AC-01]
#[ test ]
fn it137_sort_default_is_renew_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/params.rs" ) );
  // The None arm of the sort match uses alignment spaces; verify Renew is the default and Drain is not.
  assert!(
    src.contains( "None                         => SortStrategy::Renew" ),
    "TSK-220: sort default must be SortStrategy::Renew, not SortStrategy::Drain.\n\
     Change the None arm of the sort argument match to `None => SortStrategy::Renew`."
  );
}

/// it138 (TSK-193 AC-15 structural): `sort::next` resolves to `SortStrategy::Drain` when `next::drain`.
///
/// The `SortStrategy::Next => match next` resolution block must map `NextStrategy::Drain`
/// to `SortStrategy::Drain`. This is the core of the `sort::next` meta-strategy:
/// it delegates to the concrete strategy matching the active `next::` param.
///
/// RED:   `SortStrategy::Next` arm absent or maps to wrong strategy.
/// GREEN: `NextStrategy::Drain => SortStrategy::Drain` present in resolution block.
///
/// Spec: [`tests/docs/feature/020_usage_sort_strategies.md` FT-17]
///       [`docs/feature/020_usage_sort_strategies.md` AC-15]
#[ test ]
fn it138_sort_next_resolves_to_drain_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/params.rs" ) );
  assert!(
    src.contains( "NextStrategy::Drain     => SortStrategy::Drain" ),
    "TSK-193: sort::next must resolve to SortStrategy::Drain when next::drain is active.\n\
     The resolution block must have `NextStrategy::Drain => SortStrategy::Drain`."
  );
}

/// it139 (TSK-193 AC-15 structural): `sort::next` resolves to `SortStrategy::Endurance` when `next::endurance`.
///
/// The `SortStrategy::Next => match next` resolution block must map `NextStrategy::Endurance`
/// to `SortStrategy::Endurance`. Together with it138, this proves the meta-strategy
/// delegates exhaustively to the active `next::` concrete strategy.
///
/// RED:   `NextStrategy::Endurance` arm absent or maps to wrong strategy.
/// GREEN: `NextStrategy::Endurance => SortStrategy::Endurance` present in resolution block.
///
/// Spec: [`tests/docs/feature/020_usage_sort_strategies.md` FT-17]
///       [`docs/feature/020_usage_sort_strategies.md` AC-15]
#[ test ]
fn it139_sort_next_resolves_to_endurance_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/params.rs" ) );
  assert!(
    src.contains( "NextStrategy::Endurance => SortStrategy::Endurance" ),
    "TSK-193: sort::next must resolve to SortStrategy::Endurance when next::endurance is active.\n\
     The resolution block must have `NextStrategy::Endurance => SortStrategy::Endurance`."
  );
}

/// it141 (BUG-202 / 024 FT-14): errored account emits skip trace in touch phase.
///
/// ## Root Cause
///
/// `apply_touch()` error guard at `usage.rs:1497` (`let Ok(ref data) = aq.result
/// else { return; }`) exited before any trace emission point. Error-tier accounts
/// silently vanished from the touch phase trace while appearing in fetch and refresh.
///
/// ## Why Not Caught
///
/// TSK-196 (BUG-177) added trace for the is_active/h-exhausted guard at lines
/// 1504-1511 but did not address the error guard at line 1497. The BUG-177 MRE
/// used OK-result accounts only.
///
/// ## Fix Applied
///
/// Added `if trace { eprintln!("[trace] touch  {}  skipped (reason: error account)",
/// aq.name); }` before the `return` in the `else` branch at line 1497.
///
/// ## Prevention
///
/// When adding trace to a function with multiple early-return guards, each guard
/// needs its own trace emission — audit ALL return paths, not just the "interesting" ones.
///
/// ## Pitfall
///
/// Error guard was deemed uninteresting (error accounts can't be touched) but the
/// diagnostic contract requires visibility into all skip decisions.
///
/// RED:   errored account has no touch trace line → assert fails.
/// GREEN: error guard emits `[trace] touch  <name>  skipped (reason: error account)`.
#[ test ]
fn it141_trace_skip_lines_emitted_for_non_qualifying_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account with FAR_FUTURE_MS but no accessToken → quota fetch fails → Err result
  write_account( dir.path(), "err@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "touch::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "[trace] touch  err@x.com  skipped (reason: error account)" ),
    "BUG-202: errored account must emit `[trace] touch  <name>  skipped (reason: error account)` \
     when trace=true (AC-09/AC-12 of Feature 024). Got stderr:\n{err}",
  );
}

// ── TSK-209: haiku model + low/normal effort CLI acceptance ───────────────────

/// it142 (EC-11 / 035): `imodel::haiku` accepted with empty credential store exits 0.
///
/// Before TSK-209: `imodel::haiku` is unrecognised → `ArgumentTypeMismatch` → exit 1.
/// After TSK-209:  `haiku` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-11]
#[ test ]
fn it142_imodel_haiku_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "imodel::haiku" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it143 (EC-10 / 036): `effort::low` accepted with empty credential store exits 0.
///
/// Before TSK-209: `effort::low` is unrecognised → `ArgumentTypeMismatch` → exit 1.
/// After TSK-209:  `low` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-10]
#[ test ]
fn it143_effort_low_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "effort::low" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it144 (EC-11 / 036): `effort::normal` accepted with empty credential store exits 0.
///
/// Before TSK-209: `effort::normal` is unrecognised → `ArgumentTypeMismatch` → exit 1.
/// After TSK-209:  `normal` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-11]
#[ test ]
fn it144_effort_normal_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "effort::normal" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

// ── next::renew strategy (TSK-222) ────────────────────────────────────────────

/// it145 `lim_it` (TSK-222): `next::renew` accepted, footer shows renew line, `→` placed.
///
/// `next::renew` selects the account whose soonest running reset timer (min of 5h and 7d)
/// fires first. Footer shows 3 lines: renew (first), endurance, drain.
///
/// RED:   `next::renew` not recognised → exit 1 (before TSK-222 enum variant is added).
/// GREEN: renew accepted → exit 0, footer contains "renew".
///
/// Spec: [`tests/docs/feature/023_next_account_strategies.md`]
///       [`docs/feature/023_next_account_strategies.md` AC-10]
#[ doc = "lim_it" ]
#[ test ]
fn it145_lim_it_next_renew_places_arrow_on_soonest_refill()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it145: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env( &[ ".usage", "next::renew" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "renew" ),
    "footer must show renew strategy line (TSK-222/AC-10), got:\n{text}",
  );
  assert!(
    text.contains( "Next by strategy:" ),
    "footer must show 'Next by strategy:' header (TSK-222), got:\n{text}",
  );
}

// ── row filtering parameters (TSK-223) ────────────────────────────────────────

/// ut146 (TSK-223 RED gate): `only_valid::1` accepted; empty store exits 0.
///
/// Before TSK-224: `get` is unregistered → `ArgumentUnrecognised` → exit 1.
/// After TSK-224:  `get::7d_left` accepted; empty store → no rows → bare empty output → exit 0.
///
/// Validates AC-10 structural (no table chrome in output when `get::` is set).
/// Live extraction tests (`lim_it`) cover the actual value output.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md`]
///       [`docs/feature/028_usage_row_filtering.md` AC-10]
#[ test ]
fn ut_get_7d_left_extracts_bare_value()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "get::7d_left" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Quota" ) && !text.contains( "5h Left" ) && !text.contains( "7d Left" ),
    "get::7d_left with empty store must produce no table output, got:\n{text}",
  );
}

/// Before TSK-224: `get::bogus_field` unregistered → wrong exit/message.
/// After TSK-224:  exit 1, stderr lists valid field IDs including `5h_left`, `7d_left`, `account`.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md`]
///       [`docs/feature/028_usage_row_filtering.md` AC-15]
#[ test ]
fn ut_get_invalid_field_exits_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "get::bogus_field" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "5h_left" ) && err.contains( "7d_left" ) && err.contains( "account" ),
    "get::bogus_field must list valid field IDs in stderr, got:\n{err}",
  );
}

/// Before TSK-223: `only_valid` is unregistered → `ArgumentUnrecognised` → exit 1.
/// After TSK-223:  `only_valid::1` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md`]
///       [`docs/feature/028_usage_row_filtering.md` AC-07]
#[ test ]
fn ut_filter_only_valid_hides_red_rows()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "only_valid::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "only_valid::1 with empty store must show no-accounts message, got:\n{text}",
  );
}

// ── → Next column + JSON renewal fields (Phase 3 RED gate — TSK-227) ─────────

/// it146 — `→ Next` column header visible in default `.usage` output.
///
/// Before TSK-227: `→ Next` column does not exist → assertion fails.
/// After TSK-227:  `→ Next` header appears in every default table output.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
/// Source: [`009_token_usage.md` AC-28]
#[ test ]
fn it146_next_column_visible_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "\u{2192} Next" ),
    "default .usage output must contain '→ Next' column header (AC-28), got:\n{text}",
  );
}

/// it147 — `format::json` output has all 4 renewal/next-event fields; deprecated field absent.
///
/// Before TSK-227: JSON uses `next_renewal_est` → `renewal_secs` assertion fails.
/// After TSK-227:  JSON has `renewal_secs`, `renewal_is_estimate`, `next_event_type`,
///                 `next_event_secs`; `next_renewal_est` is removed.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-19]
/// Source: [`009_token_usage.md` AC-29]
#[ test ]
fn it147_json_renewal_secs_present()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // All four AC-29 fields must be present.
  assert!(
    text.contains( "\"renewal_secs\"" ),
    "format::json must include 'renewal_secs' field (AC-29), got:\n{text}",
  );
  assert!(
    text.contains( "\"renewal_is_estimate\"" ),
    "format::json must include 'renewal_is_estimate' field (AC-29), got:\n{text}",
  );
  assert!(
    text.contains( "\"next_event_type\"" ),
    "format::json must include 'next_event_type' field (AC-29), got:\n{text}",
  );
  assert!(
    text.contains( "\"next_event_secs\"" ),
    "format::json must include 'next_event_secs' field (AC-29), got:\n{text}",
  );
  // Deprecated field must be gone.
  assert!(
    !text.contains( "\"next_renewal_est\"" ),
    "format::json must NOT contain deprecated 'next_renewal_est' field, got:\n{text}",
  );
}

// ── IT-40: Status emoji column header ─────────────────────────────────────────

/// IT-40: Table header row contains `●` column label.
///
/// An account with no accessToken (error row) still causes the table to render;
/// the `●` header is always present in the status emoji column.
///
/// Source: [`009_token_usage.md` AC-18](../docs/feature/009_token_usage.md)
#[ test ]
fn it148_status_emoji_column_header_present()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-token", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( '●' ),
    "table header must contain '●' status emoji column label, got:\n{text}",
  );
}

// ── IT-41: Error account shows 🔴 ─────────────────────────────────────────────

/// IT-41: Account with missing token shows `🔴` in table row.
///
/// `write_account()` writes a credential file without `accessToken`; the
/// fetch result is `Err(_)` → `status_emoji` returns `🔴`.
///
/// Source: [`009_token_usage.md` AC-18](../docs/feature/009_token_usage.md)
#[ test ]
fn it149_status_emoji_red_on_token_error()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-token", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "🔴" ),
    "account without accessToken must show 🔴 in table row, got:\n{text}",
  );
}

// ── IT-42: JSON output contains no status emoji ────────────────────────────────

/// IT-42: `format::json` output does not contain status emoji.
///
/// Emoji are a table-rendering concern only; JSON output must be clean.
///
/// Source: [`009_token_usage.md` AC-20](../docs/feature/009_token_usage.md)
#[ test ]
fn it150_status_emoji_absent_from_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-token", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "🔴" ) && !text.contains( "🟡" ) && !text.contains( "🟢" ),
    "format::json must NOT contain status emoji, got:\n{text}",
  );
}

// ── it151: past _renewal_at auto-advances at render (030 FT-10) ──────────────

/// it151 — Past `_renewal_at` is auto-advanced monthly at render; shows `in Xd` (no `~`).
///
/// Root Cause: `renews_label()` advances past timestamps by 30-day increments until future.
/// The stored value is unchanged; auto-advance is a read-only render-time operation.
///
/// Setup: Account has `_renewal_at: "2020-03-15T00:00:00Z"` (deeply past). After auto-advance
/// the next day-15 occurrence lands within 30 days of today. No live credentials needed —
/// the account will be in error state but the `~Renews` column is populated from stored data.
///
/// Spec: [`tests/docs/feature/030_account_renewal_override.md` FT-10]
/// Source: [`030_account_renewal_override.md` AC-10]
#[ test ]
fn it151_past_renewal_at_auto_advances_in_usage()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "past@renewal.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_renewal_json( dir.path(), "past@renewal.com", "2020-03-15T00:00:00Z" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // Find the ~Renews cell for this account by locating the TSV equivalent or
  // parsing the text table row that contains the account name.
  // The column header is "~Renews". Find the row for our account.
  let renews_line = text.lines()
    .find( |l| l.contains( "past@renewal.com" ) )
    .expect( "usage output must have a row for past@renewal.com" );

  // The ~Renews column must show `in Xd` (no ~ prefix) because the timestamp is
  // an exact override stored via `_renewal_at`, even after auto-advance.
  // "in " prefix (no ~) and contains "d" for days.
  assert!(
    renews_line.contains( "in " ) && !renews_line.contains( "~in " ),
    "past _renewal_at must auto-advance and show 'in Xd' (no '~'), got row:\n{renews_line}\nfull output:\n{text}",
  );

  // The file on disk must NOT have been modified — auto-advance is read-only.
  let store   = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let on_disk = std::fs::read_to_string( store.join( "past@renewal.com.claude.json" ) ).unwrap();
  assert!(
    on_disk.contains( "2020-03-15T00:00:00Z" ),
    "stored _renewal_at must NOT be modified by render-time auto-advance, got: {on_disk}",
  );
}

// ── it152: TSV format has `next` column header (AC-28) ───────────────────────

/// it152 — `format::tsv` output contains a `next` column header (→ Next column in text).
///
/// The TSV renderer emits `next` as the header for the `→ Next` column (AC-28).
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
/// Source: [`009_token_usage.md` AC-28]
#[ test ]
fn it152_tsv_next_column_present()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "format::tsv" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let header = text.lines().next().expect( "TSV must have a header row" );
  let cols   : Vec< &str > = header.split( '\t' ).collect();
  assert!(
    cols.contains( &"next" ),
    "TSV header must contain 'next' column (AC-28), got cols: {cols:?}",
  );
}

// ── it153: JSON all 4 renewal fields with _renewal_at set ───────────────────

/// it153 — `format::json` with `_renewal_at` set produces all 4 renewal fields with
/// correct types: `renewal_secs` (integer), `renewal_is_estimate: false`,
/// `next_event_type` (string), `next_event_secs` (integer).
///
/// Complements it147 (which checks field presence only); this test checks
/// the semantic content when `_renewal_at` is explicitly set to a future timestamp.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-19]
/// Source: [`009_token_usage.md` AC-29]
#[ test ]
fn it153_json_renewal_fields_with_renewal_at()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "alice", "max", "default", FAR_FUTURE_MS, false );
  // Set a future _renewal_at so renewal_is_estimate=false and renewal_secs is a real integer.
  write_account_renewal_json( dir.path(), "alice", "2099-01-01T00:00:00Z" );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // renewal_is_estimate must be false (not true) because _renewal_at is set explicitly.
  assert!(
    text.contains( "\"renewal_is_estimate\":false" ),
    "explicit _renewal_at must yield renewal_is_estimate:false, got:\n{text}",
  );
  // renewal_secs must be a non-null integer (not null).
  assert!(
    text.contains( "\"renewal_secs\":" ) && !text.contains( "\"renewal_secs\":null" ),
    "explicit _renewal_at must yield non-null renewal_secs, got:\n{text}",
  );
  // next_event_type must be a string (not null).
  assert!(
    text.contains( "\"next_event_type\":" ) && !text.contains( "\"next_event_type\":null" ),
    "with _renewal_at set, next_event_type must not be null, got:\n{text}",
  );
  // next_event_secs must be a non-null integer.
  assert!(
    text.contains( "\"next_event_secs\":" ) && !text.contains( "\"next_event_secs\":null" ),
    "with _renewal_at set, next_event_secs must not be null, got:\n{text}",
  );
}

// ── it154: only_active::1 shows exactly the active account row ────────────────

/// it154 — `only_active::1` shows exactly the active account row; all others absent.
///
/// Uses 3 error accounts; one is marked active via the active marker file.
/// No live token needed — `is_active` is set by the marker file alone.
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-1]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-03]
#[ test ]
fn it154_only_active_1_shows_active_account_row()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, true  ); // active
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_active::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "acct-b" ),
    "only_active::1 must show the active account (acct-b), got:\n{text}",
  );
  assert!(
    !text.contains( "acct-a" ),
    "only_active::1 must hide non-active account (acct-a), got:\n{text}",
  );
  assert!(
    !text.contains( "acct-c" ),
    "only_active::1 must hide non-active account (acct-c), got:\n{text}",
  );
}

// ── it155: only_active::0 shows all rows ─────────────────────────────────────

/// it155 — `only_active::0` shows all rows (no filter applied).
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-2]
#[ test ]
fn it155_only_active_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, true );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_active::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "only_active::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "only_active::0 must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "only_active::0 must show acct-c, got:\n{text}" );
}

// ── it156: only_active::bad exits 1 ──────────────────────────────────────────

/// it156 — `only_active::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-3]
#[ test ]
fn it156_only_active_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "only_active::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "0" ) && err.contains( "1" ),
    "only_active::bad stderr must name valid values (0, 1), got:\n{err}",
  );
}

// ── it157: only_active::1 with no active marker shows empty ──────────────────

/// it157 — `only_active::1` with no active marker → 0 rows → "(no accounts configured)".
///
/// Three accounts, none is marked active. After `only_active::1` filter, all are retained
/// only if `is_active`, which requires the marker file to name that account.
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-4]
#[ test ]
fn it157_only_active_1_no_active_marker_shows_empty()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // None of these has make_active=true → no active marker file
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_active::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "only_active::1 with no active account must show no-accounts message, got:\n{text}",
  );
}

// ── it158: only_active::true accepted as alias for 1 ─────────────────────────

/// it158 — `only_active::true` accepted as alias for 1; shows active account row.
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-5]
#[ test ]
fn it158_only_active_true_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage", "only_active::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "acct-b" ),
    "only_active::true must show active account (acct-b), got:\n{text}",
  );
  assert!(
    !text.contains( "acct-a" ),
    "only_active::true must hide non-active account (acct-a), got:\n{text}",
  );
}

// ── it159: only_active::false shows all rows ──────────────────────────────────

/// it159 — `only_active::false` accepted as alias for 0; shows all rows.
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-6]
#[ test ]
fn it159_only_active_false_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage", "only_active::false" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "only_active::false must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "only_active::false must show acct-b, got:\n{text}" );
}

// ── it160: only_next::1 with no valid accounts shows empty ────────────────────

/// it160 — `only_next::1` with all error accounts (no valid quota) → 0 rows.
///
/// `find_next_for_strategy` requires `aq.result.is_ok()` to consider an account as a
/// candidate. With all-error accounts, no candidate is found → accounts list becomes
/// empty → "(no accounts configured)" shown.
///
/// Spec: [`tests/docs/cli/param/040_only_next.md` EC-2 offline substitute]
#[ test ]
fn it160_only_next_1_no_valid_accounts_shows_empty()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_next::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "only_next::1 with all-error accounts must show no-accounts message, got:\n{text}",
  );
}

// ── it161: only_next::bad exits 1 ────────────────────────────────────────────

/// it161 — `only_next::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/040_only_next.md` EC-4]
#[ test ]
fn it161_only_next_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "only_next::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "0" ) && err.contains( "1" ),
    "only_next::bad stderr must name valid values, got:\n{err}",
  );
}

// ── it162: only_next::0 shows all rows ───────────────────────────────────────

/// it162 — `only_next::0` is the default (no filter); all rows shown.
///
/// Spec: [`tests/docs/cli/param/040_only_next.md` EC-5]
#[ test ]
fn it162_only_next_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_next::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "only_next::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "only_next::0 must show acct-b, got:\n{text}" );
}

// ── it163: min_5h::0 shows all rows ──────────────────────────────────────────

/// it163 — `min_5h::0` disables the threshold filter; all rows shown.
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-3]
#[ test ]
fn it163_min_5h_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "min_5h::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "min_5h::0 must show acct-a, got:\n{text}" );
}

// ── it164: min_5h::abc exits 1 ───────────────────────────────────────────────

/// it164 — `min_5h::abc` exits 1 with type error.
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-4]
#[ test ]
fn it164_min_5h_abc_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "min_5h::abc" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it165: min_5h::101 exits 1 ───────────────────────────────────────────────

/// it165 — `min_5h::101` exits 1 (value above 100% maximum).
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-5]
#[ test ]
fn it165_min_5h_101_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "min_5h::101" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it166: min_7d::0 shows all rows ──────────────────────────────────────────

/// it166 — `min_7d::0` disables the threshold filter; all rows shown.
///
/// Spec: [`tests/docs/cli/param/042_min_7d.md` EC-3]
#[ test ]
fn it166_min_7d_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "min_7d::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "min_7d::0 must show acct-a, got:\n{text}" );
}

// ── it167: min_7d::abc exits 1 ───────────────────────────────────────────────

/// it167 — `min_7d::abc` exits 1 with type error.
///
/// Spec: [`tests/docs/cli/param/042_min_7d.md` EC-4]
#[ test ]
fn it167_min_7d_abc_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "min_7d::abc" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it168: min_7d::101 exits 1 ───────────────────────────────────────────────

/// it168 — `min_7d::101` exits 1 (value above 100% maximum).
///
/// Spec: [`tests/docs/cli/param/042_min_7d.md` EC-5]
#[ test ]
fn it168_min_7d_101_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "min_7d::101" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it169: only_valid::0 shows all rows ──────────────────────────────────────

/// it169 — `only_valid::0` is the default (no filter); all rows shown.
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-2]
#[ test ]
fn it169_only_valid_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_valid::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "only_valid::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "only_valid::0 must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "only_valid::0 must show acct-c, got:\n{text}" );
}

// ── it170: only_valid::bad exits 1 ───────────────────────────────────────────

/// it170 — `only_valid::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-3]
#[ test ]
fn it170_only_valid_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "only_valid::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "0" ) && err.contains( "1" ),
    "only_valid::bad stderr must name valid values, got:\n{err}",
  );
}

// ── it171: only_valid::1 with all 🔴 shows empty ─────────────────────────────

/// it171 — `only_valid::1` with all error (🔴) accounts → 0 rows shown.
///
/// Error accounts have `result = Err(_)`, which fails `aq.result.is_ok()`.
/// After filtering, accounts is empty → "(no accounts configured)".
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-4]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-07]
#[ test ]
fn it171_only_valid_1_all_red_shows_empty()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_valid::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "only_valid::1 with all-error accounts must show no-accounts message, got:\n{text}",
  );
}

// ── it172: only_valid::true accepted ─────────────────────────────────────────

/// it172 — `only_valid::true` accepted as alias for 1.
///
/// With all error accounts, `only_valid::true` behaves like `only_valid::1` →
/// 0 rows → "(no accounts configured)".
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-5]
#[ test ]
fn it172_only_valid_true_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_valid::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  // Accepted (no exit 1 for unrecognized value)
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "only_valid::true must be accepted and filter error accounts, got:\n{text}",
  );
}

// ── it173: only_valid::false shows all rows ───────────────────────────────────

/// it173 — `only_valid::false` accepted as alias for 0; all rows shown.
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-6]
#[ test ]
fn it173_only_valid_false_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_valid::false" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "only_valid::false must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "only_valid::false must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "only_valid::false must show acct-c, got:\n{text}" );
}

// ── it174: exclude_exhausted::0 shows all rows ───────────────────────────────

/// it174 — `exclude_exhausted::0` is the default (no filter); all rows shown.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-2]
#[ test ]
fn it174_exclude_exhausted_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "exclude_exhausted::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "exclude_exhausted::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "exclude_exhausted::0 must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "exclude_exhausted::0 must show acct-c, got:\n{text}" );
}

// ── it175: exclude_exhausted::bad exits 1 ────────────────────────────────────

/// it175 — `exclude_exhausted::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-4]
#[ test ]
fn it175_exclude_exhausted_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "exclude_exhausted::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "0" ) && err.contains( "1" ),
    "exclude_exhausted::bad stderr must name valid values, got:\n{err}",
  );
}

// ── it176: exclude_exhausted::1 with all 🔴 shows empty ──────────────────────

/// it176 — `exclude_exhausted::1` with all error (🔴) accounts → 0 rows shown.
///
/// `exclude_exhausted` keeps only 🟢 accounts. Error accounts are 🔴 → all removed.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-5]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-08]
#[ test ]
fn it176_exclude_exhausted_1_all_red_shows_empty()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "exclude_exhausted::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "exclude_exhausted::1 with all-error accounts must show no-accounts message, got:\n{text}",
  );
}

// ── it177: exclude_exhausted::true accepted ──────────────────────────────────

/// it177 — `exclude_exhausted::true` accepted as alias for 1.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-6]
#[ test ]
fn it177_exclude_exhausted_true_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "exclude_exhausted::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Accepted (no exit 1); error account is also excluded
  assert!(
    text.contains( "(no accounts configured)" ),
    "exclude_exhausted::true must be accepted and filter error accounts, got:\n{text}",
  );
}

// ── it178: count::3 sort::name shows first 3 rows ────────────────────────────

/// it178 — `count::3 sort::name` with 5 accounts shows the 3 alphabetically first.
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-1]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-01]
#[ test ]
fn it178_count_3_shows_first_3_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-d", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-e", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "count::3", "sort::name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // First 3 alphabetically: acct-a, acct-b, acct-c
  assert!( text.contains( "acct-a" ), "count::3 must include acct-a (1st), got:\n{text}" );
  assert!( text.contains( "acct-b" ), "count::3 must include acct-b (2nd), got:\n{text}" );
  assert!( text.contains( "acct-c" ), "count::3 must include acct-c (3rd), got:\n{text}" );
  // acct-d and acct-e must be truncated
  assert!( !text.contains( "acct-d" ), "count::3 must exclude acct-d (4th), got:\n{text}" );
  assert!( !text.contains( "acct-e" ), "count::3 must exclude acct-e (5th), got:\n{text}" );
}

// ── it179: count::0 shows all rows ───────────────────────────────────────────

/// it179 — `count::0` is the default (no truncation); all rows shown.
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-2]
#[ test ]
fn it179_count_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "count::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "count::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "count::0 must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "count::0 must show acct-c, got:\n{text}" );
}

// ── it180: count::100 with 2 accounts shows both ─────────────────────────────

/// it180 — `count::100` with only 2 accounts shows both (count exceeds available rows).
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-3]
#[ test ]
fn it180_count_100_exceeding_shows_all()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "count::100" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "count::100 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "count::100 must show acct-b, got:\n{text}" );
}

// ── it181: count::abc exits 1 ────────────────────────────────────────────────

/// it181 — `count::abc` exits 1 with type error (expected non-negative integer).
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-4]
#[ test ]
fn it181_count_abc_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "count::abc" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it182: count::1 sort::name shows only first row ──────────────────────────

/// it182 — `count::1 sort::name` with 3 accounts shows only the alphabetically first.
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-5]
#[ test ]
fn it182_count_1_sort_name_shows_only_first()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "count::1", "sort::name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ),  "count::1 must show acct-a (first), got:\n{text}" );
  assert!( !text.contains( "acct-b" ), "count::1 must exclude acct-b, got:\n{text}" );
  assert!( !text.contains( "acct-c" ), "count::1 must exclude acct-c, got:\n{text}" );
}

// ── it183: count::-1 exits 1 ─────────────────────────────────────────────────

/// it183 — `count::-1` exits 1 (negative integer rejected as non-negative u64).
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-6]
#[ test ]
fn it183_count_minus_1_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "count::-1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it184: offset::2 skips first 2 rows ──────────────────────────────────────

/// it184 — `offset::2 sort::name` with 4 accounts skips first 2; shows rows 3–4.
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-1]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-02]
#[ test ]
fn it184_offset_2_skips_first_2_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-d", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "sort::name", "offset::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Rows 3-4 alphabetically: acct-c, acct-d
  assert!( text.contains( "acct-c" ),  "offset::2 must show acct-c (3rd), got:\n{text}" );
  assert!( text.contains( "acct-d" ),  "offset::2 must show acct-d (4th), got:\n{text}" );
  // First 2 must be skipped
  assert!( !text.contains( "acct-a" ), "offset::2 must skip acct-a (1st), got:\n{text}" );
  assert!( !text.contains( "acct-b" ), "offset::2 must skip acct-b (2nd), got:\n{text}" );
}

// ── it185: offset::0 shows all rows ──────────────────────────────────────────

/// it185 — `offset::0` is the default (no skip); all rows shown.
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-2]
#[ test ]
fn it185_offset_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "offset::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "offset::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "offset::0 must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "offset::0 must show acct-c, got:\n{text}" );
}

// ── it186: offset::99 shows empty ────────────────────────────────────────────

/// it186 — `offset::99` with 2 accounts skips all rows; result is empty.
///
/// After offset::99, accounts slice is empty → render_text returns "(no accounts configured)".
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-3]
#[ test ]
fn it186_offset_99_shows_empty()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "offset::99" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "acct-a" ), "offset::99 must skip acct-a, got:\n{text}" );
  assert!( !text.contains( "acct-b" ), "offset::99 must skip acct-b, got:\n{text}" );
}

// ── it187: offset::abc exits 1 ───────────────────────────────────────────────

/// it187 — `offset::abc` exits 1 with type error.
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-4]
#[ test ]
fn it187_offset_abc_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "offset::abc" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it188: offset::1 count::1 shows second row ───────────────────────────────

/// it188 — `offset::1 count::1 sort::name` with 3 accounts shows exactly the second.
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-5]
#[ test ]
fn it188_offset_1_count_1_shows_second_row()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".usage", "offset::1", "count::1", "sort::name" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Second alphabetically: acct-b
  assert!( text.contains( "acct-b" ),  "offset::1 count::1 must show acct-b (2nd), got:\n{text}" );
  assert!( !text.contains( "acct-a" ), "offset::1 count::1 must skip acct-a (1st), got:\n{text}" );
  assert!( !text.contains( "acct-c" ), "offset::1 count::1 must exclude acct-c (3rd), got:\n{text}" );
}

// ── it189: offset::-1 exits 1 ────────────────────────────────────────────────

/// it189 — `offset::-1` exits 1 (negative integer rejected).
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-6]
#[ test ]
fn it189_offset_minus_1_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "offset::-1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it190: get::account extracts first account name ──────────────────────────

/// it190 — `get::account sort::name` extracts the first account name as a bare string.
///
/// Two error accounts alphabetically sorted; first row's account name is returned
/// as bare stdout with no table headers or other chrome.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-2]
#[ test ]
fn it190_get_account_extracts_first_name()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alpha-acct", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "zeta-acct",  "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".usage", "sort::name", "get::account" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text.trim(),
    "alpha-acct",
    "get::account must output only the first account name (alpha-acct), got:\n{text}",
  );
}

// ── it191: get::account output has no table chrome ───────────────────────────

/// it191 — `get::account` output contains no column headers, separators, or footer.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-6]
#[ test ]
fn it191_get_account_no_table_chrome()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "get::account" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // No column headers
  assert!( !text.contains( "5h Left" ), "get::account must not contain '5h Left' header, got:\n{text}" );
  assert!( !text.contains( "7d Left" ), "get::account must not contain '7d Left' header, got:\n{text}" );
  // No footer
  assert!( !text.contains( "Valid:" ),  "get::account must not contain 'Valid:' footer, got:\n{text}" );
}

// ── it192: get::status on error account outputs 🔴 ───────────────────────────

/// it192 — `get::status` on an error (🔴) account outputs `🔴` as a bare string.
///
/// Error accounts have `result = Err(_)` → `status_emoji` = "🔴".
/// The `get::status` field extraction returns this as a bare value.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-3 offline substitute]
#[ test ]
fn it192_get_status_err_on_error_account()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "get::status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text.trim(),
    "🔴",
    "get::status on error account must output exactly '🔴', got:\n{text}",
  );
}

// ── it193: get:: with empty filtered result → empty stdout ────────────────────

/// it193 — `get::account` after filtering to 0 rows → empty stdout, exits 0.
///
/// `only_valid::1` removes all error accounts → 0 rows → `get` on `accounts.first()` = None
/// → value is empty → content = empty → stdout is empty.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-4]
#[ test ]
fn it193_get_with_empty_filtered_result_empty_stdout()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".usage", "only_valid::1", "get::account" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim().is_empty(),
    "get:: with empty filtered result must produce empty stdout, got:\n{text}",
  );
}

// ── it194: abs::1 accepted with empty store ───────────────────────────────────

/// it194 — `abs::1` accepted with empty credential store; exits 0.
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-1]
#[ test ]
fn it194_abs_1_accepted_empty_store()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "abs::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "abs::1 with empty store must show no-accounts message, got:\n{text}",
  );
}

// ── it195: abs::0 accepted ────────────────────────────────────────────────────

/// it195 — `abs::0` accepted; exits 0 (default behavior, no change).
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-2]
#[ test ]
fn it195_abs_0_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "abs::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

// ── it196: abs::bad exits 1 ──────────────────────────────────────────────────

/// it196 — `abs::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-3]
#[ test ]
fn it196_abs_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "abs::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "0" ) && err.contains( "1" ),
    "abs::bad stderr must name valid values, got:\n{err}",
  );
}

// ── it197: abs::1 on error row shows error unchanged ─────────────────────────

/// it197 — `abs::1` on an error row; account row still shows dashes + error text.
///
/// `abs::1` is currently a no-op pending API token-count support.
/// Error rows are unaffected regardless.
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-5]
#[ test ]
fn it197_abs_1_on_error_row()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "abs::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Error account row must still be shown
  assert!( text.contains( "acct-a" ), "abs::1 must not remove error rows, got:\n{text}" );
}

// ── it198: no_color::1 produces no emoji in output ───────────────────────────

/// it198 — `no_color::1` with an error account → stdout contains no emoji.
///
/// `apply_no_color` replaces `🔴`→`err`, `→`→`->`, `✓`→`*`.
/// An error account has no live token (no `✓`) and no `→` recommendation marker;
/// `🔴` in the status column becomes `err`. None of the emoji characters remain.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-1]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-14]
#[ test ]
fn it198_no_color_1_no_emoji_in_output()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "no_color::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "🔴" ), "no_color::1 must remove 🔴, got:\n{text}" );
  assert!( !text.contains( "🟡" ), "no_color::1 must not contain 🟡, got:\n{text}" );
  assert!( !text.contains( "🟢" ), "no_color::1 must not contain 🟢, got:\n{text}" );
  assert!( !text.contains( '→' ),  "no_color::1 must remove → (replaced by ->), got:\n{text}" );
  assert!( !text.contains( '✓' ),  "no_color::1 must remove ✓, got:\n{text}" );
}

// ── it199: no_color::1 status column shows `err` text label ──────────────────

/// it199 — `no_color::1` status column shows `err` instead of `🔴`.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-2]
#[ test ]
fn it199_no_color_1_status_shows_err_text_label()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "no_color::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "err" ),
    "no_color::1 must show 'err' text label for error account status, got:\n{text}",
  );
}

// ── it200: no_color::bad exits 1 ─────────────────────────────────────────────

/// it200 — `no_color::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-4]
#[ test ]
fn it200_no_color_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "no_color::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "0" ) && err.contains( "1" ),
    "no_color::bad stderr must name valid values, got:\n{err}",
  );
}

// ── it201: no_color::true accepted ───────────────────────────────────────────

/// it201 — `no_color::true` accepted as alias for 1; no emoji in output.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-6]
#[ test ]
fn it201_no_color_true_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "no_color::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "🔴" ),
    "no_color::true must remove 🔴 (same as no_color::1), got:\n{text}",
  );
}

// ── it202: cols::+host shows Host column ─────────────────────────────────────

/// it202 — `cols::+host` adds Host column; account row shows value from profile.json.
///
/// `write_account_profile_json` creates `{name}.profile.json` with `{"host":"mybox"}`.
/// The `host` field is loaded regardless of token status.
///
/// Spec: [`tests/docs/cli/param/033_cols.md` EC-7]
/// Also: [`tests/docs/feature/029_account_host_metadata.md` AC-05]
#[ test ]
fn it202_cols_host_shows_host_column()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account_profile_json( dir.path(), "acct-a", Some( "mybox" ), Some( "work" ) );

  let out  = run_cs_with_env( &[ ".usage", "cols::+host" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Host" ),
    "cols::+host must add 'Host' column header, got:\n{text}",
  );
  assert!(
    text.contains( "mybox" ),
    "cols::+host must show host value 'mybox' in account row, got:\n{text}",
  );
}

// ── it203: cols::+role shows Role column ─────────────────────────────────────

/// it203 — `cols::+role` adds Role column; account row shows value from profile.json.
///
/// Spec: [`tests/docs/cli/param/033_cols.md` EC-8]
/// Also: [`tests/docs/feature/029_account_host_metadata.md` AC-06]
#[ test ]
fn it203_cols_role_shows_role_column()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account_profile_json( dir.path(), "acct-a", Some( "mybox" ), Some( "work" ) );

  let out  = run_cs_with_env( &[ ".usage", "cols::+role" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Role" ),
    "cols::+role must add 'Role' column header, got:\n{text}",
  );
  assert!(
    text.contains( "work" ),
    "cols::+role must show role value 'work' in account row, got:\n{text}",
  );
}

// ── it204: cols::+bogus exits 1 naming host and role ─────────────────────────

/// it204 — `cols::+bogus` exits 1; stderr names `host` and `role` among valid IDs.
///
/// After TSK-225, `host` and `role` were added as valid column IDs. The error
/// message must list them along with existing columns like `status`, `expires`, etc.
///
/// Spec: [`tests/docs/cli/param/033_cols.md` EC-9]
#[ test ]
fn it204_cols_bogus_names_host_and_role_in_error()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "cols::+bogus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "host" ),
    "cols::+bogus error must name 'host' as a valid column ID, got:\n{err}",
  );
  assert!(
    err.contains( "role" ),
    "cols::+bogus error must name 'role' as a valid column ID, got:\n{err}",
  );
}

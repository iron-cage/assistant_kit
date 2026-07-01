# tests/usage/

Integration tests for `src/usage/` module — relocated from `src/usage/` to comply with
file-size limits (`files_structure.rulebook.md § Code Organization : Source File Size Limits`).
Tests access `pub(crate)` items through `claude_profile::usage::test_bridge` (requires
`testing` Cargo feature, which enables `claude_profile::usage::test_bridge` as `pub mod`).

| File | Responsibility |
|------|----------------|
| `api_tests_a.rs` | Tests for api.rs Part A: pre_switch_touch_ctx, early apply_model_override. |
| `api_tests_b.rs` | Tests for api.rs Part B: mre_bug244+, apply_post_switch_touch, PreSwitchOutcome. |
| `refresh_tests_a.rs` | Tests for refresh.rs Part A: apply_refresh T01–FT-03. |
| `refresh_tests_b.rs` | Tests for refresh.rs Part B: apply_refresh FT-04+. |
| `render_tests_a.rs` | Tests for render.rs Part A: render_text, render_tsv, render_json FT-01–FT-28. |
| `render_tests_b.rs` | Tests for render.rs Part B: render_text FT-29+ sessions table tests. |
| `format_tests.rs` | Tests for format.rs: shorten_error, status_emoji, quota_text_cells, etc. |
| `sort_next_tests.rs` | Tests for sort_next.rs: find_next_for_strategy, strategy_metric (Part A). |
| `sort_next_tests_b.rs` | Tests for sort_next.rs BUG-229+ (Part B). |
| `touch_tests.rs` | Tests for touch.rs: apply_touch (Part A). |
| `touch_tests_b.rs` | Tests for touch.rs: CC-B6+ (Part B). |
| `fetch_tests.rs` | Tests for fetch.rs: inject_synthetic_if_new, parse_u64_from_str, read_cached_quota (Part A). |
| `fetch_tests_b.rs` | Tests for fetch.rs: CC-7+ (Part B). |
| `approx_tests.rs` | Tests for approx.rs: approximate_utilization polynomial extrapolation. |
| `live_tests.rs` | Tests for live.rs: secs_to_hms_utc UTC formatting. |
| `mod_tests.rs` | Tests for mod.rs: render_text, render_json, SortStrategy, three-tier grouping. |
| `params_tests.rs` | Tests for params.rs: parse_usage_params parameter parsing. |
| `refresh_predicate_tests.rs` | Tests for refresh_predicate.rs: should_refresh decision logic. |
| `sort_tests.rs` | Tests for sort.rs: sort_indices, status_group_of, StatusGroup. |
| `subprocess_tests.rs` | Tests for subprocess.rs: resolve_model, resolve_effort, effort_pre_args. |

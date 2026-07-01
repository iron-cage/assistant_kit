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
| `sort_next_tests.rs` | Tests for sort_next.rs: find_next_for_strategy, strategy_metric. |
| `touch_tests.rs` | Tests for touch.rs: apply_touch. |
| `fetch_tests.rs` | Tests for fetch.rs: inject_synthetic_if_new, parse_u64_from_str, read_cached_quota. |

//! Integration test entry point for `src/usage/` internals.
//!
//! These tests access `pub(crate)` items through `claude_profile::usage::test_bridge`,
//! which is only available under the `testing` Cargo feature. Requires `--all-features`
//! or `--features testing` to compile.
//!
//! | Module | Domain |
//! |--------|--------|
//! | `api_tests_a` | pre_switch_touch_ctx, early apply_model_override |
//! | `api_tests_b` | apply_post_switch_touch, PreSwitchOutcome, BUG-244+ |
//! | `refresh_tests_a` | apply_refresh T01–FT-03 |
//! | `refresh_tests_b` | apply_refresh FT-04+ |
//! | `render_tests_a` | render_text, render_tsv, render_json FT-01–FT-28 |
//! | `render_tests_b` | render_text FT-29+, sessions table |
//! | `format_tests` | shorten_error, status_emoji, quota_text_cells, etc. |
//! | `sort_next_tests` | find_next_for_strategy, strategy_metric Part A |
//! | `sort_next_tests_b` | find_next_for_strategy BUG-229+ Part B |
//! | `touch_tests` | apply_touch Part A |
//! | `touch_tests_b` | apply_touch CC-B6+ Part B |
//! | `fetch_tests` | inject_synthetic_if_new, parse_u64_from_str, read_cached_quota Part A |
//! | `fetch_tests_b` | read_cached_quota CC-7+ Part B |
//! | `approx_tests` | approximate_utilization polynomial extrapolation |
//! | `live_tests` | secs_to_hms_utc UTC formatting |
//! | `mod_tests` | render_text/render_json, SortStrategy, three-tier grouping |
//! | `params_tests` | parse_usage_params parameter parsing |
//! | `refresh_predicate_tests` | should_refresh decision logic |
//! | `sort_tests` | sort_indices, status_group_of, StatusGroup |
//! | `subprocess_tests` | resolve_model, resolve_effort, effort_pre_args |

#[ path = "usage/api_tests_a.rs" ]
mod api_tests_a;

#[ path = "usage/api_tests_b.rs" ]
mod api_tests_b;

#[ path = "usage/refresh_tests_a.rs" ]
mod refresh_tests_a;

#[ path = "usage/refresh_tests_b.rs" ]
mod refresh_tests_b;

#[ path = "usage/render_tests_a.rs" ]
mod render_tests_a;

#[ path = "usage/render_tests_b.rs" ]
mod render_tests_b;

#[ path = "usage/format_tests.rs" ]
mod format_tests;

#[ path = "usage/sort_next_tests.rs" ]
mod sort_next_tests;

#[ path = "usage/sort_next_tests_b.rs" ]
mod sort_next_tests_b;

#[ path = "usage/touch_tests.rs" ]
mod touch_tests;

#[ path = "usage/touch_tests_b.rs" ]
mod touch_tests_b;

#[ path = "usage/fetch_tests.rs" ]
mod fetch_tests;

#[ path = "usage/fetch_tests_b.rs" ]
mod fetch_tests_b;

#[ path = "usage/approx_tests.rs" ]
mod approx_tests;

#[ path = "usage/live_tests.rs" ]
mod live_tests;

#[ path = "usage/mod_tests.rs" ]
mod mod_tests;

#[ path = "usage/params_tests.rs" ]
mod params_tests;

#[ path = "usage/refresh_predicate_tests.rs" ]
mod refresh_predicate_tests;

#[ path = "usage/sort_tests.rs" ]
mod sort_tests;

#[ path = "usage/subprocess_tests.rs" ]
mod subprocess_tests;

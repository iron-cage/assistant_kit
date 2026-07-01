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
//! | `sort_next_tests` | find_next_for_strategy, strategy_metric |
//! | `touch_tests` | apply_touch |
//! | `fetch_tests` | inject_synthetic_if_new, parse_u64_from_str, read_cached_quota |

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

#[ path = "usage/touch_tests.rs" ]
mod touch_tests;

#[ path = "usage/fetch_tests.rs" ]
mod fetch_tests;

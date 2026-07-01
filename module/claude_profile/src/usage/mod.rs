//! `.usage` command — quota fetch, render, and live-monitor for all saved accounts.
//!
//! Public surface: `usage_routine` (command handler), `PreSwitchOutcome`,
//! `validate_imodel_str`, `validate_effort_str`, `validate_set_model`,
//! `pre_switch_touch_ctx`, `apply_post_switch_touch`, `attempt_expired_token_refresh`.

mod types;
mod fetch;
mod fetch_cache;
mod format;
mod sort;
mod sort_next;
mod render;
mod render_sessions;
mod render_json;
mod render_tsv;
mod live;
mod subprocess;
mod refresh;
mod refresh_predicate;
mod touch;
mod params;
mod api_switch;
mod api_dispatch;
mod api;
pub( crate ) mod approx;

pub( crate ) use api::{
  validate_imodel_str, validate_effort_str, pre_switch_touch_ctx, apply_post_switch_touch,
  attempt_expired_token_refresh,
  PreSwitchOutcome,
};
pub( crate ) use types::{ validate_set_model, map_model_shorthand };
pub use api::usage_routine;

// ── Test support ──────────────────────────────────────────────────────────────

#[ cfg( feature = "testing" ) ]
pub mod test_support;

// ── Test visibility bridge ─────────────────────────────────────────────────────

#[ cfg( feature = "testing" ) ]
pub mod test_bridge;


// Tests live in tests/usage/mod_tests.rs (integration tests via test_bridge).

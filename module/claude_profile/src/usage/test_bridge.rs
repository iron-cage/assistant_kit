//! Test visibility bridge: re-exports `pub(crate)` items as `pub` under the `testing` feature
//! so that integration tests in `tests/usage/` can access them without widening
//! visibility in production builds. Requires `enabled` feature (`claude_quota` types).

#![ allow( missing_docs ) ]

// ── Types sub-module ──────────────────────────────────────────────────────
pub mod types
{
  pub use super::super::types::{
    AccountQuota, SortStrategy, PreferStrategy, ColsVisibility,
    SubprocessModel, SubprocessEffort,
    OPUS_OVERRIDE_THRESHOLD, H_EXHAUSTED_THRESHOLD, WEEKLY_EXHAUSTION_THRESHOLD,
  };
}

// ── Test support helpers (flat re-export) ─────────────────────────────────
pub use super::test_support::*;

// ── Approx ────────────────────────────────────────────────────────────────
pub use super::approx::approximate_utilization;

// ── Sort ──────────────────────────────────────────────────────────────────
pub use super::sort::{ sort_indices, status_group_of, StatusGroup };

// ── Sort next ─────────────────────────────────────────────────────────────
pub use super::sort_next::{ find_next_for_strategy, strategy_metric };

// ── Render ────────────────────────────────────────────────────────────────
pub use super::render::{ render_text, render_tsv, render_json };

// ── Refresh ───────────────────────────────────────────────────────────────
pub use super::refresh::{ apply_refresh, reason_label };

// ── Touch ─────────────────────────────────────────────────────────────────
pub use super::touch::{ apply_touch, touch_skip_reason };

// ── Format ────────────────────────────────────────────────────────────────
pub use super::format::{
  token_exp_label, compute_expires_cell, unix_to_date, renewal_secs,
  renews_label, next_event_raw, next_event_label, sub_label, shorten_error,
  five_hour_left, seven_day_left, relevant_quotas, prefer_weekly,
  recommended_model, quota_text_cells, status_emoji,
};

// ── API ───────────────────────────────────────────────────────────────────
pub use super::api::{ pre_switch_touch_ctx, apply_post_switch_touch, PreSwitchOutcome };
pub use super::api_switch::{ apply_model_override, TouchCtx, model_override_direction };

// ── Params ────────────────────────────────────────────────────────────────
pub use super::params::parse_usage_params;

// ── Refresh predicate ─────────────────────────────────────────────────────
pub use super::refresh_predicate::should_refresh;

// ── Live ──────────────────────────────────────────────────────────────────
pub use super::live::secs_to_hms_utc;

// ── Subprocess ────────────────────────────────────────────────────────────
pub use super::subprocess::{ resolve_model, resolve_effort, effort_pre_args };

// ── Fetch ─────────────────────────────────────────────────────────────────
pub use super::fetch::{ inject_synthetic_if_new, parse_u64_from_str, fetch_quota_for_list };
pub use super::fetch_cache::read_cached_quota;

// ── Types (UsageParams) ───────────────────────────────────────────────────
pub use super::types::UsageParams;

//! `.usage` command — quota fetch, render, and live-monitor for all saved accounts.
//!
//! Public surface: `usage_routine` (command handler), `PreSwitchOutcome`,
//! `validate_imodel_str`, `validate_effort_str`, `validate_set_model`,
//! `pre_switch_touch_ctx`, `apply_post_switch_touch`, `attempt_expired_token_refresh`.

mod types;
mod fetch;
mod format;
mod sort;
mod sort_next;
mod render;
mod live;
mod subprocess;
mod refresh;
mod refresh_predicate;
mod touch;
mod params;
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

/// Shared test helpers for the `usage` module and all submodules.
///
/// All helpers live here so submodule test blocks can do
/// `use crate::usage::test_support::*` without duplicating factory code.
#[ cfg( test ) ]
pub( crate ) mod test_support
{
  use super::types::AccountQuota;
  use super::format::unix_to_date;

  /// Token expiry far enough in the future that any expiry-aware logic treats it as valid.
  pub( crate ) const FAR_FUTURE_MS : u64 = u64::MAX / 2;

  /// Build an `AccountQuota` with a single `five_hour` period (no weekly data).
  pub( crate ) fn mk_aq_ok( utilization : f64 ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name          : "test@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( data ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build an `AccountQuota` in error state.
  pub( crate ) fn mk_aq_err() -> AccountQuota
  {
    AccountQuota
    {
      name          : "bad@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Err( "missing accessToken".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build an `AccountQuota` with both `five_hour` and `seven_day` periods.
  ///
  /// Used by SE-AND tests — `seven_day_sonnet` is absent.
  pub( crate ) fn mk_aq_ok_both( h5_util : f64, d7_util : f64 ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : h5_util, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : d7_util, resets_at : None } ),
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name          : "test@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( data ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build an `AccountQuota` with controlled `5h_left` and no weekly data.
  ///
  /// Pitfall: `seven_day=None` → `prefer_weekly=100.0` for all accounts (absent data treated as
  /// 0% utilization). Tests using this helper exercise the TIEBREAK path for weekly-sensitive
  /// strategies. To test `prefer_weekly` primary-key ordering with distinct weekly quotas,
  /// use `mk_aq_sort_weekly` instead.
  pub( crate ) fn mk_aq_sort( name : &str, five_hour_util : f64, expires_at_ms : u64 ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : five_hour_util, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name          : name.to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms,
      result        : Ok( data ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build an `AccountQuota` with controlled `5h_left` AND weekly quota data.
  ///
  /// Use when tests need to exercise `prefer_weekly` ordering (secondary key of `sort::renew`).
  /// `resets_at` is None for all periods.
  pub( crate ) fn mk_aq_sort_weekly(
    name                  : &str,
    five_hour_util        : f64,
    seven_day_util        : f64,
    seven_day_sonnet_util : f64,
  ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : five_hour_util, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : seven_day_util, resets_at : None } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage { utilization : seven_day_sonnet_util, resets_at : None } ),
    };
    AccountQuota
    {
      name          : name.to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( data ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build ISO-8601 reset string at `now_secs + offset_secs`.
  ///
  /// Used by `sort::renew` tests that need concrete `resets_at` values.
  pub( crate ) fn reset_iso_at( now_secs : u64, offset_secs : u64 ) -> String
  {
    let ts = now_secs + offset_secs;
    let ( y, mo, d ) = unix_to_date( ts );
    let sod = ts % 86400;
    let h   = sod / 3600;
    let mi  = ( sod % 3600 ) / 60;
    let s   = sod % 60;
    format!( "{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z" )
  }

  /// Build `AccountQuota` with `five_hour.resets_at` set to `now_secs + reset_offset_secs`.
  ///
  /// Sets only the 5h reset timestamp; `seven_day` is None. Use for tests that need a concrete
  /// 5h reset value. Pitfall: Do NOT use for `sort::renew` ordering tests — the Renew arm reads
  /// `seven_day.resets_at`. Use `mk_aq_with_7d_reset` for `sort::renew` ordering tests.
  pub( crate ) fn mk_aq_with_reset(
    name             : &str,
    five_hour_util   : f64,
    now_secs         : u64,
    reset_offset_secs : u64,
  ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage
      {
        utilization : five_hour_util,
        resets_at   : Some( reset_iso_at( now_secs, reset_offset_secs ) ),
      } ),
      seven_day        : None,
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name          : name.to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( data ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build `AccountQuota` with `seven_day.resets_at` set to `now_secs + reset_offset_secs`.
  ///
  /// Use for `sort::renew` tests. `seven_day.utilization` is 0.0 (100% left).
  /// Pitfall: Use `mk_aq_with_reset` if you need `five_hour.resets_at` instead.
  pub( crate ) fn mk_aq_with_7d_reset(
    name              : &str,
    five_hour_util    : f64,
    now_secs          : u64,
    reset_offset_secs : u64,
  ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : five_hour_util, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage
      {
        utilization : 0.0,
        resets_at   : Some( reset_iso_at( now_secs, reset_offset_secs ) ),
      } ),
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name          : name.to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( data ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build `AccountQuota` with `seven_day.utilization` and `seven_day.resets_at` both set.
  ///
  /// Use for `sort::renew` tests needing a weekly-exhausted account with a reset timer.
  /// `mk_aq_with_7d_reset` hardcodes `seven_day.util=0.0`; this helper lets you specify it.
  /// Pitfall: do NOT use `mk_aq_with_7d_reset` when testing weekly-exhaustion paths — its
  /// zero utilization makes every account appear fully available (`prefer_weekly=100.0`).
  pub( crate ) fn mk_aq_with_7d_reset_util(
    name              : &str,
    five_hour_util    : f64,
    seven_day_util    : f64,
    now_secs          : u64,
    reset_offset_secs : u64,
  ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : five_hour_util, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage
      {
        utilization : seven_day_util,
        resets_at   : Some( reset_iso_at( now_secs, reset_offset_secs ) ),
      } ),
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name          : name.to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( data ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build a named `AccountQuota` with both `five_hour` and `seven_day` quota.
  ///
  /// Used by three-tier grouping tests where account name matters.
  pub( crate ) fn mk_named_aq( name : &str, h5_util : f64, d7_util : f64 ) -> AccountQuota
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : h5_util, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : d7_util, resets_at : None } ),
      seven_day_sonnet : None,
    };
    AccountQuota
    {
      name          : name.to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( data ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build a named `AccountQuota` in error state.
  pub( crate ) fn mk_named_aq_err( name : &str ) -> AccountQuota
  {
    AccountQuota
    {
      name          : name.to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Err( "missing accessToken".to_string() ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build an `AccountQuota` with only `seven_day_sonnet` populated.
  ///
  /// Used by `resolve_model` tests in `subprocess.rs`.
  pub( crate ) fn mk_aq_with_sonnet_util( utilization : f64 ) -> AccountQuota
  {
    AccountQuota
    {
      name          : "test@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( claude_quota::OauthUsageData
      {
        five_hour        : None,
        seven_day        : None,
        seven_day_sonnet : Some( claude_quota::PeriodUsage { utilization, resets_at : None } ),
      } ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build an `AccountQuota` with all quota fields absent.
  ///
  /// Used by `resolve_model` fallback tests in `subprocess.rs`.
  pub( crate ) fn mk_aq_no_sonnet_data() -> AccountQuota
  {
    AccountQuota
    {
      name          : "test@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( claude_quota::OauthUsageData
      {
        five_hour        : None,
        seven_day        : None,
        seven_day_sonnet : None,
      } ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build an `AccountQuota` with `five_hour.resets_at` set to the given value.
  ///
  /// Used by `apply_touch` trigger tests to distinguish active (Some) from idle (None) 5h windows.
  pub( crate ) fn mk_aq_with_resets_at( resets_at : Option< &str > ) -> AccountQuota
  {
    AccountQuota
    {
      name          : "test@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( claude_quota::OauthUsageData
      {
        five_hour        : Some( claude_quota::PeriodUsage
        {
          utilization : 50.0,
          resets_at   : resets_at.map( str::to_string ),
        } ),
        seven_day        : None,
        seven_day_sonnet : None,
      } ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Build an `AccountQuota` with `son_idle=true`.
  ///
  /// Produces: `five_h_running=true`, `d7_running=true` (absent), `son_idle=true`.
  /// - `five_hour.resets_at=Some(...)` → `five_h_running=true`
  /// - `seven_day=None` (absent → `d7_running=true` per `map_or` semantics)
  /// - `seven_day_sonnet=Some({resets_at:None})` → `son_idle=true`
  ///
  /// Used by `resolve_model` `son_idle` gate tests (BUG-289/BUG-290 fix).
  pub( crate ) fn mk_aq_with_son_idle() -> AccountQuota
  {
    AccountQuota
    {
      name          : "test@example.com".to_string(),
      is_current    : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms : FAR_FUTURE_MS,
      result        : Ok( claude_quota::OauthUsageData
      {
        five_hour        : Some( claude_quota::PeriodUsage
        {
          utilization : 50.0,
          resets_at   : Some( "2026-06-14T10:00:00Z".to_string() ),
        } ),
        seven_day        : None,
        seven_day_sonnet : Some( claude_quota::PeriodUsage
        {
          utilization : 50.0,
          resets_at   : None,
        } ),
      } ),
      account       : None,
      host          : String::new(),
      role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
    }
  }

  /// Mutex serializing tests that redirect the process-global stderr fd via `gag`.
  ///
  /// `gag::BufferRedirect::stderr()` redirects fd 2 via `dup2`; concurrent captures from
  /// different test threads corrupt each other. Acquire this lock before every
  /// `gag::BufferRedirect::stderr()` call; the guard is dropped automatically when
  /// the test or block ends. Uses `unwrap_or_else(|e| e.into_inner())` to ignore
  /// mutex poison from a prior panicking test and prevent cascade failures.
  pub( crate ) static STDERR_LOCK : std::sync::Mutex< () > = std::sync::Mutex::new( () );
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use tempfile::TempDir;
  use super::render::{ render_text, render_json };
  use super::refresh::apply_refresh;
  use super::types::{ AccountQuota, SortStrategy, PreferStrategy, ColsVisibility, SubprocessModel, SubprocessEffort };
  use crate::usage::test_support::
  {
    FAR_FUTURE_MS,
    mk_aq_ok, mk_aq_err, mk_aq_sort, mk_aq_sort_weekly,
    mk_named_aq, mk_named_aq_err,
  };

  // ── status_emoji via render_text / render_json ────────────────────────────

  /// SE-1 — Err result → 🔴.
  #[ test ]
  fn test_status_emoji_red()
  {
    let aq = mk_aq_err();
    let output = render_text(
      &[ aq ], SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );
    assert!( output.contains( "🔴" ), "Err account must show 🔴. Got:\n{output}" );
  }

  /// SE-2 — Ok, `5h_left` = 90% (util=10.0) → 🟢.
  #[ test ]
  fn test_status_emoji_green()
  {
    let aq = mk_aq_ok( 10.0 );
    let output = render_text(
      &[ aq ], SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );
    assert!( output.contains( "🟢" ), "90% left must show 🟢. Got:\n{output}" );
  }

  /// SE-3 — Ok, `5h_left` = 3% (util=97.0) → 🟡.
  #[ test ]
  fn test_status_emoji_yellow()
  {
    let aq = mk_aq_ok( 97.0 );
    let output = render_text(
      &[ aq ], SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );
    assert!( output.contains( "🟡" ), "3% left must show 🟡. Got:\n{output}" );
  }

  /// SE-4 — Boundary: 15% exactly (util=85.0) → 🟡 (inclusive at 15% for 5h).
  /// SE-4b — Boundary: 15.1% (util=84.9) → 🟢.
  #[ test ]
  fn test_status_emoji_boundary()
  {
    let aq_15pct   = mk_aq_ok( 85.0 );
    let aq_15_1pct = mk_aq_ok( 84.9 );
    let out_15   = render_text(
      &[ aq_15pct ], SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );
    let out_15_1 = render_text(
      &[ aq_15_1pct ], SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );
    assert!( out_15.contains( "🟡" ),   "exactly 15% left must show 🟡. Got:\n{out_15}" );
    assert!( out_15_1.contains( "🟢" ), "15.1% left must show 🟢. Got:\n{out_15_1}" );
  }

  /// SE-5 — Synthetic current-session row (`is_current=true`) shows correct emoji.
  #[ test ]
  fn test_status_emoji_on_synthetic_row()
  {
    let mut aq = mk_aq_ok( 20.0 );
    aq.is_current = true;
    aq.name = "(current session)".to_string();
    let output = render_text(
      &[ aq ], SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );
    assert!( output.contains( "🟢" ), "80% left synthetic row must show 🟢. Got:\n{output}" );
  }

  /// SE-6 — JSON output must NOT contain emoji (AC-20 no JSON equivalent).
  #[ test ]
  fn test_status_emoji_absent_in_json()
  {
    let aq = mk_aq_ok( 50.0 );
    let json = render_json( &[ aq ] );
    assert!(
      !json.contains( "🔴" ) && !json.contains( "🟡" ) && !json.contains( "🟢" ),
      "JSON must not contain status emoji. Got:\n{json}",
    );
  }

  // ── render_text ───────────────────────────────────────────────────────────

  /// C19 — Empty accounts → "(no accounts configured)".
  #[ test ]
  fn test_render_text_empty()
  {
    let result = render_text(
      &[], SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );
    assert!( result.contains( "no accounts configured" ), "empty must say no accounts, got: {result}" );
  }

  // ── render_json ───────────────────────────────────────────────────────────

  /// C20 — Empty accounts → "[]".
  #[ test ]
  fn test_render_json_empty()
  {
    let result = render_json( &[] );
    assert_eq!( result.trim(), "[]" );
  }

  /// C21 — Err account → JSON contains "error" field.
  #[ test ]
  fn test_render_json_error_account()
  {
    let accounts = vec![
      AccountQuota
      {
        name : "fail@test.com".to_string(), is_current : false, is_active : false, is_occupied_elsewhere : false,
        expires_at_ms : 0, result : Err( "auth failed".to_string() ), account : None,
        host : String::new(), role : String::new(), renewal_at : None,
        cached : false, cache_age_secs : None, is_owned : true, owner : String::new(),
      },
    ];
    let result = render_json( &accounts );
    assert!( result.contains( "\"error\":" ), "Err account must have error field, got: {result}" );
    assert!( result.contains( "auth failed" ), "error message must be preserved, got: {result}" );
  }

  /// C22 — Account name with quotes is JSON-escaped.
  #[ test ]
  fn test_render_json_escapes_quotes_in_name()
  {
    let accounts = vec![
      AccountQuota
      {
        name : "test\"@evil.com".to_string(), is_current : false, is_active : false, is_occupied_elsewhere : false,
        expires_at_ms : 0, result : Err( "fail".to_string() ), account : None,
        host : String::new(), role : String::new(), renewal_at : None,
        cached : false, cache_age_secs : None, is_owned : true, owner : String::new(),
      },
    ];
    let result = render_json( &accounts );
    assert!(
      result.contains( r#"test\"@evil.com"# ),
      "quotes in name must be escaped, got: {result}",
    );
  }

  /// FT-08 — Mixed Ok and Err accounts both appear in `format::json` output.
  #[ test ]
  fn test_render_json_ft8_mixed_ok_and_err_both_present()
  {
    let store = TempDir::new().unwrap();
    let quota = claude_quota::OauthUsageData
    {
      five_hour        : None,
      seven_day        : None,
      seven_day_sonnet : None,
    };
    let mut accounts = vec![
      AccountQuota
      {
        name          : "ok@example.com".to_string(),
        is_current    : false,
        is_active             : false,
        is_occupied_elsewhere : false,
        expires_at_ms : FAR_FUTURE_MS,
        result        : Ok( quota ),
        account       : None,
        host          : String::new(),
        role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
      },
      AccountQuota
      {
        name          : "err@example.com".to_string(),
        is_current    : false,
        is_active             : false,
        is_occupied_elsewhere : false,
        expires_at_ms : 0,
        result        : Err( "HTTP transport error: HTTP 401".to_string() ),
        account       : None,
        host          : String::new(),
        role          : String::new(),
      renewal_at     : None,
      cached         : false,
      cache_age_secs : None,
      is_owned       : true,
      owner                : String::new(),
      },
    ];

    apply_refresh( &mut accounts, store.path(), None, false, SubprocessModel::Auto, SubprocessEffort::Auto, false );

    let json = render_json( &accounts );

    assert!( json.contains( "ok@example.com" ),  "Ok account must appear in JSON; got: {json}" );
    assert!( json.contains( "err@example.com" ), "Err account must appear in JSON; got: {json}" );
    assert!( json.contains( "\"error\":" ),                "Err account must have error field; got: {json}" );
    assert!( json.contains( "\"session_5h_left_pct\":" ),  "Ok account must have quota fields; got: {json}" );

    let trimmed = json.trim();
    assert!( trimmed.starts_with( '[' ), "JSON must start with '['; got: {json}" );
    assert!( trimmed.ends_with(   ']' ), "JSON must end with ']'; got: {json}" );
  }

  // ── SortStrategy / PreferStrategy enum parsing ───────────────────────────

  /// AC-09 — `SortStrategy::parse` rejects unknown values with descriptive error.
  #[ test ]
  fn test_sort_strategy_parse_invalid_rejected()
  {
    let err = SortStrategy::parse( "bogus" ).unwrap_err();
    assert!( err.contains( "bogus" ),  "error must name the bad value; got: {err}" );
    assert!( err.contains( "name" ),   "error must name valid values; got: {err}" );
    assert!( err.contains( "renew" ),  "error must name valid values; got: {err}" );
    assert!( err.contains( "renews" ), "error must name valid values; got: {err}" );
  }

  /// AC-10 — `PreferStrategy::parse` rejects unknown values with descriptive error.
  #[ test ]
  fn test_prefer_strategy_parse_invalid_rejected()
  {
    let err = PreferStrategy::parse( "bogus" ).unwrap_err();
    assert!( err.contains( "bogus" ),  "error must name the bad value; got: {err}" );
    assert!( err.contains( "any" ),    "error must name valid values; got: {err}" );
    assert!( err.contains( "opus" ),   "error must name valid values; got: {err}" );
    assert!( err.contains( "sonnet" ), "error must name valid values; got: {err}" );
  }

  // ── sort display order via render_text ────────────────────────────────────

  /// AC-13 — `render_json` output is NOT sorted by `sort::` strategy.
  #[ test ]
  fn test_json_unaffected_by_sort()
  {
    let accounts = vec![
      mk_aq_sort( "zzz@test.com", 30.0, FAR_FUTURE_MS ),
      mk_aq_sort( "aaa@test.com", 80.0, FAR_FUTURE_MS ),
    ];
    let json = render_json( &accounts );
    let zzz_pos = json.find( "zzz@test.com" ).unwrap_or( 0 );
    let aaa_pos = json.find( "aaa@test.com" ).unwrap_or( usize::MAX );
    assert!(
      zzz_pos < aaa_pos,
      "render_json must preserve input order; zzz first in input must appear first in JSON",
    );
  }

  /// AC-11 — `sort::`-driven single-strategy footer recommends the first eligible account.
  #[ test ]
  fn test_sort_recommendation_unaffected_by_sort_strategy()
  {
    let a = mk_aq_sort( "a@x.com", 20.0, FAR_FUTURE_MS );  // 80% 5h_left — alphabetically first
    let b = mk_aq_sort( "b@x.com", 75.0, FAR_FUTURE_MS );  // 25% 5h_left
    // cur@x.com: is_current=true — triggers 2-line `·`-delimited footer where line 2 names the recommendation.
    let mut cur = mk_aq_sort( "cur@x.com", 10.0, FAR_FUTURE_MS );
    cur.is_current = true;
    let accounts = vec![ a, b, cur ];

    // sort::name: a@x.com first alphabetically among non-current → recommended in footer line 2.
    let output = render_text(
      &accounts, SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );

    assert!( output.contains( "a@x.com" ), "output must contain a@x.com; got:\n{output}" );
    assert!(
      output.contains( "Next (name) ·" ),
      "footer must show 'Next (name) ·'; got:\n{output}",
    );
    assert!(
      output.contains( "Next (name) · a@x.com" ),
      "footer must recommend a@x.com (first alphabetically under sort::name); got:\n{output}",
    );
  }

  // ── Three-tier grouping ────────────────────────────────────────────────────

  /// TT-T07/T08 — three-tier grouping: 🟢 → 🟡 → 🔴 overrides sort order.
  #[ test ]
  fn test_three_tier_grouping_green_before_yellow_before_red()
  {
    let a = mk_named_aq(     "a@x.com", 97.0, 0.0  ); // 5h=3% → 🟡
    let b = mk_named_aq(     "b@x.com", 10.0, 10.0 ); // 5h=90%, 7d=90% → 🟢
    let c = mk_named_aq_err( "c@x.com"             ); // Err → 🔴
    let accounts = vec![ a, b, c ];
    let output = render_text(
      &accounts, SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );
    let pos_a = output.find( "a@x.com" ).expect( "a@x.com must appear in output" );
    let pos_b = output.find( "b@x.com" ).expect( "b@x.com must appear in output" );
    let pos_c = output.find( "c@x.com" ).expect( "c@x.com must appear in output" );
    assert!( pos_b < pos_a, "🟢(b) must appear before 🟡(a). Got:\n{output}" );
    assert!( pos_a < pos_c, "🟡(a) must appear before 🔴(c). Got:\n{output}" );
  }

  /// FT-16 of feature/009 — within 🟡 tier, session-exhausted appears before weekly-exhausted.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-16]
  ///       [`docs/feature/009_token_usage.md` AC-26]
  #[ test ]
  fn test_ft16_009_yellow_tier_session_before_weekly()
  {
    let a = mk_named_aq( "a@x.com", 10.0, 98.0 );  // 5h=90%, 7d=2% → weekly-exhausted
    let b = mk_named_aq( "b@x.com", 99.0, 30.0 );  // 5h=1%, 7d=70% → session-exhausted
    let c = mk_named_aq( "c@x.com", 97.0, 50.0 );  // 5h=3%, 7d=50% → session-exhausted
    let d = mk_named_aq( "d@x.com", 10.0, 10.0 );  // 5h=90%, 7d=90% → 🟢
    let accounts = vec![ a, b, c, d ];

    let output = render_text(
      &accounts, SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );

    let pos_d = output.find( "d@x.com" ).expect( "d@x.com must appear" );
    let pos_b = output.find( "b@x.com" ).expect( "b@x.com must appear" );
    let pos_c = output.find( "c@x.com" ).expect( "c@x.com must appear" );
    let pos_a = output.find( "a@x.com" ).expect( "a@x.com must appear" );

    assert!( pos_d < pos_b, "🟢(d) must appear before session-yellow(b) (FT-16/009 AC-26);\n{output}" );
    assert!( pos_b < pos_a, "session-exhausted(b) must appear before weekly-exhausted(a) (FT-16/009 AC-26);\n{output}" );
    assert!( pos_c < pos_a, "session-exhausted(c) must appear before weekly-exhausted(a) (FT-16/009 AC-26);\n{output}" );
    assert!( pos_b < pos_c, "within session-yellow sub-group, alpha order must be preserved (FT-16/009 AC-26);\n{output}" );
  }

  /// FT-15 of feature/020 — `desc::1` reverses within each 🟡 sub-group but does NOT swap sub-group order.
  ///
  /// Spec: [`tests/docs/feature/020_usage_sort_strategies.md` FT-15]
  ///       [`docs/feature/020_usage_sort_strategies.md` AC-14]
  #[ test ]
  fn test_ft15_020_yellow_sub_grouping_not_reversed_by_desc()
  {
    let a = mk_named_aq( "a@x.com", 99.0, 30.0 );  // 5h=1%, 7d=70% → session-exhausted
    let b = mk_named_aq( "b@x.com", 97.0, 50.0 );  // 5h=3%, 7d=50% → session-exhausted
    let c = mk_named_aq( "c@x.com", 10.0, 10.0 );  // 5h=90%, 7d=90% → 🟢
    let z = mk_named_aq( "z@x.com", 10.0, 98.0 );  // 5h=90%, 7d=2% → weekly-exhausted

    let accounts = vec![ a, b, c, z ];

    let output = render_text(
      &accounts, SortStrategy::Name, Some( true ), PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );

    let pos_c = output.find( "c@x.com" ).expect( "c@x.com must appear" );
    let pos_b = output.find( "b@x.com" ).expect( "b@x.com must appear" );
    let pos_a = output.find( "a@x.com" ).expect( "a@x.com must appear" );
    let pos_z = output.find( "z@x.com" ).expect( "z@x.com must appear" );

    assert!( pos_b < pos_z, "session-exhausted(b) must appear before weekly-exhausted(z) even with desc::1 (FT-15/020 AC-14);\n{output}" );
    assert!( pos_a < pos_z, "session-exhausted(a) must appear before weekly-exhausted(z) even with desc::1 (FT-15/020 AC-14);\n{output}" );
    assert!( pos_c < pos_b, "🟢(c) must appear before session-yellow(b) (FT-15/020 AC-14);\n{output}" );
    assert!( pos_b < pos_a, "within session-yellow, desc::1 puts b before a (FT-15/020 AC-14);\n{output}" );
  }

  // ── Footer: no eligible candidate ─────────────────────────────────────────

  /// FT-08 of feature/020 — footer omits the `→ Next` recommendation line when no eligible
  /// candidate exists (all accounts are `is_current`).
  ///
  /// Spec: [`docs/feature/020_usage_sort_strategies.md` AC-09]
  #[ test ]
  fn test_ft08_020_footer_omits_recommendation_when_no_eligible_candidate()
  {
    let mut a = mk_aq_sort( "a@test.com", 30.0, FAR_FUTURE_MS );
    let mut b = mk_aq_sort( "b@test.com", 60.0, FAR_FUTURE_MS );
    a.is_current = true;
    b.is_current = true;
    let accounts = vec![ a, b ];

    let output = render_text(
      &accounts, SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );

    assert!(
      !output.contains( "→ Next (" ),
      "footer must omit recommendation line when no eligible candidate (FT-08/020), got:\n{output}",
    );
  }

  // ── Footer: model label ───────────────────────────────────────────────────

  /// FT-28 of feature/009 — footer shows session model (sonnet/opus) for the
  /// recommended account, based on `seven_day_sonnet` utilization threshold.
  ///
  /// Session model mirrors `apply_model_override()` in `api.rs`:
  ///   `sonnet_left` = 100.0 - utilization; if `sonnet_left` < 15.0 → opus, else → sonnet.
  ///
  /// Spec: [`tests/docs/feature/09_token_usage.md` FT-28]
  ///       [`docs/feature/009_token_usage.md` AC-10]
  #[ test ]
  fn test_ft28_009_footer_model_label()
  {
    // Scenario 1: recommended account has seven_day_sonnet.utilization = 50.0
    //             → sonnet_left = 50.0 ≥ 15.0 → session model = sonnet
    let mut current_1 = mk_aq_sort_weekly( "a@x.com", 10.0, 10.0, 50.0 );
    current_1.is_current = true;
    let sonnet_ok = mk_aq_sort_weekly( "b@x.com", 10.0, 10.0, 50.0 );
    // Default utilization from helper is 50.0 → sonnet_left = 50.0 ≥ 15.0
    // Footer line 2: `Next (name) · b@x.com · sonnet · {metric}`
    let output = render_text(
      &[ current_1, sonnet_ok ], SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );
    assert!(
      output.contains( "· sonnet" ),
      "FT-28 scenario 1: footer line 2 must show '· sonnet' when sonnet_left=50% ≥ 15%; got:\n{output}",
    );

    // Scenario 2: recommended account has seven_day_sonnet.utilization = 90.0
    //             → sonnet_left = 10.0 < 15.0 → session model = opus (override fires)
    let mut current_2 = mk_aq_sort_weekly( "a@x.com", 10.0, 10.0, 50.0 );
    current_2.is_current = true;
    let mut opus_override = mk_aq_sort_weekly( "c@x.com", 10.0, 10.0, 50.0 );
    if let Ok( ref mut data ) = opus_override.result
    {
      if let Some( ref mut son ) = data.seven_day_sonnet
      {
        son.utilization = 90.0; // 10% left < 15% threshold → opus
      }
    }
    // Footer line 2: `Next (name) · c@x.com · opus · {metric}`
    let output = render_text(
      &[ current_2, opus_override ], SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None,
    );
    assert!(
      output.contains( "· opus" ),
      "FT-28 scenario 2: footer line 2 must show '· opus' when sonnet_left=10% < 15%; got:\n{output}",
    );
  }
}

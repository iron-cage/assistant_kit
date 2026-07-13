//! Shared test helpers for the `usage` module and all submodules.
//!
//! All helpers live here so submodule test blocks can do
//! `use crate::usage::test_support::*` without duplicating factory code.

#![ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate, missing_docs ) ]

use super::types::AccountQuota;
use super::format::unix_to_date;

/// Token expiry far enough in the future that any expiry-aware logic treats it as valid.
pub const FAR_FUTURE_MS : u64 = u64::MAX / 2;

/// Build an `AccountQuota` with a single `five_hour` period (no weekly data).
pub fn mk_aq_ok( utilization : f64 ) -> AccountQuota
{
  let data = claude_quota::OauthUsageData
  {
    five_hour        : Some( claude_quota::PeriodUsage { utilization, resets_at : None } ),
    seven_day        : None,
    seven_day_sonnet : None,
  };
  AccountQuota
  {
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build an `AccountQuota` in error state.
pub fn mk_aq_err() -> AccountQuota
{
  AccountQuota
  {
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build an `AccountQuota` with both `five_hour` and `seven_day` periods.
///
/// Used by SE-AND tests — `seven_day_sonnet` is absent.
pub fn mk_aq_ok_both( h5_util : f64, d7_util : f64 ) -> AccountQuota
{
  let data = claude_quota::OauthUsageData
  {
    five_hour        : Some( claude_quota::PeriodUsage { utilization : h5_util, resets_at : None } ),
    seven_day        : Some( claude_quota::PeriodUsage { utilization : d7_util, resets_at : None } ),
    seven_day_sonnet : None,
  };
  AccountQuota
  {
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build an `AccountQuota` with controlled `5h_left` and no weekly data.
///
/// Pitfall: `seven_day=None` → `prefer_weekly=100.0` for all accounts (absent data treated as
/// 0% utilization). Tests using this helper exercise the TIEBREAK path for weekly-sensitive
/// strategies. To test `prefer_weekly` primary-key ordering with distinct weekly quotas,
/// use `mk_aq_sort_weekly` instead.
pub fn mk_aq_sort( name : &str, five_hour_util : f64, expires_at_ms : u64 ) -> AccountQuota
{
  let data = claude_quota::OauthUsageData
  {
    five_hour        : Some( claude_quota::PeriodUsage { utilization : five_hour_util, resets_at : None } ),
    seven_day        : None,
    seven_day_sonnet : None,
  };
  AccountQuota
  {
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build an `AccountQuota` with controlled `5h_left` AND weekly quota data.
///
/// Use when tests need to exercise `prefer_weekly` ordering (secondary key of `sort::renew`).
/// `resets_at` is None for all periods.
pub fn mk_aq_sort_weekly(
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
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build ISO-8601 reset string at `now_secs + offset_secs`.
///
/// Used by `sort::renew` tests that need concrete `resets_at` values.
pub fn reset_iso_at( now_secs : u64, offset_secs : u64 ) -> String
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
pub fn mk_aq_with_reset(
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
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build `AccountQuota` with `seven_day.resets_at` set to `now_secs + reset_offset_secs`.
///
/// Use for `sort::renew` tests. `seven_day.utilization` is 0.0 (100% left).
/// Pitfall: Use `mk_aq_with_reset` if you need `five_hour.resets_at` instead.
pub fn mk_aq_with_7d_reset(
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
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build `AccountQuota` with `seven_day.utilization` and `seven_day.resets_at` both set.
///
/// Use for `sort::renew` tests needing a weekly-exhausted account with a reset timer.
/// `mk_aq_with_7d_reset` hardcodes `seven_day.util=0.0`; this helper lets you specify it.
/// Pitfall: do NOT use `mk_aq_with_7d_reset` when testing weekly-exhaustion paths — its
/// zero utilization makes every account appear fully available (`prefer_weekly=100.0`).
pub fn mk_aq_with_7d_reset_util(
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
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build a named `AccountQuota` with both `five_hour` and `seven_day` quota.
///
/// Used by three-tier grouping tests where account name matters.
pub fn mk_named_aq( name : &str, h5_util : f64, d7_util : f64 ) -> AccountQuota
{
  let data = claude_quota::OauthUsageData
  {
    five_hour        : Some( claude_quota::PeriodUsage { utilization : h5_util, resets_at : None } ),
    seven_day        : Some( claude_quota::PeriodUsage { utilization : d7_util, resets_at : None } ),
    seven_day_sonnet : None,
  };
  AccountQuota
  {
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build a named `AccountQuota` in error state.
pub fn mk_named_aq_err( name : &str ) -> AccountQuota
{
  AccountQuota
  {
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build an `AccountQuota` with only `seven_day_sonnet` populated.
///
/// Used by `resolve_model` tests in `subprocess.rs`.
pub fn mk_aq_with_sonnet_util( utilization : f64 ) -> AccountQuota
{
  AccountQuota
  {
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build an `AccountQuota` with all quota fields absent.
///
/// Used by `resolve_model` fallback tests in `subprocess.rs`.
pub fn mk_aq_no_sonnet_data() -> AccountQuota
{
  AccountQuota
  {
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build an `AccountQuota` with `five_hour.resets_at` set to the given value.
///
/// Used by `apply_touch` trigger tests to distinguish active (Some) from idle (None) 5h windows.
pub fn mk_aq_with_resets_at( resets_at : Option< &str > ) -> AccountQuota
{
  AccountQuota
  {
    fallback_reason : None,
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
    org_created_at       : None,
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
pub fn mk_aq_with_son_idle() -> AccountQuota
{
  AccountQuota
  {
    fallback_reason : None,
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
    org_created_at       : None,
  }
}

/// Build an `AccountQuota` with a cancelled subscription (`billing_type = "none"`).
///
/// Simulates a dead account: subscription cancelled, no renewal path. The name
/// and quota utilizations are configurable — a cancelled account may still have
/// good quota right before the JWT expires, which is the BUG-317 scenario
/// (`billing_type = "none"` was ignored by `status_group_of()`).
///
/// Pitfall: `account = None` is ambiguous (API fetch failed, not confirmed cancelled).
/// This helper always sets `account = Some({billing_type: "none"})` — the definitive
/// cancelled signal.
pub fn mk_aq_cancelled(
  name    : &str,
  h5_util : f64,
  d7_util : f64,
) -> AccountQuota
{
  let data = claude_quota::OauthUsageData
  {
    five_hour        : Some( claude_quota::PeriodUsage { utilization : h5_util, resets_at : None } ),
    seven_day        : Some( claude_quota::PeriodUsage { utilization : d7_util, resets_at : None } ),
    seven_day_sonnet : None,
  };
  AccountQuota
  {
    fallback_reason : None,
    name          : name.to_string(),
    is_current    : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms : FAR_FUTURE_MS,
    result        : Ok( data ),
    account       : Some( claude_quota::OauthAccountData
    {
      tagged_id       : String::new(),
      uuid            : String::new(),
      email_address   : String::new(),
      full_name       : String::new(),
      display_name    : String::new(),
      billing_type    : "none".to_string(),
      has_max         : false,
      capabilities    : vec![],
      rate_limit_tier : String::new(),
      org_created_at  : String::new(),
      memberships     : vec![],
    } ),
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
    cached        : false,
    cache_age_secs : None,
    is_owned      : true,
    owner                : String::new(),
    org_created_at       : None,
  }
}

/// Mutex serializing tests that redirect the process-global stderr fd via `gag`.
///
/// `gag::BufferRedirect::stderr()` redirects fd 2 via `dup2`; concurrent captures from
/// different test threads corrupt each other. Acquire this lock before every
/// `gag::BufferRedirect::stderr()` call; the guard is dropped automatically when
/// the test or block ends. Uses `unwrap_or_else(|e| e.into_inner())` to ignore
/// mutex poison from a prior panicking test and prevent cascade failures.
pub static STDERR_LOCK : std::sync::Mutex< () > = std::sync::Mutex::new( () );

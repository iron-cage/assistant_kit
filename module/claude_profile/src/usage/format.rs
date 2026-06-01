//! Display formatting helpers for the quota table.
//!
//! All functions here are pure data-to-string converters: no I/O, no side effects.
//! They are called by `render.rs`, `sort.rs`, `touch.rs`, and `fetch.rs`.

use crate::output::format_duration_secs;
use super::types::{ AccountQuota, PreferStrategy };

// ── Token expiry label ────────────────────────────────────────────────────────

/// Format token expiry as a human-readable label for trace output.
///
/// Returns `"expired(Xd Yh ago)"` or `"valid(Xd Yh left)"` using the same
/// duration format as `format_duration_secs`.
pub( crate ) fn token_exp_label( expires_at_ms : u64 ) -> String
{
  let now_ms = u64::try_from(
    std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .unwrap_or_default()
      .as_millis()
  ).unwrap_or( u64::MAX );
  if now_ms >= expires_at_ms
  {
    format!( "expired({} ago)", format_duration_secs( ( now_ms - expires_at_ms ) / 1000 ) )
  }
  else
  {
    format!( "valid({} left)", format_duration_secs( ( expires_at_ms - now_ms ) / 1000 ) )
  }
}

// ── Token expiry cell ─────────────────────────────────────────────────────────

/// Compute the `Expires` cell value for a given token expiry and current time.
///
/// Returns `"EXPIRED"` when `expires_at_ms / 1000 ≤ now_secs` (saturating), or
/// `"in Xh Ym"` when the token is still valid.
pub( crate ) fn compute_expires_cell( expires_at_ms : u64, now_secs : u64 ) -> String
{
  let remaining = ( expires_at_ms / 1000 ).saturating_sub( now_secs );
  if remaining == 0
  {
    "EXPIRED".to_string()
  }
  else
  {
    format!( "in {}", format_duration_secs( remaining ) )
  }
}

// ── Date helpers ──────────────────────────────────────────────────────────────

/// Convert a Unix timestamp (seconds) to a Gregorian `(year, month, day)` tuple.
///
/// Month is 1-based (1 = January). Day is 1-based (1 = first of month).
/// No external dependencies — hand-rolled Gregorian arithmetic.
pub( crate ) fn unix_to_date( unix_secs : u64 ) -> ( u64, u64, u64 )
{
  let is_leap     = |y : u64| ( y % 4 == 0 && y % 100 != 0 ) || y % 400 == 0;
  let mut days    = unix_secs / 86_400;
  let mut year    = 1970_u64;
  loop
  {
    let in_year = if is_leap( year ) { 366 } else { 365 };
    if days < in_year { break; }
    days -= in_year;
    year += 1;
  }
  let feb = if is_leap( year ) { 29 } else { 28 };
  let month_days : [ u64; 12 ] = [ 31, feb, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 ];
  let mut month = 0_u64;
  for d in &month_days
  {
    if days < *d { break; }
    days -= d;
    month += 1;
  }
  ( year, month + 1, days + 1 )
}

// ── ISO-8601 parsing helpers ──────────────────────────────────────────────────

/// Convert a `(year, month, day)` tuple to Unix seconds at midnight UTC.
///
/// Month is 1-based (1 = January). Day is 1-based. Assumes year ≥ 1970.
fn date_to_unix( year : u64, month : u64, day : u64 ) -> u64
{
  let is_leap  = |y : u64| ( y % 4 == 0 && y % 100 != 0 ) || y % 400 == 0;
  let mut days = 0_u64;
  for y in 1970..year { days += if is_leap( y ) { 366 } else { 365 }; }
  let feb        = if is_leap( year ) { 29 } else { 28 };
  let month_days : [ u64; 12 ] = [ 31, feb, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 ];
  for &month_day in month_days.iter().take( usize::try_from( month - 1 ).unwrap_or( 0 ) ) { days += month_day; }
  days += day - 1;
  days * 86_400
}

/// Parse an ISO-8601 UTC timestamp (`"YYYY-MM-DDTHH:MM:SSZ"`) to Unix seconds.
///
/// Returns `None` on parse failure or year before 1970.
fn parse_iso_secs( s : &str ) -> Option< u64 >
{
  if s.len() < 19 { return None; }
  let year  : u64 = s[ 0..4   ].parse().ok()?;
  let month : u64 = s[ 5..7   ].parse().ok()?;
  let day   : u64 = s[ 8..10  ].parse().ok()?;
  let hour  : u64 = s[ 11..13 ].parse().ok()?;
  let min   : u64 = s[ 14..16 ].parse().ok()?;
  let sec   : u64 = s[ 17..19 ].parse().ok()?;
  if year < 1970 || month == 0 || month > 12 || day == 0 || day > 31 { return None; }
  Some( date_to_unix( year, month, day ) + hour * 3_600 + min * 60 + sec )
}

// ── Renewal timing ─────────────────────────────────────────────────────────────

/// Compute seconds until the next billing renewal and whether the value is an estimate.
///
/// Priority:
/// 1. **Exact** (`renewal_at_opt` set): parse the ISO-8601 string; auto-advance monthly
///    (+ 2 592 000 s per step) until the timestamp is in the future; return `(secs, false)`.
/// 2. **Estimate** (`org_created_at_opt` set): derive the billing day-of-month from the
///    `org_created_at` string and find the next occurrence; return `(secs, true)`.
/// 3. **Absent** (both `None`) or parse failure: return `None`.
pub( crate ) fn renewal_secs(
  renewal_at_opt     : Option< &str >,
  org_created_at_opt : Option< &str >,
  now_secs           : u64,
) -> Option< ( u64, bool ) >
{
  if let Some( renewal_at ) = renewal_at_opt
  {
    let mut ts = parse_iso_secs( renewal_at )?;
    while ts < now_secs { ts = ts.saturating_add( 2_592_000 ); }
    return Some( ( ts.saturating_sub( now_secs ), false ) );
  }
  if let Some( org_created_at ) = org_created_at_opt
  {
    if org_created_at.len() < 10 { return None; }
    let billing_day : u64 = org_created_at[ 8..10 ].parse().ok()?;
    if billing_day == 0 || billing_day > 31 { return None; }
    let ( year, month, day ) = unix_to_date( now_secs );
    let ( renewal_year, renewal_month ) = if billing_day > day
    {
      ( year, month )
    }
    else if month == 12
    {
      ( year + 1, 1 )
    }
    else
    {
      ( year, month + 1 )
    };
    let renewal_ts = date_to_unix( renewal_year, renewal_month, billing_day );
    return Some( ( renewal_ts.saturating_sub( now_secs ), true ) );
  }
  None
}

/// Format the next billing renewal as a duration string.
///
/// - Both absent → `"?"`.
/// - Parse failure → `"—"` (em-dash).
/// - Exact (`_renewal_at` set, auto-advanced) → `"in Xh Ym"` (no `~`).
/// - Estimate (only `org_created_at`) → `"~in Xd"`.
pub( crate ) fn renews_label(
  renewal_at_opt     : Option< &str >,
  org_created_at_opt : Option< &str >,
  now_secs           : u64,
) -> String
{
  if renewal_at_opt.is_none() && org_created_at_opt.is_none()
  {
    return "?".to_string();
  }
  match renewal_secs( renewal_at_opt, org_created_at_opt, now_secs )
  {
    None                    => "\u{2014}".to_string(),
    Some( ( secs, false ) ) => format!( "in {}",  format_duration_secs( secs ) ),
    Some( ( secs, true  ) ) => format!( "~in {}", format_duration_secs( secs ) ),
  }
}

// ── Next event label ─────────────────────────────────────────────────────────

/// Return the winning next-event candidate as `(secs, prefix, is_estimate)`.
///
/// Candidates with `secs == 0` are excluded. Minimum-secs wins; ties by iteration order.
/// Prefixes: `"!tok"` (token expiry — most urgent, exact), `"+7d"` (7d reset), `"$ren"` (renewal).
/// Pass `token_expires_secs = None` when `expires_at_ms == 0` (expiry unknown or already expired).
///
/// Fix(BUG-227): token expiry re-added as a candidate so the column surfaces the most urgent event.
/// Root cause: TSK-228 removed it to simplify the 3-param signature; the "account becomes unusable"
///   scenario was not considered when token expires before any quota or billing event.
/// Pitfall: `consider()` already skips secs==0, so `Some(0)` (just-expired) is safe to pass.
pub( crate ) fn next_event_raw(
  seven_day_resets_secs : Option< u64 >,
  renewal_secs_opt      : Option< u64 >,
  renewal_is_estimate   : bool,
  token_expires_secs    : Option< u64 >,
) -> Option< ( u64, &'static str, bool ) >
{
  let consider = |current : Option< ( u64, &'static str, bool ) >,
                  secs    : u64,
                  prefix  : &'static str,
                  est     : bool|
    -> Option< ( u64, &'static str, bool ) >
  {
    if secs == 0 { return current; }
    match current
    {
      None                                   => Some( ( secs, prefix, est ) ),
      Some( ( best, _, _ ) ) if secs < best => Some( ( secs, prefix, est ) ),
      other                                  => other,
    }
  };
  let mut best = None;
  if let Some( s ) = token_expires_secs     { best = consider( best, s, "!tok", false               ); }
  if let Some( s ) = seven_day_resets_secs  { best = consider( best, s, "+7d",  false               ); }
  if let Some( s ) = renewal_secs_opt       { best = consider( best, s, "$ren", renewal_is_estimate ); }
  best
}

/// Format the soonest upcoming strategic event as a compact label for the `→ Next` column.
///
/// Candidates: `!tok` (token expiry — soonest/most urgent), `+7d` (7-day reset), `$ren` (renewal).
/// All absent / zero → `"—"`. Pass `token_expires_secs = None` when expiry is unknown.
pub( crate ) fn next_event_label(
  seven_day_resets_secs : Option< u64 >,
  renewal_secs_opt      : Option< u64 >,
  renewal_is_estimate   : bool,
  token_expires_secs    : Option< u64 >,
) -> String
{
  match next_event_raw(
    seven_day_resets_secs,
    renewal_secs_opt, renewal_is_estimate,
    token_expires_secs,
  )
  {
    None                             => "\u{2014}".to_string(),
    Some( ( secs, prefix, true  ) ) => format!( "{prefix} ~in {}", format_duration_secs( secs ) ),
    Some( ( secs, prefix, false ) ) => format!( "{prefix} in {}",  format_duration_secs( secs ) ),
  }
}

// ── Subscription label ────────────────────────────────────────────────────────

/// Map account billing state to a short subscription label for the `Sub` column.
///
/// - `None`                      → `"?"` (fetch failed — state unknown)
/// - `billing_type == "none"`    → `"—"` (no active subscription)
/// - `has_max`                   → `"max"` (Claude Max plan)
/// - `"stripe_subscription"` + `!has_max` → `"pro"` (paid but not Max)
/// - anything else               → `"?"`
pub( crate ) fn sub_label( account : Option< &claude_quota::OauthAccountData > ) -> &'static str
{
  let Some( a ) = account else { return "?"; };
  if a.billing_type == "none"                { return "\u{2014}"; }
  if a.has_max                               { return "max"; }
  if a.billing_type == "stripe_subscription" { return "pro"; }
  "?"
}

// ── Error shortener ───────────────────────────────────────────────────────────

// Fix(BUG-152)
// Root cause: shorten_error had no HTTP 401 branch; the else { reason } arm returned the
//   verbose "HTTP transport error: HTTP 401" string verbatim into the 7d Reset column,
//   violating AC-03 ("shortened error reason"). HTTP 401 was added to T05 as a
//   pass-through regression guard in task 150, inadvertently documenting the wrong behaviour.
//   task/claude_profile/bug/152_shorten_error_omits_401.md
// Pitfall: shorten_error is a manual allowlist — each new HTTP error code from
//   QuotaError::HttpTransport needs an explicit branch. The else arm is NOT a shortener;
//   it is a verbatim passthrough. test_shorten_error_no_raw_http_transport_passthrough
//   enforces this invariant for known codes (401, 403, 429).
/// Shorten verbose quota error strings for display in the final table column.
///
/// `QuotaError::HttpTransport` formats errors as `"HTTP transport error: HTTP NNN"`.
/// Handled codes: `429` → `"rate limited (429)"`; `401` → `"auth expired (401)"`;
/// `403` → `"auth forbidden (403)"` (permission error returned by the usage API).
/// `QuotaError::MissingHeader` (displays as `"rate-limit header missing: …"`) is
/// shortened to `"no header"`. All other strings pass through unchanged.
/// The caller is responsible for wrapping the result in parentheses.
pub( crate ) fn shorten_error( reason : &str ) -> &str
{
  if reason.starts_with( "HTTP transport error: HTTP 429" )
  {
    "rate limited (429)"
  }
  else if reason.starts_with( "HTTP transport error: HTTP 401" )
  {
    "auth expired (401)"
  }
  else if reason.starts_with( "HTTP transport error: HTTP 403" )
  {
    "auth forbidden (403)"
  }
  else if reason.starts_with( "rate-limit header missing:" )
  {
    "no header"
  }
  else
  {
    reason
  }
}

// ── Quota left helpers ────────────────────────────────────────────────────────

/// Return `5h Left` as a percentage for sorting purposes.
///
/// Returns `100.0 - five_hour.utilization` for `Ok` accounts, or `-1.0` for `Err`
/// accounts (treated as below-exhausted for drain/reset floor sinking).
pub( crate ) fn five_hour_left( aq : &AccountQuota ) -> f64
{
  if let Ok( data ) = &aq.result
  {
    100.0 - data.five_hour.as_ref().map_or( 0.0, |p| p.utilization )
  }
  else
  {
    -1.0
  }
}

/// Return `7d Left` as a percentage for the `apply_touch` skip guard.
///
/// Returns `100.0 - seven_day.utilization` for `Ok` accounts with `seven_day` data,
/// `100.0` for `Ok` accounts where `seven_day` is absent (absent data ≠ exhausted),
/// or `0.0` for `Err` accounts (treated as fully exhausted — no touch beneficial).
pub( crate ) fn seven_day_left( aq : &AccountQuota ) -> f64
{
  let Ok( ref data ) = aq.result else { return 0.0; };
  100.0 - data.seven_day.as_ref().map_or( 0.0, |p| p.utilization )
}

/// Return the weekly quota left (%) for a given `prefer` strategy.
///
/// - `Opus`   → `7d Left` only.
/// - `Sonnet` → `7d(Son)` only.
/// - `Any`    → `min(7d Left, 7d(Son))` — conservative: whichever cap is more constrained.
///
/// Absent period data is treated as `0.0` left. `Err` accounts return `0.0`.
pub( crate ) fn prefer_weekly( aq : &AccountQuota, prefer : PreferStrategy ) -> f64
{
  let Ok( data ) = &aq.result else { return 0.0; };
  let left_7d  = 100.0 - data.seven_day.as_ref().map_or( 0.0, |p| p.utilization );
  let left_son = 100.0 - data.seven_day_sonnet.as_ref().map_or( 0.0, |p| p.utilization );
  match prefer
  {
    PreferStrategy::Opus   => left_7d,
    PreferStrategy::Sonnet => left_son,
    PreferStrategy::Any    => left_7d.min( left_son ),
  }
}

// ── Cell renderers ────────────────────────────────────────────────────────────

/// Compute the 5 quota display cells for a successful OAuth usage fetch.
///
/// Returns `[5h_left, 5h_reset, 7d_left, 7d_son, 7d_reset]` as display strings.
/// `5h Left` and `7d Left` cells carry a `🟢`/`🟡` prefix (same threshold as `status_emoji`).
/// Absent periods render as em-dash; absent reset timestamps render as em-dash.
pub( crate ) fn quota_text_cells( data : &claude_quota::OauthUsageData, now_secs : u64 ) -> [ String; 5 ]
{
  let dash      = "\u{2014}".to_string();
  let pct_cell  = |util : Option< f64 >| -> String
  {
    util.map_or_else( || dash.clone(), |u| format!( "{:.0}%", 100.0 - u ) )
  };
  let pct_emoji = |util : Option< f64 >, threshold : f64| -> String
  {
    util.map_or_else( || dash.clone(), |u|
    {
      let left  = 100.0 - u;
      let emoji = if left > threshold { "🟢" } else { "🟡" };
      format!( "{emoji} {left:.0}%" )
    } )
  };
  let reset_cell = |iso : Option< &str >| -> String
  {
    iso.and_then( claude_quota::iso_to_unix_secs )
      .map_or_else( || dash.clone(), |t|
        format!( "in {}", format_duration_secs( t.saturating_sub( now_secs ) ) )
      )
  };
  [
    pct_emoji( data.five_hour.as_ref().map( |p| p.utilization ), 15.0 ),
    reset_cell( data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
    pct_emoji( data.seven_day.as_ref().map( |p| p.utilization ), 5.0 ),
    pct_cell(  data.seven_day_sonnet.as_ref().map( |p| p.utilization ) ),
    reset_cell( data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
  ]
}

/// Return the single-glyph quota status emoji for an account row.
///
/// - `"🔴"` — token is invalid or missing (`result` is `Err`).
/// - `"🟡"` — token valid, but `5h Left ≤ 15%` or `7d Left ≤ 5%`.
/// - `"🟢"` — token valid, `5h Left > 15%` AND `7d Left > 5%`.
///
/// Absent period data is treated as fully available (conservative, 0% utilised).
pub( crate ) fn status_emoji( result : &Result< claude_quota::OauthUsageData, String > ) -> &'static str
{
  match result
  {
    Err( _ ) => "🔴",
    Ok( data ) =>
    {
      let h5_left = 100.0 - data.five_hour.as_ref().map_or( 0.0, |p| p.utilization );
      let d7_left = 100.0 - data.seven_day.as_ref().map_or( 0.0, |p| p.utilization );
      if h5_left > 15.0 && d7_left > 5.0 { "🟢" } else { "🟡" }
    }
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use super::*;
  use crate::usage::test_support::mk_aq_ok_both;

  // ── shorten_error ──────────────────────────────────────────────────────────

  /// T04 — `shorten_error` maps HTTP 429 transport string to the compact label.
  ///
  /// # Root Cause
  /// `apply_refresh` had HTTP 429 in its retry guard condition. HTTP 429 is a
  /// rate-limit response, not an auth failure; the token is still valid. Task 142
  /// added the 429 code to the guard by mistake; task 143 removes it and adds a
  /// `shorten_error` branch so the table shows a compact label instead of the
  /// verbose transport string.
  ///
  /// # Why Not Caught
  /// No existing test covered this string — `shorten_error` only had a single
  /// branch for `"rate-limit header missing:"`.
  ///
  /// # Fix Applied
  /// Added `"HTTP transport error: HTTP 429"` → `"rate limited (429)"` branch to
  /// `shorten_error()` before the pass-through else.
  ///
  /// # Prevention
  /// This test acts as a regression guard: if the branch is removed, the function
  /// returns the verbose 40-character string and this assertion fails.
  ///
  /// # Pitfall
  /// The match is an exact prefix check — `starts_with` — so partial or differently
  /// formatted 429 strings would still pass through. Only
  /// `claude_quota::QuotaError::HttpTransport` formats as `"HTTP transport error: HTTP N"`.
  #[ doc = "bug_reproducer(issue-150)" ]
  #[ test ]
  fn test_shorten_error_429_returns_rate_limited()
  {
    assert_eq!(
      shorten_error( "HTTP transport error: HTTP 429" ),
      "rate limited (429)",
    );
  }

  /// T05 — `shorten_error` must return `"auth expired (401)"` for HTTP 401 transport strings.
  ///
  /// # Root Cause
  /// `shorten_error` is an explicit allowlist. When task 150 added the HTTP 429 branch, it
  /// also added an HTTP 401 case to T05 as a regression guard — but as a pass-through check,
  /// documenting the wrong (non-AC-03) behaviour: HTTP 401 was not shortened.
  /// AC-03 (`docs/feature/009_token_usage.md:116`) requires "a shortened error reason" in the
  /// final column for ALL error cases, not only 429.
  ///
  /// # Why Not Caught
  /// T05 was written to assert the pass-through (current) behaviour, not the AC-03 requirement.
  /// No test verified the AC-03 invariant holistically — that ALL HTTP transport codes are
  /// shortened before reaching the table column.
  ///
  /// # Fix Applied
  /// Added `else if reason.starts_with( "HTTP transport error: HTTP 401" ) { "auth expired (401)" }`
  /// branch in `shorten_error()` between the 429 branch and the `"rate-limit header missing:"`
  /// branch. Fix(BUG-152).
  ///
  /// # Prevention
  /// `test_shorten_error_no_raw_http_transport_passthrough` asserts that no `"HTTP transport
  /// error:"` string passes through `shorten_error` unchanged. This test will fail for any
  /// future unshortened HTTP code, catching the gap early.
  ///
  /// # Pitfall
  /// `shorten_error` is a manual allowlist — each new HTTP error code from
  /// `QuotaError::HttpTransport` needs an explicit branch. The `else { reason }` arm is NOT
  /// a shortener; it is a verbatim passthrough. A new auth-failure code (e.g., 403) that the
  /// quota API might return in the future would silently appear in full in the table.
  #[ doc = "bug_reproducer(issue-152)" ]
  #[ test ]
  fn test_shorten_error_mre_401_shortened()
  {
    assert_eq!(
      shorten_error( "HTTP transport error: HTTP 401" ),
      "auth expired (401)",
      "HTTP 401 transport string must be shortened per AC-03 (BUG-152)",
    );
  }

  /// T06 — `shorten_error` maps HTTP 403 transport string to compact label.
  ///
  /// HTTP 403 (Forbidden) is returned by the usage API as a permission error and is handled
  /// by `apply_refresh` as an auth-failure trigger. Without `refresh::1`, a 403 error would
  /// previously appear verbatim as "(HTTP transport error: HTTP 403)" in the table column,
  /// violating AC-03 ("shortened error reason"). This branch shortens it to "auth forbidden (403)".
  #[ test ]
  fn test_shorten_error_403_returns_auth_forbidden()
  {
    assert_eq!(
      shorten_error( "HTTP transport error: HTTP 403" ),
      "auth forbidden (403)",
      "HTTP 403 transport string must be shortened per AC-03",
    );
  }

  /// Invariant — `shorten_error` must never return a raw `"HTTP transport error:"` string
  /// for HTTP error codes that appear in the current shortening allowlist.
  ///
  /// When adding a new HTTP error code to `claude_quota` fetch paths AND to `shorten_error`,
  /// add it to `shortened_codes` here too.
  #[ test ]
  fn test_shorten_error_no_raw_http_transport_passthrough()
  {
    // All codes with explicit branches in shorten_error are listed here.
    let shortened_codes = &[
      "HTTP transport error: HTTP 401",  // Fix(BUG-152): "auth expired (401)"
      "HTTP transport error: HTTP 403",  // "auth forbidden (403)" — usage API permission error
      "HTTP transport error: HTTP 429",  // task 150: "rate limited (429)"
    ];
    for &e in shortened_codes
    {
      let shortened = shorten_error( e );
      assert!(
        !shortened.starts_with( "HTTP transport error:" ),
        "shorten_error must shorten {e:?}; got verbatim passthrough {shortened:?}",
      );
    }
  }

  /// C6 regression — existing `"rate-limit header missing:"` branch still works.
  #[ test ]
  fn test_shorten_error_no_header_preserved()
  {
    assert_eq!( shorten_error( "rate-limit header missing: X-RateLimit-Remaining" ), "no header" );
  }

  /// A5 — empty string passes through `shorten_error` unchanged.
  #[ test ]
  fn test_shorten_error_empty_passthrough()
  {
    assert_eq!( shorten_error( "" ), "" );
  }

  /// A6 — arbitrary non-matching string passes through `shorten_error` unchanged.
  #[ test ]
  fn test_shorten_error_arbitrary_passthrough()
  {
    assert_eq!( shorten_error( "network timeout" ), "network timeout" );
  }

  // ── compute_expires_cell ────────────────────────────────────────────────────

  /// C6 — Both zero: `expires_at_ms=0, now_secs=0` → "EXPIRED".
  #[ test ]
  fn test_compute_expires_cell_both_zero()
  {
    assert_eq!( compute_expires_cell( 0, 0 ), "EXPIRED" );
  }

  /// C7 — Sub-second truncation: `expires_at_ms=999` rounds down to 0 seconds → "EXPIRED".
  #[ test ]
  fn test_compute_expires_cell_subsecond_truncation()
  {
    assert_eq!( compute_expires_cell( 999, 0 ), "EXPIRED" );
  }

  /// C8 — Exactly 1 second remaining → "in ..." (not "EXPIRED").
  #[ test ]
  fn test_compute_expires_cell_one_second_remaining()
  {
    let result = compute_expires_cell( 1000, 0 );
    assert!( result.starts_with( "in " ), "1 second remaining must start with 'in ', got: {result}" );
  }

  /// C9 — Saturating subtraction: now exceeds expires → "EXPIRED", no underflow.
  #[ test ]
  fn test_compute_expires_cell_now_exceeds_expires()
  {
    assert_eq!( compute_expires_cell( 1000, 9999 ), "EXPIRED" );
  }

  // ── token_exp_label ────────────────────────────────────────────────────────

  /// EC-1 — epoch timestamp (ms=0) is always in the past → `expired(... ago)`.
  ///
  /// # Root Cause
  /// `token_exp_label` is a private helper used in the `[trace]` GET line.
  /// It branches on `now_ms >= expires_at_ms`. Epoch zero is always ≤ now,
  /// so the expired branch must fire for any realistic system clock.
  ///
  /// # Why Not Caught
  /// New function added in BUG-169 trace enhancement; no tests existed.
  ///
  /// # Fix Applied
  /// Added unit test with deterministic input (ms=0 is always past).
  ///
  /// # Prevention
  /// Cover both branches of `token_exp_label` with deterministic inputs that
  /// are guaranteed past (0) and guaranteed future (`u64::MAX`).
  ///
  /// # Pitfall
  /// `token_exp_label` calls `SystemTime::now()` internally — cannot be mocked.
  /// Use extreme boundary values (0 and `u64::MAX`) to guarantee branch coverage
  /// regardless of wall-clock time.
  #[ test ]
  fn tel_epoch_zero_is_expired()
  {
    let label = token_exp_label( 0 );
    assert!( label.starts_with( "expired(" ), "expected expired prefix; got: {label}" );
    assert!( label.ends_with( " ago)" ),      "expected ' ago)' suffix; got: {label}" );
  }

  /// EC-2 — far-future timestamp (`u64::MAX` ms) is always in the future → `valid(... left)`.
  ///
  /// # Root Cause
  /// See `tel_epoch_zero_is_expired` — covers the `valid` branch of `token_exp_label`.
  ///
  /// # Why Not Caught
  /// New function; no tests existed.
  ///
  /// # Fix Applied
  /// Added unit test with `u64::MAX` as the expiry — always future for any real clock.
  ///
  /// # Prevention
  /// Use `u64::MAX` to guarantee the `valid` branch fires without mocking `SystemTime`.
  ///
  /// # Pitfall
  /// `u64::MAX` milliseconds is ~584 million years from epoch — safe for all foreseeable use.
  #[ test ]
  fn tel_far_future_is_valid()
  {
    let label = token_exp_label( u64::MAX );
    assert!( label.starts_with( "valid(" ), "expected valid prefix; got: {label}" );
    assert!( label.ends_with( " left)" ),   "expected ' left)' suffix; got: {label}" );
  }

  // ── status_emoji AND logic (SE-AND-T01 to T04) ────────────────────────────

  /// SE-AND-T01: `5h_left`=50%, `7d_left`=50% → 🟢 (5h > 15% and 7d > 5%).
  #[ test ]
  fn test_status_emoji_and_both_ample_green()
  {
    let aq = mk_aq_ok_both( 50.0, 50.0 );
    assert_eq!( status_emoji( &aq.result ), "🟢", "5h > 15% and 7d > 5% → 🟢" );
  }

  /// SE-AND-T02: `5h_left`=50%, `7d_left`=3% (`d7_util`=97) → 🟡 (7d ≤ 5%).
  #[ test ]
  fn test_status_emoji_and_7d_low_yellow()
  {
    let aq = mk_aq_ok_both( 50.0, 97.0 );
    assert_eq!( status_emoji( &aq.result ), "🟡", "7d ≤ 5% despite 5h ample → 🟡" );
  }

  /// SE-AND-T03: `5h_left`=3% (`h5_util`=97), `7d_left`=50% → 🟡 (5h ≤ 15%).
  #[ test ]
  fn test_status_emoji_and_5h_low_yellow()
  {
    let aq = mk_aq_ok_both( 97.0, 50.0 );
    assert_eq!( status_emoji( &aq.result ), "🟡", "5h ≤ 15% despite 7d ample → 🟡" );
  }

  /// SE-AND-T04: `5h_left`=15%, `7d_left`=5% → 🟡 (5h at boundary, 7d at boundary).
  #[ test ]
  fn test_status_emoji_and_both_at_threshold_yellow()
  {
    let aq = mk_aq_ok_both( 85.0, 95.0 );
    assert_eq!( status_emoji( &aq.result ), "🟡", "5h=15% and 7d=5% → 🟡 (neither > threshold)" );
  }

  /// IT-43 — Exact boundary precision: each threshold tested independently.
  ///
  /// Composite AND: `5h_left > 15.0%` AND `7d_left > 5.0%` required for 🟢.
  ///
  /// - A: `h5_util=85.0` (`5h_left=15.0`%, at threshold) → 🟡; 7d is ample.
  /// - B: `h5_util=84.9` (`5h_left=15.1`%, just above) → 🟢; 7d is ample.
  /// - C: `d7_util=95.0` (`7d_left=5.0`%, at threshold) → 🟡; 5h is ample.
  ///
  /// Source: [`009_token_usage.md` AC-19](../../docs/feature/009_token_usage.md)
  #[ test ]
  fn it151_status_emoji_boundary_precision()
  {
    let aq_a = mk_aq_ok_both( 85.0, 50.0 );
    let aq_b = mk_aq_ok_both( 84.9, 50.0 );
    let aq_c = mk_aq_ok_both( 50.0, 95.0 );
    assert_eq!( status_emoji( &aq_a.result ), "🟡", "A: 5h=15.0% (at threshold) → 🟡" );
    assert_eq!( status_emoji( &aq_b.result ), "🟢", "B: 5h=15.1% (just above) → 🟢" );
    assert_eq!( status_emoji( &aq_c.result ), "🟡", "C: 7d=5.0% (at threshold) → 🟡" );
  }

  // ── status_emoji with absent period data ──────────────────────────────────

  /// SE-7 — `five_hour=None` treated as 100% left → 🟢 (conservative, 0% utilised).
  ///
  /// Doc comment: "Absent period data is treated as fully available (conservative, 0% utilised)."
  /// `five_hour`=None → `map_or`(0.0) → `h5_left`=100% > 15% → 🟢 (given `seven_day` also absent → 100%).
  #[ test ]
  fn test_status_emoji_five_hour_none_is_green()
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour : None, seven_day : None, seven_day_sonnet : None,
    };
    let result : Result< claude_quota::OauthUsageData, String > = Ok( data );
    assert_eq!(
      status_emoji( &result ), "🟢",
      "five_hour=None must yield 🟢 (conservative 100% left)",
    );
  }

  // ── quota_text_cells ──────────────────────────────────────────────────────

  /// QT-T05: `5h_left`=86% (util=14.0) → cells[0] = "🟢 86%".
  #[ test ]
  fn test_quota_text_cells_5h_emoji_green()
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 14.0, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    };
    let cells = quota_text_cells( &data, 0 );
    assert_eq!( cells[ 0 ], "🟢 86%", "86% 5h left → 🟢 86%" );
  }

  /// QT-T06: `5h_left`=3% (util=97.0) → cells[0] = "🟡 3%".
  #[ test ]
  fn test_quota_text_cells_5h_emoji_yellow()
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 97.0, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    };
    let cells = quota_text_cells( &data, 0 );
    assert_eq!( cells[ 0 ], "🟡 3%", "3% 5h left → 🟡 3%" );
  }

  /// QT-T07 — `five_hour=None` in `quota_text_cells` → cells[0] = "—" (em dash).
  ///
  /// `pct_emoji(None)` → `util.map_or_else(|| dash.clone(), ...)` → "—".
  /// The absence of period data is displayed as em dash, not as a percentage.
  /// This is semantically distinct from `status_emoji` which treats None as 100% left.
  #[ test ]
  fn test_quota_text_cells_five_hour_none_shows_dash()
  {
    let data = claude_quota::OauthUsageData
    {
      five_hour : None, seven_day : None, seven_day_sonnet : None,
    };
    let cells = quota_text_cells( &data, 0 );
    assert_eq!(
      cells[ 0 ], "\u{2014}",
      "five_hour=None must produce em-dash in cells[0], not a percentage",
    );
  }

  /// FT-11 of feature/009 — per-column emoji prefix in `5h Left` cell values.
  ///
  /// `quota_text_cells` must attach `🟢` prefix when `5h_left` > 15%, `🟡` when ≤ 15%.
  /// The boundary (exactly 15.0%) is inclusive for `🟡`.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-11]
  #[ test ]
  fn test_ft11_009_per_column_emoji_prefix_three_cases()
  {
    let mk_5h = |util : f64| -> claude_quota::OauthUsageData
    {
      claude_quota::OauthUsageData
      {
        five_hour        : Some( claude_quota::PeriodUsage { utilization : util, resets_at : None } ),
        seven_day        : None,
        seven_day_sonnet : None,
      }
    };

    // Pct A: util=10.0 → 90% left (> 15%) → 🟢
    let cells_a = quota_text_cells( &mk_5h( 10.0 ), 0 );
    assert_eq!( cells_a[ 0 ], "🟢 90%", "Pct A (90% left) must have 🟢 prefix (FT-11/009)" );

    // Pct B: util=97.0 → 3% left (≤ 15%) → 🟡
    let cells_b = quota_text_cells( &mk_5h( 97.0 ), 0 );
    assert_eq!( cells_b[ 0 ], "🟡 3%", "Pct B (3% left) must have 🟡 prefix (FT-11/009)" );

    // Pct C: util=85.0 → exactly 15% left (≤ 15%) → 🟡 (boundary inclusive)
    let cells_c = quota_text_cells( &mk_5h( 85.0 ), 0 );
    assert_eq!( cells_c[ 0 ], "🟡 15%", "Pct C (exactly 15% left) must have 🟡 prefix — boundary inclusive (FT-11/009)" );
  }

  // ── renews_label (Phase 3 RED gate — TSK-227) ─────────────────────────────

  /// FT-17 — `renews_label` exact: `_renewal_at` set, result has no `~` prefix.
  ///
  /// `renewal_at = "2030-01-01T03:47:00Z"` (unix `1_893_469_620`);
  /// `now_secs = 1_893_456_000` (2030-01-01T00:00:00Z) → delta = 13620s = 3h 47m.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-17]
  /// Source: [`009_token_usage.md` AC-27]
  #[ test ]
  fn rl_exact_from_renewal_at()
  {
    let now_secs = 1_893_456_000_u64;
    let result   = renews_label( Some( "2030-01-01T03:47:00Z" ), None, now_secs );
    assert_eq!(
      result, "in 3h 47m",
      "exact _renewal_at must produce 'in 3h 47m' (no ~ prefix), got: {result}",
    );
  }

  /// FT-17 variant — `renews_label` estimate: only `org_created_at` available.
  ///
  /// Billing day = 15; now = 2030-01-01 (day 1) → next billing Jan 15 = 14 days away.
  /// Result must start with `"~in "` (estimate prefix).
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-17]
  /// Source: [`009_token_usage.md` AC-27]
  #[ test ]
  fn rl_estimate_from_org_created_at()
  {
    let now_secs = 1_893_456_000_u64;
    let result   = renews_label( None, Some( "2025-01-15T00:00:00Z" ), now_secs );
    assert!(
      result.starts_with( "~in " ),
      "estimate must start with '~in ', got: {result}",
    );
    assert!(
      result.contains( 'd' ),
      "estimate must include days unit, got: {result}",
    );
  }

  /// FT-17 variant — `renews_label` auto-advance: past `_renewal_at` advanced monthly.
  ///
  /// `renewal_at = "2020-01-01T00:00:00Z"` (unix `1_577_836_800`);
  /// `now_secs = 1_893_456_000` (2030-01-01). After 122 × 30d increments the
  /// timestamp lands ~7 days ahead of now. Result must start `"in "` (no `~`).
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-17]
  /// Source: [`009_token_usage.md` AC-27]
  #[ test ]
  fn rl_auto_advance_past_renewal_at()
  {
    let now_secs = 1_893_456_000_u64;
    let result   = renews_label( Some( "2020-01-01T00:00:00Z" ), None, now_secs );
    assert!(
      result.starts_with( "in " ),
      "auto-advanced _renewal_at must start with 'in ' (no ~ prefix), got: {result}",
    );
    // Must be ≤ 30 days from now (one advance step).
    assert!(
      !result.contains( "31d" ) && !result.contains( "32d" ),
      "auto-advance must land within 30d of now, got: {result}",
    );
  }

  /// FT-17 variant — `renews_label` absent: both sources absent → `"?"`.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-17]
  /// Source: [`009_token_usage.md` AC-27]
  #[ test ]
  fn rl_absent_returns_question()
  {
    let result = renews_label( None, None, 1_893_456_000 );
    assert_eq!( result, "?", "both absent must return '?', got: {result}" );
  }

  // ── next_event_label (Phase 3 RED gate — TSK-227) ─────────────────────────

  /// BUG-227 MRE: `next_event_label` must select `!tok` when token expiry fires before 7d reset.
  ///
  /// # Root Cause
  /// TSK-228 removed token expiry as a candidate to simplify the function signature. The "account
  /// becomes unusable" scenario was not considered — if the token expires before any quota or billing
  /// event, the column showed a misleading quota event days away.
  ///
  /// # Why Not Caught
  /// `ne_tok_excluded_after_tsk228` was a deliberate regression guard FOR the exclusion — it asserted
  /// the wrong behavior as the expected behavior.
  ///
  /// # Fix Applied
  /// `token_expires_secs: Option<u64>` param added to `next_event_raw()` and `next_event_label()`;
  /// `"!tok"` candidate added before `"+7d"` and `"$ren"` in the consideration loop.
  ///
  /// # Prevention
  /// `→ Next` column's purpose is "most actionable upcoming event." Token expiry = account becomes
  /// unusable = maximally actionable. Any future removal requires explicit written rationale.
  ///
  /// # Pitfall
  /// Call sites must pass `token_expires_secs = None` when `expires_at_ms == 0` (expiry unknown),
  /// not `Some(0)`, to avoid misleading `!tok in 0s` output. (Both work due to `consider()` secs==0
  /// guard, but `None` is semantically cleaner.)
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
  #[ doc = "bug_reproducer(BUG-227)" ]
  #[ test ]
  fn mre_bug227_tok_soonest_when_before_7d_reset()
  {
    // Token expires in 5m, 7d reset in 2h — !tok must win.
    let result = next_event_label( Some( 7200 ), None, false, Some( 300 ) );
    assert_eq!( result, "!tok in 5m", "token expiry soonest → '!tok in 5m', got: {result}" );
  }

  /// BUG-227: `!tok` defers when 7d reset is soonest.
  #[ doc = "bug_reproducer(BUG-227)" ]
  #[ test ]
  fn mre_bug227_7d_wins_when_tok_later()
  {
    // 7d reset in 1h, token expires in 6h — +7d must win.
    let result = next_event_label( Some( 3600 ), None, false, Some( 21600 ) );
    assert_eq!( result, "+7d in 1h", "7d reset soonest → '+7d in 1h', got: {result}" );
  }

  /// FT-18 variant — `next_event_label`: 7d reset soonest (no token expiry) → `"+7d in 2d"`.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
  /// Source: [`009_token_usage.md` AC-28]
  #[ test ]
  fn ne_7d_soonest()
  {
    let result = next_event_label( Some( 2 * 86400 ), None, false, None );
    assert_eq!( result, "+7d in 2d", "7d reset soonest → '+7d in 2d', got: {result}" );
  }

  /// FT-18 variant — `next_event_label`: exact billing renewal soonest → `"$ren in 6h"`.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
  /// Source: [`009_token_usage.md` AC-28]
  #[ test ]
  fn ne_renewal_soonest_exact()
  {
    let result = next_event_label( None, Some( 21600 ), false, None );
    assert_eq!( result, "$ren in 6h", "exact renewal soonest → '$ren in 6h', got: {result}" );
  }

  /// FT-18 variant — `next_event_label`: estimated billing renewal soonest → `"$ren ~in 3d"`.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
  /// Source: [`009_token_usage.md` AC-28]
  #[ test ]
  fn ne_renewal_soonest_estimate()
  {
    let result = next_event_label( None, Some( 3 * 86400 ), true, None );
    assert_eq!( result, "$ren ~in 3d", "estimate renewal soonest → '$ren ~in 3d', got: {result}" );
  }

  /// FT-18 variant — `next_event_label`: all sources absent or zero → em-dash.
  ///
  /// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
  /// Source: [`009_token_usage.md` AC-28]
  #[ test ]
  fn ne_all_none_returns_dash()
  {
    let result = next_event_label( None, None, false, None );
    assert_eq!( result, "\u{2014}", "all absent → em-dash, got: {result}" );
  }

}

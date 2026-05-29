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

// ── Billing label ─────────────────────────────────────────────────────────────

/// Format the estimated next billing renewal as `"Mon DD"` (e.g. `"Jun  5"`).
///
/// Billing day is taken from `org_created_at` (ISO-8601 `"YYYY-MM-DD..."`).
/// Returns em-dash if parsing fails or `org_created_at` is too short.
pub( crate ) fn next_billing_label( org_created_at : &str, now_secs : u64 ) -> String
{
  const MONTHS : [ &str; 12 ] = [ "Jan", "Feb", "Mar", "Apr", "May", "Jun",
                                   "Jul", "Aug", "Sep", "Oct", "Nov", "Dec" ];
  if org_created_at.len() < 10 { return "\u{2014}".to_string(); }
  let billing_day : u64 = match org_created_at[ 8..10 ].parse()
  {
    Ok( d ) => d,
    Err( _ ) => return "\u{2014}".to_string(),
  };
  if billing_day == 0 || billing_day > 31 { return "\u{2014}".to_string(); }
  let ( _, current_month, current_day ) = unix_to_date( now_secs );
  let renewal_month = if billing_day > current_day
  {
    current_month
  }
  else if current_month == 12
  {
    1
  }
  else
  {
    current_month + 1
  };
  #[ allow( clippy::cast_possible_truncation ) ] // renewal_month is always 1–12; cast to usize is safe
  let month_name = MONTHS[ ( renewal_month - 1 ) as usize ];
  format!( "{month_name} {billing_day:2}" )
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

}

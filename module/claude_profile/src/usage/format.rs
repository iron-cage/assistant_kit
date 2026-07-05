//! Display formatting helpers for the quota table.
//!
//! All functions here are pure data-to-string converters: no I/O, no side effects.
//! They are called by `render.rs`, `sort.rs`, `touch.rs`, and `fetch.rs`.
// Items are pub for test_bridge re-export; these lints are suppressed because all
// functions here are internal API exposed only via the feature-gated test_bridge module.
#![ allow( clippy::missing_inline_in_public_items, clippy::must_use_candidate, clippy::missing_errors_doc, clippy::missing_panics_doc ) ]

use crate::output::format_duration_secs;
use super::types::{ AccountQuota, PreferStrategy, OPUS_OVERRIDE_THRESHOLD, H_EXHAUSTED_THRESHOLD, WEEKLY_EXHAUSTION_THRESHOLD };

// ── Token expiry label ────────────────────────────────────────────────────────

/// Format token expiry as a human-readable label for trace output.
///
/// Returns `"expired(Xd Yh ago)"` or `"valid(Xd Yh left)"` using the same
/// duration format as `format_duration_secs`.
pub fn token_exp_label( expires_at_ms : u64 ) -> String
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
pub fn compute_expires_cell( expires_at_ms : u64, now_secs : u64 ) -> String
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
pub fn unix_to_date( unix_secs : u64 ) -> ( u64, u64, u64 )
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

/// Return the number of days in `(year, month)` — Gregorian, leap-aware.
fn days_in_month( year : u64, month : u64 ) -> u64
{
  let is_leap = ( year % 4 == 0 && year % 100 != 0 ) || year % 400 == 0;
  match month { 2 => if is_leap { 29 } else { 28 }, 4 | 6 | 9 | 11 => 30, _ => 31 }
}

// ── Renewal timing ─────────────────────────────────────────────────────────────

/// Compute seconds until the next billing renewal and whether the value is an estimate.
///
/// Priority:
/// 1. **Exact** (`renewal_at_opt` set): parse the ISO-8601 string; advance month-by-month,
///    clamping the day-of-month to each target month's length via `days_in_month()`, until
///    the timestamp is in the future; return `(secs, false)`.
/// 2. **Estimate** (`org_created_at_opt` set): derive the billing day-of-month from the
///    `org_created_at` string and find the next occurrence, clamped the same way;
///    return `(secs, true)`.
/// 3. **Absent** (both `None`) or parse failure: return `None`.
pub fn renewal_secs(
  renewal_at_opt     : Option< &str >,
  org_created_at_opt : Option< &str >,
  now_secs           : u64,
) -> Option< ( u64, bool ) >
{
  // Fix(BUG-329): day-of-month drift when advancing renewal_at/org_created_at across
  // months of different lengths (e.g. a day-31 anchor advancing into a 30-day month).
  // Root cause: date_to_unix() received the raw anchor day-of-month uncapped; whenever
  // the target month has fewer days than the anchor, the excess overflowed into the
  // following month instead of landing on that month's last day.
  // Pitfall: clamping must be applied independently in BOTH priority branches below —
  // Exact and Estimate each compute their own (year, month) and call date_to_unix()
  // separately, so clamping only one branch leaves the other still buggy.
  if let Some( renewal_at ) = renewal_at_opt
  {
    let mut ts = parse_iso_secs( renewal_at )?;
    let ( mut cur_year, mut cur_month, orig_day ) = unix_to_date( ts );
    while ts < now_secs
    {
      cur_month += 1;
      if cur_month > 12 { cur_month = 1; cur_year += 1; }
      ts = date_to_unix( cur_year, cur_month, orig_day.min( days_in_month( cur_year, cur_month ) ) );
    }
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
    let renewal_ts = date_to_unix( renewal_year, renewal_month, billing_day.min( days_in_month( renewal_year, renewal_month ) ) );
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
pub fn renews_label(
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
/// Prefixes: `"+7d"` (7d reset), `"$ren"` (renewal). Token expiry (`!tok`) is not a candidate —
/// it is already surfaced in the `Expires` column. 5h resets are not candidates either.
pub fn next_event_raw(
  seven_day_resets_secs : Option< u64 >,
  renewal_secs_opt      : Option< u64 >,
  renewal_is_estimate   : bool,
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
  if let Some( s ) = seven_day_resets_secs  { best = consider( best, s, "+7d",  false               ); }
  if let Some( s ) = renewal_secs_opt       { best = consider( best, s, "$ren", renewal_is_estimate ); }
  best
}

/// Format the soonest upcoming strategic event as a compact label for the `→ Next` column.
///
/// Candidates: `+7d` (7-day reset), `$ren` (renewal). All absent / zero → `"—"`.
pub fn next_event_label(
  seven_day_resets_secs : Option< u64 >,
  renewal_secs_opt      : Option< u64 >,
  renewal_is_estimate   : bool,
) -> String
{
  match next_event_raw( seven_day_resets_secs, renewal_secs_opt, renewal_is_estimate )
  {
    None                             => "\u{2014}".to_string(),
    Some( ( secs, prefix, true  ) ) => format!( "~in {} {prefix}", format_duration_secs( secs ) ),
    Some( ( secs, prefix, false ) ) => format!( "in {} {prefix}",  format_duration_secs( secs ) ),
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
pub fn sub_label( account : Option< &claude_quota::OauthAccountData > ) -> &'static str
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
pub fn shorten_error( reason : &str ) -> &str
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
pub fn five_hour_left( aq : &AccountQuota ) -> f64
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
pub fn seven_day_left( aq : &AccountQuota ) -> f64
{
  let Ok( ref data ) = aq.result else { return 0.0; };
  100.0 - data.seven_day.as_ref().map_or( 0.0, |p| p.utilization )
}

/// Return `(five_hour_left, relevant_7d_left)` for a given `prefer` strategy.
///
/// `five_hour_left` = `100.0 - five_hour.utilization` for `Ok` accounts; `-1.0` for `Err`.
///
/// `relevant_7d_left` is model-aware:
/// - `Opus`   → raw `seven_day_left` (Sonnet cap irrelevant for Opus intent).
/// - `Sonnet` → `100.0 - sonnet.utilization` when `Some`; **`0.0`** when `None` (absent = unknown).
/// - `Any`    → `min(seven_day_left, 100.0 - sonnet.utilization)` when `Some`; else `seven_day_left`.
/// - `Err(_)` result → `(-1.0, 0.0)`.
///
/// Fix(BUG Phase-2): old `prefer_weekly` used `map_or(0.0, ...)` for Sonnet utilization —
///   when `seven_day_sonnet = None`, `100.0 - 0.0 = 100.0`, silently inflating the quota
///   and making accounts with absent Sonnet tiers appear fully eligible under `prefer::son`.
/// Root cause: `map_or(0.0, ...)` is correct for DISPLAY (absent = show nothing / 0% label)
///   but wrong for eligibility gates — absent ≠ exhausted ≠ available.
/// Pitfall: always use `if let Some(ref son)` for quota-gate logic. `map_or` folds None into
///   a numeric sentinel that is indistinguishable from an actual measured value.
pub fn relevant_quotas( aq : &AccountQuota, prefer : PreferStrategy ) -> ( f64, f64 )
{
  let Ok( data ) = &aq.result else { return ( -1.0, 0.0 ); };
  let five_h_left = 100.0 - data.five_hour.as_ref().map_or( 0.0, |p| p.utilization );
  let left_7d     = 100.0 - data.seven_day.as_ref().map_or( 0.0, |p| p.utilization );
  let relevant_7d = match prefer
  {
    PreferStrategy::Opus   => left_7d,
    PreferStrategy::Sonnet =>
    {
      if let Some( ref son ) = data.seven_day_sonnet { 100.0 - son.utilization }
      else { 0.0 }
    }
    PreferStrategy::Any =>
    {
      if let Some( ref son ) = data.seven_day_sonnet { left_7d.min( 100.0 - son.utilization ) }
      else { left_7d }
    }
  };
  ( five_h_left, relevant_7d )
}

/// Return the weekly quota left (%) for a given `prefer` strategy.
///
/// - `Opus`   → `7d Left` only.
/// - `Sonnet` → `7d(Son)` only; **`0.0`** when `seven_day_sonnet` is absent (unknown ≠ 100%).
/// - `Any`    → `min(7d Left, 7d(Son))` when Sonnet present; `7d Left` when absent.
///
/// Absent period data is treated as `0.0` left. `Err` accounts return `0.0`.
/// Delegates to `relevant_quotas()` for the model-aware computation.
pub fn prefer_weekly( aq : &AccountQuota, prefer : PreferStrategy ) -> f64
{
  relevant_quotas( aq, prefer ).1
}

// ── Model recommendation ──────────────────────────────────────────────────────


/// Return the recommended session model for the next rotation candidate.
///
/// - `Ok(data)` with `seven_day_sonnet` present and `< OPUS_OVERRIDE_THRESHOLD` left → `"opus"`.
/// - `Ok(data)` with `seven_day_sonnet` absent (tier unknown) → `"sonnet"` (conservative).
/// - `Err(_)` → `"sonnet"` (quota unknown → conservative).
///
/// Mirrors the guard in `apply_model_override()`. Both reference `OPUS_OVERRIDE_THRESHOLD`
/// — the literal must not be duplicated.
pub fn recommended_model( aq : &AccountQuota ) -> &'static str
{
  match &aq.result
  {
    Ok( data ) => match &data.seven_day_sonnet
    {
      Some( s ) if 100.0 - s.utilization < OPUS_OVERRIDE_THRESHOLD => "opus",
      _ => "sonnet",
    },
    Err( _ ) => "sonnet",
  }
}

// ── Cell renderers ────────────────────────────────────────────────────────────

/// Compute the 5 quota display cells for a successful OAuth usage fetch.
///
/// Returns `[5h_left, 5h_reset, 7d_left, 7d_son, 7d_reset]` as display strings.
/// `5h Left` and `7d Left` cells carry a `🟢`/`🟡` prefix (same threshold as `status_emoji`).
/// Absent periods render as em-dash; absent reset timestamps render as em-dash.
pub fn quota_text_cells( data : &claude_quota::OauthUsageData, now_secs : u64 ) -> [ String; 5 ]
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
    pct_emoji( data.five_hour.as_ref().map( |p| p.utilization ), H_EXHAUSTED_THRESHOLD ),
    reset_cell( data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
    pct_emoji( data.seven_day.as_ref().map( |p| p.utilization ), WEEKLY_EXHAUSTION_THRESHOLD ),
    pct_cell(  data.seven_day_sonnet.as_ref().map( |p| p.utilization ) ),
    reset_cell( data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
  ]
}

/// Return the single-glyph quota status emoji for an account row.
///
/// - `"🔴"` — token is invalid or missing (`result` is `Err`), OR subscription is
///   cancelled (`billing_type="none"`).
/// - `"🟡"` — token valid, subscription active, but `5h Left ≤ 15%` or `7d Left ≤ 5%`.
/// - `"🟢"` — token valid, subscription active, `5h Left > 15%` AND `7d Left > 5%`.
///
/// Absent period data is treated as fully available (conservative, 0% utilised).
/// `account=None` (API fetch failed) is NOT classified 🔴 — absent data is ambiguous.
// Fix(BUG-317): billing_type="none" was not checked — cancelled accounts with good quota
//   appeared 🟢/🟡, misleading the user into thinking the account was temporarily exhausted
//   rather than permanently dead.
// Root cause: function only inspected result; billing_type lives in account which was ignored.
// Pitfall: account=None is ambiguous (API fetch failed, not confirmed cancelled) —
//   only fire the cancelled gate when account=Some(billing_type="none") is definitively present.
pub fn status_emoji( aq : &AccountQuota ) -> &'static str
{
  if aq.result.is_err() { return "🔴"; }
  // Fix(BUG-317): cancelled subscription → permanently unusable → 🔴 regardless of quota.
  // Root cause: status_emoji() only checked quota thresholds — billing_type="none" accounts
  //   with remaining quota appeared 🟢/🟡 even though they can never be used.
  // Pitfall: billing_type gate must fire BEFORE quota threshold checks in all classification
  //   functions; cancelled accounts are dead regardless of their quota readings.
  if aq.account.as_ref().is_some_and( |a| a.billing_type == "none" ) { return "🔴"; }
  let Ok( data ) = &aq.result else { unreachable!() };
  let h5_left = 100.0 - data.five_hour.as_ref().map_or( 0.0, |p| p.utilization );
  let d7_left = 100.0 - data.seven_day.as_ref().map_or( 0.0, |p| p.utilization );
  // Fix(BUG-321): both-exhausted (h5 ≤ 15% AND d7 ≤ 5%) → 🟡 (G3 weekly-exhausted), not 🔴.
  // BUG-319's fix used `(false,false)→🔴` as a proxy for "dead" — premise-incorrect.
  // Both quota dimensions depleted with result=Ok is recoverable (7d reset restores both).
  // Root cause: BUG-319 fix assumed the (false,false) arm mapped to "dead" — it does not;
  //   dead is gated exclusively by result.is_err() and billing_type="none".
  // Pitfall: 🔴 must only follow result.is_err() or billing_type="none" guards, never quota thresholds.
  // Fix(BUG-321): `status_emoji()` and `status_group_of()` now agree: both-exhausted = 🟡/G3.
  // Root cause: same as line 470 — BUG-319 fix premise was incorrect.
  // Pitfall: emoji and group classification must be kept in sync; divergence produces
  //   inconsistent table rows where 🔴 emoji appears in a 🟡 sort group.
  match ( h5_left > H_EXHAUSTED_THRESHOLD, d7_left > WEEKLY_EXHAUSTION_THRESHOLD )
  {
    ( true, true ) => "🟢",
    _              => "🟡",
  }
}

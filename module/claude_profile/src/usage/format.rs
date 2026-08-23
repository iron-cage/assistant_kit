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

/// Seconds remaining until token expiry (saturating at 0 when already past).
///
/// Shared by `compute_expires_cell` (text formatting) and `render_json`'s
/// `expires_in_secs` field, so both surfaces derive from the same arithmetic.
pub fn expires_remaining_secs( expires_at_ms : u64, now_secs : u64 ) -> u64
{
  ( expires_at_ms / 1000 ).saturating_sub( now_secs )
}

/// Compute the `Expires` cell value for a given token expiry and current time.
///
/// Returns `"EXPIRED"` when `expires_at_ms / 1000 ≤ now_secs` (saturating), or
/// `"in Xh Ym"` when the token is still valid.
pub fn compute_expires_cell( expires_at_ms : u64, now_secs : u64 ) -> String
{
  let remaining = expires_remaining_secs( expires_at_ms, now_secs );
  if remaining == 0
  {
    "EXPIRED".to_string()
  }
  else
  {
    format!( "in {}", format_duration_secs( remaining ) )
  }
}

/// Fix(BUG-345): `compute_expires_cell` alone cannot indicate cache-fallback staleness — it
///   takes only `expires_at_ms`/`now_secs`, with no way to know the reading came from a cache
///   fallback rather than a fresh live fetch this invocation.
/// Root cause: `AccountQuota.cached` (fetch provenance) and `expires_at_ms` (the raw value)
///   were never combined at any of the 3 formatted-string call sites (text table, `.get
///   field::expires`, TSV) — each showed the same string whether the reading was fresh or
///   stale-cached.
/// Pitfall: never call `compute_expires_cell` directly at a call site that has `aq.cached` in
///   scope — use this cache-aware wrapper instead, mirroring the `~`-prefix convention
///   `render.rs`'s `prefix_tilde()` already applies to the other quota cells.
///
/// Same as `compute_expires_cell`, prefixed with `~` when `cached` is `true`.
pub fn compute_expires_cell_cached( expires_at_ms : u64, now_secs : u64, cached : bool ) -> String
{
  let cell = compute_expires_cell( expires_at_ms, now_secs );
  if cached { format!( "~{cell}" ) } else { cell }
}

/// `Expires` cell from a full `AccountQuota` row — the preferred call form wherever an
/// `aq` is in scope (supersedes calling `compute_expires_cell_cached` directly there).
///
/// Feature 071: a redirect-backend row shows `static` — its key has no OAuth expiry on
/// `clp`'s clock (`expires_at_ms` is 0), and the raw arithmetic would render a healthy
/// static-key account as `EXPIRED`. Matches `.credentials.status`'s `Token: static`
/// vocabulary. All other rows delegate to `compute_expires_cell_cached` unchanged.
pub fn expires_cell_for( aq : &AccountQuota, now_secs : u64 ) -> String
{
  if aq.is_redirect_backend() { return "static".to_string(); }
  compute_expires_cell_cached( aq.expires_at_ms, now_secs, aq.cached )
}

/// `Sub` cell from a full `AccountQuota` row — the preferred call form wherever an
/// `aq` is in scope (the `expires_cell_for` pattern from BUG-345).
///
/// Fix(BUG-540): redirect rows showed `?` on every Sub surface (text `cols::+sub`,
///   TSV, `get::sub`) — their `account: None` fell through to `sub_label`'s unknown
///   fallback.
/// Root cause: a redirect account has no Anthropic subscription BY DESIGN — the same
///   known-absence fact BUG-538 taught the `~Renews` cell — but no `sub_label` call
///   site carried the predicate, because each surface computed the cell independently.
/// Pitfall: do not push the redirect check into `sub_label` itself — its `None` input
///   also means "fetch failed, genuinely unknown" for anthropic rows, where `?` is the
///   truthful output.
pub fn sub_cell_for( aq : &AccountQuota ) -> String
{
  if aq.is_redirect_backend() { return "\u{2014}".to_string(); }
  sub_label( aq.account.as_ref() ).to_string()
}

/// `~Renews` cell from a full `AccountQuota` row — the preferred call form wherever an
/// `aq` is in scope (the `expires_cell_for` pattern from BUG-345).
///
/// Fix(BUG-232): `billing_type == "none"` → no active subscription → no renewal date.
/// Root cause: `renews_label` uses `org_created_at` unconditionally; it has no
///   billing-type parameter, so the check must run before the call.
/// Pitfall: `org_created_at` may be present even when the subscription is cancelled.
///
/// Fix(BUG-538/BUG-540): redirect rows rendered `?` — `renews_label`'s both-`None`
///   fallback; a redirect account has no Anthropic billing org BY DESIGN, so "no
///   renewal" is a known fact, not missing data.
/// Root cause (BUG-540): BUG-538's fix patched this predicate at 2 of its 3 duplicated
///   call sites (text + TSV tables) and missed `extract_get_field`'s Renews arm —
///   `get::renews` said `?` while the table cell said `—`. Consolidated here so the
///   next surface cannot miss a site.
/// Pitfall: any new renews accessor must call this helper, never `renews_label`
///   directly.
pub fn renews_cell_for( aq : &AccountQuota, now_secs : u64 ) -> String
{
  if aq.is_no_subscription() || aq.is_redirect_backend()
  {
    return "\u{2014}".to_string();
  }
  renews_label(
    aq.renewal_at.as_deref(),
    aq.org_created_at.as_deref(),
    now_secs,
  )
}

/// Append the 🔒 claim-lock marker to an account-name cell when the row is locked.
///
/// Feature 070 lock visibility: a `claim_lock: true` account must be visibly
/// highlighted in `.usage` output by default — the lock silently blocks rotation
/// (Gate 9) and `.account.use`, so an invisible lock surprises the user at switch
/// time. Suffix form composes with the cache-age/fallback-reason suffixes and keeps
/// name-column prefix matching intact for TSV consumers. `no_color::1` maps the
/// glyph to `(locked)` via `apply_no_color`.
#[ must_use ]
pub fn with_lock_marker( aq : &AccountQuota, name : String ) -> String
{
  if aq.claim_lock { format!( "{name} \u{1F512}" ) } else { name }
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
  // Fix(audit-iso-char-boundary): `.get(range)` instead of `s[range]` — indexing panics
  //   when a slice boundary lands inside a multi-byte UTF-8 character.
  // Root cause: the old `len() < 19` guard checked byte count, not boundary validity —
  //   a timestamp-like string with a multi-byte char (e.g. fullwidth digits) passed the
  //   guard and panicked at the first mid-char boundary.
  // Pitfall: every fixed-position field needs its own guarded slice; `.get` also covers
  //   the too-short case, so no separate length precheck is needed.
  let year  : u64 = s.get( 0..4   )?.parse().ok()?;
  let month : u64 = s.get( 5..7   )?.parse().ok()?;
  let day   : u64 = s.get( 8..10  )?.parse().ok()?;
  let hour  : u64 = s.get( 11..13 )?.parse().ok()?;
  let min   : u64 = s.get( 14..16 )?.parse().ok()?;
  let sec   : u64 = s.get( 17..19 )?.parse().ok()?;
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
    // .get: same char-boundary guard as parse_iso_secs — indexing would panic mid-char.
    let billing_day : u64 = org_created_at.get( 8..10 )?.parse().ok()?;
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

// ── Projected 5h window end ──────────────────────────────────────────────────

/// Project the end of the 5h session window a touch at `touch_secs` opened.
///
/// Anthropic snaps 5h windows to 10-minute boundaries: the window starts at the last
/// 10-minute mark at or before the session's first request and runs `WINDOW_5H_S` from
/// there. Flooring is what makes this exact rather than approximate — validated against
/// 19 live accounts, where `floor10(last_touch_at) + 5h` reproduced the endpoint's own
/// `resets_at` within a second for all 16 whose touch fell inside the grace window, while
/// the unfloored `last_touch_at + 5h` matched none.
// Fix(BUG-551): the 5h Reset cell was the one estimate-capable column with no estimator,
//   so a row whose window the endpoint had not yet reported rendered the opaque literal
//   "(touched)" instead of a countdown.
// Root cause: no `start + 5h -> reset instant` helper existed anywhere in src/; the
//   sibling `~Renews` column's renewal_secs/renews_label pair already established the
//   derive-from-anchor-and-flag-with-`~` pattern this column never received.
// Pitfall: display-only — the projection must never be written back into
//   `five_hour.resets_at`, or sort, skip and forecast logic stop seeing the API's own state.
#[ must_use ]
pub fn projected_window_end_secs( touch_secs : u64 ) -> u64
{
  ( touch_secs / 600 ) * 600 + super::forecast::WINDOW_5H_S
}

/// Render a projected 5h window end as the `5h Reset` cell: `"~in Xh Ym"`.
///
/// The `~` marks the value as derived rather than endpoint-reported, matching the
/// convention `renews_label` established for the `~Renews` column.
#[ must_use ]
pub fn projected_reset_label( touch_secs : u64, now_secs : u64 ) -> String
{
  let end = projected_window_end_secs( touch_secs );
  format!( "~in {}", format_duration_secs( end.saturating_sub( now_secs ) ) )
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
//   QuotaError needs an explicit branch. The else arm is NOT a shortener;
//   it is a verbatim passthrough. test_shorten_error_no_raw_http_transport_passthrough
//   enforces this invariant for known codes (401, 403, 429).
// Fix(audit-stringly-http-status)
// Root cause: HTTP status failures rendered as "HTTP transport error: HTTP NNN"
//   (folded into QuotaError::HttpTransport free text); the typed
//   QuotaError::HttpStatus variant now renders the stable form "HTTP NNN".
// Pitfall: the old prefixed form survives on disk in persisted fallback_reason
//   strings (quota cache files), so both forms stay matched here — removing the
//   legacy branches would un-shorten historical cache entries.
/// Shorten verbose quota error strings for display in the final table column.
///
/// `QuotaError::HttpStatus` formats errors as `"HTTP NNN"`; the pre-typed form
/// `"HTTP transport error: HTTP NNN"` still occurs in persisted cache reasons.
/// Handled codes: `429` → `"rate limited (429)"`; `401` → `"auth expired (401)"`;
/// `403` → `"auth forbidden (403)"` (permission error returned by the usage API).
/// `QuotaError::MissingHeader` (displays as `"rate-limit header missing: …"`) is
/// shortened to `"no header"`. All other strings pass through unchanged.
/// The caller is responsible for wrapping the result in parentheses.
pub fn shorten_error( reason : &str ) -> &str
{
  // Accept both the typed form ("HTTP NNN", anchored at the start) and the
  // legacy transport-folded form persisted by older cache writes.
  let code_part = reason.strip_prefix( "HTTP transport error: " ).unwrap_or( reason );
  if code_part.starts_with( "HTTP 429" )
  {
    "rate limited (429)"
  }
  else if code_part.starts_with( "HTTP 401" )
  {
    "auth expired (401)"
  }
  else if code_part.starts_with( "HTTP 403" )
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

/// Return `true` when the error string `e` denotes the HTTP status `code`.
///
/// Accepted forms (the only forms quota error strings carry):
/// - the bare sentinel — the whole string is exactly the code (`"401"`), emitted by
///   `api_switch.rs` for synthetic placeholder rows;
/// - `"HTTP NNN"` at any position with a non-digit (or end-of-string) boundary after
///   the code — covers the typed `QuotaError::HttpStatus` Display form and the legacy
///   `"HTTP transport error: HTTP NNN"` form persisted in older cache reasons.
// Fix(audit-bare-status-substring): anchored matching replaces bare `e.contains( "401" )`.
// Root cause: a bare substring check false-matches any digit run containing the code —
//   "read 14290 bytes", "field '4013'", an epoch timestamp — misclassifying transport
//   errors as auth/rate-limit failures in should_refresh and the fetch cache-fallback guard.
// Pitfall: the boundary check is one-sided by design — "HTTP 1401" cannot contain
//   "HTTP 401" (the space anchors the left edge), so only the trailing digit needs testing.
#[ must_use ]
pub fn is_http_code( e : &str, code : u16 ) -> bool
{
  if e.parse::< u16 >().ok() == Some( code ) { return true; }
  let needle = format!( "HTTP {code}" );
  let mut rest = e;
  while let Some( pos ) = rest.find( &needle )
  {
    let after = &rest[ pos + needle.len().. ];
    if !after.starts_with( | c : char | c.is_ascii_digit() ) { return true; }
    rest = after;
  }
  false
}

// ── Quota left helpers ────────────────────────────────────────────────────────

/// Return `5h Left` as a percentage for sorting purposes.
///
/// Returns `100.0 - five_hour.utilization` for `Ok` accounts, or `-1.0` for `Err`
/// accounts (treated as below-exhausted for drain/reset floor sinking).
// Fix(BUG-336): raw `left` returned unrounded, so callers comparing it against threshold
//   constants could disagree with pct_emoji()'s BUG-331-rounded display for the same account.
// Root cause: pct_emoji() (BUG-331) rounds before comparing/displaying, but this helper's
//   raw return let every consumer's own threshold check run at full float precision.
// Pitfall: round exactly once here, at the source — never let a raw float feed a threshold
//   check while a sibling function classifies/displays a rounded value from the same measurement.
pub fn five_hour_left( aq : &AccountQuota ) -> f64
{
  if let Ok( data ) = &aq.result
  {
    ( 100.0 - data.five_hour.as_ref().map_or( 0.0, |p| p.utilization ) ).round()
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
// Fix(BUG-336): same raw-vs-rounded disagreement as five_hour_left() above — this helper's
//   unrounded return let status_group_of()/sort_next.rs threshold checks diverge from
//   pct_emoji()'s rounded display for the same account.
// Root cause: helper returned raw float; every caller's threshold comparison ran unrounded.
// Pitfall: round once here and reuse — do not let callers each apply their own rounding
//   (or none), since that reintroduces the same cross-function disagreement this fixes.
pub fn seven_day_left( aq : &AccountQuota ) -> f64
{
  let Ok( ref data ) = aq.result else { return 0.0; };
  ( 100.0 - data.seven_day.as_ref().map_or( 0.0, |p| p.utilization ) ).round()
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
/// Fix(BUG-489, shipped in feature-039 Phase-2, commit 5c5815c2): old `prefer_weekly`
///   used `map_or(0.0, ...)` for Sonnet utilization —
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
      // Fix(BUG-336): raw `100.0 - s.utilization` compared directly against
      //   OPUS_OVERRIDE_THRESHOLD, while apply_model_override() (BUG-331) already rounds
      //   before the identical comparison — could recommend the opposite model it would select.
      // Root cause: this function's doc comment asserts it "mirrors apply_model_override()"
      //   but only the latter received BUG-331's round-before-compare fix.
      // Pitfall: any future change to apply_model_override()'s comparison basis must be
      //   mirrored here too — the doc-asserted relationship is not compiler-enforced.
      Some( s ) if ( 100.0 - s.utilization ).round() < OPUS_OVERRIDE_THRESHOLD => "opus",
      _ => "sonnet",
    },
    Err( _ ) => "sonnet",
  }
}

// ── Cell renderers ────────────────────────────────────────────────────────────

/// Whether the `5h Left` / `7d Left` percentage cells carry their `🟢`/`🟡` prefix.
///
/// The text table shows the emoji; TSV and `get::<field>` want the bare number so a
/// consumer can parse it. This is the *only* legitimate difference between the surfaces'
/// quota cells — every other rule is shared (`quota_cells_for`).
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum PctStyle
{
  /// `🟢 88%` — the text table.
  Emoji,
  /// `88%` — TSV and `get::<field>`.
  Bare,
}

/// Compute the 5 quota display cells for a successful OAuth usage fetch.
///
/// Returns `[5h_left, 5h_reset, 7d_left, 7d_son, 7d_reset]` as display strings.
/// `5h Left` and `7d Left` cells carry a `🟢`/`🟡` prefix (same threshold as `status_emoji`).
/// Absent periods render as em-dash; absent reset timestamps render as em-dash.
///
/// Pitfall: this is the *data*-only layer — it cannot see the account, so it cannot apply
///   cache staleness or the touch projection. Never call it from a render surface that has
///   an `aq` in scope; call `quota_cells_for` there (BUG-553), exactly as `expires_cell_for`
///   supersedes `compute_expires_cell` at such call sites.
///
/// Fix(audit-quota-text-cells-dead-code): gated behind `testing`, matching its sole
///   re-export path (`usage::test_bridge`, itself `cfg( feature = "testing" )`).
/// Root cause: BUG-553 moved every render surface onto `quota_cells_for`, leaving this
///   wrapper with no production caller. Ungated, it tripped `dead_code` whenever
///   `claude_profile` was built as a dependency without `testing` — and the test gate's
///   `RUSTFLAGS="-D warnings"` promoted that warning to a hard error, breaking `assistant`
///   and `assistant_kit` while `claude_profile`'s own suite (which enables `testing`) passed.
/// Pitfall: `cargo check` shows this only as a warning; it surfaces as an error solely under
///   `-D warnings`, so a clean `cargo check` is not evidence the dependents build.
#[ cfg( feature = "testing" ) ]
pub fn quota_text_cells( data : &claude_quota::OauthUsageData, now_secs : u64 ) -> [ String; 5 ]
{
  quota_data_cells( data, now_secs, PctStyle::Emoji )
}

/// `quota_text_cells` with the percentage style selectable — the shared core behind both.
fn quota_data_cells
(
  data     : &claude_quota::OauthUsageData,
  now_secs : u64,
  style    : PctStyle,
) -> [ String; 5 ]
{
  let dash = "\u{2014}".to_string();
  // Fix(BUG-331): compared raw `left` against threshold but rounded only for display, so any
  //   account whose raw `left` landed within floating-point noise of a threshold could show
  //   identical rounded percentage text with a different color than another account on the
  //   opposite side of the same noise band.
  //   Root cause: `left` was computed once but consumed twice — raw for the comparison,
  //   rounded for the `{left:.0}%` display — letting the two diverge at sub-percent precision.
  //   Pitfall: always round once and reuse the rounded value for both the threshold comparison
  //   and the display text; never compare a raw float against a threshold when the display
  //   shows a rounded value derived from the same float.
  //
  // Fix(BUG-553 S4): `7d Son` used a separate unrounded closure here while `render_tsv`'s own
  //   copy rounded and `extract_get_field`'s did not — three roundings of one value, disagreeing
  //   by 1% at *.5 (bare `{:.0}` rounds half-to-even, `.round()` half-away).
  //   Root cause: each surface carried its own percentage closure, so BUG-331's round-once
  //   doctrine had to be re-applied per copy and was applied to only some of them.
  //   Pitfall: round once here, in the one shared closure — never in a format string, and
  //   never in a per-surface copy.
  let pct = |util : Option< f64 >, threshold : Option< f64 >| -> String
  {
    util.map_or_else( || dash.clone(), |u|
    {
      let left = ( 100.0 - u ).round();
      match ( style, threshold )
      {
        ( PctStyle::Emoji, Some( t ) ) =>
        {
          let emoji = if left > t { "🟢" } else { "🟡" };
          format!( "{emoji} {left:.0}%" )
        }
        _ => format!( "{left:.0}%" ),
      }
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
    pct( data.five_hour.as_ref().map( |p| p.utilization ), Some( H_EXHAUSTED_THRESHOLD ) ),
    reset_cell( data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
    pct( data.seven_day.as_ref().map( |p| p.utilization ), Some( WEEKLY_EXHAUSTION_THRESHOLD ) ),
    // `7d Son` never carries an emoji on any surface — no threshold, hence no prefix.
    pct( data.seven_day_sonnet.as_ref().map( |p| p.utilization ), None ),
    reset_cell( data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() ) ),
  ]
}

/// Prefix every non-em-dash cell with `~`, marking the whole row as cache-derived.
fn prefix_tilde( cells : &mut [ String ] )
{
  let dash = "\u{2014}";
  for cell in cells.iter_mut()
  {
    if *cell != dash
    {
      *cell = format!( "~{cell}" );
    }
  }
}

/// The 6 quota display cells for a successful fetch, **with every account-dependent display
/// rule applied** — the preferred call form wherever an `aq` is in scope (supersedes calling
/// `quota_text_cells` there).
///
/// Returns `[5h_left, 5h_reset, 7d_left, 7d_son, 7d_reset, 7d_son_reset]`. On top of
/// `quota_text_cells`'s data-only rendering it applies, in order:
///
/// 1. **Cache staleness** (`aq.cached`) — every non-dash cell gains a `~` prefix, and any
///    reset timestamp that has already elapsed becomes `(stale)` rather than a countdown
///    `saturating_sub` would clamp to a misleading `in 0s`.
/// 2. **Touch projection** (`aq.touched_at_secs`, BUG-551) — a corroborated-touch row whose
///    fetch still reports the 5h window idle shows the projected `~in Xh Ym` instead of the
///    em-dash the API's lagged state would produce.
///
/// Fix(BUG-553): three of the four render surfaces rebuilt these cells from local closures and
///   so silently missed both rules — TSV showed a cached row as live, `get::` disagreed with
///   the table cell it documents itself as equalling, and neither TSV nor JSON projected a
///   touched row.
/// Root cause: `quota_text_cells` takes only `data` + the clock, so any rule depending on the
///   *account* had to be bolted on per caller; only `render_text` did. Each new aq-dependent
///   rule multiplied the divergence by three.
/// Pitfall: add every future account-dependent quota rule here, never at a call site — a rule
///   applied after a shared helper returns exists on exactly the surfaces someone remembered.
pub fn quota_cells_for
(
  aq       : &AccountQuota,
  data     : &claude_quota::OauthUsageData,
  now_secs : u64,
  style    : PctStyle,
) -> [ String; 6 ]
{
  let dash       = "\u{2014}".to_string();
  let base       = quota_data_cells( data, now_secs, style );
  let son_reset  = data.seven_day_sonnet.as_ref().and_then( |p| p.resets_at.as_deref() );
  let mut cells  =
  [
    base[ 0 ].clone(), base[ 1 ].clone(), base[ 2 ].clone(), base[ 3 ].clone(), base[ 4 ].clone(),
    son_reset.and_then( claude_quota::iso_to_unix_secs )
      .map_or_else( || dash.clone(), |t|
        format!( "in {}", format_duration_secs( t.saturating_sub( now_secs ) ) )
      ),
  ];

  if aq.cached
  {
    prefix_tilde( &mut cells );
    // `saturating_sub` clamps an elapsed countdown to 0 in `quota_data_cells`, making "in 0s"
    // indistinguishable from an imminent future event. Re-check the raw timestamps here.
    let is_past = |iso : Option< &str >| -> bool
    {
      iso.and_then( claude_quota::iso_to_unix_secs ).is_some_and( |t| t < now_secs )
    };
    if is_past( data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ) ) { cells[ 1 ] = "(stale)".to_string(); }
    if is_past( data.seven_day.as_ref().and_then( |p| p.resets_at.as_deref() ) ) { cells[ 4 ] = "(stale)".to_string(); }
    if is_past( son_reset )                                                      { cells[ 5 ] = "(stale)".to_string(); }
  }

  // Fix(BUG-551): a corroborated-touch row whose re-fetch still reports the 5h window idle
  //   renders the projected countdown "~in Xh Ym" — replacing BUG-488's opaque "(touched)",
  //   which named the cause but withheld the value the column exists to show, on a row where
  //   that value is exactly derivable from the touch instant.
  // Root cause: `touched_recently` was a bool, so render had no instant to project from;
  //   `touched_at_secs` now carries the anchor `derive_touched_recently` parses.
  // Pitfall: display-only — never fabricate a `resets_at` into `data` itself; sort, skip and
  //   forecast logic must keep seeing the API's own (lagged) state.
  if let Some( touched_at ) = aq.touched_at_secs
  {
    if data.five_hour.as_ref().and_then( |p| p.resets_at.as_deref() ).is_none()
    {
      cells[ 1 ] = projected_reset_label( touched_at, now_secs );
    }
  }

  cells
}

/// Return the single-glyph quota status emoji for an account row.
///
/// - `"⚪"` — redirect-backend account (Feature 071): no Anthropic quota semantics at all;
///   neither healthy-🟢 nor error-🔴 applies. Sorts in the last group regardless (see
///   `status_group_of`) — a quota table orders anthropic candidates, and a redirect row
///   is never one.
/// - `"🔴"` — token is invalid or missing (`result` is `Err`), OR subscription is
///   cancelled (`billing_type="none"`).
/// - `"🟡"` — token valid, subscription active, but `5h Left ≤ 15%` or `7d Left ≤ 3%`.
/// - `"🟢"` — token valid, subscription active, `5h Left > 15%` AND `7d Left > 3%`.
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
  // Feature 071: checked before the generic Err guard — a redirect row's placeholder Err
  // is a backend fact, not a failure; 🔴 would misread a healthy static-key account as
  // broken (the exact confusion observed on a live kimi seat).
  if aq.is_redirect_backend() { return "⚪"; }
  if aq.result.is_err() { return "🔴"; }
  // Fix(BUG-317): cancelled subscription → permanently unusable → 🔴 regardless of quota.
  // Root cause: status_emoji() only checked quota thresholds — billing_type="none" accounts
  //   with remaining quota appeared 🟢/🟡 even though they can never be used.
  // Pitfall: billing_type gate must fire BEFORE quota threshold checks in all classification
  //   functions; cancelled accounts are dead regardless of their quota readings.
  if aq.account.as_ref().is_some_and( |a| a.billing_type == "none" ) { return "🔴"; }
  let Ok( data ) = &aq.result else { unreachable!() };
  // Fix(BUG-336): h5_left/d7_left compared raw against threshold constants, while
  //   pct_emoji() (BUG-331) already rounds the identical measurement before its own
  //   comparison/display — the Status dot and the 5h/7d Left cell could disagree for the
  //   same account in the same table row (e.g. raw left=5.4 rounds to 5, flipping the verdict).
  // Root cause: status_emoji() computed its own threshold inputs independently of
  //   pct_emoji(), and only pct_emoji() received BUG-331's round-before-compare fix.
  // Pitfall: round exactly once here and reuse the rounded value for the match below —
  //   never compare a raw float against a threshold that a sibling display function
  //   already rounds before comparing against the identical constant.
  let h5_left = ( 100.0 - data.five_hour.as_ref().map_or( 0.0, |p| p.utilization ) ).round();
  let d7_left = ( 100.0 - data.seven_day.as_ref().map_or( 0.0, |p| p.utilization ) ).round();
  // Fix(BUG-321): both-exhausted (h5 ≤ 15% AND d7 ≤ 3%) → 🟡 (G3 weekly-exhausted), not 🔴.
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

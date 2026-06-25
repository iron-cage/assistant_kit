// Path-referenced test module for format.rs — compiled as `mod tests` via `#[path]`.
// Lives in src/usage/ (not tests/) to access pub(crate) shorten_error, compute_expires_cell,
// token_exp_label, status_emoji, quota_text_cells, renews_label, and next_event_label
// without widening their visibility. See src/usage/readme.md § Inline Test Exception.

use super::*;
use crate::usage::test_support::{ FAR_FUTURE_MS, mk_aq_ok_both, mk_aq_sort, mk_aq_sort_weekly, mk_aq_err, mk_aq_cancelled };
use crate::usage::types::{ AccountQuota, PreferStrategy };

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
#[ doc = "bug_reproducer(BUG-271)" ]
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
#[ doc = "bug_reproducer(BUG-152)" ]
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
  assert_eq!( status_emoji( &aq ), "🟢", "5h > 15% and 7d > 5% → 🟢" );
}

/// SE-AND-T02: `5h_left`=50%, `7d_left`=3% (`d7_util`=97) → 🟡 (7d ≤ 5%).
#[ test ]
fn test_status_emoji_and_7d_low_yellow()
{
  let aq = mk_aq_ok_both( 50.0, 97.0 );
  assert_eq!( status_emoji( &aq ), "🟡", "7d ≤ 5% despite 5h ample → 🟡" );
}

/// SE-AND-T03: `5h_left`=3% (`h5_util`=97), `7d_left`=50% → 🟡 (5h ≤ 15%).
#[ test ]
fn test_status_emoji_and_5h_low_yellow()
{
  let aq = mk_aq_ok_both( 97.0, 50.0 );
  assert_eq!( status_emoji( &aq ), "🟡", "5h ≤ 15% despite 7d ample → 🟡" );
}

/// SE-AND-T04: `5h_left`=15%, `7d_left`=5% → 🔴 (both-exhausted → Red; neither threshold passes).
#[ test ]
fn test_status_emoji_and_both_at_threshold_red()
{
  let aq = mk_aq_ok_both( 85.0, 95.0 );
  assert_eq!( status_emoji( &aq ), "🔴", "5h=15% and 7d=5% → 🔴 (both-exhausted → Red; neither > threshold)" );
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
  assert_eq!( status_emoji( &aq_a ), "🟡", "A: 5h=15.0% (at threshold) → 🟡" );
  assert_eq!( status_emoji( &aq_b ), "🟢", "B: 5h=15.1% (just above) → 🟢" );
  assert_eq!( status_emoji( &aq_c ), "🟡", "C: 7d=5.0% (at threshold) → 🟡" );
}

// ── status_emoji with absent period data ──────────────────────────────────

/// SE-7 — `five_hour=None` treated as 100% left → 🟢 (conservative, 0% utilised).
///
/// Doc comment: "Absent period data is treated as fully available (conservative, 0% utilised)."
/// `five_hour`=None → `map_or`(0.0) → `h5_left`=100% > 15% → 🟢 (given `seven_day` also absent → 100%).
/// `account=None` — API fetch failed, not a confirmed cancelled account.
#[ test ]
fn test_status_emoji_five_hour_none_is_green()
{
  let data = claude_quota::OauthUsageData
  {
    five_hour : None, seven_day : None, seven_day_sonnet : None,
  };
  let aq = AccountQuota
  {
    name                  : String::new(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( data ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                 : String::new(),
  };
  assert_eq!(
    status_emoji( &aq ), "🟢",
    "five_hour=None must yield 🟢 (conservative 100% left)",
  );
}

/// MRE(BUG-317): cancelled account (`billing_type="none"`) must show 🔴 in the `●` column.
///
/// # Root Cause
/// `status_emoji` only checked `result.is_err()` and quota thresholds; it never inspected
/// `billing_type`. A cancelled account with good quota (e.g., 80% 5h, 80% 7d) returned 🟢,
/// contradicting the 🔴 classification in `status_group_of()` and misleading the user into
/// thinking the account was temporarily exhausted rather than permanently unusable.
///
/// # Why Not Caught
/// All existing `status_emoji` tests used `account=None` (no subscription data). The
/// `billing_type` field was never present in any format.rs test fixture.
///
/// # Fix Applied
/// `status_emoji` now accepts `&AccountQuota` and checks `billing_type="none"` → 🔴 after
/// the `result.is_err()` check, before quota threshold evaluation.
///
/// # Prevention
/// This test uses `mk_aq_cancelled` which sets `account=Some({billing_type: "none"})` so
/// the gate fires. All future `status_emoji` tests should use full `&AccountQuota`.
///
/// # Pitfall
/// `account=None` (API fetch failed) is ambiguous — do NOT penalize it. Only fire when
/// `account=Some(billing_type="none")` is definitively present.
#[ doc = "bug_reproducer(BUG-317)" ]
#[ test ]
fn mre_bug317_cancelled_status_emoji_is_red()
{
  let aq = mk_aq_cancelled( "dead@test.com", 20.0, 20.0 );
  assert_eq!(
    status_emoji( &aq ), "🔴",
    "BUG-317: cancelled account (billing_type='none') must show 🔴 in the ● column",
  );
}

/// BUG-319 MRE — both-exhausted (5h ≤ 15% AND 7d ≤ 5%) must show 🔴, not 🟡.
///
/// # Root Cause
/// `status_emoji()` used `if h5_left > 15.0 && d7_left > 5.0 { "🟢" } else { "🟡" }`.
/// The `else` branch captured all non-green states: h-exhausted (G2), weekly-exhausted (G3),
/// and both-exhausted (G4). G4 must be 🔴 — `status_group_of()` already returned Red for it
/// (sort order was correct), but the displayed emoji was wrong.
///
/// # Why Not Caught
/// SE-AND-T04 tested both-at-threshold and asserted 🟡 — it codified the bug as expected
/// behavior. No test covered both-deeply-exhausted with a distinct expected value.
///
/// # Fix Applied
/// Changed `status_emoji()` to a 3-arm match: `(true,true)→🟢`, `(false,false)→🔴`, `_→🟡`.
///
/// # Prevention
/// MRE values (5h=6%, 7d=4%) match the live i14@wbox.pro observation that triggered discovery.
/// Keep both thresholds well inside their exhaustion zones to clearly exercise the Red arm.
///
/// # Pitfall
/// `status_emoji()` and `status_group_of()` must agree: both-exhausted = Red/🔴. Any change
/// to group boundary thresholds must update both functions in lockstep.
#[ doc = "bug_reproducer(BUG-319)" ]
#[ test ]
fn mre_bug319_both_exhausted_status_emoji_is_red()
{
  // 5h_util=94% → 5h_left=6% (h-exhausted: ≤ 15%); 7d_util=96% → 7d_left=4% (weekly-exhausted: ≤ 5%).
  // Both below thresholds → both-exhausted → Group 4 (Red) → must be 🔴.
  let aq = mk_aq_ok_both( 94.0, 96.0 );
  assert_eq!(
    status_emoji( &aq ), "🔴",
    "BUG-319: both-exhausted (5h=6%, 7d=4%) must be 🔴 (Red/G4), not 🟡",
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

// ── next_event_label ───────────────────────────────────────────────────────

/// TSK-235 guard: `!tok` is not a candidate; `+7d` must win even when token expires sooner.
///
/// `→ Next` is strategic-only: `+7d` and `$ren` only. Token expiry is already shown in `Expires`.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
/// Source: [`009_token_usage.md` AC-28]
#[ test ]
fn ne_tok_excluded_after_tsk228()
{
  // Even if a token would expire in 5m, !tok is not a candidate — in 2h +7d must win.
  let result = next_event_label( Some( 7200 ), None, false );
  assert_eq!( result, "in 2h +7d", "!tok must not be a candidate; got: {result}" );
}

/// FT-18 variant — `next_event_label`: 7d reset soonest → `"in 2d +7d"`.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
/// Source: [`009_token_usage.md` AC-28]
#[ test ]
fn ne_7d_soonest()
{
  let result = next_event_label( Some( 2 * 86400 ), None, false );
  assert_eq!( result, "in 2d +7d", "7d reset soonest → 'in 2d +7d', got: {result}" );
}

/// FT-18 variant — `next_event_label`: exact billing renewal soonest → `"in 6h $ren"`.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
/// Source: [`009_token_usage.md` AC-28]
#[ test ]
fn ne_renewal_soonest_exact()
{
  let result = next_event_label( None, Some( 21600 ), false );
  assert_eq!( result, "in 6h $ren", "exact renewal soonest → 'in 6h $ren', got: {result}" );
}

/// FT-18 variant — `next_event_label`: estimated billing renewal soonest → `"~in 3d $ren"`.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
/// Source: [`009_token_usage.md` AC-28]
#[ test ]
fn ne_renewal_soonest_estimate()
{
  let result = next_event_label( None, Some( 3 * 86400 ), true );
  assert_eq!( result, "~in 3d $ren", "estimate renewal soonest → '~in 3d $ren', got: {result}" );
}

/// FT-18 variant — `next_event_label`: all sources absent or zero → em-dash.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
/// Source: [`009_token_usage.md` AC-28]
#[ test ]
fn ne_all_none_returns_dash()
{
  let result = next_event_label( None, None, false );
  assert_eq!( result, "\u{2014}", "all absent → em-dash, got: {result}" );
}

// ── relevant_quotas ────────────────────────────────────────────────────────

/// `prefer::any` + absent Sonnet → `relevant_7d_left` equals raw `seven_day_left`.
///
/// When `seven_day_sonnet = None`, `prefer::any` must fall back to `seven_day_left`
/// (cannot take min with an absent value). Bug: old `prefer_weekly` computed
/// `min(7d_left, 100.0 - map_or(0.0, ...))` = `min(7d_left, 100.0)` = `7d_left` —
/// accidentally correct for `prefer::any`. Verified here as an explicit contract.
#[ test ]
fn test_relevant_quotas_any_no_sonnet()
{
  // h5_util=20.0, d7_util=30.0 → five_h_left=80.0, d7_left=70.0, seven_day_sonnet=None.
  let aq     = mk_aq_ok_both( 20.0, 30.0 );
  let quotas = relevant_quotas( &aq, PreferStrategy::Any );
  assert!(
    ( quotas.1 - 70.0 ).abs() < 1e-9,
    "prefer::any + absent Sonnet → relevant_7d_left must equal d7_left (70.0); got: {}",
    quotas.1,
  );
}

/// `prefer::son` + absent Sonnet → `relevant_7d_left = 0.0` (absent = unknown, not 100%).
///
/// # Root Cause (Phase 2 bug fix)
/// Old `prefer_weekly` computed `100.0 - map_or(0.0, ...)`. When `seven_day_sonnet = None`,
/// `map_or(0.0, ...)` yields `0.0`, so the result is `100.0 - 0.0 = 100.0`. An absent
/// Sonnet tier was treated as 100% remaining — silently inflating `prefer_weekly` and making
/// h-exhausted accounts appear eligible for `sort::renew` despite unknown Sonnet capacity.
///
/// # Fix Applied
/// `relevant_quotas` uses `if let Some(ref son)` — when `None`, returns `0.0`. Absent = unknown;
/// unknown Sonnet is treated as unavailable for `prefer::son` eligibility purposes.
///
/// # Pitfall
/// `map_or(0.0, ...)` is correct for DISPLAY (show 0% when tier absent). It is WRONG for
/// quota-eligibility gates — absent is not the same as exhausted, but for preference purposes
/// where the user explicitly requests Sonnet, absent Sonnet must be treated as ineligible.
#[ test ]
fn test_relevant_quotas_son_no_sonnet()
{
  // d7_util=30.0, seven_day_sonnet=None — the bug case.
  let aq     = mk_aq_ok_both( 20.0, 30.0 );
  let quotas = relevant_quotas( &aq, PreferStrategy::Sonnet );
  assert!(
    quotas.1.abs() < 1e-9,
    "prefer::son + absent Sonnet → relevant_7d_left must be 0.0 (absent = unknown); got: {}",
    quotas.1,
  );
}

/// `prefer::son` + present Sonnet → `relevant_7d_left` = 100.0 - utilization.
///
/// Standard case: Sonnet tier present with 70% utilization → 30% remaining.
#[ test ]
fn test_relevant_quotas_son_with_sonnet()
{
  // mk_aq_sort_weekly: h5=20.0, d7=30.0, son=70.0 → son_left = 30.0.
  let aq     = mk_aq_sort_weekly( "t@x.com", 20.0, 30.0, 70.0 );
  let quotas = relevant_quotas( &aq, PreferStrategy::Sonnet );
  assert!(
    ( quotas.1 - 30.0 ).abs() < 1e-9,
    "prefer::son + son_util=70.0 → relevant_7d_left must be 30.0; got: {}",
    quotas.1,
  );
}

/// `prefer::opus` → `relevant_7d_left` = raw `seven_day_left`, Sonnet tier irrelevant.
///
/// Opus intent uses only the overall 7d quota; Sonnet cap is not a constraint.
/// Even with a nearly-exhausted Sonnet tier (`son_util=95.0`), the result is raw `d7_left`.
#[ test ]
fn test_relevant_quotas_opus()
{
  // d7_util=40.0 → d7_left=60.0; son_util=95.0 (high, but irrelevant for Opus).
  let aq     = mk_aq_sort_weekly( "t@x.com", 20.0, 40.0, 95.0 );
  let quotas = relevant_quotas( &aq, PreferStrategy::Opus );
  assert!(
    ( quotas.1 - 60.0 ).abs() < 1e-9,
    "prefer::opus → relevant_7d_left must equal raw d7_left (60.0), ignoring Sonnet; got: {}",
    quotas.1,
  );
}

/// `Err` result → sentinel `(-1.0, 0.0)`.
///
/// Error accounts have no quota data; `relevant_quotas` must return the ineligibility
/// sentinel without panicking or accessing unavailable data.
#[ test ]
fn test_relevant_quotas_err()
{
  let aq     = mk_aq_err();
  let quotas = relevant_quotas( &aq, PreferStrategy::Any );
  assert!(
    ( quotas.0 - ( -1.0 ) ).abs() < 1e-9 && quotas.1.abs() < 1e-9,
    "Err result → relevant_quotas must return (-1.0, 0.0); got: {quotas:?}",
  );
}

// ── recommended_model: FT-01..FT-04, EC-01 ────────────────────────────────────

/// FT-01 — `recommended_model()` returns `"sonnet"` when `seven_day_sonnet` tier is absent.
///
/// Absent Sonnet tier → unknown capacity → conservative recommendation is Sonnet.
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` FT-01]
#[ test ]
fn ft01_recommended_model_sonnet_when_tier_absent()
{
  let aq = mk_aq_sort( "a@test.com", 0.0, FAR_FUTURE_MS );  // seven_day_sonnet = None
  assert_eq!(
    recommended_model( &aq ), "sonnet",
    "absent Sonnet tier must return sonnet; got: {:?}", recommended_model( &aq ),
  );
}

/// FT-02 — `recommended_model()` returns `"sonnet"` when Sonnet left is exactly 15%.
///
/// Guard is strict `< OPUS_OVERRIDE_THRESHOLD`; utilization = 85.0 → 15.0% left → NOT opus.
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` FT-02]
#[ test ]
fn ft02_recommended_model_sonnet_at_exactly_15_pct_left()
{
  let aq = mk_aq_sort_weekly( "a@test.com", 0.0, 0.0, 85.0 );  // 15% left exactly
  assert_eq!(
    recommended_model( &aq ), "sonnet",
    "utilization=85.0 (15% left) must return sonnet (strict < boundary); got: {:?}", recommended_model( &aq ),
  );
}

/// FT-03 — `recommended_model()` returns `"opus"` when Sonnet left is < 15%.
///
/// utilization = 86.0 → 14.0% left → opus override fires.
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` FT-03]
#[ test ]
fn ft03_recommended_model_opus_when_sonnet_below_threshold()
{
  let aq = mk_aq_sort_weekly( "a@test.com", 0.0, 0.0, 86.0 );  // 14% left
  assert_eq!(
    recommended_model( &aq ), "opus",
    "utilization=86.0 (14% left) must return opus; got: {:?}", recommended_model( &aq ),
  );
}

/// FT-04 — `recommended_model()` returns `"sonnet"` on Err result.
///
/// Quota fetch failed → cannot determine Sonnet capacity → conservative: sonnet.
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` FT-04]
#[ test ]
fn ft04_recommended_model_sonnet_on_err()
{
  let aq = mk_aq_err();
  assert_eq!(
    recommended_model( &aq ), "sonnet",
    "Err result must return sonnet; got: {:?}", recommended_model( &aq ),
  );
}

/// EC-01 — `recommended_model()` boundary: utilization = 84.999 returns `"sonnet"`.
///
/// 100.0 - 84.999 = 15.001% left → strictly above threshold → sonnet (not opus).
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` EC-01]
#[ test ]
fn ec01_recommended_model_sonnet_at_15_001_pct_left()
{
  let aq = mk_aq_sort_weekly( "a@test.com", 0.0, 0.0, 84.999 );
  assert_eq!(
    recommended_model( &aq ), "sonnet",
    "utilization=84.999 (15.001% left) must return sonnet; got: {:?}", recommended_model( &aq ),
  );
}

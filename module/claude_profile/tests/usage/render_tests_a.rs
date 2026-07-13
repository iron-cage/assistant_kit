// Integration tests for render.rs — Part A (split from src/usage/render_tests.rs).
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::{ render_text, render_tsv, render_json };
use claude_profile::usage::test_bridge::types::{ AccountQuota, SortStrategy, PreferStrategy, ColsVisibility };
use claude_profile::usage::test_bridge::FAR_FUTURE_MS;

/// FT-20/009 — `~Renews` must retain billing date (not error reason) for 429-errored accounts.
///
/// # Root Cause
/// `render_text()` Err arm used `*row.last_mut().unwrap() = error_str` (positional blind
/// overwrite). Under default layout (`host`/`role` OFF), `~Renews` was the last pushed
/// column, so the billing date was discarded and replaced with the quota API error reason.
/// `render_tsv()` Err arm explicitly pushed `error_str` for the renews cell, same effect.
///
/// # Why Not Caught
/// All prior Err-arm tests used `account: None` (→ `renews_str = "?"`), so the overwrite
/// was invisible — both the buggy value and the intended value were "a non-date string".
/// No test combined `account: Some(OauthAccountData { ... })` with `result: Err(...)`.
///
/// # Fix Applied
/// `render_text()`: replaced `last_mut()` with `row[ quota_end_len - 1 ] = error_str`
/// (targets only the last visible quota-data column; `~Renews` is outside that range).
/// `render_tsv()`: push `col_count - 1` dashes then push `error_str` directly as the last
/// quota entry; renews cell changed from `error_str` to `renews_str`.
///
/// # Prevention
/// Any Err-arm render test covering 429/401/403 accounts must supply `account: Some(...)` and
/// assert that the renews cell retains a billing date, not the error reason.
///
/// # Pitfall
/// `mk_aq_err()` sets `account: None` → `renews_str = "?"` → the assertion
/// `!= "?"` would pass even with the bug present. Always construct the struct literal
/// directly when testing Err-arm behavior with `OauthAccountData` present.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-20]
#[ doc = "bug_reproducer(BUG-220)" ]
#[ test ]
fn mre_bug_220_renews_preserved_for_429_accounts()
{
  let aq = AccountQuota
  {
    fallback_reason : None,
    name          : "i11@test.com".to_string(),
    is_current    : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Err( "rate limited (429)".to_string() ),
    account       : Some( claude_quota::OauthAccountData
    {
      tagged_id       : String::new(),
      uuid            : String::new(),
      email_address   : String::new(),
      full_name       : String::new(),
      display_name    : String::new(),
      billing_type    : "stripe_subscription".to_string(),
      has_max         : true,
      capabilities    : vec![],
      rate_limit_tier : String::new(),
      org_created_at  : "1970-01-15T00:00:00Z".to_string(),
      memberships     : vec![],
    }),
    host          : String::new(),
    role          : String::new(),
    renewal_at    : None,
    cached        : false,
    cache_age_secs : None,
    // Fix(BUG-327): mirrors account.org_created_at above, matching what fetch.rs's
    //   live-fetch path actually derives (account.as_ref().map(|a| a.org_created_at.clone())).
    // Root cause: -add_org_created_at.py blindly inserted None into every AccountQuota
    //   literal missing the field, without syncing it to an already-populated account field.
    // Pitfall: this literal bypasses fetch.rs entirely, so nothing else keeps the two
    //   fields consistent — a literal with `account: Some(..)` must set this by hand.
    org_created_at : Some( "1970-01-15T00:00:00Z".to_string() ),
    is_owned      : true,
    owner                : String::new(),
  };
  let accounts = vec![ aq ];
  let cols     = ColsVisibility::default_set();

  // TSV: `renews` column must hold the billing date — NOT the error reason.
  let tsv        = render_tsv( &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols );
  let mut lines  = tsv.lines();
  let header     = lines.next().expect( "TSV must have a header row" );
  let data_row   = lines.next().expect( "TSV must have a data row" );
  let headers    : Vec< &str > = header.split( '\t' ).collect();
  let fields     : Vec< &str > = data_row.split( '\t' ).collect();
  let renews_idx = headers.iter().position( |h| *h == "renews" )
    .expect( "renews column must be present in TSV header" );
  let renews_val = fields.get( renews_idx ).copied().unwrap_or( "" );

  assert_ne!(
    renews_val,
    "(rate limited (429))",
    "BUG-220: TSV ~Renews must not contain error_str for 429 accounts with valid \
     OauthAccountData; got {renews_val:?}",
  );
  assert_ne!(
    renews_val,
    "?",
    "BUG-220: TSV ~Renews must show billing date when OauthAccountData is available; \
     got {renews_val:?}",
  );

  // Text renderer: the error reason must appear somewhere in output (in a quota column).
  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols, None, None, None, None, false,
  );
  assert!(
    text.contains( "(rate limited (429))" ),
    "BUG-220: error reason must appear in render_text output (in a quota column)",
  );
}

/// FT-21/009 — `@` flag in text and TSV for accounts occupied on another machine.
///
/// Priority chain under test: `✓` (`is_current`) outranks `@`; `@` appears when
/// `is_current=false` AND `is_active=false` AND `is_occupied_elsewhere=true`.
///
/// Both renderers use the same flag priority — this test covers both.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-21]
#[ test ]
fn test_ft21_009_occupied_elsewhere_at_flag()
{
  let mk_aq = | name : &str, is_current : bool, is_active : bool, is_occupied_elsewhere : bool |
  {
    AccountQuota
    {
      fallback_reason : None,
      name                  : name.to_string(),
      is_current,
      is_active,
      is_occupied_elsewhere,
      expires_at_ms         : FAR_FUTURE_MS,
      result                : Ok( claude_quota::OauthUsageData
      {
        five_hour        : Some( claude_quota::PeriodUsage { utilization : 50.0, resets_at : None } ),
        seven_day        : None,
        seven_day_sonnet : None,
      } ),
      account               : None,
      host                  : String::new(),
      role                  : String::new(),
      renewal_at            : None,
      cached                : false,
      cache_age_secs        : None,
      is_owned              : true,
      owner                : String::new(),
          org_created_at : None,
    }
  };

  // alice: is_current=true → ✓; bob: is_occupied_elsewhere=true, not current/active → @
  let accounts = vec!
  [
    mk_aq( "alice@test.com", true,  true,  false ),
    mk_aq( "bob@test.com",   false, false, true  ),
  ];
  let cols = ColsVisibility::default_set();

  // --- text renderer ---
  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols, None, None, None, None, false,
  );
  let alice_text = text.lines().find( | l | l.contains( "alice@test.com" ) )
    .expect( "FT-21: alice line missing from render_text" );
  let bob_text   = text.lines().find( | l | l.contains( "bob@test.com" ) )
    .expect( "FT-21: bob line missing from render_text" );

  assert!(
    alice_text.contains( '\u{2713}' ),
    "FT-21: alice (is_current=true) must show ✓ in text render; got: {alice_text:?}",
  );
  assert_eq!(
    bob_text.split_whitespace().next().unwrap_or( "" ),
    "@",
    "FT-21: bob (is_occupied_elsewhere=true) first token must be @ in text render; got: {bob_text:?}",
  );

  // --- TSV renderer ---
  let tsv   = render_tsv( &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols );
  let mut tsv_lines = tsv.lines();
  let _header  = tsv_lines.next().expect( "FT-21: TSV must have a header row" );
  // rows sorted by Name: alice before bob
  let alice_tsv = tsv_lines.next().expect( "FT-21: alice TSV row missing" );
  let bob_tsv   = tsv_lines.next().expect( "FT-21: bob TSV row missing" );

  let alice_flag_tsv = alice_tsv.split( '\t' ).next().unwrap_or( "" );
  let bob_flag_tsv   = bob_tsv.split( '\t' ).next().unwrap_or( "" );

  assert_eq!(
    alice_flag_tsv, "\u{2713}",
    "FT-21: alice TSV flag cell must be ✓; got: {alice_flag_tsv:?}",
  );
  assert_eq!(
    bob_flag_tsv, "@",
    "FT-21: bob TSV flag cell must be @; got: {bob_flag_tsv:?}",
  );
}

/// FT-03/033 — `render_text` prefixes non-dash quota cells with `~` for cached rows.
///
/// # Root Cause
/// Cached rows return `Ok()` with `cached=true`; `render_text` must prefix
/// all non-dash percentage cells with `~` to indicate stale data via `prefix_tilde`.
///
/// # Why Not Caught
/// No test exercised a cached `AccountQuota` through `render_text`.
///
/// # Fix Applied
/// `prefix_tilde` mutates cells in-place when `aq.cached` is true.
///
/// # Prevention
/// Any change to the `Ok` render path must verify tilde prefix for cached rows.
///
/// # Pitfall
/// Em-dash cells must NOT receive the tilde prefix — only percentage and reset cells.
///
/// Spec: [`tests/docs/feature/033_quota_cache.md` FT-03]
#[ test ]
fn ft03_033_render_text_cached_shows_tilde_prefix()
{
  let aq = AccountQuota
  {
    fallback_reason : None,
    name                  : "cached@example.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 14.0, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : true,
    cache_age_secs        : Some( 300 ),
    is_owned              : true,
    owner                : String::new(),
      org_created_at : None,
  };
  let accounts = vec![ aq ];
  let cols     = ColsVisibility::default_set();
  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols, None, None, None, None, false,
  );
  assert!(
    text.contains( '~' ),
    "FT-03/033: cached row must show ~ prefix on non-dash quota cells; got:\n{text}",
  );
  assert!(
    text.contains( "~🟢 86%" ),
    "FT-03/033: 5h Left cell (14% util → 86% left, green) must be '~🟢 86%'; got:\n{text}",
  );
}

/// FT-09/033 — `render_json` includes `"cached"` and `"cache_age_secs"` fields.
///
/// # Root Cause
/// JSON output must surface cache metadata so consumers can distinguish
/// live from stale quota data.
///
/// # Why Not Caught
/// No test exercised a cached `AccountQuota` through `render_json`.
///
/// # Fix Applied
/// `cache_json_fields` appended to each JSON entry in both `Ok` and `Err` arms.
///
/// # Prevention
/// Any change to the JSON output format must verify `"cached"` and `"cache_age_secs"` present.
///
/// # Pitfall
/// `cache_age_secs` must emit the numeric value (not `null`) when `Some(_)` is set.
///
/// Spec: [`tests/docs/feature/033_quota_cache.md` FT-09]
#[ test ]
fn ft09_033_render_json_cached_includes_fields()
{
  let aq = AccountQuota
  {
    fallback_reason : None,
    name                  : "cached@example.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 14.0, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : true,
    cache_age_secs        : Some( 720 ),
    is_owned              : true,
    owner                : String::new(),
      org_created_at : None,
  };
  let accounts = vec![ aq ];
  let json = render_json( &accounts );
  assert!(
    json.contains( "\"cached\":true" ),
    "FT-09/033: render_json must include '\"cached\":true' for cached rows; got:\n{json}",
  );
  assert!(
    json.contains( "\"cache_age_secs\":720" ),
    "FT-09/033: render_json must include '\"cache_age_secs\":720'; got:\n{json}",
  );
}

/// FT-03/033 — cached sonnet reset column must show `~in` tilde prefix.
///
/// `son_reset` was computed separately from the 5-cell `cells` array and
/// bypassed `prefix_tilde`. Only visible when `cols::7d_son_reset` is enabled.
#[ test ]
fn ft03_033_cached_sonnet_reset_shows_tilde()
{
  let aq = AccountQuota
  {
    fallback_reason : None,
    name                  : "cached@example.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : None,
      seven_day        : None,
      seven_day_sonnet : Some( claude_quota::PeriodUsage
      {
        utilization : 80.0,
        resets_at   : Some( "2099-01-01T00:00:00Z".to_string() ),
      } ),
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : true,
    cache_age_secs        : Some( 600 ),
    is_owned              : true,
    owner                : String::new(),
      org_created_at : None,
  };
  let accounts = vec![ aq ];
  let mut cols = ColsVisibility::default_set();
  cols.d7_son_reset = true;
  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols, None, None, None, None, false,
  );
  assert!(
    text.contains( "~in " ),
    "cached sonnet reset must show ~in prefix; got:\n{text}",
  );
}

/// FT-23/009 — `~Renews` must show `"—"` for cancelled-subscription accounts.
///
/// # Root Cause
/// `renews_label` uses `org_created_at` unconditionally to project a billing date —
/// it has no `billing_type` parameter. Accounts with `billing_type == "none"` showed
/// `~in Nd` in `~Renews` despite `Sub = "—"` — the two columns contradicted each other.
///
/// # Why Not Caught
/// No prior Err-arm test supplied `OauthAccountData { billing_type: "none" }` and
/// checked the `~Renews` column. All prior tests used `account: None` → `"?"`.
///
/// # Fix Applied
/// Caller-side guard in `render_text()` and `render_tsv()`: when `billing_type == "none"`,
/// short-circuit to `"\u{2014}"` before passing `org_created_at` to `renews_label`.
/// Fix(BUG-232).
///
/// # Prevention
/// Any Err-arm test for cancelled-subscription accounts must verify `~Renews = "—"`,
/// not just the error label column.
///
/// # Pitfall
/// `org_created_at` may be present and parseable even after subscription cancellation.
/// The guard must check `billing_type` BEFORE consulting `org_created_at` — presence of
/// the org date does not imply an active renewal cycle.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-23]
#[ doc = "bug_reproducer(BUG-232)" ]
#[ test ]
fn test_ft23_009_renews_dash_for_cancelled_subscription()
{
  let aq = AccountQuota
  {
    fallback_reason : None,
    name                  : "cancelled@test.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Err( "no subscription".to_string() ),
    account               : Some( claude_quota::OauthAccountData
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
      org_created_at  : "2024-01-15T00:00:00Z".to_string(),
      memberships     : vec![],
    } ),
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    org_created_at        : None,
    is_owned              : true,
    owner                : String::new(),
  };
  let accounts = vec![ aq ];
  let cols     = ColsVisibility::default_set();

  // text renderer: ~Renews must be "—", NOT "~in Nd"
  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols, None, None, None, None, false,
  );
  assert!(
    text.contains( "\u{2014}" ),
    "FT-23: render_text must contain em-dash for cancelled subscription ~Renews; got:\n{text}",
  );
  assert!(
    !text.contains( "~in " ),
    "FT-23: render_text must NOT contain '~in ' for cancelled subscription; got:\n{text}",
  );

  // TSV renderer: renews column must be "—"
  let tsv       = render_tsv( &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols );
  let mut lines = tsv.lines();
  let header    = lines.next().expect( "FT-23: TSV must have header row" );
  let data_row  = lines.next().expect( "FT-23: TSV must have data row" );
  let headers   : Vec< &str > = header.split( '\t' ).collect();
  let fields    : Vec< &str > = data_row.split( '\t' ).collect();
  let renews_idx = headers.iter().position( |h| *h == "renews" )
    .expect( "FT-23: renews column must be present in TSV header" );
  let renews_val = fields.get( renews_idx ).copied().unwrap_or( "" );
  assert_eq!(
    renews_val, "\u{2014}",
    "FT-23: TSV ~Renews must be em-dash for billing_type='none'; got: {renews_val:?}",
  );
}

/// BUG-332 — render layer's `billing_type=="none"` gate must also require `result.is_err()`
/// (matching `fetch.rs:255`'s BUG-236 conjunction), not `billing_type` alone.
///
/// # Root Cause
/// `render.rs:108`, `render.rs:374` (`GetField::Renews`), and `render_tsv.rs:72` all blanked
/// `~Renews` to "—" whenever `billing_type == "none"`, regardless of `aq.result`. BUG-236
/// already established that `billing_type == "none"` alone is not sufficient proof of "no
/// active subscription" — it must be conjoined with `result.is_err()`. The render layer
/// never inherited that conjunction, so a live non-stripe account with `result = Ok(...)`
/// and a real renewal date still showed "—" instead of the actual date.
///
/// # Why Not Caught
/// All prior `billing_type == "none"` render tests (`test_ft23_009_renews_dash_for_cancelled_subscription`)
/// paired it with `result: Err(...)`. No test paired `billing_type == "none"` with `result: Ok(...)`.
///
/// # Fix Applied
/// Extracted `AccountQuota::is_no_subscription()` encoding the full conjunction
/// (`billing_type == "none" && result.is_err()`) and applied it at all 3 render-layer sites.
///
/// # Prevention
/// Any new `billing_type=="none"` call site in the render layer must call
/// `is_no_subscription()`, never re-derive the literal condition.
///
/// # Pitfall
/// `billing_type == "none"` alone is NOT sufficient — the conjunction with `result.is_err()`
/// is what BUG-236 proved necessary at the fetch layer; the render layer must match it.
///
/// Spec: [`docs/invariant/011_shared_predicate_consistency.md`]
#[ doc = "bug_reproducer(BUG-332)" ]
#[ test ]
fn mre_bug332_renews_shown_for_billing_none_with_ok_result()
{
  let aq = AccountQuota
  {
    fallback_reason : None,
    name                  : "live@test.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : Some( "2099-01-01T00:00:00Z".to_string() ) } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : Some( "2099-01-01T00:00:00Z".to_string() ) } ),
      seven_day_sonnet : None,
    } ),
    account               : Some( claude_quota::OauthAccountData
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
      org_created_at  : "2024-01-15T00:00:00Z".to_string(),
      memberships     : vec![],
    } ),
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    org_created_at        : None,
    is_owned              : true,
    owner                 : String::new(),
  };
  let accounts = vec![ aq ];
  let cols     = ColsVisibility::default_set();

  // text renderer: ~Renews must show a real date, NOT "—". `seven_day` is populated (unlike
  // `seven_day_sonnet`, irrelevant here) so 7d Left/7d Reset don't contribute an unrelated
  // legitimate em-dash — the only em-dash this row could show is a buggy ~Renews.
  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols, None, None, None, None, false,
  );
  assert!(
    !text.contains( "\u{2014}" ),
    "BUG-332: render_text must NOT show em-dash for ~Renews when result=Ok and billing_type=none; got:\n{text}",
  );

  // TSV renderer: renews column must show a real date, not "—".
  let tsv       = render_tsv( &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols );
  let mut lines = tsv.lines();
  let header    = lines.next().expect( "BUG-332: TSV must have header row" );
  let data_row  = lines.next().expect( "BUG-332: TSV must have data row" );
  let headers   : Vec< &str > = header.split( '\t' ).collect();
  let fields    : Vec< &str > = data_row.split( '\t' ).collect();
  let renews_idx = headers.iter().position( |h| *h == "renews" )
    .expect( "BUG-332: renews column must be present in TSV header" );
  let renews_val = fields.get( renews_idx ).copied().unwrap_or( "" );
  assert_ne!(
    renews_val, "\u{2014}",
    "BUG-332: TSV ~Renews must NOT be em-dash when result=Ok and billing_type=none; got: {renews_val:?}",
  );
}

// ── Non-owned account display ─────────────────────────────────────────────

/// FT-05 (AC-05): Non-owned account display — `~` prefix when cache present; Err when absent.
///
/// Case A: `is_owned=false, cached=true` with quota data → rendered with `~` prefix.
/// Case B: `is_owned=false, cached=false` with Err result → no `~` prefix; error shown.
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-05]
#[ test ]
fn ft05_non_owned_display_tilde_or_dashes()
{
  let cols = ColsVisibility::default_set();

  // Case A: non-owned + cache present → tilde prefix on quota cells (same as any cached row).
  let aq_cached = AccountQuota
  {
    fallback_reason : None,
    name                  : "alice@test.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 30.0, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : true,
    cache_age_secs        : Some( 600 ),
    is_owned              : false,
    owner                : String::new(),
      org_created_at : None,
  };
  let text_a = render_text(
    &[ aq_cached ],
    SortStrategy::Name, None, PreferStrategy::Any,
    &cols, None, None, None, None, false,
  );
  assert!(
    text_a.contains( '~' ),
    "FT-05 case A: non-owned cached row must show ~ prefix; got:\n{text_a}",
  );

  // Case B: non-owned + no cache → Err result; no tilde; error indicator shown.
  let aq_no_cache = AccountQuota
  {
    fallback_reason : None,
    name                  : "bob@test.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Err( "not owned".to_string() ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : false,
    owner                : String::new(),
      org_created_at : None,
  };
  let text_b = render_text(
    &[ aq_no_cache ],
    SortStrategy::Name, None, PreferStrategy::Any,
    &cols, None, None, None, None, false,
  );
  // No tilde prefix when no cache data.
  assert!(
    !text_b.contains( "~🟢" ) && !text_b.contains( "~🟡" ) && !text_b.contains( "~🔴" ),
    "FT-05 case B: non-owned non-cached must not show ~ tilde on status emoji; got:\n{text_b}",
  );
  // Error account renders with 🔴 status.
  assert!(
    text_b.contains( "🔴" ),
    "FT-05 case B: non-owned non-cached Err must show 🔴 status; got:\n{text_b}",
  );
}

/// FT-12 (AC-12): `format::json` includes `"is_owned": true` or `"is_owned": false`.
///
/// Spec: [`tests/docs/feature/036_account_ownership.md` FT-12]
#[ test ]
fn ft12_json_output_includes_is_owned()
{
  let owned = AccountQuota
  {
    fallback_reason : None,
    name                  : "alice@test.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 20.0, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                : String::new(),
      org_created_at : None,
  };
  let not_owned = AccountQuota
  {
    fallback_reason : None,
    name                  : "bob@test.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Err( "not owned".to_string() ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : false,
    owner                : String::new(),
      org_created_at : None,
  };

  let json = render_json( &[ owned, not_owned ] );

  assert!(
    json.contains( "\"is_owned\":true" ),
    "FT-12: JSON must include '\"is_owned\":true' for owned account; got:\n{json}",
  );
  assert!(
    json.contains( "\"is_owned\":false" ),
    "FT-12: JSON must include '\"is_owned\":false' for non-owned account; got:\n{json}",
  );
}

/// FT-28 boundary — footer shows `model: sonnet` when `seven_day_sonnet` utilization = 85.0
/// (exactly 10% left). Threshold is strict `< 10%`, so 10.0% must NOT trigger the opus override.
///
/// In RED (before fix): `10.0 < 20.0 == true` → footer shows `model: opus` → this test FAILS.
/// In GREEN (after fix at render.rs:258): `10.0 < 10.0 == false` → footer shows `model: sonnet`.
///
/// Spec: [`tests/docs/feature/09_token_usage.md` FT-28]
#[ test ]
fn test_render_footer_model_label_at_10pct_no_override()
{
  // a@x.com: non-current, alphabetically first → footer winner with sort::Name.
  // son_util = 90.0 → sonnet_left = 10.0% — exactly at the 10% threshold.
  let aq_a = AccountQuota
  {
    fallback_reason : None,
    name                  : "a@x.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage { utilization : 90.0, resets_at : None } ),
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                 : String::new(),
      org_created_at : None,
  };
  // b@x.com: second valid account required for footer (≥ 2 valid triggers footer display).
  let aq_b = AccountQuota
  {
    fallback_reason : None,
    name                  : "b@x.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage { utilization : 50.0, resets_at : None } ),
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                 : String::new(),
      org_created_at : None,
  };
  // cur@x.com: is_current=true — triggers 2-line `·`-delimited footer so the model label appears.
  let aq_cur = AccountQuota
  {
    fallback_reason : None,
    name                  : "cur@x.com".to_string(),
    is_current            : true,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage { utilization : 50.0, resets_at : None } ),
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                 : String::new(),
      org_created_at : None,
  };
  let output = render_text(
    &[ aq_cur, aq_a, aq_b ], SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  // Footer line 2: `Next (name) · a@x.com · sonnet · {metric}` — 10.0% left is NOT < 10%.
  assert!(
    output.contains( "· sonnet" ),
    "FT-28 boundary: footer line 2 must show '· sonnet' when sonnet_left = 10.0% (not < 10%); got:\n{output}",
  );
}

/// FT-28 boundary — footer shows `model: opus` when `seven_day_sonnet` utilization = 91.0
/// (9.0% left, below the 10% threshold).
///
/// Regression guard: both old (`< 20.0`) and new (`< 10.0`) code fire at 9.0% left — opus
/// must appear before and after the fix. Ensures the fix doesn't break below-threshold behaviour.
///
/// Fix(BUG-336): originally used utilization=90.1 (left≈9.9) — once `recommended_model()`
///   rounds its comparison input (this file's own BUG-336 fix), 9.9 rounds UP to 10 (the
///   threshold), no longer demonstrating "below threshold". Recalibrated to 91.0 (left=9.0,
///   an exact integer, unambiguously below threshold both before and after rounding).
///
/// Spec: [`tests/docs/feature/09_token_usage.md` FT-28]
#[ test ]
fn test_render_footer_model_label_below_10pct_opus()
{
  // a@x.com: non-current, alphabetically first → footer winner with sort::Name.
  // son_util = 91.0 → sonnet_left = 9.0% — below 10% threshold.
  let aq_a = AccountQuota
  {
    fallback_reason : None,
    name                  : "a@x.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage { utilization : 91.0, resets_at : None } ),
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                 : String::new(),
      org_created_at : None,
  };
  // b@x.com: second valid account required for footer (≥ 2 valid triggers footer display).
  let aq_b = AccountQuota
  {
    fallback_reason : None,
    name                  : "b@x.com".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage { utilization : 50.0, resets_at : None } ),
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                 : String::new(),
      org_created_at : None,
  };
  // cur@x.com: is_current=true — triggers 2-line `·`-delimited footer so the model label appears.
  let aq_cur = AccountQuota
  {
    fallback_reason : None,
    name                  : "cur@x.com".to_string(),
    is_current            : true,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
      seven_day_sonnet : Some( claude_quota::PeriodUsage { utilization : 50.0, resets_at : None } ),
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : false,
    cache_age_secs        : None,
    is_owned              : true,
    owner                 : String::new(),
      org_created_at : None,
  };
  let output = render_text(
    &[ aq_cur, aq_a, aq_b ], SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
  // Footer line 2: `Next (name) · a@x.com · opus · {metric}` — 9.0% left IS < 10%.
  assert!(
    output.contains( "· opus" ),
    "FT-28 boundary: footer line 2 must show '· opus' when sonnet_left = 9.0% (< 10%); got:\n{output}",
  );
}

/// FT-14/033 — cache-fallback rows must surface the original fetch-failure reason on all 3
/// render surfaces (text, TSV, JSON), not just the pre-existing cache-age suffix (AC-03).
///
/// # Root Cause
/// `AccountQuota` had no field to carry a cache-fallback `Err→Ok` conversion's original reason
/// forward from `fetch.rs` to any render layer — `fetch.rs`'s cache-fallback conversion arm
/// discarded `e` entirely once `read_cached_quota` returned cached data, so cache-fallback rows
/// always rendered as if they were ordinary successes with only an age suffix, no indication of
/// why the row is stale.
///
/// # Why Not Caught
/// No prior test constructed a cache-fallback scenario (`cached: true` with a populated failure
/// reason) — every existing `cached: true` test used the only value that could exist before this
/// fix (`fallback_reason` did not exist as a field), so the missing-reason gap was structurally
/// invisible to every prior assertion.
///
/// # Fix Applied
/// Added `AccountQuota.fallback_reason : Option<String>`, populated with `Some(e.clone())` only
/// in `fetch.rs`'s cache-fallback conversion arm; `None` everywhere else (103 construction sites
/// across the crate). `render_text` combines it with `cache_age_label()`'s existing suffix via
/// `shorten_error()`; `render_tsv` (which has no pre-existing age-suffix mechanism) appends it as
/// a standalone parenthetical; `render_json` emits it as a new `"fallback_reason"` key.
///
/// # Prevention
/// Any new `AccountQuota` render surface must explicitly decide how to surface
/// `fallback_reason` — it is `Some` only on true cache-fallback rows and must never be silently
/// dropped the way the original bug dropped `e`.
///
/// # Pitfall
/// `render_tsv`'s NAME cell has NO pre-existing age-suffix mechanism (unlike `render_text`) — do
/// not assume the two formats combine `fallback_reason` identically; TSV's shortened reason is
/// the cell's only staleness indicator, standing alone with no age label to pair with.
///
/// Spec: [`docs/feature/033_quota_cache.md` AC-14]
#[ doc = "bug_reproducer(BUG-335)" ]
#[ test ]
fn mre_bug335_cache_fallback_reason_surfaced_on_all_render_surfaces()
{
  let aq = AccountQuota
  {
    fallback_reason : Some( "HTTP transport error: HTTP 429 Too Many Requests".to_string() ),
    name                  : "alice".to_string(),
    is_current            : false,
    is_active             : false,
    is_occupied_elsewhere : false,
    expires_at_ms         : FAR_FUTURE_MS,
    result                : Ok( claude_quota::OauthUsageData
    {
      five_hour        : Some( claude_quota::PeriodUsage { utilization : 50.0, resets_at : None } ),
      seven_day        : None,
      seven_day_sonnet : None,
    } ),
    account               : None,
    host                  : String::new(),
    role                  : String::new(),
    renewal_at            : None,
    cached                : true,
    cache_age_secs        : Some( 7200 ),
    is_owned              : true,
    owner                 : String::new(),
      org_created_at : None,
  };
  let accounts = vec![ aq ];
  let cols     = ColsVisibility::default_set();

  // T03: text table combines cache_age_label()'s existing suffix (AC-03) with the shortened
  // fallback reason (AC-14) in ONE parenthetical.
  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols, None, None, None, None, false,
  );
  assert!(
    text.contains( "alice (2h ago, rate limited (429))" ),
    "BUG-335: text table must combine the age suffix and shortened fallback reason in one \
     parenthetical; got:\n{text}",
  );

  // T04: TSV NAME cell has no pre-existing age-suffix mechanism — the shortened reason stands
  // alone as the cell's only staleness indicator.
  let tsv       = render_tsv( &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols );
  let mut lines = tsv.lines();
  let header    = lines.next().expect( "TSV must have a header row" );
  let data_row  = lines.next().expect( "TSV must have a data row" );
  let headers   : Vec< &str > = header.split( '\t' ).collect();
  let fields    : Vec< &str > = data_row.split( '\t' ).collect();
  let name_idx  = headers.iter().position( |h| *h == "account" )
    .expect( "account column must be present in TSV header" );
  let name_cell = fields.get( name_idx ).copied().unwrap_or( "" );
  assert_eq!(
    name_cell, "alice (rate limited (429))",
    "BUG-335: TSV NAME cell must append the shortened fallback reason standalone (no age \
     mechanism exists in this format); got {name_cell:?}",
  );

  // T05: JSON emits the shortened reason as its own field.
  let json = render_json( &accounts );
  assert!(
    json.contains( "\"fallback_reason\":\"rate limited (429)\"" ),
    "BUG-335: JSON must emit a \"fallback_reason\" field with the shortened reason; got:\n{json}",
  );
}


// Path-referenced test module for render.rs — compiled as `mod tests` via `#[path]`.
// Lives in src/usage/ (not tests/) to access pub(crate) render_text, render_tsv, and render_json
// without widening their visibility. See src/usage/readme.md § Inline Test Exception.

  use super::{ render_text, render_tsv, render_json };
  use crate::usage::types::{ AccountQuota, SortStrategy, PreferStrategy, NextStrategy, ColsVisibility };
  use crate::usage::test_support::FAR_FUTURE_MS;

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
      is_owned      : true,
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
      &accounts, SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &cols,
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
      &accounts, SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &cols,
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
    };
    let accounts = vec![ aq ];
    let cols     = ColsVisibility::default_set();
    let text = render_text(
      &accounts, SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &cols,
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
    };
    let accounts = vec![ aq ];
    let mut cols = ColsVisibility::default_set();
    cols.d7_son_reset = true;
    let text = render_text(
      &accounts, SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &cols,
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
      is_owned              : true,
    };
    let accounts = vec![ aq ];
    let cols     = ColsVisibility::default_set();

    // text renderer: ~Renews must be "—", NOT "~in Nd"
    let text = render_text(
      &accounts, SortStrategy::Name, None, PreferStrategy::Any, NextStrategy::Endurance, &cols,
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
    };
    let text_a = render_text(
      &[ aq_cached ],
      SortStrategy::Name, None, PreferStrategy::Any,
      NextStrategy::Endurance, &cols,
    );
    assert!(
      text_a.contains( '~' ),
      "FT-05 case A: non-owned cached row must show ~ prefix; got:\n{text_a}",
    );

    // Case B: non-owned + no cache → Err result; no tilde; error indicator shown.
    let aq_no_cache = AccountQuota
    {
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
    };
    let text_b = render_text(
      &[ aq_no_cache ],
      SortStrategy::Name, None, PreferStrategy::Any,
      NextStrategy::Endurance, &cols,
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
    };
    let not_owned = AccountQuota
    {
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

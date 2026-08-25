// Integration tests for render.rs — Part B (split from src/usage/render_tests.rs).
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::account::TagFilter;
use claude_profile::usage::test_bridge::render_text;
use claude_profile::usage::test_bridge::types::{ AccountQuota, SortStrategy, PreferStrategy, ColsVisibility };
use claude_profile::usage::test_bridge::{ FAR_FUTURE_MS, mk_aq_sort, mk_aq_sort_weekly };

/// FT-29/009 — footer line 1 shows `session:` and `effort:` only when supplied.
///
/// Three scenarios: both present, model only, neither — verifying optional field rendering
/// in the footer's first line (`Valid: N / M   session: <model>  effort: <level>`).
///
/// Spec: [`tests/docs/feature/09_token_usage.md` FT-29]
#[ test ]
#[ allow( clippy::too_many_lines ) ]
fn test_ft29_009_footer_session_effort_display()
{
  // Inner helper — builds three valid accounts; cur@x.com is `is_current=true` so the
  // 2-line `·`-delimited footer is used and session model/effort appear in col 3 of line 1.
  fn make_accounts() -> Vec< AccountQuota >
  {
    vec![
      AccountQuota
      {
        fallback_reason : None,
        touched_at_secs : None,
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
        claim_lock            : false,
        reserve               : false,
        inference_provider    : String::new(),
        tags : Vec::new(),
              org_created_at : None,
      },
      AccountQuota
      {
        fallback_reason : None,
        touched_at_secs : None,
        name                  : "a@x.com".to_string(),
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
        claim_lock            : false,
        reserve               : false,
        inference_provider    : String::new(),
        tags : Vec::new(),
              org_created_at : None,
      },
      AccountQuota
      {
        fallback_reason : None,
        touched_at_secs : None,
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
        claim_lock            : false,
        reserve               : false,
        inference_provider    : String::new(),
        tags : Vec::new(),
              org_created_at : None,
      },
    ]
  }

  // Scenario 1 — both session_model and session_effort supplied.
  // Footer line 1: `Current · cur@x.com · claude-sonnet-5/low · N/M`
  {
    let accounts = make_accounts();
    let output = render_text(
      &accounts, SortStrategy::Renew, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), Some( "claude-sonnet-5" ), Some( "low" ), None, None, false, &TagFilter::default() );
    assert!(
      output.contains( "claude-sonnet-5/low" ),
      "FT-29 s1: footer Current line col3 must be 'claude-sonnet-5/low'; got:\n{output}",
    );
    assert!(
      output.contains( "Current" ),
      "FT-29 s1: footer must have Current line; got:\n{output}",
    );
  }

  // Scenario 2 — session_model only; effort must be absent.
  // Footer line 1: `Current · cur@x.com · claude-sonnet-5 · N/M` (no slash)
  {
    let accounts = make_accounts();
    let output = render_text(
      &accounts, SortStrategy::Renew, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), Some( "claude-sonnet-5" ), None, None, None, false, &TagFilter::default() );
    assert!(
      output.contains( "claude-sonnet-5" ),
      "FT-29 s2: footer Current line must contain model name; got:\n{output}",
    );
    assert!(
      !output.contains( "effort:" ),
      "FT-29 s2: footer must not contain 'effort:' label when effort is None; got:\n{output}",
    );
    assert!(
      !output.contains( "/low" ),
      "FT-29 s2: footer must not contain '/low' when effort is None; got:\n{output}",
    );
  }

  // Scenario 3 — neither model nor effort; col3 of Current line is empty.
  {
    let accounts = make_accounts();
    let output = render_text(
      &accounts, SortStrategy::Renew, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, None, None, false, &TagFilter::default() );
    assert!(
      output.contains( "Current" ),
      "FT-29 s3: footer must have Current line even with no model/effort; got:\n{output}",
    );
    assert!(
      !output.contains( "session:" ),
      "FT-29 s3: footer must not contain 'session:' label when model is None; got:\n{output}",
    );
    assert!(
      !output.contains( "effort:" ),
      "FT-29 s3: footer must not contain 'effort:' label when effort is None; got:\n{output}",
    );
  }
}

// ── Sessions table ─────────────────────────────────────────────────────────

/// FT-30/009 — sessions table shown automatically when >1 `_active_*` marker exists.
///
/// 3 marker files in `TempDir`; own session identified by `active_marker_filename()`.
/// `who=None` (auto) → `marker_count` > 1 → table shown with `✓` on own row.
///
/// Spec: [`tests/docs/feature/09_token_usage.md` FT-30]
#[ test ]
fn ft30_009_sessions_table_shown_auto_multiple_markers()
{
  use tempfile::TempDir;
  use claude_profile::usage::test_bridge::mk_aq_ok;
  let store = TempDir::new().unwrap();
  let spath = store.path();

  // Own marker: exact filename from active_marker_filename().
  let own_fname = claude_profile_core::account::active_marker_filename();
  std::fs::write( spath.join( &own_fname ), "own@example.com" ).unwrap();
  // Other sessions on other machines.
  std::fs::write( spath.join( "_active_serverA_bob" ),   "bob@example.com" ).unwrap();
  std::fs::write( spath.join( "_active_serverB_carol" ), "carol@example.com" ).unwrap();

  let accounts = vec![ mk_aq_ok( 10.0 ) ];
  let cols     = ColsVisibility::default_set();
  let output   = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &cols, None, None, Some( spath ), None, false, &TagFilter::default() );

  assert!(
    output.contains( "Sessions" ),
    "FT-30: sessions table header must appear with 3 markers (who=None); got:\n{output}",
  );
  // Own session: account cell = "{account} ✓"
  assert!(
    output.contains( "own@example.com \u{2713}" ),
    "FT-30: own session must show '\u{2713}' on account cell; got:\n{output}",
  );
  assert!(
    output.contains( "bob@example.com" ),
    "FT-30: bob row must appear in sessions table; got:\n{output}",
  );
  assert!(
    output.contains( "carol@example.com" ),
    "FT-30: carol row must appear in sessions table; got:\n{output}",
  );
}

/// FT-31/009 — sessions table hidden automatically when ≤1 `_active_*` marker exists.
///
/// Only own marker present; `who=None` → `marker_count` = 1, `1 > 1 = false` → table not shown.
///
/// Spec: [`tests/docs/feature/09_token_usage.md` FT-31]
#[ test ]
fn ft31_009_sessions_table_hidden_auto_single_marker()
{
  use tempfile::TempDir;
  use claude_profile::usage::test_bridge::mk_aq_ok;
  let store = TempDir::new().unwrap();
  let spath = store.path();

  let own_fname = claude_profile_core::account::active_marker_filename();
  std::fs::write( spath.join( &own_fname ), "own@example.com" ).unwrap();

  let accounts = vec![ mk_aq_ok( 10.0 ) ];
  let cols     = ColsVisibility::default_set();
  let output   = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &cols, None, None, Some( spath ), None, false, &TagFilter::default() );

  assert!(
    !output.contains( "Sessions" ),
    "FT-31: sessions table must be hidden with only 1 marker (who=None); got:\n{output}",
  );
}

/// FT-32/009 — `who::` overrides automatic sessions table visibility.
///
/// `who=Some(true)` forces on with 1 marker; `who=Some(false)` suppresses with 3 markers.
///
/// Spec: [`tests/docs/feature/09_token_usage.md` FT-32]
#[ test ]
fn ft32_009_sessions_table_who_override()
{
  use tempfile::TempDir;
  use claude_profile::usage::test_bridge::mk_aq_ok;

  // who=Some(true) with 1 marker: force-on shows the table.
  {
    let store = TempDir::new().unwrap();
    let spath = store.path();
    let own_fname = claude_profile_core::account::active_marker_filename();
    std::fs::write( spath.join( &own_fname ), "own@example.com" ).unwrap();

    let accounts = vec![ mk_aq_ok( 10.0 ) ];
    let output = render_text(
      &accounts, SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, Some( spath ), Some( true ), false, &TagFilter::default() );
    assert!(
      output.contains( "Sessions" ),
      "FT-32 who=1: sessions table must appear with who=Some(true) even with 1 marker; got:\n{output}",
    );
  }

  // who=Some(false) with 3 markers: force-off suppresses the table.
  {
    let store = TempDir::new().unwrap();
    let spath = store.path();
    let own_fname = claude_profile_core::account::active_marker_filename();
    std::fs::write( spath.join( &own_fname ), "own@example.com" ).unwrap();
    std::fs::write( spath.join( "_active_serverA_bob" ),   "bob@example.com" ).unwrap();
    std::fs::write( spath.join( "_active_serverB_carol" ), "carol@example.com" ).unwrap();

    let accounts = vec![ mk_aq_ok( 10.0 ) ];
    let output = render_text(
      &accounts, SortStrategy::Name, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), None, None, Some( spath ), Some( false ), false, &TagFilter::default() );
    assert!(
      !output.contains( "Sessions" ),
      "FT-32 who=0: sessions table must be suppressed with who=Some(false) even with 3 markers; got:\n{output}",
    );
  }
}

/// FT-13/025 — cross-feature: sessions table renders `_active_*` markers as
/// `{user}@{host}` session identity → account rows.
///
/// Three markers: own + 2 others. Verifies Session column parsing from
/// `_active_{host}_{user}` filename, Account column from file content,
/// and `✓` on the own session row.
///
/// # Root Cause (BUG-308)
/// Previous version hardcoded `_active_devbox_devuser` and `_active_buildbox_devuser2` as
/// "other machine" marker filenames. On the test machine (hostname=devbox, user=devuser),
/// `active_marker_filename()` returns `_active_devbox_devuser` — the same name as the
/// hardcoded "other" marker. The second `fs::write` overwrote the own marker content
/// (`"own@test.com"` → `"alice@test.com"`), so `build_sessions_table` showed
/// `alice@test.com ✓` instead of `own@test.com ✓`.
///
/// # Why Not Caught
/// Test was written and validated on a machine where `active_marker_filename()` did not
/// collide with `_active_devbox_devuser`. The fragility is machine-specific and silent —
/// the test panics with a misleading message rather than a setup-collision error.
///
/// # Fix Applied
/// Fix(BUG-308): replaced hardcoded `_active_devbox_devuser` / `_active_buildbox_devuser2` with
/// clearly synthetic names `_active_testhost1_tst1` / `_active_testhost2_tst2`. Added
/// `assert_ne!` guards to fail loudly on any machine where a collision still occurs.
/// Own marker is written LAST to ensure it is never overwritten.
///
/// # Prevention
/// Any test writing `_active_*` marker files for "other machines" must use names that
/// cannot collide with `active_marker_filename()` on the real machine. Use synthetic
/// host/user identifiers and add `assert_ne!` guards as a safety net.
///
/// # Pitfall
/// `active_marker_filename()` depends on the actual hostname and `$USER` env var —
/// both vary across machines. Never hardcode expected identities like `devuser@devbox`
/// directly; use synthetic names or derive them from `active_marker_filename()`.
///
/// Spec: [`tests/docs/feature/25_per_machine_active_marker.md` FT-13]
#[ doc = "bug_reproducer(BUG-308)" ]
#[ test ]
fn ft13_025_sessions_table_parses_marker_identity_from_filename()
{
  use tempfile::TempDir;
  use claude_profile::usage::test_bridge::mk_aq_ok;
  let store = TempDir::new().unwrap();
  let spath = store.path();

  // Own marker: exact filename from `active_marker_filename()`.
  let own_fname = claude_profile_core::account::active_marker_filename();

  // "Other machine" markers use synthetic hostnames/users that no real machine is expected
  // to have. Fix(BUG-308): previous hardcoded `_active_devbox_devuser` overwrote the own marker
  // on machines where hostname=devbox, user=devuser (same name as `active_marker_filename()`).
  let other1_fname = "_active_testhost1_tst1";
  let other2_fname = "_active_testhost2_tst2";
  assert_ne!(
    own_fname.as_str(), other1_fname,
    "BUG-308 guard: own marker '{own_fname}' must not equal other1 '{other1_fname}' — pick different synthetic names",
  );
  assert_ne!(
    own_fname.as_str(), other2_fname,
    "BUG-308 guard: own marker '{own_fname}' must not equal other2 '{other2_fname}' — pick different synthetic names",
  );

  // Write "other" markers first, own marker LAST — ensures own is never overwritten.
  std::fs::write( spath.join( other1_fname ), "alice@test.com" ).unwrap();
  std::fs::write( spath.join( other2_fname ), "bob@test.com" ).unwrap();
  std::fs::write( spath.join( &own_fname ), "own@test.com" ).unwrap();

  let accounts = vec![ mk_aq_ok( 10.0 ) ];
  let cols     = ColsVisibility::default_set();
  // who=None: auto-shows because marker_count=3 > 1.
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &cols, None, None, Some( spath ), None, false, &TagFilter::default() );

  // Sessions table header must appear (3 markers, who=None → auto-show).
  assert!(
    output.contains( "Sessions" ),
    "FT-13: sessions table must appear with 3 markers; got:\n{output}",
  );
  // Session column: identity parsed as {user}@{host} from _active_{host}_{user} filename.
  // `_active_testhost1_tst1` → rsplit_once('_') → host="testhost1", user="tst1" → "tst1@testhost1"
  assert!(
    output.contains( "tst1@testhost1" ),
    "FT-13: `_active_testhost1_tst1` must render session 'tst1@testhost1'; got:\n{output}",
  );
  assert!(
    output.contains( "tst2@testhost2" ),
    "FT-13: `_active_testhost2_tst2` must render session 'tst2@testhost2'; got:\n{output}",
  );
  // Account column: file content (account names).
  assert!(
    output.contains( "alice@test.com" ),
    "FT-13: alice account from file content must appear; got:\n{output}",
  );
  assert!(
    output.contains( "bob@test.com" ),
    "FT-13: bob account from file content must appear; got:\n{output}",
  );
  // Own session: account cell = "{account} ✓".
  assert!(
    output.contains( "own@test.com \u{2713}" ),
    "FT-13: own session row must show '\u{2713}' on account cell; got:\n{output}",
  );
}

// test_kind: bug_reproducer(BUG-347)
/// FT-33/009 — sessions table flags a marker naming an account absent from the
/// credential store with `(stale)`, leaving a marker naming a live account
/// unflagged.
///
/// ## Root Cause (AC-33 coverage)
/// `build_sessions_table` rendered marker content verbatim with no existence
/// check, so a marker orphaned by a cross-machine delete (BUG-347) looked
/// identical to a genuinely active session.
///
/// ## Setup
/// Own marker plus two "other machine" markers — one naming `live@test.com`
/// (backed by a `.credentials.json` file in the same store) and one naming
/// `ghost@test.com` (no such file). Own marker is also backed, so only the
/// two rows under test are exercised by the stale check.
///
/// ## Assert
/// `ghost@test.com` renders with `(stale)` appended; `live@test.com` does not.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-33]
#[ test ]
fn ft33_009_sessions_table_flags_stale_marker_account()
{
  use tempfile::TempDir;
  use claude_profile::usage::test_bridge::mk_aq_ok;
  let store = TempDir::new().unwrap();
  let spath = store.path();

  let own_fname   = claude_profile_core::account::active_marker_filename();
  let live_fname  = "_active_testhost3_tst3";
  let ghost_fname = "_active_testhost4_tst4";
  assert_ne!( own_fname.as_str(), live_fname,  "BUG-347 guard: pick a different synthetic name" );
  assert_ne!( own_fname.as_str(), ghost_fname, "BUG-347 guard: pick a different synthetic name" );

  std::fs::write( spath.join( "live@test.com.credentials.json" ), "{}" ).unwrap();
  std::fs::write( spath.join( live_fname ), "live@test.com" ).unwrap();
  std::fs::write( spath.join( ghost_fname ), "ghost@test.com" ).unwrap();
  std::fs::write( spath.join( "own@test.com.credentials.json" ), "{}" ).unwrap();
  std::fs::write( spath.join( &own_fname ), "own@test.com" ).unwrap();

  let accounts = vec![ mk_aq_ok( 10.0 ) ];
  let cols     = ColsVisibility::default_set();
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &cols, None, None, Some( spath ), None, false, &TagFilter::default() );

  assert!(
    output.contains( "ghost@test.com (stale)" ),
    "FT-33: marker naming a since-deleted account must render with '(stale)' appended; got:\n{output}",
  );
  assert!(
    !output.contains( "live@test.com (stale)" ),
    "FT-33: marker naming a live account (backed by a .credentials.json) must NOT be flagged stale; got:\n{output}",
  );
  assert!(
    !output.contains( "own@test.com \u{2713} (stale)" ),
    "FT-33: own session backed by a .credentials.json must NOT be flagged stale; got:\n{output}",
  );
}

/// EC-5/062 — `who::1` with 0 `_active_*` markers → sessions table omitted gracefully.
///
/// `build_sessions_table` returns an empty string when no markers exist.
/// The `if show && !sessions_text.is_empty()` guard suppresses the append even when
/// `who = Some(true)`, so the output contains no `Sessions` section.
///
/// Spec: [`tests/docs/cli/param/62_who.md` EC-5]
#[ test ]
fn ec5_062_who_force_on_zero_markers_omits_gracefully()
{
  use tempfile::TempDir;
  use claude_profile::usage::test_bridge::mk_aq_ok;
  let store = TempDir::new().unwrap();
  // Deliberately empty — no `_active_*` files written.
  let spath = store.path();

  let accounts = vec![ mk_aq_ok( 10.0 ) ];
  let output   = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, Some( spath ), Some( true ), false, &TagFilter::default() );

  assert!(
    !output.contains( "Sessions" ),
    "EC-5: sessions table must be gracefully omitted when store has 0 markers \
     and who=Some(true); got:\n{output}",
  );
}

// ── Corner-case tests ───────────────────────────────────────────────────────

/// CC-06: Single valid account → footer is NOT emitted.
///
/// Root Cause: `valid_count < 2` guard at render.rs:226 early-returns without footer.
/// Verifies the `< 2` threshold — a single valid account must not show
/// "Current" / "Next" / "Valid:" lines.
#[ test ]
fn cc_single_valid_account_no_footer()
{
  use claude_profile::usage::test_bridge::mk_aq_ok;
  let mut aq = mk_aq_ok( 20.0 );
  aq.is_current = true;
  let accounts = vec![ aq ];
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false, &TagFilter::default() );
  // "Current ·" is the footer format; bare "Current" could appear elsewhere.
  assert!(
    !output.contains( "Current \u{00b7}" ),
    "CC-06: single valid account must not show 'Current ·' footer; got:\n{output}",
  );
  // "Next (" is the footer format `Next (name)` / `Next (renew)`.
  assert!(
    !output.contains( "Next (" ),
    "CC-06: single valid account must not show 'Next (...)' footer; got:\n{output}",
  );
  assert!(
    !output.contains( "Valid:" ),
    "CC-06: single valid account must not show 'Valid:' footer; got:\n{output}",
  );
}

/// CC-07: No `is_current` account among ≥2 valid → legacy "Valid: N / M" footer.
///
/// Root Cause: the `if let Some( cur ) = accounts.iter().find(|aq| aq.is_current)`
/// guard at render.rs:260 falls to the `else` branch producing "Valid: N / M".
/// Verifies the fallback format when credentials are unreadable.
#[ test ]
fn cc_no_current_account_uses_legacy_footer()
{
  // Two valid accounts, neither is_current → legacy footer.
  let mk = | name : &str |
  {
    AccountQuota
    {
      fallback_reason : None,
      touched_at_secs : None,
      name                  : name.to_string(),
      is_current            : false,
      is_active             : false,
      is_occupied_elsewhere : false,
      expires_at_ms         : FAR_FUTURE_MS,
      result                : Ok( claude_quota::OauthUsageData
      {
        five_hour        : Some( claude_quota::PeriodUsage { utilization : 10.0, resets_at : None } ),
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
      owner                 : String::new(),
      claim_lock            : false,
      reserve               : false,
      inference_provider    : String::new(),
      tags : Vec::new(),
          org_created_at : None,
    }
  };
  let accounts = vec![ mk( "a@x.com" ), mk( "b@x.com" ) ];
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false, &TagFilter::default() );
  assert!(
    output.contains( "Valid:" ),
    "CC-07: no is_current among ≥2 valid must use legacy 'Valid:' footer; got:\n{output}",
  );
  assert!(
    !output.contains( "Current" ),
    "CC-07: legacy footer must not contain 'Current' line; got:\n{output}",
  );
}

/// CC-08: Effort-only session (no model) → footer col3 shows just the effort level.
///
/// Root Cause: the `(None, Some(se)) => se.to_string()` branch at render.rs:266.
/// Verifies that effort alone is rendered without a leading "/" or "session:" label.
#[ test ]
fn cc_effort_only_footer_shows_effort_without_model()
{
  // 3 accounts: cur + 2 non-current → 2-line footer.
  let mk = | name : &str, cur : bool |
  {
    AccountQuota
    {
      fallback_reason : None,
      touched_at_secs : None,
      name                  : name.to_string(),
      is_current            : cur,
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
      claim_lock            : false,
      reserve               : false,
      inference_provider    : String::new(),
      tags : Vec::new(),
          org_created_at : None,
    }
  };
  let accounts = vec![ mk( "cur@x.com", true ), mk( "a@x.com", false ), mk( "b@x.com", false ) ];
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, Some( "high" ), None, None, false, &TagFilter::default() );
  // Footer Current line col3 must contain "high" (effort only, no model prefix).
  // The Next line legitimately shows "sonnet/high" (Feature 062, AC-03) — scope
  // the no-slash check to the Current line only, not the full output.
  let current_line = output.lines().find( |l| l.trim_start().starts_with( "Current" ) )
    .unwrap_or( "" );
  assert!(
    current_line.contains( "high" ),
    "CC-08: effort-only footer Current line must contain effort level 'high'; got:\n{output}",
  );
  assert!(
    !current_line.contains( "/high" ),
    "CC-08: effort-only footer Current line must not have model prefix '/high'; got:\n{output}",
  );
}

// ── Footer Next effort display: FT-05..FT-08 (Feature 062) ──────────────────

/// FT-05 — Footer Next line shows `sonnet/high` from model-derived effort when Sonnet available.
///
/// After TSK-335, `rec_display` is always `{rec_model}/{rec_effort}` where `rec_effort` is
/// model-derived (`"high"` for Sonnet, `"max"` for Opus). The `session_effort` param is now
/// irrelevant for the Next line — it only governs the Current line's `model_effort` display.
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` FT-05]
#[ test ]
fn ft05_footer_next_shows_model_and_effort_when_set()
{
  let mut cur = mk_aq_sort( "cur@x.com", 50.0, FAR_FUTURE_MS );
  cur.is_current = true;
  let rec = mk_aq_sort_weekly( "aaa@x.com", 50.0, 50.0, 80.0 );  // 20% Sonnet left → sonnet
  let accounts = vec![ cur, rec ];
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, Some( "high" ), None, None, false, &TagFilter::default() );
  assert!(
    output.contains( "sonnet/high" ),
    "FT-05: footer Next must contain 'sonnet/high' when session_effort=Some(\"high\") and Sonnet available; got:\n{output}",
  );
}

/// FT-07 — Footer Next line shows `opus/max` when Sonnet exhausted (model-derived effort).
///
/// After TSK-335, `rec_effort = "max"` for Opus is computed inside `render.rs`, not from
/// `session_effort`. The `session_effort = Some("max")` param passed here only affects the
/// Current line display; the Next line value is purely model-derived.
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` FT-07]
#[ test ]
fn ft07_footer_next_shows_opus_and_effort_when_sonnet_exhausted()
{
  let mut cur = mk_aq_sort( "cur@x.com", 50.0, FAR_FUTURE_MS );
  cur.is_current = true;
  let rec = mk_aq_sort_weekly( "aaa@x.com", 50.0, 50.0, 91.0 );  // 9% Sonnet left → opus
  let accounts = vec![ cur, rec ];
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, Some( "max" ), None, None, false, &TagFilter::default() );
  assert!(
    output.contains( "opus/max" ),
    "FT-07: footer Next must contain 'opus/max' when Sonnet exhausted and session_effort=Some(\"max\"); got:\n{output}",
  );
}

/// FT-08 — Column alignment: third `·` at same char position in Current and Next lines.
///
/// After TSK-335, `rec_display` always includes `/{effort}`, so:
/// `model_effort` = "s" (1 char); `rec_display` = "sonnet/high" (11 chars, model-derived).
/// `col3_w` = max(1, 11) = 11 → Current col3 padded to 11; Next col3 is 11 — third `·` aligns.
///
/// Spec: [`tests/docs/feature/62_unified_session_config.md` FT-08]
#[ test ]
fn ft08_footer_column_alignment_third_dot()
{
  let mut cur = mk_aq_sort( "cur@x.com", 50.0, FAR_FUTURE_MS );
  cur.is_current = true;
  // rec has Sonnet available (20% left) → rec_display = "sonnet/high" (11 chars, model-derived).
  // session_model = "s" (1 char) → model_effort = "s"; col3_w = max(1, 11) = 11.
  let rec = mk_aq_sort_weekly( "aaa@x.com", 50.0, 50.0, 80.0 );
  let accounts = vec![ cur, rec ];
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), Some( "s" ), None, None, None, false, &TagFilter::default() );
  let footer_lines : Vec< &str > = output.lines()
    .filter( |l| l.contains( '·' ) )
    .collect();
  assert!(
    footer_lines.len() >= 2,
    "FT-08: must have ≥2 footer lines with ·; got:\n{output}",
  );
  let cur_line  = footer_lines[ footer_lines.len() - 2 ];
  let next_line = footer_lines[ footer_lines.len() - 1 ];
  let third_dot_char_pos = |line : &str| -> Option< usize >
  {
    let mut count = 0usize;
    for ( i, ch ) in line.chars().enumerate()
    {
      if ch == '·' { count += 1; if count == 3 { return Some( i ); } }
    }
    None
  };
  let cur_pos  = third_dot_char_pos( cur_line );
  let next_pos = third_dot_char_pos( next_line );
  assert_eq!(
    cur_pos, next_pos,
    "FT-08: third · must be at same char position in Current and Next lines;\n  cur:  '{cur_line}'\n  next: '{next_line}'",
  );
}

// ── FT-20: model-derived effort on Next line ─────────────────────────────

/// FT-20 — Footer Next line always shows `{model}/{effort}` even when `session_effort = None`.
///
/// # Root Cause
/// `rec_display` was built with `match session_effort { Some(se) => model + "/" + se, None =>
/// model }`. When the caller passed `session_effort = None` (e.g., live monitor mode or no prior
/// effort in settings.json), the Next line showed only `"sonnet"` with no slash. The recommended
/// account's effort level was invisible regardless of what model it would receive (TSK-335 H3).
///
/// # Why Not Caught
/// FT-05 and FT-07 both passed `session_effort = Some(...)` — the carry-forward path.
/// FT-06 explicitly verified the `None` → no-slash behavior as correct. When the behavior
/// was identified as a bug (effort should be model-derived, not carried), FT-06 had to be deleted.
///
/// # Fix Applied
/// Replaced `match session_effort { ... }` with:
/// `let rec_effort = if rec_model == "opus" { "max" } else { "high" };`
/// `let rec_display = rec_model.to_string() + "/" + rec_effort;`
/// Now `rec_display` always includes `/effort`; `session_effort` is irrelevant for Next line.
///
/// # Prevention
/// This test calls `render_text` with `session_effort = None` and asserts the Next line contains
/// `"sonnet/high"` — verifying model-derived effort even when no effort is passed in.
///
/// # Pitfall
/// `session_effort` still governs the CURRENT line's `model_effort` display. The fix only
/// affects `rec_display` (Next line). Do not confuse the two uses.
#[ test ]
fn ft20_next_line_always_shows_effort_without_session_effort()
{
  let mut cur = mk_aq_sort( "cur@x.com", 50.0, FAR_FUTURE_MS );
  cur.is_current = true;
  // rec has 20% Sonnet left → rec_model = "sonnet"; rec_effort must be "high".
  let rec = mk_aq_sort_weekly( "aaa@x.com", 50.0, 50.0, 80.0 );
  let accounts = vec![ cur, rec ];
  // session_effort = None — no carry-forward; model-derived effort must appear anyway.
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false, &TagFilter::default() );
  let next_line = output.lines()
    .find( |l| l.trim_start().starts_with( "Next" ) )
    .unwrap_or( "" );
  assert!(
    next_line.contains( "sonnet/high" ),
    "FT-20: Next line must contain 'sonnet/high' even when session_effort=None (model-derived); got:\n{next_line:?}",
  );
}

// ── BUG-320 reproducer ────────────────────────────────────────────────────

/// BUG-320 reproducer — `render_text(gate_ownership=true)` skips non-owned accounts
/// in the footer Next recommendation.
///
/// # Root Cause
/// `render.rs` hardcoded `gate_ownership=false` when calling `find_next_for_strategy`.
/// Auto-switch used `gate_ownership = params.rotate && !params.force`, so the footer
/// could recommend a non-owned account that auto-switch would reject, violating
/// Feature 038 AC-10 ("recommended == switched-to").
///
/// # Why Not Caught
/// `render_text` had no `gate_ownership` param; all callers implicitly passed `false`.
/// No test exercised the ownership-gated recommendation path.
///
/// # Fix Applied
/// Added `gate_ownership: bool` as the 10th param to `render_text` and `render_plain`.
/// `api.rs` passes `params.rotate && !params.force`; display-only callers pass `false`.
///
/// # Prevention
/// Footer Next tests that involve rotate mode must cover both
/// `gate_ownership=false` (non-owned eligible) and `gate_ownership=true` (non-owned skipped).
///
/// # Pitfall
/// `mk_aq_sort()` defaults to `is_owned=true`; set `is_owned=false` explicitly to create
/// a non-owned account. Also: non-owned accounts still appear in table rows — only the
/// footer Next line is affected by the gate.
#[ doc = "bug_reproducer(BUG-320)" ]
#[ test ]
fn mre_bug320_footer_excludes_non_owned_when_rotate_force_0()
{
  // "aaa" prefix → sorts first under SortStrategy::Name; lower utilisation → eligible.
  let mut non_owned = mk_aq_sort( "aaa_nonowned@x.com", 10.0, FAR_FUTURE_MS );
  non_owned.is_owned = false;
  let owned = mk_aq_sort( "bbb_owned@x.com", 20.0, FAR_FUTURE_MS );
  let mut cur = mk_aq_sort( "cur@x.com", 50.0, FAR_FUTURE_MS );
  cur.is_current = true;
  let accounts = vec![ cur, non_owned, owned ];

  // Without gate: non-owned "aaa" wins (sorts first by name, fully eligible).
  let without_gate = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false, &TagFilter::default() );
  let next_without = without_gate.lines()
    .find( |l| l.trim_start().starts_with( "Next" ) )
    .unwrap_or( "" );
  assert!(
    next_without.contains( "aaa_nonowned" ),
    "BUG-320 (control): gate_ownership=false must recommend the non-owned account that sorts first;\n  next: {next_without:?}",
  );

  // With gate (fix): non-owned "aaa" skipped → owned "bbb" recommended.
  let with_gate = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, true, &TagFilter::default() );
  let next_with = with_gate.lines()
    .find( |l| l.trim_start().starts_with( "Next" ) )
    .unwrap_or( "" );
  assert!(
    next_with.contains( "bbb_owned" ),
    "BUG-320: gate_ownership=true must recommend the owned account in footer Next;\n  next: {next_with:?}",
  );
  assert!(
    !next_with.contains( "aaa_nonowned" ),
    "BUG-320: gate_ownership=true must not show non-owned account in footer Next;\n  next: {next_with:?}",
  );
}

// ── BUG-488: touched-now display signal in the 5h Reset column ────────────

/// BUG-488 render: a just-touched account whose re-fetch still reports the 5h window
/// idle renders distinguishably in the `5h Reset` cell — not the idle `—`. The cell's
/// *value* is BUG-551's concern (a `~in Xh Ym` projection, once the opaque `(touched)`
/// literal was replaced); what this test pins is only that the two states differ at all.
///
/// # Root Cause
///
/// The quota endpoint lags session starts; `apply_touch`'s single AC-03 re-fetch races
/// that lag and can lose, leaving `five_hour.resets_at=None` for an account whose touch
/// subprocess just succeeded. Render had no signal distinguishing that from a
/// never-touched idle account, so the table showed `5h Reset —` / 100% for all 13
/// just-touched accounts (2026-08-16 09:11 sync incident).
///
/// # Why Not Caught
///
/// No render test built an `AccountQuota` in the touched-but-lagging state — the field
/// carrying that state did not exist before Fix(BUG-488).
///
/// # Fix Applied
///
/// Fix(BUG-488): `apply_touch` records the touch on the row after a successful touch
/// subprocess; `render_text` overrides `cells[ 1 ]` when a touch is on record and
/// `five_hour.resets_at.is_none()`, so the row cannot read as never-touched.
///
/// Fix(BUG-551): the override's *value* changed from the opaque literal `"(touched)"` to
/// the projected countdown `"~in Xh Ym"`. This test asserts BUG-488's own requirement —
/// distinguishability from a never-touched idle row — rather than the literal, because
/// pinning the literal is exactly what let BUG-551 hide here for so long.
///
/// # Prevention
///
/// Any state the render layer must distinguish needs a field on `AccountQuota` — a
/// successful side effect that leaves API-visible state unchanged is otherwise invisible.
/// Assert the requirement the field exists to serve, never the specific string the
/// implementation happened to choose.
///
/// # Pitfall
///
/// Display-only: the override never fabricates a `resets_at` into the data itself, so
/// sort/recommendation logic still sees the account as idle. The control account below
/// proves a row with no touch on record keeps the plain `—`.
#[ doc = "bug_reproducer(BUG-488)" ]
#[ test ]
fn test_bug488_touched_row_distinguishable_from_never_touched_idle()
{
  use claude_profile::usage::test_bridge::mk_aq_ok;

  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH ).unwrap().as_secs();

  // Touched account: subprocess succeeded this invocation, endpoint still reports idle.
  let mut touched = mk_aq_ok( 0.0 );
  touched.name = "touched@x.com".to_string();
  touched.touched_at_secs = Some( now );

  // Control: identical idle state, no touch on record.
  let mut idle = mk_aq_ok( 0.0 );
  idle.name = "idle@x.com".to_string();

  let accounts = vec![ touched, idle ];
  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false, &TagFilter::default() );

  let touched_line = text.lines().find( |l| l.contains( "touched@x.com" ) )
    .expect( "touched account row must be present" );
  let idle_line = text.lines().find( |l| l.contains( "idle@x.com" ) )
    .expect( "idle control row must be present" );

  assert!(
    touched_line != idle_line,
    "BUG-488: a row with a touch on record must not render identically to a never-touched \
     idle row;\n  touched: {touched_line:?}\n  idle:    {idle_line:?}",
  );
  assert!(
    idle_line.contains( '\u{2014}' ),
    "BUG-488 (control): a row with no touch on record must keep the idle em-dash;\n  row: {idle_line:?}",
  );
}

/// BUG-551: the `5h Reset` cell rendered the opaque literal `(touched)` on a row whose
/// window end is exactly derivable, withholding the one value the column exists to show.
///
/// # Root Cause
///
/// `AccountQuota` carried the touch as a bare `bool`. `derive_touched_recently` parsed
/// `last_touch_at` into Unix seconds, used it for the grace comparison, then discarded it,
/// so the render layer received only the verdict "a touch happened" and had no instant to
/// project a window end from. `quota_text_cells`'s `reset_cell` accepts only an ISO string,
/// so no projection could enter through the normal path either.
///
/// # Why Not Caught
///
/// The BUG-488 render test asserted the literal `"(touched)"` as its expected output,
/// pinning the placeholder as the specification — a test whose expected value is a string
/// the implementation invented cannot detect that the string was the wrong answer.
///
/// # Fix Applied
///
/// Fix(BUG-551): `touched_recently : bool` became `touched_at_secs : Option<u64>` carrying
/// the parsed instant; `format::projected_reset_label` renders
/// `floor10(touch) + WINDOW_5H_S` as `"~in Xh Ym"`, the `~` marking it derived per the
/// convention `renews_label` established for `~Renews`.
///
/// # Prevention
///
/// When a predicate parses a rich value and returns only a boolean verdict, every consumer
/// past that boundary is permanently limited to expressing the verdict. Return the parsed
/// value alongside it.
///
/// # Pitfall
///
/// The projection floors to the 10-minute boundary Anthropic snaps windows to — validated
/// against 19 live accounts, where the unfloored `touch + 5h` matched none of them.
/// Display-only: never written back into `five_hour.resets_at`.
#[ doc = "bug_reproducer(BUG-551)" ]
#[ test ]
fn test_mre_bug551_touched_row_renders_projected_countdown_not_placeholder()
{
  use claude_profile::usage::test_bridge::mk_aq_ok;

  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH ).unwrap().as_secs();

  // Touched 10 minutes ago; the endpoint still reports no window.
  let mut touched = mk_aq_ok( 0.0 );
  touched.name = "touched@x.com".to_string();
  touched.touched_at_secs = Some( now - 600 );

  let accounts = vec![ touched ];
  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false, &TagFilter::default() );

  let row = text.lines().find( |l| l.contains( "touched@x.com" ) )
    .expect( "touched account row must be present" );

  assert!(
    !row.contains( "(touched)" ),
    "BUG-551: the opaque placeholder must not survive where the window end is derivable;\n  row: {row:?}",
  );
  assert!(
    row.contains( "~in " ),
    "BUG-551: a touched row with no endpoint-reported window must render the projected \
     countdown '~in Xh Ym' in 5h Reset;\n  row: {row:?}",
  );
  // A touch 10 min ago projects a window end ~4h50m out — the countdown must name hours,
  // not collapse to a bare minute/second value that would signal an imminent reset.
  assert!(
    row.contains( "~in 4h" ),
    "BUG-551: projection must be floor10(touch) + 5h, leaving ~4h50m after a 10-minute-old \
     touch;\n  row: {row:?}",
  );
}

/// BUG-551 (surface consistency): `get::5h_reset` must project the same window end the
/// text table shows for the same row — the two surfaces previously disagreed.
///
/// # Root Cause
/// `extract_get_field` carries its own `reset_cell` closure and never saw the touched-row
/// branch `render_text` applies after `quota_text_cells`, so a row rendering `(touched)` in
/// the table extracted as the bare em-dash. `render_json` / `render_tsv` had the same gap.
///
/// # Why Not Caught
/// Every `GetField::FiveHourReset` assertion used a row with an endpoint-reported
/// `resets_at`, where both paths agree trivially; none used the touched-with-no-window row.
///
/// # Fix Applied
/// Fix(BUG-551): the `FiveHourReset` arm routes through `projected_reset_label` when the
/// endpoint reports no window and a corroborated touch is on record, so both surfaces
/// project from the same anchor.
///
/// # Prevention
/// A per-surface closure duplicating a display rule is how one surface silently misses an
/// addition to it — the same shape as BUG-540's Sub/renews extractor divergence.
///
/// # Pitfall
/// A row with an endpoint-reported `resets_at` must keep the plain `in Xh Ym` with no `~`
/// on either surface — the projection applies only where the endpoint reported nothing.
#[ doc = "bug_reproducer(BUG-551)" ]
#[ test ]
fn test_bug551_get_field_and_table_agree_on_touched_row()
{
  use claude_profile::usage::test_bridge::{ mk_aq_ok, extract_get_field, projected_reset_label };
  use claude_profile::usage::test_bridge::types::GetField;

  let now_unix = || std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH ).unwrap().as_secs();

  let now        = now_unix();
  let touched_at = now - 600;
  let mut touched = mk_aq_ok( 0.0 );
  touched.name = "touched@x.com".to_string();
  touched.touched_at_secs = Some( touched_at );

  let field = extract_get_field( &touched, GetField::FiveHourReset, now );
  assert!(
    field.starts_with( "~in " ),
    "BUG-551: get::5h_reset must project the window end for a touched row, not fall back \
     to the em-dash the table no longer shows; got {field:?}",
  );

  // `render_text` samples its own clock with no injection point (render.rs:73), so the
  // table's countdown is computed at some instant strictly after `now` — and the rendered
  // minute ticks down as that gap grows. Comparing the table against a single `now`-derived
  // string is therefore a race that fails whenever the two reads straddle a minute boundary.
  // Bracket render's clock instead: `projected_reset_label` is monotonic in `now_secs`, so
  // the cell must equal the label for some instant in [before, after]. Both ends collapse to
  // the same string in the common case and straddle exactly one boundary otherwise.
  let accounts = vec![ touched ];
  let before   = now_unix();
  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false, &TagFilter::default() );
  let after = now_unix();

  let at_before = projected_reset_label( touched_at, before );
  let at_after  = projected_reset_label( touched_at, after );

  let row = text.lines().find( |l| l.contains( "touched@x.com" ) ).expect( "row present" );
  assert!(
    row.contains( &at_before ) || row.contains( &at_after ),
    "BUG-551: the table cell and get::5h_reset must be the same projected value;\n  \
     get: {field:?}\n  table candidates: {at_before:?} | {at_after:?}\n  row: {row:?}",
  );

  // Control: an endpoint-reported window keeps the plain countdown on both surfaces.
  let mut reported = mk_aq_ok( 0.0 );
  reported.touched_at_secs = Some( now - 600 );
  if let Ok( ref mut data ) = reported.result
  {
    if let Some( ref mut p ) = data.five_hour
    {
      p.resets_at = Some( "2099-01-01T00:00:00Z".to_string() );
    }
  }
  let reported_field = extract_get_field( &reported, GetField::FiveHourReset, now );
  assert!(
    reported_field.starts_with( "in " ),
    "BUG-551 (control): an endpoint-reported reset must stay exact, never marked `~`; \
     got {reported_field:?}",
  );
}

/// FT-29/553 — `bug_reproducer(BUG-553)` — one cached fixture rendered through all four
/// surfaces must agree on every quota cell.
///
/// # Root Cause
/// `quota_text_cells( data, now_secs )` takes only the raw usage data and the clock, so any
/// display rule depending on the *account* — cache staleness (`aq.cached`), BUG-551's touch
/// projection (`aq.touched_at_secs`) — could not live inside it and had to be re-applied by
/// each caller afterward. Only `render_text` applied them. `render_tsv` referenced `aq.cached`
/// nowhere in its Ok arm and `extract_get_field` carried its own `pct_bare`/`reset_cell`
/// closures, so both rendered a cache-fallback row as if it were live.
///
/// # Why Not Caught
/// Every pre-existing per-surface test asserted a surface against a *literal* expectation,
/// never against another surface: "TSV shows `88%`" passes whether or not the text table
/// shows `~88%` for that same account. No test constructed a `cached = true` fixture and
/// rendered it through TSV or `get::` at all.
///
/// # Fix Applied
/// `format.rs` gained `quota_cells_for( aq, data, now_secs, style )` — the aq-aware layer that
/// applies every account-dependent rule once, mirroring how `expires_cell_for` supersedes
/// `compute_expires_cell` (BUG-345) and `renews_cell_for` supersedes the three inline renews
/// copies (BUG-540). All three display surfaces now take their cells from it; `PctStyle`
/// carries the one legitimate difference between them (emoji prefix vs bare number).
///
/// # Prevention
/// Assert surfaces against **each other**, not against literals — as this test does. A rule
/// applied after a shared helper returns exists on exactly the surfaces someone remembered,
/// and the count of forgotten surfaces grows with every rule added. The percentage cells are
/// clock-independent, so cross-surface equality on them is exact and race-free even though
/// each renderer samples its own `SystemTime::now()`.
///
/// # Pitfall
/// Widen the shared helper's input until the account-dependent rule fits inside it; never
/// re-apply the rule per call site. Taking `aq` as the parameter is what makes skipping a
/// rule a compile-time impossibility rather than an omission nobody notices.
#[ doc = "bug_reproducer(BUG-553)" ]
#[ test ]
fn test_bug553_all_surfaces_agree_on_cached_row()
{
  use claude_profile::usage::test_bridge::{ render_tsv, render_json, extract_get_field, mk_aq_ok_both };
  use claude_profile::usage::test_bridge::types::GetField;

  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH ).unwrap().as_secs();

  // 12% used → 88% left on the 5h window; 4% used → 96% left on the weekly.
  let mut aq = mk_aq_ok_both( 12.0, 4.0 );
  aq.name   = "cached@x.com".to_string();
  aq.cached = true;
  aq.cache_age_secs = Some( 7200 );

  let accounts = vec![ aq ];
  // Both surfaces read the *same* account object — no chance of a fixture drifting apart.
  let aq       = &accounts[ 0 ];
  let cols     = ColsVisibility::default_set();

  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols,
    None, None, None, None, false, &TagFilter::default() );
  let tsv  = render_tsv( &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols );

  // S1 — TSV must disclose staleness. It is the one surface with no `cached` column of its
  // own (JSON emits `cached`/`cache_age_secs` outright), so the `~` prefix is its *only*
  // staleness signal; without it a watchdog cannot tell a live reading from a cached one.
  let tsv_fields = |header : &str| -> String
  {
    let mut lines = tsv.lines();
    let heads : Vec< &str > = lines.next().expect( "TSV header" ).split( '\t' ).collect();
    let vals  : Vec< &str > = lines.next().expect( "TSV data row" ).split( '\t' ).collect();
    let idx = heads.iter().position( |h| *h == header )
      .unwrap_or_else( || panic!( "TSV column {header:?} must exist; headers: {heads:?}" ) );
    ( *vals.get( idx ).unwrap_or( &"" ) ).to_string()
  };

  // The reset columns are pinned here to lock in the dash exemption, not an oversight: this
  // fixture carries `resets_at: None` on both windows, and `prefix_tilde` deliberately leaves a
  // bare `—` unprefixed. `~` qualifies how fresh a *value* is, and `—` is the absence of one —
  // `~—` would assert staleness about nothing. Asserting the exemption explicitly is what keeps
  // a later "prefix every cell uniformly" simplification from silently introducing `~—`.
  for ( col, expect ) in
  [
    ( "5h_left", "~88%" ), ( "7d_left", "~96%" ),
    ( "5h_reset", "\u{2014}" ), ( "7d_reset", "\u{2014}" ),
  ]
  {
    let got = tsv_fields( col );
    assert_eq!(
      got, expect,
      "BUG-553 S1: TSV {col} must carry the `~` cache-staleness prefix the text table shows — \
       TSV has no `cached` column, so this is its only staleness signal; got {got:?}\n{tsv}",
    );
  }

  // S2 — `get::` must equal the table cell, which is its own documented contract.
  for ( field, expect ) in
  [
    ( GetField::FiveHourLeft, "~88%" ),
    ( GetField::SevenDayLeft, "~96%" ),
    ( GetField::FiveHourReset, "\u{2014}" ),
    ( GetField::SevenDayReset, "\u{2014}" ),
  ]
  {
    let got = extract_get_field( aq, field, now );
    assert_eq!(
      got, expect,
      "BUG-553 S2: extract_get_field documents itself as returning the same value as the \
       corresponding table cell; for a cached row it dropped the `~`. got {got:?}",
    );
  }

  // S2, reset half: the loop above pins the dash exemption, so it cannot show that staleness
  // reaches a reset cell that *does* carry a value — the case S2 actually named. A second
  // cached fixture with a server-reported `resets_at` covers it. Assert only the `~` prefix,
  // not the whole string: reset cells render a countdown, and TSV samples its own clock while
  // `extract_get_field` is handed `now`, so an exact match would be a Fragile Test.
  let mut with_reset = mk_aq_ok_both( 12.0, 4.0 );
  with_reset.name           = "cached_reset@x.com".to_string();
  with_reset.cached         = true;
  with_reset.cache_age_secs = Some( 7200 );
  if let Ok( ref mut data ) = with_reset.result
  {
    if let Some( ref mut p ) = data.seven_day { p.resets_at = Some( "2099-01-01T00:00:00Z".to_string() ); }
  }
  let reset_accounts = vec![ with_reset ];
  let reset_aq       = &reset_accounts[ 0 ];
  let reset_tsv      = render_tsv( &reset_accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols );
  let reset_tsv_cell =
  {
    let mut lines = reset_tsv.lines();
    let heads : Vec< &str > = lines.next().expect( "TSV header" ).split( '\t' ).collect();
    let vals  : Vec< &str > = lines.next().expect( "TSV data row" ).split( '\t' ).collect();
    let idx = heads.iter().position( |h| *h == "7d_reset" ).expect( "7d_reset column" );
    ( *vals.get( idx ).unwrap_or( &"" ) ).to_string()
  };
  let reset_get = extract_get_field( reset_aq, GetField::SevenDayReset, now );
  for ( surface, got ) in [ ( "TSV 7d_reset", &reset_tsv_cell ), ( "get::7d_reset", &reset_get ) ]
  {
    assert!(
      got.starts_with( '~' ),
      "BUG-553 S2: {surface} must carry the `~` cache-staleness prefix on a reset cell that \
       holds a real countdown — this is the field S2 named; got {got:?}\n{reset_tsv}",
    );
  }

  // Cross-surface agreement: the same percentage string appears on every display surface,
  // differing only by the emoji `PctStyle::Emoji` attaches — `prefix_tilde` marks the whole
  // cell, so the text form is `~🟢 88%` against TSV's `~88%`. Percentages do not depend on
  // the clock, so this comparison is exact despite each renderer sampling its own `now`.
  for expect in [ "~🟢 88%", "~🟢 96%" ]
  {
    assert!(
      text.contains( expect ),
      "BUG-553: text table must show {expect:?} for the cached row; got:\n{text}",
    );
  }

  // JSON discloses staleness through its own `cached` field rather than a `~` prefix — the
  // numbers stay plain and parseable. Assert that contract explicitly so a future change
  // cannot quietly start emitting `~` into a numeric field.
  let json = render_json( &accounts );
  assert!(
    json.contains( "\"cached\":true" ) && json.contains( "\"session_5h_left_pct\":88" ),
    "BUG-553: JSON must keep quota numbers plain and disclose staleness via its own \
     `cached` field; got:\n{json}",
  );
  assert!(
    !json.contains( '~' ),
    "BUG-553: the `~` convention is display-only — JSON must never emit it into a numeric \
     field; got:\n{json}",
  );
}

/// FT-30/553 — `bug_reproducer(BUG-553)` — TSV and JSON must apply BUG-551's touch projection.
///
/// # Root Cause
/// BUG-551 added the projected `~in Xh Ym` countdown for a corroborated-touch row by patching
/// `render_text` and `extract_get_field` individually — the only two surfaces it was tested
/// against. `render_tsv` and `render_json` were never touched, so the same account answered
/// `—` / `null` there while the human table showed a concrete countdown.
///
/// # Why Not Caught
/// BUG-551's own regression test asserted exactly one field pair (`get::5h_reset` vs the table
/// cell) — the pair that bug touched. Nothing exercised the other two surfaces.
///
/// # Fix Applied
/// The projection moved inside `quota_cells_for`, which TSV now calls. JSON cannot carry the
/// `~` marker (its reset field is a number), so it follows the established
/// `renewal_secs`/`renewal_is_estimate` pairing and gained `session_5h_reset_is_estimate`.
///
/// # Prevention
/// A projected value must never be emitted indistinguishably from a server-reported one. On
/// display surfaces the `~` prefix carries that; on JSON it needs a companion boolean, and the
/// flag must be `null` exactly when the value is — "not an estimate" is a positive claim that
/// cannot be made about an absent reading.
///
/// # Pitfall
/// Adding a display rule to the surface where the bug was *reported* leaves it missing from
/// every other surface. Land the rule in the shared aq-aware helper instead.
#[ doc = "bug_reproducer(BUG-553)" ]
#[ test ]
fn test_bug553_tsv_and_json_project_touched_row()
{
  use claude_profile::usage::test_bridge::{ render_tsv, render_json, extract_get_field, mk_aq_ok };
  use claude_profile::usage::test_bridge::types::GetField;

  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH ).unwrap().as_secs();

  // Corroborated touch 10 minutes ago, endpoint still reporting the 5h window idle.
  let mut aq = mk_aq_ok( 0.0 );
  aq.name            = "touched@x.com".to_string();
  aq.touched_at_secs = Some( now - 600 );

  let accounts = vec![ aq ];
  // Both surfaces read the *same* account object — no chance of a fixture drifting apart.
  let aq       = &accounts[ 0 ];
  let cols     = ColsVisibility::default_set();

  let tsv    = render_tsv( &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols );
  let heads  : Vec< &str > = tsv.lines().next().expect( "TSV header" ).split( '\t' ).collect();
  let vals   : Vec< &str > = tsv.lines().nth( 1 ).expect( "TSV data row" ).split( '\t' ).collect();
  let idx    = heads.iter().position( |h| *h == "5h_reset" ).expect( "5h_reset column" );
  let cell   = *vals.get( idx ).unwrap_or( &"" );

  assert!(
    cell.starts_with( "~in " ),
    "BUG-553 S3: TSV 5h_reset must show the projected countdown for a corroborated-touch row, \
     not the em-dash the endpoint's lagged state would produce; got {cell:?}\n{tsv}",
  );

  // The projection must be the same value `get::` and the table already show.
  let field = extract_get_field( aq, GetField::FiveHourReset, now );
  assert!(
    field.starts_with( "~in " ),
    "BUG-553 S3: get::5h_reset must project the same window end; got {field:?}",
  );

  // JSON: a number plus its estimate flag, never a bare number indistinguishable from a
  // server-reported reset.
  let json = render_json( &accounts );
  assert!(
    json.contains( "\"session_5h_reset_is_estimate\":true" ),
    "BUG-553 S3: JSON must flag a projected reset as an estimate, mirroring \
     `renewal_secs`/`renewal_is_estimate`; got:\n{json}",
  );
  assert!(
    !json.contains( "\"session_5h_resets_in_secs\":null" ),
    "BUG-553 S3: JSON emitted null for a row the other surfaces project a countdown for; \
     got:\n{json}",
  );

  // Control: an endpoint-reported reset is not an estimate, and an absent one flags `null`
  // rather than falsely claiming "not an estimate" about a value that does not exist.
  let mut reported = mk_aq_ok( 0.0 );
  reported.name = "reported@x.com".to_string();
  if let Ok( ref mut data ) = reported.result
  {
    if let Some( ref mut p ) = data.five_hour { p.resets_at = Some( "2099-01-01T00:00:00Z".to_string() ); }
  }
  let reported_json = render_json( &[ reported ] );
  assert!(
    reported_json.contains( "\"session_5h_reset_is_estimate\":false" ),
    "BUG-553 (control): a server-reported reset must flag `false`; got:\n{reported_json}",
  );

  let absent_json = render_json( &[ mk_aq_ok( 0.0 ) ] );
  assert!(
    absent_json.contains( "\"session_5h_resets_in_secs\":null" )
      && absent_json.contains( "\"session_5h_reset_is_estimate\":null" ),
    "BUG-553 (control): the estimate flag must be null exactly when the value is — \
     `false` would assert something about a reading that does not exist; got:\n{absent_json}",
  );
}

/// FT-31/553 — `bug_reproducer(BUG-553)` — one percentage, one rounding, on every surface.
///
/// # Root Cause
/// Three independent percentage closures existed for one logical value. `quota_text_cells`'s
/// `pct_emoji` applied `.round()` (BUG-331's round-once doctrine); its sibling `pct_cell` for
/// `7d(Son)` did not; `render_tsv`'s local `pct_bare` did; `extract_get_field`'s local
/// `pct_bare` did not. Bare `{:.0}` formatting rounds half-to-even, `.round()` rounds
/// half-away — so at a `*.5` value the surfaces disagreed by a full percent.
///
/// # Why Not Caught
/// `tests/docs/algorithm/011_rounding_boundary_classification_hazards.md` covers the emoji
/// path's rounding thoroughly, but only within `quota_text_cells`. No test compared the
/// rounded value across surfaces, and no test exercised `7d(Son)` at a half-percent boundary.
///
/// # Fix Applied
/// A single `pct` closure inside `quota_data_cells` rounds once for every cell and every
/// surface; `PctStyle` selects only whether the emoji prefix is attached.
///
/// # Prevention
/// Round once, in the shared closure — never in a format string, and never in a per-surface
/// copy. A `*.5` input is the only value that separates the two modes, so it is the only
/// input that can detect the divergence: `11.5` used → `88.5` left renders `88` under
/// half-to-even and `89` under half-away.
///
/// # Pitfall
/// A duplicated formatting closure is a latent divergence even when both copies look correct
/// today — the next doctrine change lands in whichever copies someone finds.
#[ doc = "bug_reproducer(BUG-553)" ]
#[ test ]
fn test_bug553_one_rounding_across_surfaces()
{
  use claude_profile::usage::test_bridge::{ render_tsv, extract_get_field, mk_aq_ok_both };
  use claude_profile::usage::test_bridge::types::GetField;

  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH ).unwrap().as_secs();

  // 11.5 used → 88.5 left. Half-away (.round()) → 89; half-to-even (bare `{:.0}`) → 88.
  let mut aq = mk_aq_ok_both( 11.5, 11.5 );
  aq.name = "half@x.com".to_string();
  if let Ok( ref mut data ) = aq.result
  {
    data.seven_day_sonnet = Some( claude_quota::PeriodUsage { utilization : 11.5, resets_at : None } );
  }

  let accounts = vec![ aq ];
  // Both surfaces read the *same* account object — no chance of a fixture drifting apart.
  let aq       = &accounts[ 0 ];
  // `7d_son` is off by default (BUG-334: the upstream field has been universally None since
  // the 2026-06-25 API restructuring) — enable it explicitly, since its unrounded `pct_cell`
  // was one of the three disagreeing copies this test exists to collapse.
  let mut cols = ColsVisibility::default_set();
  cols.apply_modifier( "+7d_son" ).expect( "7d_son is a valid cols modifier" );
  let tsv      = render_tsv( &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols );
  let heads    : Vec< &str > = tsv.lines().next().expect( "TSV header" ).split( '\t' ).collect();
  let vals     : Vec< &str > = tsv.lines().nth( 1 ).expect( "TSV data row" ).split( '\t' ).collect();

  for ( col, field ) in
  [
    ( "5h_left", GetField::FiveHourLeft ),
    ( "7d_left", GetField::SevenDayLeft ),
    ( "7d_son",  GetField::SevenDaySon  ),
  ]
  {
    let idx      = heads.iter().position( |h| *h == col ).expect( "column present" );
    let tsv_cell = *vals.get( idx ).unwrap_or( &"" );
    let get_cell = extract_get_field( aq, field, now );

    assert_eq!(
      tsv_cell, "89%",
      "BUG-553 S4: TSV {col} must round half-away (89%), matching BUG-331's round-once \
       doctrine; got {tsv_cell:?}",
    );
    assert_eq!(
      get_cell, tsv_cell,
      "BUG-553 S4: get::{col} and TSV {col} rendered one value through two closures and \
       disagreed; got {get_cell:?} vs {tsv_cell:?}",
    );
  }

  // The text table carries the same number behind its emoji prefix.
  let text = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any, &cols,
    None, None, None, None, false, &TagFilter::default() );
  assert!(
    text.contains( "89%" ) && !text.contains( "88%" ),
    "BUG-553 S4: the text table must show the same rounded percentage as TSV and `get::`; \
     got:\n{text}",
  );
}

// Integration tests for render.rs — Part B (split from src/usage/render_tests.rs).
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

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
      },
      AccountQuota
      {
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
      },
      AccountQuota
      {
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
      },
    ]
  }

  // Scenario 1 — both session_model and session_effort supplied.
  // Footer line 1: `Current · cur@x.com · claude-sonnet-4-6/low · N/M`
  {
    let accounts = make_accounts();
    let output = render_text(
      &accounts, SortStrategy::Renew, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), Some( "claude-sonnet-4-6" ), Some( "low" ), None, None, false,
    );
    assert!(
      output.contains( "claude-sonnet-4-6/low" ),
      "FT-29 s1: footer Current line col3 must be 'claude-sonnet-4-6/low'; got:\n{output}",
    );
    assert!(
      output.contains( "Current" ),
      "FT-29 s1: footer must have Current line; got:\n{output}",
    );
  }

  // Scenario 2 — session_model only; effort must be absent.
  // Footer line 1: `Current · cur@x.com · claude-sonnet-4-6 · N/M` (no slash)
  {
    let accounts = make_accounts();
    let output = render_text(
      &accounts, SortStrategy::Renew, None, PreferStrategy::Any,
      &ColsVisibility::default_set(), Some( "claude-sonnet-4-6" ), None, None, None, false,
    );
    assert!(
      output.contains( "claude-sonnet-4-6" ),
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
      &ColsVisibility::default_set(), None, None, None, None, false,
    );
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
    &cols, None, None, Some( spath ), None, false,
  );

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
    &cols, None, None, Some( spath ), None, false,
  );

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
      &ColsVisibility::default_set(), None, None, Some( spath ), Some( true ), false,
    );
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
      &ColsVisibility::default_set(), None, None, Some( spath ), Some( false ), false,
    );
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
/// Previous version hardcoded `_active_w003_user1` and `_active_w004_user2` as
/// "other machine" marker filenames. On the test machine (hostname=w003, user=user1),
/// `active_marker_filename()` returns `_active_w003_user1` — the same name as the
/// hardcoded "other" marker. The second `fs::write` overwrote the own marker content
/// (`"own@test.com"` → `"alice@test.com"`), so `build_sessions_table` showed
/// `alice@test.com ✓` instead of `own@test.com ✓`.
///
/// # Why Not Caught
/// Test was written and validated on a machine where `active_marker_filename()` did not
/// collide with `_active_w003_user1`. The fragility is machine-specific and silent —
/// the test panics with a misleading message rather than a setup-collision error.
///
/// # Fix Applied
/// Fix(BUG-308): replaced hardcoded `_active_w003_user1` / `_active_w004_user2` with
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
/// both vary across machines. Never hardcode expected identities like `user1@w003`
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
  // to have. Fix(BUG-308): previous hardcoded `_active_w003_user1` overwrote the own marker
  // on machines where hostname=w003, user=user1 (same name as `active_marker_filename()`).
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
    &cols, None, None, Some( spath ), None, false,
  );

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
    &ColsVisibility::default_set(), None, None, Some( spath ), Some( true ), false,
  );

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
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
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
    }
  };
  let accounts = vec![ mk( "a@x.com" ), mk( "b@x.com" ) ];
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
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
    }
  };
  let accounts = vec![ mk( "cur@x.com", true ), mk( "a@x.com", false ), mk( "b@x.com", false ) ];
  let output = render_text(
    &accounts, SortStrategy::Name, None, PreferStrategy::Any,
    &ColsVisibility::default_set(), None, Some( "high" ), None, None, false,
  );
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
    &ColsVisibility::default_set(), None, Some( "high" ), None, None, false,
  );
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
    &ColsVisibility::default_set(), None, Some( "max" ), None, None, false,
  );
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
    &ColsVisibility::default_set(), Some( "s" ), None, None, None, false,
  );
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
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
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
    &ColsVisibility::default_set(), None, None, None, None, false,
  );
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
    &ColsVisibility::default_set(), None, None, None, None, true,
  );
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

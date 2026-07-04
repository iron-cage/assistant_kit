// Integration tests for api.rs — Part A (split from src/usage/api_tests.rs).
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::{ pre_switch_touch_ctx, apply_model_override, PreSwitchOutcome };
use tempfile::TempDir;

/// Structural test: `pre_switch_touch_ctx` with an invalid credential file (no accessToken).
///
/// # Root Cause (BUG-210)
/// `pre_switch_touch_ctx()` returned `None` in the already-active branch without emitting
/// `model:` + `effort:` trace lines. Model/effort resolution lived in `apply_post_switch_touch`
/// only, which is never called on the skip path.
///
/// # Why Not Caught
/// No test called `pre_switch_touch_ctx` directly with a failing credential read; the only
/// coverage was via the live `aw29` integration test (`lim_it`, excluded from CI).
///
/// # Fix Applied
/// Extended `pre_switch_touch_ctx` to accept `imodel_str` and `effort_str` parameters;
/// added model/effort resolution and trace emission in the already-active branch.
///
/// # Prevention
/// This test acts as a compile-time guard: if the function reverts to 3 params, this
/// call (with 5 args) causes a compile error. Also verifies that the fetch-failed path does
/// NOT emit `model:` lines — the model/effort emission belongs only to quota-fetch-OK paths.
///
/// # Pitfall
/// The fetch-failed path must NOT emit `model:` even after the BUG-210 fix — model/effort
/// resolution requires quota data, which is unavailable when fetch fails.
#[ doc = "bug_reproducer(BUG-210)" ]
#[ test ]
fn test_pre_switch_touch_ctx_model_effort_absent_on_fetch_failure()
{
  let store = TempDir::new().unwrap();

  // Write a credential file with no accessToken — quota fetch will fail.
  std::fs::write(
    store.path().join( "noaccess@example.com.credentials.json" ),
    r#"{"subscriptionType":"pro"}"#,
  ).unwrap();

  // 5-arg signature — compile error before BUG-210 fix extends the function.
  let result = pre_switch_touch_ctx(
    "noaccess@example.com",
    store.path(),
    true,
    "auto",
    "auto",
  );

  // Fetch-failed path must return Unavailable (credential read OK but accessToken absent).
  assert!(
    matches!( result, PreSwitchOutcome::Unavailable ),
    "fetch-failed path must return Unavailable, got {result:?}",
  );
}

/// `mre_bug238` — `apply_model_override()` writes opus when 7d(Son) consumed > 80%.
///
/// # Root Cause
/// `pre_switch_touch_ctx()` originally returned `PreSwitchOutcome::AlreadyActive` for
/// accounts with an active 5h window, and `apply_post_switch_touch()` skipped
/// `apply_model_override()` in that branch — the model override never fired for
/// already-active accounts.
///
/// # Why Not Caught
/// No unit test for `apply_model_override()` existed. Full CLI path requires a live OAuth
/// token to fetch quota, which is impractical in CI.
///
/// # Fix Applied
/// BUG-238 fix: wired `apply_model_override` into the `AlreadyActive` branch.
/// BUG-285 follow-up: the `AlreadyActive` variant was subsequently removed entirely —
///   `pre_switch_touch_ctx` now always returns `NeedTouch` when the quota fetch succeeds
///   (the subprocess is idempotent). `apply_model_override` fires unconditionally for all
///   fetch-succeeded outcomes.
///
/// # Prevention
/// Test `apply_model_override` directly to stay independent of CLI plumbing. Any future
/// `PreSwitchOutcome` variant that carries `OauthUsageData` must wire in `apply_model_override`.
///
/// # Pitfall
/// `apply_model_override` only fires when `sonnet_left < 15.0`. Test input must push
/// `utilization` to > 85.0 (leaving < 15%). Also: the `.claude/` parent dir must exist
/// before the write or `override_session_model_to_opus` silently no-ops.
#[ doc = "bug_reproducer(BUG-238)" ]
#[ test ]
fn mre_bug238_model_override_fires_for_active_account()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  // Create ~/.claude/ so override_session_model_to_opus can write settings.json.
  std::fs::create_dir_all( paths.base() ).unwrap();

  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 91.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "account.use", "test-account" );

  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"opus\"" ) && !content.contains( "claude-opus-4-8" ),
    "apply_model_override must write opus shorthand to settings.json when 7d(Son) is 91% consumed (9% left), got: {content}",
  );
}

/// `mre_bug286` — full model ID `"claude-opus-4-8"` in settings.json must be
/// normalized to shorthand `"opus"` by `apply_model_override`.
///
/// # Root Cause
/// BUG-257 fix added `contains("sonnet")` for sonnet alias coverage, but did not
/// add the analogous full-ID opus → shorthand normalization path. When settings.json
/// contained `"claude-opus-4-8"` (written by a prior `set_session_model`), the gate
/// `contains("sonnet") || is_empty()` was false — override never fired.
///
/// # Why Not Caught
/// No test wrote a full-ID opus value to settings.json before calling
/// `apply_model_override`. All existing tests started from empty or shorthand values.
///
/// # Fix Applied
/// Added `current == "claude-opus-4-8"` arm to the gate condition in
/// `override_session_model_to_opus`.
///
/// # Prevention
/// This test pre-writes the full opus ID and asserts normalization to shorthand.
///
/// # Pitfall
/// Any path that writes model to settings.json must write Claude Code shorthand
/// ("opus"/"sonnet"), never full IDs ("claude-opus-4-8").
#[ doc = "bug_reproducer(BUG-286)" ]
#[ test ]
fn mre_bug286_full_opus_id_normalized_to_shorthand()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-write full opus ID — as stored in {name}.json snapshots from older convention.
  std::fs::write( paths.settings_file(), r#"{"model":"claude-opus-4-8"}"# ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 91.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "account.use", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"opus\"" ) && !content.contains( "claude-opus-4-8" ),
    "BUG-286: full-ID opus must be normalized to shorthand 'opus', got: {content}",
  );
}

/// `mre_bug300` — `apply_model_override()` fires unconditionally when `seven_day_sonnet = None`.
///
/// # Root Cause
/// `apply_model_override()` at `src/usage/api.rs:267` uses
/// `quota.seven_day_sonnet.as_ref().map_or(0.0, |p| 100.0 - p.utilization)`.
/// When `seven_day_sonnet = None`, `map_or` returns `0.0`. `0.0 < 20.0` is always true,
/// so the Opus override fires unconditionally for every account without a Sonnet tier.
///
/// # Why Not Caught
/// All existing tests supply `seven_day_sonnet: Some(PeriodUsage { utilization: 90.0, ... })`.
/// No test used `None` as input. The buggy `map_or(0.0, ...)` path was invisible in CI.
///
/// # Fix Applied
/// Replace `map_or(0.0, ...)` with `if let Some( ref sonnet ) = quota.seven_day_sonnet { ... }`.
/// When `None`, the entire override block is skipped.
/// `None` = "tier absent/unknown", not "quota fully exhausted".
///
/// # Prevention
/// This test catches any regression to `map_or(0.0, ...)` or other "treat None as 0%"
/// patterns. Observable contract: when `seven_day_sonnet` is absent, `"opus"` must NOT be
/// written — absent tier is unknown quota, not exhaustion.
///
/// # Pitfall
/// `map_or(0.0, ...)` is correct for display (show 0% when tier absent) but WRONG for
/// conditional gates. `None` = "tier absent/unknown". Always use `if let Some(...)` for
/// quota-exhaustion logic — never treat absence as exhaustion.
///
/// # Fix(BUG-311) behaviour change
/// BUG-311 added the else-branch: when tier is absent, `"sonnet"` is now written
/// conservatively. The BUG-300 invariant still holds — `"opus"` is never written for
/// absent tier. The new invariant: absent tier → model = `"sonnet"` (safe default).
#[ doc = "bug_reproducer(BUG-300)" ]
#[ test ]
fn mre_bug300_model_override_absent_sonnet_no_override()
{
  use claude_quota::OauthUsageData;
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // seven_day_sonnet = None: Sonnet tier absent — opus must NOT fire; sonnet written conservatively.
  let quota = OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap_or_default();
  assert!(
    content.contains( "\"sonnet\"" ),
    "BUG-300/BUG-311: absent Sonnet tier must write sonnet (conservative default), got: {content}",
  );
  assert!(
    !content.contains( "\"opus\"" ),
    "BUG-300: absent Sonnet tier must NOT write opus (absent tier ≠ exhausted), got: {content}",
  );
}

// ── BUG-244 tests: label param + usage_routine wiring ─────────────────────

#[ test ]
fn t01_model_override_fires_when_sonnet_below_threshold()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 91.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"opus\"" ) && !content.contains( "claude-opus-4-8" ),
    "must write opus shorthand when 7d(Son) utilization=91% (9% left), got: {content}",
  );
}

/// Fix(BUG-311): when Sonnet quota is sufficient, opus override skips and sonnet is
/// written as the recommended session model.
#[ test ]
fn t02_model_override_writes_sonnet_when_quota_sufficient()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 70.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap_or_default();
  assert!(
    content.contains( "\"sonnet\"" ),
    "Fix(BUG-311): 7d(Son) utilization=70% (30% left) must write sonnet as recommended model, got: {content}",
  );
  assert!(
    !content.contains( "\"opus\"" ),
    "7d(Son) 30% left: opus override must NOT fire, got: {content}",
  );
}

#[ test ]
fn t03_model_override_skips_when_already_opus()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-write settings.json with opus shorthand already set (production convention).
  std::fs::write( paths.settings_file(), r#"{"model":"opus"}"# ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 91.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"opus\"" ),
    "settings.json must still contain opus after call when already opus, got: {content}",
  );
  assert!(
    !content.contains( "sonnet" ),
    "must not downgrade to sonnet, got: {content}",
  );
}

#[ test ]
fn t04_model_override_skips_on_error_result()
{
  // Guard test: usage_routine must guard apply_model_override with result.is_ok(),
  // since OauthUsageData is unavailable for Err accounts.
  let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  let fn_start = src.find( "pub fn usage_routine" ).expect( "usage_routine not found" );
  let call_rel = src[ fn_start.. ]
    .find( "apply_model_override(" )
    .expect( "BUG-244: apply_model_override must be called in usage_routine" );
  // before_call: from fn_start to call site — ASCII boundaries, no multibyte risk.
  let before_call = &src[ fn_start .. fn_start + call_rel ];
  assert!(
    before_call.contains( "is_ok()" ) || before_call.contains( "Ok( ref " ),
    "apply_model_override call in usage_routine must be guarded by result.is_ok()",
  );
}

#[ test ]
fn t05_apply_model_override_absent_from_usage_routine_before_fix()
{
  let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  let fn_start = src.find( "pub fn usage_routine" ).expect( "usage_routine not found" );
  let body_end = {
    let after = &src[ fn_start + 1.. ];
    let a     = after.find( "\npub fn " ).unwrap_or( after.len() );
    let b     = after.find( "\n#[ cfg( t" ).unwrap_or( after.len() );
    let c     = after.find( "\n#[cfg(t" ).unwrap_or( after.len() );
    fn_start + 1 + a.min( b ).min( c )
  };
  let body       = &src[ fn_start..body_end ];
  let call_count = body.matches( "apply_model_override(" ).count();
  // BUG-244 fix: one call for the current-account usage path (guarded by is_current).
  // Feature 062 (AC-05): one call for the rotation-winner path (after switch_account).
  assert_eq!(
    call_count, 2,
    "apply_model_override must be called exactly twice in usage_routine body: \
    once for current-account (BUG-244) and once for rotation-winner (Feature 062 AC-05)",
  );
}

#[ test ]
fn t06_model_override_skips_for_non_current_account()
{
  // Guard test: usage_routine must only call apply_model_override for is_current accounts.
  let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  let fn_start = src.find( "pub fn usage_routine" ).expect( "usage_routine not found" );
  let call_rel = src[ fn_start.. ]
    .find( "apply_model_override(" )
    .expect( "BUG-244: apply_model_override must be called in usage_routine" );
  // before_call: from fn_start to call site — ASCII boundaries, no multibyte risk.
  let before_call = &src[ fn_start .. fn_start + call_rel ];
  assert!(
    before_call.contains( "is_current" ),
    "apply_model_override call in usage_routine must be guarded by is_current check",
  );
}

/// Fix(BUG-311): at the exact 10% boundary (left == 10%, not < 10%), opus override skips
/// and sonnet is written as recommended model.
#[ test ]
fn t07_model_override_writes_sonnet_at_10pct_boundary()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // utilization=90.0 → sonnet_left = 100.0 - 90.0 = 10.0; 10.0 < 10.0 == false → sonnet wins.
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 90.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap_or_default();
  assert!(
    content.contains( "\"sonnet\"" ),
    "Fix(BUG-311): exact 10% boundary (utilization=90.0) must write sonnet, got: {content}",
  );
  assert!(
    !content.contains( "\"opus\"" ),
    "exact 10% boundary: opus override must NOT fire (strict <), got: {content}",
  );
}

#[ test ]
fn t08_model_override_trace_label_is_usage()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  use std::io::Read;
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 90.0, resets_at : None } ),
  };
  let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
  let mut buf = gag::BufferRedirect::stderr().unwrap();
  apply_model_override( &quota, &paths, true, "usage", "test-account" );
  let mut output = String::new();
  buf.read_to_string( &mut output ).unwrap();
  assert!(
    output.contains( " · usage" ),
    "trace output must contain ' · usage' when label='usage', got: {output}",
  );
  assert!(
    !output.contains( " · account.use  " ),
    "trace output must NOT contain ' · account.use' when label='usage', got: {output}",
  );
}

// ── BUG-311 tests: bidirectional model override ────────────────────────────

/// `mre_bug311` — `apply_model_override()` never restores `"sonnet"` when Sonnet quota recovers.
///
/// # Root Cause
/// `apply_model_override()` was one-way: it only wrote `"opus"` when Sonnet was below
/// 15% threshold. The else-branch was absent — when Sonnet was sufficient, no write
/// occurred and `settings.json` retained whatever model was set before the switch
/// (commonly `"opus"` from a prior exhaustion cycle).
///
/// # Why Not Caught
/// All pre-existing tests only checked the opus-write path. No test pre-wrote `"opus"`
/// to `settings.json` then called `apply_model_override()` with sufficient Sonnet quota.
///
/// # Fix Applied
/// Added else-branch in `apply_model_override()`: when `sonnet_left >= threshold`, call
/// `override_session_model_to_sonnet()`. Added `override_session_model_to_sonnet()` in
/// `claude_profile_core::account` mirroring `override_session_model_to_opus()`.
///
/// # Prevention
/// This test pre-writes `"opus"` and asserts restoration to `"sonnet"` when quota allows.
///
/// # Pitfall
/// Model state after `.account.use` must always reflect the current quota — not whatever
/// the previous session had. Unidirectional overrides silently preserve stale state.
#[ doc = "bug_reproducer(BUG-311)" ]
#[ test ]
fn mre_bug311_model_restored_to_sonnet_when_opus_and_quota_sufficient()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-write "opus" — simulates state left from a prior quota-exhaustion cycle.
  std::fs::write( paths.settings_file(), r#"{"model":"opus"}"# ).unwrap();
  // Sonnet quota is fine: 4% utilization → 96% left > 15% threshold.
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 4.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"sonnet\"" ) && !content.contains( "\"opus\"" ),
    "BUG-311: must restore sonnet when prior model=opus and 7d(Son) 96% left, got: {content}",
  );
}

/// BUG-311 trace: `apply_model_override()` emits `opus→sonnet` trace when restoring model.
#[ test ]
fn t09_model_override_trace_opus_to_sonnet()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  use std::io::Read;
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-write "opus" so the gate in override_session_model_to_sonnet fires.
  std::fs::write( paths.settings_file(), r#"{"model":"opus"}"# ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 4.0, resets_at : None } ),
  };
  let _stderr_guard = claude_profile::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
  let mut buf = gag::BufferRedirect::stderr().unwrap();
  apply_model_override( &quota, &paths, true, "account.use", "test-account" );
  let mut output = String::new();
  buf.read_to_string( &mut output ).unwrap();
  assert!(
    output.contains( "model override: opus→sonnet" ),
    "BUG-311: trace must contain 'opus→sonnet' when restoring from stale opus, got: {output}",
  );
}

// ── BUG-312 tests: effort initialization ──────────────────────────────────

/// `mre_bug312` — `apply_model_override()` never initializes `effortLevel`; footer omits effort.
///
/// # Root Cause
/// `set_session_effort()` was only called from `.usage rotate::1` (carry-forward path).
/// Neither `.account.use` nor plain `.usage` initialized `effortLevel` in `settings.json`.
/// The footer reads `effortLevel` from `settings.json`; when absent, it renders as
/// `model` without `/effort` — effort was invisible regardless of usage pattern.
///
/// # Why Not Caught
/// All prior effort tests exercised the carry-forward path (`rotate::1`). No test called
/// `apply_model_override()` and then checked for `effortLevel` in `settings.json`.
///
/// # Fix Applied
/// TSK-335: `apply_model_override()` now writes effort unconditionally in all 3 branches.
/// When `settings.json` is absent (fresh install), the Sonnet branch fires and writes `"high"`.
/// The BUG-312 fallback guard (`get_session_effort().is_none()`) remains as a safety net
/// but is now unreachable — the unconditional branch always fires first.
///
/// # Prevention
/// This test asserts that `effortLevel` is written with value `"high"` after calling
/// `apply_model_override()` when `settings.json` has no `effortLevel` key.
///
/// # Pitfall
/// The BUG-312 fallback (`get_session_effort().is_none()`) must remain AFTER all 3 branches
/// so it only fires in unforeseen code paths where all branches somehow skipped effort writes.
#[ doc = "bug_reproducer(BUG-312)" ]
#[ test ]
fn mre_bug312_effort_initialized_to_high_when_absent()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // No settings.json initially — simulates fresh install or first .account.use.
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 4.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"effortLevel\"" ) && content.contains( "\"high\"" ),
    "BUG-312: effortLevel must be initialized to 'high' when absent (Sonnet branch unconditional write), got: {content}",
  );
}

/// TSK-335: Sonnet branch unconditionally writes `"high"`, overwriting any pre-existing effort value.
///
/// After TSK-335, `apply_model_override()` writes effort on every call regardless of model change.
/// The old BUG-312 guard (`get_session_effort().is_none()`) that preserved user-configured effort
/// is now superseded — the Sonnet branch writes `"high"` even when a different effort was set.
#[ test ]
fn t10_sonnet_branch_writes_effort_high()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-seed settings.json with effort "low" — Sonnet branch must overwrite it with "high".
  std::fs::write( paths.settings_file(), r#"{"model":"claude-sonnet-5","effortLevel":"low"}"# ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 4.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"high\"" ),
    "TSK-335: Sonnet branch must unconditionally write 'high', overwriting pre-seeded 'low', got: {content}",
  );
  assert!(
    !content.contains( "\"low\"" ),
    "TSK-335: 'low' must be overwritten by Sonnet branch unconditional write, got: {content}",
  );
}

// ── BUG-322 tests: effort must track model override ───────────────────────

/// `mre_bug322` — `apply_model_override()` sets effort `"low"` when overriding model to Opus.
///
/// # Root Cause
/// BUG-312 fix initialized `effortLevel` to `"low"` when absent, regardless of the model
/// being overridden to Opus. The Opus branch wrote the model but never set effort.
/// Result: `opus/low` in the footer instead of `opus/max`.
///
/// # Why Not Caught
/// No test verified effort after an Opus model override. All effort tests exercised the
/// init-only path (BUG-312) or the carry-forward path (rotation).
///
/// # Fix Applied
/// TSK-335: Opus branch writes `set_session_effort(paths, "max")` unconditionally.
/// Sonnet branch writes `set_session_effort(paths, "high")` unconditionally.
/// Effort is now model-derived and always synced regardless of whether the model changed.
///
/// # Prevention
/// This test asserts `effortLevel = "max"` after Opus model override, not `"low"` or `"high"`.
///
/// # Pitfall
/// Opus effort is `"max"` (extended thinking), not `"high"`. Sonnet effort is `"high"` (standard
/// high-quality). These are distinct values — never use `"high"` for Opus assertions.
#[ doc = "bug_reproducer(BUG-322)" ]
#[ test ]
fn mre_bug322_opus_override_sets_effort_max()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 91.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"opus\"" ),
    "BUG-322: model must be opus when 7d(Son) is 91% consumed, got: {content}",
  );
  assert!(
    content.contains( "\"max\"" ),
    "BUG-322: effortLevel must be 'max' when model overridden to opus, got: {content}",
  );
}

/// TSK-335: effort syncs to `"high"` when model reverts from Opus to Sonnet (Sonnet branch unconditional).
#[ test ]
fn t11_opus_to_sonnet_sets_effort_high()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-write opus/max — simulates state after Opus override.
  std::fs::write( paths.settings_file(), r#"{"model":"opus","effortLevel":"max"}"# ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 4.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"sonnet\"" ),
    "TSK-335: model must revert to sonnet when quota recovers, got: {content}",
  );
  assert!(
    content.contains( "\"high\"" ),
    "TSK-335: effortLevel must be 'high' when model reverts to sonnet (Sonnet branch unconditional write), got: {content}",
  );
}

/// TSK-335: effort syncs to `"high"` when absent tier forces model from Opus to Sonnet (absent-tier branch unconditional).
#[ test ]
fn t12_absent_tier_with_opus_sets_effort_high()
{
  use claude_quota::OauthUsageData;
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  std::fs::write( paths.settings_file(), r#"{"model":"opus","effortLevel":"max"}"# ).unwrap();
  let quota = OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  apply_model_override( &quota, &paths, false, "test", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"sonnet\"" ),
    "TSK-335: absent tier + opus session must write sonnet, got: {content}",
  );
  assert!(
    content.contains( "\"high\"" ),
    "TSK-335: absent tier branch must unconditionally write 'high', got: {content}",
  );
}

/// FT-19 — `apply_model_override()` syncs effort even when model is already at target (always-sync).
///
/// # Root Cause
/// `set_session_effort()` was gated inside `if overrode { }`. When the model was already at the
/// target value (e.g., Sonnet while Sonnet quota ≥ 15%), `overrode = false` → no effort write.
/// Sessions that never changed model never received an `effortLevel` key in settings.json until
/// the BUG-312 fallback kicked in — but the fallback wrote `"low"`, not `"high"` (TSK-335 H2).
///
/// # Why Not Caught
/// All prior tests covered only model-change paths (`overrode = true`). No test exercised the
/// stable-model path where the model was already at target before the call.
///
/// # Fix Applied
/// Moved `set_session_effort()` outside `if overrode { }` in all branches of
/// `apply_model_override()`. Every call now writes effort unconditionally.
/// Sonnet branch writes `"high"`; Opus branch writes `"max"`.
///
/// # Prevention
/// This test calls `apply_model_override()` with Sonnet model already set (so `overrode = false`
/// in the Sonnet branch) and asserts that `effortLevel` is still written with value `"high"`.
///
/// # Pitfall
/// Do not gate effort writes on `overrode` — the predicate answers "did the model change?" not
/// "should effort be updated?". Effort must always match the model branch executed.
#[ test ]
fn ft19_effort_synced_when_model_already_at_target()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-seed settings.json with Sonnet model but no effortLevel — stable-model state.
  std::fs::write( paths.settings_file(), r#"{"model":"claude-sonnet-5"}"# ).unwrap();
  // 80% utilization → 20% left → ≥ 15% threshold → Sonnet branch fires.
  // override_session_model_to_sonnet() returns false (model already "claude-sonnet-5").
  // Without fix: overrode=false → no effort write; BUG-312 fallback wrote "low".
  // With fix: Sonnet branch unconditionally writes "high".
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 80.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"effortLevel\"" ) && content.contains( "\"high\"" ),
    "FT-19: effortLevel must be synced to 'high' even when model unchanged (stable Sonnet path), got: {content}",
  );
  assert!(
    !content.contains( "\"low\"" ),
    "FT-19: 'low' must not appear — Sonnet branch always writes 'high', got: {content}",
  );
}

/// # MRE: BUG-245 + BUG-246 — `only_active` retain fires after HTTP fetch
///
/// ## Root Cause
/// `usage_routine()` placed `accounts.retain( |aq| aq.is_active )` at ~line 490,
/// after `fetch_all_quota` (~line 430). With N saved accounts and `only_active::1`,
/// all N accounts were HTTP-fetched before the row filter reduced output to 1 row.
///
/// ## Why Not Caught
/// No structural test enforced ordering of the pre-filter vs. the HTTP fetch call.
/// Output was correct (1 row shown) but N unnecessary HTTP round trips occurred.
///
/// ## Fix Applied
/// Pre-filter via `account::list()` (filesystem marker, no HTTP) in `usage_routine()`
/// before the HTTP fetch. When `only_active::1`, only the active account is passed to
/// `fetch_quota_for_list()` — exactly 1 HTTP call per invocation. BUG-246 (touch loop)
/// is fixed simultaneously: `apply_touch` iterates the pre-filtered 1-entry slice.
///
/// ## Prevention
/// This structural test: assert that `retain( |aq| aq.is_active )` appears in
/// `usage_routine` source BEFORE `fetch_quota_for_list(` — maintains the ordering invariant.
///
/// ## Pitfall
/// The pre-filter uses `Vec<Account>` (from `account::list()`), not `Vec<AccountQuota>`.
/// Both types have `is_active: bool` — the retain closure `|aq| aq.is_active` works on either.
#[ doc = "bug_reproducer(BUG-245)" ]
#[ doc = "bug_reproducer(BUG-246)" ]
#[ test ]
fn mre_bug245_only_active_retain_fires_before_http_fetch()
{
  let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  let fn_start = src.find( "pub fn usage_routine" ).expect( "usage_routine not found" );
  let body_end = {
    let after = &src[ fn_start + 1.. ];
    let a     = after.find( "\npub fn " ).unwrap_or( after.len() );
    let b     = after.find( "\n#[ cfg( t" ).unwrap_or( after.len() );
    let c     = after.find( "\n#[cfg(t" ).unwrap_or( after.len() );
    fn_start + 1 + a.min( b ).min( c )
  };
  let body       = &src[ fn_start..body_end ];
  let retain_pos = body.find( "retain( |aq| aq.is_active )" )
    .expect( "BUG-245: retain( |aq| aq.is_active ) must exist in usage_routine body" );
  let fetch_pos  = body.find( "fetch_quota_for_list(" )
    .expect( "BUG-245: fetch_quota_for_list call must exist in usage_routine body" );
  assert!(
    retain_pos < fetch_pos,
    "BUG-245/246: retain must fire BEFORE fetch_quota_for_list — \
    retain at ~{retain_pos}, fetch at ~{fetch_pos}",
  );
}


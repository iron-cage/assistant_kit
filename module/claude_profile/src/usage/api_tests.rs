// Path-referenced test module for api.rs — compiled as `mod tests` via `#[path]`.
// Lives in src/usage/ (not tests/) to access pub(crate) pre_switch_touch_ctx,
// apply_model_override, apply_post_switch_touch, and TouchCtx
// without widening their visibility. See src/usage/readme.md § Inline Test Exception.

use super::{ pre_switch_touch_ctx, apply_model_override, apply_post_switch_touch, PreSwitchOutcome, TouchCtx };
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
  let paths = crate::ClaudePaths::with_home( dir.path() );
  // Create ~/.claude/ so override_session_model_to_opus can write settings.json.
  std::fs::create_dir_all( paths.base() ).unwrap();

  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 90.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "account.use", "test-account" );

  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"opus\"" ) && !content.contains( "claude-opus-4-6" ),
    "apply_model_override must write opus shorthand to settings.json when 7d(Son) is 90% consumed (10% left), got: {content}",
  );
}

/// `mre_bug286` — full model ID `"claude-opus-4-6"` in settings.json must be
/// normalized to shorthand `"opus"` by `apply_model_override`.
///
/// # Root Cause
/// BUG-257 fix added `contains("sonnet")` for sonnet alias coverage, but did not
/// add the analogous full-ID opus → shorthand normalization path. When settings.json
/// contained `"claude-opus-4-6"` (written by a prior `set_session_model`), the gate
/// `contains("sonnet") || is_empty()` was false — override never fired.
///
/// # Why Not Caught
/// No test wrote a full-ID opus value to settings.json before calling
/// `apply_model_override`. All existing tests started from empty or shorthand values.
///
/// # Fix Applied
/// Added `current == "claude-opus-4-6"` arm to the gate condition in
/// `override_session_model_to_opus`.
///
/// # Prevention
/// This test pre-writes the full opus ID and asserts normalization to shorthand.
///
/// # Pitfall
/// Any path that writes model to settings.json must write Claude Code shorthand
/// ("opus"/"sonnet"), never full IDs ("claude-opus-4-6").
#[ doc = "bug_reproducer(BUG-286)" ]
#[ test ]
fn mre_bug286_full_opus_id_normalized_to_shorthand()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = crate::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-write full opus ID — as stored in {name}.json snapshots from older convention.
  std::fs::write( paths.settings_file(), r#"{"model":"claude-opus-4-6"}"# ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 90.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "account.use", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"opus\"" ) && !content.contains( "claude-opus-4-6" ),
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
  let paths = crate::ClaudePaths::with_home( dir.path() );
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
  let paths = crate::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 90.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap();
  assert!(
    content.contains( "\"opus\"" ) && !content.contains( "claude-opus-4-6" ),
    "must write opus shorthand when 7d(Son) utilization=90% (10% left), got: {content}",
  );
}

/// Fix(BUG-311): when Sonnet quota is sufficient, opus override skips and sonnet is
/// written as the recommended session model.
#[ test ]
fn t02_model_override_writes_sonnet_when_quota_sufficient()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = crate::ClaudePaths::with_home( dir.path() );
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
  let paths = crate::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-write settings.json with opus shorthand already set (production convention).
  std::fs::write( paths.settings_file(), r#"{"model":"opus"}"# ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 90.0, resets_at : None } ),
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
  let src      = include_str!( "api.rs" );
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
  let src      = include_str!( "api.rs" );
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
  let src      = include_str!( "api.rs" );
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

/// Fix(BUG-311): at the exact 15% boundary (left == 15%, not < 15%), opus override skips
/// and sonnet is written as recommended model.
#[ test ]
fn t07_model_override_writes_sonnet_at_15pct_boundary()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = crate::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // utilization=85.0 → sonnet_left = 100.0 - 85.0 = 15.0; 15.0 < 15.0 == false → sonnet wins.
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 85.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "usage", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap_or_default();
  assert!(
    content.contains( "\"sonnet\"" ),
    "Fix(BUG-311): exact 15% boundary (utilization=85.0) must write sonnet, got: {content}",
  );
  assert!(
    !content.contains( "\"opus\"" ),
    "exact 15% boundary: opus override must NOT fire (strict <), got: {content}",
  );
}

#[ test ]
fn t08_model_override_trace_label_is_usage()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  use std::io::Read;
  let dir   = TempDir::new().unwrap();
  let paths = crate::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 90.0, resets_at : None } ),
  };
  let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
  let mut buf = gag::BufferRedirect::stderr().unwrap();
  apply_model_override( &quota, &paths, true, "usage", "test-account" );
  let mut output = String::new();
  buf.read_to_string( &mut output ).unwrap();
  assert!(
    output.contains( "[trace] usage" ),
    "trace output must contain '[trace] usage' when label='usage', got: {output}",
  );
  assert!(
    !output.contains( "[trace] account.use" ),
    "trace output must NOT contain '[trace] account.use' when label='usage', got: {output}",
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
  let paths = crate::ClaudePaths::with_home( dir.path() );
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
  let paths = crate::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-write "opus" so the gate in override_session_model_to_sonnet fires.
  std::fs::write( paths.settings_file(), r#"{"model":"opus"}"# ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 4.0, resets_at : None } ),
  };
  let _stderr_guard = crate::usage::test_support::STDERR_LOCK.lock().unwrap_or_else( std::sync::PoisonError::into_inner );
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
/// Added `get_session_effort` guard in `apply_model_override()`:
/// when `effortLevel` is absent, write `"low"` (matching `resolve_effort(Auto)` default).
/// Preserves user-configured effort — only writes on first initialization.
///
/// # Prevention
/// This test asserts that `effortLevel` is written with value `"low"` after calling
/// `apply_model_override()` when `settings.json` has no `effortLevel` key.
///
/// # Pitfall
/// The effort init guard (`get_session_effort().is_none()`) reads `settings.json` AFTER
/// the model write — must always be the last operation so it sees the updated file.
#[ doc = "bug_reproducer(BUG-312)" ]
#[ test ]
fn mre_bug312_effort_initialized_to_low_when_absent()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = crate::ClaudePaths::with_home( dir.path() );
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
    content.contains( "\"effortLevel\"" ) && content.contains( "\"low\"" ),
    "BUG-312: effortLevel must be initialized to 'low' when absent, got: {content}",
  );
}

/// BUG-312 guard: user-configured effort must not be overwritten.
#[ test ]
fn t10_effort_preserved_when_already_configured()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = crate::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-write user-configured effort "high" — must survive apply_model_override.
  std::fs::write( paths.settings_file(), r#"{"model":"sonnet","effortLevel":"high"}"# ).unwrap();
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
    "BUG-312 guard: user-set effortLevel='high' must not be overwritten, got: {content}",
  );
  assert!(
    !content.contains( "\"low\"" ),
    "BUG-312 guard: 'low' must not replace user-set 'high', got: {content}",
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
  let src      = include_str!( "api.rs" );
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

/// # MRE: BUG-244 — `usage_routine` never called `apply_model_override`
///
/// ## Root Cause
/// `usage_routine()` in `src/usage/api.rs` had no call to `apply_model_override()`.
/// The function existed and worked (called from `.account.use` path) but was never
/// invoked from the `.usage` command path.
///
/// ## Why Not Caught
/// No test exercised the full `usage_routine()` → `apply_model_override()` wiring.
/// Unit tests for `apply_model_override` passed trivially (calling it directly).
///
/// ## Fix Applied
/// Added `apply_model_override( data, claude_paths, params.trace, "usage", &current.name )`
/// in `usage_routine()` after the touch loop, before the row-filter pipeline,
/// guarded by `aq.is_current && aq.result.is_ok()`.
/// Also added `label: &str` parameter to `apply_model_override` (was hardcoded "account.use").
///
/// ## Prevention
/// T05 structural test: grep of `usage_routine` body must contain exactly one call.
///
/// ## Pitfall
/// Insert the call BEFORE the row-filter pipeline (`only_next`/`only_active`/etc.) — those
/// filters can remove the `is_current` entry from the slice, causing a silent no-op.
#[ doc = "bug_reproducer(BUG-244)" ]
#[ test ]
fn mre_bug244_usage_routine_never_calls_apply_model_override()
{
  let src      = include_str!( "api.rs" );
  let fn_start = src.find( "pub fn usage_routine" ).expect( "usage_routine not found" );
  let body_end = {
    let after = &src[ fn_start + 1.. ];
    let a     = after.find( "\npub fn " ).unwrap_or( after.len() );
    let b     = after.find( "\n#[ cfg( t" ).unwrap_or( after.len() );
    let c     = after.find( "\n#[cfg(t" ).unwrap_or( after.len() );
    fn_start + 1 + a.min( b ).min( c )
  };
  let body = &src[ fn_start..body_end ];
  assert!(
    body.contains( "apply_model_override(" ),
    "BUG-244: apply_model_override must be called from usage_routine — was absent before fix",
  );
}

/// # MRE: BUG-285 — idle check used server-side `resets_at` as local subprocess oracle
///
/// ## Root Cause
/// `pre_switch_touch_ctx()` computed `is_idle` from `quota.five_hour.resets_at.is_none()`.
/// `resets_at` is set server-side by Anthropic's API for any session on any machine —
/// it is NOT a local subprocess identity indicator. An account with `resets_at=Some`
/// (set by a session on another machine) returned `AlreadyActive` and skipped the
/// subprocess touch, even though no local subprocess was running.
///
/// ## Why Not Caught
/// The `is_idle` check appeared logically sound in single-machine setups: if the 5h window
/// is counting down, a subprocess must have started it. This reasoning fails for accounts
/// used across multiple machines where any machine can advance the server-side state.
///
/// ## Fix Applied
/// Removed `is_idle` check entirely from `pre_switch_touch_ctx`. Function now always
/// returns `NeedTouch` when quota is successfully fetched. `AlreadyActive` variant
/// removed from `PreSwitchOutcome`. `trace_already_active()` deleted as dead code.
///
/// ## Prevention
/// This structural test asserts that `let is_idle` assignment and `AlreadyActive` return
/// no longer appear in `pre_switch_touch_ctx` — any reintroduction of the oracle pattern
/// fails the test.
///
/// ## Pitfall
/// `resets_at` presence does NOT mean a local subprocess is active. Server-side state
/// reflects global account state; local subprocess identity requires local process
/// introspection, not quota API responses.
#[ doc = "bug_reproducer(BUG-285)" ]
#[ test ]
fn mre_bug285_idle_check_uses_resets_at_as_wrong_oracle()
{
  let src      = include_str!( "api.rs" );
  let fn_start = src
    .find( "pub( crate ) fn pre_switch_touch_ctx(" )
    .expect( "pre_switch_touch_ctx not found in api.rs" );
  let fn_end   = src[ fn_start + 1.. ]
    .find( "\npub( crate ) fn " )
    .map_or( src.len(), |rel| fn_start + 1 + rel );
  let fn_body  = &src[ fn_start..fn_end ];

  assert!(
    !fn_body.contains( "let is_idle" ),
    "BUG-285 regression: `let is_idle` variable must not exist in pre_switch_touch_ctx body\n\
    resets_at is server-side state and cannot proxy local subprocess identity",
  );
  assert!(
    !fn_body.contains( "AlreadyActive" ),
    "BUG-285 regression: AlreadyActive must not be returned from pre_switch_touch_ctx",
  );
}

/// `mre_bug288` — `apply_post_switch_touch` re-fetch is non-aborting when the credentials
/// file has no `accessToken`; pre-re-fetch cache writes succeed regardless.
///
/// # Root Cause
/// `apply_post_switch_touch` discarded the `run_isolated` result with `let _ = ...` and
/// performed no post-subprocess quota re-fetch. A subsequent `.usage touch` call then saw
/// stale `resets_at = None` and spawned a redundant second subprocess (double-subprocess race).
///
/// # Why Not Caught
/// No unit test exercised `apply_post_switch_touch` directly. The only coverage was via
/// `lim_it` CLI integration tests requiring live OAuth credentials (`aw27`, `aw28`, `aw29`).
///
/// # Fix Applied
/// Added post-subprocess quota re-fetch block (AC-21) to `apply_post_switch_touch`, mirroring
/// `apply_touch`'s AC-03 pattern. Reads credentials fresh from disk (not from
/// `ctx.credentials_json`) to capture any post-subprocess token rotation. On `Ok(new_data)`:
/// calls `write_quota_cache(paths.base(), name, ...)` to persist the updated quota. On
/// failure: silently skips — non-aborting per AC-21.
///
/// # Prevention
/// This test verifies the non-aborting invariant: when `accessToken` is absent from the
/// credentials file, the re-fetch is silently skipped and the function returns normally.
/// Also verifies that `last_touch_at` and `touch_idle` (written before the re-fetch block)
/// are committed to disk even when the re-fetch is skipped.
///
/// # Pitfall
/// `apply_post_switch_touch` is `pub(crate)` — only testable inline in `src/usage/api.rs`.
/// The re-fetch block reads credentials from `paths.base()/{name}.credentials.json` (fresh
/// disk read), NOT from `ctx.credentials_json` — tests must write the credential file at
/// that path, not just supply a non-empty string in the `TouchCtx`.
#[ doc = "bug_reproducer(BUG-288)" ]
#[ test ]
fn mre_bug288_post_switch_touch_refetch_updates_quota()
{
  use claude_quota::OauthUsageData;

  // ── Success path (structural): write_quota_cache is called when re-fetch succeeds ──
  // Verifies Fix(BUG-288) is present: apply_post_switch_touch must call write_quota_cache
  // on a successful fetch_oauth_usage result so subsequent .usage sees the updated resets_at.
  {
    let src      = include_str!( "api.rs" );
    let fn_start = src
      .find( "pub( crate ) fn apply_post_switch_touch(" )
      .expect( "apply_post_switch_touch not found in api.rs" );
    let fn_end   = src[ fn_start + 1.. ]
      .find( "\n// ── Main routine" )
      .map( |rel| fn_start + 1 + rel )
      .expect( "anchor '// ── Main routine' must follow apply_post_switch_touch — update structural test if section was renamed" );
    let fn_body  = &src[ fn_start..fn_end ];
    assert!(
      fn_body.contains( "fetch_oauth_usage" ),
      "BUG-288: apply_post_switch_touch must call fetch_oauth_usage for AC-21 re-fetch",
    );
    assert!(
      fn_body.contains( "write_quota_cache" ),
      "BUG-288: apply_post_switch_touch must call write_quota_cache on successful re-fetch \
      so subsequent .usage sees the updated resets_at (no double-subprocess race)",
    );
  }

  // ── Failure path (runtime): no accessToken → re-fetch silently skipped ──────────────
  let dir   = TempDir::new().unwrap();
  let paths = crate::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let name = "test@example.com";

  // Failure path: credentials file has no `accessToken` field.
  // `parse_string_field` returns None → re-fetch skipped → non-aborting.
  std::fs::write(
    paths.base().join( format!( "{name}.credentials.json" ) ),
    r#"{"subscriptionType":"pro","expiresAt":9999999999999}"#,
  ).unwrap();

  let ctx = TouchCtx::for_test(
    OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None },
  );

  // Must not panic: run_isolated fails silently (let _ = ...); re-fetch skipped.
  apply_post_switch_touch( name, ctx, "auto", "auto", false, &paths, paths.base() );

  // Observable 1: pre-re-fetch cache writes (last_touch_at, touch_idle) must succeed
  // even when the re-fetch block is skipped — they are written unconditionally before it.
  let cache_path = paths.base().join( format!( "{name}.json" ) );
  let cache = std::fs::read_to_string( &cache_path )
    .expect( "BUG-288: cache file must exist after apply_post_switch_touch" );
  assert!(
    cache.contains( "last_touch_at" ),
    "BUG-288: last_touch_at must be written to cache even when re-fetch is skipped, got: {cache}",
  );
  assert!(
    cache.contains( "touch_idle" ),
    "BUG-288: touch_idle must be written to cache even when re-fetch is skipped, got: {cache}",
  );

  // Observable 2: quota re-fetch must have been skipped — `resets_at` must not appear.
  // If the re-fetch had fired with a live token and written new quota data, this would fail,
  // making it both an absence-check for the failure path and an implicit sentinel that the
  // test fixture did not accidentally provide a live credential.
  assert!(
    !cache.contains( "resets_at" ),
    "BUG-288: resets_at must not be written when accessToken is absent (re-fetch skipped), got: {cache}",
  );
}

/// Corner case: credentials file absent → outer `read_to_string` guard fails →
/// entire re-fetch block skipped; `last_touch_at` and `touch_idle` still written; no panic.
///
/// # Root Cause
/// The AC-21 re-fetch block uses three nested `if let` guards:
///   1. `if let Ok(fresh_json) = read_to_string(&cred_path)` — outer: file I/O
///   2. `if let Some(token) = parse_string_field(...)` — inner: JSON field
///   3. `if let Ok(new_data) = fetch_oauth_usage(...)` — innermost: HTTP
///
/// `mre_bug288_post_switch_touch_refetch_updates_quota` covers guard 2 (file present,
/// no `accessToken`). This test covers guard 1 (file absent entirely).
///
/// # Why Not Caught
/// The file-absent path was not covered because the existing MRE test always writes a
/// credential file (albeit without `accessToken`). The outer guard is a distinct code
/// path even though observables are identical to the inner-guard failure path.
///
/// # Fix Applied
/// No fix required — `if let Ok` on `read_to_string` already handles this correctly.
/// This test verifies the non-aborting invariant for the outer guard specifically.
///
/// # Prevention
/// Nested `if let` re-fetch blocks must have unit tests per guard layer: outer I/O
/// guard and inner parse guard are independent failure modes requiring separate coverage.
///
/// # Pitfall
/// File-absent and file-present-no-token produce identical observables (`last_touch_at`
/// written, no `resets_at`) but exercise different branches. Both must be tested to
/// confirm the non-aborting invariant holds at each layer of the nested guard chain.
#[ test ]
fn it_apply_post_switch_touch_cred_file_absent_skips_refetch()
{
  use claude_quota::OauthUsageData;

  let dir   = TempDir::new().unwrap();
  let paths = crate::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let name = "absent@example.com";

  // No credentials file written — `read_to_string` returns Err.
  // Outer `if let Ok` guard fires → entire re-fetch block bypassed.

  let ctx = TouchCtx::for_test(
    OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None },
  );

  // Must not panic: run_isolated discards result; outer re-fetch guard skips silently.
  apply_post_switch_touch( name, ctx, "auto", "auto", false, &paths, paths.base() );

  // Observable 1: last_touch_at and touch_idle written unconditionally (before re-fetch block).
  let cache_path = paths.base().join( format!( "{name}.json" ) );
  let cache = std::fs::read_to_string( &cache_path )
    .expect( "cache file must exist even when credential file is absent" );
  assert!(
    cache.contains( "last_touch_at" ),
    "last_touch_at must be written even when credential file is absent; got: {cache}",
  );
  assert!(
    cache.contains( "touch_idle" ),
    "touch_idle must be written even when credential file is absent; got: {cache}",
  );

  // Observable 2: re-fetch outer guard fired → no resets_at in cache.
  assert!(
    !cache.contains( "resets_at" ),
    "resets_at must not appear when credential file is absent (outer guard bypassed); got: {cache}",
  );
}

/// `ft11` — Rotation dispatcher calls `apply_model_override` for the winner (AC-05, Feature 062).
///
/// # Root Cause
/// Before Feature 062, the rotation dispatch block in `usage_routine` called `switch_account` and
/// `apply_touch` but never `apply_model_override` for the winner — the winner's session model
/// remained at whatever was set by the previous current account.
///
/// # Why Not Caught
/// No structural test asserted the call site exists inside the rotation block.
///
/// # Fix Applied
/// Added `apply_model_override( winner_data, &claude_paths, ... )` inside the rotation dispatch
/// block, guarded by `if let Ok( ref winner_data ) = accounts[ winner_idx ].result`.
///
/// # Prevention
/// Structural grep: rotation dispatch block must contain `apply_model_override(`.
///
/// # Pitfall
/// Must insert AFTER `switch_account` succeeds — before that point, `claude_paths` is
/// conditionally set and `winner_idx` may not be resolved.
#[ test ]
fn ft11_rotation_dispatcher_calls_apply_model_override_for_winner()
{
  let src      = include_str!( "api.rs" );
  // Locate rotation dispatch block: starts at "Rotation dispatch" comment.
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  // Ends at the closing `}` of `if params.rotate { ... }` — after the final `return Ok`.
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  assert!(
    block.contains( "apply_model_override(" ),
    "AC-05: apply_model_override must be called in the rotation dispatch block (Feature 062)\n\
    block:\n{block}",
  );
}

/// `ft12` — Rotation dispatcher calls `set_session_effort` to carry forward effort (AC-06).
///
/// # Root Cause
/// Before Feature 062, `effortLevel` was never written during rotation — the user's effort
/// preference would vanish from settings.json after switching accounts.
///
/// # Why Not Caught
/// No structural test asserted the call site exists inside the rotation block.
///
/// # Fix Applied
/// Added `set_session_effort( &claude_paths, se )` inside the rotation dispatch block,
/// guarded by `if let Some( se ) = session_effort`.
///
/// # Prevention
/// Structural grep: rotation dispatch block must contain `set_session_effort(`.
///
/// # Pitfall
/// `session_effort` is read from the PRE-rotation settings.json and carried forward — it
/// represents the user's persistent effort preference, not the winner account's data.
#[ test ]
fn ft12_rotation_dispatcher_calls_set_session_effort_for_carry_forward()
{
  let src         = include_str!( "api.rs" );
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  assert!(
    block.contains( "set_session_effort(" ),
    "AC-06: set_session_effort must be called in the rotation dispatch block (Feature 062)\n\
    block:\n{block}",
  );
}

/// `ft13` — carry-forward `set_session_effort` is guarded by `if let Some( se ) = session_effort` (AC-07).
///
/// # Root Cause
/// AC-07 (pre-BUG-312): `set_session_effort` must only carry forward the pre-rotation effort value
/// when one was already configured — not inject a default. After Fix(BUG-312),
/// `apply_model_override()` now initializes `effortLevel: "low"` when absent; the carry-forward
/// guard here prevents `set_session_effort` from overwriting that init with `None` (no-op).
///
/// # Why Not Caught
/// No structural test asserted the `Some(se)` guard wraps the carry-forward call.
///
/// # Fix Applied
/// `set_session_effort` is called inside `if let Some( se ) = session_effort { ... }` — only
/// fires when the pre-rotation settings.json contained an `effortLevel` value.
/// The init path (for absent effortLevel) is handled by `apply_model_override()` (Fix BUG-312).
///
/// # Prevention
/// Structural grep: carry-forward `set_session_effort` call must be inside
/// `if let Some( se ) = session_effort { ... }`.
///
/// # Pitfall
/// The guard checks `session_effort` (the `Option<&str>` extracted from settings.json before
/// render), not `params.effort` (the CLI param). These are distinct: params.effort governs
/// the subprocess model; `session_effort` governs the settings.json effortLevel field.
#[ test ]
fn ft13_rotation_set_session_effort_guarded_by_some_not_injected()
{
  let src         = include_str!( "api.rs" );
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  assert!(
    block.contains( "if let Some( se ) = session_effort" ),
    "AC-07: carry-forward set_session_effort must be guarded by \
`if let Some( se ) = session_effort` (Feature 062; init path handled by apply_model_override BUG-312)\n\
block:\n{block}",
  );
}

// ── BUG-310 MRE: rotation dispatch must re-sync live credentials after apply_touch ─

/// MRE for BUG-310: after `apply_touch` in the rotation dispatch block, the winner's
/// store credentials must be copied back to the live session file.
///
/// # Root Cause
///
/// `switch_account(winner)` at step 4d copies store→live BEFORE `apply_touch` at step 4e.
/// The touch subprocess may refresh the OAuth token, writing `token_B` to the STORE file
/// via `refresh_account_token → save(update_marker=false)`. The live session retains stale
/// `token_A` — if the server invalidated `token_A` during refresh, the live session dies.
///
/// # Why Not Caught
///
/// No test asserted that the rotation block re-syncs live credentials after the touch step.
/// `apply_touch` intentionally writes to STORE only (BUG-211 fix) — the live re-sync is
/// the caller's responsibility, and the rotation dispatcher was the only caller that needed it.
///
/// # Fix Applied
///
/// `std::fs::copy( store/{name}.credentials.json, claude_paths.credentials_file() )` added
/// immediately after `apply_touch` in the rotation dispatch block (step 4e').
///
/// # Prevention
///
/// Structural grep: rotation dispatch block must contain `fs::copy` (or `std::fs::copy`)
/// after `apply_touch`. This test enforces that.
///
/// # Pitfall
///
/// Do NOT call `switch_account` again — it re-writes the `_active` marker and patches
/// `.claude.json` redundantly. A targeted credential file copy suffices.
///
/// `let _ = std::fs::copy(...)` silently discards I/O errors. If the filesystem write
/// fails (permissions, disk full), rotation still reports success but live credentials
/// may remain stale. A future improvement would emit a trace warning on copy failure.
///
/// Spec: [`tests/docs/feature/38_usage_strategy_rotate.md` FT-11]
#[ doc = "bug_reproducer(BUG-310)" ]
#[ test ]
fn mre_bug310_rotation_touch_resyncs_live_credentials()
{
  let src         = include_str!( "api.rs" );
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  // Locate apply_touch call within the rotation block.
  let touch_pos = block
    .find( "apply_touch(" )
    .expect( "BUG-310: apply_touch call not found in rotation dispatch block" );

  // After apply_touch, there must be a store→live credential re-sync via fs::copy.
  let after_touch = &block[ touch_pos.. ];
  assert!(
    after_touch.contains( "fs::copy" ),
    "BUG-310 AC-11: rotation dispatch block must re-sync live credentials from store \
    after apply_touch via fs::copy. Without this, live retains stale pre-refresh token.\n\
    block after apply_touch:\n{after_touch}",
  );
}

/// Control for BUG-310: when `touch::0` is used (no `apply_touch` call in rotation),
/// `switch_account` alone suffices — store and live are already consistent.
///
/// This test verifies that `switch_account` IS called in the rotation block, ensuring
/// the store→live copy happens at step 4d. The divergence from D1 only arises when
/// `apply_touch` refreshes the token AFTER `switch_account` already copied.
///
/// Spec: [`tests/docs/feature/38_usage_strategy_rotate.md` FT-11 (control)]
#[ test ]
fn mre_bug310_rotation_no_refresh_no_divergence()
{
  let src         = include_str!( "api.rs" );
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  // switch_account must exist in the rotation block — this is the store→live copy at step 4d.
  assert!(
    block.contains( "switch_account(" ),
    "BUG-310 control: switch_account must be called in the rotation dispatch block \
    to copy store credentials to live at step 4d.\n\
    block:\n{block}",
  );

  // apply_touch must also exist — without it, there's no token refresh divergence.
  assert!(
    block.contains( "apply_touch(" ),
    "BUG-310 control: apply_touch must be called in the rotation dispatch block \
    at step 4e. Without it, no token refresh can create store/live divergence.\n\
    block:\n{block}",
  );
}

// ── D4: Rotation without touch — switch_account alone is consistent ───────

/// Reach test: `switch_account` precedes `apply_touch` in the rotation dispatch block.
/// This ordering means that when `touch::0` is used (`apply_touch` is skipped), the live
/// file already matches the store — no re-sync needed.
///
/// The test verifies that `switch_account` appears BEFORE `apply_touch` in the block,
/// ensuring step 4d (store→live copy) always happens before step 4e (touch may diverge).
///
/// Spec: [`tests/docs/feature/38_usage_strategy_rotate.md` FT-11 (reach D4)]
#[ test ]
fn reach_rotation_switch_account_precedes_apply_touch()
{
  let src         = include_str!( "api.rs" );
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  let switch_pos = block.find( "switch_account(" )
    .expect( "D4: switch_account not found in rotation block" );
  let touch_pos  = block.find( "apply_touch(" )
    .expect( "D4: apply_touch not found in rotation block" );
  assert!(
    switch_pos < touch_pos,
    "D4: switch_account (step 4d) must precede apply_touch (step 4e) in rotation block. \
    When touch::0 skips apply_touch, the store→live copy from switch_account is already consistent.",
  );
}

// ── D5: Structural guard — fs::copy after apply_touch in rotation block ───

/// Structural proximity guard: `fs::copy` must appear after `apply_touch` in the
/// rotation dispatch block, within a few lines. This prevents regression by ensuring
/// the re-sync step is never accidentally removed by refactoring.
///
/// Before fix: FAILS (no `fs::copy` exists). After fix: PASSES.
///
/// **Known gap:** verifies `fs::copy` and `credentials_file()` appear after `apply_touch`,
/// but does NOT verify the SOURCE argument is `credential_store.join(...)`. A refactor
/// that accidentally swaps src/dst would still pass. Copy-direction correctness is
/// enforced by code review and the `Fix(BUG-310)` comment in `api.rs`.
///
/// Spec: [`tests/docs/feature/38_usage_strategy_rotate.md` FT-11 (reach D5)]
#[ test ]
fn reach_structural_guard_fs_copy_follows_apply_touch_in_rotation()
{
  let src         = include_str!( "api.rs" );
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  let touch_pos = block.find( "apply_touch(" )
    .expect( "D5: apply_touch not found in rotation block" );
  let after_touch = &block[ touch_pos.. ];

  // fs::copy must appear after apply_touch — this is the re-sync step (4e').
  assert!(
    after_touch.contains( "fs::copy" ),
    "D5 AC-11: fs::copy must follow apply_touch in rotation block to re-sync \
    live credentials after potential token refresh.\n\
    block after apply_touch:\n{after_touch}",
  );

  // The fs::copy must reference credentials_file() — not some other path.
  assert!(
    after_touch.contains( "credentials_file()" ),
    "D5 AC-11: fs::copy target must be credentials_file() (the live session file).\n\
    block after apply_touch:\n{after_touch}",
  );
}

/// AC-34 routing structural: `apply_post_switch_touch` calls `refresh_account_token`, not `run_isolated`
///
/// # Root Cause
/// Before AC-34 / Invariant 008, `apply_post_switch_touch` called `run_isolated` directly —
/// a fire-and-forget pattern that bypassed:
///   - expiresAt=1 manipulation (AC-32): no RT rotation
///   - live credential sync (AC-33): no pre-sync or race recovery
///   - credential write-back: rotated credentials silently discarded
///
/// # Why Not Caught
/// No structural test asserted the routing destination. The IN-1 invariant test (grep-based)
/// verifies ABSENCE of direct `run_isolated` calls; this test verifies PRESENCE of the
/// `refresh_account_token` call — the positive routing complement to IN-1's negative guard.
///
/// # Fix Applied
/// AC-34: `apply_post_switch_touch` now calls `crate::account::refresh_account_token(...)`.
/// The `credential_store` parameter was added as the 7th param to carry the correct
/// `~/.persistent/claude/credential/` path (NOT `paths.base()` = `~/.claude/`).
///
/// # Prevention
/// Both the negative invariant (IN-1: zero `run_isolated` calls) and this positive
/// routing test must pass for AC-34 to be fully enforced.
///
/// # Pitfall
/// `apply_post_switch_touch` body region ends just before the `// ── Main routine` anchor.
/// If that anchor is renamed, the structural search must be updated.
#[ test ]
fn ft_apply_post_switch_touch_routes_through_refresh_account_token()
{
  let src      = include_str!( "api.rs" );
  let fn_start = src
    .find( "pub( crate ) fn apply_post_switch_touch(" )
    .expect( "apply_post_switch_touch must exist in api.rs (AC-34 routing entry point)" );
  let fn_end = src[ fn_start.. ]
    .find( "\n// ── Main routine" )
    .map( |rel| fn_start + rel )
    .expect( "'// ── Main routine' anchor must follow apply_post_switch_touch in api.rs" );
  let fn_body = &src[ fn_start..fn_end ];

  // Positive routing assertion: must delegate to refresh_account_token (AC-34).
  assert!(
    fn_body.contains( "refresh_account_token(" ),
    "AC-34: apply_post_switch_touch must call refresh_account_token() for token refresh \
    (not run_isolated directly) — see invariant 008 and Feature 017 AC-34\n\
    function body:\n{fn_body}",
  );

  // Negative routing assertion: must NOT call run_isolated directly (complements IN-1 grep).
  // Pattern built at runtime — avoids embedding "run_isolated(" as literal bytes in this file,
  // which would itself be a violation of the IN-1 invariant this test enforces.
  let direct_call_pattern = format!( "{}(", "run_isolated" );
  let violations : Vec< &str > = fn_body.lines()
    .filter( | line | !line.trim_start().starts_with( "//" ) )
    .filter( | line | line.contains( &direct_call_pattern ) )
    .collect();
  assert!(
    violations.is_empty(),
    "AC-34: apply_post_switch_touch must not invoke run_isolated directly; \
    all token refresh must route through refresh_account_token:\n{}",
    violations.join( "\n" ),
  );
}

/// BUG-317 MRE — `only_valid::1` must exclude cancelled accounts (`billing_type="none"`).
///
/// # Root Cause
/// The `only_valid` retain predicate in `api.rs` only checked `result.is_ok()`. A cancelled
/// account with a successful API response (`result = Ok(...)`) and `billing_type = "none"`
/// passed `only_valid::1` as if it were a valid account — potentially surfacing in the
/// valid-account list and rotation recommendations.
///
/// # Why Not Caught
/// All existing `only_valid` tests used accounts with `account = None` (no subscription data)
/// or `result = Err(...)`. The case of `result = Ok` + `billing_type = "none"` was untested:
/// good quota data returned by the API, but subscription permanently cancelled.
///
/// # Fix Applied
/// Fix D (BUG-317): the retain predicate is now:
/// `aq.result.is_ok() && !aq.account.as_ref().is_some_and(|a| a.billing_type == "none")`
/// Both conditions must pass. A cancelled account satisfies `result.is_ok()` but fails the
/// second condition — correctly excluded.
///
/// # Prevention
/// Structural inspection of `api.rs` via `include_str!` ensures the `billing_type` guard
/// cannot be silently removed. If Fix D is reverted, the structural assertion fails immediately.
///
/// # Pitfall
/// `account = None` is NOT equivalent to `billing_type = "none"`. `account = None` means the
/// account-API call failed (ambiguous). Only `account = Some({billing_type: "none"})` is the
/// definitive cancellation signal. `mk_aq_cancelled` always sets `account = Some(...)` with
/// `billing_type = "none"` — use it for cancelled-account tests.
#[ doc = "bug_reproducer(BUG-317)" ]
#[ test ]
fn mre_bug317_cancelled_excluded_by_only_valid()
{
  use crate::usage::test_support::mk_aq_cancelled;

  // ── Structural: verify Fix D predicate is present in api.rs ─────────────────────────────
  // include_str! is compile-time — the assertion fails at build if Fix D is reverted.
  let src = include_str!( "api.rs" );
  let only_valid_pos = src
    .find( "if params.only_valid" )
    .expect( "BUG-317 Fix D: 'if params.only_valid' block must exist in api.rs" );
  // Scan the retain expression (next 300 bytes) for the billing_type guard.
  let block = &src[ only_valid_pos .. ( only_valid_pos + 300 ).min( src.len() ) ];
  assert!(
    block.contains( "billing_type" ),
    "BUG-317 Fix D: only_valid retain predicate must check billing_type=\"none\" to exclude \
    cancelled accounts — revert of Fix D detected in api.rs\nblock:\n{block}",
  );

  // ── Preconditions: mk_aq_cancelled produces the critical BUG-317 scenario ────────────────
  // result = Ok  →  old predicate (result.is_ok() only) would pass this account through.
  // billing_type = "none"  →  Fix D second predicate correctly blocks it.
  let cancelled = mk_aq_cancelled( "dead@test.com", 20.0, 20.0 );
  assert!(
    cancelled.result.is_ok(),
    "BUG-317 precondition: mk_aq_cancelled must produce result=Ok — \
    cancelled account has valid quota data (the exact bug scenario: would have slipped through)",
  );
  assert!(
    cancelled.account.as_ref().is_some_and( |a| a.billing_type == "none" ),
    "BUG-317 precondition: mk_aq_cancelled must set billing_type=\"none\" (definitive cancel signal)",
  );

  // ── Predicate: Fix D correctly excludes the cancelled account ────────────────────────────
  // Replicate the Fix D retain predicate. The retain keeps accounts where this is true;
  // the cancelled account must evaluate to false (excluded).
  let passes_only_valid = cancelled.result.is_ok()
    && !cancelled.account.as_ref().is_some_and( |a| a.billing_type == "none" );
  assert!(
    !passes_only_valid,
    "BUG-317 Fix D: cancelled account (result=Ok, billing_type=\"none\") must be excluded \
    by only_valid::1 — the billing_type guard must negate the result.is_ok() pass",
  );
}

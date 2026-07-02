//! Integration tests: IT-122–IT-153 — `.usage` `imodel::`, `effort::`, structural gates.
//!
//! Covers TSK-191 `imodel::` and `effort::` parameters, BUG-181 trigger inversion fix,
//! TSK-220 `sort::renew` + `sort::next` meta-strategy, TSK-209 haiku model acceptance,
//! `sort::renew` strategy, row filtering basics, and Next column + JSON renewal fields.
//!
//! Live tests (names contain `lim_it`) require a real Anthropic OAuth access token.

use crate::cli_runner::{
  run_cs, run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token, write_account_renewal_json,
  write_live_credentials_with_token, live_active_token, require_live_api,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── TSK-191 — imodel:: and effort:: parameters ────────────────────────────────

/// it122 (IT-66 / EC-1): `imodel::auto` accepted with empty credential store exits 0.
///
/// Before TSK-191: `imodel::` is unregistered → `ArgumentUnrecognised` → exit 1.
/// After TSK-191:  `imodel::` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-1]
///       [`tests/docs/cli/command/009_usage.md` IT-66]
#[ test ]
fn it122_imodel_auto_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out  = run_cs_with_env( &[ ".usage", "imodel::auto" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ) || text.contains( "No accounts" ),
    "imodel::auto with empty store must exit 0 (IT-66/EC-1), got:\n{text}",
  );
}

/// it123 (IT-67 / EC-5): `imodel::bogus` exits 1; stderr names all five valid values.
///
/// The parser rejects any value not in {auto, sonnet, opus, keep, haiku} with exit 1.
/// All five valid values must appear in stderr to help the user correct the mistake.
/// TSK-209: updated from four to five values (added `haiku`).
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-5]
///       [`tests/docs/cli/command/009_usage.md` IT-67]
#[ test ]
fn it123_imodel_bogus_exits_1()
{
  let out  = run_cs( &[ ".usage", "imodel::bogus" ] );
  assert_exit( &out, 1 );
  let err  = stderr( &out );
  assert!( err.contains( "auto" ),   "stderr must name valid value 'auto', got:\n{err}" );
  assert!( err.contains( "sonnet" ), "stderr must name valid value 'sonnet', got:\n{err}" );
  assert!( err.contains( "opus" ),   "stderr must name valid value 'opus', got:\n{err}" );
  assert!( err.contains( "keep" ),   "stderr must name valid value 'keep', got:\n{err}" );
  assert!( err.contains( "haiku" ),  "stderr must name valid value 'haiku', got:\n{err}" );
}

/// it124 (IT-68 / EC-1): `effort::auto` accepted with empty credential store exits 0.
///
/// Before TSK-191: `effort::` is unregistered → `ArgumentUnrecognised` → exit 1.
/// After TSK-191:  `effort::` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-1]
///       [`tests/docs/cli/command/009_usage.md` IT-68]
#[ test ]
fn it124_effort_auto_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out  = run_cs_with_env( &[ ".usage", "effort::auto" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ) || text.contains( "No accounts" ),
    "effort::auto with empty store must exit 0 (IT-68/EC-1), got:\n{text}",
  );
}

/// it125 (IT-69 / EC-4): `effort::bogus` exits 1; stderr names all five valid values.
///
/// The parser rejects any value not in {auto, high, max, low, normal} with exit 1.
/// TSK-209: updated from three to five values (added `low` and `normal`).
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-4]
///       [`tests/docs/cli/command/009_usage.md` IT-69]
#[ test ]
fn it125_effort_bogus_exits_1()
{
  let out = run_cs( &[ ".usage", "effort::bogus" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "auto" ),   "stderr must name valid value 'auto', got:\n{err}" );
  assert!( err.contains( "high" ),   "stderr must name valid value 'high', got:\n{err}" );
  assert!( err.contains( "max" ),    "stderr must name valid value 'max', got:\n{err}" );
  assert!( err.contains( "low" ),    "stderr must name valid value 'low', got:\n{err}" );
  assert!( err.contains( "normal" ), "stderr must name valid value 'normal', got:\n{err}" );
}

/// it126 (IT-70): `.usage.help` lists `imodel` and `effort` as registered parameters.
///
/// Both params must appear in the help output after TSK-191 registration.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-70]
#[ test ]
fn it126_usage_help_shows_imodel_effort_params()
{
  let out  = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "imodel" ), ".usage.help must list param `imodel` (IT-70), got:\n{text}" );
  assert!( text.contains( "effort" ), ".usage.help must list param `effort` (IT-70), got:\n{text}" );
}

/// it127 (EC-2): `imodel::sonnet` accepted with empty credential store exits 0.
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-2]
#[ test ]
fn it127_imodel_sonnet_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "imodel::sonnet" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it128 (EC-3): `imodel::opus` accepted with empty credential store exits 0.
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-3]
#[ test ]
fn it128_imodel_opus_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "imodel::opus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it129 (EC-4): `imodel::keep` accepted with empty credential store exits 0.
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-4]
#[ test ]
fn it129_imodel_keep_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "imodel::keep" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it130 (EC-2 for effort): `effort::high` accepted with empty credential store exits 0.
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-2]
#[ test ]
fn it130_effort_high_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "effort::high" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it131 (EC-3 for effort): `effort::max` accepted with empty credential store exits 0.
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-3]
#[ test ]
fn it131_effort_max_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "effort::max" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

// ── 026: imodel/effort structural and JSON invariance ─────────────────────────

/// `it_ft026_13` (026 FT-13 / AC-08 structural): `resolve_model` and `effort_pre_args` appear
/// in both the touch path (`touch.rs`) and the refresh path (`refresh.rs`).
///
/// AC-08: `imodel::` and `effort::` parameters must route through the shared
/// `resolve_model()`/`effort_pre_args()` functions in both subprocess dispatch paths.
/// This structural test guards against either path accidentally bypassing model/effort control.
///
/// RED:   one of the four assertions fails (function absent from a path file).
/// GREEN: all four assertions pass (both functions present in both files).
///
/// Spec: [`tests/docs/feature/26_subprocess_model_effort.md` FT-13]
///       [`docs/feature/026_subprocess_model_effort.md` AC-08]
#[ test ]
fn it_ft026_13_imodel_effort_both_paths_structural()
{
  let touch   = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs"   ) );
  let refresh = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/refresh.rs" ) );
  assert!(
    touch.contains( "resolve_model(" ),
    "026 AC-08: touch.rs must call resolve_model() so imodel:: applies to the touch path",
  );
  assert!(
    touch.contains( "effort_pre_args(" ),
    "026 AC-08: touch.rs must call effort_pre_args() so effort:: applies to the touch path",
  );
  assert!(
    refresh.contains( "resolve_model(" ),
    "026 AC-08: refresh.rs must call resolve_model() so imodel:: applies to the refresh path",
  );
  assert!(
    refresh.contains( "effort_pre_args(" ),
    "026 AC-08: refresh.rs must call effort_pre_args() so effort:: applies to the refresh path",
  );
}

/// `it_ft026_14` (026 FT-14 / AC-09): `imodel::` and `effort::` do not affect `format::json` schema.
///
/// Both default `.usage format::json` and `.usage imodel::opus effort::max format::json`
/// must produce identical JSON output. Model/effort selection affects only subprocess
/// invocation; the JSON rendering path is independent.
///
/// Uses a single error account (no accessToken) to get a deterministic offline JSON response.
/// Comparing two invocations on the same fixed credential store guarantees schema identity.
///
/// RED:   params alter JSON output structure (e.g. inject extra keys or change shape).
/// GREEN: both invocations produce byte-identical JSON.
///
/// Spec: [`tests/docs/feature/26_subprocess_model_effort.md` FT-14]
///       [`docs/feature/026_subprocess_model_effort.md` AC-09]
#[ test ]
fn it_ft026_14_imodel_effort_no_effect_on_json_schema()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Error account (no accessToken) → deterministic offline JSON response.
  write_account( dir.path(), "a@x.com", "max", "standard", FAR_FUTURE_MS, false );

  let out_default  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  let out_override = run_cs_with_env(
    &[ ".usage", "imodel::opus", "effort::max", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_default,  0 );
  assert_exit( &out_override, 0 );
  assert_eq!(
    stdout( &out_default ), stdout( &out_override ),
    "026 AC-09: format::json output must be identical regardless of imodel::/effort:: values",
  );
}

// ── BUG-181: trigger inversion fix + structural gates ─────────────────────────

/// it132 (BUG-181 fix AC-02 structural): `apply_touch` trigger uses `is_none()`, not `is_some()`.
///
/// The touch trigger must fire for accounts whose `five_hour.resets_at` is **absent**
/// (idle account — no active 5h window). BUG-181: previous code (`is_some()`) fired for
/// active accounts, wasting subprocess cost while skipping idle accounts that need activation.
///
/// The guard must read: `let is_idle = ...is_none(); if !is_idle { return; }`.
///
/// RED:   source contains `let is_active` (old inverted guard using `is_some()`).
/// GREEN: source contains `let is_idle` + `is_none()` guard.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-11]
///       [`docs/feature/024_session_touch.md` AC-02]
#[ test ]
fn it132_apply_touch_trigger_is_is_none_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs" ) );
  assert!(
    !src.contains( "let is_active = data.five_hour" ),
    "BUG-181: `apply_touch` trigger must use `is_idle` + `is_none()`, not `is_active` + `is_some()`.\n\
     Fix the guard: `let is_idle = data.five_hour.as_ref().and_then(|p| p.resets_at.as_deref()).is_none();\n\
     if !is_idle {{ return; }}`",
  );
}

/// it133 (TSK-192 AC-09 structural): `refresh_account_token` uses `label` variable, not hardcoded `"refresh"`.
///
/// All 14 trace `eprintln!` calls in `refresh_account_token()` must use a `label: &str`
/// parameter so callers can inject `"touch"` or `"refresh"` to distinguish subprocess types
/// in trace output. Currently all calls hardcode `"refresh"` making touch trace indistinguishable.
///
/// RED:   `account.rs` contained `"[trace] refresh  {name}  switch_account: OK"` (hardcoded label).
/// GREEN: all calls use `{label}` variable; both that literal and all `[trace] ` literals are absent (Feature 067).
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-09]
///       [`docs/feature/024_session_touch.md` AC-09]
#[ test ]
fn it133_refresh_account_token_has_label_param_structural()
{
  let src = include_str!( concat!(
    env!( "CARGO_MANIFEST_DIR" ),
    "/../claude_profile_core/src/account.rs"
  ) );
  assert!(
    !src.contains( "[trace] refresh  {name}  switch_account: OK" ),
    "TSK-192: `refresh_account_token()` must accept `label: &str` and use `{{label}}` in all\n\
     trace calls instead of the hardcoded string `\"refresh\"`.\n\
     Note: Feature 067 replaced all `[trace] ` literals with `trace_ts()` calls.",
  );
}

/// it134 (TSK-192 AC-09 structural): `apply_touch` call site passes `"touch"` label.
///
/// The `refresh_account_token()` call in `apply_touch()` must pass the literal `"touch"`
/// as the `label` argument so trace output reads `YYYY-MM-DD · HH:MM:SS · touch ...` (not `… refresh …`).
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-09]
///       [`docs/feature/024_session_touch.md` AC-09]
#[ test ]
fn it134_apply_touch_passes_touch_label_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs" ) );
  assert!(
    src.contains( r#"credential_store, claude_paths, trace, "touch","# ),
    "TSK-192: `apply_touch()` must pass `\"touch\"` as the label argument to `refresh_account_token()`."
  );
}

/// it135 (TSK-192 AC-09 structural): `apply_refresh` call site passes `"refresh"` label.
///
/// The `refresh_account_token()` call in `apply_refresh()` must pass the literal `"refresh"`
/// as the `label` argument so trace output reads `YYYY-MM-DD · HH:MM:SS · refresh ...`.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-09]
///       [`docs/feature/024_session_touch.md` AC-09]
#[ test ]
fn it135_apply_refresh_passes_refresh_label_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/refresh.rs" ) );
  assert!(
    src.contains( r#"credential_store, claude_paths, trace, "refresh","# ),
    "TSK-192: `apply_refresh()` must pass `\"refresh\"` as the label argument to `refresh_account_token()`."
  );
}

/// it136 (TSK-192 AC-09 structural): `refresh_account_token` has per-step `Instant` timing.
///
/// Both `switch_account` and `run_isolated` steps in `refresh_account_token()` must be
/// wrapped with `std::time::Instant::now()` so elapsed seconds appear in trace output.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-09]
///       [`docs/feature/024_session_touch.md` AC-09]
#[ test ]
fn it136_refresh_account_token_has_instant_timing_structural()
{
  let src = include_str!( concat!(
    env!( "CARGO_MANIFEST_DIR" ),
    "/../claude_profile_core/src/account.rs"
  ) );
  assert!(
    src.contains( "Instant::now()" ),
    "TSK-192: `refresh_account_token()` must use `std::time::Instant::now()` for per-step timing."
  );
}

// ── TSK-220 — sort default renew + sort::next meta-strategy ──────────────────

/// it137 (TSK-220 AC-01 structural): sort default is `SortStrategy::Renew` when no `sort::` arg.
///
/// `parse_usage_params` must return `SortStrategy::Renew` when the `sort` argument is absent.
/// This ensures `clp .usage` (no `sort::` flag) orders rows by 7d reset — soonest weekly reset first.
///
/// RED:   `None => SortStrategy::Drain` (old default).
/// GREEN: `None => SortStrategy::Renew` present in parse block.
///
/// Spec: [`tests/docs/feature/020_usage_sort_strategies.md` FT-14]
///       [`docs/feature/020_usage_sort_strategies.md` AC-01]
#[ test ]
fn it137_sort_default_is_renew_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/params.rs" ) );
  // The None arm of the sort match uses alignment spaces; verify Renew is the default and Drain is not.
  assert!(
    src.contains( "None                         => SortStrategy::Renew" ),
    "TSK-220: sort default must be SortStrategy::Renew, not SortStrategy::Drain.\n\
     Change the None arm of the sort argument match to `None => SortStrategy::Renew`."
  );
}

// it138 (TSK-193 AC-15 structural): `sort::next` resolves to `SortStrategy::Drain` when `next::drain`.
// it138/it139 (sort::next resolves to Drain/Endurance structural) removed after Feature 037/038
// — NextStrategy, SortStrategy::Next, and next:: parameter all removed.

/// it141 (BUG-202 / 024 FT-14): errored account emits skip trace in touch phase.
///
/// ## Root Cause
///
/// `apply_touch()` error guard at `usage.rs:1497` (`let Ok(ref data) = aq.result
/// else { return; }`) exited before any trace emission point. Error-tier accounts
/// silently vanished from the touch phase trace while appearing in fetch and refresh.
///
/// ## Why Not Caught
///
/// TSK-196 (BUG-177) added trace for the is_active/h-exhausted guard at lines
/// 1504-1511 but did not address the error guard at line 1497. The BUG-177 MRE
/// used OK-result accounts only.
///
/// ## Fix Applied
///
/// Added `if trace { eprintln!("{}touch  {}  skipped (reason: error account)", trace_ts(), aq.name); }`
/// before the `return` in the `else` branch (now uses `trace_ts()` per Feature 067).
///
/// ## Prevention
///
/// When adding trace to a function with multiple early-return guards, each guard
/// needs its own trace emission — audit ALL return paths, not just the "interesting" ones.
///
/// ## Pitfall
///
/// Error guard was deemed uninteresting (error accounts can't be touched) but the
/// diagnostic contract requires visibility into all skip decisions.
///
/// RED:   errored account has no touch trace line → assert fails.
/// GREEN: error guard emits `YYYY-MM-DD · HH:MM:SS · touch  <name>  skipped (reason: error account)`.
#[ test ]
fn it141_trace_skip_lines_emitted_for_non_qualifying_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account with FAR_FUTURE_MS but no accessToken → quota fetch fails → Err result
  write_account( dir.path(), "err@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "touch::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( " · touch  err@x.com  skipped (reason: error account)" ),
    "BUG-202: errored account must emit touch trace skipped (reason: error account) \
     when trace=true (AC-09/AC-12 of Feature 024). Got stderr:\n{err}",
  );
}

// ── TSK-209: haiku model + low/normal effort CLI acceptance ───────────────────

/// it142 (EC-11 / 035): `imodel::haiku` accepted with empty credential store exits 0.
///
/// Before TSK-209: `imodel::haiku` is unrecognised → `ArgumentTypeMismatch` → exit 1.
/// After TSK-209:  `haiku` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/cli/param/035_imodel.md` EC-11]
#[ test ]
fn it142_imodel_haiku_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "imodel::haiku" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it143 (EC-10 / 036): `effort::low` accepted with empty credential store exits 0.
///
/// Before TSK-209: `effort::low` is unrecognised → `ArgumentTypeMismatch` → exit 1.
/// After TSK-209:  `low` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-10]
#[ test ]
fn it143_effort_low_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "effort::low" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it144 (EC-11 / 036): `effort::normal` accepted with empty credential store exits 0.
///
/// Before TSK-209: `effort::normal` is unrecognised → `ArgumentTypeMismatch` → exit 1.
/// After TSK-209:  `normal` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/cli/param/036_effort.md` EC-11]
#[ test ]
fn it144_effort_normal_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "effort::normal" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

// ── sort::renew strategy ──────────────────────────────────────────────────────

/// it145 `lim_it` (feature/020): `sort::renew` (default) places `→` and shows footer.
///
/// `sort::renew` (default strategy) selects the account whose soonest running reset
/// timer (min of 5h and 7d) fires first. Footer shows one recommendation line.
///
/// Spec: [`docs/feature/020_usage_sort_strategies.md` AC-09]
#[ doc = "lim_it" ]
#[ test ]
fn it145_lim_it_sort_renew_places_arrow_on_soonest_refill()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it145: no live token — skipping" );
    return;
  };
  require_live_api( "it145" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // acct-a is is_current (token matches credentials); acct-b uses fake token → eligible as Next.
  write_live_credentials_with_token( dir.path(), &token );
  write_account_with_token( dir.path(), "acct-a@test.com", &token,       true  );
  write_account_with_token( dir.path(), "acct-b@test.com", "fake-token", false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "renew" ),
    "footer must show renew strategy line (020/AC-09), got:\n{text}",
  );
}

// ── row filtering parameters (TSK-223) ────────────────────────────────────────

/// ut146 (TSK-223 RED gate): `only_valid::1` accepted; empty store exits 0.
///
/// Before TSK-224: `get` is unregistered → `ArgumentUnrecognised` → exit 1.
/// After TSK-224:  `get::7d_left` accepted; empty store → no rows → bare empty output → exit 0.
///
/// Validates AC-10 structural (no table chrome in output when `get::` is set).
/// Live extraction tests (`lim_it`) cover the actual value output.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md`]
///       [`docs/feature/028_usage_row_filtering.md` AC-10]
#[ test ]
fn ut_get_7d_left_extracts_bare_value()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "get::7d_left" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Quota" ) && !text.contains( "5h Left" ) && !text.contains( "7d Left" ),
    "get::7d_left with empty store must produce no table output, got:\n{text}",
  );
}

/// Before TSK-224: `get::bogus_field` unregistered → wrong exit/message.
/// After TSK-224:  exit 1, stderr lists valid field IDs including `5h_left`, `7d_left`, `account`.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md`]
///       [`docs/feature/028_usage_row_filtering.md` AC-15]
#[ test ]
fn ut_get_invalid_field_exits_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "get::bogus_field" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "5h_left" ) && err.contains( "7d_left" ) && err.contains( "account" ),
    "get::bogus_field must list valid field IDs in stderr, got:\n{err}",
  );
}

/// Before TSK-223: `only_valid` is unregistered → `ArgumentUnrecognised` → exit 1.
/// After TSK-223:  `only_valid::1` accepted, empty store → no-accounts message → exit 0.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md`]
///       [`docs/feature/028_usage_row_filtering.md` AC-07]
#[ test ]
fn ut_filter_only_valid_hides_red_rows()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "only_valid::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "only_valid::1 with empty store must show no-accounts message, got:\n{text}",
  );
}

// ── → Next column + JSON renewal fields (Phase 3 RED gate — TSK-227) ─────────

/// it146 — `→ Next` column header visible in default `.usage` output.
///
/// Before TSK-227: `→ Next` column does not exist → assertion fails.
/// After TSK-227:  `→ Next` header appears in every default table output.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
/// Source: [`009_token_usage.md` AC-28]
#[ test ]
fn it146_next_column_visible_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "\u{2192} Next" ),
    "default .usage output must contain '→ Next' column header (AC-28), got:\n{text}",
  );
}

/// it147 — `format::json` output has all 4 renewal/next-event fields; deprecated field absent.
///
/// Before TSK-227: JSON uses `next_renewal_est` → `renewal_secs` assertion fails.
/// After TSK-227:  JSON has `renewal_secs`, `renewal_is_estimate`, `next_event_type`,
///                 `next_event_secs`; `next_renewal_est` is removed.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-19]
/// Source: [`009_token_usage.md` AC-29]
#[ test ]
fn it147_json_renewal_secs_present()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // All four AC-29 fields must be present.
  assert!(
    text.contains( "\"renewal_secs\"" ),
    "format::json must include 'renewal_secs' field (AC-29), got:\n{text}",
  );
  assert!(
    text.contains( "\"renewal_is_estimate\"" ),
    "format::json must include 'renewal_is_estimate' field (AC-29), got:\n{text}",
  );
  assert!(
    text.contains( "\"next_event_type\"" ),
    "format::json must include 'next_event_type' field (AC-29), got:\n{text}",
  );
  assert!(
    text.contains( "\"next_event_secs\"" ),
    "format::json must include 'next_event_secs' field (AC-29), got:\n{text}",
  );
  // Deprecated field must be gone.
  assert!(
    !text.contains( "\"next_renewal_est\"" ),
    "format::json must NOT contain deprecated 'next_renewal_est' field, got:\n{text}",
  );
}

// ── IT-40: Status emoji column header ─────────────────────────────────────────

/// IT-40: Table header row contains `●` column label.
///
/// An account with no accessToken (error row) still causes the table to render;
/// the `●` header is always present in the status emoji column.
///
/// Source: [`009_token_usage.md` AC-18](../docs/feature/009_token_usage.md)
#[ test ]
fn it148_status_emoji_column_header_present()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-token", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( '●' ),
    "table header must contain '●' status emoji column label, got:\n{text}",
  );
}

// ── IT-41: Error account shows 🔴 ─────────────────────────────────────────────

/// IT-41: Account with missing token shows `🔴` in table row.
///
/// `write_account()` writes a credential file without `accessToken`; the
/// fetch result is `Err(_)` → `status_emoji` returns `🔴`.
///
/// Source: [`009_token_usage.md` AC-18](../docs/feature/009_token_usage.md)
#[ test ]
fn it149_status_emoji_red_on_token_error()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-token", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "🔴" ),
    "account without accessToken must show 🔴 in table row, got:\n{text}",
  );
}

// ── IT-42: JSON output contains no status emoji ────────────────────────────────

/// IT-42: `format::json` output does not contain status emoji.
///
/// Emoji are a table-rendering concern only; JSON output must be clean.
///
/// Source: [`009_token_usage.md` AC-20](../docs/feature/009_token_usage.md)
#[ test ]
fn it150_status_emoji_absent_from_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-token", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "🔴" ) && !text.contains( "🟡" ) && !text.contains( "🟢" ),
    "format::json must NOT contain status emoji, got:\n{text}",
  );
}

// ── it151: past _renewal_at auto-advances at render (030 FT-10) ──────────────

/// it151 — Past `_renewal_at` is auto-advanced monthly at render; shows `in Xd` (no `~`).
///
/// Root Cause: `renews_label()` advances past timestamps by 30-day increments until future.
/// The stored value is unchanged; auto-advance is a read-only render-time operation.
///
/// Setup: Account has `_renewal_at: "2020-03-15T00:00:00Z"` (deeply past). After auto-advance
/// the next day-15 occurrence lands within 30 days of today. No live credentials needed —
/// the account will be in error state but the `~Renews` column is populated from stored data.
///
/// Spec: [`tests/docs/feature/030_account_renewal_override.md` FT-10]
/// Source: [`030_account_renewal_override.md` AC-10]
#[ test ]
fn it151_past_renewal_at_auto_advances_in_usage()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "past@renewal.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_renewal_json( dir.path(), "past@renewal.com", "2020-03-15T00:00:00Z" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // Find the ~Renews cell for this account by locating the TSV equivalent or
  // parsing the text table row that contains the account name.
  // The column header is "~Renews". Find the row for our account.
  let renews_line = text.lines()
    .find( |l| l.contains( "past@renewal.com" ) )
    .expect( "usage output must have a row for past@renewal.com" );

  // The ~Renews column must show `in Xd` (no ~ prefix) because the timestamp is
  // an exact override stored via `_renewal_at`, even after auto-advance.
  // "in " prefix (no ~) and contains "d" for days.
  assert!(
    renews_line.contains( "in " ) && !renews_line.contains( "~in " ),
    "past _renewal_at must auto-advance and show 'in Xd' (no '~'), got row:\n{renews_line}\nfull output:\n{text}",
  );

  // The file on disk must NOT have been modified — auto-advance is read-only.
  let store   = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let on_disk = std::fs::read_to_string( store.join( "past@renewal.com.json" ) ).unwrap();
  assert!(
    on_disk.contains( "2020-03-15T00:00:00Z" ),
    "stored _renewal_at must NOT be modified by render-time auto-advance, got: {on_disk}",
  );
}

// ── it152: TSV format has `next` column header (AC-28) ───────────────────────

/// it152 — `format::tsv` output contains a `next` column header (→ Next column in text).
///
/// The TSV renderer emits `next` as the header for the `→ Next` column (AC-28).
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-18]
/// Source: [`009_token_usage.md` AC-28]
#[ test ]
fn it152_tsv_next_column_present()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "format::tsv" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let header = text.lines().next().expect( "TSV must have a header row" );
  let cols   : Vec< &str > = header.split( '\t' ).collect();
  assert!(
    cols.contains( &"next" ),
    "TSV header must contain 'next' column (AC-28), got cols: {cols:?}",
  );
}

// ── it153: JSON all 4 renewal fields with _renewal_at set ───────────────────

/// it153 — `format::json` with `_renewal_at` set produces all 4 renewal fields with
/// correct types: `renewal_secs` (integer), `renewal_is_estimate: false`,
/// `next_event_type` (string), `next_event_secs` (integer).
///
/// Complements it147 (which checks field presence only); this test checks
/// the semantic content when `_renewal_at` is explicitly set to a future timestamp.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-19]
/// Source: [`009_token_usage.md` AC-29]
#[ test ]
fn it153_json_renewal_fields_with_renewal_at()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "alice", "max", "default", FAR_FUTURE_MS, false );
  // Set a future _renewal_at so renewal_is_estimate=false and renewal_secs is a real integer.
  write_account_renewal_json( dir.path(), "alice", "2099-01-01T00:00:00Z" );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // renewal_is_estimate must be false (not true) because _renewal_at is set explicitly.
  assert!(
    text.contains( "\"renewal_is_estimate\":false" ),
    "explicit _renewal_at must yield renewal_is_estimate:false, got:\n{text}",
  );
  // renewal_secs must be a non-null integer (not null).
  assert!(
    text.contains( "\"renewal_secs\":" ) && !text.contains( "\"renewal_secs\":null" ),
    "explicit _renewal_at must yield non-null renewal_secs, got:\n{text}",
  );
  // next_event_type must be a string (not null).
  assert!(
    text.contains( "\"next_event_type\":" ) && !text.contains( "\"next_event_type\":null" ),
    "with _renewal_at set, next_event_type must not be null, got:\n{text}",
  );
  // next_event_secs must be a non-null integer.
  assert!(
    text.contains( "\"next_event_secs\":" ) && !text.contains( "\"next_event_secs\":null" ),
    "with _renewal_at set, next_event_secs must not be null, got:\n{text}",
  );
}


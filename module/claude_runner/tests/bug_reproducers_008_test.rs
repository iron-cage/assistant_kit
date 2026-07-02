//! Bug reproducer — BUG-008
//!
//! `clp .model.select` pin silently ignored by `clr run` and `clr ask`.
//!
//! ## Root Cause
//!
//! `read_subprocess_model_pref()` was only wired into `run_isolated_ext()`; `dispatch_run()`
//! and `dispatch_ask()` never read `~/.clr/prefs.json`. The function was also private (`fn`,
//! not `pub fn`) to `claude_runner_core`, blocking cross-crate access.
//!
//! ## Why Not Caught
//!
//! `run_isolated_ext()` called `read_subprocess_model_pref()` correctly, so manual tests of
//! isolated mode appeared to confirm the full integration. No cross-crate integration test existed
//! for `dispatch_run()` or `dispatch_ask()` with a non-empty `prefs.json`, because the function
//! was `fn` (private) — unreachable from outside `claude_runner_core`. The `.model.select`
//! feature test verified only the write path (preference stored in `prefs.json`); it did not
//! verify that `clr run` subsequently honored the stored preference.
//!
//! ## Fix Applied
//!
//! `read_subprocess_model_pref()` exposed as `pub fn` and re-exported from `claude_runner_core`.
//! `dispatch_run()` now reads the preference after `apply_env_vars()`, guarded by
//! `cli.model.is_none()` so an explicit `--model` flag or `CLR_MODEL` env var always wins.
//! `dispatch_ask()` is a pure alias calling `dispatch_run()` — fixed automatically.
//!
//! ## Prevention
//!
//! When a preference-reading function is added to one dispatch path, all other paths honoring
//! the same preference must be updated in the same change — otherwise partial implementation
//! creates a misleading success message with no actual effect on the uncovered paths.
//! See fix comment in `module/claude_runner/src/cli/mod.rs` `dispatch_run()`.
//!
//! ## Pitfall
//!
//! A `pub fn` boundary in a library crate is the only structural signal that a function is
//! part of the public contract and reachable from cross-crate tests. A private function can
//! be exercised by one internal call site while remaining invisible to all others — the
//! coverage gap is structural, not incidental, and will not show in line-coverage metrics
//! that only see the one wired path. When adding preference-reading or environment-reading
//! behavior to any dispatch path, explicitly audit all other executing dispatch paths
//! (`dispatch_run`, `dispatch_ask`, `run_isolated_ext`) for the same wiring in the same PR.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli_with_env;

// ── BUG-008-1: prefs.json pin injected as --model when no explicit --model ────

/// BUG-008: `dispatch_run_uses_pinned_model_from_prefs_json`
///
/// Without `--model` flag, the model stored in `~/.clr/prefs.json` under
/// `subprocess_model` is injected as `--model` in the assembled command.
#[ test ]
fn dispatch_run_uses_pinned_model_from_prefs_json()
{
  let home_dir = tempfile::TempDir::new().expect( "failed to create temp HOME dir" );
  let clr_dir  = home_dir.path().join( ".clr" );
  std::fs::create_dir_all( &clr_dir ).expect( "failed to create .clr dir" );
  std::fs::write(
    clr_dir.join( "prefs.json" ),
    r#"{"subprocess_model":"claude-opus-4-6"}"#,
  )
  .expect( "failed to write prefs.json" );

  let home_str = home_dir.path().to_str().expect( "HOME path must be valid UTF-8" );
  let out      = run_cli_with_env( &[ "--dry-run", "Fix bug" ], &[ ( "HOME", home_str ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout   = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--model" ),
    "assembled command must contain --model when prefs.json pins a model. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "claude-opus-4-6" ),
    "assembled command must contain the pinned model value. Got:\n{stdout}"
  );
}

// ── BUG-008-2: explicit --model flag wins over prefs.json ─────────────────────

/// BUG-008: explicit `--model` CLI flag wins over `~/.clr/prefs.json` pin.
///
/// The `cli.model.is_none()` guard ensures `apply_env_vars()` / CLI flag values
/// always take precedence over the preference file.
#[ test ]
fn dispatch_run_explicit_model_flag_wins_over_pref()
{
  let home_dir = tempfile::TempDir::new().expect( "failed to create temp HOME dir" );
  let clr_dir  = home_dir.path().join( ".clr" );
  std::fs::create_dir_all( &clr_dir ).expect( "failed to create .clr dir" );
  std::fs::write(
    clr_dir.join( "prefs.json" ),
    r#"{"subprocess_model":"claude-opus-4-6"}"#,
  )
  .expect( "failed to write prefs.json" );

  let home_str = home_dir.path().to_str().expect( "HOME path must be valid UTF-8" );
  let out      = run_cli_with_env(
    &[ "--dry-run", "--model", "claude-sonnet-5", "Fix bug" ],
    &[ ( "HOME", home_str ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "claude-sonnet-5" ),
    "explicit --model value must appear in assembled command. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "claude-opus-4-6" ),
    "prefs.json model must NOT override explicit --model flag. Got:\n{stdout}"
  );
}

// ── BUG-008-4: CLR_MODEL env var (tier 3) beats prefs.json pin (tier 4) ───────

/// BUG-008: `CLR_MODEL` env var wins over `~/.clr/prefs.json` pin.
///
/// Precedence: CLI flag > `CLR_MODEL` > `prefs.json`. `apply_env_vars()` sets `cli.model` from
/// `CLR_MODEL` before the prefs-read block fires; the `is_none()` guard then skips prefs.
#[ test ]
fn dispatch_run_clr_model_env_var_beats_prefs_json()
{
  let home_dir = tempfile::TempDir::new().expect( "failed to create temp HOME dir" );
  let clr_dir  = home_dir.path().join( ".clr" );
  std::fs::create_dir_all( &clr_dir ).expect( "failed to create .clr dir" );
  std::fs::write(
    clr_dir.join( "prefs.json" ),
    r#"{"subprocess_model":"claude-opus-4-6"}"#,
  )
  .expect( "failed to write prefs.json" );

  let home_str = home_dir.path().to_str().expect( "HOME path must be valid UTF-8" );
  let out      = run_cli_with_env(
    &[ "--dry-run", "Fix bug" ],
    &[ ( "HOME", home_str ), ( "CLR_MODEL", "claude-haiku-4-5" ) ],
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "claude-haiku-4-5" ),
    "CLR_MODEL must win over prefs.json pin. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "claude-opus-4-6" ),
    "prefs.json pin must NOT override CLR_MODEL env var. Got:\n{stdout}"
  );
}

// ── BUG-008-3: no --model injected when prefs.json absent ─────────────────────

/// BUG-008: without `prefs.json`, no `--model` flag is injected.
///
/// `read_subprocess_model_pref()` returns `None` when `~/.clr/prefs.json` is absent —
/// the assembled command must not include `--model`.
#[ test ]
fn dispatch_run_no_model_injected_when_prefs_absent()
{
  let home_dir = tempfile::TempDir::new().expect( "failed to create temp HOME dir" );
  let home_str = home_dir.path().to_str().expect( "HOME path must be valid UTF-8" );
  let out      = run_cli_with_env( &[ "--dry-run", "Fix bug" ], &[ ( "HOME", home_str ) ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout   = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--model" ),
    "no --model must be injected when prefs.json is absent. Got:\n{stdout}"
  );
}

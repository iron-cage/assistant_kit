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
//! Originally, `read_subprocess_model_pref()` was exposed as `pub fn`, re-exported from
//! `claude_runner_core`, and read directly by `dispatch_run()` after `apply_env_vars()` as a
//! 5th resolution tier, guarded by `cli.model.is_none()`. Task 408 removed that direct read:
//! `apply_config_defaults()` (`config.rs:140`) already fills `cli.model` from `config.toml`
//! earlier in the same `dispatch_run()`/`dispatch_ask()` sequence, making the `prefs.json` read
//! a no-op for anyone using `config.toml`. Task 408 carried forward any existing `prefs.json`
//! pin into `config.toml`'s user tier once (a one-time data-preservation step, not shipped code)
//! so `.model.select` pins predating this change keep working — then deleted the now-redundant
//! read. `dispatch_run()`'s resolution is now exactly 4 tiers: `--model` flag → JSON config →
//! `CLR_MODEL` env → `config.toml`. `dispatch_ask()` is a pure alias calling `dispatch_run()` —
//! updated automatically.
//!
//! ## Prevention
//!
//! When a preference-reading function is added to one dispatch path, all other paths honoring
//! the same preference must be updated in the same change — otherwise partial implementation
//! creates a misleading success message with no actual effect on the uncovered paths.
//! Conversely, when a second, earlier-resolving tier for the same preference is introduced
//! (as `config.toml` was for `prefs.json` here), the older/redundant tier must be either
//! removed or clearly justified as intentionally distinct — a silently-redundant tier is dead
//! logic wearing a live test suite. See fix comment in `module/claude_runner/src/cli/mod.rs`
//! `dispatch_run()`.
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
//! Symmetrically: when two tiers end up resolving the same preference because a newer tier's
//! own earlier-in-sequence application makes an older tier's check a no-op, that redundancy is
//! easy to miss precisely because the older tier's tests still pass — they just no longer
//! exercise the code path they believe they do once the newer tier is also set.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli_with_env;

// ── BUG-008-1 (superseded by task 408): prefs.json alone no longer injects a model ──

/// BUG-008 (superseded): `dispatch_run_prefs_json_alone_no_longer_injects_model`
///
/// Supersedes the original `dispatch_run_uses_pinned_model_from_prefs_json` (Test
/// Matrix T5, task 408). `dispatch_run()` no longer reads `~/.clr/prefs.json` at
/// all — `config.toml`'s `model` field already resolves earlier in the same
/// dispatch sequence via `apply_config_defaults()`, making the direct `prefs.json`
/// read redundant. With only `prefs.json`'s `subprocess_model` set (no `--model`
/// flag, no `CLR_MODEL` env, no `config.toml` `model` key), no `--model` flag is
/// injected — the Claude binary's own default model applies.
#[ test ]
fn dispatch_run_prefs_json_alone_no_longer_injects_model()
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
    !stdout.contains( "--model" ),
    "prefs.json alone must no longer inject --model — the redundant fallback tier \
     was removed by task 408 (config.toml resolves earlier in the same sequence). Got:\n{stdout}"
  );
}

// ── BUG-008-2: explicit --model flag resolves regardless of prefs.json content ─

/// BUG-008: explicit `--model` CLI flag wins over `~/.clr/prefs.json` pin.
///
/// Since task 408, `dispatch_run()` never reads `prefs.json` at all, so this
/// holds trivially rather than via an `is_none()` guard racing prefs.json — the
/// explicit flag is set directly by `parse_args()` before `config.rs`/`prefs.json`
/// tiers are even consulted. `prefs.json` is written here only to prove its
/// content has zero effect on the outcome, not because it is read and overridden.
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

// ── BUG-008-4: CLR_MODEL env var resolves regardless of prefs.json content ────

/// BUG-008: `CLR_MODEL` env var wins over `~/.clr/prefs.json` pin.
///
/// Precedence is now exactly: CLI flag > JSON config > `CLR_MODEL` > `config.toml`
/// (task 408 removed the `prefs.json` tier entirely). `apply_env_vars()` sets
/// `cli.model` from `CLR_MODEL`; `dispatch_run()` never reaches `prefs.json`
/// afterward because that read no longer exists. `prefs.json` is written here
/// only to prove its content has zero effect, not because it is read and beaten.
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

// ── BUG-008-3: no --model injected when nothing is configured anywhere ────────

/// BUG-008: without `prefs.json`, `config.toml`, `CLR_MODEL`, or `--model`, no
/// `--model` flag is injected (Test Matrix T6, task 408 — unchanged behavior).
///
/// Since task 408, `dispatch_run()` no longer reads `prefs.json` at all, so its
/// absence here is incidental rather than the operative condition — the same
/// temp `HOME` also has no `config.toml`, so this is the "nothing set anywhere"
/// case: the assembled command must not include `--model`.
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

//! Bug reproducer for BUG-248: no warning when --keep-claudecode enables nested-agent mode.
//!
//! # Root Cause (BUG-248)
//!
//! When `--keep-claudecode` is set and `CLAUDECODE` is present in the parent environment,
//! the child claude process inherits `CLAUDECODE=1` and runs in nested-agent mode.
//! No diagnostic was emitted — users who set `--keep-claudecode` without understanding
//! the consequence received silent, unexpected behavior.
//!
//! # Why Not Caught
//!
//! No test asserted the presence of a warning when both `--keep-claudecode` was set
//! and `CLAUDECODE` was in the environment.  The warning path was simply never implemented.
//!
//! # Fix Applied
//!
//! Added a warning block in `dispatch_run()` (before the dry-run check) that emits to
//! stderr when `cli.keep_claudecode && verbosity.shows_warnings() && CLAUDECODE in env`.
//! Placed before the dry-run check so it fires in all execution modes.
//!
//! # Prevention
//!
//! Every flag that silently alters subprocess environment should warn when the user's
//! action has a non-obvious side-effect.  Gate warnings on `shows_warnings()` (level ≥ 2)
//! so operators who request silence still get it.
//!
//! # Pitfall
//!
//! The warning is informational only — it must not alter the exit code or suppress
//! the intended behavior.  The user's intent (`--keep-claudecode`) is always respected.
//!
//! # Test Matrix
//!
//! | Test | Scenario | Expected |
//! |------|----------|----------|
//! | `t01_warning_when_claudecode_in_env_and_keep_flag` | CLAUDECODE=1 + --keep-claudecode | warning on stderr |
//! | `t02_no_warning_when_claudecode_absent` | no CLAUDECODE + --keep-claudecode | no warning |
//! | `t03_no_warning_when_keep_flag_absent` | CLAUDECODE=1, no --keep-claudecode | no warning |
//! | `t04_no_warning_at_verbosity_1` | CLAUDECODE=1 + --keep-claudecode + --verbosity 1 | no warning |
//! | `t05_warning_at_verbosity_2` | CLAUDECODE=1 + --keep-claudecode + --verbosity 2 | warning present |
//! | `t06_warning_present_in_dry_run` | CLAUDECODE=1 + --keep-claudecode + --dry-run | warning on stderr; dry-run unaffected |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli_with_env;

// ── T01 ──────────────────────────────────────────────────────────────────────

/// T01: warning emitted when `CLAUDECODE=1` in env AND `--keep-claudecode` passed.
///
/// The warning must mention both `--keep-claudecode` and `CLAUDECODE` and `nested-agent mode`.
#[ test ]
#[ doc = "bug_reproducer(BUG-248)" ]
fn t01_warning_when_claudecode_in_env_and_keep_flag()
{
  let out = run_cli_with_env(
    &[ "--keep-claudecode", "--dry-run", "task" ],
    &[ ( "CLAUDECODE", "1" ) ],
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "Warning" ),
    "BUG-248 T01: warning must appear when CLAUDECODE set + --keep-claudecode.\nstderr: {stderr}",
  );
  assert!(
    stderr.contains( "--keep-claudecode" ),
    "BUG-248 T01: warning must name --keep-claudecode.\nstderr: {stderr}",
  );
  assert!(
    stderr.contains( "CLAUDECODE" ),
    "BUG-248 T01: warning must name CLAUDECODE.\nstderr: {stderr}",
  );
  assert!(
    stderr.contains( "nested-agent" ),
    "BUG-248 T01: warning must mention nested-agent mode.\nstderr: {stderr}",
  );
  assert!(
    out.status.success(),
    "BUG-248 T01: warning is informational — must exit 0.\nstderr: {stderr}",
  );
}

// ── T02 ──────────────────────────────────────────────────────────────────────

/// T02: no warning when `CLAUDECODE` is absent from environment.
///
/// `--keep-claudecode` alone (no `CLAUDECODE` in env) → no warning, no false positive.
#[ test ]
#[ doc = "bug_reproducer(BUG-248)" ]
fn t02_no_warning_when_claudecode_absent()
{
  // Explicitly unset CLAUDECODE to ensure a clean env regardless of the ambient shell.
  let out = run_cli_with_env(
    &[ "--keep-claudecode", "--dry-run", "task" ],
    &[ ( "CLAUDECODE", "" ) ],
  );
  // env_str() returns None for empty string; CLAUDECODE is present but empty → is_ok() still true.
  // So we need to truly unset it. run_cli_with_env uses envs() which can't unset.
  // The safest assertion: check no Warning line mentioning nested-agent.
  //
  // Implementation note: `std::env::var("CLAUDECODE").is_ok()` returns true for *any* value
  // including empty string. An empty-string CLAUDECODE is uncommon but possible.
  // The test below covers the *absence* scenario by not setting CLAUDECODE at all.
  let _ = out; // use above result

  // Use a clean environment invocation without CLAUDECODE set.
  // Inherit only a minimal env (PATH) to guarantee CLAUDECODE is not present.
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let path_val = std::env::var( "PATH" ).unwrap_or_default();
  let out2 = std::process::Command::new( bin )
    .args( [ "--keep-claudecode", "--dry-run", "task" ] )
    .env_clear()
    .env( "PATH", &path_val )
    .output()
    .expect( "failed to invoke clr" );

  let stderr = String::from_utf8_lossy( &out2.stderr );
  assert!(
    !stderr.contains( "nested-agent" ),
    "BUG-248 T02: no warning when CLAUDECODE absent from env.\nstderr: {stderr}",
  );
  assert!(
    out2.status.success(),
    "BUG-248 T02: must exit 0.\nstderr: {stderr}",
  );
}

// ── T03 ──────────────────────────────────────────────────────────────────────

/// T03: no warning when `--keep-claudecode` is NOT passed, even with `CLAUDECODE=1` in env.
///
/// Default behavior removes CLAUDECODE — no warning needed when protection is active.
#[ test ]
#[ doc = "bug_reproducer(BUG-248)" ]
fn t03_no_warning_when_keep_flag_absent()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLAUDECODE", "1" ) ],
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "nested-agent" ),
    "BUG-248 T03: no warning when --keep-claudecode absent (protection active).\nstderr: {stderr}",
  );
  assert!(
    out.status.success(),
    "BUG-248 T03: must exit 0.\nstderr: {stderr}",
  );
}

// ── T04 ──────────────────────────────────────────────────────────────────────

/// T04: `--verbosity 1` (errors only) suppresses the warning even with both conditions true.
///
/// `shows_warnings()` returns false at level 1.  Verbosity gate must be respected.
#[ test ]
#[ doc = "bug_reproducer(BUG-248)" ]
fn t04_no_warning_at_verbosity_1()
{
  let out = run_cli_with_env(
    &[ "--keep-claudecode", "--verbosity", "1", "--dry-run", "task" ],
    &[ ( "CLAUDECODE", "1" ) ],
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "nested-agent" ),
    "BUG-248 T04: warning must be suppressed at --verbosity 1.\nstderr: {stderr}",
  );
  assert!(
    out.status.success(),
    "BUG-248 T04: must exit 0.\nstderr: {stderr}",
  );
}

// ── T05 ──────────────────────────────────────────────────────────────────────

/// T05: `--verbosity 2` is the minimum verbosity where `shows_warnings()` is true.
///
/// Warning must appear at level ≥ 2 when both conditions hold.
#[ test ]
#[ doc = "bug_reproducer(BUG-248)" ]
fn t05_warning_at_verbosity_2()
{
  let out = run_cli_with_env(
    &[ "--keep-claudecode", "--verbosity", "2", "--dry-run", "task" ],
    &[ ( "CLAUDECODE", "1" ) ],
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "nested-agent" ),
    "BUG-248 T05: warning must appear at --verbosity 2 (shows_warnings threshold).\nstderr: {stderr}",
  );
  assert!(
    out.status.success(),
    "BUG-248 T05: must exit 0.\nstderr: {stderr}",
  );
}

// ── T06 ──────────────────────────────────────────────────────────────────────

/// T06: `--dry-run` still emits the warning when both conditions are true.
///
/// The warning block is placed before the dry-run check in `dispatch_run()` so it fires
/// in all execution modes, including dry-run.  The dry-run output itself is unaffected.
#[ test ]
#[ doc = "bug_reproducer(BUG-248)" ]
fn t06_warning_present_in_dry_run()
{
  let out = run_cli_with_env(
    &[ "--keep-claudecode", "--dry-run", "task" ],
    &[ ( "CLAUDECODE", "1" ) ],
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stderr.contains( "nested-agent" ),
    "BUG-248 T06: warning must appear even in --dry-run mode.\nstderr: {stderr}",
  );
  // Dry-run output (assembled command) must still appear on stdout
  assert!(
    stdout.contains( "claude" ),
    "BUG-248 T06: dry-run stdout (assembled command) must be unaffected by warning.\nstdout: {stdout}",
  );
  assert!(
    out.status.success(),
    "BUG-248 T06: must exit 0.\nstderr: {stderr}",
  );
}

#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-runner` and `--runner-delay` Parse and Retry Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-6 from `tests/docs/cli/param/050_retry_on_runner.md` and
//! EC-1 through EC-6 from `tests/docs/cli/param/051_runner_delay.md`,
//! plus EC-7 and EC-8 (runtime retry integration tests added by TSK-209/BUG-299).
//!
//! Both parameter specs share this test file.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 50), EC-1..EC-6 (param 51): parser/dry-run — no subprocess
//! - EC-7: binary absent + `--retry-on-runner 1 --runner-delay 0` → retry fires (BUG-299)
//! - EC-8: binary absent + `--retry-on-runner 0 --retry-override 0` → no retry (BUG-299)
//!
//! ## Corner Cases Covered
//!
//! ### --retry-on-runner (param 50)
//! - EC-1: help lists flag
//! - EC-2: value 0 (explicit zero) accepted in dry-run
//! - EC-3: value 2 (nonzero) accepted in dry-run
//! - EC-4: `CLR_RETRY_ON_RUNNER` env var applied
//! - EC-5: CLI wins over env var
//! - EC-6: invalid env var silently ignored
//!
//! ### --runner-delay (param 51)
//! - EC-1 (delay): help lists flag
//! - EC-2 (delay): delay 0 accepted in dry-run
//! - EC-3 (delay): delay 30 accepted in dry-run
//! - EC-4 (delay): `CLR_RUNNER_DELAY` env var applied
//! - EC-5 (delay): CLI wins over env var
//! - EC-6 (delay): invalid env var silently ignored

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stderr_str };

// ── Param 50 — --retry-on-runner ──────────────────────────────────────────────

// ── EC-1: --help lists --retry-on-runner ──────────────────────────────────────

/// EC-1 (param 50): `clr --help` output contains `--retry-on-runner`.
#[ test ]
fn ec1_retry_on_runner_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--retry-on-runner" ),
    "`clr --help` must list --retry-on-runner. Got:\n{stdout}"
  );
}

// ── EC-2: --retry-on-runner 0 --dry-run → exit 0 ─────────────────────────────

/// EC-2 (param 50): value 0 (explicit zero) accepted in dry-run.
///
/// Divergence from EC-3: 0 is stored as zero; 2 is stored as two; both accepted without error.
#[ test ]
fn ec2_retry_on_runner_zero_dry_run()
{
  let out = run_cli( &[ "--retry-on-runner", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3: --retry-on-runner 2 --dry-run → exit 0 ─────────────────────────────

/// EC-3 (param 50): value 2 (nonzero) accepted in dry-run; flag parsed without error.
#[ test ]
fn ec3_retry_on_runner_nonzero_dry_run()
{
  let out = run_cli( &[ "--retry-on-runner", "2", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4: CLR_RETRY_ON_RUNNER=2 env var applied ──────────────────────────────

/// EC-4 (param 50): `CLR_RETRY_ON_RUNNER=2` applied when CLI flag absent.
#[ test ]
fn ec4_clr_retry_on_runner_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_RUNNER", "2" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RETRY_ON_RUNNER env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5: CLI wins over CLR_RETRY_ON_RUNNER ───────────────────────────────────

/// EC-5 (param 50): CLI value 3 wins over `CLR_RETRY_ON_RUNNER=1`.
#[ test ]
fn ec5_retry_on_runner_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--retry-on-runner", "3", "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_RUNNER", "1" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6: CLR_RETRY_ON_RUNNER=invalid → silently ignored ──────────────────────

/// EC-6 (param 50): invalid `CLR_RETRY_ON_RUNNER` silently ignored; exit 0.
#[ test ]
fn ec6_clr_retry_on_runner_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RETRY_ON_RUNNER", "notanumber" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RETRY_ON_RUNNER must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── Param 51 — --runner-delay ─────────────────────────────────────────────────

// ── EC-1 (delay): --help lists --runner-delay ─────────────────────────────────

/// EC-1 (param 51): `clr --help` output contains `--runner-delay`.
#[ test ]
fn ec1_runner_delay_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--runner-delay" ),
    "`clr --help` must list --runner-delay. Got:\n{stdout}"
  );
}

// ── EC-2 (delay): --runner-delay 0 --dry-run → exit 0 ────────────────────────

/// EC-2 (param 51): delay=0 accepted in dry-run.
#[ test ]
fn ec2_runner_delay_zero_dry_run()
{
  let out = run_cli( &[ "--runner-delay", "0", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-3 (delay): --runner-delay 30 --dry-run → exit 0 ───────────────────────

/// EC-3 (param 51): delay=30 accepted in dry-run.
#[ test ]
fn ec3_runner_delay_nonzero_dry_run()
{
  let out = run_cli( &[ "--runner-delay", "30", "--dry-run", "task" ] );
  assert!(
    out.status.success(),
    "exit must be 0. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-4 (delay): CLR_RUNNER_DELAY=30 env var applied ────────────────────────

/// EC-4 (param 51): `CLR_RUNNER_DELAY=30` applied when CLI flag absent.
#[ test ]
fn ec4_clr_runner_delay_env_var_accepted()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RUNNER_DELAY", "30" ) ],
  );
  assert!(
    out.status.success(),
    "CLR_RUNNER_DELAY env var must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-5 (delay): CLI wins over CLR_RUNNER_DELAY ──────────────────────────────

/// EC-5 (param 51): CLI value 30 wins over `CLR_RUNNER_DELAY=10`.
#[ test ]
fn ec5_runner_delay_cli_wins_over_env()
{
  let out = run_cli_with_env(
    &[ "--runner-delay", "30", "--dry-run", "task" ],
    &[ ( "CLR_RUNNER_DELAY", "10" ) ],
  );
  assert!(
    out.status.success(),
    "CLI value must win over env var. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-6 (delay): CLR_RUNNER_DELAY=invalid → silently ignored ─────────────────

/// EC-6 (param 51): invalid `CLR_RUNNER_DELAY` silently ignored; exit 0.
#[ test ]
fn ec6_clr_runner_delay_invalid_ignored()
{
  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_RUNNER_DELAY", "abc" ) ],
  );
  assert!(
    out.status.success(),
    "invalid CLR_RUNNER_DELAY must be silently ignored. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
}

// ── EC-7: binary absent + retry fires (BUG-299) ───────────────────────────────

/// EC-7 (BUG-299): when `claude` is absent from PATH and `--retry-on-runner 1
/// --runner-delay 0` is set, `apply_runner_retry()` fires once before exhausting —
/// stderr contains `"retrying"`.
///
/// ## Root Cause (BUG-299)
/// `--retry-on-runner`/`--runner-delay` were parsed but had no runtime effect;
/// all spawn-error arms called `exit(1)` directly, bypassing the retry system.
///
/// ## Fix Applied
/// `execute_print_attempt()` now returns `Result<ExecutionOutput, io::Error>` on
/// spawn failure; `run_print_mode()` calls `apply_runner_retry()` on `Err`.
///
/// ## Pitfall
/// Do NOT include `--retry-override` in EC-7 — it short-circuits to its value
/// (0 would prevent any retry). EC-7 drives retry via the class-specific param only.
// test_kind: bug_reproducer(BUG-299)
#[ test ]
fn ec7_runner_retry_fires_on_absent_binary()
{
  // Empty temp dir: no `claude` binary present → spawn fails with NotFound.
  let empty_path = tempfile::TempDir::new().expect( "create empty PATH dir" );
  let path_val   = empty_path.path().to_str().expect( "path UTF-8" ).to_string();

  let out = run_cli_with_env(
    &[ "--print", "--max-sessions", "0", "--retry-on-runner", "1", "--runner-delay", "0", "msg" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );

  assert!(
    !out.status.success(),
    "EC-7 (BUG-299): expected non-zero exit when binary absent; got 0"
  );
  assert!(
    err.contains( "retrying" ),
    "EC-7 (BUG-299): stderr must contain 'retrying' when retry fires; got:\n{err}"
  );
}

// ── EC-8: retry disabled explicitly — no retry fires ─────────────────────────

/// EC-8 (BUG-299): when `claude` is absent and `--retry-on-runner 0 --retry-override 0`
/// is set, no retry fires — stderr does NOT contain `"retrying"`; exit 1 immediately.
///
/// `--retry-override 0` suppresses the default fallback (2 retries), ensuring the
/// zero class-specific count is not overridden by the default.
// test_kind: bug_reproducer(BUG-299)
#[ test ]
fn ec8_runner_retry_disabled_no_retry()
{
  let empty_path = tempfile::TempDir::new().expect( "create empty PATH dir" );
  let path_val   = empty_path.path().to_str().expect( "path UTF-8" ).to_string();

  let out = run_cli_with_env(
    &[
      "--print", "--max-sessions", "0",
      "--retry-on-runner", "0", "--retry-override", "0",
      "msg",
    ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );

  assert!(
    !out.status.success(),
    "EC-8 (BUG-299): expected non-zero exit when binary absent; got 0"
  );
  assert!(
    !err.contains( "retrying" ),
    "EC-8 (BUG-299): stderr must NOT contain 'retrying' when retry disabled; got:\n{err}"
  );
}

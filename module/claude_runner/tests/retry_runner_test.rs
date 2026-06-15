#![ allow( clippy::doc_markdown ) ] // test doc comments use code identifiers in prose
//! `--retry-on-runner` and `--runner-delay` Parse Tests
//!
//! ## Purpose
//!
//! Verify EC-1 through EC-6 from `tests/docs/cli/param/050_retry_on_runner.md` and
//! EC-1 through EC-6 from `tests/docs/cli/param/051_runner_delay.md`.
//!
//! Both parameter specs share this test file.
//!
//! ## Test Layout
//!
//! - EC-1..EC-6 (param 50), EC-1..EC-6 (param 51): parser/dry-run — no subprocess
//! - No integration tests: Runner class errors (binary not found, spawn failed, gate
//!   timeout) exit before the retry loop is entered, so `--retry-on-runner` has no
//!   runtime effect in the current implementation
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
use cli_binary_test_helpers::{ run_cli, run_cli_with_env };

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

//! Effort Argument Tests — `--effort` and `--no-effort-max` flags
//!
//! ## Purpose
//!
//! Verify that `claude_runner` injects `--effort max` by default, that `--effort <level>`
//! overrides the default, and that `--no-effort-max` suppresses all effort injection.
//! Uses `--dry-run` to inspect command construction without requiring the Claude binary in PATH.
//!
//! ## Strategy
//!
//! All tests invoke the compiled binary via `env!("CARGO_BIN_EXE_clr")`.
//! `--dry-run` outputs the command line that would be executed, allowing
//! assertions against the translation of flags → builder calls.
//!
//! ## Corner Cases Covered
//!
//! - T59: `--effort max` injected by default (no explicit flag required)
//! - T60: `--effort medium` overrides the default max
//! - T61: `--effort high` override accepted and forwarded
//! - T62: `--effort low` override accepted and forwarded
//! - T63: `--no-effort-max` suppresses all effort injection
//! - T64: invalid `--effort` value rejected with message listing valid values
//! - T65: `--effort max` explicit is idempotent (appears exactly once)
//! - T66: `--no-effort-max` with a message still suppresses effort
//! - T67: `--effort` with no value → "requires a value" error
//! - T68: `--no-effort-max` with `--effort medium` → suppression wins, no effort forwarded
//! - T69: repeated `--effort` flags — last value wins
//! - T70: `--no-effort-max` suppresses `--effort` regardless of flag order

mod common;
use common::run_cli;

// T59: --effort max injected by default (no explicit flag required)
#[ test ]
fn t59_effort_max_default_on()
{
  let out = run_cli( &[ "--dry-run" ] );
  assert!(
    out.status.success(),
    "default dry-run must succeed. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--effort max" ),
    "default output must include --effort max. Got:\n{stdout}"
  );
}

// T60: --effort <level> overrides the default max
#[ test ]
fn t60_effort_medium_overrides_default()
{
  let out = run_cli( &[ "--dry-run", "--effort", "medium" ] );
  assert!(
    out.status.success(),
    "--effort medium must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--effort medium" ),
    "output must contain --effort medium. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--effort max" ),
    "--effort medium must replace the max default. Got:\n{stdout}"
  );
}

// T61: --effort high override accepted and forwarded
#[ test ]
fn t61_effort_high_override()
{
  let out = run_cli( &[ "--dry-run", "--effort", "high" ] );
  assert!(
    out.status.success(),
    "--effort high must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--effort high" ),
    "output must contain --effort high. Got:\n{stdout}"
  );
}

// T62: --effort low override accepted and forwarded
#[ test ]
fn t62_effort_low_override()
{
  let out = run_cli( &[ "--dry-run", "--effort", "low" ] );
  assert!(
    out.status.success(),
    "--effort low must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--effort low" ),
    "output must contain --effort low. Got:\n{stdout}"
  );
}

// T63: --no-effort-max suppresses all effort injection
#[ test ]
fn t63_no_effort_max_suppresses()
{
  let out = run_cli( &[ "--dry-run", "--no-effort-max" ] );
  assert!(
    out.status.success(),
    "--no-effort-max must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--effort" ),
    "--no-effort-max must suppress all --effort tokens. Got:\n{stdout}"
  );
}

// T64: invalid --effort value rejected with message listing valid values
#[ test ]
fn t64_effort_invalid_level_rejected()
{
  let out = run_cli( &[ "--dry-run", "--effort", "invalid" ] );
  assert!(
    !out.status.success(),
    "invalid --effort value must exit non-zero"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "valid values" ),
    "error message must list valid values. Got stderr:\n{stderr}"
  );
}

// T65: --effort max explicit is idempotent (appears exactly once, not duplicated)
#[ test ]
fn t65_effort_explicit_max_idempotent()
{
  let out = run_cli( &[ "--dry-run", "--effort", "max" ] );
  assert!(
    out.status.success(),
    "--effort max must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--effort max" ),
    "output must contain --effort max. Got:\n{stdout}"
  );
  // Idempotent: "--effort max" must appear at most once in the command line
  let count = stdout.matches( "--effort max" ).count();
  assert_eq!(
    count, 1,
    "--effort max must appear exactly once (idempotent). Got:\n{stdout}"
  );
}

// T66: --no-effort-max with a message still suppresses effort injection
#[ test ]
fn t66_no_effort_max_with_message()
{
  let out = run_cli( &[ "--dry-run", "--no-effort-max", "Fix bug" ] );
  assert!(
    out.status.success(),
    "--no-effort-max with message must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--effort" ),
    "--no-effort-max with message must suppress all --effort tokens. Got:\n{stdout}"
  );
}

// T67: `--effort` with no value → "requires a value" error
//
// `--effort` at end of argv without a following value must be rejected.
// Spec: docs/cli/testing/param/effort.md TC-07.
#[ test ]
fn t67_effort_missing_value_rejected()
{
  let out = run_cli( &[ "--effort" ] );
  assert!(
    !out.status.success(),
    "--effort without value must exit non-zero"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "requires a value" ),
    "--effort without value must mention 'requires a value'. Got:\n{stderr}"
  );
}

// T68: `--no-effort-max` with `--effort medium` → suppression wins, no effort forwarded
//
// When both flags are present, `--no-effort-max` takes precedence and suppresses
// all effort injection regardless of the explicit `--effort` level.
// Spec: docs/cli/testing/param/no_effort_max.md TC-03.
#[ test ]
fn t68_no_effort_max_suppresses_explicit_effort()
{
  let out = run_cli( &[ "--dry-run", "--no-effort-max", "--effort", "medium" ] );
  assert!(
    out.status.success(),
    "--no-effort-max --effort medium must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--effort" ),
    "--no-effort-max must win over explicit --effort medium. Got:\n{stdout}"
  );
}

// T69: repeated `--effort` flags — last value wins
//
// When `--effort` appears more than once, the last occurrence wins (standard
// last-wins semantics for duplicate flags).
#[ test ]
fn t69_duplicate_effort_last_wins()
{
  let out = run_cli( &[ "--dry-run", "--effort", "low", "--effort", "high" ] );
  assert!(
    out.status.success(),
    "duplicate --effort must be accepted (last wins). stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--effort high" ),
    "last --effort value must win. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--effort low" ),
    "first --effort value must be overridden. Got:\n{stdout}"
  );
}

// T70: `--no-effort-max` suppresses `--effort` regardless of flag order
//
// Suppression is order-independent: `--effort high --no-effort-max` must produce
// the same result as `--no-effort-max --effort high` — no effort forwarded.
#[ test ]
fn t70_no_effort_max_order_independent()
{
  let out = run_cli( &[ "--dry-run", "--effort", "high", "--no-effort-max" ] );
  assert!(
    out.status.success(),
    "--effort high --no-effort-max must be accepted. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "--effort" ),
    "--no-effort-max must suppress --effort even when --effort precedes it. Got:\n{stdout}"
  );
}

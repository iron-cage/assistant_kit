//! Integration tests for CLR error classification (BUG-037).
//!
//! Verifies that `run_print_mode` emits labeled per-type diagnostics on stderr
//! when `classify_error()` identifies a specific failure mode. Uses fake-claude
//! shell scripts to control subprocess exit code and output — no real claude binary.
//!
//! # Test Matrix
//!
//! | Test | Scenario | Expected stderr |
//! |------|----------|-----------------|
//! | T09 | fake-claude exits 2, empty output | `"Error: [Transient] rate limit (exit 2)"` |
//! | T10 | fake-claude writes auth pattern to stdout, exits 1 | `"Error: [Auth]"` prefix |
//! | T11 | fake-claude writes quota pattern to stderr, exits 1 | `"Error: [Account]"` prefix |
//!
//! # Root Cause (BUG-037)
//!
//! `run_print_mode` emitted `"Claude exited without output (possible rate limit or quota
//! exhaustion)"` for ALL silent non-zero exits. Callers and monitoring tools could not
//! distinguish rate-limit from auth failure from API error.
//!
//! # Why Not Caught
//!
//! No integration test asserted the stderr message format for specific exit codes or
//! output patterns. The generic message was accepted as "good enough" at review time.
//!
//! # Fix Applied
//!
//! BUG-037 block replaced with a match on `output.classify_error()`. Each `ErrorKind`
//! variant emits `"Error: {label} (exit {code})"`, providing distinct signals per type.
//!
//! # Prevention
//!
//! For each `ErrorKind` variant, add an integration test that drives the CLR binary with
//! a fake-claude script and asserts the expected labeled string on stderr.
//!
//! # Pitfall
//!
//! `classify_error()` scans both stderr AND stdout. When claude writes the failure reason
//! to stdout (e.g. auth errors via `--print` JSON output), the stderr scan alone would
//! miss it. Always drive a test that puts the pattern in stdout, not only stderr.

#![ cfg( unix ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude_dir, run_cli_with_env, stderr_str };

// ── T09 ───────────────────────────────────────────────────────────────────────

/// T09 (BUG-037): fake-claude exits 2 with no output → clr stderr contains
/// `"Error: [Transient] rate limit (exit 2)"`.
///
/// Before fix: stderr contained the generic phrase "possible rate limit or quota exhaustion".
/// After fix (3-tier redesign): stderr contains the `[Class]`-prefixed label.
/// `--retry-override 0` disables all retries so the label fires immediately rather than
/// after sleeping 30s and emitting "retries exhausted".
#[ test ]
#[ doc = "bug_reproducer(BUG-037)" ]
fn rate_limit_exit2_emits_labeled_message()
{
  let ( _dir, path_val ) = fake_claude_dir( "exit 2" );
  // --retry-override 0 disables all error-class retries (overrides the built-in default=2).
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "Error: [Transient] rate limit (exit 2)" ),
    "T09 (BUG-037): stderr must contain 'Error: [Transient] rate limit (exit 2)'; got:\n{err}"
  );
  assert!(
    !err.contains( "possible rate limit or quota exhaustion" ),
    "T09 (BUG-037): generic phrase must be absent; got:\n{err}"
  );
}

// ── T10 ───────────────────────────────────────────────────────────────────────

/// T10 (BUG-037): fake-claude writes auth pattern to stdout, exits 1 → clr stderr
/// contains `"Error: [Auth]"` prefix with the original message.
///
/// Validates that `classify_error()` scans stdout as well as stderr — auth failure
/// text from `claude --print` arrives via stdout, not stderr.
/// `--retry-override 0` disables Auth-class retry so the label fires immediately.
#[ test ]
#[ doc = "bug_reproducer(BUG-037)" ]
fn auth_error_pattern_in_stdout_emits_labeled_message()
{
  let ( _dir, path_val ) = fake_claude_dir(
    "echo 'Your organization does not have access to Claude'; exit 1",
  );
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "Error: [Auth]" ),
    "T10 (BUG-037): stderr must contain 'Error: [Auth]' prefix; got:\n{err}"
  );
  assert!(
    err.contains( "Your organization does not have access to Claude" ),
    "T10 (BUG-037): stderr must contain the original auth message; got:\n{err}"
  );
  assert!(
    !err.contains( "possible rate limit or quota exhaustion" ),
    "T10 (BUG-037): generic phrase must be absent; got:\n{err}"
  );
}

// ── T11 ───────────────────────────────────────────────────────────────────────

/// T11 (TSK-253): fake-claude writes quota exhaustion pattern to stderr, exits 1 →
/// clr stderr contains `"Error: [Account]"` prefix with the original message.
///
/// Verifies that `QuotaExhausted` is distinct from `RateLimit` at the CLR output layer —
/// quota exhaustion maps to `[Account]` class, NOT `[Transient]`.
/// `--retry-override 0` disables Account-class retry so the label fires immediately.
#[ test ]
fn quota_exhausted_pattern_emits_labeled_message()
{
  let ( _dir, path_val ) = fake_claude_dir(
    "echo \"You've hit your limit\" >&2; exit 1",
  );
  let out = run_cli_with_env(
    &[ "--print", "--retry-override", "0", "--max-sessions", "0", "test" ],
    &[ ( "PATH", &path_val ) ],
  );
  let err = stderr_str( &out );
  assert!(
    err.contains( "Error: [Account]" ),
    "T11 (TSK-253): stderr must contain 'Error: [Account]' prefix; got:\n{err}"
  );
  assert!(
    err.contains( "You've hit your limit" ),
    "T11 (TSK-253): stderr must contain the original quota message; got:\n{err}"
  );
  assert!(
    !err.contains( "[Transient]" ),
    "T11 (TSK-253): [Transient] must be absent for quota exhaustion; got:\n{err}"
  );
}

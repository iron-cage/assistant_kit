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
//! | T09 | fake-claude exits 2, empty output | `"Error: rate limit (exit 2)"` |
//! | T10 | fake-claude writes auth pattern to stdout, exits 1 | `"Error: auth error"` |
//! | T11 | fake-claude writes quota pattern to stderr, exits 1 | `"Error: quota exhausted (exit 1)"` |
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
/// `"Error: rate limit (exit 2)"`.
///
/// Before fix: stderr contained the generic phrase "possible rate limit or quota exhaustion".
/// After fix: stderr contains the labeled "Error: rate limit (exit 2)".
#[ test ]
#[ doc = "bug_reproducer(BUG-037)" ]
fn rate_limit_exit2_emits_labeled_message()
{
  let ( _dir, path_val ) = fake_claude_dir( "exit 2" );
  let out = run_cli_with_env( &[ "--print", "test" ], &[ ( "PATH", &path_val ) ] );
  let err = stderr_str( &out );
  assert!(
    err.contains( "Error: rate limit (exit 2)" ),
    "T09 (BUG-037): stderr must contain 'Error: rate limit (exit 2)'; got:\n{err}"
  );
  assert!(
    !err.contains( "possible rate limit or quota exhaustion" ),
    "T09 (BUG-037): generic phrase must be absent; got:\n{err}"
  );
}

// ── T10 ───────────────────────────────────────────────────────────────────────

/// T10 (BUG-037): fake-claude writes auth pattern to stdout, exits 1 → clr stderr
/// contains `"Error: auth error"`.
///
/// Validates that `classify_error()` scans stdout as well as stderr — auth failure
/// text from `claude --print` arrives via stdout, not stderr.
#[ test ]
#[ doc = "bug_reproducer(BUG-037)" ]
fn auth_error_pattern_in_stdout_emits_labeled_message()
{
  let ( _dir, path_val ) = fake_claude_dir(
    "echo 'Your organization does not have access to Claude'; exit 1",
  );
  let out = run_cli_with_env( &[ "--print", "test" ], &[ ( "PATH", &path_val ) ] );
  let err = stderr_str( &out );
  assert!(
    err.contains( "Error: auth error" ),
    "T10 (BUG-037): stderr must contain 'Error: auth error'; got:\n{err}"
  );
  assert!(
    !err.contains( "possible rate limit or quota exhaustion" ),
    "T10 (BUG-037): generic phrase must be absent; got:\n{err}"
  );
}

// ── T11 ───────────────────────────────────────────────────────────────────────

/// T11 (TSK-253): fake-claude writes quota exhaustion pattern to stderr, exits 1 →
/// clr stderr contains `"Error: quota exhausted (exit 1)"`.
///
/// Verifies that `QuotaExhausted` is distinct from `RateLimit` at the CLR
/// output layer — quota exhaustion emits "quota exhausted" not "rate limit".
#[ test ]
fn quota_exhausted_pattern_emits_labeled_message()
{
  let ( _dir, path_val ) = fake_claude_dir(
    "echo \"You've hit your limit\" >&2; exit 1",
  );
  let out = run_cli_with_env( &[ "--print", "test" ], &[ ( "PATH", &path_val ) ] );
  let err = stderr_str( &out );
  assert!(
    err.contains( "Error: quota exhausted (exit 1)" ),
    "T11 (TSK-253): stderr must contain 'Error: quota exhausted (exit 1)'; got:\n{err}"
  );
  assert!(
    !err.contains( "rate limit" ),
    "T11 (TSK-253): 'rate limit' must be absent for quota exhaustion; got:\n{err}"
  );
}

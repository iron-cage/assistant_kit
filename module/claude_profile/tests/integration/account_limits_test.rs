//! Integration tests: lim (Account Limits).
//!
//! Tests the `.account.limits` command — FR-18.
//!
//! ## Test Coverage
//!
//! | TC | IT | Description |
//! |----|----|-------------|
//! | lim01 | IT-6 | `name::ghost` unknown account exits 2 |
//! | lim02 | IT-7 | No active credentials exits 2 with actionable error |
//! | lim03 | IT-8 | Active credentials present but data unavailable exits 2 (not silent 0) |
//! | lim04 | IT-9 | `name::` with invalid chars exits 1 (usage error) |
//!
//! ## Happy Path Status (IT-1 through IT-5)
//!
//! IT-1 through IT-5 require a live POST /v1/messages API call to retrieve
//! `anthropic-ratelimit-unified-*` response headers. This crate is architecturally
//! prohibited from making network calls (see `responsibility_no_process_execution_test.rs`).
//! HTTP client infrastructure must be added before these tests can be written.
//! Until then, they are tracked in the manual test plan at `tests/manual/readme.md`.
//!
//! ## Root Cause Context
//!
//! Rate-limit utilization is returned as response headers from the Anthropic API:
//! - `anthropic-ratelimit-unified-5h-utilization` (decimal 0.0–1.0, session window)
//! - `anthropic-ratelimit-unified-7d-utilization` (decimal 0.0–1.0, weekly)
//! - `anthropic-ratelimit-unified-status` (`allowed` / `allowed_warning` / `rejected`)
//!
//! These headers are never cached locally by Claude Code. An HTTP call is required.
//!
//! ## Prevention
//!
//! Any future HTTP client added to `claude_profile` MUST be gated behind the `enabled`
//! feature and MUST NOT use [`std::process::Command`] (banned by the responsibility test).

use crate::helpers::{
  run_cs_with_env,
  stderr, assert_exit,
  write_credentials, write_account,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── lim01 ─────────────────────────────────────────────────────────────────────

/// lim01 (IT-6): `name::ghost` — syntactically valid but non-existent account.
///
/// Root Cause: `account.limits` must distinguish "not found" (exit 2) from
///   "invalid name chars" (exit 1); both use the `name::` parameter.
/// Why Not Caught: No test existed before lim01.
/// Fix Applied: Existence check on `{name}.credentials.json` before data fetch.
/// Prevention: Always add not-found tests for all `name::` parameters.
/// Pitfall: Do not return exit 1 for "not found" — that code is reserved for
///   usage errors (invalid characters), not runtime "record not found" errors.
#[ test ]
fn lim01_unknown_named_account_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Active credentials exist; `ghost` account does NOT.
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );
  std::fs::create_dir_all( dir.path().join( ".claude" ).join( "accounts" ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.limits", "name::ghost" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    err.contains( "ghost" ) || err.contains( "not found" ),
    "error must mention 'ghost' or 'not found', got:\n{err}",
  );
}

// ── lim02 ─────────────────────────────────────────────────────────────────────

/// lim02 (IT-7): No active credentials configured.
///
/// Root Cause: `account.limits` must fail with an actionable error when no
///   `.credentials.json` exists, not silently with exit 0 or a panic.
/// Why Not Caught: No test existed before lim02.
/// Fix Applied: `require_active_credentials()` checks file existence before fetch.
/// Prevention: Every command that reads credentials must guard against absent file.
/// Pitfall: `require_claude_paths()` only checks HOME — it does NOT check for
///   `.credentials.json` existence. A separate guard is always required.
#[ test ]
fn lim02_no_active_credentials_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Create .claude dir but NO .credentials.json
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.limits" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "error message must be non-empty when no credentials exist, got:\n{err}",
  );
  assert!(
    err.to_lowercase().contains( "account" )
      || err.to_lowercase().contains( "auth" )
      || err.to_lowercase().contains( "credential" ),
    "error must mention account/auth/credential, got:\n{err}",
  );
}

// ── lim03 ─────────────────────────────────────────────────────────────────────

/// lim03 (IT-8): Active credentials present but rate-limit data unavailable.
///
/// Root Cause: `account.limits` must NEVER exit 0 silently when data cannot
///   be obtained. AC-04 from `feature/013_account_limits.md` requires exit 2
///   with an actionable message.
/// Why Not Caught: No test existed before lim03.
/// Fix Applied: `fetch_rate_limits()` always returns Err until HTTP is added;
///   the command exits 2 with an actionable message pointing to `claude /usage`.
/// Prevention: Add AC-04-style "must not be silent" assertions to any command
///   that has an optional-data data source.
/// Pitfall: An empty stdout with exit 0 is a silent success — always assert
///   that unavailable data produces a non-zero exit, never just empty output.
#[ test ]
fn lim03_data_unavailable_exits_2_not_silent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.limits" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.is_empty(),
    "exit 2 must emit an actionable error message, not empty stderr, got:\n{err}",
  );
}

// ── lim04 ─────────────────────────────────────────────────────────────────────

/// lim04 (IT-9): `name::` with path-separator characters exits 1 (usage error).
///
/// Root Cause: Path-separator characters in account names would allow directory
///   traversal; they must be caught at the parameter layer with exit 1, not
///   treated as a "not found" runtime error (exit 2).
/// Why Not Caught: No test existed before lim04.
/// Fix Applied: `account::validate_name()` rejects forbidden chars; mapped to
///   `ArgumentTypeMismatch` error code (exit 1) via `io_err_to_error_data`.
/// Prevention: Any `name::` parameter must call `validate_name()` before any
///   filesystem operation, and the resulting error must map to exit 1.
/// Pitfall: Do NOT use exit 2 for "invalid name" — exit 2 is reserved for
///   runtime errors (not found, unavailable). Invalid input is always exit 1.
#[ test ]
fn lim04_invalid_name_chars_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.limits", "name::foo/bar" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "invalid" ) || err.to_lowercase().contains( "character" ),
    "exit 1 error must mention invalid/character, got:\n{err}",
  );
}

// ── lim05 ─────────────────────────────────────────────────────────────────────

/// lim05 (IT-6 variant): `name::work` when named account exists exits 2 with
/// data-unavailable (not not-found).
///
/// Verifies that the not-found path and the data-unavailable path produce
/// different exit codes/messages for the same `name::` parameter.
#[ test ]
fn lim05_existing_named_account_exits_2_with_data_unavailable()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Write a named account "work" (not active) and active credentials
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );
  write_account( dir.path(), "work", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.limits", "name::work" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  // Must NOT say "not found" — the account EXISTS; the issue is data unavailability.
  assert!(
    !err.to_lowercase().contains( "not found" ),
    "existing account must not produce 'not found' error, got:\n{err}",
  );
  assert!(
    !err.is_empty(),
    "must emit an actionable data-unavailable error, got empty stderr",
  );
}

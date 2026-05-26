//! Unit tests for `account::refresh_account_token` — failure-path contracts.
//!
//! ## Purpose
//!
//! Verify `account::refresh_account_token` returns `None` on every early-exit
//! path without spawning the `claude` binary. Tests cover:
//! - `Some(paths)` branch: no credential file in persistent store
//! - `Some(paths)` branch: credential in store but `.claude/` directory absent (switch fails)
//! - `None` branch: no credential file in persistent store
//!
//! ## Test Matrix
//!
//! | Test | Branch | Scenario | Expected |
//! |------|--------|----------|----------|
//! | `art_some_paths_no_store_cred_returns_none`             | `Some(paths)` | `{name}.credentials.json` absent in store         | `None`             |
//! | `art_some_paths_dot_claude_absent_returns_none`          | `Some(paths)` | Cred in store; `.claude/` dir absent              | `None`             |
//! | `art_none_paths_no_store_cred_returns_none`             | `None`        | `{name}.credentials.json` absent in store         | `None`             |
//! | `art_some_paths_no_store_cred_trace_does_not_panic`     | `Some(paths)` | `trace=true`; cred absent in store                | no panic, `None`   |
//! | `art_some_paths_dot_claude_absent_trace_does_not_panic` | `Some(paths)` | `trace=true`; cred in store; `.claude/` absent    | no panic, `None`   |
//! | `art_none_paths_no_store_cred_trace_does_not_panic`     | `None`        | `trace=true`; cred absent in store                | no panic, `None`   |
//! | `art_some_paths_run_isolated_invoked_trace_no_panic`    | `Some(paths)` | `trace=true`; cred in store; `.claude/` exists    | no panic, `None`   |
//! | `bug_mre_bug205_refresh_token_read_write_ok_trace_structural` | structural | grep account.rs for `"read credentials: OK"` and `"write credentials: OK"` | ≥2 each |
//!
//! ## Pitfall: Consumer Feature Activation
//!
//! All tests are gated `#[cfg(feature = "enabled")]` to mirror the function's own gate.
//! Consumer crates whose workspace dep on `claude_profile_core` carries
//! `default-features = false` must explicitly add `features = ["enabled"]` in their
//! own `Cargo.toml` dep entry — without it the function compiles away silently and
//! call sites produce `error[E0425]: cannot find function refresh_account_token`.
//! (TSK-167 Phase 3 reiteration root cause.)
//!
//! ## Pitfall: `#[must_use]` Hidden by Docker Image Cache
//!
//! `refresh_account_token` carries `#[must_use]`. The test
//! `art_some_paths_run_isolated_invoked_trace_no_panic` validates the "does not panic"
//! contract, not the return value — hence the explicit `let _ =` discard. Without
//! `let _ =`, `-D warnings` produces `error: unused return value of 'refresh_account_token'
//! that must be used`, but this error is invisible while the Docker image cache is valid.
//! It only surfaces when the image is rebuilt after any `account.rs` source change forces
//! recompilation. Always use `let _ =` when intentionally discarding a `#[must_use]`
//! return value — never rely on cache masking to suppress the warning. (BUG-168.)

use tempfile::TempDir;
use claude_profile_core::account;
use claude_core::ClaudePaths;

// ── helpers ───────────────────────────────────────────────────────────────────

fn write_cred_file( store : &std::path::Path, name : &str )
{
  std::fs::write(
    store.join( format!( "{name}.credentials.json" ) ),
    r#"{"accessToken":"tok","expiresAt":9999999999999}"#,
  ).unwrap();
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[ cfg( feature = "enabled" ) ]
#[ test ]
fn art_some_paths_no_store_cred_returns_none()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  // No {name}.credentials.json in store — switch_account returns NotFound.
  let paths  = ClaudePaths::with_home( fake_home.path() );
  let result = account::refresh_account_token( "ghost@example.com", store.path(), Some( &paths ), false, "test", claude_runner_core::IsolatedModel::Default, &[] );
  assert_eq!( result, None, "must return None when store credential file absent" );
}

#[ cfg( feature = "enabled" ) ]
#[ test ]
fn art_some_paths_dot_claude_absent_returns_none()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  // Cred file in store, but {fake_home}/.claude/ absent — switch_account fails Io (copy to nonexistent parent).
  // Pitfall: do NOT create fake_home/.claude/ — its absence causes fs::copy to fail.
  write_cred_file( store.path(), "ghost@example.com" );
  let paths  = ClaudePaths::with_home( fake_home.path() );
  let result = account::refresh_account_token( "ghost@example.com", store.path(), Some( &paths ), false, "test", claude_runner_core::IsolatedModel::Default, &[] );
  assert_eq!( result, None, "must return None when .claude/ directory absent (switch_account fails Io)" );
}

#[ cfg( feature = "enabled" ) ]
#[ test ]
fn art_none_paths_no_store_cred_returns_none()
{
  let store  = TempDir::new().unwrap();
  // No {name}.credentials.json in store — read_to_string fails, None branch early-exit.
  let result = account::refresh_account_token( "ghost@example.com", store.path(), None, false, "test", claude_runner_core::IsolatedModel::Default, &[] );
  assert_eq!( result, None, "must return None when store credential file absent (None branch)" );
}

// ── trace=true variants ────────────────────────────────────────────────────────

#[ cfg( feature = "enabled" ) ]
#[ test ]
fn art_some_paths_no_store_cred_trace_does_not_panic()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  // No {name}.credentials.json — switch_account returns NotFound; trace logs the Err.
  let paths  = ClaudePaths::with_home( fake_home.path() );
  let result = account::refresh_account_token( "ghost@example.com", store.path(), Some( &paths ), true, "test", claude_runner_core::IsolatedModel::Default, &[] );
  assert_eq!( result, None, "trace=true must still return None when store credential file absent" );
}

#[ cfg( feature = "enabled" ) ]
#[ test ]
fn art_some_paths_dot_claude_absent_trace_does_not_panic()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  // Cred file in store but {fake_home}/.claude/ absent — switch_account fails Io; trace logs the Err.
  write_cred_file( store.path(), "ghost@example.com" );
  let paths  = ClaudePaths::with_home( fake_home.path() );
  let result = account::refresh_account_token( "ghost@example.com", store.path(), Some( &paths ), true, "test", claude_runner_core::IsolatedModel::Default, &[] );
  assert_eq!( result, None, "trace=true must still return None when .claude/ dir absent" );
}

#[ cfg( feature = "enabled" ) ]
#[ test ]
fn art_none_paths_no_store_cred_trace_does_not_panic()
{
  let store  = TempDir::new().unwrap();
  // No {name}.credentials.json — read_to_string fails; trace logs the Err.
  let result = account::refresh_account_token( "ghost@example.com", store.path(), None, true, "test", claude_runner_core::IsolatedModel::Default, &[] );
  assert_eq!( result, None, "trace=true must still return None when store credential file absent (None branch)" );
}

#[ cfg( feature = "enabled" ) ]
#[ test ]
fn art_some_paths_run_isolated_invoked_trace_no_panic()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  // Cred file in store AND .claude/ dir exists — switch_account succeeds;
  // run_isolated is invoked but fails fast (claude binary absent or fake token) →
  // trace logs Err or "OK credentials=None" → returns None; must not panic.
  write_cred_file( store.path(), "ghost@example.com" );
  std::fs::create_dir_all( fake_home.path().join( ".claude" ) ).unwrap();
  let paths = ClaudePaths::with_home( fake_home.path() );
  // FT-15 / BUG-166: trace must reach run_isolated invocation without panic.
  // BUG-168: `let _ =` required — discards `#[must_use]` return value intentionally.
  // BUG-169: args corrected to `["--print", "."]` — vec\![] regression fixed.
  // This test validates "does not panic", not the return value.
  let _ = account::refresh_account_token( "ghost@example.com", store.path(), Some( &paths ), true, "test", claude_runner_core::IsolatedModel::Default, &[] );
}

// ── structural (BUG-205) ──────────────────────────────────────────────────────

#[ test ]
// Root Cause: Ok(s) => s bare arms in refresh_account_token() emitted no trace on
//   success — only Err arms had instrumentation, creating a silent gap in trace::1 output.
// Why Not Caught: AC-26 implemented incrementally; Ok arms left uninstrumented;
//   no assertion checked both arms per step.
// Fix Applied: if trace { eprintln!(...) } after Ok(s) arms and after fs::write success
//   blocks in both Some(paths) and else branches (4 insertions total).
// Prevention: Lifecycle trace functions must instrument both Ok and Err for every step.
// Pitfall: Multi-branch functions duplicate lifecycle steps — both branches must be updated.
fn bug_mre_bug205_refresh_token_read_write_ok_trace_structural()
{
  let account_rs = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/account.rs" );
  let content    = std::fs::read_to_string( &account_rs )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", account_rs.display() ) );
  let read_ok_count  = content.matches( "read credentials: OK" ).count();
  let write_ok_count = content.matches( "write credentials: OK" ).count();
  assert!(
    read_ok_count >= 2,
    "BUG-205: expected ≥2 occurrences of 'read credentials: OK' in account.rs, found {read_ok_count}"
  );
  assert!(
    write_ok_count >= 2,
    "BUG-205: expected ≥2 occurrences of 'write credentials: OK' in account.rs, found {write_ok_count}"
  );
}



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
//! | `art_some_paths_no_store_cred_returns_none`    | `Some(paths)` | `{name}.credentials.json` absent in store | `None` |
//! | `art_some_paths_dot_claude_absent_returns_none` | `Some(paths)` | Cred in store; `.claude/` dir absent      | `None` |
//! | `art_none_paths_no_store_cred_returns_none`    | `None`        | `{name}.credentials.json` absent in store | `None` |
//!
//! ## Pitfall: Consumer Feature Activation
//!
//! All tests are gated `#[cfg(feature = "enabled")]` to mirror the function's own gate.
//! Consumer crates whose workspace dep on `claude_profile_core` carries
//! `default-features = false` must explicitly add `features = ["enabled"]` in their
//! own `Cargo.toml` dep entry — without it the function compiles away silently and
//! call sites produce `error[E0425]: cannot find function refresh_account_token`.
//! (TSK-167 Phase 3 reiteration root cause.)

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
  let result = account::refresh_account_token( "ghost@example.com", store.path(), Some( &paths ) );
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
  let result = account::refresh_account_token( "ghost@example.com", store.path(), Some( &paths ) );
  assert_eq!( result, None, "must return None when .claude/ directory absent (switch_account fails Io)" );
}

#[ cfg( feature = "enabled" ) ]
#[ test ]
fn art_none_paths_no_store_cred_returns_none()
{
  let store  = TempDir::new().unwrap();
  // No {name}.credentials.json in store — read_to_string fails, None branch early-exit.
  let result = account::refresh_account_token( "ghost@example.com", store.path(), None );
  assert_eq!( result, None, "must return None when store credential file absent (None branch)" );
}

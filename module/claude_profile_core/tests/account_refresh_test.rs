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
//! | `bug_mre_bug175_no_switch_account_in_some_branch` | structural | grep account.rs for `"switch_account( name, credential_store, p )"` | 0 occurrences |
//! | `bug_mre_bug221_some_branch_no_p_credentials_file_write` | structural | grep account.rs for `std::fs::write( p.credentials_file(),` | 0 occurrences |
//! | `mre_bug221_save_some_creds_writes_to_store_not_live_file` | unit | `save("acct", store, paths, false, Some(b"data"))` | store = `b"data"`; live file unchanged |
//! | `mre_bug221_save_none_creds_copies_from_live_file` | unit | `save("acct", store, paths, false, None)` | store = live file content; live file unchanged |
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
  // Cred file in store; {fake_home}/.claude/ absent — read from store succeeds;
  // run_isolated fails (no real claude binary or fake token → error) → None.
  // Fix(BUG-175): .claude/ absence no longer causes early-exit directly; switch_account removed.
  write_cred_file( store.path(), "ghost@example.com" );
  let paths  = ClaudePaths::with_home( fake_home.path() );
  let result = account::refresh_account_token( "ghost@example.com", store.path(), Some( &paths ), false, "test", claude_runner_core::IsolatedModel::Default, &[] );
  assert_eq!( result, None, "must return None when run_isolated fails (no real claude binary)" );
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
  // Cred file in store; {fake_home}/.claude/ absent — read from store succeeds;
  // run_isolated fails (no real claude binary → error); trace logs the Err → None.
  // Fix(BUG-175): .claude/ absence no longer causes early-exit directly; switch_account removed.
  write_cred_file( store.path(), "ghost@example.com" );
  let paths  = ClaudePaths::with_home( fake_home.path() );
  let result = account::refresh_account_token( "ghost@example.com", store.path(), Some( &paths ), true, "test", claude_runner_core::IsolatedModel::Default, &[] );
  assert_eq!( result, None, "trace=true must still return None when run_isolated fails" );
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

// ── structural (BUG-175) ─────────────────────────────────────────────────────

#[ test ]
// Root Cause: Some(paths) branch called switch_account(name, credential_store, p) solely to
//   populate ~/.claude/.credentials.json so the immediately-following read_to_string(p.credentials_file())
//   could read it; run_isolated creates its own temp HOME and never reads from real ~/.claude/.
// Why Not Caught: BUG-165's fix extracted the lifecycle as switch→refresh→save; the switch step
//   was motivated by needing to read p.credentials_file(), not by intent to write ~/.claude/;
//   no test asserted the absence of global writes in a multi-account batch scenario.
// Fix Applied: removed t_switch + match switch_account(...) block; changed read to use
//   credential_store.join(format!("{name}.credentials.json")) — same pattern as None branch.
// Prevention: Both branches of a multi-branch function should use the same access path;
//   when the None branch proves direct store reads work, the Some branch must match.
// Pitfall: switch_account before a read looks like defensive initialization; the unnecessary
//   global write is only observable in concurrent multi-account batch scenarios.
fn bug_mre_bug175_no_switch_account_in_some_branch()
{
  let account_rs = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/account.rs" );
  let content    = std::fs::read_to_string( &account_rs )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", account_rs.display() ) );
  let count = content.matches( "switch_account( name, credential_store, p )" ).count();
  assert!(
    count == 0,
    "BUG-175: expected 0 occurrences of 'switch_account( name, credential_store, p )' in account.rs, found {count}"
  );
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

// ── structural (BUG-221) ──────────────────────────────────────────────────────

#[ test ]
// Root Cause: refresh_account_token() Some(paths) branch called std::fs::write(p.credentials_file(), &new_creds)
//   before calling save(), clobbering the live session credentials file (~/.claude/.credentials.json)
//   on every batch refresh call. BUG-175's fix (TSK-208) removed switch_account() but left this write intact.
// Why Not Caught: Some(paths) branch tests covered only error paths (no store cred, no .claude/ dir);
//   no test verified the live file was untouched after a successful refresh cycle in the Some branch.
// Fix Applied: changed write target from p.credentials_file() to credential_store path; added
//   creds: Option<&[u8]> to save() so save(Some(&new_creds)) writes from bytes without reading live file.
// Prevention: structural test asserts 0 occurrences of the old write-to-live-file pattern; any regression
//   reintroducing the write to p.credentials_file() in the Some(paths) branch fails immediately.
// Pitfall: grep for the full function-call pattern to avoid matching doc comments or other write() calls
//   that are not the live-file clobber.
fn bug_mre_bug221_some_branch_no_p_credentials_file_write()
{
  let account_rs = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/account.rs" );
  let content    = std::fs::read_to_string( &account_rs )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", account_rs.display() ) );
  let count = content.matches( "std::fs::write( p.credentials_file()," ).count();
  assert!(
    count == 0,
    "BUG-221: expected 0 occurrences of 'std::fs::write( p.credentials_file(),' in account.rs, found {count}"
  );
}

#[ test ]
// Root Cause: save() always copied from paths.credentials_file() (the live session file); refresh_account_token()
//   Some(paths) branch had to write to the live file first so save() could copy refreshed credentials.
//   This orphaned write was the core of BUG-221.
// Why Not Caught: save() callers (.account.save, .account.relogin) legitimately copy from the live file;
//   no test exercised a code path where save() needed to write from bytes without touching the live file.
// Fix Applied: save() gained creds: Option<&[u8]>; Some(bytes) writes directly to the store file;
//   None copies from the live file as before (existing .account.save / .account.relogin behaviour preserved).
// Prevention: unit test calls save(Some(bytes)) directly and asserts the store file = bytes and live file unchanged.
// Pitfall: save() with Some(bytes) still runs oauthAccount merge and _active marker logic; only the
//   credential file write is bypassed — the rest of save() runs identically for both Some and None.
fn mre_bug221_save_some_creds_writes_to_store_not_live_file()
{
  let store      = TempDir::new().unwrap();
  let fake_home  = TempDir::new().unwrap();
  let dot_claude = fake_home.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  let live_file  = dot_claude.join( ".credentials.json" );
  std::fs::write( &live_file, b"original_live_creds" ).unwrap();
  let paths      = ClaudePaths::with_home( fake_home.path() );

  account::save( "acct@test.com", store.path(), &paths, false, Some( b"new_creds_bytes" ), None, None, None ).unwrap();

  let store_file = store.path().join( "acct@test.com.credentials.json" );
  assert!( store_file.exists(), "save(Some(bytes)) must create the credential store file" );
  assert_eq!(
    std::fs::read( &store_file ).unwrap(),
    b"new_creds_bytes",
    "save(Some(bytes)) must write bytes to the credential store file",
  );
  assert_eq!(
    std::fs::read( &live_file ).unwrap(),
    b"original_live_creds",
    "save(Some(bytes)) must NOT modify the live credentials file",
  );
}

#[ test ]
// Root Cause: (see mre_bug221_save_some_creds_writes_to_store_not_live_file)
// Why Not Caught: (same root cause — no tests exercised the None path in isolation)
// Fix Applied: (same — save() creds param; None path copies from live file, unchanged from before)
// Prevention: unit test verifies save(None) still copies from the live file (callers .account.save
//   and .account.relogin depend on this behaviour — breaking it would silently break account saving).
// Pitfall: save(None) is the pre-existing behaviour; this test guards against accidentally breaking it
//   while fixing the Some(bytes) path.
fn mre_bug221_save_none_creds_copies_from_live_file()
{
  let store      = TempDir::new().unwrap();
  let fake_home  = TempDir::new().unwrap();
  let dot_claude = fake_home.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  let live_file  = dot_claude.join( ".credentials.json" );
  std::fs::write( &live_file, b"live_creds_content" ).unwrap();
  let paths      = ClaudePaths::with_home( fake_home.path() );

  account::save( "acct@test.com", store.path(), &paths, false, None, None, None, None ).unwrap();

  let store_file = store.path().join( "acct@test.com.credentials.json" );
  assert!( store_file.exists(), "save(None) must create the credential store file" );
  assert_eq!(
    std::fs::read( &store_file ).unwrap(),
    b"live_creds_content",
    "save(None) must copy from the live credentials file",
  );
  assert_eq!(
    std::fs::read( &live_file ).unwrap(),
    b"live_creds_content",
    "save(None) must NOT modify the live credentials file",
  );
}

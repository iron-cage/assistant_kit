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
//! | `bug_mre_bug205_refresh_token_read_write_ok_trace_structural` | structural | grep refresh.rs for `"read credentials: OK"` and `"write credentials: OK"` | ≥2 each |
//! | `bug_mre_bug175_no_switch_account_in_some_branch` | structural | grep refresh.rs for `"switch_account( name, credential_store, p )"` | 0 occurrences |
//! | `bug_mre_bug221_some_branch_no_p_credentials_file_write` | structural | grep refresh.rs for `atomic_write_secret( &p.credentials_file(),` | exactly 1 occurrence (BUG-318 active-account live sync) |
//! | `mre_bug318_rotation_live_sync_structural` | structural | grep refresh.rs for `is_still_active` and `Fix(BUG-318)` | present |
//! | `mre_bug221_save_some_creds_writes_to_store_not_live_file` | unit | `save("acct", store, paths, false, Some(b"data"))` | store = `b"data"`; live file unchanged |
//! | `mre_bug221_save_none_creds_copies_from_live_file` | unit | `save("acct", store, paths, false, None)` | store = live file content; live file unchanged |
//! | `bug_reproducer_343_save_does_not_merge_live_identity_into_non_active_target` | bug_reproducer(BUG-343) | live session identifies as `live@test.com`; `save()` called for a different, non-active `target@test.com` | target's own `{name}.json` has no `oauthAccount` merged from the live session |
//! | `ft22_manipulate_expires_at_replaces_numeric_value` | behavioral | `manipulate_expires_at` with numeric `expiresAt` value | value replaced (original absent from result) |
//! | `ft22_manipulate_expires_at_replaces_quoted_value` | behavioral | `manipulate_expires_at` with quoted `expiresAt` value | value replaced (original absent from result) |
//! | `ft22_manipulate_expires_at_noop_when_key_absent` | behavioral | `manipulate_expires_at` when `expiresAt` key absent | string returned unchanged |
//! | `ft22_manipulate_expires_at_called_before_run_isolated_structural` | structural | grep `refresh.rs` for `manipulate_expires_at(` before first `run_isolated(` | in order |
//! | `ft23_live_sync_returns_live_creds_without_subprocess` | behavioral | live creds differ from stored → sync and return `Some(live)` without subprocess | `Some(live_json)` |
//! | `ft24_some_paths_branch_reads_credentials_file_twice_structural` | structural | grep `refresh.rs` `Some(paths)` branch for ≥2 `credentials_file()` calls | ≥2 occurrences |
//! | `ft25_071_refresh_redirect_bypass_checked_before_run_isolated_structural` | structural | Feature 071/AC-09: redirect-backend check appears before `run_isolated(` and before the `Some(paths)`/`None` branch dispatch | in order |
//! | `ft26_071_refresh_redirect_account_returns_none_credentials_unchanged` | behavioral | Feature 071/AC-09: `refresh_account_token()` against a `backend: redirect` account | `None`; credential store file byte-for-byte unchanged |
//! | `mre_bug483_refresh_write_back_must_guard_blank_credentials` | bug_reproducer(BUG-483) | grep refresh.rs: `fn credentials_usable(` exists; ≥2 non-comment call sites; `Fix(BUG-483)` at ≥2 sites | guard present in both write-back branches |
//! | `bug483_credentials_usable_accepts_non_blank_pair` | behavioral | `credentials_usable` with both tokens non-empty (flat + nested `claudeAiOauth` shapes) | `true` |
//! | `bug483_credentials_usable_rejects_blank_or_missing_tokens` | behavioral | `credentials_usable` with the logged-out sandbox shape, each token blank/missing individually, and degenerate inputs | `false` for all |
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
//! It only surfaces when the image is rebuilt after any `src/account/` source change forces
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
  let account_rs = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/account/refresh.rs" );
  let content    = std::fs::read_to_string( &account_rs )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", account_rs.display() ) );
  let count = content.matches( "switch_account( name, credential_store, p )" ).count();
  assert!(
    count == 0,
    "BUG-175: expected 0 occurrences of 'switch_account( name, credential_store, p )' in refresh.rs, found {count}"
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
  let account_rs = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/account/refresh.rs" );
  let content    = std::fs::read_to_string( &account_rs )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", account_rs.display() ) );
  let read_ok_count  = content.matches( "read credentials: OK" ).count();
  let write_ok_count = content.matches( "write credentials: OK" ).count();
  assert!(
    read_ok_count >= 2,
    "BUG-205: expected ≥2 occurrences of 'read credentials: OK' in refresh.rs, found {read_ok_count}"
  );
  assert!(
    write_ok_count >= 2,
    "BUG-205: expected ≥2 occurrences of 'write credentials: OK' in refresh.rs, found {write_ok_count}"
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
// Prevention: structural test asserts exactly 1 occurrence of the pattern — the BUG-318 conditional live sync
//   (write to p.credentials_file() only when is_still_active, after rotation). 0 occurrences means the live
//   sync was removed; >1 means an unconditional clobber was reintroduced. Both are regressions.
// Pitfall: grep for the full function-call pattern to avoid matching doc comments or other write() calls
//   that are not the live-file clobber.
// Update(BUG-318): changed assertion from count==0 to count==1 — Fix(BUG-318) adds one conditional
//   write to p.credentials_file() in the success path (post-rotation live sync for active account).
//   The old invariant (0 occurrences) was correct for batch refresh but prevented the needed live sync.
// Update(audit-credential-file-perms): the live sync now routes through atomic_write_secret
//   (0o600 + unique tmp + rename), so the guarded pattern is the atomic_write_secret call.
//   Same count==1 invariant, same regression semantics in both directions.
fn bug_mre_bug221_some_branch_no_p_credentials_file_write()
{
  let account_rs = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/account/refresh.rs" );
  let content    = std::fs::read_to_string( &account_rs )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", account_rs.display() ) );
  let count = content.matches( "atomic_write_secret( &p.credentials_file()," ).count();
  assert!(
    count == 1,
    "BUG-221/BUG-318: expected exactly 1 occurrence of 'atomic_write_secret( &p.credentials_file(),' \
     in refresh.rs (the BUG-318 is_still_active live sync); 0 = live sync removed, >1 = unconditional \
     clobber reintroduced. Found {count}"
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

  account::save( "acct@test.com", store.path(), &paths, false, Some( b"new_creds_bytes" ), None, None, None, account::AccountBackend::Anthropic, None, None, None, None ).unwrap();

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

  account::save( "acct@test.com", store.path(), &paths, false, None, None, None, None, account::AccountBackend::Anthropic, None, None, None, None ).unwrap();

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

#[ test ]
// test_kind: bug_reproducer(BUG-343)
// Root Cause: save()'s oauthAccount merge (store.rs; account.rs:367-377 at fix time) reads the machine-local live
//   session file (paths.claude_json_file()) unconditionally and merges its `oauthAccount` into
//   {name}.json with zero comparison against `name` — corrupting a non-active target account's
//   own file with whichever identity happens to be locally active on this machine. Reachable
//   via the unguarded save() call inside refresh_token_with_live_path() (account.rs:1209 at fix time),
//   which is routinely invoked for non-active accounts by apply_touch/apply_refresh's default
//   (non-only_active) full-account-list loops (refresh.rs; account.rs:1116-1118 at fix time).
// Why Not Caught: no existing test constructs a live session identity DIFFERENT from the
//   `name` being saved — mre_bug221_save_* above use a single identity throughout and never
//   populate paths.claude_json_file() with oauthAccount data at all.
// Fix Applied: save()'s merge block now gates on the live session's own oauthAccount.emailAddress
//   equaling `name` before merging (see Fix(BUG-343) comment in refresh.rs).
// Prevention: this test asserts the target's own oauthAccount is unaffected by a divergent live
//   session identity; must keep passing for any future save() merge-block change.
// Pitfall: a function that reads "the machine's live session" to enrich a named account's file
//   is only safe when the caller can guarantee `name` IS the live session's own account — once
//   shared with a caller that saves non-active accounts (background refresh, by design), every
//   unguarded live-session read becomes a cross-account identity leak.
fn bug_reproducer_343_save_does_not_merge_live_identity_into_non_active_target()
{
  let store      = TempDir::new().unwrap();
  let fake_home  = TempDir::new().unwrap();
  let dot_claude = fake_home.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  // This machine's live session identifies as "live@test.com" — NOT the account saved below.
  std::fs::write(
    fake_home.path().join( ".claude.json" ),
    r#"{"oauthAccount":{"emailAddress":"live@test.com","organizationUuid":"org-live"}}"#,
  ).unwrap();
  let paths = ClaudePaths::with_home( fake_home.path() );

  // save() invoked for a DIFFERENT, non-active account — the routine background-refresh
  // scenario per refresh_token_with_live_path (account.rs:1116-1118 at fix time; saves accounts that are
  // "NOT yet the active account").
  account::save( "target@test.com", store.path(), &paths, false, Some( b"new_creds_bytes" ), None, None, None, account::AccountBackend::Anthropic, None, None, None, None ).unwrap();

  let meta_path = store.path().join( "target@test.com.json" );
  let meta_text = std::fs::read_to_string( &meta_path ).unwrap();
  let meta_val  : serde_json::Value = serde_json::from_str( &meta_text ).unwrap();

  assert!(
    meta_val.get( "oauthAccount" ).is_none(),
    "save() must NOT merge the live session's oauthAccount into a non-active target's own file; found: {:?}",
    meta_val.get( "oauthAccount" ),
  );
}

// ── FT-22: manipulate_expires_at ──────────────────────────────────────────────

// FT-22a: numeric expiresAt value is replaced with 1
//
// When credentials JSON contains `"expiresAt":BIGNUM`, the returned string must
// replace the big numeric value so the CLI treats the access token as expired and
// uses the refresh token — forcing RT rotation on every subprocess call (AC-32).
#[ test ]
fn ft22_manipulate_expires_at_replaces_numeric_value()
{
  let input  = r#"{"accessToken":"tok","expiresAt":9999999999999}"#;
  let result = account::manipulate_expires_at( input );
  assert!(
    !result.contains( "9999999999999" ),
    "ft22: numeric expiresAt must be replaced; original value must not appear in result, got: {result}",
  );
  assert!(
    result.contains( "expiresAt" ),
    "ft22: expiresAt key must still be present in result, got: {result}",
  );
  assert!(
    result.contains( "accessToken" ),
    "ft22: accessToken must be preserved unchanged, got: {result}",
  );
}

// FT-22b: quoted expiresAt value is replaced
//
// Some Claude CLI versions store expiresAt as a quoted string rather than a bare
// number. The function must handle both formats.
#[ test ]
fn ft22_manipulate_expires_at_replaces_quoted_value()
{
  let input  = r#"{"accessToken":"tok","expiresAt":"9999999999999"}"#;
  let result = account::manipulate_expires_at( input );
  assert!(
    !result.contains( "9999999999999" ),
    "ft22: quoted expiresAt must be replaced; original value must not appear in result, got: {result}",
  );
  assert!(
    result.contains( "expiresAt" ),
    "ft22: expiresAt key must still be present in result, got: {result}",
  );
}

// FT-22c: string returned unchanged when expiresAt key is absent
//
// Not all credential JSON objects include expiresAt. The function must return the
// original string unchanged when the key is absent — not panic or corrupt the JSON.
#[ test ]
fn ft22_manipulate_expires_at_noop_when_key_absent()
{
  let input  = r#"{"accessToken":"tok","refreshToken":"rt"}"#;
  let result = account::manipulate_expires_at( input );
  assert_eq!(
    result, input,
    "ft22: when expiresAt key is absent, manipulate_expires_at must return the input unchanged",
  );
}

// FT-23: live credentials different from stored → sync without subprocess, return Some(live)
//
// AC-33 (Change B): When `Some(paths)` is supplied and the live credentials file
// `~/.claude/.credentials.json` differs from the stored per-account file, `refresh_account_token`
// must sync live→store and return `Some(live_creds)` WITHOUT spawning `run_isolated`.
//
// Observable: since `run_isolated` fails with `Err` in the test environment (no real claude
// binary), any return of `Some(...)` proves the pre-sync path fired — the subprocess was NOT called.
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn ft23_live_sync_returns_live_creds_without_subprocess()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  let dot_claude = fake_home.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();

  let stored_json = r#"{"accessToken":"tok_A","expiresAt":9999999999999,"refreshToken":"rt_A"}"#;
  let live_json   = r#"{"accessToken":"tok_B","expiresAt":9999999999999,"refreshToken":"rt_B"}"#;

  // Write stored credentials (token A)
  std::fs::write(
    store.path().join( "ghost@example.com.credentials.json" ),
    stored_json,
  ).unwrap();
  // Write DIFFERENT live credentials (token B) at ~/.claude/.credentials.json
  let live_creds_path = dot_claude.join( ".credentials.json" );
  std::fs::write( &live_creds_path, live_json ).unwrap();
  // Write active marker so the is_active guard in Change B passes for "ghost@example.com".
  // The pre-sync only fires when name == active account; without the marker it is skipped.
  std::fs::write(
    store.path().join( account::active_marker_filename() ),
    "ghost@example.com",
  ).unwrap();

  let paths  = ClaudePaths::with_home( fake_home.path() );
  let result = account::refresh_account_token(
    "ghost@example.com", store.path(), Some( &paths ), false, "test",
    claude_runner_core::IsolatedModel::Default, &[],
  );

  // AC-33: live diff detected → returned Some(live_json) without subprocess
  assert!(
    result.is_some(),
    "ft23: live creds differ from stored — expected Some(live_creds) via pre-sync, got None \
     (None means run_isolated was called and failed, meaning pre-sync did not fire)",
  );
  let returned = result.unwrap();
  assert!(
    returned.contains( "tok_B" ),
    "ft23: returned credentials must be live creds (tok_B), got: {returned}",
  );
  // Verify stored file was updated to the live version
  let stored_after = std::fs::read_to_string(
    store.path().join( "ghost@example.com.credentials.json" ),
  ).expect( "stored cred file must exist after sync" );
  assert!(
    stored_after.contains( "tok_B" ),
    "ft23: stored credentials must be updated to live version (tok_B), got: {stored_after}",
  );
}

// FT-22d structural: manipulate_expires_at is called before run_isolated in refresh_account_token
//
// Change A requires that both branches of refresh_account_token call manipulate_expires_at
// BEFORE passing credentials to run_isolated. A structural source scan enforces this ordering.
#[ test ]
fn ft22_manipulate_expires_at_called_before_run_isolated_structural()
{
  let account_rs = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/account/refresh.rs" );
  let content    = std::fs::read_to_string( &account_rs )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", account_rs.display() ) );

  // Locate refresh_account_token body (starts at `pub fn refresh_account_token(`)
  let fn_start = content.find( "pub fn refresh_account_token(" )
    .expect( "refresh_account_token must exist in refresh.rs" );
  let fn_body = &content[ fn_start.. ];

  let manip_pos = fn_body.find( "manipulate_expires_at(" )
    .expect( "AC-32: manipulate_expires_at must be called inside refresh_account_token" );
  let run_pos = fn_body.find( "run_isolated(" )
    .expect( "run_isolated must still be called inside refresh_account_token" );

  assert!(
    manip_pos < run_pos,
    "AC-32: manipulate_expires_at must appear before run_isolated in refresh_account_token body \
     (manip_pos={manip_pos}, run_pos={run_pos})",
  );
}

// FT-24 structural: refresh_token_with_live_path reads credentials_file() at least twice
//
// AC-33 (Change B) adds two reads of the live credentials file via `p.credentials_file()`:
//   1. Pre-sync: before run_isolated — detect if live already refreshed and sync early
//   2. Race recovery: after run_isolated returns Ok(isolated) with credentials=None — sync if changed
// The Some(paths) branch of `refresh_account_token` delegates to `refresh_token_with_live_path`;
// the structural count verifies both reads are present in the helper's body.
// Fix(BUG-313 refactor): FT-24 updated to search `refresh_token_with_live_path` (the extracted helper)
// instead of the Some(paths) branch of `refresh_account_token`, which now contains only a delegation call.
// Root cause: helper extraction moved `credentials_file()` calls into the private helper, but the
//   structural test still searched the public function's Some(paths) block (now a one-liner).
// Pitfall: structural tests must track the function that ACTUALLY contains the logic, not just the
//   public entry point — delegation patterns move the code without necessarily breaking the invariant.
#[ test ]
fn ft24_some_paths_branch_reads_credentials_file_twice_structural()
{
  let account_rs = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/account/refresh.rs" );
  let content    = std::fs::read_to_string( &account_rs )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", account_rs.display() ) );

  // Extract the refresh_token_with_live_path helper body.
  // This private function implements the Some(paths) branch logic after extraction.
  let helper_start = content.find( "fn refresh_token_with_live_path(" )
    .expect( "refresh_token_with_live_path must exist in refresh.rs (private helper for Some(paths) branch)" );
  // The helper ends at its closing brace; find the next top-level function or end of file.
  // Use the leading `\n}` that closes the helper body (followed by a blank line).
  let helper_body_start = content[ helper_start.. ].find( '{' )
    .expect( "helper body opening brace must exist" );
  let helper_region = &content[ helper_start + helper_body_start.. ];

  // Count only code-level calls — exclude comment lines (// prefix after trimming).
  // Comments in the helper mention credentials_file() by name but are not calls.
  let count = helper_region.lines()
    .filter( | line | !line.trim_start().starts_with( "//" ) )
    .filter( | line | line.contains( "credentials_file()" ) )
    .count();
  assert!(
    count >= 2,
    "AC-33: expected ≥2 code-level credentials_file() calls in refresh_token_with_live_path \
     (pre-sync read + race recovery read), found {count}",
  );
}

// FT-22e: negative expiresAt value is NOT replaced — treated as absent per doc contract
//
// AC-32 doc: "Negative values (e.g. `"expiresAt":-1`) are not matched — treated as absent."
// The implementation finds `-` as the first non-digit character, producing old_val="" (empty),
// which is filtered by `!old_val.is_empty()` → falls through to return unchanged.
#[ test ]
fn ft22_manipulate_expires_at_noop_for_negative_value()
{
  let input  = r#"{"accessToken":"tok","expiresAt":-1,"refreshToken":"rt"}"#;
  let result = account::manipulate_expires_at( input );
  assert_eq!(
    result, input,
    "ft22e: negative expiresAt must be treated as absent (not replaced); string must be unchanged",
  );
}

// FT-22f: expiresAt already set to numeric 1 — idempotent, no double-replacement
//
// When expiresAt is already 1, replacen replaces "expiresAt":1 with "expiresAt":1 (no change).
// Verifies the function is safe to call repeatedly without corrupting the JSON.
#[ test ]
fn ft22_manipulate_expires_at_idempotent_already_numeric_one()
{
  let input  = r#"{"accessToken":"tok","expiresAt":1,"refreshToken":"rt"}"#;
  let result = account::manipulate_expires_at( input );
  assert!(
    result.contains( "\"expiresAt\":1" ),
    "ft22f: expiresAt:1 must still be present in result, got: {result}",
  );
  assert!(
    result.contains( "accessToken" ),
    "ft22f: accessToken must be preserved after idempotent call, got: {result}",
  );
  // The overall string must not be corrupted — both known fields still present.
  assert!(
    result.contains( "refreshToken" ),
    "ft22f: refreshToken must be preserved after idempotent call, got: {result}",
  );
}

// FT-23b: non-active account with live ≠ stored — pre-sync does NOT fire
//
// AC-33 pre-sync guard: only fires when name IS the currently active account.
// When the active marker names a DIFFERENT account, `is_active = false` — the function
// must NOT overwrite the store with the live credentials. The live file belongs to a
// different account; writing it to name's store would corrupt the credential store.
//
// Safety property: the function returns None (run_isolated fails in test env) and the
// stored file content is UNCHANGED (still tok_A, not tok_B from live).
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn ft23_non_active_account_skips_live_presync()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  let dot_claude = fake_home.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();

  let stored_json = r#"{"accessToken":"tok_A","expiresAt":9999999999999,"refreshToken":"rt_A"}"#;
  let live_json   = r#"{"accessToken":"tok_B","expiresAt":9999999999999,"refreshToken":"rt_B"}"#;

  // Write stored credentials for alice (tok_A).
  std::fs::write(
    store.path().join( "alice@example.com.credentials.json" ),
    stored_json,
  ).unwrap();
  // Write DIFFERENT live credentials (tok_B) — belongs to another account's live session.
  std::fs::write( dot_claude.join( ".credentials.json" ), live_json ).unwrap();
  // Active marker names "other@example.com", NOT "alice@example.com".
  // This means `is_active = false` for alice — the pre-sync guard must block the sync.
  std::fs::write(
    store.path().join( account::active_marker_filename() ),
    "other@example.com",
  ).unwrap();

  let paths  = ClaudePaths::with_home( fake_home.path() );
  let result = account::refresh_account_token(
    "alice@example.com", store.path(), Some( &paths ), false, "test",
    claude_runner_core::IsolatedModel::Default, &[],
  );

  // Pre-sync must NOT have fired (would return Some(tok_B) if it did).
  // run_isolated fails in test env → returns None.
  assert!(
    result.is_none(),
    "ft23b: non-active account must not trigger pre-sync; expected None (run_isolated path), \
     got Some(...) — this means pre-sync fired for a non-active account, which corrupts the store",
  );
  // Store must still contain tok_A — NOT overwritten with live tok_B.
  let stored_after = std::fs::read_to_string(
    store.path().join( "alice@example.com.credentials.json" ),
  ).expect( "stored cred file must still exist after call" );
  assert!(
    stored_after.contains( "tok_A" ),
    "ft23b: stored credentials must remain tok_A after non-active call; \
     got: {stored_after} — tok_B written means pre-sync leaked for non-active account",
  );
  assert!(
    !stored_after.contains( "tok_B" ),
    "ft23b: stored credentials must NOT contain tok_B (live creds from another account); \
     got: {stored_after}",
  );
}

// FT-23c: active account — live == stored → no early return, falls through to run_isolated
//
// AC-33 pre-sync: early return fires ONLY when live creds DIFFER from stored (`!=` check).
// When live == stored, the pre-sync guard is satisfied but its body is skipped — execution
// falls through to run_isolated to rotate the RT. This verifies the `!=` comparison direction.
//
// Observable: function returns None (run_isolated fails in test env), not Some.
// If the comparison were accidentally flipped to `==`, it would return Some(live) here,
// failing the test. This prevents a sign-error regression in the comparison.
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn ft23_active_account_same_creds_falls_through_to_run_isolated()
{
  let store     = TempDir::new().unwrap();
  let fake_home = TempDir::new().unwrap();
  let dot_claude = fake_home.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();

  // Live and stored are IDENTICAL.
  let same_json = r#"{"accessToken":"tok_A","expiresAt":9999999999999,"refreshToken":"rt_A"}"#;

  std::fs::write(
    store.path().join( "alice@example.com.credentials.json" ),
    same_json,
  ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), same_json ).unwrap();
  // Active marker names alice — so is_active = true, pre-sync guard runs.
  std::fs::write(
    store.path().join( account::active_marker_filename() ),
    "alice@example.com",
  ).unwrap();

  let paths  = ClaudePaths::with_home( fake_home.path() );
  let result = account::refresh_account_token(
    "alice@example.com", store.path(), Some( &paths ), false, "test",
    claude_runner_core::IsolatedModel::Default, &[],
  );

  // live == stored: pre-sync early return must NOT fire.
  // Expected: None from run_isolated failure (not Some from pre-sync).
  // If Some is returned, the `!=` check is inverted and the function returns early
  // when creds are equal — skipping run_isolated and silently not rotating the RT.
  assert!(
    result.is_none(),
    "ft23c: when live == stored, pre-sync must not return early; expected None from run_isolated \
     path, got Some(...) — the != comparison may be inverted, causing skipped RT rotation",
  );
}

// ── MRE BUG-316 ───────────────────────────────────────────────────────────────

/// MRE BUG-316: `refresh_token_with_live_path` re-reads the active marker at each use site.
///
/// # Root Cause
///
/// `is_active` was computed ONCE before `run_isolated` and reused 35 seconds later in the
/// race-recovery block. A concurrent `switch_account("B")` during the subprocess window
/// changed the marker to "B". The stale cached `is_active=true` caused B's live credentials
/// (now in `~/.claude/.credentials.json`) to be written into A's credential store slot —
/// silently corrupting the credential of the account that was active at function entry.
///
/// # Why Not Caught
///
/// No test existed for the TOCTOU scenario (filesystem boolean cached across a blocking
/// subprocess call). The design assumption (upstream ensures valid AT) was never tested.
///
/// # Fix Applied
///
/// Replaced the single cached `let is_active = {...}` with two independent inline re-reads:
/// one at the pre-sync site (anonymous `if { ... }` block) and one at the race-recovery
/// site (`is_active_now`), both in `refresh_token_with_live_path`.
///
/// # Prevention
///
/// This structural test verifies that `is_active_now` exists (the named re-read at the
/// race-recovery site) and that `Fix(BUG-316)` is annotated at both re-read sites.
///
/// # Pitfall
///
/// Never cache a filesystem-derived boolean across a blocking call (subprocess, network I/O)
/// in a multi-process environment — re-read at each use site instead.
// BUG-485: this test's `Fix(BUG-316)` count assertion (below) never traces which code
// block a match occurs in, so it passed for 2 months while the pre-sync site carried the
// annotation without the re-read. The pre-sync site's own coverage now lives in
// `mre_bug485_presync_marker_reread_gates_store_write`, which bounds its search to the
// pre-sync block's span instead of matching anywhere in the file.
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn mre_bug316_stale_is_active_race_recovery_copies_wrong_account_creds()
{
  // test_kind: bug_reproducer(BUG-316)
  let src = std::fs::read_to_string(
    concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/account/refresh.rs" )
  ).expect( "read refresh.rs" );

  // Fix: race-recovery site must use a fresh re-read variable, not a cached bool.
  assert!(
    src.contains( "is_active_now" ),
    "BUG-316 fix: `is_active_now` must exist — fresh re-read at race-recovery block"
  );

  // Fix: both re-read sites must carry the Fix(BUG-316) annotation.
  let fix_count = src.matches( "Fix(BUG-316)" ).count();
  assert!(
    fix_count >= 2,
    "BUG-316 fix: `Fix(BUG-316)` must appear at ≥2 sites (pre-sync + race-recovery). \
     Found: {fix_count}"
  );
}

// ── MRE BUG-485 ───────────────────────────────────────────────────────────────

/// MRE BUG-485: the pre-sync block re-reads the active marker immediately before
/// the store write it gates.
///
/// # Root Cause
///
/// `is_active_pre_sync` was captured once at the top of the pre-sync block and
/// trusted across the intervening live-file read. `switch_account("B")` renames the
/// live credentials BEFORE updating the marker, so an interleaving between the
/// capture and the write left the stale bool `true` while the live file already held
/// B's credentials — which the block then wrote into A's store slot. BUG-316's fix
/// commit (d1ff4a4c) claimed this site was fixed, but the diff shows only a variable
/// rename (`is_active` → `is_active_pre_sync`); no re-read was ever added.
///
/// # Why Not Caught
///
/// The BUG-316 regression test matched `Fix(BUG-316)` annotations ANYWHERE in
/// refresh.rs. The pre-sync site's own comment satisfied the count without the code
/// re-reading the marker — the test certified an annotation, not a control flow.
///
/// # Fix Applied
///
/// An `is_active_at_write` re-read of the active marker sits between the live-file
/// read and the store write; the write is additionally atomic (`atomic_write_secret`),
/// so a lost race can no longer land a partial or misattributed credential file.
///
/// # Prevention
///
/// This test bounds its search to the pre-sync block's span (capture → first
/// `manipulate_expires_at` call) and asserts TWO `active_marker_filename()` reads in
/// that span, ordered capture → live-file read → re-read → store write. It fails if
/// the re-read is removed, hoisted above the live-file read, or moved below the write.
///
/// # Pitfall
///
/// A bug report's History claiming a fix landed is not evidence it did — cross-check
/// the actual commit diff. And a regression test that matches fix annotations
/// file-wide certifies comments, not code: bound structural assertions to the block
/// they guard.
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn mre_bug485_presync_marker_reread_gates_store_write()
{
  // test_kind: bug_reproducer(BUG-485)
  let src = std::fs::read_to_string(
    concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/account/refresh.rs" )
  ).expect( "read refresh.rs" );

  // Bound the search to the pre-sync block: from the capture to the AC-32 call that
  // immediately follows the block's close.
  let start = src.find( "let is_active_pre_sync" )
    .expect( "BUG-485: pre-sync capture `is_active_pre_sync` must exist in refresh.rs" );
  let end = start + src[ start.. ].find( "manipulate_expires_at" )
    .expect( "BUG-485: pre-sync block must be followed by the manipulate_expires_at call" );
  let block = &src[ start..end ];

  let marker_reads = block.matches( "active_marker_filename()" ).count();
  assert_eq!(
    marker_reads, 2,
    "BUG-485 fix: the pre-sync span must read the active marker exactly twice \
     (capture + pre-write re-read); found {marker_reads} — 1 means the re-read regressed"
  );
  let reread_pos = block.find( "is_active_at_write" )
    .expect( "BUG-485 fix: `is_active_at_write` re-read must exist in the pre-sync block" );
  let live_read_pos = block.find( "p.credentials_file()" )
    .expect( "pre-sync block must read the live credentials file" );
  let write_pos = block.find( "atomic_write_secret( &store_path" )
    .expect( "pre-sync block must write the store slot via atomic_write_secret" );
  assert!(
    live_read_pos < reread_pos && reread_pos < write_pos,
    "BUG-485 fix: the marker re-read must sit between the live-file read and the store \
     write (read@{live_read_pos} < re-read@{reread_pos} < write@{write_pos})"
  );
}

// ── structural (BUG-318) ──────────────────────────────────────────────────────

/// # Root Cause
///
/// `refresh_token_with_live_path` (called from `apply_post_switch_touch` during `.account.use`)
/// runs `run_isolated` with `expiresAt=1` (AC-32), causing Claude to perform OAuth token
/// rotation. Claude writes `AT_new + RT_new` to LIVE (`~/.claude/.credentials.json`). The
/// function then writes `new_creds` to STORE and calls `save()` — but never updates LIVE.
/// LIVE retains `AT_old` (now revoked by Anthropic). A subsequent `.account.save` reads LIVE
/// (`AT_old`, revoked) and copies it to STORE, overwriting `AT_new` with the revoked token.
/// All future API calls with `AT_old` return 401; the revoked RT cannot recover the account.
///
/// # Why Not Caught
///
/// No test verified that LIVE is updated after a successful rotation in `refresh_token_with_live_path`.
/// BUG-221's structural test asserted `count == 0` for `std::fs::write( p.credentials_file(),` —
/// which inadvertently prevented the needed conditional live sync from being added.
///
/// # Fix Applied
///
/// Added `is_still_active` re-read after `save()` in the success path of
/// `refresh_token_with_live_path`. When the account is still the active session (marker
/// re-read, same pattern as `is_active_now` in Fix(BUG-316)), writes `new_creds` to
/// `p.credentials_file()` (LIVE), keeping LIVE consistent with STORE after rotation.
///
/// # Prevention
///
/// This structural test verifies that `is_still_active` (the post-rotation live-sync variable)
/// and `Fix(BUG-318)` annotation exist in `refresh.rs`. The BUG-221 structural test was
/// updated from `count == 0` to `count == 1` — one conditional live sync is correct; zero
/// means the sync was removed; more than one means an unconditional clobber was reintroduced.
///
/// # Pitfall
///
/// After any `run_isolated` call that rotates credentials, LIVE must be kept in sync with
/// STORE for the currently active account. Removing all writes to `p.credentials_file()`
/// (as BUG-221 did for the batch-refresh case) must be paired with a conditional write for
/// the active-account single-switch case. The invariant: after rotation, LIVE == STORE for
/// the active account.
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn mre_bug318_rotation_live_sync_structural()
{
  // test_kind: bug_reproducer(BUG-318)
  let src = std::fs::read_to_string(
    concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/account/refresh.rs" )
  ).expect( "read refresh.rs" );

  // Fix: post-rotation live sync variable must exist in the success path.
  assert!(
    src.contains( "is_still_active" ),
    "BUG-318 fix: `is_still_active` must exist — post-rotation live sync variable in success path"
  );

  // Fix: the live sync site must carry the Fix(BUG-318) annotation.
  assert!(
    src.contains( "Fix(BUG-318)" ),
    "BUG-318 fix: `Fix(BUG-318)` annotation must appear at the live-sync write site in refresh.rs"
  );
}

// ── structural + behavioral (Feature 071 / AC-09) ──────────────────────────────

/// T04/433/AC-09: `refresh_account_token()` checks `backend == Redirect` and returns
/// early — before `args` is built and before either the `Some(paths)`/`None` branch can
/// dispatch a subprocess — so a redirect account's refresh is a true no-op covering both
/// branches. A black-box behavioral test cannot distinguish "bypassed" from "attempted and
/// failed" in this sandboxed environment (no real `claude` binary — both return `None` with
/// an unchanged store file), so this structural check is the actual AC-09 proof; the
/// behavioral test below is a regression guard on the observable half only.
#[ test ]
fn ft25_071_refresh_redirect_bypass_checked_before_run_isolated_structural()
{
  let account_rs = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src/account/refresh.rs" );
  let content    = std::fs::read_to_string( &account_rs )
    .unwrap_or_else( |e| panic!( "cannot read {}: {e}", account_rs.display() ) );

  let fn_start = content.find( "pub fn refresh_account_token(" )
    .expect( "refresh_account_token must exist in refresh.rs" );
  let fn_body = &content[ fn_start.. ];

  let bypass_pos = fn_body.find( "AccountBackend::Redirect" )
    .expect( "AC-09: refresh_account_token must check AccountBackend::Redirect" );
  let run_pos = fn_body.find( "run_isolated(" )
    .expect( "run_isolated must still be reachable in refresh_account_token" );
  let branch_pos = fn_body.find( "if let Some( p ) = paths" )
    .expect( "refresh_account_token must still dispatch on the Some(paths)/None branches" );

  assert!(
    bypass_pos < run_pos,
    "AC-09: the redirect-backend check must appear before run_isolated( in refresh_account_token \
     (bypass_pos={bypass_pos}, run_pos={run_pos})",
  );
  assert!(
    bypass_pos < branch_pos,
    "AC-09: the redirect-backend check must appear before the Some(paths)/None branch dispatch, \
     so both branches are covered (bypass_pos={bypass_pos}, branch_pos={branch_pos})",
  );
}

/// T04/433/AC-09: `refresh_account_token()` against a `backend: redirect` account returns
/// `None` and leaves the credential store file byte-for-byte unchanged.
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn ft26_071_refresh_redirect_account_returns_none_credentials_unchanged()
{
  let store = TempDir::new().unwrap();
  std::fs::write(
    store.path().join( "kimi@moonshot.ai.credentials.json" ),
    r#"{"accessToken":"sk-foreign-key-abc123"}"#,
  ).unwrap();
  std::fs::write(
    store.path().join( "kimi@moonshot.ai.json" ),
    r#"{"backend":"redirect","base_url":"https://api.moonshot.ai/anthropic","redirect_model":"kimi-k3"}"#,
  ).unwrap();
  let before = std::fs::read_to_string( store.path().join( "kimi@moonshot.ai.credentials.json" ) ).unwrap();

  let result = account::refresh_account_token(
    "kimi@moonshot.ai", store.path(), None, false, "test", claude_runner_core::IsolatedModel::Default, &[],
  );

  assert_eq!( result, None, "AC-09: refresh against a redirect account must return None" );
  let after = std::fs::read_to_string( store.path().join( "kimi@moonshot.ai.credentials.json" ) ).unwrap();
  assert_eq!( before, after, "AC-09: refresh against a redirect account must not modify the credential store file" );
}

// ── MRE BUG-483 ───────────────────────────────────────────────────────────────

/// MRE BUG-483: refresh write-back must guard against blank (logged-out) credentials.
///
/// # Root Cause
///
/// When the store's refresh token is stale (already rotated by another machine —
/// Anthropic RTs are single-use), the AC-32 forced rotation (`expiresAt=1`) makes the
/// sandboxed Claude subprocess hit `invalid_grant`, log itself out, and leave a
/// logged-out credential file (`accessToken:"", refreshToken:"", expiresAt:0`) in its
/// temp HOME. `run_isolated` captures that file as `Some(blank)` and every write-back
/// site persisted it verbatim over the store's non-blank record: the `None`-branch
/// `fs::write`, the `Some(paths)`-branch store write + `save(..., Some(bytes))`, and
/// the `is_still_active` live-file write. On 2026-08-12 one sweep on w003 blanked ten
/// accounts' store records in 19 seconds (commit c19b8003de).
///
/// # Why Not Caught
///
/// No test constructed a `run_isolated` result whose `credentials` payload is a
/// logged-out blank and asserted the store record survives. All refresh-path tests
/// exercise early-exit failure paths (`credentials=None`, missing files) or structural
/// invariants; the `credentials=Some(blank)` success-shape-with-poison-payload case had
/// zero coverage.
///
/// # Fix Applied
///
/// `credentials_usable()` predicate (accessToken AND refreshToken non-empty) gates the
/// write-back in both branches: on a blank payload the refresh returns `None` (caller
/// already maps that to `Err("token refresh failed")`) and the stored record is left
/// untouched for diagnosis and cross-machine self-heal. `Fix(BUG-483)` annotations at
/// both guard sites.
///
/// # Prevention
///
/// This structural test verifies the predicate exists, is invoked at ≥2 non-comment
/// call sites (one per branch), and that both sites carry the `Fix(BUG-483)` marker.
///
/// # Pitfall
///
/// A subprocess "success with credentials captured" is not proof the credentials are
/// usable — a sandboxed logout produces a well-formed credential file whose tokens are
/// empty strings. Validate content before persisting over a known-good record; a failed
/// refresh must fail loudly (`None`) rather than destroy the only recoverable copy.
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn mre_bug483_refresh_write_back_must_guard_blank_credentials()
{
  // test_kind: bug_reproducer(BUG-483)
  let src = std::fs::read_to_string(
    concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/account/refresh.rs" )
  ).expect( "read refresh.rs" );

  // Fix: the blank-credentials predicate must exist.
  assert!(
    src.contains( "fn credentials_usable(" ),
    "BUG-483 fix: `fn credentials_usable(` must exist in refresh.rs — blank-payload guard \
     for subprocess-returned credentials"
  );

  // Fix: the predicate must be invoked at >=2 non-comment call sites
  // (None branch of refresh_account_token + Some(paths) branch in
  // refresh_token_with_live_path), per the FT-24 comment-filtered counting pattern.
  let call_count = src.lines()
    .filter( | line | !line.trim_start().starts_with( "//" ) )
    .filter( | line | line.contains( "credentials_usable(" ) && !line.contains( "fn credentials_usable(" ) )
    .count();
  assert!(
    call_count >= 2,
    "BUG-483 fix: `credentials_usable(` must be invoked at >=2 non-comment sites \
     (one per write-back branch). Found: {call_count}"
  );

  // Fix: both guard sites must carry the Fix(BUG-483) annotation.
  let fix_count = src.matches( "Fix(BUG-483)" ).count();
  assert!(
    fix_count >= 2,
    "BUG-483 fix: `Fix(BUG-483)` must appear at >=2 sites (None branch + Some(paths) branch). \
     Found: {fix_count}"
  );
}

/// BUG-483 companion: `credentials_usable` accepts payloads with both tokens non-empty.
///
/// Covers the flat shape and the real store shape (tokens nested under
/// `claudeAiOauth`) — `parse_string_field` scans the whole blob, so nesting must
/// not affect the verdict.
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn bug483_credentials_usable_accepts_non_blank_pair()
{
  assert!( account::credentials_usable(
    r#"{"accessToken":"at-1","refreshToken":"rt-1"}"#
  ) );
  assert!( account::credentials_usable(
    r#"{"claudeAiOauth":{"accessToken":"sk-ant-oat01-x","refreshToken":"sk-ant-ort01-y","expiresAt":1770000000000,"scopes":["user:inference"],"subscriptionType":"max"}}"#
  ) );
}

/// BUG-483 companion: `credentials_usable` rejects every blank/missing-token shape.
///
/// The logged-out sandbox shape (`accessToken:""`, `refreshToken:""`, `expiresAt:0`)
/// is the exact payload that blanked ten store records on 2026-08-12 — it MUST be
/// rejected, as must each token blank or missing individually, and degenerate inputs.
#[ cfg( feature = "enabled" ) ]
#[ test ]
fn bug483_credentials_usable_rejects_blank_or_missing_tokens()
{
  // The exact logged-out shape a sandboxed invalid_grant logout leaves in the temp HOME.
  assert!( !account::credentials_usable(
    r#"{"claudeAiOauth":{"accessToken":"","refreshToken":"","expiresAt":0,"scopes":[],"subscriptionType":""}}"#
  ) );
  // Each token blank individually.
  assert!( !account::credentials_usable( r#"{"accessToken":"","refreshToken":"rt-1"}"# ) );
  assert!( !account::credentials_usable( r#"{"accessToken":"at-1","refreshToken":""}"# ) );
  // Each token missing individually.
  assert!( !account::credentials_usable( r#"{"refreshToken":"rt-1"}"# ) );
  assert!( !account::credentials_usable( r#"{"accessToken":"at-1"}"# ) );
  // Degenerate inputs.
  assert!( !account::credentials_usable( "{}" ) );
  assert!( !account::credentials_usable( "" ) );
}

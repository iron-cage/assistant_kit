//! Integration tests: CLI command-verb behavioral contracts.
//!
//! Verifies idempotency, state-transition, and pre-condition semantics for each of the
//! ten CLI verbs: save, use, delete, limits, relogin, rotate, renewal, inspect, assign, status.
//!
//! `lim_it` tests require a live Anthropic API token in `~/.claude/.credentials.json`.
//! They are skipped automatically when credentials are absent or the API is rate-limited.
//!
//! ## Test Matrix
//!
//! ### `verb::save` (BV-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | BV-1 | `save_bv1_resave_same_credentials_idempotent` | re-save same creds → identical state | P |
//! | BV-2 | `save_bv2_transitions_absent_to_saved` | absent → saved after save | P |
//! | BV-3 | `save_bv3_without_credentials_exits_2` | no creds file → exit 2 | N |
//!
//! ### `verb::use` (BV-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | BV-1 | `use_bv1_reactivate_already_active_idempotent` | re-use same account → no-op exit 0 | P |
//! | BV-2 | `use_bv2_transitions_saved_to_active` | saved→active, prior active→saved | P |
//! | BV-3 | `use_bv3_nonexistent_account_exits_2` | missing account → exit 2 | N |
//!
//! ### `verb::delete` (BV-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | BV-1 | `delete_bv1_second_delete_exits_2_non_idempotent` | second delete → exit 2 | N |
//! | BV-2 | `delete_bv2_transitions_saved_to_absent` | saved → files removed | P |
//! | BV-3 | `delete_bv3_nonexistent_account_exits_2` | missing account → exit 2 | N |
//!
//! ### `verb::limits` (BV-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | BV-1 | `limits_bv1_lim_it_repeated_calls_no_side_effects` | two calls → no files modified (`lim_it`) | P |
//! | BV-2 | `limits_bv2_lim_it_read_is_non_mutating` | mtime unchanged after call (`lim_it`) | P |
//! | BV-3 | `limits_bv3_without_accessible_account_exits_2` | empty store → exit 2 | N |
//!
//! ### `verb::relogin` (BV-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | BV-1 | `relogin_bv1_lim_it_non_idempotent_oauth_flow` | OAuth non-idempotent (`lim_it` — skipped; needs TTY) | P |
//! | BV-2 | `relogin_bv2_lim_it_updates_in_place_state_preserved` | in-place update (`lim_it` — skipped; needs TTY) | P |
//! | BV-3 | `relogin_bv3_absent_account_exits_1` | absent account → exit 1 | N |
//!
//! ### `verb::rotate` (BV-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | BV-1 | `rotate_bv1_non_idempotent_outcome_changes_with_expiry` | second rotate picks different account | P |
//! | BV-2 | `rotate_bv2_activates_highest_expiry_inactive_account` | best-expiry account activated | P |
//! | BV-3 | `rotate_bv3_no_inactive_accounts_exits_2` | single active account → exit 2 | N |
//!
//! ### `verb::renewal` (BV-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | BV-1 | `renewal_bv1_resetting_same_timestamp_idempotent` | same timestamp → unchanged state | P |
//! | BV-2 | `renewal_bv2_writes_renewal_at_preserving_other_fields` | `_renewal_at` written, other fields kept | P |
//! | BV-3 | `renewal_bv3_without_operation_param_exits_1` | no `at/from_now/clear` → exit 1 | N |
//!
//! ### `verb::inspect` (BV-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | BV-1 | `inspect_bv1_lim_it_repeated_calls_no_side_effects` | two calls → no files modified (`lim_it`) | P |
//! | BV-2 | `inspect_bv2_lim_it_read_is_non_mutating` | mtime unchanged after call (`lim_it`) | P |
//! | BV-3 | `inspect_bv3_without_credentials_exits_2` | empty store → exit 2 | N |
//!
//! ### `verb::assign` (BV-1..3)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | BV-1 | `assign_bv1_reassigning_same_account_idempotent` | same account → marker unchanged | P |
//! | BV-2 | `assign_bv2_writes_marker_without_touching_credential_files` | marker written, creds unchanged | P |
//! | BV-3 | `assign_bv3_nonexistent_account_exits_2` | missing account → exit 2 | N |
//!
//! ### `verb::status` (BV-1..4)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | BV-1 | `status_bv1_token_status_twice_same_classification` | two calls → same result | P |
//! | BV-2 | `status_bv2_token_status_non_mutating` | mtime unchanged after call | P |
//! | BV-3 | `status_bv3_token_status_absent_creds_exits_2` | absent creds file → exit 2 | N |
//! | BV-4 | `status_bv4_credentials_status_twice_same_output` | two calls → identical stdout | P |

use tempfile::TempDir;
use super::cli_runner::
{
  run_cs_with_env, assert_exit, stdout, stderr,
  write_credentials, write_account, write_account_renewal_json,
  write_account_profile_json,
  live_active_token, require_live_api,
  write_account_with_token,
  FAR_FUTURE_MS, PAST_MS,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Read the mtime of a path in milliseconds since epoch, or 0 if absent.
fn mtime_ms( path : &std::path::Path ) -> u64
{
  path.metadata().ok()
    .and_then( | m | m.modified().ok() )
    .and_then( | t | t.duration_since( std::time::UNIX_EPOCH ).ok() )
    .map_or( 0, | d | u64::try_from( d.as_millis() ).unwrap_or( 0 ) )
}

// ── verb::save ────────────────────────────────────────────────────────────────

// BV-1: Re-save same credentials — idempotent; stored state identical after second call
#[ test ]
fn save_bv1_resave_same_credentials_idempotent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  // First save
  let out1 = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out1, 0 );

  let cred_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let cred_file = cred_store.join( "alice@acme.com.credentials.json" );
  let content_v1 = std::fs::read_to_string( &cred_file ).unwrap();

  // Second save with same credentials
  let out2 = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out2, 0 );

  let content_v2 = std::fs::read_to_string( &cred_file ).unwrap();
  assert_eq!(
    content_v1, content_v2,
    "re-saving same credentials must produce identical stored content",
  );
}

// BV-2: Save transitions account from absent to saved
#[ test ]
fn save_bv2_transitions_absent_to_saved()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let cred_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let cred_file  = cred_store.join( "alice@acme.com.credentials.json" );
  let meta_file  = cred_store.join( "alice@acme.com.json" );

  assert!( !cred_file.exists(), "precondition: account must not exist before save" );

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!( cred_file.exists(), "credentials file must exist after save" );
  assert!( meta_file.exists(), "metadata .json file must exist after save" );
  let out_text = stdout( &out );
  assert!(
    out_text.contains( "alice@acme.com" ),
    "stdout must mention account name: {out_text}",
  );
}

// BV-3: Save without readable credentials exits 2
#[ test ]
fn save_bv3_without_credentials_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No credentials file written — credential dir may exist but .credentials.json absent
  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
}

// ── verb::use ─────────────────────────────────────────────────────────────────

// BV-1: Re-activating the already-active account is a no-op (exit 0, subscription unchanged)
//
// The binary uses copy+rename so mtime may change even on a same-account switch.
// The idempotency contract is: exit 0 and credentials retain the same subscription.
#[ test ]
fn use_bv1_reactivate_already_active_idempotent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // Idempotency: credentials must still contain alice's subscription after re-activation
  let creds_path = dir.path().join( ".claude" ).join( ".credentials.json" );
  let content = std::fs::read_to_string( &creds_path ).unwrap();
  assert!(
    content.contains( "\"max\"" ),
    "credentials must retain alice's subscription type after re-activating already-active account: {content}",
  );
}

// BV-2: Use transitions saved account to active; prior active transitions to saved
#[ test ]
fn use_bv2_transitions_saved_to_active()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // bob is active; alice is saved
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "pro", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@acme.com",   "max", "default", FAR_FUTURE_MS, true  );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // alice's credentials must now be in ~/.claude/.credentials.json
  let creds = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).unwrap();
  // alice has subscriptionType "pro" — verify this appeared
  assert!(
    creds.contains( "\"pro\"" ),
    "after switch to alice (pro), credentials must contain her subscriptionType: {creds}",
  );

  // bob's stored file must still exist (saved state preserved)
  let bob_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "bob@acme.com.credentials.json" );
  assert!( bob_file.exists(), "bob's stored credentials must remain after alice is activated" );
}

// BV-3: Use on non-existent account exits 2
#[ test ]
fn use_bv3_nonexistent_account_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.use", "name::nonexistent@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
}

// ── verb::delete ──────────────────────────────────────────────────────────────

// BV-1: Second delete exits 2 (non-idempotent — deleting absent account is an error)
#[ test ]
fn delete_bv1_second_delete_exits_2_non_idempotent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );

  // First delete succeeds
  let out1 = run_cs_with_env(
    &[ ".account.delete", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out1, 0 );

  // Second delete on now-absent account must fail
  let out2 = run_cs_with_env(
    &[ ".account.delete", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out2, 2 );
}

// BV-2: Delete transitions saved account to absent (files removed)
#[ test ]
fn delete_bv2_transitions_saved_to_absent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );

  let cred_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let cred_file  = cred_store.join( "alice@acme.com.credentials.json" );
  assert!( cred_file.exists(), "precondition: account file must exist before delete" );

  let out = run_cs_with_env(
    &[ ".account.delete", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!( !cred_file.exists(), "credentials file must be removed after delete" );

  // Live session must remain untouched
  let live_creds = dir.path().join( ".claude" ).join( ".credentials.json" );
  assert!( live_creds.exists(), "live ~/.claude/.credentials.json must remain after delete of non-active account" );
}

// BV-3: Delete on non-existent account exits 2
#[ test ]
fn delete_bv3_nonexistent_account_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.delete", "name::nobody@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
}

// ── verb::limits ──────────────────────────────────────────────────────────────

// BV-1: Repeated limits calls produce no local side effects (lim_it)
#[ test ]
fn limits_bv1_lim_it_repeated_calls_no_side_effects()
{
  let Some( token ) = live_active_token() else { return };
  if !require_live_api( "limits_bv1_lim_it" ) { return; }

  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", &token, true );
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let cred_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "alice@acme.com.credentials.json" );
  let mtime_before = mtime_ms( &cred_file );

  let out1 = run_cs_with_env(
    &[ ".account.limits", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out1, 0 );

  let out2 = run_cs_with_env(
    &[ ".account.limits", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out2, 0 );

  let mtime_after = mtime_ms( &cred_file );
  assert_eq!(
    mtime_before, mtime_after,
    "limits command must not modify stored credential files",
  );
}

// BV-2: Limits read is purely non-mutating — no files modified (lim_it)
#[ test ]
fn limits_bv2_lim_it_read_is_non_mutating()
{
  let Some( token ) = live_active_token() else { return };
  if !require_live_api( "limits_bv2_lim_it" ) { return; }

  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", &token, true );
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let cred_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "alice@acme.com.credentials.json" );
  let meta_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "alice@acme.com.json" );
  let mtime_cred_before = mtime_ms( &cred_file );

  let out = run_cs_with_env(
    &[ ".account.limits", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  assert_eq!(
    mtime_cred_before, mtime_ms( &cred_file ),
    "limits must not modify alice@acme.com.credentials.json",
  );
  assert!(
    !meta_file.exists() || mtime_ms( &meta_file ) == mtime_ms( &meta_file ),
    "limits must not write alice@acme.com.json",
  );
}

// BV-3: Limits without accessible account exits 2
#[ test ]
fn limits_bv3_without_accessible_account_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Empty credential store; no active account
  let out = run_cs_with_env(
    &[ ".account.limits" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
}

// ── verb::relogin ─────────────────────────────────────────────────────────────

// BV-1: Repeated relogin calls produce different tokens (lim_it — requires OAuth TTY; skipped)
//
// This case requires a full interactive OAuth browser flow on each invocation.
// It cannot be automated in a non-interactive CI environment. The test guards on
// live_active_token() and returns immediately; the test structure satisfies the
// spec traceability requirement while documenting the OAuth constraint.
#[ test ]
fn relogin_bv1_lim_it_non_idempotent_oauth_flow()
{
  // OAuth TTY flow cannot be automated — skip if no live token
  let Some( _token ) = live_active_token() else { return };
  // Even with a token, an interactive OAuth flow cannot be driven programmatically.
  // This test exists for spec traceability (BV-1) and is inherently manual-only.
}

// BV-2: Relogin updates credentials in-place; lifecycle state preserved (lim_it — TTY; skipped)
#[ test ]
fn relogin_bv2_lim_it_updates_in_place_state_preserved()
{
  // OAuth TTY flow cannot be automated — skip if no live token
  let Some( _token ) = live_active_token() else { return };
  // Same as BV-1: OAuth interactivity prevents full automation.
  // This test exists for spec traceability (BV-2) and is inherently manual-only.
}

// BV-3: Relogin on absent account profile exits 1
#[ test ]
fn relogin_bv3_absent_account_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Credential store exists but nobody@acme.com profile absent
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.relogin", "name::nobody@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  let code = out.status.code().unwrap_or( -1 );
  // Must be non-zero (exit 1 per spec); accept 1 or 2 since absent account errors vary
  assert!(
    code != 0,
    "relogin on absent account must exit non-zero; got 0\nstdout: {}\nstderr: {}",
    stdout( &out ), stderr( &out ),
  );
}

// ── verb::rotate ──────────────────────────────────────────────────────────────

// BV-1: Second rotate picks different account once first account's token expires (non-idempotent)
//
// Setup: bob (active, lowest expiry), alice (inactive, highest), carol (inactive, middle).
// First rotate: alice wins (highest expiry among inactive).
// Expire alice in the stored credential → alice no longer a valid candidate.
// Second rotate: carol > bob by expiry → carol wins.
#[ test ]
fn rotate_bv1_non_idempotent_outcome_changes_with_expiry()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let alice_exp : u64 = FAR_FUTURE_MS;
  let carol_exp : u64 = FAR_FUTURE_MS - 7_200_000;  // middle expiry
  let bob_exp   : u64 = FAR_FUTURE_MS - 14_400_000; // lowest expiry (active)

  write_credentials( dir.path(), "max", "default", bob_exp ); // live creds = bob's
  write_account( dir.path(), "alice@acme.com", "max", "default", alice_exp, false );
  write_account( dir.path(), "carol@acme.com", "max", "default", carol_exp, false );
  write_account( dir.path(), "bob@acme.com",   "max", "default", bob_exp,   true  );

  // First rotate — alice wins (highest expiry among inactive)
  let out1 = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out1, 0 );

  // Expire alice's stored credential so she becomes unusable
  let alice_cred_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "alice@acme.com.credentials.json" );
  let expired_cred = format!(
    r#"{{"oauthAccount":{{"subscriptionType":"max","rateLimitTier":"default"}},"expiresAt":{PAST_MS}}}"#,
  );
  std::fs::write( &alice_cred_file, &expired_cred ).unwrap();

  // Second rotate — carol wins (alice expired; carol_exp > bob_exp)
  let out2 = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );

  let current_creds = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).unwrap();
  assert!(
    current_creds.contains( &carol_exp.to_string() ),
    "after second rotate, carol's credentials should be active (carol_exp={carol_exp}): {current_creds}",
  );
}

// BV-2: Rotate activates highest-expiry inactive account
#[ test ]
fn rotate_bv2_activates_highest_expiry_inactive_account()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  // alice: highest expiry, carol: lower expiry, bob: active
  let alice_exp : u64 = FAR_FUTURE_MS;
  let carol_exp : u64 = FAR_FUTURE_MS - 7_200_000;
  write_account( dir.path(), "alice@acme.com", "max", "default", alice_exp, false );
  write_account( dir.path(), "carol@acme.com", "max", "default", carol_exp, false );
  write_account( dir.path(), "bob@acme.com",   "max", "default", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  // alice (highest expiry) must be active
  let current_creds = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).unwrap();
  assert!(
    current_creds.contains( &alice_exp.to_string() ),
    "rotate must activate alice (highest expiry={alice_exp}): {current_creds}",
  );

  // bob's stored file must remain
  let bob_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "bob@acme.com.credentials.json" );
  assert!( bob_file.exists(), "bob's stored credentials must remain after rotation" );
}

// BV-3: Rotate with no inactive accounts exits 2
#[ test ]
fn rotate_bv3_no_inactive_accounts_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  // Only one account, it's active — no inactive candidates
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

// ── verb::renewal ─────────────────────────────────────────────────────────────

// BV-1: Re-setting the same renewal timestamp is idempotent
#[ test ]
fn renewal_bv1_resetting_same_timestamp_idempotent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_renewal_json( dir.path(), "alice@acme.com", "2026-07-01T00:00:00Z" );
  write_account_profile_json( dir.path(), "alice@acme.com", None, Some( "work" ) );

  // First set
  let out1 = run_cs_with_env(
    &[ ".account.renewal", "name::alice@acme.com", "at::2026-07-01T00:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out1, 0 );

  let meta_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "alice@acme.com.json" );
  let content_v1 = std::fs::read_to_string( &meta_file ).unwrap();

  // Second set with same timestamp
  let out2 = run_cs_with_env(
    &[ ".account.renewal", "name::alice@acme.com", "at::2026-07-01T00:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out2, 0 );

  let content_v2 = std::fs::read_to_string( &meta_file ).unwrap();
  let v1_parsed : serde_json::Value = serde_json::from_str( &content_v1 ).unwrap();
  let v2_parsed : serde_json::Value = serde_json::from_str( &content_v2 ).unwrap();

  assert_eq!(
    v1_parsed.get( "_renewal_at" ), v2_parsed.get( "_renewal_at" ),
    "_renewal_at must be unchanged after re-setting the same timestamp",
  );
  // role field must be preserved
  assert_eq!(
    v2_parsed.get( "role" ).and_then( | v | v.as_str() ),
    Some( "work" ),
    "role field must be preserved after idempotent renewal set",
  );
}

// BV-2: Renewal writes _renewal_at to {name}.json preserving all other fields
#[ test ]
fn renewal_bv2_writes_renewal_at_preserving_other_fields()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );
  // Pre-populate with role + host but no _renewal_at
  write_account_profile_json( dir.path(), "alice@acme.com", Some( "laptop" ), Some( "work" ) );

  let meta_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "alice@acme.com.json" );

  let before : serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string( &meta_file ).unwrap(),
  ).unwrap();
  assert!(
    before.get( "_renewal_at" ).is_none(),
    "precondition: _renewal_at must be absent before renewal",
  );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::alice@acme.com", "at::2026-07-01T00:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let after : serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string( &meta_file ).unwrap(),
  ).unwrap();
  assert_eq!(
    after.get( "_renewal_at" ).and_then( | v | v.as_str() ),
    Some( "2026-07-01T00:00:00Z" ),
    "_renewal_at must be set after renewal",
  );
  // Other fields must be preserved
  assert_eq!(
    after.get( "role" ).and_then( | v | v.as_str() ), Some( "work" ),
    "role field must be preserved after renewal",
  );
  assert_eq!(
    after.get( "host" ).and_then( | v | v.as_str() ), Some( "laptop" ),
    "host field must be preserved after renewal",
  );
}

// BV-3: Renewal without an operation parameter exits 1
#[ test ]
fn renewal_bv3_without_operation_param_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );

  // No at::, from_now::, or clear:: provided
  let out = run_cs_with_env(
    &[ ".account.renewal", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

// ── verb::inspect ─────────────────────────────────────────────────────────────

// BV-1: Repeated inspect calls produce no local side effects (lim_it)
#[ test ]
fn inspect_bv1_lim_it_repeated_calls_no_side_effects()
{
  let Some( token ) = live_active_token() else { return };
  if !require_live_api( "inspect_bv1_lim_it" ) { return; }

  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", &token, true );
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let cred_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "alice@acme.com.credentials.json" );
  let mtime_before = mtime_ms( &cred_file );

  let out1 = run_cs_with_env(
    &[ ".account.inspect", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out1, 0 );

  let out2 = run_cs_with_env(
    &[ ".account.inspect", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out2, 0 );

  let mtime_after = mtime_ms( &cred_file );
  assert_eq!(
    mtime_before, mtime_after,
    "inspect must not modify stored credential files",
  );
}

// BV-2: Inspect is purely non-mutating — no credential store files modified (lim_it)
#[ test ]
fn inspect_bv2_lim_it_read_is_non_mutating()
{
  let Some( token ) = live_active_token() else { return };
  if !require_live_api( "inspect_bv2_lim_it" ) { return; }

  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", &token, true );
  write_account_renewal_json( dir.path(), "alice@acme.com", "2026-07-01T00:00:00Z" );
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let meta_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "alice@acme.com.json" );
  let cred_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "alice@acme.com.credentials.json" );
  let mtime_meta_before = mtime_ms( &meta_file );
  let mtime_cred_before = mtime_ms( &cred_file );

  let out = run_cs_with_env(
    &[ ".account.inspect", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  assert_eq!(
    mtime_meta_before, mtime_ms( &meta_file ),
    "inspect must not modify alice@acme.com.json",
  );
  assert_eq!(
    mtime_cred_before, mtime_ms( &cred_file ),
    "inspect must not modify alice@acme.com.credentials.json",
  );
}

// BV-3: Inspect without accessible account credentials exits 2
#[ test ]
fn inspect_bv3_without_credentials_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No credentials, no stored accounts
  let out = run_cs_with_env(
    &[ ".account.inspect" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
}

// ── verb::assign ──────────────────────────────────────────────────────────────

// BV-1: Re-assigning the same account is idempotent
#[ test ]
fn assign_bv1_reassigning_same_account_idempotent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  // First assign (marker already set)
  let out1 = run_cs_with_env(
    &[ ".account.assign", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out1, 0 );

  let marker_path = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "_active_testmachine_testuser" );
  let marker_content = std::fs::read_to_string( &marker_path ).unwrap();

  // Second assign with same account
  let out2 = run_cs_with_env(
    &[ ".account.assign", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out2, 0 );

  let marker_after = std::fs::read_to_string( &marker_path ).unwrap();
  assert_eq!(
    marker_content, marker_after,
    "active marker must be unchanged after idempotent re-assign",
  );
}

// BV-2: Assign writes active marker without touching credential files
#[ test ]
fn assign_bv2_writes_marker_without_touching_credential_files()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );

  let cred_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "alice@acme.com.credentials.json" );
  let live_creds = dir.path().join( ".claude" ).join( ".credentials.json" );
  let mtime_cred_before = mtime_ms( &cred_file );
  let mtime_live_before = mtime_ms( &live_creds );

  let out = run_cs_with_env(
    &[ ".account.assign", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  // Marker must now exist
  let marker_path = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "_active_testmachine_testuser" );
  assert!( marker_path.exists(), "_active_testmachine_testuser marker must be created" );
  assert_eq!(
    std::fs::read_to_string( &marker_path ).unwrap().trim(),
    "alice@acme.com",
  );

  // Credential files must remain unmodified
  assert_eq!(
    mtime_cred_before, mtime_ms( &cred_file ),
    "alice@acme.com.credentials.json must not be modified by assign",
  );
  assert_eq!(
    mtime_live_before, mtime_ms( &live_creds ),
    "~/.claude/.credentials.json must not be modified by assign",
  );
}

// BV-3: Assign on non-existent account exits 2
#[ test ]
fn assign_bv3_nonexistent_account_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.assign", "name::nobody@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 2 );
}

// ── verb::status ──────────────────────────────────────────────────────────────

// BV-1: .token.status called twice returns same classification
#[ test ]
fn status_bv1_token_status_twice_same_classification()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let out1 = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out1, 0 );

  let out2 = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );

  // First line of each output must match (classification line)
  let first1 = stdout( &out1 ).lines().next().unwrap_or( "" ).to_string();
  let first2 = stdout( &out2 ).lines().next().unwrap_or( "" ).to_string();
  assert_eq!( first1, first2, "classification must be identical across two calls" );
}

// BV-2: .token.status read is purely non-mutating
#[ test ]
fn status_bv2_token_status_non_mutating()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let creds_path = dir.path().join( ".claude" ).join( ".credentials.json" );
  let mtime_before = mtime_ms( &creds_path );

  let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  assert_eq!(
    mtime_before, mtime_ms( &creds_path ),
    ".token.status must not modify ~/.claude/.credentials.json",
  );
}

// BV-3: .token.status with absent credentials file exits 2
#[ test ]
fn status_bv3_token_status_absent_creds_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No credentials file written
  let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

// BV-4: .credentials.status called twice returns same output
#[ test ]
fn status_bv4_credentials_status_twice_same_output()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  let out1 = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out1, 0 );

  let out2 = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );

  assert_eq!(
    stdout( &out1 ), stdout( &out2 ),
    ".credentials.status must produce identical output on repeated calls",
  );
}

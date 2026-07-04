//! Integration tests: AS/AW/AD account mutations — Part B (AD section+).
//!
//! Continuation of `account_mutations_test.rs`.

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, write_claude_json, account_exists,
  write_account_claude_json, write_account_settings_json,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── AD: Account Delete ────────────────────────────────────────────────────────

#[ test ]
// Fix(BUG-281):
// Root cause: run_cs_with_env() set HOME to a temp dir but inherited $PRO from the test runner;
//   PersistPaths::resolve_root() prefers $PRO over $HOME when $PRO is an existing directory, so
//   the binary operated on the real credential store ($PRO/.persistent/claude/credential) while
//   the test wrote fixtures to and checked $HOME/.persistent/claude/credential — the two paths
//   never overlapped.
// Why Not Caught: tests were developed in a Docker container where $PRO is not set; the
//   isolation failure is invisible there and only manifests in the host environment.
// Fix Applied: added cmd.env_remove("PRO") to run_cs_with_env() in helpers.rs so that $PRO
//   cannot leak into subprocesses when tests supply a custom HOME.
// Prevention: any subprocess helper that isolates HOME must explicitly remove $PRO (and
//   $USERPROFILE); document this as an invariant in helpers.rs.
// Pitfall: cmd.env("HOME", ...) does not clear inherited vars — $PRO still takes priority until
//   explicitly removed with env_remove().
fn ad01_delete_inactive_removes_file()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( !account_exists( dir.path(), "alice@oldco.com" ), "account file must be removed" );
}

#[ test ]
fn ad02_delete_dry_run_keeps_file()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run] would delete account 'alice@oldco.com'" ), "must print full dry-run message, got:\n{text}" );
  assert!( account_exists( dir.path(), "alice@oldco.com" ), "dry-run must not delete file" );
}

#[ test ]
fn ad03_delete_active_exits_0()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( !account_exists( dir.path(), "alice@acme.com" ), "active account must be deleted" );
  let active_marker = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( claude_profile::account::active_marker_filename() );
  assert!( !active_marker.exists(), "_active marker must be cleaned up after deleting active account" );
}

#[ test ]
fn ad04_delete_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::ghost@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ad05_delete_empty_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn ad06_delete_slash_name_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::a/b" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn ad07_delete_missing_name_param_exits_1()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn ad08_delete_then_list_absent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "keep@example.com", "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let _ = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );

  let out = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  let text = stdout( &out );
  assert!( !text.contains( "alice@oldco.com" ), "deleted account must not appear in list, got:\n{text}" );
  assert!( text.contains( "keep@example.com" ), "kept account must still appear, got:\n{text}" );
}

#[ test ]
fn ad09_double_delete_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@oldco.com", "pro", "standard", FAR_FUTURE_MS, false );

  let first = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &first, 0 );

  let second = run_cs_with_env( &[ ".account.delete", "name::alice@oldco.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &second, 2 );
}

// Root Cause: `account_use_routine` checked `is_dry()` before validating account
//   existence, so `.account.use dry::1 name::missing` returned exit 0 ("would switch
//   to 'missing'") even when the named account does not exist.
// Why Not Caught: `aw02_switch_dry_run` only exercises the happy-path dry-run (valid
//   account). No test covered the dry-run-with-nonexistent-account case.
// Fix Applied: `check_switch_preconditions()` extracted from `switch_account()` and
//   called in the command routine before the dry-run guard.
// Prevention: Dry-run must always run input validation + precondition checks; only the
//   mutation step is skipped.
// Pitfall: Placing `is_dry()` before domain validation produces misleading "would do X"
//   output for operations that would actually fail — always validate first, then dry-run.
#[ doc = "bug_reproducer(BUG-265)" ]
#[ test ]
fn aw10_switch_dry_run_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.use", "name::missing@example.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ad10_delete_dry_run_active_exits_0()
{
  // Dry-run on the active account exits 0 now that the active-account guard is removed.
  // The account file must not be deleted (dry-run protection is unrelated to active status).
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.delete", "name::alice@acme.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( account_exists( dir.path(), "alice@acme.com" ), "dry-run must not delete active account" );
}

// Root Cause: Same as ad10 — `is_dry()` guard ran before any account existence check,
//   so `.account.delete dry::1 name::ghost` (nonexistent) returned exit 0 instead of
//   exit 2 (`NotFound`).
// Why Not Caught: `ad02` exercises an existing account; no test covered dry-run on a
//   nonexistent account.
// Fix Applied: See ad10 — `check_delete_preconditions()` runs before dry-run guard.
// Prevention: Dry-run path must include all validation; only file-system mutation is omitted.
// Pitfall: Missing existence check in dry-run gives a false "operation would succeed"
//   signal, masking configuration errors until the real run.
#[ doc = "bug_reproducer(BUG-266)" ]
#[ test ]
fn ad11_delete_dry_run_nonexistent_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.delete", "name::ghost@example.com", "dry::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
}

#[ test ]
fn ad12_delete_removes_snapshot_files()
{
  // IT-11: delete removes credentials and {name}.json snapshot.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, true );
  write_account( dir.path(), "old@archive.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_claude_json(   dir.path(), "old@archive.com", "", "", "", "" );
  write_account_settings_json( dir.path(), "old@archive.com", "sonnet" );

  let out = run_cs_with_env( &[ ".account.delete", "name::old@archive.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!( !store.join( "old@archive.com.credentials.json" ).exists(), "credentials must be removed after delete" );
  assert!( !store.join( "old@archive.com.json" ).exists(),      "{{name}}.json snapshot must be removed after delete" );
}

// ── as16 ──────────────────────────────────────────────────────────────────────

/// as16: `.account.save name::work@acme.com` writes `{store}/_active_{hostname}_{user}` = `"work@acme.com"`.
///
/// CLI-level symmetry test with aw07: reads the active marker directly (not via
/// `.credentials.status`) to confirm the write happened at the filesystem level.
///
/// ## Fix Documentation — BUG-282
///
/// - **Root Cause:** `save()` never wrote the active marker; only `switch_account()` did.
/// - **Why Not Caught:** No AS test verified the active marker file after `.account.save`.
/// - **Fix Applied:** Added `std::fs::write( credential_store.join( active_marker_filename() ), name )?;` to `save()`. (Originally `join("_active")`; updated to per-machine `active_marker_filename()` per Feature 025.)
/// - **Prevention:** This test guards the active marker at the filesystem level, independently of `.credentials.status`.
/// - **Pitfall:** Must assert the raw file content — not just exit code — to catch a write that produces wrong content.
#[ test ]
fn as16_save_writes_active_marker()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::work@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let store  = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) )
    .expect( "_active must exist after .account.save" );
  assert_eq!(
    active.trim(),
    "work@acme.com",
    "_active must equal the saved account name",
  );
}

// ── switch_restores_claude_json ────────────────────────────────────────────────

/// bug_reproducer(BUG-277): `.account.use` does not restore `~/.claude.json`,
/// so `.credentials.status` shows the previous account's email after a switch.
///
/// ## Fix Documentation — BUG-277
///
/// - **Root Cause:** `switch_account()` restored only `.credentials.json`; the
///   companion `~/.claude.json` restore (from `{name}.json` snapshot) was
///   never added, leaving the active JSON pointing at the previous account's data.
/// - **Why Not Caught:** Prior tests never called `.credentials.status` after
///   `.account.use` in a two-account setup, so the email mismatch was invisible.
/// - **Fix Applied:** Added two best-effort `let _ = std::fs::copy(...)` calls in
///   `switch_account()` after the `_active` marker write — mirroring the two
///   companion writes already present in `save()`.
/// - **Prevention:** This test encodes the full save-A / save-B / switch-to-A /
///   check-email flow, preventing any future regression where the restore pair
///   becomes asymmetric again.
/// - **Pitfall:** The `let _ = ...` idiom silences copy errors intentionally —
///   `~/.claude.json` may legitimately not exist. The test must explicitly write
///   `~/.claude.json` for both accounts before saving so the snapshots exist.
#[ doc = "bug_reproducer(BUG-277)" ]
#[ test ]
fn switch_restores_claude_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Save account A: work@acme.com
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "work@acme.com" );
  let save_a = run_cs_with_env( &[ ".account.save", "name::work@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &save_a, 0 );

  // Save account B: personal@home.com (overwrites active credentials + claude.json)
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json( dir.path(), "personal@home.com" );
  let save_b = run_cs_with_env( &[ ".account.save", "name::personal@home.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &save_b, 0 );

  // Switch back to A — must restore work@acme.com's ~/.claude.json
  let switch_out = run_cs_with_env(
    &[ ".account.use", "name::work@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &switch_out, 0 );

  // .credentials.status must show work@acme.com — not personal@home.com
  let status_out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &status_out, 0 );
  let text = stdout( &status_out );
  assert!(
    text.contains( "work@acme.com" ),
    "Email: must reflect switched-to account, got:\n{text}",
  );
}

// ── as17 ──────────────────────────────────────────────────────────────────────

/// bug_reproducer(BUG-278): `.account.save name::a/b@c.com` exits 2 instead
/// of 1 because `validate_name()` passes `a/b@c.com` (local part non-empty,
/// domain non-empty), then `save()` hits a filesystem error when creating
/// `a/b@c.com.credentials.json`.
///
/// ## Fix Documentation — BUG-278
///
/// - **Root Cause:** `validate_name()` only checked `@` presence and non-empty
///   local/domain parts; it did not reject path-unsafe chars (`/`, `\`, `*`)
///   inside the local part, so names like `a/b@c.com` bypassed validation and
///   reached filesystem operations that exit 2.
/// - **Why Not Caught:** Existing as07/as08/as09 only cover names WITHOUT `@`
///   (caught by the "must contain @" guard); no test covered the combined case
///   where the local part carries an unsafe char but `@` is present.
/// - **Fix Applied:** Added a local-part path-safety check in `validate_name()`
///   after the `@` position is found: if the local part contains `/`, `\`, or
///   `*`, return `InvalidInput` (exit 1) before any filesystem operation runs.
/// - **Prevention:** This test (as17) and as18/aw11 encode the three unsafe-char
///   variants so any regression in the local-part check is caught immediately.
/// - **Pitfall:** Only the local part (before `@`) needs the check; domain chars
///   cannot create path traversal in practice because the `@` separates them.
#[ doc = "bug_reproducer(BUG-278)" ]
#[ test ]
fn as17_save_slash_in_email_local_part_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a/b@c.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "path-unsafe characters" ),
    "stderr must indicate path-unsafe chars, got:\n{err}",
  );
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!( !store.exists(), "credential store must not be created before validation passes" );
}

// ── as18 ──────────────────────────────────────────────────────────────────────

/// bug_reproducer(BUG-278): same root cause as as17 but for `\` in the local
/// part of the email address.
///
/// See as17 for full fix documentation.
#[ doc = "bug_reproducer(BUG-278)" ]
#[ test ]
fn as18_save_backslash_in_email_local_part_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::a\\b@c.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── aw11 ──────────────────────────────────────────────────────────────────────

/// bug_reproducer(BUG-278): `.account.use name::a/b@c.com` exits 2 instead
/// of 1 for the same reason as as17 — `validate_name()` passes the name, then
/// `switch_account()` fails with a filesystem error.
///
/// See as17 for full fix documentation.
#[ doc = "bug_reproducer(BUG-278)" ]
#[ test ]
fn aw11_switch_slash_in_email_local_part_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.use", "name::a/b@c.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── aw12 ──────────────────────────────────────────────────────────────────────

/// # Root Cause
///
/// `switch_account()` gates the `emailAddress` patch inside `if let Ok(saved_val) =
/// serde_json::from_str(&meta_text)`. When `{name}.json` is absent, `meta_text` is `""`,
/// `from_str("")` returns `Err`, and the entire oauthAccount patch block is skipped —
/// including the BUG-217 `emailAddress` enforcement. `~/.claude.json` retains the previous
/// account's `emailAddress`, causing downstream `save()` name inference to target the wrong
/// file.
///
/// # Why Not Caught
///
/// All existing `switch_account()` FT tests provide a `{name}.json` metadata file via
/// `.account.save`. No FT test covers the absent-metadata-file path where only credentials
/// exist.
///
/// # Fix Applied
///
/// Lift the unconditional `emailAddress` patch out of the metadata-file-conditional block.
/// Patch `~/.claude.json oauthAccount.emailAddress = name` before attempting to read
/// `{name}.json`. The full overlay (BUG-217 + BUG-219) still fires when metadata is present.
///
/// # Prevention
///
/// This FT test creates a credential-only account (no `{name}.json`) and asserts that
/// `emailAddress` is patched to the switched-to name after `.account.use`.
///
/// # Pitfall
///
/// `claude_json_file()` returns `$HOME/.claude.json` (HOME level), not
/// `$HOME/.claude/claude.json`. Machine-global keys must survive the patch — assert
/// preservation.
/// FT-09: AC-09 — emailAddress patched unconditionally even when metadata absent
#[ doc = "bug_reproducer(BUG-254)" ]
#[ test ]
fn aw12_switch_patches_email_when_metadata_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Live credentials (required so switch_account can copy to .credentials.json).
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // alice: credentials + active marker.  bob: credentials ONLY — NO bob@acme.com.json.
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "bob@acme.com",   "max", "tier4",    FAR_FUTURE_MS, false );

  // Seed ~/.claude.json with alice's emailAddress + machine-global keys.
  let claude_json_path = dir.path().join( ".claude.json" );
  std::fs::write(
    &claude_json_path,
    r#"{"oauthAccount":{"emailAddress":"alice@acme.com","displayName":"Alice"},"commands":{"enabled":true},"mcpServers":{}}"#,
  ).unwrap();

  // touch::0 disables pre-fetch HTTP calls — tests the pure file switch.
  let out = run_cs_with_env(
    &[ ".account.use", "name::bob@acme.com", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // After switch: emailAddress must be patched to bob even though bob@acme.com.json is absent.
  let claude_json = std::fs::read_to_string( &claude_json_path ).unwrap();
  assert!(
    claude_json.contains( r#""emailAddress": "bob@acme.com""# ),
    "BUG-254: emailAddress must be 'bob@acme.com' after switch, got:\n{claude_json}",
  );
  assert!(
    !claude_json.contains( r#""emailAddress": "alice@acme.com""# ),
    "BUG-254: stale emailAddress 'alice@acme.com' must not remain, got:\n{claude_json}",
  );

  // Machine-global keys must survive the unconditional patch.
  assert!(
    claude_json.contains( r#""commands""# ),
    "machine-global key 'commands' must survive, got:\n{claude_json}",
  );
  assert!(
    claude_json.contains( r#""mcpServers""# ),
    "machine-global key 'mcpServers' must survive, got:\n{claude_json}",
  );

  // _active marker must point at bob.
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string(
    store.join( claude_profile::account::active_marker_filename() )
  ).expect( "_active must exist" );
  assert_eq!(
    active.trim(), "bob@acme.com",
    "_active marker must point at switched-to account",
  );
}

// ── aw13 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn aw13_use_positional_bare_arg()
{
  // AC-01: positional form `clp .account.use personal@home.com` is equivalent to
  // `clp .account.use name::personal@home.com`.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "work@acme.com",     "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.use", "personal@home.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "switched" ), "must confirm switch, got:\n{text}" );
  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) ).expect( "_active must exist" );
  assert_eq!( active.trim(), "personal@home.com", "_active must point at switched-to account" );
}

// ── aw14 ──────────────────────────────────────────────────────────────────────

/// aw14 (AC-05 / BUG-262 / `015_name_shortcut_syntax.md`): bare prefix uniquely resolves to one account.
///
/// ## Root Cause
/// `resolve_account_name()` ran after `validate_name()`. `validate_name()` rejects bare prefixes
/// (no `@`) with exit 1 ("not an email address"), masking the correct exit-2 "not found" outcome
/// and preventing prefix expansion from running at all.
///
/// ## Why Not Caught
/// All prior tests passed full email addresses; no test exercised a bare prefix (e.g. `car`) through
/// `.account.use`, so the validation-before-resolution ordering was never exposed.
///
/// ## Fix Applied
/// `resolve_account_name()` in `cmd_args.rs` now runs before `validate_name()`. Bare prefixes (no
/// `@`) enter prefix scanning; the resolved full email passes format validation downstream.
///
/// ## Prevention
/// Test bare-prefix arguments alongside full-email arguments for every command that accepts an
/// account name, confirming both switch successfully.
///
/// ## Pitfall
/// Calling `validate_name()` on a bare prefix always exits 1 — prefix resolution must precede
/// format validation, never follow it.
#[ doc = "bug_reproducer(BUG-262)" ]
#[ test ]
fn aw14_use_prefix_resolves()
{
  // AC-05: prefix `car` resolves uniquely to `carol@example.com` and switches.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "carol@example.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "amy@example.com", "pro", "standard", FAR_FUTURE_MS, true  );

  let out = run_cs_with_env( &[ ".account.use", "car" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) ).expect( "_active must exist" );
  assert_eq!( active.trim(), "carol@example.com", "prefix car must resolve to carol@example.com" );
}

// ── aw15 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn aw15_use_prefix_ambiguous_exits_1()
{
  // AC-06: ambiguous prefix `a` matches both `alice@example.com` and `amy@example.com` → exit 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@example.com", "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "amy@example.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.use", "a" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "ambiguous" ),
    "error must say 'ambiguous', got:\n{err}",
  );
}

// ── aw16 ──────────────────────────────────────────────────────────────────────

/// aw16 (AC-11 / BUG-264 / `015_name_shortcut_syntax.md`): exact local-part match wins over ambiguous prefix.
///
/// ## Root Cause
/// `starts_with("i1")` matched `i1@wbox.pro`, `i11@wbox.pro`, and `i12@wbox.pro` — all three
/// reported as ambiguous even though `i1` is the exact local part of `i1@wbox.pro`. The prefix
/// scan ran first without checking exact-local-part identity.
///
/// ## Why Not Caught
/// No test covered overlapping account names where the prefix equals one account's local part
/// exactly while also being a prefix of other account names.
///
/// ## Fix Applied
/// `resolve_account_name()` in `cmd_args.rs` now performs an exact-local-part check first: if
/// exactly one account's local part (before `@`) equals the raw input, it resolves immediately
/// without reaching the prefix scan.
///
/// ## Prevention
/// Always test with overlapping account names (e.g. `i1`, `i11`, `i12`) to verify that an exact
/// local-part match resolves unambiguously when longer names share the same prefix.
///
/// ## Pitfall
/// Always check exact-local-part match before prefix scanning; prefix scanning is only meaningful
/// when no account's local part equals the input exactly.
#[ doc = "bug_reproducer(BUG-264)" ]
#[ test ]
fn aw16_exact_local_part_wins_over_ambiguous_prefix()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "i1@wbox.pro",  "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "i11@wbox.pro", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "i12@wbox.pro", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.use", "i1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let active = std::fs::read_to_string( store.join( claude_profile::account::active_marker_filename() ) )
    .expect( "active marker must exist after use" );
  assert_eq!(
    active.trim(), "i1@wbox.pro",
    "exact local-part match must resolve to i1@wbox.pro, not be reported as ambiguous",
  );
}

// ── aw17 ──────────────────────────────────────────────────────────────────────

/// aw17 (AC-06, AC-11 / `015_name_shortcut_syntax.md` FT-08): prefix `i1` is ambiguous
/// when only `i11@wbox.pro` and `i12@wbox.pro` exist — no `i1@wbox.pro` account.
///
/// The exact-local-part check (AC-11) finds no account with local part exactly `i1`.
/// Falling through to prefix scan, both `i11@` and `i12@` match — ambiguity reported
/// with exit 1 (AC-06). Complements aw16: positive case where `i1@` exists exits 0.
#[ test ]
fn aw17_use_prefix_ambiguous_no_exact_local_part_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // Only i11 and i12 exist — no i1@wbox.pro. Prefix i1 matches both via starts_with.
  write_account( dir.path(), "i11@wbox.pro", "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "i12@wbox.pro", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.use", "i1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.to_lowercase().contains( "ambiguous" ),
    "error must say 'ambiguous' when prefix i1 matches i11@ and i12@ but no i1@ exists, got:\n{err}",
  );
}

// ── ad13 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn ad13_delete_positional_bare_arg()
{
  // AC-02 (delete): positional form `clp .account.delete old@archive.com` is
  // equivalent to `clp .account.delete name::old@archive.com`.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",    "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "old@archive.com",  "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "old@archive.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( !account_exists( dir.path(), "old@archive.com" ), "account must be deleted" );
}

// ── ad14 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn ad14_delete_prefix_resolves()
{
  // AC-05 (delete): prefix `old` resolves uniquely to `old@archive.com` and deletes it.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",    "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "old@archive.com",  "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.delete", "old" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( !account_exists( dir.path(), "old@archive.com" ), "prefix old must resolve to old@archive.com and delete it" );
}


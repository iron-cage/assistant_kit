//! Integration tests: AR (Account Relogin) — Part B (aw26–aw30+).
//!
//! Continuation of `account_relogin_test.rs`.

use crate::cli_runner::{
  run_cs, run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account,
  write_account_with_token, live_active_token, require_live_api,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

/// aw26: `.account.use.help` lists `touch`, `imodel`, `effort`, `trace`, and `refresh`
/// parameters (IT-23).
///
/// Extended from Feature 027 (touch/imodel/effort) to include `trace::` per BUG-207,
/// and `refresh::` per BUG-230.
#[ test ]
fn aw26_help_shows_touch_imodel_effort()
{
  let out  = run_cs( &[ ".account.use.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "touch" ),   "`.account.use.help` must list `touch` param, got:\n{text}" );
  assert!( text.contains( "imodel" ),  "`.account.use.help` must list `imodel` param, got:\n{text}" );
  assert!( text.contains( "effort" ),  "`.account.use.help` must list `effort` param, got:\n{text}" );
  assert!( text.contains( "trace" ),   "`.account.use.help` must list `trace` param, got:\n{text}" );
  assert!( text.contains( "refresh" ), "`.account.use.help` must list `refresh` param, got:\n{text}" );
}

/// aw27: `lim_it` — live token + `touch::1` → switch exits 0 (IT-17/IT-19).
///
/// Uses real credentials. Whether `pre_switch_touch_ctx` returns `Some` (idle) or `None`
/// (active/fetch fail) depends on live quota state; either path must exit 0. The subprocess
/// is fire-and-forget — its success or failure does not affect the command exit code.
#[ test ]
fn aw27_lim_it_touch_with_live_token()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "aw27: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Create ~/.claude/ so switch_account() can copy credentials there (it does not create the dir).
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );
  // Source account (provides live credentials in the store).
  write_account_with_token( dir.path(), "source@example.com", &token, true );
  // Target account — same token so quota fetch may succeed if account is idle.
  write_account_with_token( dir.path(), "target@example.com", &token, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "touch::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "switch with live token must exit 0 and say switched, got:\n{}", stdout( &out ),
  );
}

/// aw28: `trace::1 touch::1` live token — subprocess always dispatched when quota fetch OK (FT-11, IT-24).
///
/// `lim_it` — skips without a live OAuth token. Verifies reading, quota fetch, and subprocess
/// dispatch trace lines. Fix(BUG-285): idle check removed — subprocess always fires when fetch
/// succeeds regardless of `resets_at` state; `idle check:` trace line no longer emitted.
///
/// Fix(BUG-207): `pre_switch_touch_ctx` had no `trace` param — all operations were invisible.
/// Root cause: Feature 027 put `trace::` Out-of-Scope; no trace lines were emitted for .account.use.
/// Pitfall: trace lines go to stderr, not stdout — assert on `stderr(&out)`, not `stdout(&out)`.
#[ test ]
fn aw28_lim_it_trace_idle_account_all_lines()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "aw28: no live token — skipping" );
    return;
  };
  require_live_api( "aw28" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Create ~/.claude/ so switch_account() can copy credentials there (it does not create the dir).
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );
  write_account_with_token( dir.path(), "source@example.com", &token, true );
  write_account_with_token( dir.path(), "target@example.com", &token, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "aw28: stdout must contain 'switched', got:\n{}", stdout( &out ),
  );
  let err = stderr( &out );
  assert!(
    err.contains( " · account.use  " ),
    "aw28: stderr must contain account.use trace prefix, got:\n{err}",
  );
  assert!(
    err.contains( "reading" ) && err.contains( "reading: OK" ),
    "aw28: stderr must contain reading + reading: OK trace lines, got:\n{err}",
  );
  // Fix(BUG-285): idle check removed — no `idle check:` line emitted; only scheduled + spawned.
  assert!(
    !err.contains( "idle check:" ),
    "aw28: `idle check:` trace line must not appear (BUG-285 removed idle check), got:\n{err}",
  );
  if err.contains( "quota fetch: OK" )
  {
    assert!(
      err.contains( "subprocess: scheduled (idle check removed)" ),
      "aw28: fetch-OK path must emit subprocess: scheduled (idle check removed), got:\n{err}",
    );
    assert!(
      err.contains( "model:" ),
      "aw28: fetch-OK path must emit model: line, got:\n{err}",
    );
    assert!(
      err.contains( "subprocess: spawned" ),
      "aw28: fetch-OK path must emit subprocess: spawned (always; BUG-285 fix), got:\n{err}",
    );
  }
  else
  {
    eprintln!( "aw28: quota fetch failed — fetch-OK assertions skipped" );
  }
}

/// aw29: `trace::1 touch::1` live account — subprocess always spawned when quota fetch OK (FT-12).
///
/// `lim_it` — skips without a live OAuth token. Verifies that when quota fetch succeeded,
/// the subprocess is always dispatched regardless of `resets_at` state (Fix(BUG-285): idle
/// check removed; `AlreadyActive` variant removed from `PreSwitchOutcome`; subprocess is
/// idempotent and exits immediately when already active).
#[ test ]
fn aw29_lim_it_trace_active_account_subprocess_skipped()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "aw29: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Create ~/.claude/ so switch_account() can copy credentials there (it does not create the dir).
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );
  write_account_with_token( dir.path(), "source@example.com", &token, true );
  write_account_with_token( dir.path(), "target@example.com", &token, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( " · account.use  " ),
    "aw29: stderr must contain account.use trace prefix, got:\n{err}",
  );
  // Fix(BUG-285): no idle check — subprocess always fires when fetch OK.
  // Old trace `idle check: resets_at=present → already active` no longer exists.
  if err.contains( "quota fetch: OK" )
  {
    assert!(
      err.contains( "subprocess: scheduled (idle check removed)" ),
      "aw29: fetch-OK path must emit subprocess: scheduled (idle check removed), got:\n{err}",
    );
    assert!(
      err.contains( "model:" ),
      "aw29: fetch-OK path must emit model: line, got:\n{err}",
    );
    assert!(
      err.contains( "effort:" ),
      "aw29: fetch-OK path must emit effort: line, got:\n{err}",
    );
    assert!(
      err.contains( "subprocess: spawned" ),
      "aw29: fetch-OK path must emit subprocess: spawned (always; BUG-285 fix), got:\n{err}",
    );
    assert!(
      !err.contains( "subprocess: skipped (reason: already active)" ),
      "aw29: subprocess: skipped (reason: already active) must not appear (BUG-285 fix), got:\n{err}",
    );
  }
  else
  {
    eprintln!( "aw29: quota fetch failed — fetch-OK assertions skipped" );
  }
}

/// aw30: `trace::1 touch::1` invalid token → fetch-err + subprocess-skipped trace lines (FT-13).
///
/// Uses an invalid `accessToken` so quota fetch fails. Verifies that:
/// - reading: OK and quota fetch: Err( are emitted
/// - subprocess: skipped (reason: fetch failed) is emitted
/// - idle check: and model: lines are NOT emitted (short-circuit on fetch failure)
/// - switch still exits 0 (fetch failure is non-fatal to the switch)
///
/// Fix(BUG-207): `pre_switch_touch_ctx` must emit fetch-err trace when quota API fails.
/// Root cause: original function collapsed all failures into None with no tracing.
/// Pitfall: the switch exits 0 regardless of fetch failure; assert on stderr, not exit code.
#[ test ]
fn aw30_trace_fetch_failure_skips_idle_model_lines()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // Invalid token ensures quota fetch fails with auth error.
  write_account_with_token( dir.path(), "target@example.com", "invalid-token-for-fetch-failure", false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "aw30: fetch failure must not block switch, got:\n{}", stdout( &out ),
  );
  let err = stderr( &out );
  assert!(
    err.contains( " · account.use  " ),
    "aw30: stderr must contain account.use trace prefix, got:\n{err}",
  );
  assert!(
    err.contains( "reading: OK" ),
    "aw30: stderr must contain reading: OK (credential file was read), got:\n{err}",
  );
  assert!(
    err.contains( "quota fetch: Err(" ),
    "aw30: stderr must contain quota fetch: Err(, got:\n{err}",
  );
  assert!(
    err.contains( "subprocess: skipped (reason: fetch failed)" ),
    "aw30: stderr must contain subprocess: skipped (reason: fetch failed), got:\n{err}",
  );
  assert!(
    !err.contains( "idle check:" ),
    "aw30: fetch-failed path must NOT emit idle check: line, got:\n{err}",
  );
  assert!(
    !err.contains( "model:" ),
    "aw30: fetch-failed path must NOT emit model: line, got:\n{err}",
  );
  assert!(
    err.contains( "expiry check: valid" ),
    "aw30: fetch Err + FAR_FUTURE_MS expiresAt must emit expiry check: valid, got:\n{err}",
  );
}

/// aw31: `trace::1 touch::0` → no ` · account.use` timestamp lines emitted (FT-14, EC-7).
///
/// When `touch::0` is set, `pre_switch_touch_ctx` is never called — no quota fetch
/// operations occur, so no ` · account.use` timestamp lines should appear on stderr.
/// The `trace::1` parameter is accepted (exit 0) but has no effect without touch operations.
#[ test ]
fn aw31_trace_touch_disabled_no_trace_lines()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "target@example.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "touch::0", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "aw31: touch::0 trace::1 must not block switch, got:\n{}", stdout( &out ),
  );
  let err = stderr( &out );
  assert!(
    !err.contains( " · account.use  " ),
    "aw31: touch::0 must produce no account.use trace lines, got stderr:\n{err}",
  );
}

/// aw32: `trace::bad` → exit 1; stderr names all four valid values (FT-16, IT-26).
///
/// Validation fires before any filesystem I/O — empty account store is sufficient.
///
/// Fix(BUG-207): `trace::` was absent from .account.use; before fix, `trace::bad` produced
///   "unrecognized parameter" (different message), not an invalid-value exit 1.
/// Root cause: parameter not registered; `parse_int_flag` never ran; parse never saw the value.
/// Pitfall: must assert on exit code AND stderr content — exit code alone is insufficient.
#[ test ]
fn aw32_trace_bad_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::any@example.com", "trace::bad" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( '0' ) && err.contains( '1' ) && err.contains( "false" ) && err.contains( "true" ),
    "aw32: stderr must name all valid trace:: values (0, 1, false, true), got:\n{err}",
  );
}

// ── it_trace_account_save_accepted ────────────────────────────────────────────

/// EC-11 (023): `trace::1` accepted by `.account.save` — no "Unknown parameter" error.
/// TSK-210 RED gate: fails before `trace::` is registered (exit 1 + Unknown parameter).
#[ test ]
fn it_trace_account_save_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // write_account creates the credential store dir so require_credential_store succeeds.
  write_account( dir.path(), "existing@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "dry::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .account.save, got stderr:\n{err}",
  );
  assert!(
    err.contains( " · " ),
    "trace::1 must emit trace lines to stderr for .account.save, got:\n{err}",
  );
}

// ── it_trace_account_use_accepted ─────────────────────────────────────────────

/// EC-12 (023): `trace::1` accepted by `.account.use` — no "Unknown parameter" error.
/// `test@example.com` does not exist so command exits 2, but must not exit 1 for unknown-param.
/// `.account.use` already has `trace::` registered — this test is expected to pass before impl.
#[ test ]
fn it_trace_account_use_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::test@example.com", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .account.use, got stderr:\n{err}",
  );
}

// ── it_trace_account_delete_accepted ──────────────────────────────────────────

/// EC-13 (023): `trace::1` accepted by `.account.delete` — no "Unknown parameter" error.
/// TSK-210 RED gate: fails before `trace::` is registered (exit 1 + Unknown parameter).
#[ test ]
fn it_trace_account_delete_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account creates the credential store dir and the target account file.
  write_account( dir.path(), "test@example.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.delete", "name::test@example.com", "dry::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .account.delete, got stderr:\n{err}",
  );
  assert!(
    err.contains( " · " ),
    "trace::1 must emit trace lines to stderr for .account.delete, got:\n{err}",
  );
}

// ── it_trace_account_relogin_accepted ─────────────────────────────────────────

/// EC-14 (023): `trace::1` accepted by `.account.relogin` — no "Unknown parameter" error.
/// TSK-210 RED gate: fails before `trace::` is registered (exit 1 + Unknown parameter).
#[ test ]
fn it_trace_account_relogin_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.relogin", "dry::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .account.relogin, got stderr:\n{err}",
  );
  assert!(
    err.contains( " · " ),
    "trace::1 must emit trace lines to stderr for .account.relogin, got:\n{err}",
  );
}

// ── Bug Reproducers ───────────────────────────────────────────────────────────

/// bug_reproducer(BUG-209): `.account.save` reads stale `emailAddress` from `~/.claude.json`
/// instead of the per-machine `_active` marker after `.account.use B`.
///
/// ## Fix Documentation — BUG-209
///
/// - **Root Cause:** `account_save_routine()` reads top-level `emailAddress` from `~/.claude.json`
///   as the fallback name when `name::` is omitted. `switch_account()` patches only the
///   `oauthAccount` subtree — `emailAddress` remains stale. After `.account.use B`, running
///   `.account.save` (no `name::`) saves under account `A` and overwrites `_active` with `A`.
/// - **Why Not Caught:** No test exercised the two-step sequence `.account.use B` →
///   `.account.save` (no `name::`). `as15` tested emailAddress inference without a stale case.
/// - **Fix Applied:** Read `_active_{hostname}_{user}` (per `active_marker_filename()`) instead of
///   `emailAddress`. The `_active` marker is authoritative — `switch_account()` always writes it.
/// - **Prevention:** Any code that infers "current account name" must read from the `_active`
///   marker, not from `~/.claude.json` fields that are not synced on every switch.
/// - **Pitfall:** `emailAddress` in `~/.claude.json` becomes stale immediately after any
///   `switch_account()` call; never use it as a proxy for the active account name.
#[ test ]
fn mre_bug_209_account_save_uses_active_marker_not_stale_email()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Credentials required for save to succeed (write_credentials also creates ~/.claude/).
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Write stale ~/.claude.json: top-level emailAddress = "a@test.com" (prior account).
  // switch_account() never updates this field — it is always stale after a switch.
  std::fs::write(
    dir.path().join( ".claude.json" ),
    r#"{"emailAddress":"a@test.com","oauthAccount":{"emailAddress":"b@test.com"}}"#,
  ).unwrap();

  // Write _active marker = "b@test.com" — set by prior .account.use b@test.com.
  let store = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  let marker = store.join( claude_profile::account::active_marker_filename() );
  std::fs::write( &marker, "b@test.com" ).unwrap();

  // .account.save with no name:: — must read _active (b@test.com), not emailAddress (a@test.com).
  let out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let stdout_text = stdout( &out );
  assert!(
    stdout_text.contains( "b@test.com" ),
    "must save as b@test.com (active marker), got:\n{stdout_text}",
  );
  assert!(
    !stdout_text.contains( "a@test.com" ),
    "must NOT save as a@test.com (stale emailAddress), got:\n{stdout_text}",
  );

  // Active marker must still be b@test.com after save (save() writes the marker with the saved name).
  let marker_content = std::fs::read_to_string( &marker ).unwrap_or_default();
  assert_eq!(
    marker_content.trim(), "b@test.com",
    "active marker must remain b@test.com after save, got: {marker_content}",
  );
}

/// # Root Cause
///
/// `account_save_routine()` (BUG-209 fix) read the `_active_{hostname}_{user}` marker
/// as the SOLE name inference source when `name::` is absent. The marker is only written
/// by clp ops (`switch_account`, `save`). External OAuth login writes `~/.claude.json`
/// (including `oauthAccount.emailAddress`) without updating `_active` — leaving the marker
/// stale. BUG-209 fix introduced this regression by swapping one stale source for another.
///
/// # Why Not Caught
///
/// The BUG-209 MRE (`mre_bug_209_*`) pre-populates the `_active` marker with the correct
/// target account before calling `.account.save`. It validates that the marker beats stale
/// top-level `emailAddress` — but does NOT test a stale marker itself. No test simulated
/// the external-login scenario: set marker=A, write live `oauthAccount.emailAddress`=B,
/// assert save targets B not A.
///
/// # Fix Applied
///
/// `account_save_routine()` now reads `oauthAccount.emailAddress` from `~/.claude.json` as
/// the primary source; falls back to `_active` marker only when emailAddress is absent/empty.
/// `oauthAccount.emailAddress` is updated by BOTH `switch_account()` (snapshot restore) AND
/// external OAuth login (Claude writes `~/.claude.json` on every authentication).
///
/// # Prevention
///
/// Add MRE test: write `_active` = stale account; write `~/.claude.json` with live
/// `oauthAccount.emailAddress`; call `.account.save` (no `name::`) — assert save targets
/// the live email, not the stale marker.
///
/// # Pitfall
///
/// Any inference that relies on a single marker written only by one class of credential-change
/// ops fails silently when other classes bypass that marker. Always prefer a source that ALL
/// credential-change paths maintain — `oauthAccount.emailAddress` is the universal source.
#[ doc = "bug_reproducer(BUG-212)" ]
#[ test ]
fn mre_bug_212_account_save_stale_marker_uses_oauth_email()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Credentials required for save to succeed (write_credentials also creates ~/.claude/).
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Fresh oauthAccount.emailAddress from external OAuth login (i5 authenticated via browser).
  // _active marker is NOT updated by external login — only clp ops (.account.use/.account.save) write it.
  std::fs::write(
    dir.path().join( ".claude.json" ),
    r#"{"oauthAccount":{"emailAddress":"i5@wbox.pro"}}"#,
  ).unwrap();

  // Stale _active marker = "i2@wbox.pro" — set by prior .account.use i2; not updated by external login.
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write(
    store.join( claude_profile::account::active_marker_filename() ),
    "i2@wbox.pro",
  ).unwrap();

  // .account.save with no name:: — must use oauthAccount.emailAddress (i5), not _active (i2).
  let out = run_cs_with_env( &[ ".account.save" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let stdout_text = stdout( &out );
  assert!(
    stdout_text.contains( "i5@wbox.pro" ),
    "must save as i5@wbox.pro (oauthAccount.emailAddress), got:\n{stdout_text}",
  );
  assert!(
    !stdout_text.contains( "i2@wbox.pro" ),
    "must NOT save as i2@wbox.pro (stale _active marker), got:\n{stdout_text}",
  );

  // BUG-212: before fix, i2's file is created instead of i5's.
  let i5_file = store.join( "i5@wbox.pro.credentials.json" );
  let i2_file = store.join( "i2@wbox.pro.credentials.json" );
  assert!( i5_file.exists(), "i5@wbox.pro.credentials.json must be created" );
  assert!( !i2_file.exists(), "i2@wbox.pro.credentials.json must NOT be created (stale marker must not win)" );
}

// ── mre_bug213_account_use_refuses_expired_token_on_fetch_error ───────────────

/// MRE for BUG-213: `.account.use` with `touch::1` (default) must refuse to call
/// `switch_account()` when the target account's `expiresAt` is in the past and
/// the quota fetch returns an Err. Before fix: exits 0, installs expired credentials.
/// After fix: exits 3, `~/.claude/.credentials.json` unchanged.
///
/// # Root Cause
///
/// `account_use_routine()` calls `pre_switch_touch_ctx()` which returns `None`
/// when the quota fetch fails (no `accessToken`, HTTP error, etc.). The routine
/// then calls `switch_account()` unconditionally — never consulting `expiresAt`
/// from the target's credential file. Expired credentials are silently installed;
/// subsequent API calls immediately fail with 401, violating the invariant:
/// "after `.account.use X` reports success, X is usable for API calls."
///
/// # Why Not Caught
///
/// FT-04 (`aw23`) tests fetch failure with `expiresAt = FAR_FUTURE_MS` — the
/// non-expired path that silently skips touch and exits 0. AC-04 (pre-fix) said
/// "touch is skipped silently" without distinguishing expired vs valid credentials.
/// No test exercised the expired-`expiresAt` + fetch-Err combination that is the
/// actual BUG-213 failure mode.
///
/// # Fix Applied
///
/// BUG-213: In `account_use_routine()`, after `pre_switch_touch_ctx()` returns `None`:
/// when `touch != 0 && touch_ctx.is_none()`, read `expiresAt` from the target
/// credential file. If `now_ms > expiresAt`, emit a clear error on stderr and
/// call `std::process::exit(3)` without calling `switch_account()`.
///
/// BUG-230: The exit-3 block now first attempts `attempt_expired_token_refresh()`
/// when `refresh::1` (default). In this test, the target has no `accessToken`, so
/// the refresh attempt fails immediately → exit 3 with `"and refresh failed"`. The
/// `err.contains("account credentials expired")` assertion still holds because the
/// new message `"account credentials expired and refresh failed"` contains the substring.
/// For the `refresh::0` (immediate-exit) path, see `aw33_refresh_disabled_exits_3_immediately`.
///
/// # Prevention
///
/// After any probe function returns None due to a fetch error, independently
/// read credential state before proceeding. Verify that `aw23` (fetch fail +
/// `FAR_FUTURE_MS`) still exits 0 — the not-expired path must not be blocked.
/// Use `expiresAt = 1000` (epoch + 1s) for expired fixtures, never `FAR_FUTURE_MS`.
///
/// # Pitfall
///
/// A `None` return from a probe function that also reads credential state conflates
/// "valid-but-fetch-failed" with "expired-and-fetch-failed". Never treat all `None`
/// returns from stateful probe functions identically at the decision point — add
/// an explicit expiry check for each distinct None cause.
#[ doc = "bug_reproducer(BUG-213)" ]
#[ test ]
fn mre_bug213_account_use_refuses_expired_token_on_fetch_error()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Active session credentials — the file switch_account() would overwrite.
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Target account: expiresAt = 1000ms since epoch (~56 years in the past); no accessToken.
  // No accessToken → quota fetch fails immediately (no HTTP call). expiresAt 1000 → expired.
  // BUG-213: before fix, switch_account() is called and active creds are overwritten.
  write_account( dir.path(), "alice@home.com", "max", "default", 1000, false );

  // Capture current main credentials — must be unchanged after exit 3.
  let creds_path   = dir.path().join( ".claude" ).join( ".credentials.json" );
  let creds_before = std::fs::read_to_string( &creds_path ).unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@home.com" ],
    &[ ( "HOME", home ) ],
  );

  // BUG-213: before fix this exits 0 and overwrites creds_path with alice's expired token.
  assert_exit( &out, 3 );
  let err = stderr( &out );
  assert!(
    err.contains( "account credentials expired" ),
    "stderr must contain 'account credentials expired', got:\n{err}",
  );
  assert!(
    err.contains( "alice@home.com" ),
    "stderr must name the account, got:\n{err}",
  );

  // switch_account() must NOT have been called — credentials file unchanged.
  let creds_after = std::fs::read_to_string( &creds_path ).unwrap();
  assert_eq!(
    creds_before,
    creds_after,
    "~/.claude/.credentials.json must be unchanged when exit 3 fires before switch_account()",
  );

  // Active marker must NOT have been updated.
  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let marker = std::fs::read_to_string(
    store.join( claude_profile::account::active_marker_filename() )
  ).unwrap_or_default();
  assert!(
    !marker.contains( "alice@home.com" ),
    "active marker must NOT be updated when exit 3 fires, got: '{marker}'",
  );
}

// ── BUG-230 ───────────────────────────────────────────────────────────────────

/// `.account.use` with an expired token and `refresh::1` (default) attempts refresh;
/// when refresh fails (no `accessToken` in credential file), exits 3 with
/// `"account credentials expired and refresh failed"` on stderr.
///
/// # Root Cause (BUG-230)
///
/// The BUG-213 guard added `exit(3)` on expired token without attempting an OAuth
/// token refresh. Token expiry is recoverable via `refresh_account_token()` —
/// the same mechanism used by `.usage refresh::1`. The guard gave up without trying.
///
/// # Why Not Caught
///
/// BUG-213 tests verified that the switch is refused on expiry but did not distinguish
/// "refuse immediately" from "try refresh then refuse". The `refresh::` parameter did
/// not exist at the time BUG-213 was fixed, so no test covered the refresh-attempt path.
///
/// # Fix Applied
///
/// Added `refresh::` parameter (default 1) to `.account.use`. When `refresh::1` and
/// token is locally expired: calls `attempt_expired_token_refresh()`. If `None` returned
/// (refresh failed), exits 3 with `"account credentials expired and refresh failed: {name}"`.
///
/// # Prevention
///
/// Any "refuse on expired credential" guard must first attempt refresh when `refresh::1`.
/// Use `err.contains("and refresh failed")` to detect the failure-after-attempt path;
/// use `refresh::0` to test the immediate-refusal path.
///
/// # Pitfall
///
/// The `mre_bug213` test (expired + no accessToken) still passes because the new message
/// `"account credentials expired and refresh failed"` contains `"account credentials expired"`.
/// But it now exercises the refresh-attempt path, not the immediate-refusal path.
/// Use `refresh::0` for tests that need the old immediate-exit-3 semantics.
#[ doc = "bug_reproducer(BUG-230)" ]
#[ test ]
fn mre_bug230_account_use_refresh_fails_exits_3_with_updated_message()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Active session — must be unchanged.
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Target: expiresAt = 1000 (expired), no accessToken (refresh will fail immediately).
  write_account( dir.path(), "alice@home.com", "max", "default", 1000, false );

  let creds_path   = dir.path().join( ".claude" ).join( ".credentials.json" );
  let creds_before = std::fs::read_to_string( &creds_path ).unwrap();

  // Default refresh::1 — will attempt refresh, fail (no accessToken), then exit 3.
  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@home.com" ],
    &[ ( "HOME", home ) ],
  );

  assert_exit( &out, 3 );
  let err = stderr( &out );
  assert!(
    err.contains( "account credentials expired and refresh failed" ),
    "BUG-230: stderr must contain 'account credentials expired and refresh failed', got:\n{err}",
  );
  assert!(
    err.contains( "alice@home.com" ),
    "BUG-230: stderr must name the account, got:\n{err}",
  );

  // switch_account() must NOT have been called — credentials unchanged.
  let creds_after = std::fs::read_to_string( &creds_path ).unwrap();
  assert_eq!(
    creds_before,
    creds_after,
    "BUG-230: credentials must be unchanged when exit 3 fires after refresh failure",
  );
}

/// aw33: `refresh::0` on an expired token → exits 3 immediately with old message,
/// no refresh attempt (FT-18).
///
/// Verifies that the immediate-refusal path (BUG-213 semantics) is preserved when
/// `refresh::0` is explicitly set.
#[ test ]
fn aw33_refresh_disabled_exits_3_immediately()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "default", 1000, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@home.com", "refresh::0", "trace::1" ],
    &[ ( "HOME", home ) ],
  );

  assert_exit( &out, 3 );
  let err = stderr( &out );
  // refresh::0 uses the old message (no "and refresh failed").
  assert!(
    err.contains( "account credentials expired: alice@home.com" ),
    "aw33: stderr must contain 'account credentials expired: alice@home.com', got:\n{err}",
  );
  assert!(
    !err.contains( "and refresh failed" ),
    "aw33: refresh::0 must NOT emit 'and refresh failed' (no refresh attempted), got:\n{err}",
  );
  // Trace must show refused (refresh::0) annotation.
  assert!(
    err.contains( "refused (refresh::0)" ),
    "aw33: trace must include 'refused (refresh::0)', got:\n{err}",
  );
}

/// aw34: `refresh::bad` → exit 1; stderr names all valid values (IT-28).
///
/// Validation fires before any filesystem I/O — no accounts needed in the temp dir.
#[ test ]
fn aw34_refresh_bad_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::any@example.com", "refresh::bad" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "refresh::" ) && err.contains( '0' ) && err.contains( '1' ) && err.contains( "false" ) && err.contains( "true" ),
    "aw34: stderr must name all valid refresh:: values (0, 1, false, true); got:\n{err}",
  );
}

// ── aw35 ──────────────────────────────────────────────────────────────────────

/// aw35 (015 FT-10 / AC-10): `.account.use.help` Examples section shows the positional form
/// `clp .account.use alice@acme.com` — without `name::` prefix.
///
/// Feature 015 requires that help text shows the shortcut syntax, not only the explicit form.
/// Prevents doc-drift where the help block lists only `name::EMAIL` examples.
#[ test ]
fn aw35_help_shows_positional_example()
{
  let out  = run_cs( &[ ".account.use.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // The examples section must include a bare email (no "name::" keyword) to demonstrate
  // positional syntax.  We check for a known example from the spec (or any bare-email pattern).
  let has_positional = text
    .lines()
    .any( |l| l.contains( '@' ) && !l.contains( "name::" ) && !l.trim_start().starts_with( "##" ) );
  assert!(
    has_positional,
    ".account.use.help must show a positional example (email without name:: prefix), got:\n{text}",
  );
}

// ── aw36 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn aw36_positional_after_key_value()
{
  // AC-14 (FT-14): reversed arg order `clp .account.use dry::1 alice@home.com` works
  // identically to `clp .account.use alice@home.com dry::1`.
  // Fix(BUG-294): old adapter hardcoded argv[1] check; key::val at argv[1] suppressed
  //   the positional rewrite, leaving bare name at argv[2] to fail the "::" requirement.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@home.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "dry::1", "alice@home.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run] would switch to 'alice@home.com'" ),
    "reversed-order positional must resolve account and switch in dry-run mode, got:\n{text}",
  );
}

// ── BUG-217 ───────────────────────────────────────────────────────────────────

/// `switch_account()` inserts `oauthAccount` verbatim from the per-account snapshot,
/// carrying the stale `emailAddress` field into `~/.claude.json`.
///
/// # Root Cause
///
/// `switch_account()` called `obj.insert("oauthAccount", oauth)` where `oauth` was cloned
/// verbatim from `{name}.json`. When `emailAddress` in the snapshot was stale (from
/// a prior corruption cycle), the wrong email propagated to `~/.claude.json`, causing
/// `account_save_routine()` to infer the wrong account name on subsequent saves.
///
/// # Why Not Caught
///
/// `switch_restores_claude_json` saves via `.account.save` so snapshots are always
/// correct — it never exercised a pre-existing stale snapshot. No test seeded
/// `{name}.json` with a wrong `emailAddress` before switching.
///
/// # Fix Applied
///
/// After extracting `oauth` from the snapshot, `as_object_mut()` is used to overwrite
/// `emailAddress` with `name` before `obj.insert("oauthAccount", oauth)`.
///
/// # Prevention
///
/// Identity fields in per-account snapshots must not be trusted when the account key IS
/// the canonical source. Override before inserting into shared files.
///
/// # Pitfall
///
/// Corruption is self-perpetuating: stale email installed in shared file → read by save
/// as primary name source → saved under wrong account → same stale email re-installed on
/// next switch. Both BUG-217 and BUG-218 must be fixed together.
#[ doc = "bug_reproducer(BUG-217)" ]
#[ test ]
fn mre_bug_217_switch_account_enforces_emailaddress()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Create ~/.claude/.credentials.json so switch_account can write the adjacent temp file.
  // Without this directory, std::fs::copy to .credentials.json.tmp fails with NotFound.
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  // Target account: credentials in the credential store (no accessToken required — touch::0).
  write_account( dir.path(), "i7@wbox.pro", "pro", "standard", FAR_FUTURE_MS, false );

  // Stale snapshot: emailAddress is "i1@wbox.pro" — should be "i7@wbox.pro".
  // BUG-217: switch_account() reads this and installs it verbatim into ~/.claude.json.
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::write(
    store.join( "i7@wbox.pro.json" ),
    r#"{"oauthAccount":{"emailAddress":"i1@wbox.pro","id":"uuid-placeholder"}}"#,
  ).unwrap();

  // Initial ~/.claude.json — switch_account() patches oauthAccount in-place.
  let claude_json_path = dir.path().join( ".claude.json" );
  std::fs::write(
    &claude_json_path,
    r#"{"someGlobalKey":true,"oauthAccount":{"emailAddress":"i9@wbox.pro"}}"#,
  ).unwrap();

  // touch::0 disables pre-fetch HTTP calls and the expiry guard — tests the pure file switch.
  let out = run_cs_with_env(
    &[ ".account.use", "name::i7@wbox.pro", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // After switch: oauthAccount.emailAddress must equal the target account name — not the
  // stale value from the snapshot.
  // BUG-217: before fix, actual = "i1@wbox.pro" (verbatim from snapshot).
  let claude_json = std::fs::read_to_string( &claude_json_path ).unwrap();
  assert!(
    claude_json.contains( r#""emailAddress": "i7@wbox.pro""# ),
    "BUG-217: expected emailAddress='i7@wbox.pro' in ~/.claude.json after switch, got:\n{claude_json}",
  );
  assert!(
    !claude_json.contains( r#""emailAddress": "i1@wbox.pro""# ),
    "BUG-217: stale emailAddress 'i1@wbox.pro' must not appear in ~/.claude.json, got:\n{claude_json}",
  );

  // Global keys must be preserved — switch must not wholesale overwrite ~/.claude.json.
  assert!(
    claude_json.contains( "\"someGlobalKey\": true" ),
    "switch_account must preserve global keys in ~/.claude.json, got:\n{claude_json}",
  );
}


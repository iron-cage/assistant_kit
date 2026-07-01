//! Integration tests: IT-21–IT-52 — `.usage` live mode, streaming, and session window tests.
//!
//! Covers live-mode continuous updates, active-account markers, SIGKILL, session
//! window columns, feature027 `touch::` disable integration, and trace lines.
//!
//! Live tests (names contain `lim_it`) require a real Anthropic OAuth access token.

use crate::cli_runner::{
  BIN,
  run_cs, run_cs_with_env, run_cs_bytes_for_secs,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token, write_claude_json,
  write_live_credentials_with_token, live_active_token,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── it021 ──────────────────────────────────────────────────────────────────────

/// it021 (`lim_it`): `live::1 interval::30 jitter::0` with a real token.
///
/// Runs the live monitor for 10 seconds then kills the process. Within that window
/// the first fetch cycle completes and the countdown footer is written to stdout —
/// the raw byte capture must contain "Next update".
///
/// Requires one saved account with a real token. The process is killed via
/// `Child::kill()` (SIGKILL); SIGINT clean-exit is covered separately (AC-30).
#[ test ]
fn it021_lim_it_live_mode()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it021: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "myaccount", &token, true );

  // Run for 10 s — enough for one stagger (0.2–1.5 s) + network fetch + table render.
  let bytes = run_cs_bytes_for_secs(
    &[ ".usage", "live::1", "interval::30", "jitter::0" ],
    &[ ( "HOME", home ) ],
    10,
  );
  let text = String::from_utf8_lossy( &bytes );
  assert!(
    text.contains( "Next update" ),
    "live mode must emit countdown footer 'Next update ...', got:\n{text}",
  );
}

// ── it022 ──────────────────────────────────────────────────────────────────────

/// it022: `live::1 interval::60 jitter::70` — jitter exceeds interval → exit 1.
///
/// Validation guard fires before any network call; no credentials required.
/// Verifies AC-27: `jitter > interval` is rejected.
#[ test ]
fn it022_live_jitter_exceeds_interval()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::60", "jitter::70" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "jitter > interval must produce error on stderr",
  );
}

// ── it023 ──────────────────────────────────────────────────────────────────────

/// it023: `live::1 interval::5` — interval below minimum → exit 1, message contains "30".
///
/// Validation guard fires before any network call; no credentials required.
/// Verifies AC-26: `interval < 30` is rejected; error message cites the minimum (30).
#[ test ]
fn it023_live_interval_below_minimum()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::5", "jitter::0" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "30" ),
    "interval-too-small error must mention the minimum (30), got:\n{err}",
  );
}

// ── it024 ──────────────────────────────────────────────────────────────────────

/// it024: `live::1 format::json` — JSON format rejected in live mode → exit 1.
///
/// Validation guard fires before any network call; no credentials required.
/// Verifies AC-25: `live::1 format::json` is incompatible.
#[ test ]
fn it024_live_incompatible_with_json()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::1", "format::json" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "live + json must produce error on stderr",
  );
}

// ── it025 ──────────────────────────────────────────────────────────────────────

/// it025: live token unmatched + `~/.claude.json` has `emailAddress` →
/// synthetic row shows the email, NOT the `"(current session)"` fallback.
///
/// Pitfall (AC-09): the synthetic row resolution has TWO paths:
///   • `.claude.json` present with non-empty `emailAddress` → use it (this test)
///   • `.claude.json` absent or empty `emailAddress` → `"(current session)"` (it018)
/// it018 covers the fallback; this test covers the happy path that it018 cannot.
#[ test ]
fn it025_synthetic_row_uses_claude_json_email()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // alice is saved; live creds use a different token → no saved match → synthetic row.
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", false );
  write_live_credentials_with_token( dir.path(), "tok-unsaved" );
  // .claude.json supplies the email for the synthetic row.
  write_claude_json( dir.path(), "unsaved@example.com" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "unsaved@example.com" ),
    "synthetic row must use emailAddress from .claude.json, got:\n{text}",
  );
  assert!(
    !text.contains( "(current session)" ),
    "must NOT fall back to '(current session)' when .claude.json has emailAddress, got:\n{text}",
  );
  let synthetic_current = text.lines().any( |l|
    l.contains( '\u{2713}' ) && l.contains( "unsaved@example.com" )
  );
  assert!( synthetic_current, "synthetic row must carry ✓ flag, got:\n{text}" );
}

// ── it026 ──────────────────────────────────────────────────────────────────────

/// it026: `live::1 interval::30 jitter::30` — jitter EQUAL to interval is accepted.
///
/// The guard is `jitter > interval` (strict greater-than).  Equal values must not
/// trigger the error.  Exit 2 (store unreadable) proves the guards were passed and
/// `execute_live_mode()` was entered before failing on the unreadable store.
/// Exit 1 would indicate a guard fired, which would be a bug.
#[ cfg( unix ) ]
#[ test ]
fn it026_live_jitter_equals_interval_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::30", "jitter::30" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered, store unreadable (guards passed).
  // Exit 1 = a guard fired — that would be a bug (equal is allowed).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "jitter" ),
    "jitter == interval must not trigger the guard, stderr:\n{err}",
  );
}

// ── it027 ──────────────────────────────────────────────────────────────────────

/// it027: `format::json` for an account whose quota fetch fails → JSON has `"error"` field.
///
/// `write_account()` produces a credential file without `accessToken`, so `read_token()`
/// returns `Err("missing accessToken")` → `AccountQuota.result = Err(...)` →
/// `render_json()` emits `{"account":…,"error":"…"}` instead of quota fields.
///
/// Root cause of gap: it004 and it016 verify JSON structure for successful fetches;
/// neither explicitly asserts the `error` key is present on a failed account.
#[ test ]
fn it027_json_error_field_on_failed_account()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No accessToken → read_token() fails → result is Err.
  write_account( dir.path(), "no-token@acme.com", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let json = stdout( &out );

  assert!(
    json.contains( "\"error\":" ),
    "failed account must produce JSON with 'error' key, got:\n{json}",
  );
  assert!(
    !json.contains( "session_5h_left_pct" ),
    "failed account must NOT have quota fields, got:\n{json}",
  );
  // Mandatory base fields must still be present.
  assert!( json.contains( "\"is_current\""     ), "must have is_current, got:\n{json}" );
  assert!( json.contains( "\"is_active\""      ), "must have is_active, got:\n{json}" );
  assert!( json.contains( "\"expires_in_secs\"" ), "must have expires_in_secs, got:\n{json}" );
}

// ── it028 ──────────────────────────────────────────────────────────────────────

/// it028: `interval::5 jitter::70` without `live::1` → no guard fires, exit 0.
///
/// Live-mode guards (interval minimum, jitter ceiling) only activate when
/// `live == 1`.  Specifying invalid interval/jitter in non-live mode must be
/// silently ignored — the params are undefined outside live mode.
#[ test ]
fn it028_interval_jitter_ignored_when_not_live()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  // interval::5 would fail the live-mode guard if live::1 were set.
  // jitter::70 > interval::5 would also fail. Neither should fire here.
  let out = run_cs_with_env(
    &[ ".usage", "interval::5", "jitter::70" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ),
    "non-live mode must ignore interval/jitter and show no-accounts message, got:\n{text}",
  );
}

// ── it030 ──────────────────────────────────────────────────────────────────────

/// it030: `live::1` with a no-token account — SIGINT after 3s → exit 0, "Monitor stopped." in stdout.
///
/// Verifies AC-30: Ctrl-C (SIGINT) causes a clean exit (code 0) without error output.
/// Uses an account with no `accessToken` so the per-account fetch fails instantly (no HTTP call),
/// the binary renders the error table, starts the countdown, then receives SIGINT.
/// `kill -INT` is used as a subprocess to avoid a `libc` dev-dependency.
#[ cfg( unix ) ]
#[ test ]
fn it030_live_sigint_exits_0()
{
  use std::process::Stdio;

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // No accessToken → read_token() fails instantly (no HTTP call); render error row; countdown starts.
  write_account( dir.path(), "myaccount", "max", "default", FAR_FUTURE_MS, true );

  let child = std::process::Command::new( BIN )
    .args( [ ".usage", "live::1", "interval::30", "jitter::0" ] )
    .env( "HOME", home )
    .env_remove( "PRO" )
    .stdout( Stdio::piped() )
    .stderr( Stdio::piped() )
    .spawn()
    .expect( "failed to spawn clp binary" );

  // Wait for the cycle to complete: stagger (200–1500 ms) + instant fail + render + countdown start.
  std::thread::sleep( core::time::Duration::from_secs( 3 ) );

  // Send SIGINT via the system `kill` utility — no libc dep needed.
  let _ = std::process::Command::new( "kill" )
    .args( [ "-INT", &child.id().to_string() ] )
    .status();

  let out = child.wait_with_output().expect( "failed to wait on clp binary" );
  let text = String::from_utf8_lossy( &out.stdout );

  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "SIGINT must cause clean exit 0, got: {:?}\nstdout: {text}\nstderr: {}",
    out.status,
    String::from_utf8_lossy( &out.stderr ),
  );
  assert!(
    text.contains( "Monitor stopped." ),
    "clean SIGINT exit must print 'Monitor stopped.', got:\n{text}",
  );
}

// ── it029 ──────────────────────────────────────────────────────────────────────

/// it029: `live::1` alone — default `interval=30` satisfies the `>= 30` guard.
///
/// When neither `interval::` nor `jitter::` are specified, the binary applies
/// defaults: `interval=30`, `jitter=0`.  `30 < 30` is false so the interval
/// guard does not fire.  Exit 2 (unreadable store) proves `execute_live_mode()`
/// was entered; exit 1 would mean a guard incorrectly fired.
#[ cfg( unix ) ]
#[ test ]
fn it029_live_default_interval_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = guards passed with default interval; exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "interval" ),
    "default interval (30) must not trigger the interval guard, stderr:\n{err}",
  );
}

// ── it031 ──────────────────────────────────────────────────────────────────────

/// it031: `.usage.help` lists `live`, `interval`, and `jitter` params.
///
/// Verifies AC-32: all three live-monitor params must appear in the per-command
/// help output so users can discover them without reading source code.
/// The params are registered via `register_commands()` in `src/lib.rs`; this
/// test confirms the registration produces visible output in `.usage.help`.
#[ test ]
fn it031_usage_help_shows_live_params()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  for param in &[ "live", "interval", "jitter" ]
  {
    assert!(
      text.contains( param ),
      ".usage.help must list param `{param}` (AC-32), got:\n{text}",
    );
  }
}

// ── it033 ──────────────────────────────────────────────────────────────────────

/// it033: `.usage.help` refresh description mentions 401/403 but NOT 429.
///
/// # Root Cause
/// Task 150 removed 429 from the `apply_refresh` retry guard, but the parameter
/// description in `lib.rs register_commands()` was not updated — it still said
/// "401/403/429". Users reading `--help` would believe 429 triggers a refresh.
///
/// # Why Not Caught
/// Existing help test (it031) only checked for `live`, `interval`, `jitter` params.
/// No test verified the refresh description text excluded 429.
///
/// # Fix Applied
/// Changed description from "401/403/429" to "401/403" in `lib.rs:167`.
///
/// # Prevention
/// This test asserts `help` output contains "401/403" but NOT "401/403/429".
///
/// # Pitfall
/// The assertion relies on the exact substring "401/403/429" — a reformulated
/// description that mentions 429 in different phrasing would not be caught.
#[ doc = "bug_reproducer(BUG-279)" ]
#[ test ]
fn it033_mre_refresh_help_excludes_429()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "401/403" ),
    "refresh description must mention 401/403, got:\n{text}",
  );
  assert!(
    !text.contains( "401/403/429" ),
    "refresh description must NOT mention 429 (task 150 removed it), got:\n{text}",
  );
}

// ── it032 ──────────────────────────────────────────────────────────────────────

/// it032 (`lim_it`): `refresh::1` with a real saved account — exercises the
/// per-account refresh loop (AC-19) and verifies no panic + exit 0.
///
/// The per-account loop reads `{credential_store}/{name}.credentials.json`
/// (not the live session file). When the account's quota fetch succeeds on the
/// first pass, `should_retry` is false and the loop is a no-op — the test
/// proves no regression in the happy path. When credentials are stale/expired,
/// the loop runs `run_isolated` and updates `aq.result`.
///
/// Requires one saved account with a live token reachable via `live_active_token()`.
#[ test ]
fn it032_lim_it_refresh_per_account()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it032: no live token — skipping" );
    return;
  };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "test-acct", &token, true );
  write_live_credentials_with_token( dir.path(), &token );

  let out = run_cs_with_env( &[ ".usage", "refresh::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "test-acct" ),
    "account must appear in output with refresh::1 (AC-19), got:\n{text}",
  );
}

// ── it034 ──────────────────────────────────────────────────────────────────────

/// it034: `trace::1` with a no-token account → stderr contains timestamped diagnostic lines.
///
/// `trace::1` causes `fetch_all_quota` to emit timestamped diagnostic lines per account to
/// stderr — one before reading credentials and one after. With a credential file
/// that has no `accessToken`, `read_token()` returns Err → trace emits
/// "cannot read token: missing accessToken". This test confirms the `trace`
/// parameter is accepted, wired through to `fetch_all_quota`, and produces
/// observable stderr output without affecting exit code or stdout.
#[ test ]
fn it034_trace_param_writes_to_stderr()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "trace-acct", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( " · " ),
    "trace::1 must write trace lines to stderr, got:\n{err}",
  );
  assert!(
    err.contains( "trace-acct" ),
    "trace::1 must mention the account name, got:\n{err}",
  );
}

// ── it035 ──────────────────────────────────────────────────────────────────────

/// it035: empty credential store + `format::json` → output is `[]`.
///
/// `render_json(&[])` returns `"[]\n"` via the short-circuit branch. This verifies
/// that `format::json` and the empty-store path are compatible — no crash, no
/// "no accounts configured" text (that message is text-format-only).
#[ test ]
fn it035_empty_store_json_format()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path()
    .join( ".persistent" )
    .join( "claude" )
    .join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out  = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text.trim(),
    "[]",
    "empty store with format::json must output '[]', got:\n{text}",
  );
}

// ── it037 ──────────────────────────────────────────────────────────────────────

/// it037: `.usage.help` shows `refresh::` default as `1` (enabled), not `0`.
///
/// ## Root Cause
/// `usage_routine()` in `src/usage.rs` matched `refresh` with fallback `_ => 0`,
/// making `refresh` default to disabled. Every `clp .usage` call without `refresh::`
/// skipped `apply_refresh()`, showing stale `(auth expired (401))` rows instead
/// of refreshing the token and retrying. Both the runtime default and the help-text
/// description were wrong — `lib.rs` said "(0 = disabled; 1 = enabled)" with no
/// indication which is default; `unilang.commands.yaml` carried `default: "0"`.
///
/// ## Why Not Caught
/// Existing tests (it019/it020) checked that both `refresh::0` and `refresh::1` are
/// accepted. Neither verified that the DEFAULT (no arg) was 1. The help text test
/// (it033) only checked the 429 exclusion, not the default value annotation.
///
/// ## Fix Applied
/// `usage_routine()` fallback changed from `_ => 0` to `_ => 1`. Description in
/// `lib.rs:167` updated to "(1 = enabled, default; 0 = disabled)". `unilang.commands.yaml`
/// default updated to `"1"`. All feature/CLI docs and IT specs updated to reflect
/// the new default.
///
/// ## Prevention
/// This test asserts `.usage.help` output contains `"1 = enabled, default"` — the
/// exact phrase added to the description — and does NOT contain `"0 = disabled, default"`.
///
/// ## Pitfall
/// Any future edit to the description string in `lib.rs` that removes `"1 = enabled, default"`
/// (e.g., reformulation keeping 429 but changing default wording) would break this test.
#[ doc = "bug_reproducer(BUG-155)" ]
#[ test ]
fn it037_mre_bug155_refresh_defaults_to_1()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "1 = enabled, default" ),
    "refresh help must indicate 1 is the default (BUG-155), got:\n{text}",
  );
  // The `live` description legitimately contains "0 = off, default"; only check that
  // the refresh-specific "(0 = disabled, default)" phrasing is absent.
  assert!(
    !text.contains( "0 = disabled, default" ),
    "refresh help must NOT say '0 = disabled, default' (BUG-155), got:\n{text}",
  );
}

// ── it038 ──────────────────────────────────────────────────────────────────────

/// it038: `.usage.help` refresh description mentions 429 with locally-expired token.
///
/// ## Root Cause
/// `apply_refresh()` unconditionally excluded 429 from its retry guard. Accounts
/// returning 429 with a locally-expired `expiresAt` (stale per-account credentials
/// file) were never refreshed — the `Expires` column showed `EXPIRED` and the
/// 429 was displayed with no refresh attempt made. The guard now conditionally
/// includes 429 when `expires_at_ms / 1000 ≤ now_secs`.
///
/// ## Why Not Caught
/// Existing tests (it033, it019/it020) checked 401/403 refresh and the absence of
/// "401/403/429" as a combined string. None verified the 429+locally-expired case.
///
/// ## Fix Applied
/// `should_refresh()` extracted as a private helper; extended to return `true` for
/// 429 when `expires_at_ms / 1000 <= now_secs`. Description in `lib.rs:167` and
/// `unilang.commands.yaml` updated to document the conditional 429 case.
/// `apply_refresh()` propagates retry errors to `aq.result` (was: silent discard).
/// `aq.expires_at_ms` updated from credentials file after successful write (was: stale).
///
/// ## Prevention
/// This test asserts `.usage.help` contains "429", confirming the description was
/// updated — the code and docs are consistent with the new 429+expired behavior.
///
/// ## Pitfall
/// it033 still guards against the old "401/403/429" combined string. This test
/// adds the positive check: "429" appears separately for the conditional case.
#[ doc = "bug_reproducer(BUG-156)" ]
#[ test ]
fn it038_mre_bug156_refresh_help_mentions_429_expired()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "429" ),
    "refresh help must mention 429 case (BUG-156), got:\n{text}",
  );
  // Ensure 429 appears in the conditional context, not as the old "401/403/429" pattern.
  assert!(
    !text.contains( "401/403/429" ),
    "refresh help must NOT say '401/403/429' (old incorrect format), got:\n{text}",
  );
}

// ── it036 ──────────────────────────────────────────────────────────────────────

/// it036: single no-token account → no "Valid:" footer (`valid_count` = 0 < 2).
///
/// The footer line "Valid: X / Y   →  Next: ..." is only emitted when
/// `valid_count >= 2` AND a recommendation exists. With one account whose quota
/// fetch fails (no `accessToken`), `valid_count = 0` → the footer is suppressed.
/// This guards against a regression where footer threshold checking is removed.
#[ test ]
fn it036_no_footer_when_no_valid_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-quota@test.com", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Valid:" ),
    "single failed account must NOT show 'Valid:' footer line, got:\n{text}",
  );
}

// ── it039 ──────────────────────────────────────────────────────────────────────

/// it039 (EC-3): `refresh::2` is out of range for the boolean
/// parameter (only 0 and 1 are valid) → exit 1 with error on stderr.
///
/// Source: `tests/docs/cli/param/19_refresh.md § EC-3`.
#[ test ]
fn it039_refresh_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "refresh::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "refresh::2 must produce error on stderr",
  );
}

// ── it040 ──────────────────────────────────────────────────────────────────────

/// it040 (EC-4): `refresh::yes` is a type mismatch — the param
/// is a boolean integer, not a string → exit 1.
///
/// Source: `tests/docs/cli/param/19_refresh.md § EC-4`.
#[ test ]
fn it040_refresh_yes_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "refresh::yes" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "refresh::yes must produce error on stderr",
  );
}

// ── it041 ──────────────────────────────────────────────────────────────────────

/// it041 (EC-2): `live::0` explicit — single fetch exits 0; no
/// countdown footer emitted.
///
/// `live::0` disables live-monitor mode.  The command performs one fetch cycle
/// (here: empty store → "no accounts") and exits immediately without entering
/// the continuous loop.  The countdown footer ("Next update …") must not appear.
/// Source: `tests/docs/cli/param/20_live.md § EC-2`.
#[ test ]
fn it041_live_0_single_fetch_exits_0()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".usage", "live::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Next update" ),
    "live::0 must not emit countdown footer, got:\n{text}",
  );
}

// ── it042 ──────────────────────────────────────────────────────────────────────

/// it042 (EC-4): `live::2` is out of range for the boolean parameter
/// (only 0 and 1 are valid) → exit 1.
///
/// Source: `tests/docs/cli/param/20_live.md § EC-4`.
#[ test ]
fn it042_live_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "live::2 must produce error on stderr",
  );
}

// ── it043 ──────────────────────────────────────────────────────────────────────

/// it043 (EC-5): `live::yes` is a type mismatch → exit 1.
///
/// Source: `tests/docs/cli/param/20_live.md § EC-5`.
#[ test ]
fn it043_live_yes_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "live::yes" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "live::yes must produce error on stderr",
  );
}

// ── it044 ──────────────────────────────────────────────────────────────────────

/// it044 (EC-6): `interval::abc` is a type error — the param is
/// `u64`, not a string → exit 1 before any credential or live-mode processing.
///
/// Type validation fires at argument parse time; the `live::` mode flag does not
/// affect it (contrast EC-5 where a valid-type but out-of-range value is accepted
/// in non-live mode).
/// Source: `tests/docs/cli/param/21_interval.md § EC-6`.
#[ test ]
fn it044_interval_abc_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "interval::abc" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "interval::abc must produce error on stderr",
  );
}

// ── it045 ──────────────────────────────────────────────────────────────────────

/// it045 (EC-3): `live::1 interval::60` — non-default value
/// accepted; the interval guard (≥ 30) passes for 60 → live mode is entered.
///
/// A chmod-000 credential store forces exit 2 after the guards pass, proving
/// live mode was entered.  Exit 1 would indicate a guard incorrectly fired.
/// Source: `tests/docs/cli/param/21_interval.md § EC-3`.
#[ cfg( unix ) ]
#[ test ]
fn it045_interval_60_live_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::60" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered (interval guard passed); exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "interval" ),
    "interval::60 must not trigger the interval guard, stderr:\n{err}",
  );
}

// ── it046 ──────────────────────────────────────────────────────────────────────

/// it046 (EC-1): `live::1 jitter::0` — explicit zero jitter accepted;
/// the jitter guard (jitter ≤ interval) passes for 0 ≤ 30 → live mode is entered.
///
/// Uses a chmod-000 store for offline verification.  Distinct from `it029` which
/// uses the implicit default (no `jitter::` param) — this test exercises the
/// explicit `jitter::0` path.
/// Source: `tests/docs/cli/param/22_jitter.md § EC-1`.
#[ cfg( unix ) ]
#[ test ]
fn it046_jitter_0_explicit_live_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "jitter::0" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered; exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "jitter" ),
    "explicit jitter::0 must not trigger the jitter guard, stderr:\n{err}",
  );
}

// ── it047 ──────────────────────────────────────────────────────────────────────

/// it047 (EC-2): `live::1 interval::30 jitter::10` — jitter less
/// than interval is accepted; the guard (jitter ≤ interval) passes → live mode
/// is entered.
///
/// Uses a chmod-000 store for offline verification.
/// Source: `tests/docs/cli/param/22_jitter.md § EC-2`.
#[ cfg( unix ) ]
#[ test ]
fn it047_jitter_10_live_accepted()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "live::1", "interval::30", "jitter::10" ],
    &[ ( "HOME", home ) ],
  );

  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  // Exit 2 = live mode entered (jitter::10 ≤ interval::30); exit 1 = guard fired (bug).
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    !err.contains( "jitter" ),
    "jitter::10 with interval::30 must not trigger the jitter guard, stderr:\n{err}",
  );
}

// ── it048 ──────────────────────────────────────────────────────────────────────

/// it048 (EC-7): `jitter::abc` is a type error — the param is `u64`,
/// not a string → exit 1.
///
/// Source: `tests/docs/cli/param/22_jitter.md § EC-7`.
#[ test ]
fn it048_jitter_abc_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "jitter::abc" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "jitter::abc must produce error on stderr",
  );
}

// ── it049 ──────────────────────────────────────────────────────────────────────

/// it049 (EC-2): `trace::0` explicit disable — no timestamped trace lines
/// appear on stderr; exit 0.
///
/// Uses a no-token account so the fetch path is exercised (increasing the chance
/// of accidental trace leakage if the disable is broken).
/// Source: `tests/docs/cli/param/23_trace.md § EC-2`.
#[ test ]
fn it049_trace_0_no_trace_on_stderr()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "trace-off-acct", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "trace::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( " · " ),
    "trace::0 must not emit trace lines on stderr, got:\n{err}",
  );
}

// ── it050 ──────────────────────────────────────────────────────────────────────

/// it050 (EC-3): `trace::2` is out of range for the boolean parameter
/// (only 0 and 1 are valid) → exit 1.
///
/// Source: `tests/docs/cli/param/23_trace.md § EC-3`.
#[ test ]
fn it050_trace_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "trace::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "trace::2 must produce error on stderr",
  );
}

// ── it051 ──────────────────────────────────────────────────────────────────────

/// it051 (EC-4): `trace::yes` is a type mismatch → exit 1.
///
/// Source: `tests/docs/cli/param/23_trace.md § EC-4`.
#[ test ]
fn it051_trace_yes_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "trace::yes" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    !stderr( &out ).is_empty(),
    "trace::yes must produce error on stderr",
  );
}

// ── it052 ──────────────────────────────────────────────────────────────────────

/// it052 (EC-5): default behavior (no `trace::` param) — no timestamped
/// trace lines appear on stderr; trace is off by default (default = 0).
///
/// Uses a no-token account to exercise the fetch path; absence of timestamped trace lines
/// confirms the default is correctly set to disabled.
/// Source: `tests/docs/cli/param/23_trace.md § EC-5`.
#[ test ]
fn it052_trace_default_off()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-trace-acct", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    !err.contains( " · " ),
    "default (no trace:: param) must not emit trace lines on stderr, got:\n{err}",
  );
}


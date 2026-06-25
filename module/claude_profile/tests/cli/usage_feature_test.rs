//! Feature tests: FT (Usage — AC coverage for `009_token_usage` and
//! `018_live_monitor`).
//!
//! Each FT case maps to one acceptance criterion from the corresponding feature
//! doc.  Command-level tests (IT-N) live in `tests/cli/usage_test.rs`.
//!
//! Live tests (names contain `lim_it`) require network access and are excluded
//! from Docker CI by the nextest filter `!test(lim_it)`.
//!
//! ## Test Matrix
//!
//! ### Feature 009 — All-Accounts Live Quota Reporting
//!
//! | ID   | Test Function                                    | AC    | Live? |
//! |------|--------------------------------------------------|-------|-------|
//! | ft01   | `ft01_missing_access_token_short_error`          | AC-03          | no  |
//! | ft02   | `ft02_lim_it_http_401_shortens_to_auth_expired`  | AC-03          | yes |
//! | ft03   | `ft03_both_accounts_appear_regardless_of_active` | AC-01          | no  |
//! | ft04   | `ft04_check_mark_follows_live_token_not_active`  | AC-02          | no  |
//! | ft05   | `ft05_unreadable_credential_store_exits_2`       | AC-06          | no  |
//! | mre162 | `mre_bug_162_jwt_exp_ms`                          | AC-25 / BUG-162 | no |
//!
//! ### Feature 018 — Live Quota Monitor Mode
//!
//! | ID    | Test Function                                   | AC    | Live? |
//! |-------|-------------------------------------------------|-------|-------|
//! | f18ft01 | `f18_ft01_live_0_single_fetch`                | AC-24 | no    |
//! | f18ft06 | `f18_ft06_live_stagger_per_account_trace`     | AC-29 | no    |
//!
//! ### Feature 037 — Accounts/Usage Param Unification
//!
//! | ID      | Test Function                         | AC    | Live? |
//! |---------|---------------------------------------|-------|-------|
//! | f37ft02 | `f37_ft02_usage_accepts_32_params`    | AC-02 | no    |
//! | f37ft04 | `f37_ft04_usage_default_profile`      | AC-04 | no    |
//! | f37ft16 | `f37_ft16_usage_unclaim_mirrors_accounts` | AC-16 | no | <!-- tests owner::0 (Feature 064 successor) -->
//! | f37ft17 | `f37_ft17_usage_assign_mirrors_accounts`  | AC-17 | no | <!-- tests assignee:: (Feature 065 successor) -->
//!
//! ### Feature 063 — Explicit Ownership Claim on `.usage`
//!
//! | ID      | Test Function                             | AC    | Live? |
//! |---------|-------------------------------------------|-------|-------|
//! | f63ft12 | `f63_ft12_usage_owner_mirrors_accounts`   | AC-12 | no    |
//!
//! ### Feature 064 — Active Marker and Owner Param Redesign (`.usage` `REMOVED_TOGGLE` parity)
//!
//! | ID      | Test Function                             | AC    | Live? |
//! |---------|-------------------------------------------|-------|-------|
//! | f64ft05 | `f64_ft05_usage_assign_removed_toggle`    | AC-05 | no    |
//! | f64ft07 | `f64_ft07_usage_unclaim_removed_toggle`   | AC-07 | no    |
//! | f64ft06b| `f64_ft06b_usage_for_removed_toggle`      | AC-06 | no    |
//!
//! ### Feature 065 — Assignee Param Redesign (`.usage` parity)
//!
//! | ID       | Test Function                                    | AC    | Live? |
//! |----------|--------------------------------------------------|-------|-------|
//! | f65ft02  | `f65_ft02_usage_assignee_zero_sentinel_assign`   | AC-02 | no    |
//! | f65ft03  | `f65_ft03_usage_assignee_unassign`               | AC-03 | no    |
//! | f65ft04  | `f65_ft04_usage_assignee_zero_sentinel_unassign` | AC-04 | no    |
//! | f65ft05  | `f65_ft05_usage_assignee_dry_run_assign`         | AC-05 | no    |
//! | f65ft10  | `f65_ft10_usage_active_removed_toggle`           | AC-10 | no    |
//! | f65ec5   | `f65_ec5_usage_assignee_badvalue`                | EC-09 | no    |

use crate::cli_runner::{
  BIN,
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token,
  write_account_owner,
  write_live_credentials_with_token, require_live_api,
  FAR_FUTURE_MS, PAST_MS,
};
use claude_profile::output::jwt_exp_ms;
use tempfile::TempDir;

// ── FT-01: Error reason shortened — missing accessToken (AC-03) ──────────────

/// FT-01 (AC-03): account whose credential file has no `accessToken` field →
/// the account row appears in the table; the error reason in the last column
/// does NOT begin with the verbose `HTTP transport error:` prefix. Exit 0.
///
/// Source: `tests/docs/feature/09_token_usage.md § FT-01`.
#[ test ]
fn ft01_missing_access_token_short_error()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "noaccess@test.com", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "noaccess@test.com" ),
    "account row must appear in the table, got:\n{text}",
  );
  assert!(
    !text.contains( "HTTP transport error:" ),
    "error must be shortened — must NOT contain verbose 'HTTP transport error:', got:\n{text}",
  );
}

// ── FT-02: HTTP 401 shortens to (auth expired (401)) (AC-03) ─────────────────

/// Write a saved account credential with `PAST_MS` expiry AND an `accessToken`
/// so `read_token()` succeeds but the usage API rejects the token with 401.
fn write_account_with_expired_token( home : &std::path::Path, name : &str, token : &str )
{
  let store = home.join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  let json = format!(
    r#"{{"oauthAccount":{{"subscriptionType":"max","rateLimitTier":"default_claude_max_20x"}},"expiresAt":{PAST_MS},"accessToken":"{token}"}}"#,
  );
  std::fs::write( store.join( format!( "{name}.credentials.json" ) ), json ).unwrap();
}

/// FT-02 (AC-03, `lim_it`): saved account has `PAST_MS` `expiresAt` and a token
/// the usage API rejects with HTTP 401 → rendered row shows `EXPIRED` in the
/// Expires column and `auth expired (401)` in the 7d Reset column, NOT the
/// verbose `HTTP transport error: HTTP 401` string. Exit 0.
///
/// Requires network access — the fake token triggers a real API 401 response.
/// Source: `tests/docs/feature/09_token_usage.md § FT-02`.
#[ test ]
fn ft02_lim_it_http_401_shortens_to_auth_expired()
{
  if !require_live_api( "ft02" ) { return; }
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_expired_token( dir.path(), "expired@test.com", "invalid-token-for-401-test" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "expired@test.com" ),
    "account row must appear in the table, got:\n{text}",
  );
  assert!(
    text.contains( "EXPIRED" ),
    "account with PAST_MS expiresAt must show EXPIRED in Expires column, got:\n{text}",
  );
  assert!(
    text.contains( "auth expired (401)" ),
    "HTTP 401 must shorten to 'auth expired (401)', got:\n{text}",
  );
  assert!(
    !text.contains( "HTTP transport error:" ),
    "verbose HTTP error string must NOT appear in output, got:\n{text}",
  );
}

// ── FT-03: All saved accounts fetched, not only _active (AC-01) ──────────────

/// FT-03 (AC-01): two saved accounts with neither stored as `_active` → both
/// names appear in stdout. Exit 0.
///
/// Source: `tests/docs/feature/09_token_usage.md § FT-03`.
#[ test ]
fn ft03_both_accounts_appear_regardless_of_active()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@a.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@a.com",   "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "alice@a.com" ),
    "alice@a.com must appear in output regardless of _active marker, got:\n{text}",
  );
  assert!(
    text.contains( "bob@a.com" ),
    "bob@a.com must appear in output regardless of _active marker, got:\n{text}",
  );
}

// ── FT-04: ✓ follows live token match, not _active marker (AC-02) ────────────

/// FT-04 (AC-02): `alice@a.com` is stored as `_active`; the live
/// `~/.claude/.credentials.json` has an `accessToken` matching `work@a.com`'s
/// saved token → a line in stdout contains `✓` and `work@a.com`; no line
/// contains `✓` and `alice@a.com`. Exit 0.
///
/// Source: `tests/docs/feature/09_token_usage.md § FT-04`.
#[ test ]
fn ft04_check_mark_follows_live_token_not_active()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@a.com", "tok-alice", true  );
  write_account_with_token( dir.path(), "work@a.com",  "tok-work",  false );
  write_live_credentials_with_token( dir.path(), "tok-work" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let work_has_check = text.lines().any( |l| l.contains( '\u{2713}' ) && l.contains( "work@a.com" ) );
  assert!(
    work_has_check,
    "work@a.com must have ✓ (live token match), got:\n{text}",
  );
  let alice_has_check = text.lines().any( |l| l.contains( '\u{2713}' ) && l.contains( "alice@a.com" ) );
  assert!(
    !alice_has_check,
    "alice@a.com must NOT have ✓ (only _active, not live token match), got:\n{text}",
  );
}

// ── FT-05: Unreadable credential store exits 2 (AC-06) ───────────────────────

/// FT-05 (AC-06): credential store directory chmod 000 → `account::list()` fails
/// → `.usage` exits 2 with a non-empty error on stderr.
///
/// Source: `tests/docs/feature/09_token_usage.md § FT-05`.
#[ cfg( unix ) ]
#[ test ]
fn ft05_unreadable_credential_store_exits_2()
{
  use std::os::unix::fs::PermissionsExt;

  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path()
    .join( ".persistent" )
    .join( "claude" )
    .join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o000 ) ).unwrap();

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );

  // Restore before any assertion so TempDir cleanup can delete the directory.
  std::fs::set_permissions( &store, std::fs::Permissions::from_mode( 0o755 ) ).unwrap();

  assert_exit( &out, 2 );
  assert!( !stderr( &out ).is_empty(), "unreadable store must produce error on stderr" );
}

// ── MRE-162: jwt_exp_ms extracts future exp from JWT, not stale expiresAt (AC-25 / BUG-162) ─

/// MRE-162 (AC-25, BUG-162): `jwt_exp_ms` must read the JWT `exp` claim from `accessToken`
/// and return it in milliseconds, NOT the stale `expiresAt` field left unchanged by the
/// isolated subprocess.
///
/// Root Cause: `apply_refresh` called `parse_u64_field(&creds_path, "expiresAt")` after
///   writing refreshed credentials; but the subprocess never updates `expiresAt` — that
///   field is a server-issued JWT claim not emitted during the OAuth refresh exchange.
/// Why Not Caught: Fix(BUG-156) assumed the subprocess writes `expiresAt`; that assumption
///   was never tested with a credentials fixture where `expiresAt` and `exp` differ.
/// Fix Applied: `apply_refresh` now calls `jwt_exp_ms(&new_creds)` to extract `exp * 1000`
///   from the refreshed `accessToken`, which always reflects the new token's true expiry.
/// Prevention: `_mre_` test locks the correct source (JWT `exp`) vs the wrong source
///   (`expiresAt` file field); any regression that re-introduces the file read will fail here.
/// Pitfall: `expiresAt` in the credentials file is a server-issued claim set at token
///   issuance; the OAuth refresh exchange does not update it — only `accessToken` and
///   `refreshToken` are refreshed. Never use `expiresAt` for post-refresh expiry.
///
/// Source: `tests/docs/feature/17_token_refresh.md § AC-25`, `bug/162_expiresAt_not_updated_by_subprocess.md`.
#[ doc = "bug_reproducer(BUG-162)" ]
#[ test ]
fn mre_bug_162_jwt_exp_ms()
{
  // Construct fake credentials JSON with:
  //   - expired expiresAt (demonstrates what the stale read would return)
  //   - accessToken JWT with a future exp claim (demonstrates the correct source)
  //
  // JWT payload: {"exp":2000000000}  (year 2033, unambiguously future)
  // base64url({"exp":2000000000}) = "eyJleHAiOjIwMDAwMDAwMDB9"
  // Full fake token: "eyJhbGciOiJub25lIn0.eyJleHAiOjIwMDAwMDAwMDB9.fakesig"
  let creds = r#"{"claudeAiOauth":{"accessToken":"eyJhbGciOiJub25lIn0.eyJleHAiOjIwMDAwMDAwMDB9.fakesig","expiresAt":1000000000000}}"#;
  // Stale approach (what old code did): reads expiresAt = 1_000_000_000_000 ms (year 2001, EXPIRED)
  // Correct approach: jwt_exp_ms extracts exp = 2_000_000_000 → 2_000_000_000_000 ms (year 2033)
  assert_eq!( jwt_exp_ms( creds ), Some( 2_000_000_000_000 ) );
}

// ── f18-FT-01: live::0 default — single fetch, exits; no loop overhead ────────

/// f18-FT-01 (AC-24, `018_live_monitor`): `live::0` performs one fetch cycle
/// then exits; no countdown footer, no screen clear, no loop.
///
/// Uses a no-token account so the fetch fails instantly (no HTTP call).
/// Verifies single-exit behavior: the account row is rendered, the command
/// exits 0, and the countdown footer ("Next update …") does not appear.
///
/// Source: `tests/docs/feature/18_live_monitor.md § FT-01`.
#[ test ]
fn f18_ft01_live_0_single_fetch()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "no-token@test.com", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "live::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no-token@test.com" ),
    "live::0 must render the account row on single fetch, got:\n{text}",
  );
  assert!(
    !text.contains( "Next update" ),
    "live::0 must not emit countdown footer, got:\n{text}",
  );
}

// ── f18-FT-06: per-account stagger delay present in live mode ─────────────────

/// f18-FT-06 (AC-29): `live::1 trace::1` with 2 no-token accounts — per-account
/// stagger delay of 200–1500 ms fires before each credential read, confirmed by
/// ≥ 2 `[trace] … reading` lines on stderr after a SIGINT-terminated run.
///
/// Stagger fires before `read_token()` in `fetch_all_quota` (`stagger=true` only
/// in live mode). No live token required — the credential JSON files have no
/// `accessToken` field, so `read_token()` fails instantly after the sleep.
///
/// Source: `tests/docs/feature/018_live_monitor.md § FT-06`.
#[ cfg( unix ) ]
#[ test ]
fn f18_ft06_live_stagger_per_account_trace()
{
  use std::process::Stdio;

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Two no-token accounts: stagger fires (200–1500 ms) then read_token() fails instantly.
  write_account( dir.path(), "alpha@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "beta@test.com",  "max", "default", FAR_FUTURE_MS, false );

  let child = std::process::Command::new( BIN )
    .args( [ ".usage", "live::1", "trace::1", "interval::30", "jitter::0" ] )
    .env( "HOME", home )
    .env_remove( "PRO" )
    .stdout( Stdio::piped() )
    .stderr( Stdio::piped() )
    .spawn()
    .expect( "failed to spawn clp binary" );

  // Allow both stagger delays to elapse: 2 × max 1500 ms + render overhead → 5 s.
  std::thread::sleep( core::time::Duration::from_secs( 5 ) );

  let _ = std::process::Command::new( "kill" )
    .args( [ "-INT", &child.id().to_string() ] )
    .status();

  let out = child.wait_with_output().expect( "failed to wait on clp binary" );
  let err = String::from_utf8_lossy( &out.stderr );

  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "SIGINT must cause clean exit 0, got: {:?}\nstdout: {}\nstderr: {err}",
    out.status,
    String::from_utf8_lossy( &out.stdout ),
  );

  let trace_reading_count = err
    .lines()
    .filter( |l| l.contains( "[trace]" ) && l.contains( "reading" ) )
    .count();
  assert!(
    trace_reading_count >= 2,
    "stagger must fire before each account fetch — expected ≥ 2 '[trace] … reading' lines on stderr, \
     got {trace_reading_count}:\n{err}",
  );
  // Both accounts must appear individually in trace output — catches regressions
  // that skip one account during the stagger loop.
  assert!(
    err.contains( "alpha@test.com" ),
    "trace must include alpha@test.com, got:\n{err}",
  );
  assert!(
    err.contains( "beta@test.com" ),
    "trace must include beta@test.com, got:\n{err}",
  );
}

// ── Feature 037: Accounts/Usage Param Unification ─────────────────────────────

#[ test ]
/// f37-FT-02 (AC-02): `.usage` accepts all 32 unified params; unknown param exits 1.
///
/// Structural registration test: each unified param must be accepted without a
/// "unknown parameter" error. Mutation params gated with `dry::1`.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-02]
fn f37_ft02_usage_accepts_32_params()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );

  // Display and filter params (offline-safe).
  // cols:: values must be valid for .usage (accounts-specific cols like uuid/tier are invalid).
  let out = run_cs_with_env(
    &[
      ".usage",
      "trace::1",
      "format::text",
      "cols::+host,-sub",
      "sort::name",
      "desc::0",
      "no_color::1",
      "count::10",
      "offset::0",
      "only_active::0",
      "only_next::0",
      "min_5h::0",
      "min_7d::0",
      "only_valid::0",
      "exclude_exhausted::0",
      "abs::0",
    ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // prefer/imodel/effort accepted (next:: removed in Feature 037/038).
  let out = run_cs_with_env(
    &[ ".usage", "prefer::any", "imodel::auto", "effort::auto" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // Mutation params accepted when dry::1 suppresses writes (Feature 065: assignee:: replaces active::/assign::1).
  let out = run_cs_with_env(
    &[ ".usage", "assignee::testuser@testmachine", "name::alice@acme.com", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // Unknown param exits 1.
  let out = run_cs_with_env(
    &[ ".usage", "unknown_foobar_xyz::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

#[ test ]
/// f37-FT-04 (AC-04): `.usage` default — Owner column visible; owner value from `{name}.json`.
///
/// Owner column is part of the default quota set for `.usage`. The owner value
/// comes from the local metadata file, independent of network quota fetch results.
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-04]
fn f37_ft04_usage_default_profile()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "testuser@testmachine" );

  // .usage defaults: refresh::1, touch::1 (network calls fail gracefully offline).
  // Owner column must be present regardless of fetch result.
  let out = run_cs_with_env(
    &[ ".usage" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!(
    text.contains( "Owner" ),
    "f37-FT-04: Owner column must appear in default .usage output; got:\n{text}",
  );
  assert!(
    text.contains( "testuser@testmachine" ),
    "f37-FT-04: owner value must appear for alice; got:\n{text}",
  );
}

#[ test ]
/// f37-FT-16 (AC-16): `.usage owner::0 name::X` clears owner — identical to `.accounts owner::0 name::X`.
///
/// alice is owned by testuser@testmachine (G8 passes). After `.usage owner::0 name::alice`,
/// `alice.json` has `"owner": ""`. (`unclaim::1` was REMOVED in Feature 064; `owner::0` is the successor.)
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-16]
fn f37_ft16_usage_unclaim_mirrors_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "testuser@testmachine" );

  let out = run_cs_with_env(
    &[ ".usage", "owner::0", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ),
    "",
    "f37-FT-16: .usage owner::0 must clear owner to \"\"",
  );
}

#[ test ]
/// f37-FT-17 (AC-17): `.usage assignee::USER@MACHINE name::X` writes marker — identical to `.accounts assignee::`.
///
/// After `.usage assignee::testuser@testmachine name::alice`, the per-machine marker file exists
/// and contains the account name. (`assign::1` was REMOVED in Feature 064; `assignee::` is the successor.)
///
/// Spec: [`tests/docs/feature/37_accounts_usage_param_unification.md` FT-17]
fn f37_ft17_usage_assign_mirrors_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "assignee::testuser@testmachine", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  let store   = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let content = std::fs::read_to_string( store.join( "_active_testmachine_testuser" ) )
    .expect( "f37-FT-17: .usage assignee::testuser@testmachine must write per-machine marker" );
  assert_eq!(
    content.trim(),
    "alice@acme.com",
    "f37-FT-17: marker must contain alice@acme.com",
  );
}

// ── Feature 065: assignee::0 sentinel on .usage ─────────────────────────────

#[ test ]
/// f65-FT-02 parity on `.usage` (AC-02 — `.usage` side): `.usage assignee::0 name::X` expands
/// sentinel to current identity and writes per-machine marker.
///
/// Mirrors `ec2_assignee_zero_sentinel_assign` in `account_assign_test.rs` but targets `.usage`.
/// Ensures Feature 037 parity: `.accounts assignee::0` and `.usage assignee::0` behave identically.
fn f65_ft02_usage_assignee_zero_sentinel_assign()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "assignee::0", "name::alice@acme.com" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  let store   = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let content = std::fs::read_to_string( store.join( "_active_testmachine_testuser" ) )
    .expect( "f65-FT-02: .usage assignee::0 must expand to testuser@testmachine and write marker" );
  assert_eq!(
    content.trim(),
    "alice@acme.com",
    "f65-FT-02: marker must contain alice@acme.com",
  );
}

// ── Feature 063: owner:: on .usage ─────────────────────────────────────────

/// FT-12 (AC-12, Feat 063): `.usage owner::` same behavior as `.accounts owner::`.
///
/// Spec: [`tests/docs/feature/63_explicit_ownership_claim.md` FT-12]
#[ test ]
fn f63_ft12_usage_owner_mirrors_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  let out = run_cs_with_env(
    &[ ".usage", "owner::user1@w003", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta  = std::fs::read_to_string( store.join( "alice@acme.com.json" ) ).unwrap();
  let val : serde_json::Value = serde_json::from_str( &meta ).unwrap();
  assert_eq!(
    val[ "owner" ].as_str().unwrap_or( "MISSING" ), "user1@w003",
    "FT-12: .usage owner:: must write owner field",
  );
}

// ── Feature 064: REMOVED_TOGGLE parity on .usage ────────────────────────────

/// FT-05 parity on `.usage` (AC-05 — `.usage` side): `assign::1` exits 1 with
/// REMOVED migration message pointing to `assignee::`.
///
/// Spec: [`tests/docs/feature/64_active_marker_and_owner_redesign.md` FT-05]
#[ test ]
fn f64_ft05_usage_assign_removed_toggle()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "assign::1", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "REMOVED" ) && err.contains( "assignee::" ),
    "f64-FT-05: `.usage assign::1` must exit 1 with REMOVED migration pointing to assignee::; got:\n{err}",
  );
}

/// FT-07 parity on `.usage` (AC-07 — `.usage` side): `unclaim::1` exits 1 with
/// REMOVED migration message pointing to `owner::0`.
///
/// Spec: [`tests/docs/feature/64_active_marker_and_owner_redesign.md` FT-07]
#[ test ]
fn f64_ft07_usage_unclaim_removed_toggle()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "" );

  let out = run_cs_with_env(
    &[ ".usage", "unclaim::1", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "REMOVED" ) && err.contains( "owner::0" ),
    "f64-FT-07: `.usage unclaim::1` must exit 1 with REMOVED migration pointing to owner::0; got:\n{err}",
  );
}

/// FT-06b parity on `.usage` (AC-06 — `.usage` side): `for::user@host` exits 1
/// with REMOVED migration message.
///
/// Spec: [`tests/docs/feature/64_active_marker_and_owner_redesign.md` FT-06]
#[ test ]
fn f64_ft06b_usage_for_removed_toggle()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "for::bob@laptop", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "REMOVED" ),
    "f64-FT-06b: `.usage for::user@host` must exit 1 with REMOVED migration; got:\n{err}",
  );
}

// ── Feature 065: assignee:: corner cases on .usage ───────────────────────────

#[ test ]
/// FT-10 parity on `.usage` (AC-10): `active::user@machine name::X` exits 1 with `REMOVED_TOGGLE`
/// migration message pointing to `assignee::`.
///
/// Mirrors `ft10_active_removed_toggle_migration_message` in `account_assign_test.rs` but
/// targets `.usage`. The `REMOVED_TOGGLE` check for `active::` is in `usage/api.rs`.
fn f65_ft10_usage_active_removed_toggle()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "active::user1@w003", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "REMOVED" ) && err.contains( "assignee::" ),
    "f65-FT-10: `.usage active::` must exit 1 with REMOVED migration pointing to assignee::; got:\n{err}",
  );
}

#[ test ]
/// FT-03 parity on `.usage` (AC-03): `.usage assignee::user1@w003` (no `name::`) clears the
/// per-machine marker file.
///
/// Mirrors `ft02_assignee_unassign_clears_marker` in `account_assign_test.rs` but targets `.usage`.
fn f65_ft03_usage_assignee_unassign()
{
  let dir    = TempDir::new().unwrap();
  let home   = dir.path().to_str().unwrap();
  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let marker = "_active_w003_user1";

  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write( store.join( marker ), "alice@acme.com" ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "assignee::user1@w003" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "unassigned" ) && out_text.contains( marker ),
    "f65-FT-03: .usage assignee:: unassign must confirm via stdout; got:\n{out_text}",
  );

  let still_has_content = std::fs::read_to_string( store.join( marker ) )
    .is_ok_and( |s| !s.trim().is_empty() );
  assert!( !still_has_content, "f65-FT-03: marker must be cleared or deleted after .usage unassign" );
}

#[ test ]
/// EC-5 parity on `.usage`: `assignee::badvalue` (no `@`, not `"0"`) exits 1 with format error.
///
/// Format validation runs inside the `assignee::` dispatch in `usage/api.rs` — same as accounts.rs.
fn f65_ec5_usage_assignee_badvalue()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "assignee::badvalue", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let err = stderr( &out );
  assert!(
    err.contains( "USER@MACHINE" ) || err.contains( "'@'" ),
    "f65-EC-5: `.usage assignee::badvalue` must explain USER@MACHINE format; got:\n{err}",
  );
}

#[ test ]
/// FT-05 parity on `.usage` (AC-05): `assignee::user@host name::X dry::1` prints `[dry-run]`
/// preview and writes no marker file.
///
/// Mirrors `ft03_assignee_assign_dry_run` in `account_assign_test.rs` but targets `.usage`.
fn f65_ft05_usage_assignee_dry_run_assign()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "assignee::testuser@testmachine", "name::alice@acme.com", "dry::1" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "[dry-run] would assign" ),
    "f65-FT-05: .usage assignee:: dry-run must print '[dry-run] would assign'; got:\n{out_text}",
  );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let marker_count = std::fs::read_dir( &store )
    .map_or( 0, |d| d.filter_map( core::result::Result::ok )
      .filter( |e| e.file_name().to_string_lossy().starts_with( "_active" ) )
      .count() );
  assert_eq!( marker_count, 0, "f65-FT-05: dry-run must write no marker files on .usage" );
}

#[ test ]
/// FT-04 parity on `.usage` (AC-04): `assignee::0` (no `name::`) expands sentinel to current
/// machine identity and clears the per-machine marker.
///
/// Mirrors `ec4_assignee_zero_sentinel_unassign` in `account_assign_test.rs` but targets `.usage`.
fn f65_ft04_usage_assignee_zero_sentinel_unassign()
{
  let dir    = TempDir::new().unwrap();
  let home   = dir.path().to_str().unwrap();
  let store  = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let marker = "_active_testmachine_testuser";

  std::fs::create_dir_all( &store ).unwrap();
  std::fs::write( store.join( marker ), "alice@acme.com" ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "assignee::0" ],
    &[ ( "HOME", home ), ( "USER", "testuser" ), ( "HOSTNAME", "testmachine" ) ],
  );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!(
    out_text.contains( "unassigned" ),
    "f65-FT-04: .usage assignee::0 must confirm unassign via stdout; got:\n{out_text}",
  );

  let still_has_content = std::fs::read_to_string( store.join( marker ) )
    .is_ok_and( |s| !s.trim().is_empty() );
  assert!( !still_has_content, "f65-FT-04: sentinel unassign must clear the marker on .usage" );
}

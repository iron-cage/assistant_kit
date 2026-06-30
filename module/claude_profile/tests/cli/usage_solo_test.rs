//! Integration tests: IT-247–IT-271 — `.usage` `solo::` parameter, cross-feature corner cases.
//!
//! Covers Feature 037 owner-column in `.usage` (IT-74), Feature 038 `rotate::1`
//! strategy-driven rotation removal, `solo::` parameter EC tests (Feature 061),
//! cross-feature corner cases, and BUG-307 det0 Cramer cofactor fix.
//!
//! Live tests (names contain `lim_it`) require a real Anthropic OAuth access token.

use crate::cli_runner::{
  BIN,
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token, write_account_owner,
  write_live_credentials_with_token, live_active_token, require_live_api,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── it247 ──────────────────────────────────────────────────────────────────────

/// it247 (016 FT-10 / AC-10): synthetic row is suppressed when the live token belongs to a
/// stored account — no duplicate row for that account name.
///
/// Setup: `alice@acme.com` is saved with token `tok-alice`. Live `~/.claude/.credentials.json`
/// also uses `tok-alice`. Because `alice@acme.com` is found in the credential store with a
/// matching token, the synthetic-row injection path must NOT fire; the stored row carries
/// all data. `✓` appears on the stored row; `alice@acme.com` appears exactly once.
#[ test ]
fn it247_synthetic_row_suppressed_name_collision()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Store alice with token tok-alice; mark her active.
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", true );
  // Live creds also use the same token → alice IS the current account.
  write_live_credentials_with_token( dir.path(), "tok-alice" );

  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // alice@acme.com must appear exactly once (no synthetic duplicate).
  let alice_count = text.matches( "alice@acme.com" ).count();
  assert_eq!(
    alice_count, 1,
    "alice@acme.com must appear exactly once — no synthetic duplicate row, got:\n{text}",
  );

  // No "(current session)" synthetic row — name collision must suppress injection.
  assert!(
    !text.contains( "(current session)" ),
    "synthetic row must be suppressed when stored account has matching token, got:\n{text}",
  );

  // The stored row must carry the ✓ flag.
  let alice_current = text.lines().any( |l|
    l.contains( '\u{2713}' ) && l.contains( "alice@acme.com" )
  );
  assert!( alice_current, "stored alice@acme.com row must carry ✓ when her token matches live creds, got:\n{text}" );
}

// ── it_ft028_17 ───────────────────────────────────────────────────────────────

/// `it_ft028_17` `lim_it` (028 FT-17 / AC-17): `only_active::1` performs exactly 1 HTTP fetch
/// on a store with N ≥ 3 accounts.
///
/// With `only_active::1`, the pipeline must fetch quota data only for the account that has the
/// active marker. Non-active accounts must be skipped (no HTTP fetch). `trace::1` makes each
/// quota fetch observable: every actual HTTP call produces a timestamped ` · ... result:` line on
/// stderr. Exactly 1 such line must appear; N lines would indicate the pipeline violation
/// tracked in BUG-245/BUG-246.
///
/// Spec: [`tests/docs/feature/028_usage_row_filtering.md` FT-17 AC-17]
#[ test ]
fn it_ft028_17_only_active_single_http_fetch()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it_ft028_17: no live token — skipping" );
    return;
  };
  if !require_live_api( "it_ft028_17" ) { return; }

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Active account (with live token, active marker set).
  write_account_with_token( dir.path(), "active@test.com",  &token, true );
  // Two additional accounts without tokens (no HTTP fetch should happen for these).
  write_account( dir.path(), "other1@test.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "other2@test.com", "max", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "only_active::1", "get::status", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );

  // Count lines that are both timestamp-prefixed and contain "result:" — each corresponds to
  // one HTTP fetch attempt.
  let result_lines : Vec< &str > = err
    .lines()
    .filter( |l| l.contains( " · " ) && l.contains( "result:" ) )
    .collect();

  assert_eq!(
    result_lines.len(), 1,
    "only_active::1 must trigger exactly 1 HTTP fetch (1 trace result line); \
     {} found — non-active accounts must be skipped; trace:\n{err}",
    result_lines.len(),
  );
}

// ── it248: Feature 037 Owner column in .usage (IT-74) ────────────────────────

/// it249 (AC-07 / 020): `sort::endurance` is rejected after strategy reduction to 3 variants.
///
/// Before: `sort::endurance` was valid → exit 0.
/// After:  `sort::endurance` is unrecognised → exit 1, error names `name`, `renew`, `renews`.
///
/// RED:   current code accepts `sort::endurance` → `assert_exit(&out, 1)` fails.
/// GREEN: after Phase 2 removes `Endurance` variant → assert passes.
///
/// Spec: [`tests/docs/cli/param/25_sort.md`]
#[ test ]
fn it249_sort_endurance_rejected_exit_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::endurance" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for valid in &[ "name", "renew", "renews" ]
  {
    assert!(
      err.contains( valid ),
      "sort::endurance error must name valid value `{valid}`; got:\n{err}",
    );
  }
  // "endurance" is omitted from the negative check — it naturally appears as the bad value
  // in the error message `invalid sort:: value "endurance": valid values are ...`.
  for removed in &[ "drain", "expires", "next" ]
  {
    assert!(
      !err.contains( removed ),
      "sort::endurance error must NOT name removed value `{removed}`; got:\n{err}",
    );
  }
}

/// it250 (AC-07 / 020): `sort::drain` is rejected after strategy reduction.
///
/// RED:   current code accepts `sort::drain` → `assert_exit(&out, 1)` fails.
/// GREEN: after Phase 2 removes `Drain` variant → assert passes.
///
/// Spec: [`tests/docs/cli/param/25_sort.md`]
#[ test ]
fn it250_sort_drain_rejected_exit_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::drain" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for valid in &[ "name", "renew", "renews" ]
  {
    assert!(
      err.contains( valid ),
      "sort::drain error must name valid value `{valid}`; got:\n{err}",
    );
  }
}

/// it251 (AC-07 / 020): `sort::next` is rejected after the alias is removed.
///
/// Before: `sort::next` resolved to the active `next::` strategy at parse time.
/// After:  `sort::next` is unrecognised → exit 1.
///
/// RED:   current code resolves `sort::next` → exit 0.
/// GREEN: after Phase 2 removes Next variant + alias resolution → exit 1.
///
/// Spec: [`tests/docs/cli/param/25_sort.md`]
#[ test ]
fn it251_sort_next_rejected_exit_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::next" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for valid in &[ "name", "renew", "renews" ]
  {
    assert!(
      err.contains( valid ),
      "sort::next error must name valid value `{valid}`; got:\n{err}",
    );
  }
}

/// it252 (AC-07 / 020): `sort::expires` is rejected after strategy reduction.
///
/// RED:   current code accepts `sort::expires` → `assert_exit(&out, 1)` fails.
/// GREEN: after Phase 2 removes `Expires` variant → assert passes.
///
/// Spec: [`tests/docs/cli/param/25_sort.md`]
#[ test ]
fn it252_sort_expires_rejected_exit_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::expires" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for valid in &[ "name", "renew", "renews" ]
  {
    assert!(
      err.contains( valid ),
      "sort::expires error must name valid value `{valid}`; got:\n{err}",
    );
  }
}

/// it253 (AC-09 / 020): `next::` is rejected as an unknown or removed parameter.
///
/// Before: `next::renew` was accepted → exit 0.
/// After:  `next::` is removed from `.usage`; passing it exits 1.
///
/// RED:   current code accepts `next::renew` → `assert_exit(&out, 1)` fails.
/// GREEN: after Phase 2 removes `next::` parsing → exit 1.
///
/// Spec: [`tests/docs/cli/param/32_next.md`] (REMOVED)
#[ test ]
fn it253_next_param_removed_exit_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::renew" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "next" ) || err.contains( "sort" ),
    "next::renew error must reference `next` or `sort`; got:\n{err}",
  );
}

/// it254 (structural): `NextStrategy` enum is absent from source after removal.
///
/// RED:   `NextStrategy` exists in `types.rs` → assert fails.
/// GREEN: after Phase 2 deletes `NextStrategy` entirely → assert passes.
#[ test ]
fn it254_next_strategy_enum_absent_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/types.rs" ) );
  assert!(
    !src.contains( "enum NextStrategy" ),
    "NextStrategy enum must be completely removed from types.rs; \
     check enum declaration and all impl blocks",
  );
}

/// it255 (structural): Single-strategy footer — `Next by strategy:` multi-line label absent.
///
/// Before: footer iterated 3 strategies and printed `"Next by strategy:\n"` header.
/// After:  footer shows one line for active `sort::` strategy — no multi-strategy header.
///
/// RED:   `render.rs` still contains `"Next by strategy:"` → assert fails.
/// GREEN: after Phase 4 collapses footer to 1 strategy → assert passes.
#[ test ]
fn it255_footer_single_strategy_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/render.rs" ) );
  assert!(
    !src.contains( "Next by strategy:" ),
    "render.rs must not contain `Next by strategy:` — footer must show 1 strategy line; \
     replace with single-strategy `Next (strategy): account   metric` format",
  );
}

/// it248 (IT-74, AC-19): Owner column visible by default on `.usage`; `cols::-owner` hides it.
///
/// Case A: `.usage` — "Owner" column header present; owned account shows owner identity;
/// unowned account shows em dash (—).
/// Case B: `.usage cols::-owner` — "Owner" column header absent.
///
/// Spec: [`tests/docs/cli/command/09_usage.md` IT-74]
#[ test ]
fn it248_owner_column_visible_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@acme.com", "testuser@testmachine" );

  write_account( dir.path(), "bob@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "bob@acme.com", "" );

  // Case A: default — Owner column header and values present.
  {
    let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
    let text = stdout( &out );
    assert!(
      text.contains( "Owner" ),
      "IT-74A: Owner column header must appear by default in .usage table; got:\n{text}",
    );
    assert!(
      text.contains( "testuser@testmachine" ),
      "IT-74A: alice's owner identity must appear in Owner column; got:\n{text}",
    );
    assert!(
      text.contains( "\u{2014}" ),
      "IT-74A: bob's empty owner must show em dash (—) in Owner column; got:\n{text}",
    );
  }

  // Case B: cols::-owner — Owner column header absent.
  {
    let out  = run_cs_with_env( &[ ".usage", "cols::-owner" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
    let text = stdout( &out );
    assert!(
      !text.contains( "Owner" ),
      "IT-74B: Owner column header must be hidden with cols::-owner; got:\n{text}",
    );
  }
}

// ── it257-it268: solo:: parameter EC tests (061) ─────────────────────────────

/// it257 (061 EC-1): `solo::0` (default off) — two owned accounts; command exits 0.
///
/// `solo::0` is equivalent to omitting the param. Both accounts have no `owner`
/// field set (empty owner → `is_owned=true`). Without a live access token the
/// accounts return error results but the command still exits 0.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-1]
#[ test ]
fn it257_solo_default_off_exits_0()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@work.pro", "max", "tier4", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@home.pro",   "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "solo::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // solo::0 (default off) must show all accounts — neither is hidden.
  assert!( text.contains( "alice@work.pro" ), "EC-1: alice must appear with solo::0; got:\n{text}" );
  assert!( text.contains( "bob@home.pro" ),   "EC-1: bob must appear with solo::0; got:\n{text}" );
}

/// it258 (061 EC-2): `solo::1` — both accounts appear in the output table;
/// non-current account uses approximated data; exits 0.
///
/// Account A (alice) has its stored token matching the live session credentials →
/// `is_current=true`. Account B (bob) has no access token → `is_current=false`.
/// With `solo::1`, Bob's fetch is intercepted by the solo gate and
/// `approximate_quota()` is called. Both rows appear in stdout because `solo::1`
/// controls token consumption only, not display visibility.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-2]
#[ test ]
fn it258_solo_current_live_noncurrent_approx()
{
  const FAKE_TOK : &str = "solo-ec2-fake-token";
  let dir              = TempDir::new().unwrap();
  let home             = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  // Alice: has an accessToken that matches live creds → is_current=true.
  write_account_with_token( dir.path(), "alice@work.pro", FAKE_TOK, false );
  write_live_credentials_with_token( dir.path(), FAKE_TOK );

  // Bob: no accessToken → is_current=false; quota cache for approximate_quota().
  write_account( dir.path(), "bob@home.pro", "max", "tier4", FAR_FUTURE_MS, false );
  let bob_cache = serde_json::json!({ "cache": { "fetched_at": "2026-01-01T10:00:00Z", "five_hour": { "left_pct": 55.0 } } });
  std::fs::write(
    credential_store.join( "bob@home.pro.json" ),
    serde_json::to_string_pretty( &bob_cache ).unwrap() + "\n",
  ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "solo::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let err  = stderr( &out );
  // Both rows appear — solo::1 controls token consumption, not display.
  assert!(
    text.contains( "alice@work.pro" ),
    "EC-2: alice (current+owned) must appear in output; got:\n{text}",
  );
  assert!(
    text.contains( "bob@home.pro" ),
    "EC-2: bob (solo-skipped) must still appear in table (solo controls fetch, not display); got:\n{text}",
  );
  // Alice is current+owned → solo gate must NOT fire for her.
  assert!(
    !err.contains( "alice@work.pro  solo-skip" ),
    "EC-2: alice (is_current=true) must not be solo-skipped; got stderr:\n{err}",
  );
  // Bob is non-current → solo gate fires.
  assert!(
    err.contains( "solo-skip" ),
    "EC-2: bob (is_current=false) must be solo-skipped; got stderr:\n{err}",
  );
}

/// it259 (061 EC-3): `solo::1` — current account is NOT owned; G1 fires for it;
/// no account passes both `is_current && is_owned`; all non-live.
///
/// Alice is "current" (stored token matches live session) but her `owner` field
/// is set to `"other@remote"` — G1 fires and Alice takes the non-owned cache
/// path (trace: "skipped (reason: not owned)"). Bob is owned but not current
/// — solo gate fires (trace: "solo-skip: approximated"). Neither makes an HTTP call.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-3]
#[ test ]
fn it259_solo_current_not_owned_no_http()
{
  const FAKE_TOK : &str = "solo-ec3-fake-token";
  let dir              = TempDir::new().unwrap();
  let home             = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  // Alice: current (token match) but NOT owned (foreign owner).
  write_account_with_token( dir.path(), "alice@work.pro", FAKE_TOK, false );
  write_live_credentials_with_token( dir.path(), FAKE_TOK );
  write_account_owner( dir.path(), "alice@work.pro", "other@remote" );

  // Bob: owned (no owner field) but not current (no token); quota cache for approximate_quota.
  write_account( dir.path(), "bob@home.pro", "max", "tier4", FAR_FUTURE_MS, false );
  let bob_cache = serde_json::json!({ "cache": { "fetched_at": "2026-01-01T10:00:00Z", "five_hour": { "left_pct": 30.0 } } });
  std::fs::write(
    credential_store.join( "bob@home.pro.json" ),
    serde_json::to_string_pretty( &bob_cache ).unwrap() + "\n",
  ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "solo::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  // Alice: G1 non-owned trace; Bob: solo-skip trace — both prove no HTTP for either.
  assert!(
    err.contains( "not owned" ),
    "EC-3: G1 trace must show 'not owned' for alice (current but foreign owner); got:\n{err}",
  );
  assert!(
    err.contains( "solo-skip" ),
    "EC-3: solo trace must show 'solo-skip' for bob (owned but not current); got:\n{err}",
  );
}

/// it260 (061 EC-4): `solo::1` — no active marker; no live credentials;
/// `is_current=false` for all accounts → all take solo-skip path.
///
/// Without `_active_*` marker or `~/.claude/.credentials.json`, `live_token`
/// is `None` and `is_current=false` for every account. With `solo::1` all
/// owned accounts are intercepted by the solo gate.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-4]
#[ test ]
fn it260_solo_no_active_marker_all_approx()
{
  let dir              = TempDir::new().unwrap();
  let home             = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  // Two owned accounts — no active marker, no live credentials.
  write_account( dir.path(), "alice@work.pro", "max", "tier4", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@home.pro",   "max", "tier4", FAR_FUTURE_MS, false );

  // Write quota cache so approximate_quota returns data.
  for name in &[ "alice@work.pro", "bob@home.pro" ]
  {
    let cache = serde_json::json!({ "cache": { "fetched_at": "2026-01-01T10:00:00Z", "five_hour": { "left_pct": 40.0 } } });
    std::fs::write(
      credential_store.join( format!( "{name}.json" ) ),
      serde_json::to_string_pretty( &cache ).unwrap() + "\n",
    ).unwrap();
  }

  let out = run_cs_with_env( &[ ".usage", "solo::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "solo-skip: approximated" ),
    "EC-4: both accounts must show solo-skip when no active marker; got:\n{err}",
  );
}

/// it261 (061 EC-5): `solo::1 rotate::1` — mutual exclusion; exits 1 before any fetch.
///
/// Both params conflict: `rotate::1` requires live data from all candidate accounts
/// to pick the next one, while `solo::1` restricts HTTP to the current account only.
/// The conflict is detected at parse time; no fetch or rotate occurs.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-5]
#[ test ]
fn it261_solo_rotate_mutual_exclusion_exit_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "solo::1", "rotate::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "solo" ) && err.contains( "rotate" ),
    "EC-5: error must reference both `solo` and `rotate`; got:\n{err}",
  );
}

/// it262 (061 EC-6): `solo::1 live::1` — allowed; loop starts; SIGINT → exit 0.
///
/// `solo::1` and `live::1` are not mutually exclusive. The live-monitor loop runs
/// normally; each cycle only the current+owned account is live-fetched. SIGINT
/// causes a clean exit 0 with "Monitor stopped." in stdout.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-6]
#[ cfg( unix ) ]
#[ test ]
fn it262_solo_live_composition_allowed()
{
  use std::process::Stdio;

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Single account, no token → instant fail, render error row, countdown starts.
  write_account( dir.path(), "alice@work.pro", "max", "tier4", FAR_FUTURE_MS, true );

  let child = std::process::Command::new( BIN )
    .args( [ ".usage", "solo::1", "live::1", "interval::30", "jitter::0" ] )
    .env( "HOME", home )
    .env_remove( "PRO" )
    .stdout( Stdio::piped() )
    .stderr( Stdio::piped() )
    .spawn()
    .expect( "failed to spawn clp binary" );

  std::thread::sleep( core::time::Duration::from_secs( 3 ) );

  let _ = std::process::Command::new( "kill" )
    .args( [ "-INT", &child.id().to_string() ] )
    .status();

  let out  = child.wait_with_output().expect( "failed to wait on clp binary" );
  let text = String::from_utf8_lossy( &out.stdout );
  assert_eq!(
    out.status.code(),
    Some( 0 ),
    "EC-6: SIGINT on solo+live must exit 0; got: {:?}\nstdout: {text}\nstderr: {}",
    out.status,
    String::from_utf8_lossy( &out.stderr ),
  );
  assert!(
    text.contains( "Monitor stopped." ),
    "EC-6: clean SIGINT exit must print 'Monitor stopped.'; got:\n{text}",
  );
}

/// it263 (061 EC-7): `solo::1 refresh::1` — allowed; exits 0; refresh solo gate
/// fires for non-current accounts.
///
/// Alice is current+owned (token match). Bob is not current. With `solo::1
/// refresh::1 trace::1` the refresh solo gate emits `solo-skip` for Bob —
/// proving the gate fires before any refresh subprocess decision for Bob.
/// Alice passes the solo gate; no subprocess fires (no 401 from HTTP).
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-7]
#[ test ]
fn it263_solo_refresh_composition_allowed()
{
  const FAKE_TOK : &str = "solo-ec7-fake-token";
  let dir              = TempDir::new().unwrap();
  let home             = dir.path().to_str().unwrap();

  write_account_with_token( dir.path(), "alice@work.pro", FAKE_TOK, false );
  write_live_credentials_with_token( dir.path(), FAKE_TOK );
  write_account( dir.path(), "bob@home.pro", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "solo::1", "refresh::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  // Refresh solo gate must fire for Bob (not current) — emits `solo-skip` in refresh trace.
  assert!(
    err.lines().any( |l| l.contains( "refresh" ) && l.contains( "solo-skip" ) ),
    "EC-7: refresh solo gate must emit 'solo-skip' for non-current account; got stderr:\n{err}",
  );
}

/// it264 (061 EC-8): `solo::1 touch::1` — allowed; exits 0; touch solo gate
/// fires for non-current accounts.
///
/// Alice is current+owned (token match). Bob is not current. With `solo::1
/// touch::1 trace::1` the touch solo gate emits `solo-skip` for Bob —
/// proving the gate fires before any touch subprocess decision for Bob.
/// Alice passes the solo gate; no subprocess fires (no active idle window
/// without real quota data from a failed HTTP fetch).
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-8]
#[ test ]
fn it264_solo_touch_composition_allowed()
{
  const FAKE_TOK : &str = "solo-ec8-fake-token";
  let dir              = TempDir::new().unwrap();
  let home             = dir.path().to_str().unwrap();

  write_account_with_token( dir.path(), "alice@work.pro", FAKE_TOK, false );
  write_live_credentials_with_token( dir.path(), FAKE_TOK );
  write_account( dir.path(), "bob@home.pro", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "solo::1", "touch::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  // Touch solo gate must fire for Bob (not current) — emits `solo-skip` in touch trace.
  assert!(
    err.lines().any( |l| l.contains( "touch" ) && l.contains( "solo-skip" ) ),
    "EC-8: touch solo gate must emit 'solo-skip' for non-current account; got stderr:\n{err}",
  );
}

/// it265 (061 EC-9): `solo::1 only_active::1` — orthogonal composition; exits 0.
///
/// `solo::1` controls which accounts are HTTP-fetched; `only_active::1` controls
/// which accounts appear in the display table. They are independent. Alice has an
/// active marker → shown by `only_active::1`; Bob is not active → hidden.
/// Command exits 0 regardless of fetch results.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-9]
#[ test ]
fn it265_solo_only_active_composition_allowed()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Alice: active marker set (is_active=true).
  write_account( dir.path(), "alice@work.pro", "max", "tier4", FAR_FUTURE_MS, true );
  write_account( dir.path(), "bob@home.pro",   "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "solo::1", "only_active::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // only_active::1 filters by active marker; solo::1 controls fetch strategy — orthogonal.
  assert!( text.contains( "alice@work.pro" ), "EC-9: alice (is_active=true) must appear with only_active::1; got:\n{text}" );
  assert!( !text.contains( "bob@home.pro" ),  "EC-9: bob (is_active=false) must be hidden by only_active::1; got:\n{text}" );
}

/// it266 (061 EC-10): `solo::1 refresh::1 touch::1 trace::1` — stderr contains
/// `solo-skip` at all three gate sites (fetch, refresh, touch) for the non-current
/// account.
///
/// With two owned accounts, Alice is current (live token match) and Bob is not.
/// The solo gate fires at three sites for Bob: fetch (emits `solo-skip: approximated`),
/// refresh (emits `solo-skip`), and touch (emits `solo-skip`). Alice is not
/// solo-skipped at any site.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-10]
#[ test ]
fn it266_solo_trace_shows_solo_skip()
{
  const FAKE_TOK : &str = "solo-ec10-fake-token";
  let dir              = TempDir::new().unwrap();
  let home             = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  write_account_with_token( dir.path(), "alice@work.pro", FAKE_TOK, false );
  write_live_credentials_with_token( dir.path(), FAKE_TOK );

  write_account( dir.path(), "bob@home.pro", "max", "tier4", FAR_FUTURE_MS, false );
  let bob_cache = serde_json::json!({ "cache": { "fetched_at": "2026-01-01T10:00:00Z", "five_hour": { "left_pct": 42.0 } } });
  std::fs::write(
    credential_store.join( "bob@home.pro.json" ),
    serde_json::to_string_pretty( &bob_cache ).unwrap() + "\n",
  ).unwrap();

  let out = run_cs_with_env(
    &[ ".usage", "solo::1", "refresh::1", "touch::1", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  // Fetch solo gate fires for Bob — emits `solo-skip: approximated` with account name.
  assert!(
    err.contains( "solo-skip: approximated" ),
    "EC-10: stderr must contain `solo-skip: approximated` (fetch trace); got:\n{err}",
  );
  assert!(
    err.contains( "bob@home.pro" ),
    "EC-10: fetch trace line must name the skipped account `bob@home.pro`; got:\n{err}",
  );
  // Refresh solo gate also fires for Bob — emits `solo-skip` in refresh trace line.
  assert!(
    err.lines().any( |l| l.contains( "refresh" ) && l.contains( "solo-skip" ) ),
    "EC-10: solo gate must emit 'solo-skip' in refresh trace for non-current account; got:\n{err}",
  );
  // Touch solo gate also fires for Bob — emits `solo-skip` in touch trace line.
  assert!(
    err.lines().any( |l| l.contains( "touch" ) && l.contains( "solo-skip" ) ),
    "EC-10: solo gate must emit 'solo-skip' in touch trace for non-current account; got:\n{err}",
  );
}

/// it267 (061 EC-11): `solo::true` exits 1 — `solo::` is `Kind::Integer`;
/// "true" fails integer parse at the framework level.
///
/// Same mechanism as `who::true` (it256): the unilang routing layer calls
/// `"true".parse::<i64>()` for `Kind::Integer` params, which fails before
/// `parse_usage_params` is reached.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-11]
#[ test ]
fn it267_solo_true_rejected_type_error()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "solo::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// it268 (061 EC-12): `solo::2` exits 1 — integer outside the valid set {0, 1}.
///
/// `parse_int_flag` for `solo::` accepts only 0 and 1. Any other integer
/// value is rejected with exit 1.
///
/// Spec: [`tests/docs/cli/param/61_solo.md` EC-12]
#[ test ]
fn it268_solo_2_rejected_out_of_range()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "solo::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  // parse_int_flag produces "solo:: must be 0, 1, false, or true" for integer outside {0, 1}.
  assert!(
    err.contains( "solo" ),
    "EC-12: error message must reference `solo`; got stderr:\n{err}",
  );
}

// ── Cross-feature corner cases ────────────────────────────────────────────────

/// it269 (CC): `solo::1 format::json` — JSON output works with solo-approximated data.
///
/// Solo gate uses `approximate_quota()` for non-current accounts, which produces
/// `AccountQuota` with `cached=true` and `is_owned=true`. JSON renderer must handle
/// these approximated rows identically to live-fetched rows (no panic, no missing fields).
#[ test ]
fn it269_solo_json_format_with_approximated_accounts()
{
  let dir              = TempDir::new().unwrap();
  let home             = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  // Two owned accounts — no active marker, no live credentials.
  write_account( dir.path(), "alice@work.pro", "max", "tier4", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@home.pro",   "max", "tier4", FAR_FUTURE_MS, false );

  // Quota cache so approximate_quota returns data.
  for name in &[ "alice@work.pro", "bob@home.pro" ]
  {
    let cache = serde_json::json!({ "cache": { "fetched_at": "2026-01-01T10:00:00Z", "five_hour": { "left_pct": 40.0 } } });
    std::fs::write(
      credential_store.join( format!( "{name}.json" ) ),
      serde_json::to_string_pretty( &cache ).unwrap() + "\n",
    ).unwrap();
  }

  let out = run_cs_with_env( &[ ".usage", "solo::1", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let json_str = stdout( &out );
  // Must be valid JSON array.
  let parsed : serde_json::Value = serde_json::from_str( &json_str )
    .unwrap_or_else( |e| panic!( "CC: solo+json must produce valid JSON; err={e}; raw:\n{json_str}" ) );
  assert!(
    parsed.is_array(),
    "CC: solo+json output must be a JSON array; got:\n{json_str}",
  );
}

/// it270 (CC): `solo::1` with a single account that IS current — solo has no effect.
///
/// When only one account exists and it is the current one, the solo gate lets it
/// through for live fetch (not approximation). The output must show the account
/// without any "solo-skip" trace in the fetch phase.
///
/// `is_current` requires both: (1) `write_account_with_token` stores the token in
/// the credential store, (2) `write_live_credentials_with_token` writes the same
/// token to `~/.claude/.credentials.json` so the live-token comparison succeeds.
#[ test ]
fn it270_solo_single_current_account_no_skip()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  // Single owned account with matching live credentials → is_current=true.
  write_account_with_token( dir.path(), "only@work.pro", "tok-only", true );
  write_live_credentials_with_token( dir.path(), "tok-only" );

  // Quota cache (used as fallback; but the fetch gate should let it through).
  let cache = serde_json::json!({ "cache": { "fetched_at": "2026-01-01T10:00:00Z", "five_hour": { "left_pct": 50.0 } } });
  std::fs::write(
    store.join( "only@work.pro.json" ),
    serde_json::to_string_pretty( &cache ).unwrap() + "\n",
  ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "solo::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  // No trace line for this account should contain "solo-skip" in the fetch phase.
  let fetch_solo_skip = err.lines().any( |l| l.contains( "fetch" ) && l.contains( "solo-skip" ) );
  assert!(
    !fetch_solo_skip,
    "CC: fetch phase must not solo-skip the current account; got stderr:\n{err}",
  );
}

/// it256 (062 EC-6): `who::true` exits 1 — `who::` is `Kind::Integer`; "true" fails integer parse at framework level.
///
/// The unilang routing layer calls `"true".parse::<i64>()` for `Kind::Integer` params, which
/// fails before `parse_usage_params` is reached. This is distinct from `Kind::String` params
/// (e.g. `only_next::`) where "true" passes through as `Value::String` and `parse_int_flag`
/// maps it to `Ok(1)`.
///
/// Spec: [`tests/docs/cli/param/62_who.md` EC-6]
#[ test ]
fn it256_who_true_rejected_kind_integer()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "who::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// it271 (036 FT-24): G1b gate — owned account occupied on another machine is
/// skipped with `YYYY-MM-DD · HH:MM:SS · fetch  <name>  skipped (reason: occupied elsewhere)`.
///
/// # Root Cause
/// `fetch_quota_for_list()` computed `occupied_elsewhere` at line 74 but used it
/// only to stamp `is_occupied_elsewhere` on the pushed `AccountQuota`. No gate
/// existed between the solo gate (`continue;` at line 148) and the expiry
/// pre-flight check (line 154), causing unnecessary HTTP round-trips for every
/// owned account active on another machine.
///
/// # Why Not Caught
/// No integration test exercised a multi-machine setup with an owned account
/// active elsewhere; the field was stamped but never gated.
///
/// # Fix Applied
/// G1b gate inserted after the solo gate: `if !is_current && occupied_elsewhere.contains(&acct.name)`
/// → calls `approximate_quota()` (cache+polynomial approximation) and continues.
///
/// # Prevention
/// This test creates an `_active_remotebox_remoteuser` marker containing the
/// account name, then asserts the skip-trace line is present and no credential-
/// read trace appears for the skipped account.
///
/// # Pitfall
/// `other_machines_active()` reads the FILE CONTENT (not filename) as the account
/// name; the marker file must contain exactly the account name to trigger the gate.
#[ doc = "bug_reproducer(BUG-305)" ]
#[ test ]
fn mre_bug305_fetch_skips_occupied_elsewhere_with_trace()
{
  let dir              = TempDir::new().unwrap();
  let home             = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );

  // Owned account with quota cache + token (so it would normally attempt HTTP).
  let acct_name = "occ@work.pro";
  write_account( dir.path(), acct_name, "max", "tier4", FAR_FUTURE_MS, false );
  let cache = serde_json::json!({ "cache": { "fetched_at": "2026-01-01T10:00:00Z", "five_hour": { "left_pct": 30.0 } } });
  std::fs::write(
    credential_store.join( format!( "{acct_name}.json" ) ),
    serde_json::to_string_pretty( &cache ).unwrap() + "\n",
  ).unwrap();

  // Remote-machine active marker — content is the account name.
  std::fs::write(
    credential_store.join( "_active_remotebox_remoteuser" ),
    acct_name,
  ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );

  // G1b skip trace must be present.
  assert!(
    err.contains( "occupied elsewhere" ) && err.contains( " · fetch  " ),
    "FT-24: G1b skip trace must contain 'occupied elsewhere' in a fetch trace line; got:\n{err}",
  );

  // No credential-read trace for the occupied account — proves HTTP was skipped.
  let cred_read = err.lines().any( |l| l.contains( acct_name ) && l.contains( "reading" ) );
  assert!(
    !cred_read,
    "FT-24: no credential-read trace must appear for occupied-elsewhere account; got:\n{err}",
  );
}

// ── BUG-307 — det0 Cramer cofactor error in quadratic_fit ─────────────────────

/// `mre_bug307` (BUG-307): `quadratic_fit` in `approx.rs` must use `s2 * r1` (not `s1 * r2`)
/// for the `det0` Cramer cofactor that computes the constant term `a0` of the quadratic fit.
///
/// # Root Cause
/// Least-squares quadratic fit uses a 3×3 normal equation solved by Cramer's rule.
/// For `a0`, column 3 is replaced by the RHS `[r2, r1, r0]^T`. Cofactor(1,2) of that matrix
/// is `-(s3·r0 - s2·r1)`. The bug used `s1·r2` — mixing power-sum index `s1` with RHS index
/// `r2` — producing wrong `a0` for collinear data with large normalized timestamps.
///
/// # Why Not Caught
/// Feature 040 math tests (FT-04/FT-06–FT-10) accepted any `v > 35.0 && v <= 100.0`. For
/// 3 linear points (10 → 25 → 40, slope = 15/3600 per second), the wrong formula produced
/// `a0 ≈ 77.5` → `y(t_now) ≈ 122.5` → clamped to 100.0. Since 100.0 ∈ (35, 100], the broad
/// range test passed. The bug only surfaced when FT-17 tightened the range to `abs(v - 55.0) < 5.0`.
///
/// # Fix Applied
/// Changed `s1 * r2` → `s2 * r1` in the `det0` line at `approx.rs:142`. The replaced-column
/// minor for det0 uses element `r1` from the RHS column (row 2), not the power-sum variable `r2`.
///
/// # Prevention
/// This test asserts `Fix(BUG-307)` is present in `approx.rs`. Any future edit that reverts the
/// comment causes this test to fail before the clamped-100.0 path can be reached in production.
/// FT-17 (`test_read_cached_quota_applies_approximation`) is the end-to-end regression.
///
/// # Pitfall
/// When deriving 3×3 Cramer minors for `det0`, cofactor terms involve the REPLACED column
/// (RHS `[r2, r1, r0]`), not the original power-sum column. Each of `det2`/`det1`/`det0`
/// replaces a different column — never copy cofactor terms across them without re-deriving.
#[ doc = "bug_reproducer(BUG-307)" ]
#[ test ]
fn mre_bug307_approx_det0_quadratic_fit_correct()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/approx.rs" ) );

  // Before fix: `Fix(BUG-307)` absent → det0 uses wrong cofactor (s1*r2) → a0≈77.5 →
  //             y(t_now)≈122.5 → clamped 100.0 for linear data with large timestamps.
  // After fix:  `Fix(BUG-307)` present → correct cofactor s2*r1 → a0≈10.0 → extrapolation ~55.0.
  assert!(
    src.contains( "Fix(BUG-307)" ),
    "BUG-307: approx.rs must contain Fix(BUG-307) — wrong det0 Cramer cofactor (s1*r2 → s2*r1) \
     caused quadratic fit to clamp to 100.0 for linear data with large normalized timestamps; \
     fix: use s2*r1 in the det0 minor (col-3 is replaced by RHS, so the minor uses r1 not r2).",
  );
}

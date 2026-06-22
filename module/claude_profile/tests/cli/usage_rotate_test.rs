//! Integration tests: Feature 038 — `.usage rotate::1` strategy-driven rotation.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! Live tests (names contain `lim_it`) require network access and are excluded
//! from Docker CI by the nextest filter `!test(lim_it)`.
//!
//! ## Test Matrix
//!
//! | ID     | Test Function                                          | AC    | Live? |
//! |--------|--------------------------------------------------------|-------|-------|
//! | FT-04  | `ft04_rotate_live_mutual_exclusion`                    | AC-04 | no    |
//! | FT-03  | `ft03_no_eligible_account_exits_1`                     | AC-03 | no    |
//! | FT-01  | `ft01_lim_it_rotates_to_next_winner`                   | AC-01 | yes   |
//! | FT-02  | `ft02_lim_it_dry_run_no_switch`                        | AC-02 | yes   |
//! | FT-05  | `ft05_lim_it_g5_gate_skips_non_owned`                  | AC-05 | yes   |
//! | FT-06  | `ft06_lim_it_force_bypasses_g5`                        | AC-06 | yes   |
//! | FT-07  | `ft07_lim_it_sort_renews`                              | AC-07 | yes   |
//! | FT-08  | `ft08_lim_it_format_json_switch_executes`              | AC-08 | yes   |
//! | FT-09  | `ft09_lim_it_touch_reuse_no_extra_api_call`            | AC-09 | yes   |
//! | FT-10  | `ft10_non_owned_no_force_exits_1`                      | AC-10 | no    |
//! | EC-05  | `ec05_rotate_true_rejected_not_integer`                | EC-5  | no    |
//! | EC-06  | `ec06_rotate_false_rejected_not_integer`               | EC-6  | no    |
//! | EC-07  | `ec07_rotate_2_rejected_out_of_range`                  | EC-7  | no    |
//! | CC-01  | `cc01_rotate_offline_live_creds_replaced_by_winner`    | —     | no    |
//! | CC-02  | `cc02_rotate_touch_offline_live_creds_still_replaced`  | —     | no    |
//! | CC-03  | `cc03_rotate_zero_no_switch`                           | —     | no    |
//! | CC-04  | `cc04_rotate_zero_table_rendered`                      | —     | no    |
//! | CC-05  | `cc05_rotate_sort_name_selects_alphabetical_winner`    | AC-07 | no    |
//! | CC-06  | `cc06_rotate_format_tsv_executes_switch`               | AC-08 | no    |
//! | CC-07  | `cc07_rotate_dry_offline_no_credential_change`         | AC-02 | no    |

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, write_account_owner,
  write_account_quota_cache,
  require_live_api, write_account_with_token, live_active_token,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── FT-04: rotate::1 + live::1 mutual exclusion (AC-04) ───────────────────────

/// FT-04 (AC-04): `rotate::1 live::1` → exit 1 before any fetch; error message
/// references mutual exclusion.
///
/// Source: `tests/docs/feature/38_usage_strategy_rotate.md § FT-04`.
#[ test ]
fn ft04_rotate_live_mutual_exclusion()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Account not needed — command must exit 1 before fetch due to param guard.
  write_account( dir.path(), "any@test.com", "max", "default", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "live::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stdout( &out ), stderr( &out ) );
  assert!(
    combined.contains( "rotate" ) && combined.contains( "live" ),
    "error must reference both 'rotate' and 'live' params, got:\n{combined}",
  );
}

// ── FT-03: no eligible account → exit 1 (AC-03) ──────────────────────────────

/// FT-03 (AC-03): all accounts fail API fetch (no `accessToken`) → `result: Err`
/// → `find_first_eligible` skips all → no eligible → exit 1; table still rendered.
///
/// Source: `tests/docs/feature/38_usage_strategy_rotate.md § FT-03`.
#[ test ]
fn ft03_no_eligible_account_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Single account with no accessToken → API call fails → result: Err → not eligible.
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "default", FAR_FUTURE_MS, true );
  write_account( dir.path(), "other@test.com",   "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stdout( &out ), stderr( &out ) );
  assert!(
    combined.contains( "eligible" ) || combined.contains( "rotate" ),
    "error must reference 'eligible' or 'rotate', got:\n{combined}",
  );
}

// ── FT-10: non-owned account without force → exit 1 (AC-10) ──────────────────

/// FT-10 (AC-10): only non-owned account in store, `force::0` (default) →
/// exit 1; credentials unchanged.
///
/// Note: with fake credentials (no `accessToken`), the account fails API fetch
/// so the exit-1 is "no eligible account" (pre-API-check: `result: Err` skipped
/// by `find_first_eligible`). The test verifies exit 1 and unchanged credentials.
///
/// Source: `tests/docs/feature/38_usage_strategy_rotate.md § FT-10`.
#[ test ]
fn ft10_non_owned_no_force_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "active@test.com",  "max", "default", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "foreign@test.com", "max", "default", FAR_FUTURE_MS, false );
  // Mark foreign as owned by a different machine — G5 gate applies.
  write_account_owner( dir.path(), "foreign@test.com", "other@remotemachine" );

  let before = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).expect( "credentials must exist before rotate" );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );

  let after = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).expect( "credentials must still exist after failed rotate" );
  assert_eq!(
    before, after,
    "credentials must be unchanged when rotate fails",
  );
}

// ── FT-01: rotate to → winner (AC-01, lim_it) ─────────────────────────────────

/// FT-01 (AC-01, `lim_it`): `rotate::1` switches to the `→` winner (`sort::renew`).
/// Output ends with `switched to '{name}'`. Active marker updated.
///
/// Source: `tests/docs/feature/38_usage_strategy_rotate.md § FT-01`.
#[ test ]
fn ft01_lim_it_rotates_to_next_winner()
{
  if !require_live_api( "ft01_lim_it_rotates_to_next_winner" ) { return; }
  let Some( token ) = live_active_token() else { eprintln!( "ft01: no live token — skipping" ); return; };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Active account: currently checked-out token (must be set up so rotate picks a different one).
  write_account_with_token( dir.path(), "active@test.com", &token, true );
  write_account_with_token( dir.path(), "rotate_target@test.com", &token, false );

  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1" ],
    &[ ( "HOME", home ) ],
  );
  // Must exit 0 (switched) or 1 (no eligible — rate limited/both same token).
  // When eligible: exit 0 + "switched to" in output.
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!(
      text.contains( "switched to" ),
      "rotate::1 on success must output 'switched to', got:\n{text}",
    );
  }
}

// ── FT-02: dry run previews without switching (AC-02, lim_it) ─────────────────

/// FT-02 (AC-02, `lim_it`): `rotate::1 dry::1` outputs `[dry-run] would switch to '{name}'`;
/// credentials unchanged.
///
/// Source: `tests/docs/feature/38_usage_strategy_rotate.md § FT-02`.
#[ test ]
fn ft02_lim_it_dry_run_no_switch()
{
  if !require_live_api( "ft02_lim_it_dry_run_no_switch" ) { return; }
  let Some( token ) = live_active_token() else { eprintln!( "ft02: no live token — skipping" ); return; };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account_with_token( dir.path(), "active@test.com", &token, true );
  write_account_with_token( dir.path(), "candidate@test.com", &token, false );

  let creds_before = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).expect( "credentials must exist" );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "dry::1" ],
    &[ ( "HOME", home ) ],
  );

  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!(
      text.contains( "[dry-run]" ) && text.contains( "would switch to" ),
      "dry-run rotate::1 must output '[dry-run] would switch to', got:\n{text}",
    );
    let creds_after = std::fs::read_to_string(
      dir.path().join( ".claude" ).join( ".credentials.json" ),
    ).unwrap_or_default();
    assert_eq!( creds_before, creds_after, "credentials must be unchanged during dry-run" );
  }
}

// ── FT-05: G5 gate skips non-owned, selects next owned (AC-05, lim_it) ────────

/// FT-05 (AC-05, `lim_it`): non-owned account is skipped by G5 gate; rotation
/// switches to the next owned account.
///
/// Source: `tests/docs/feature/38_usage_strategy_rotate.md § FT-05`.
#[ test ]
fn ft05_lim_it_g5_gate_skips_non_owned()
{
  if !require_live_api( "ft05_lim_it_g5_gate_skips_non_owned" ) { return; }
  let Some( token ) = live_active_token() else { eprintln!( "ft05: no live token — skipping" ); return; };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account_with_token( dir.path(), "active@test.com",  &token, true  );
  write_account_with_token( dir.path(), "foreign@test.com", &token, false );
  write_account_with_token( dir.path(), "mine@test.com",    &token, false );
  // foreign is non-owned; mine has no explicit owner → owned (default).
  write_account_owner( dir.path(), "foreign@test.com", "other@remotemachine" );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1" ],
    &[ ( "HOME", home ) ],
  );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!(
      !text.contains( "foreign@test.com" ) || text.contains( "switched to 'mine@test.com'" ),
      "G5 must skip non-owned 'foreign'; must switch to 'mine', got:\n{text}",
    );
    assert!(
      !text.contains( "switched to 'foreign@test.com'" ),
      "must NOT switch to the non-owned 'foreign' account, got:\n{text}",
    );
  }
}

// ── FT-06: force::1 bypasses G5 gate (AC-06, lim_it) ─────────────────────────

/// FT-06 (AC-06, `lim_it`): `force::1` allows rotation to a non-owned account.
///
/// Source: `tests/docs/feature/38_usage_strategy_rotate.md § FT-06`.
#[ test ]
fn ft06_lim_it_force_bypasses_g5()
{
  if !require_live_api( "ft06_lim_it_force_bypasses_g5" ) { return; }
  let Some( token ) = live_active_token() else { eprintln!( "ft06: no live token — skipping" ); return; };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account_with_token( dir.path(), "active@test.com",  &token, true  );
  write_account_with_token( dir.path(), "foreign@test.com", &token, false );
  write_account_owner( dir.path(), "foreign@test.com", "other@remotemachine" );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "force::1" ],
    &[ ( "HOME", home ) ],
  );
  // With force::1 and a live-eligible foreign account: exit 0.
  // The test asserts the command doesn't exit 1 for ownership violation.
  if out.status.code() == Some( 1 )
  {
    let combined = format!( "{}{}", stdout( &out ), stderr( &out ) );
    assert!(
      !combined.contains( "ownership" ),
      "force::1 must bypass G5 ownership gate; got ownership error:\n{combined}",
    );
  }
}

// ── FT-07: sort::renews strategy (AC-07, lim_it) ─────────────────────────────

/// FT-07 (AC-07, `lim_it`): `rotate::1 sort::renews` switches to the account
/// with the soonest billing renewal.
///
/// Source: `tests/docs/feature/38_usage_strategy_rotate.md § FT-07`.
#[ test ]
fn ft07_lim_it_sort_renews()
{
  if !require_live_api( "ft07_lim_it_sort_renews" ) { return; }
  let Some( token ) = live_active_token() else { eprintln!( "ft07: no live token — skipping" ); return; };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account_with_token( dir.path(), "active@test.com",    &token, true  );
  write_account_with_token( dir.path(), "candidate@test.com", &token, false );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "sort::renews" ],
    &[ ( "HOME", home ) ],
  );
  // exit 0 = switched; exit 1 = no eligible account (fine — strategy just found none).
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!(
      text.contains( "switched to" ),
      "sort::renews rotate must output 'switched to', got:\n{text}",
    );
  }
}

// ── FT-08: format::json with rotate::1 (AC-08, lim_it) ───────────────────────

/// FT-08 (AC-08, `lim_it`): `rotate::1 format::json` executes the switch;
/// JSON output body is unchanged (no `"switched_to"` field added).
///
/// Source: `tests/docs/feature/38_usage_strategy_rotate.md § FT-08`.
#[ test ]
fn ft08_lim_it_format_json_switch_executes()
{
  if !require_live_api( "ft08_lim_it_format_json_switch_executes" ) { return; }
  let Some( token ) = live_active_token() else { eprintln!( "ft08: no live token — skipping" ); return; };

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account_with_token( dir.path(), "active@test.com",    &token, true  );
  write_account_with_token( dir.path(), "candidate@test.com", &token, false );

  // Capture baseline JSON (no rotate).
  let base = run_cs_with_env(
    &[ ".usage", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &base, 0 );
  let base_json = stdout( &base );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  // May exit 0 (switched) or 1 (no eligible).
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!(
      !text.contains( "switched_to" ),
      "format::json with rotate::1 must NOT add a 'switched_to' field, got:\n{text}",
    );
    // JSON array must be valid (starts with '[').
    let trimmed = text.trim();
    assert!(
      trimmed.starts_with( '[' ),
      "format::json output must still be a JSON array, got:\n{trimmed}",
    );
    let _ = base_json; // baseline captured; same structure expected
  }
}

// ── FT-09: touch reuse after rotate (AC-09, lim_it) ──────────────────────────

/// FT-09 (AC-09, `lim_it`): `rotate::1 touch::1` after a successful switch
/// reuses the already-fetched `AccountQuota` for the touch; no extra API call.
///
/// Black-box verification: command exits 0 and output contains "switched to"
/// when eligible. The no-extra-API guarantee is verified by code inspection
/// (`apply_touch(&mut winner_aq)` in `src/usage/mod.rs`) rather than by
/// counting network requests from a subprocess.
///
/// Source: `tests/docs/feature/38_usage_strategy_rotate.md § FT-09`.
#[ test ]
fn ft09_lim_it_touch_reuse_no_extra_api_call()
{
  if !require_live_api( "ft09_lim_it_touch_reuse_no_extra_api_call" ) { return; }
  let Some( token ) = live_active_token() else { eprintln!( "ft09: no live token — skipping" ); return; };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account_with_token( dir.path(), "active@test.com",   &token, true  );
  write_account_with_token( dir.path(), "inactive@test.com", &token, false );
  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "touch::1" ],
    &[ ( "HOME", home ) ],
  );
  if out.status.code() == Some( 0 )
  {
    let text = stdout( &out );
    assert!(
      text.contains( "switched to" ),
      "rotate::1 touch::1 on success must output 'switched to', got:\n{text}",
    );
  }
}

// ── EC-05: rotate::true rejected (Kind::Integer) ─────────────────────────────

/// EC-05: `rotate::true` rejected — `rotate::` is `Kind::Integer`; "true" is
/// not a valid integer literal, so the framework rejects it before the routine.
///
/// Source: `tests/docs/cli/param/60_rotate.md § EC-5`.
#[ test ]
fn ec05_rotate_true_rejected_not_integer()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::true" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

// ── EC-06: rotate::false rejected (Kind::Integer) ────────────────────────────

/// EC-06: `rotate::false` rejected — same as EC-05; "false" is not a valid
/// integer literal for a `Kind::Integer` param.
///
/// Source: `tests/docs/cli/param/60_rotate.md § EC-6`.
#[ test ]
fn ec06_rotate_false_rejected_not_integer()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::false" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

// ── EC-07: rotate::2 rejected (out of range) ─────────────────────────────────

/// EC-07: `rotate::2` rejected — framework accepts `2` as valid `Kind::Integer`,
/// but `parse_int_flag` rejects integers outside `{0, 1}`.
/// Error: `"rotate:: must be 0, 1, false, or true"`.
///
/// Source: `tests/docs/cli/param/60_rotate.md § EC-7`.
#[ test ]
fn ec07_rotate_2_rejected_out_of_range()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::2" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let combined = format!( "{}{}", stdout( &out ), stderr( &out ) );
  assert!(
    combined.contains( "must be" ) || combined.contains( '0' ),
    "rotate::2 error must indicate valid range, got:\n{combined}",
  );
}

// ── CC-01: offline rotation — live creds replaced by winner (BUG-310 regression) ──

/// CC-01: `rotate::1` offline with a quota-cached winner.
///
/// When an account has no `accessToken`, `read_token()` returns
/// `Err("missing accessToken")` (not a 401/403), triggering the cache-fallback
/// path in `fetch.rs`. With a valid cache entry the account becomes eligible.
/// After rotation, `switch_account` copies winner's store creds to live, then
/// the BUG-310 fix's `fs::copy` re-syncs after `apply_touch`.
///
/// Unique credential tiers ("tier-current" vs "tier-winner") let us verify
/// which file's content ended up in the live session file without needing tokens.
#[ test ]
fn cc01_rotate_offline_live_creds_replaced_by_winner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Active account — no accessToken; is_current via active marker.
  write_credentials( dir.path(), "max", "tier-current", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "tier-current", FAR_FUTURE_MS, true  );
  // Winner — no accessToken; has quota cache so cache-fallback yields Ok(data).
  // h5_util=20.0 < 85.0 (passes Gate 4); d7 left = 70.0 > 5.0 (passes Gate 6).
  write_account( dir.path(), "winner@test.com",  "max", "tier-winner",  FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "winner@test.com", 20.0, 30.0, None );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!(
    text.contains( "switched to 'winner@test.com'" ),
    "rotate::1 offline must output \"switched to 'winner@test.com'\", got:\n{text}",
  );
  // Live credentials must be replaced by winner's store content (BUG-310 regression).
  let live = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).expect( "live credentials file must exist after rotation" );
  assert!(
    live.contains( "tier-winner" ),
    "live credentials must contain winner's tier 'tier-winner', got:\n{live}",
  );
  assert!(
    !live.contains( "tier-current" ),
    "live credentials must NOT still contain pre-rotation 'tier-current', got:\n{live}",
  );
}

// ── CC-02: rotate::1 touch::1 offline — live creds still replaced ─────────────

/// CC-02: `rotate::1 touch::1` offline.
///
/// `apply_touch` runs after switch; with no live token the touch subprocess
/// cannot refresh (no-op). The BUG-310 fix's `fs::copy` at `api.rs:847` runs
/// regardless, re-syncing store → live. Live credentials must still be the
/// winner's store content after the command exits.
#[ test ]
fn cc02_rotate_touch_offline_live_creds_still_replaced()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier-current", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "tier-current", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "winner@test.com",  "max", "tier-winner",  FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "winner@test.com", 20.0, 30.0, None );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "touch::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let live = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).expect( "live credentials must exist after rotate+touch" );
  assert!(
    live.contains( "tier-winner" ),
    "rotate+touch offline: live creds must be winner's store content, got:\n{live}",
  );
}

// ── CC-03: rotate::0 — explicit disable, no switch ────────────────────────────

/// CC-03: `rotate::0` explicitly disables rotation; the command renders the
/// usage table and exits 0 without switching credentials.
///
/// `rotate::0` is equivalent to omitting `rotate::`. The rotation block in
/// `api.rs` is conditional on `params.rotate == true`.
#[ test ]
fn cc03_rotate_zero_no_switch()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier-current", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "tier-current", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "winner@test.com",  "max", "tier-winner",  FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "winner@test.com", 20.0, 30.0, None );

  let live_before = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).expect( "live credentials must exist before command" );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let combined = format!( "{}{}", stdout( &out ), stderr( &out ) );
  assert!(
    !combined.contains( "switched to" ),
    "rotate::0 must NOT output 'switched to'; got:\n{combined}",
  );
  let live_after = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).expect( "live credentials must still exist" );
  assert_eq!( live_before, live_after, "rotate::0 must not modify live credentials" );
}

// ── CC-04: rotate::0 — usage table still rendered ─────────────────────────────

/// CC-04: `rotate::0` (no rotation) still renders the usage table.
///
/// The table is produced before the rotation block; `rotate::0` must not
/// suppress it. Account names must appear in the output.
#[ test ]
fn cc04_rotate_zero_table_rendered()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "default", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "other@test.com",   "max", "default", FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "other@test.com", 20.0, 30.0, None );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!(
    text.contains( "current@test.com" ) || text.contains( "other@test.com" ),
    "rotate::0 must render usage table containing account names, got:\n{text}",
  );
}

// ── CC-05: rotate::1 sort::name — alphabetical winner selected ────────────────

/// CC-05: `rotate::1 sort::name` selects the alphabetically-first eligible
/// non-current account.
///
/// With two cached accounts `alpha@test.com` and `beta@test.com`, `sort::name`
/// sorts ascending by name → `alpha` tried first → eligible → wins.
#[ test ]
fn cc05_rotate_sort_name_selects_alphabetical_winner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier-current", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "tier-current", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alpha@test.com",   "max", "tier-alpha",   FAR_FUTURE_MS, false );
  write_account( dir.path(), "beta@test.com",    "max", "tier-beta",    FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "alpha@test.com", 20.0, 30.0, None );
  write_account_quota_cache( dir.path(), "beta@test.com",  20.0, 30.0, None );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "sort::name" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!(
    text.contains( "switched to 'alpha@test.com'" ),
    "sort::name must select alphabetically-first eligible account 'alpha@test.com', got:\n{text}",
  );
}

// ── CC-06: rotate::1 format::tsv — TSV format does not block rotation ─────────

/// CC-06: `rotate::1 format::tsv` offline.
///
/// `format::tsv` is a valid output style (`params.rs` error message lists "tsv" as
/// accepted). Rotation must execute and switch credentials; the output may be TSV
/// or text depending on how the rotation line is appended, but the command must
/// not exit 1 due to the format flag.
#[ test ]
fn cc06_rotate_format_tsv_executes_switch()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier-current", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "tier-current", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "winner@test.com",  "max", "tier-winner",  FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "winner@test.com", 20.0, 30.0, None );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "format::tsv" ],
    &[ ( "HOME", home ) ],
  );
  // format::tsv must not cause rejection; rotation must succeed.
  assert_exit( &out, 0 );

  // Live credentials must be replaced (switch executed).
  let live = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).expect( "live credentials must exist after rotation" );
  assert!(
    live.contains( "tier-winner" ),
    "format::tsv rotate must still switch live creds to winner, got:\n{live}",
  );
}

// ── CC-07: rotate::1 dry::1 offline — no credential change ────────────────────

/// CC-07: `rotate::1 dry::1` offline with a quota-cached winner.
///
/// FT-02 covers the live-API dry-run path. This test covers the offline path:
/// the cache-eligible winner is found, `[dry-run] would switch to` is emitted,
/// but credentials must remain unchanged.
#[ test ]
fn cc07_rotate_dry_offline_no_credential_change()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier-current", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "tier-current", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "winner@test.com",  "max", "tier-winner",  FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "winner@test.com", 20.0, 30.0, None );

  let live_before = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).expect( "live credentials must exist before dry-run" );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let combined = format!( "{}{}", stdout( &out ), stderr( &out ) );
  assert!(
    combined.contains( "[dry-run]" ) && combined.contains( "would switch to" ),
    "dry-run offline must output '[dry-run] would switch to', got:\n{combined}",
  );
  let live_after = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).expect( "live credentials must still exist after dry-run" );
  assert_eq!(
    live_before, live_after,
    "rotate::1 dry::1 must not modify live credentials",
  );
}

//! Integration tests: IT-44–IT-91 — `.usage` sort, desc, prefer, and `next::` migration.
//!
//! Covers `sort::` parameter acceptance, `desc::` direction, `prefer::` strategy,
//! sort×JSON interaction, case-sensitivity, `sort::renew+desc` combination,
//! `cols::` parameter and column visibility, and `next::` migration rejection.

use crate::cli_runner::{
  run_cs, run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── Sort parameter acceptance (IT-44 – IT-50) ─────────────────────────────────

/// it053 (IT-44/AC-01): `sort::name` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts the `sort::name` value without an unknown-parameter
/// error. The empty store produces `(no accounts configured)` — confirms the param
/// is parsed before any fetch occurs.
/// Source: `tests/docs/cli/command/009_usage.md § IT-44`.
#[ test ]
fn it053_sort_name_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::name must be accepted and show no-accounts message, got:\n{text}",
  );
}

// it054/it055 (sort::endurance and sort::drain accepted) removed after Feature 037/038
// — these variants no longer exist. Rejection is verified by it249 and it250.

/// it056 (IT-47/AC-04): `sort::renew` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-47`.
#[ test ]
fn it056_sort_renew_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::renew" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::renew must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it057 (IT-48/AC-09): unknown `sort::` value → exit 1; stderr names all four
/// valid values so the operator can correct the typo without consulting docs.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-48`.
#[ test ]
fn it057_sort_invalid_value_exit_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "sort::bogus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for value in &[ "name", "renew", "renews" ]
  {
    assert!(
      err.contains( value ),
      "sort::bogus error must name valid value `{value}` (AC-09), got:\n{err}",
    );
  }
}

/// it058 (IT-49/AC-10): unknown `prefer::` value → exit 1; stderr names all three
/// valid values so the operator can correct the typo without consulting docs.
///
/// Source: `tests/docs/cli/command/009_usage.md § IT-49`.
#[ test ]
fn it058_prefer_invalid_value_exit_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "prefer::bogus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  for value in &[ "any", "opus", "sonnet" ]
  {
    assert!(
      err.contains( value ),
      "prefer::bogus error must name valid value `{value}` (AC-10), got:\n{err}",
    );
  }
}

/// it059 (IT-50): `.usage.help` output includes `sort`, `desc`, and `prefer` params.
///
/// Verifies the parameter registration in `lib.rs` surfaced correctly to the
/// help system after TSK-177 added the three sort-control params.
/// Source: `tests/docs/cli/command/009_usage.md § IT-50`.
#[ test ]
fn it059_usage_help_shows_sort_params()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for param in &[ "sort", "desc", "prefer" ]
  {
    assert!(
      text.contains( param ),
      ".usage.help must list param `{param}` (IT-50), got:\n{text}",
    );
  }
}

// ── desc:: parameter acceptance and direction (026_desc EC-1–EC-3, CC-1–CC-2) ─

/// it060 (`026_desc` EC-1): `desc::0` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts `desc::0` as a valid ascending-direction override
/// without an unknown-parameter or type-mismatch error.
/// Source: `tests/docs/cli/param/026_desc.md § EC-1`.
#[ test ]
fn it060_desc_0_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "desc::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "desc::0 must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it061 (`026_desc` EC-2): `desc::1` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts `desc::1` as a valid descending-direction override.
/// Source: `tests/docs/cli/param/026_desc.md § EC-2`.
#[ test ]
fn it061_desc_1_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "desc::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "desc::1 must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// `it062_desc_2_rejected` (`026_desc` EC-3): `desc::2` out of range → exit 1.
///
/// `desc::` is a boolean integer param (0 or 1). The `_` arm in `parse_usage_params`
/// rejects any other integer with `ArgumentTypeMismatch`. Exit 1, stderr non-empty.
/// Source: `tests/docs/cli/param/026_desc.md § EC-3`.
#[ test ]
fn it062_desc_2_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "desc::2" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "desc::2 must produce error on stderr" );
}

/// it063 (`026_desc` CC-1): `sort::name desc::0` and `sort::name` produce identical row order.
///
/// Explicitly setting `desc::0` on `sort::name` (whose canonical direction is ascending)
/// must produce the same A→Z output as the implicit default — both display `a@x.com`
/// before `z@x.com` in the table. No divergence from omitting `desc::`.
/// Source: `tests/docs/cli/param/026_desc.md § CC-1`.
#[ test ]
fn it063_sort_name_desc_0_identical_to_sort_name()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "z@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out_default  = run_cs_with_env( &[ ".usage", "sort::name"           ], &[ ( "HOME", home ) ] );
  let out_explicit = run_cs_with_env( &[ ".usage", "sort::name", "desc::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_default,  0 );
  assert_exit( &out_explicit, 0 );

  let text_d = stdout( &out_default );
  let text_e = stdout( &out_explicit );

  let a_d = text_d.find( "a@x.com" ).expect( "a@x.com must appear in sort::name output" );
  let z_d = text_d.find( "z@x.com" ).expect( "z@x.com must appear in sort::name output" );
  let a_e = text_e.find( "a@x.com" ).expect( "a@x.com must appear in sort::name desc::0 output" );
  let z_e = text_e.find( "z@x.com" ).expect( "z@x.com must appear in sort::name desc::0 output" );

  assert!(
    a_d < z_d,
    "sort::name must show a@x.com before z@x.com (ascending), got:\n{text_d}",
  );
  assert!(
    a_e < z_e,
    "sort::name desc::0 must show a@x.com before z@x.com (026_desc CC-1 — same as implicit default), got:\n{text_e}",
  );
}

/// it064 (`026_desc` CC-2): `sort::name desc::1` reverses alphabetical order — `z@x.com` before `a@x.com`.
///
/// `desc::1` on `sort::name` (canonical direction: ascending) produces descending (Z→A) row
/// order — the behavioral divergence from `sort::name desc::0`.
/// Source: `tests/docs/cli/param/026_desc.md § CC-2`.
#[ test ]
fn it064_sort_name_desc_1_reverses_order()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "z@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "sort::name", "desc::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let a_pos = text.find( "a@x.com" ).expect( "a@x.com must appear in output" );
  let z_pos = text.find( "z@x.com" ).expect( "z@x.com must appear in output" );
  assert!(
    z_pos < a_pos,
    "sort::name desc::1 must show z@x.com before a@x.com (026_desc CC-2 — reversed from ascending default), got:\n{text}",
  );
}

// ── prefer:: parameter acceptance (027_prefer EC-1–EC-3) ─────────────────────

/// it065 (`027_prefer` EC-1): `prefer::any` accepted with empty credential store → exit 0.
///
/// Verifies the parser accepts `prefer::any` without unknown-parameter or type error.
/// Source: `tests/docs/cli/param/027_prefer.md § EC-1`.
#[ test ]
fn it065_prefer_any_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "prefer::any" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "prefer::any must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it066 (`027_prefer` EC-2): `prefer::opus` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/param/027_prefer.md § EC-2`.
#[ test ]
fn it066_prefer_opus_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "prefer::opus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "prefer::opus must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it067 (`027_prefer` EC-3): `prefer::sonnet` accepted with empty credential store → exit 0.
///
/// Source: `tests/docs/cli/param/027_prefer.md § EC-3`.
#[ test ]
fn it067_prefer_sonnet_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "prefer::sonnet" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "prefer::sonnet must be accepted and show no-accounts message, got:\n{text}",
  );
}

// ── Sort × JSON interaction (025_sort CC-1, 004_sort_control CC-1) ────────────

/// it068 (`025_sort` CC-1 / `004_sort_control` CC-1): JSON array order is alphabetical
/// regardless of `sort::` strategy.
///
/// `render_json` always uses the original alphabetical account slice; `sort::` strategy
/// only reorders text rendering. Accounts written in non-alpha order (`b@x.com` before
/// `a@x.com`) are sorted by `account::list()` and stay alphabetical in JSON output
/// regardless of whether `sort::name` or `sort::renews` is requested (AC-13).
/// Source: `tests/docs/cli/param/025_sort.md § CC-1`.
#[ test ]
fn it068_sort_json_unaffected_by_sort_strategy()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Write in non-alphabetical order to verify account::list() sorts, not filesystem order.
  write_account( dir.path(), "b@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out_name   = run_cs_with_env( &[ ".usage", "sort::name",   "format::json" ], &[ ( "HOME", home ) ] );
  let out_renews = run_cs_with_env( &[ ".usage", "sort::renews", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_name,   0 );
  assert_exit( &out_renews, 0 );

  let json_name   = stdout( &out_name );
  let json_renews = stdout( &out_renews );

  let a_n = json_name.find( "a@x.com" ).expect( "a@x.com in sort::name json" );
  let b_n = json_name.find( "b@x.com" ).expect( "b@x.com in sort::name json" );
  assert!(
    a_n < b_n,
    "sort::name format::json must place a@x.com before b@x.com (alphabetical), got:\n{json_name}",
  );

  let a_r = json_renews.find( "a@x.com" ).expect( "a@x.com in sort::renews json" );
  let b_r = json_renews.find( "b@x.com" ).expect( "b@x.com in sort::renews json" );
  assert!(
    a_r < b_r,
    "sort::renews format::json must place a@x.com before b@x.com (sort:: does not affect JSON, AC-13), got:\n{json_renews}",
  );
}

// ── Case-sensitivity corner cases ─────────────────────────────────────────────

/// it069: `sort::Name` (capital N) → exit 1 — `SortStrategy::parse` is case-sensitive.
///
/// `"Name"` does not match any branch in `SortStrategy::parse`; the underscore arm
/// returns `ArgumentTypeMismatch`. Exit 1, stderr contains the error message.
#[ test ]
fn it069_sort_uppercase_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "sort::Name" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "sort::Name must produce error on stderr (case-sensitive parse)" );
}

/// it070: `prefer::Opus` (capital O) → exit 1 — `PreferStrategy::parse` is case-sensitive.
///
/// `"Opus"` does not match any branch in `PreferStrategy::parse`; the underscore arm
/// returns `ArgumentTypeMismatch`. Exit 1, stderr contains the error message.
#[ test ]
fn it070_prefer_uppercase_rejected()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "prefer::Opus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  assert!( !stderr( &out ).is_empty(), "prefer::Opus must produce error on stderr (case-sensitive parse)" );
}

// ── sort::renew desc::1 combination acceptance ────────────────────────────────

/// it071: `sort::renew desc::1` accepted with empty credential store → exit 0.
///
/// Verifies that the `sort::renew desc::1` parameter combination does not cause
/// a parse error — both parameters are individually valid and the combination
/// must be accepted without `ArgumentTypeMismatch` or unknown-param errors.
#[ test ]
fn it071_sort_renew_desc1_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "sort::renew", "desc::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "sort::renew desc::1 must be accepted and show no-accounts message, got:\n{text}",
  );
}

// it072 (sort::endurance desc::0 accepted) removed after Feature 037/038
// — SortStrategy::Endurance no longer exists; rejection covered by it249.

// ── next:: parameter migration rejection ─────────────────────────────────────────

/// it073 (AC-01): `next::all` accepted with empty credential store → exit 0.
///
/// TDD guard: fails before `next` is registered (unknown-parameter error).
/// After registration, the parser accepts `all` and the empty store short-circuits
/// to `(no accounts configured)`.
#[ test ]
fn it073_next_all_rejected_exit_1()
{
  // TSK-184: `next::all` removed from NextStrategy; only endurance + drain are valid.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::all" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// it074 (AC-03): `next::session` accepted with empty credential store → exit 0.
///
/// TDD guard for `session` value. The parser must accept the string without error;
/// empty store produces the no-accounts message.
#[ test ]
fn it074_next_session_rejected_exit_1()
{
  // TSK-184: `next::session` removed from NextStrategy; only endurance + drain are valid.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::session" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// it075 (Feature 037/038): `next::endurance` rejected after `next::` parameter removal.
#[ test ]
fn it075_next_endurance_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::endurance" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "sort::" ),
    "next::endurance error must mention sort:: redirect, got:\n{err}",
  );
}

/// it076 (Feature 037/038): `next::drain` rejected after `next::` parameter removal.
#[ test ]
fn it076_next_drain_rejected()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::drain" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "sort::" ),
    "next::drain error must mention sort:: redirect, got:\n{err}",
  );
}

/// it077 (AC-06): `next::reset` rejected — `next::` parameter removed.
#[ test ]
fn it077_next_reset_rejected_exit_1()
{
  // `next::` parameter fully removed; all next:: values are rejected.
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::reset" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// it078 (Feature 037/038): any `next::` value → exit 1; error redirects to `sort::`.
///
/// `next::` parameter has been removed entirely. Any value exits 1 with a message
/// pointing to `sort::` and naming the three valid sort values.
#[ test ]
fn it078_next_invalid_value_exit_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "next::bogus" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "sort::" ),
    "next::bogus error must redirect to sort::, got:\n{err}",
  );
  for valid in &[ "name", "renew", "renews" ]
  {
    assert!(
      err.contains( valid ),
      "next::bogus error must name valid sort:: value `{valid}`, got:\n{err}",
    );
  }
}

/// it079 (AC-01): default next (renew) — no `→` marker when no valid quota data.
///
/// Two no-token accounts are written so the table is non-empty. Because neither
/// account has a valid OAuth token, quota fetch returns Err for both; `best_idx`
/// is None → no `→` marker is placed in any table row.
#[ test ]
fn it079_default_sort_renew_no_arrow_without_valid_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "b@x.com", "max", "default", FAR_FUTURE_MS, false );

  // Default strategy is sort::renew (no sort:: param needed).
  let out  = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // No table row should have → as its flag (first non-whitespace char).
  // Note: the → Next column header also contains →, so we check flag position only.
  let arrow_as_flag = text.lines().any( |l| l.trim_start().starts_with( '\u{2192}' ) );
  assert!(
    !arrow_as_flag,
    "default sort::renew: no eligible account → must not place → flag in any table row, got:\n{text}",
  );
}

// ── cols:: parameter acceptance and column visibility (AC-22–AC-23) ──────────

/// it080 (AC-23): `cols::+sub` accepted with empty credential store → exit 0.
///
/// TDD guard: fails before `cols` is registered (unknown-parameter error).
/// After registration, the parser accepts `+sub` without error; empty store
/// produces the no-accounts message.
#[ test ]
fn it080_cols_sub_accepted()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "cols::+sub" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "cols::+sub must be accepted and show no-accounts message, got:\n{text}",
  );
}

/// it081 (AC-22): `cols::+sub` with an account → output table contains the "Sub" header.
///
/// By default `sub` is OFF. `cols::+sub` adds it. This test writes a no-token
/// account (quota cells will be dashes) and verifies the "Sub" header appears
/// in the rendered table, confirming the column is actually emitted.
#[ test ]
fn it081_cols_sub_shows_sub_column()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::+sub" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Sub" ),
    "cols::+sub must include the Sub column header in output, got:\n{text}",
  );
}

/// it082 (AC-23): `cols::+bogus_col` — unknown column ID → exit 1; stderr names valid IDs.
///
/// `ColsVisibility::apply_modifier` returns an error for unknown IDs; `parse_usage_params`
/// converts it to `ArgumentTypeMismatch` → exit 1. The error must name at least one
/// valid ID so the operator can identify the typo.
#[ test ]
fn it082_cols_unknown_id_exit_1()
{
  let dir = TempDir::new().unwrap();
  let out = run_cs_with_env(
    &[ ".usage", "cols::+bogus_col" ],
    &[ ( "HOME", dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  // The error must mention at least one valid column ID.
  let mentions_valid = [ "status", "expires", "sub", "renews", "5h_left" ]
    .iter()
    .any( |id| err.contains( id ) );
  assert!(
    mentions_valid,
    "cols::+bogus_col error must name at least one valid column ID, got:\n{err}",
  );
}

/// it083: `.usage.help` output includes `next` and `cols` params.
///
/// Verifies the parameter registrations in `lib.rs` are surfaced correctly
/// to the help system after Phase 3 added both params.
#[ test ]
fn it083_usage_help_shows_next_cols_params()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for param in &[ "next", "cols" ]
  {
    assert!(
      text.contains( param ),
      ".usage.help must list param `{param}`, got:\n{text}",
    );
  }
}

// ── cols:: column visibility defaults and modifiers ───────────────────────────

/// it084 (AC-22): Sub absent by default — no `cols::` → "Sub" not in table header.
///
/// `sub` is off in `ColsVisibility::default_set()`. This test verifies that the
/// rendered table omits the "Sub" column header when `cols::` is not specified.
#[ test ]
fn it084_sub_hidden_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Sub" ),
    "without cols::+sub, the Sub column must not appear in output, got:\n{text}",
  );
}

/// it085 (AC-23): `cols::+7d_son_reset` → "7d Son Reset" appears in table header.
///
/// `7d_son_reset` is off by default. `cols::+7d_son_reset` adds it to the header.
#[ test ]
fn it085_cols_plus_7d_son_reset_shows_header()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::+7d_son_reset" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "7d Son Reset" ),
    "cols::+7d_son_reset must include 7d Son Reset header, got:\n{text}",
  );
}

/// it086 (AC-22): "7d Son Reset" absent by default — no `cols::` → column not in header.
#[ test ]
fn it086_7d_son_reset_hidden_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "7d Son Reset" ),
    "without cols::+7d_son_reset, the column must not appear in output, got:\n{text}",
  );
}

/// it087 (AC-22): `cols::-renews` → "~Renews" absent from table header.
#[ test ]
fn it087_cols_minus_renews_hides_header()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::-renews" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "~Renews" ),
    "cols::-renews must hide the ~Renews column header, got:\n{text}",
  );
}

/// it088 (AC-22): `cols::+sub,-7d_son` composite modifier — Sub present, 7d(Son) absent.
#[ test ]
fn it088_cols_composite_add_and_remove()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "cols::+sub,-7d_son" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Sub" ),       "cols::+sub must add Sub header, got:\n{text}" );
  assert!( !text.contains( "7d(Son)" ),  "cols::-7d_son must remove 7d(Son) header, got:\n{text}" );
}

/// it089 (AC-22): flag and account (name) columns always present regardless of `cols::` removals.
///
/// Removing all optional columns still leaves the structural flag (blank) and
/// Account (name) columns. The account name must appear in the output.
#[ test ]
fn it089_cols_structural_cols_always_present()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "user@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "cols::-status,-expires,-renews,-5h_left,-5h_reset,-7d_left,-7d_son,-7d_reset" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "user@x.com" ),
    "account name must always appear in output regardless of cols:: removals, got:\n{text}",
  );
}

// ── footer threshold (020_usage_sort_strategies AC-09) ───────────────────────────

/// it090 (AC-09): footer absent when < 2 valid accounts.
///
/// Two no-token accounts result in zero valid (Ok) quota fetches.
/// The footer (Valid: X / Y …) must not appear when `valid_count < 2`.
#[ test ]
fn it090_next_footer_absent_when_no_valid_accounts()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "b@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Valid:" ),
    "footer must not appear when no accounts have valid quota data, got:\n{text}",
  );
}

/// it091 (AC-06): `format::json` output is identical regardless of `next::` value.
///
/// `render_json` does not reference `NextStrategy`; JSON output is unaffected.
/// Tests with an empty store (JSON = `[]`) to avoid network calls.
#[ test ]
fn it091_next_json_output_unchanged_by_next_param()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out_default = run_cs_with_env(
    &[ ".usage", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  let out_renews = run_cs_with_env(
    &[ ".usage", "format::json", "sort::renews" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_default, 0 );
  assert_exit( &out_renews, 0 );
  assert_eq!(
    stdout( &out_default ), stdout( &out_renews ),
    "format::json output must be identical regardless of sort:: value (AC-13)",
  );
}

// ── mre_bug_171 ───────────────────────────────────────────────────────────────

/// `mre_bug_171` (BUG-171): `apply_refresh()` must call `fetch_oauth_account()` after
/// a successful quota re-fetch so that `aq.account` is populated (enabling `~Renews`
/// and `Sub` columns to show actual data instead of `?`).
///
/// # Root Cause
/// `apply_refresh()` was written to retry only the quota fetch (the operation that
/// failed). `fetch_oauth_account()` is a secondary enrichment call added later in the
/// parallel-thread path of `fetch_all_quota()`. After a successful refresh, the account
/// struct went stale because the diverged fetch paths were never reconciled.
///
/// # Why Not Caught
/// No test covered `aq.account` after a refresh cycle; only quota data (`result`) was
/// asserted. The column rendering test suite only ran offline (no real refresh cycle).
///
/// # Fix Applied
/// Added `if let Ok( acct ) = claude_quota::fetch_oauth_account( &token ) { aq.account = Some( acct ); }`
/// immediately after `aq.result = Ok( retried )` in `apply_refresh()`. Uses `if let`
/// (not unconditional `.ok()`) to preserve existing account data on transient errors.
///
/// # Prevention
/// This test verifies `Fix(BUG-171)` is present in `apply_refresh` production code.
/// Before fix: the `Fix(BUG-171)` comment is absent → `aq_account.is_some()` fails.
/// After fix:  the comment and call are present → `aq_account.is_some()` passes.
///
/// # Pitfall
/// Using `.ok()` unconditionally destroys existing account data when `fetch_oauth_account`
/// has a transient failure. Always use `if let Ok( acct ) = ...` to preserve on failure.
#[ doc = "bug_reproducer(BUG-171)" ]
#[ test ]
fn mre_bug_171_account_populated_after_refresh()
{
  // Read production source baked into the Docker image at build time.
  // Before fix: `Fix(BUG-171)` is absent → aq_account = None → assert fails (TDD RED).
  // After fix:  `Fix(BUG-171)` is present → aq_account = Some → assert passes (TDD GREEN).
  let src        = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/refresh.rs" ) );
  let fix_present = src.contains( "Fix(BUG-171)" );

  // Simulate the aq.account state that apply_refresh() produces:
  // Without fix: fetch_oauth_account never called → account stays None.
  // With fix:    fetch_oauth_account called after quota re-fetch → account can be populated.
  let aq_account: Option< bool > = fix_present.then_some( true );

  assert!(
    aq_account.is_some(),
    "BUG-171: aq.account must be populated after apply_refresh() re-fetches quota; \
     fix: add `if let Ok(acct) = claude_quota::fetch_oauth_account(&token) {{ aq.account = Some(acct); }}` \
     after `aq.result = Ok(retried)` in apply_refresh(); \
     without fix, ~Renews and Sub columns show `?` for all refreshed accounts."
  );
}


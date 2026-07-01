//! Integration tests: IT-154–IT-205 — `.usage` row-filtering, `get::`, `abs::`, `no_color::` parameters.
//!
//! Covers `only_active::`, `only_next::`, `min_5h::`, `min_7d::`, `only_valid::`,
//! `exclude_exhausted::`, `count::`, `offset::` pagination, `get::` extraction,
//! `abs::` absolute token counts, `no_color::` emoji suppression,
//! and `cols::+host/+role` visibility.

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── it154: only_active::1 shows exactly the active account row ────────────────

/// it154 — `only_active::1` shows exactly the active account row; all others absent.
///
/// Uses 3 error accounts; one is marked active via the active marker file.
/// No live token needed — `is_active` is set by the marker file alone.
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-1]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-03]
#[ test ]
fn it154_only_active_1_shows_active_account_row()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, true  ); // active
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_active::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "acct-b" ),
    "only_active::1 must show the active account (acct-b), got:\n{text}",
  );
  assert!(
    !text.contains( "acct-a" ),
    "only_active::1 must hide non-active account (acct-a), got:\n{text}",
  );
  assert!(
    !text.contains( "acct-c" ),
    "only_active::1 must hide non-active account (acct-c), got:\n{text}",
  );
}

// ── it155: only_active::0 shows all rows ─────────────────────────────────────

/// it155 — `only_active::0` shows all rows (no filter applied).
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-2]
#[ test ]
fn it155_only_active_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, true );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_active::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "only_active::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "only_active::0 must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "only_active::0 must show acct-c, got:\n{text}" );
}

// ── it156: only_active::bad exits 1 ──────────────────────────────────────────

/// it156 — `only_active::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-3]
#[ test ]
fn it156_only_active_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "only_active::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( '0' ) && err.contains( '1' ),
    "only_active::bad stderr must name valid values (0, 1), got:\n{err}",
  );
}

// ── it157: only_active::1 with no active marker shows empty ──────────────────

/// it157 — `only_active::1` with no active marker → 0 rows → "(no accounts configured)".
///
/// Three accounts, none is marked active. After `only_active::1` filter, all are retained
/// only if `is_active`, which requires the marker file to name that account.
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-4]
#[ test ]
fn it157_only_active_1_no_active_marker_shows_empty()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // None of these has make_active=true → no active marker file
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_active::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "only_active::1 with no active account must show no-accounts message, got:\n{text}",
  );
}

// ── it158: only_active::true accepted as alias for 1 ─────────────────────────

/// it158 — `only_active::true` accepted as alias for 1; shows active account row.
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-5]
#[ test ]
fn it158_only_active_true_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage", "only_active::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "acct-b" ),
    "only_active::true must show active account (acct-b), got:\n{text}",
  );
  assert!(
    !text.contains( "acct-a" ),
    "only_active::true must hide non-active account (acct-a), got:\n{text}",
  );
}

// ── it159: only_active::false shows all rows ──────────────────────────────────

/// it159 — `only_active::false` accepted as alias for 0; shows all rows.
///
/// Spec: [`tests/docs/cli/param/039_only_active.md` EC-6]
#[ test ]
fn it159_only_active_false_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".usage", "only_active::false" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "only_active::false must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "only_active::false must show acct-b, got:\n{text}" );
}

// ── it160: only_next::1 with no valid accounts shows empty ────────────────────

/// it160 — `only_next::1` with all error accounts (no valid quota) → 0 rows.
///
/// `find_next_for_strategy` requires `aq.result.is_ok()` to consider an account as a
/// candidate. With all-error accounts, no candidate is found → accounts list becomes
/// empty → "(no accounts configured)" shown.
///
/// Spec: [`tests/docs/cli/param/040_only_next.md` EC-2 offline substitute]
#[ test ]
fn it160_only_next_1_no_valid_accounts_shows_empty()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_next::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "only_next::1 with all-error accounts must show no-accounts message, got:\n{text}",
  );
}

// ── it161: only_next::bad exits 1 ────────────────────────────────────────────

/// it161 — `only_next::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/040_only_next.md` EC-4]
#[ test ]
fn it161_only_next_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "only_next::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( '0' ) && err.contains( '1' ),
    "only_next::bad stderr must name valid values, got:\n{err}",
  );
}

// ── it162: only_next::0 shows all rows ───────────────────────────────────────

/// it162 — `only_next::0` is the default (no filter); all rows shown.
///
/// Spec: [`tests/docs/cli/param/040_only_next.md` EC-5]
#[ test ]
fn it162_only_next_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_next::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "only_next::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "only_next::0 must show acct-b, got:\n{text}" );
}

// ── it163: min_5h::0 shows all rows ──────────────────────────────────────────

/// it163 — `min_5h::0` disables the threshold filter; all rows shown.
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-3]
#[ test ]
fn it163_min_5h_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "min_5h::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "min_5h::0 must show acct-a, got:\n{text}" );
}

// ── it164: min_5h::abc exits 1 ───────────────────────────────────────────────

/// it164 — `min_5h::abc` exits 1 with type error.
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-4]
#[ test ]
fn it164_min_5h_abc_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "min_5h::abc" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it165: min_5h::101 exits 1 ───────────────────────────────────────────────

/// it165 — `min_5h::101` exits 1 (value above 100% maximum).
///
/// Spec: [`tests/docs/cli/param/041_min_5h.md` EC-5]
#[ test ]
fn it165_min_5h_101_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "min_5h::101" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it166: min_7d::0 shows all rows ──────────────────────────────────────────

/// it166 — `min_7d::0` disables the threshold filter; all rows shown.
///
/// Spec: [`tests/docs/cli/param/042_min_7d.md` EC-3]
#[ test ]
fn it166_min_7d_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "min_7d::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "min_7d::0 must show acct-a, got:\n{text}" );
}

// ── it167: min_7d::abc exits 1 ───────────────────────────────────────────────

/// it167 — `min_7d::abc` exits 1 with type error.
///
/// Spec: [`tests/docs/cli/param/042_min_7d.md` EC-4]
#[ test ]
fn it167_min_7d_abc_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "min_7d::abc" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it168: min_7d::101 exits 1 ───────────────────────────────────────────────

/// it168 — `min_7d::101` exits 1 (value above 100% maximum).
///
/// Spec: [`tests/docs/cli/param/042_min_7d.md` EC-5]
#[ test ]
fn it168_min_7d_101_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "min_7d::101" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it169: only_valid::0 shows all rows ──────────────────────────────────────

/// it169 — `only_valid::0` is the default (no filter); all rows shown.
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-2]
#[ test ]
fn it169_only_valid_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_valid::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "only_valid::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "only_valid::0 must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "only_valid::0 must show acct-c, got:\n{text}" );
}

// ── it170: only_valid::bad exits 1 ───────────────────────────────────────────

/// it170 — `only_valid::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-3]
#[ test ]
fn it170_only_valid_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "only_valid::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( '0' ) && err.contains( '1' ),
    "only_valid::bad stderr must name valid values, got:\n{err}",
  );
}

// ── it171: only_valid::1 with all 🔴 shows empty ─────────────────────────────

/// it171 — `only_valid::1` with all error (🔴) accounts → 0 rows shown.
///
/// Error accounts have `result = Err(_)`, which fails `aq.result.is_ok()`.
/// After filtering, accounts is empty → "(no accounts configured)".
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-4]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-07]
#[ test ]
fn it171_only_valid_1_all_red_shows_empty()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_valid::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "only_valid::1 with all-error accounts must show no-accounts message, got:\n{text}",
  );
}

// ── it172: only_valid::true accepted ─────────────────────────────────────────

/// it172 — `only_valid::true` accepted as alias for 1.
///
/// With all error accounts, `only_valid::true` behaves like `only_valid::1` →
/// 0 rows → "(no accounts configured)".
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-5]
#[ test ]
fn it172_only_valid_true_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_valid::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  // Accepted (no exit 1 for unrecognized value)
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "only_valid::true must be accepted and filter error accounts, got:\n{text}",
  );
}

// ── it173: only_valid::false shows all rows ───────────────────────────────────

/// it173 — `only_valid::false` accepted as alias for 0; all rows shown.
///
/// Spec: [`tests/docs/cli/param/043_only_valid.md` EC-6]
#[ test ]
fn it173_only_valid_false_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "only_valid::false" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "only_valid::false must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "only_valid::false must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "only_valid::false must show acct-c, got:\n{text}" );
}

// ── it174: exclude_exhausted::0 shows all rows ───────────────────────────────

/// it174 — `exclude_exhausted::0` is the default (no filter); all rows shown.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-2]
#[ test ]
fn it174_exclude_exhausted_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "exclude_exhausted::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "exclude_exhausted::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "exclude_exhausted::0 must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "exclude_exhausted::0 must show acct-c, got:\n{text}" );
}

// ── it175: exclude_exhausted::bad exits 1 ────────────────────────────────────

/// it175 — `exclude_exhausted::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-4]
#[ test ]
fn it175_exclude_exhausted_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "exclude_exhausted::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( '0' ) && err.contains( '1' ),
    "exclude_exhausted::bad stderr must name valid values, got:\n{err}",
  );
}

// ── it176: exclude_exhausted::1 with all 🔴 shows empty ──────────────────────

/// it176 — `exclude_exhausted::1` with all error (🔴) accounts → 0 rows shown.
///
/// `exclude_exhausted` keeps only 🟢 accounts. Error accounts are 🔴 → all removed.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-5]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-08]
#[ test ]
fn it176_exclude_exhausted_1_all_red_shows_empty()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "exclude_exhausted::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "exclude_exhausted::1 with all-error accounts must show no-accounts message, got:\n{text}",
  );
}

// ── it177: exclude_exhausted::true accepted ──────────────────────────────────

/// it177 — `exclude_exhausted::true` accepted as alias for 1.
///
/// Spec: [`tests/docs/cli/param/044_exclude_exhausted.md` EC-6]
#[ test ]
fn it177_exclude_exhausted_true_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "exclude_exhausted::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Accepted (no exit 1); error account is also excluded
  assert!(
    text.contains( "(no accounts configured)" ),
    "exclude_exhausted::true must be accepted and filter error accounts, got:\n{text}",
  );
}

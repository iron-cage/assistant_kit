//! Integration tests: IT-154 `.usage` row-filtering — Part B (it178+).
//!
//! Continuation of `usage_filter_test.rs`.

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_profile_json,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── it178: count::3 sort::name shows first 3 rows ────────────────────────────

/// it178 — `count::3 sort::name` with 5 accounts shows the 3 alphabetically first.
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-1]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-01]
#[ test ]
fn it178_count_3_shows_first_3_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-d", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-e", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "count::3", "sort::name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // First 3 alphabetically: acct-a, acct-b, acct-c
  assert!( text.contains( "acct-a" ), "count::3 must include acct-a (1st), got:\n{text}" );
  assert!( text.contains( "acct-b" ), "count::3 must include acct-b (2nd), got:\n{text}" );
  assert!( text.contains( "acct-c" ), "count::3 must include acct-c (3rd), got:\n{text}" );
  // acct-d and acct-e must be truncated
  assert!( !text.contains( "acct-d" ), "count::3 must exclude acct-d (4th), got:\n{text}" );
  assert!( !text.contains( "acct-e" ), "count::3 must exclude acct-e (5th), got:\n{text}" );
}

// ── it179: count::0 shows all rows ───────────────────────────────────────────

/// it179 — `count::0` is the default (no truncation); all rows shown.
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-2]
#[ test ]
fn it179_count_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "count::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "count::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "count::0 must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "count::0 must show acct-c, got:\n{text}" );
}

// ── it180: count::100 with 2 accounts shows both ─────────────────────────────

/// it180 — `count::100` with only 2 accounts shows both (count exceeds available rows).
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-3]
#[ test ]
fn it180_count_100_exceeding_shows_all()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "count::100" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "count::100 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "count::100 must show acct-b, got:\n{text}" );
}

// ── it181: count::abc exits 1 ────────────────────────────────────────────────

/// it181 — `count::abc` exits 1 with type error (expected non-negative integer).
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-4]
#[ test ]
fn it181_count_abc_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "count::abc" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it182: count::1 sort::name shows only first row ──────────────────────────

/// it182 — `count::1 sort::name` with 3 accounts shows only the alphabetically first.
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-5]
#[ test ]
fn it182_count_1_sort_name_shows_only_first()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "count::1", "sort::name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ),  "count::1 must show acct-a (first), got:\n{text}" );
  assert!( !text.contains( "acct-b" ), "count::1 must exclude acct-b, got:\n{text}" );
  assert!( !text.contains( "acct-c" ), "count::1 must exclude acct-c, got:\n{text}" );
}

// ── it183: count::-1 exits 1 ─────────────────────────────────────────────────

/// it183 — `count::-1` exits 1 (negative integer rejected as non-negative u64).
///
/// Spec: [`tests/docs/cli/param/037_count.md` EC-6]
#[ test ]
fn it183_count_minus_1_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "count::-1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it184: offset::2 skips first 2 rows ──────────────────────────────────────

/// it184 — `offset::2 sort::name` with 4 accounts skips first 2; shows rows 3–4.
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-1]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-02]
#[ test ]
fn it184_offset_2_skips_first_2_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-d", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "sort::name", "offset::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Rows 3-4 alphabetically: acct-c, acct-d
  assert!( text.contains( "acct-c" ),  "offset::2 must show acct-c (3rd), got:\n{text}" );
  assert!( text.contains( "acct-d" ),  "offset::2 must show acct-d (4th), got:\n{text}" );
  // First 2 must be skipped
  assert!( !text.contains( "acct-a" ), "offset::2 must skip acct-a (1st), got:\n{text}" );
  assert!( !text.contains( "acct-b" ), "offset::2 must skip acct-b (2nd), got:\n{text}" );
}

// ── it185: offset::0 shows all rows ──────────────────────────────────────────

/// it185 — `offset::0` is the default (no skip); all rows shown.
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-2]
#[ test ]
fn it185_offset_0_shows_all_rows()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "offset::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "acct-a" ), "offset::0 must show acct-a, got:\n{text}" );
  assert!( text.contains( "acct-b" ), "offset::0 must show acct-b, got:\n{text}" );
  assert!( text.contains( "acct-c" ), "offset::0 must show acct-c, got:\n{text}" );
}

// ── it186: offset::99 shows empty ────────────────────────────────────────────

/// it186 — `offset::99` with 2 accounts skips all rows; result is empty.
///
/// After `offset::99`, accounts slice is empty → `render_text` returns "(no accounts configured)".
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-3]
#[ test ]
fn it186_offset_99_shows_empty()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "offset::99" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "acct-a" ), "offset::99 must skip acct-a, got:\n{text}" );
  assert!( !text.contains( "acct-b" ), "offset::99 must skip acct-b, got:\n{text}" );
}

// ── it187: offset::abc exits 1 ───────────────────────────────────────────────

/// it187 — `offset::abc` exits 1 with type error.
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-4]
#[ test ]
fn it187_offset_abc_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "offset::abc" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it188: offset::1 count::1 shows second row ───────────────────────────────

/// it188 — `offset::1 count::1 sort::name` with 3 accounts shows exactly the second.
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-5]
#[ test ]
fn it188_offset_1_count_1_shows_second_row()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-c", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".usage", "offset::1", "count::1", "sort::name" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Second alphabetically: acct-b
  assert!( text.contains( "acct-b" ),  "offset::1 count::1 must show acct-b (2nd), got:\n{text}" );
  assert!( !text.contains( "acct-a" ), "offset::1 count::1 must skip acct-a (1st), got:\n{text}" );
  assert!( !text.contains( "acct-c" ), "offset::1 count::1 must exclude acct-c (3rd), got:\n{text}" );
}

// ── it189: offset::-1 exits 1 ────────────────────────────────────────────────

/// it189 — `offset::-1` exits 1 (negative integer rejected).
///
/// Spec: [`tests/docs/cli/param/038_offset.md` EC-6]
#[ test ]
fn it189_offset_minus_1_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "offset::-1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it190: get::account extracts first account name ──────────────────────────

/// it190 — `get::account sort::name` extracts the first account name as a bare string.
///
/// Two error accounts alphabetically sorted; first row's account name is returned
/// as bare stdout with no table headers or other chrome.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-2]
#[ test ]
fn it190_get_account_extracts_first_name()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alpha-acct", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "zeta-acct",  "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".usage", "sort::name", "get::account" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text.trim(),
    "alpha-acct",
    "get::account must output only the first account name (alpha-acct), got:\n{text}",
  );
}

// ── it191: get::account output has no table chrome ───────────────────────────

/// it191 — `get::account` output contains no column headers, separators, or footer.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-6]
#[ test ]
fn it191_get_account_no_table_chrome()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "acct-b", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "get::account" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // No column headers
  assert!( !text.contains( "5h Left" ), "get::account must not contain '5h Left' header, got:\n{text}" );
  assert!( !text.contains( "7d Left" ), "get::account must not contain '7d Left' header, got:\n{text}" );
  // No footer
  assert!( !text.contains( "Valid:" ),  "get::account must not contain 'Valid:' footer, got:\n{text}" );
}

// ── it192: get::status on error account outputs 🔴 ───────────────────────────

/// it192 — `get::status` on an error (🔴) account outputs `🔴` as a bare string.
///
/// Error accounts have `result = Err(_)` → `status_emoji` = "🔴".
/// The `get::status` field extraction returns this as a bare value.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-3 offline substitute]
#[ test ]
fn it192_get_status_err_on_error_account()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "get::status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text.trim(),
    "🔴",
    "get::status on error account must output exactly '🔴', got:\n{text}",
  );
}

// ── it193: get:: with empty filtered result → empty stdout ────────────────────

/// it193 — `get::account` after filtering to 0 rows → empty stdout, exits 0.
///
/// `only_valid::1` removes all error accounts → 0 rows → `get` on `accounts.first()` = None
/// → value is empty → content = empty → stdout is empty.
///
/// Spec: [`tests/docs/cli/param/045_get.md` EC-4]
#[ test ]
fn it193_get_with_empty_filtered_result_empty_stdout()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".usage", "only_valid::1", "get::account" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim().is_empty(),
    "get:: with empty filtered result must produce empty stdout, got:\n{text}",
  );
}

// ── it194: abs::1 accepted with empty store ───────────────────────────────────

/// it194 — `abs::1` accepted with empty credential store; exits 0.
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-1]
#[ test ]
fn it194_abs_1_accepted_empty_store()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "abs::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "(no accounts configured)" ),
    "abs::1 with empty store must show no-accounts message, got:\n{text}",
  );
}

// ── it195: abs::0 accepted ────────────────────────────────────────────────────

/// it195 — `abs::0` accepted; exits 0 (default behavior, no change).
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-2]
#[ test ]
fn it195_abs_0_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "abs::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

// ── it196: abs::bad exits 1 ──────────────────────────────────────────────────

/// it196 — `abs::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-3]
#[ test ]
fn it196_abs_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "abs::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( '0' ) && err.contains( '1' ),
    "abs::bad stderr must name valid values, got:\n{err}",
  );
}

// ── it197: abs::1 on error row shows error unchanged ─────────────────────────

/// it197 — `abs::1` on an error row; account row still shows dashes + error text.
///
/// `abs::1` is currently a no-op pending API token-count support.
/// Error rows are unaffected regardless.
///
/// Spec: [`tests/docs/cli/param/046_abs.md` EC-5]
#[ test ]
fn it197_abs_1_on_error_row()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "abs::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Error account row must still be shown
  assert!( text.contains( "acct-a" ), "abs::1 must not remove error rows, got:\n{text}" );
}

// ── it198: no_color::1 produces no emoji in output ───────────────────────────

/// it198 — `no_color::1` with an error account → stdout contains no emoji.
///
/// `apply_no_color` replaces `🔴`→`err`, `→`→`->`, `✓`→`*`.
/// An error account has no live token (no `✓`) and no `→` recommendation marker;
/// `🔴` in the status column becomes `err`. None of the emoji characters remain.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-1]
/// Also: [`tests/docs/feature/028_usage_row_filtering.md` FT-14]
#[ test ]
fn it198_no_color_1_no_emoji_in_output()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "no_color::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "🔴" ), "no_color::1 must remove 🔴, got:\n{text}" );
  assert!( !text.contains( "🟡" ), "no_color::1 must not contain 🟡, got:\n{text}" );
  assert!( !text.contains( "🟢" ), "no_color::1 must not contain 🟢, got:\n{text}" );
  assert!( !text.contains( '→' ),  "no_color::1 must remove → (replaced by ->), got:\n{text}" );
  assert!( !text.contains( '✓' ),  "no_color::1 must remove ✓, got:\n{text}" );
}

// ── it199: no_color::1 status column shows `err` text label ──────────────────

/// it199 — `no_color::1` status column shows `err` instead of `🔴`.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-2]
#[ test ]
fn it199_no_color_1_status_shows_err_text_label()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "no_color::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "err" ),
    "no_color::1 must show 'err' text label for error account status, got:\n{text}",
  );
}

// ── it200: no_color::bad exits 1 ─────────────────────────────────────────────

/// it200 — `no_color::bad` exits 1; stderr names valid values.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-4]
#[ test ]
fn it200_no_color_bad_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "no_color::bad" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( '0' ) && err.contains( '1' ),
    "no_color::bad stderr must name valid values, got:\n{err}",
  );
}

// ── it201: no_color::true accepted ───────────────────────────────────────────

/// it201 — `no_color::true` accepted as alias for 1; no emoji in output.
///
/// Spec: [`tests/docs/cli/param/047_no_color.md` EC-6]
#[ test ]
fn it201_no_color_true_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".usage", "no_color::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "🔴" ),
    "no_color::true must remove 🔴 (same as no_color::1), got:\n{text}",
  );
}

// ── it202: cols::+host shows Host column ─────────────────────────────────────

/// it202 — `cols::+host` adds Host column; account row shows value from profile.json.
///
/// `write_account_profile_json` creates `{name}.json` with `{"host":"mybox"}`.
/// The `host` field is loaded regardless of token status.
///
/// Spec: [`tests/docs/cli/param/033_cols.md` EC-7]
/// Also: [`tests/docs/feature/029_account_host_metadata.md` AC-05]
#[ test ]
fn it202_cols_host_shows_host_column()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account_profile_json( dir.path(), "acct-a", Some( "mybox" ), Some( "work" ) );

  let out  = run_cs_with_env( &[ ".usage", "cols::+host" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Host" ),
    "cols::+host must add 'Host' column header, got:\n{text}",
  );
  assert!(
    text.contains( "mybox" ),
    "cols::+host must show host value 'mybox' in account row, got:\n{text}",
  );
}

// ── it203: cols::+role shows Role column ─────────────────────────────────────

/// it203 — `cols::+role` adds Role column; account row shows value from profile.json.
///
/// Spec: [`tests/docs/cli/param/033_cols.md` EC-8]
/// Also: [`tests/docs/feature/029_account_host_metadata.md` AC-06]
#[ test ]
fn it203_cols_role_shows_role_column()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "acct-a", "max", "default", FAR_FUTURE_MS, false );
  write_account_profile_json( dir.path(), "acct-a", Some( "mybox" ), Some( "work" ) );

  let out  = run_cs_with_env( &[ ".usage", "cols::+role" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Role" ),
    "cols::+role must add 'Role' column header, got:\n{text}",
  );
  assert!(
    text.contains( "work" ),
    "cols::+role must show role value 'work' in account row, got:\n{text}",
  );
}

// ── it204: cols::+bogus exits 1 naming host and role ─────────────────────────

/// it204 — `cols::+bogus` exits 1; stderr names `host` and `role` among valid IDs.
///
/// After TSK-225, `host` and `role` were added as valid column IDs. The error
/// message must list them along with existing columns like `status`, `expires`, etc.
///
/// Spec: [`tests/docs/cli/param/033_cols.md` EC-9]
#[ test ]
fn it204_cols_bogus_names_host_and_role_in_error()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "cols::+bogus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "host" ),
    "cols::+bogus error must name 'host' as a valid column ID, got:\n{err}",
  );
  assert!(
    err.contains( "role" ),
    "cols::+bogus error must name 'role' as a valid column ID, got:\n{err}",
  );
}

// ── it205: offset::2 count::3 windows result set (028 FT-02) ─────────────────

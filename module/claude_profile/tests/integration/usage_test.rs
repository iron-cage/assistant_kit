//! Integration tests: U (Usage).
//!
//! Tests the `.usage` command which reads `stats-cache.json` and reports
//! per-model 7-day token usage.

use crate::helpers::{
  run_cs_with_env, run_cs_without_home,
  stdout, stderr, assert_exit,
  write_stats_cache, write_stats_cache_raw,
  DayEntry,
};
use tempfile::TempDir;

// ── Error paths ─────────────────────────────────────────────────────────────

/// stats-cache.json missing → exit 2 with descriptive error.
#[ test ]
fn u01_usage_missing_stats_file_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Create .claude dir but no stats-cache.json.
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!( err.contains( "stats-cache.json" ), "error must mention stats-cache.json, got:\n{err}" );
}

/// Empty file → malformed JSON → exit 2.
#[ test ]
fn u02_usage_empty_stats_file_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache_raw( dir.path(), "" );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!( err.contains( "malformed" ), "error must mention malformed JSON, got:\n{err}" );
}

/// Valid JSON but no `dailyModelTokens` key → exit 2.
#[ test ]
fn u03_usage_no_daily_model_tokens_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache_raw( dir.path(), r#"{"lastComputedDate":"2026-03-07"}"# );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!( err.contains( "dailyModelTokens" ), "error must mention dailyModelTokens, got:\n{err}" );
}

/// Missing `lastComputedDate` → exit 2.
///
/// # Root Cause
///
/// Before the fix, `lastComputedDate` defaulted to "unknown" which caused all
/// daily entries to be silently filtered out (ISO dates < "unknown" alphabetically).
///
/// # Why Not Caught
///
/// No test existed for missing `lastComputedDate`; only the happy path was covered.
///
/// # Fix Applied
///
/// Changed from `unwrap_or("unknown")` to `ok_or_else(|| ErrorData)` so a missing
/// `lastComputedDate` now returns an explicit error.
///
/// # Prevention
///
/// This test ensures the error path is exercised.
///
/// # Pitfall
///
/// Optional JSON fields that silently degrade to sentinel values can mask data
/// loss — prefer explicit errors for required fields.
#[ test ]
fn u04_usage_missing_last_computed_date_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache_raw( dir.path(), r#"{"dailyModelTokens":[]}"# );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    err.contains( "lastComputedDate" ),
    "error must mention lastComputedDate, got:\n{err}",
  );
}

/// HOME unset → exit 2.
#[ test ]
fn u05_usage_home_unset_exits_2()
{
  let out = run_cs_without_home( &[ ".usage" ] );
  assert_exit( &out, 2 );
}

// ── Empty / zero data ───────────────────────────────────────────────────────

/// Empty `dailyModelTokens` array → exit 0, total shows 0.
#[ test ]
fn u06_usage_empty_daily_array_shows_zero()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[] );

  let out = run_cs_with_env( &[ ".usage", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "0 total" ), "empty array must show 0 total, got:\n{text}" );
}

// ── Happy path: single model, single day ────────────────────────────────────

#[ test ]
fn u07_usage_single_model_single_day()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 5000 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "5.0K total" ), "must show 5.0K total, got:\n{text}" );
  assert!( text.contains( "sonnet-4-6" ), "must show shortened model name, got:\n{text}" );
}

// ── Multiple models sorted descending ───────────────────────────────────────

#[ test ]
fn u08_usage_multiple_models_sorted_desc()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![
      ( "claude-haiku-4-5-20251001", 1000 ),
      ( "claude-sonnet-4-6", 5000 ),
      ( "claude-opus-4-6", 3000 ),
    ] },
  ] );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // Sonnet (5000) should appear before opus (3000) before haiku (1000).
  let pos_sonnet = text.find( "sonnet" ).expect( "must contain sonnet" );
  let pos_opus   = text.find( "opus" ).expect( "must contain opus" );
  let pos_haiku  = text.find( "haiku" ).expect( "must contain haiku" );
  assert!(
    pos_sonnet < pos_opus && pos_opus < pos_haiku,
    "models must be sorted desc by tokens: sonnet > opus > haiku\ngot:\n{text}",
  );
}

// ── Model name shortening ───────────────────────────────────────────────────

/// `claude-` prefix stripped, 8-digit date suffix stripped.
#[ test ]
fn u09_usage_model_name_shortening()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![
      ( "claude-haiku-4-5-20251001", 1000 ),
      ( "glm-4.5-air", 500 ),
    ] },
  ] );

  let out = run_cs_with_env( &[ ".usage", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // haiku-4-5 (date stripped), not haiku-4-5-20251001
  assert!( text.contains( "haiku-4-5" ), "must strip date suffix, got:\n{text}" );
  assert!( !text.contains( "20251001" ), "date suffix must be removed, got:\n{text}" );
  // non-claude model unchanged
  assert!( text.contains( "glm-4.5-air" ), "non-claude model must be unchanged, got:\n{text}" );
}

// ── Verbosity levels ────────────────────────────────────────────────────────

#[ test ]
fn u10_usage_v0_compact_single_line()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 2_000_000 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.trim().lines().collect();
  assert_eq!( lines.len(), 1, "v::0 must be a single line, got:\n{text}" );
  assert!( text.contains( "total" ), "v::0 must contain 'total', got:\n{text}" );
}

#[ test ]
fn u11_usage_v1_default_table()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 2_000_000 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Usage" ), "v::1 must have header, got:\n{text}" );
  assert!( text.contains( "Total" ), "v::1 must have Total row, got:\n{text}" );
  assert!( text.contains( '%' ), "v::1 must have percentage, got:\n{text}" );
  // Must NOT have Daily section (that's v::2).
  assert!( !text.contains( "Daily" ), "v::1 must not have Daily section, got:\n{text}" );
}

#[ test ]
fn u12_usage_v2_daily_breakdown()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-06", models: vec![ ( "claude-sonnet-4-6", 1_000_000 ) ] },
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 2_000_000 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage", "v::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Daily" ), "v::2 must have Daily section, got:\n{text}" );
  assert!( text.contains( "2026-03-07" ), "v::2 must show individual dates, got:\n{text}" );
  assert!( text.contains( "2026-03-06" ), "v::2 must show both dates, got:\n{text}" );

  // Newest first: 03-07 before 03-06.
  let pos_07 = text.find( "2026-03-07" ).unwrap();
  let pos_06 = text.find( "2026-03-06" ).unwrap();
  assert!( pos_07 < pos_06, "daily must be newest first, got:\n{text}" );
}

// ── JSON output ─────────────────────────────────────────────────────────────

#[ test ]
fn u13_usage_json_valid()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![
      ( "claude-sonnet-4-6", 5000 ),
      ( "claude-opus-4-6", 3000 ),
    ] },
  ] );

  let out = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // Must be parseable JSON.
  let parsed : serde_json::Value = serde_json::from_str( text.trim() )
    .unwrap_or_else( |e| panic!( "JSON must be valid: {e}\ngot:\n{text}" ) );

  assert_eq!( parsed[ "period_days" ], 7, "period_days must be 7" );
  assert_eq!( parsed[ "total_tokens" ], 8000, "total must be 8000" );
  assert!( parsed[ "by_model" ].is_array(), "by_model must be an array" );

  let models = parsed[ "by_model" ].as_array().unwrap();
  assert_eq!( models.len(), 2, "must have 2 models" );
  assert_eq!( models[ 0 ][ "model" ], "sonnet-4-6", "first model (highest) must be sonnet" );
}

// ── 7-day window filtering ──────────────────────────────────────────────────

/// Entries outside the 7-day window must be excluded.
#[ test ]
fn u14_usage_filters_outside_7day_window()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Window: 2026-03-01 → 2026-03-07.
  // Include: 03-01, 03-07. Exclude: 02-28, 03-08.
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-02-28", models: vec![ ( "claude-sonnet-4-6", 9_999_999 ) ] },
    DayEntry { date: "2026-03-01", models: vec![ ( "claude-sonnet-4-6", 1000 ) ] },
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 2000 ) ] },
    DayEntry { date: "2026-03-08", models: vec![ ( "claude-sonnet-4-6", 9_999_999 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let parsed : serde_json::Value = serde_json::from_str( text.trim() ).unwrap();
  // Only 1000 + 2000 = 3000 should be counted.
  assert_eq!( parsed[ "total_tokens" ], 3000, "must only sum in-window entries, got:\n{text}" );
}

// ── Date arithmetic: month boundary ─────────────────────────────────────────

/// `lastComputedDate` = 2026-03-03, window start = 2026-02-25.
#[ test ]
fn u15_usage_month_boundary()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Window: 2026-02-25 → 2026-03-03.
  write_stats_cache( dir.path(), Some( "2026-03-03" ), &[
    DayEntry { date: "2026-02-24", models: vec![ ( "claude-sonnet-4-6", 999 ) ] },
    DayEntry { date: "2026-02-25", models: vec![ ( "claude-sonnet-4-6", 100 ) ] },
    DayEntry { date: "2026-03-03", models: vec![ ( "claude-sonnet-4-6", 200 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let parsed : serde_json::Value = serde_json::from_str( text.trim() ).unwrap();
  assert_eq!( parsed[ "period_start" ], "2026-02-25", "start must cross month boundary" );
  assert_eq!( parsed[ "total_tokens" ], 300, "must include 02-25 and 03-03, got:\n{text}" );
}

// ── Date arithmetic: year boundary ──────────────────────────────────────────

/// `lastComputedDate` = 2026-01-03, window start = 2025-12-28.
#[ test ]
fn u16_usage_year_boundary()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-01-03" ), &[
    DayEntry { date: "2025-12-27", models: vec![ ( "claude-sonnet-4-6", 999 ) ] },
    DayEntry { date: "2025-12-28", models: vec![ ( "claude-sonnet-4-6", 100 ) ] },
    DayEntry { date: "2026-01-03", models: vec![ ( "claude-sonnet-4-6", 200 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let parsed : serde_json::Value = serde_json::from_str( text.trim() ).unwrap();
  assert_eq!( parsed[ "period_start" ], "2025-12-28", "start must cross year boundary" );
  assert_eq!( parsed[ "total_tokens" ], 300, "must include 12-28 and 01-03, got:\n{text}" );
}

// ── Date arithmetic: leap year ──────────────────────────────────────────────

/// `lastComputedDate` = 2024-03-02, window start = 2024-02-25 (Feb has 29 days).
#[ test ]
fn u17_usage_leap_year_boundary()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2024-03-02" ), &[
    DayEntry { date: "2024-02-24", models: vec![ ( "claude-sonnet-4-6", 999 ) ] },
    DayEntry { date: "2024-02-25", models: vec![ ( "claude-sonnet-4-6", 100 ) ] },
    DayEntry { date: "2024-03-02", models: vec![ ( "claude-sonnet-4-6", 200 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let parsed : serde_json::Value = serde_json::from_str( text.trim() ).unwrap();
  // 2024 is a leap year: Mar 2 - 6 = Feb 25 (29 days in Feb).
  assert_eq!( parsed[ "period_start" ], "2024-02-25", "leap year: Mar 2 - 6 = Feb 25" );
  assert_eq!( parsed[ "total_tokens" ], 300 );
}

// ── Token formatting boundaries ─────────────────────────────────────────────

/// Verify compact token formatting at tier boundaries.
///
/// # Root Cause
///
/// `fmt_tokens_compact` used `n < 1_000_000` as the `K→M` boundary, but
/// `{:.1}` rounding caused `999_999` to display as "1000.0K".
///
/// # Why Not Caught
///
/// No test exercised values near the K→M rounding boundary.
///
/// # Fix Applied
///
/// Changed boundary from `n < 1_000_000` to `n < 999_950` to account for
/// `{:.1}` rounding (`999_950` / 1000 = 999.95 → rounds to "1000.0").
///
/// # Prevention
///
/// This test checks all tier boundaries including rounding edge cases.
///
/// # Pitfall
///
/// Floating-point display with fixed precision can round values across
/// intended tier boundaries — test at the rounding threshold, not just
/// at the logical boundary.
#[ test ]
fn u18_usage_token_format_boundaries()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  // Test 999 tokens → "999" (below K threshold).
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 999 ) ] },
  ] );
  let out = run_cs_with_env( &[ ".usage", "v::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "999 total" ), "999 must display as '999', got:\n{text}" );

  // Test 1000 → "1.0K".
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 1000 ) ] },
  ] );
  let out = run_cs_with_env( &[ ".usage", "v::0" ], &[ ( "HOME", home ) ] );
  let text = stdout( &out );
  assert!( text.contains( "1.0K total" ), "1000 must display as '1.0K', got:\n{text}" );

  // Test 999_949 → "999.9K" (just below rounding threshold).
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 999_949 ) ] },
  ] );
  let out = run_cs_with_env( &[ ".usage", "v::0" ], &[ ( "HOME", home ) ] );
  let text = stdout( &out );
  assert!( text.contains( "999.9K total" ), "999_949 must display as '999.9K', got:\n{text}" );

  // Test 999_950 → "1.0M" (at rounding boundary, promoted to M).
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 999_950 ) ] },
  ] );
  let out = run_cs_with_env( &[ ".usage", "v::0" ], &[ ( "HOME", home ) ] );
  let text = stdout( &out );
  assert!( text.contains( "1.0M total" ), "999_950 must display as '1.0M' (not '1000.0K'), got:\n{text}" );

  // Test 1_000_000 → "1.0M".
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 1_000_000 ) ] },
  ] );
  let out = run_cs_with_env( &[ ".usage", "v::0" ], &[ ( "HOME", home ) ] );
  let text = stdout( &out );
  assert!( text.contains( "1.0M total" ), "1_000_000 must display as '1.0M', got:\n{text}" );
}

// ── Multi-day aggregation ───────────────────────────────────────────────────

/// Same model across multiple days must aggregate correctly.
#[ test ]
fn u19_usage_multi_day_aggregation()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-05", models: vec![ ( "claude-sonnet-4-6", 1000 ) ] },
    DayEntry { date: "2026-03-06", models: vec![ ( "claude-sonnet-4-6", 2000 ) ] },
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 3000 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let parsed : serde_json::Value = serde_json::from_str( text.trim() ).unwrap();
  assert_eq!( parsed[ "total_tokens" ], 6000, "must aggregate 1000+2000+3000=6000" );
}

// ── Malformed entries ───────────────────────────────────────────────────────

/// Entries with missing `date` or `tokensByModel` are skipped gracefully.
#[ test ]
fn u20_usage_malformed_entries_skipped()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Raw JSON with one good entry, one missing date, one missing tokensByModel.
  write_stats_cache_raw( dir.path(), r#"{
    "lastComputedDate": "2026-03-07",
    "dailyModelTokens": [
      {"tokensByModel": {"claude-sonnet-4-6": 100}},
      {"date": "2026-03-07", "tokensByModel": {"claude-sonnet-4-6": 500}},
      {"date": "2026-03-06"}
    ]
  }"# );

  let out = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let parsed : serde_json::Value = serde_json::from_str( text.trim() ).unwrap();
  // Only the valid entry (500) should be counted.
  assert_eq!( parsed[ "total_tokens" ], 500, "only valid entries must be counted, got:\n{text}" );
}

// ── JSON with empty by_model ────────────────────────────────────────────────

/// Empty `by_model` produces valid JSON with empty array.
#[ test ]
fn u21_usage_json_empty_by_model()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[] );

  let out = run_cs_with_env( &[ ".usage", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let parsed : serde_json::Value = serde_json::from_str( text.trim() )
    .unwrap_or_else( |e| panic!( "empty by_model must produce valid JSON: {e}\ngot:\n{text}" ) );
  assert_eq!( parsed[ "total_tokens" ], 0 );
  assert!( parsed[ "by_model" ].as_array().unwrap().is_empty() );
}

// ── Percentage calculation ──────────────────────────────────────────────────

/// 100% of tokens from one model shows "100.0%".
#[ test ]
fn u22_usage_single_model_100_percent()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 5000 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "100.0%" ), "single model must show 100.0%, got:\n{text}" );
}

// ── Full-format token display ───────────────────────────────────────────────

/// `v::1` must show comma-separated token counts.
#[ test ]
fn u23_usage_v1_comma_formatted_tokens()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 1_234_567 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "1,234,567" ), "v::1 must show comma-formatted tokens, got:\n{text}" );
}

// ── Period display ──────────────────────────────────────────────────────────

/// `v::1` header must show the date range.
#[ test ]
fn u24_usage_v1_shows_period()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_stats_cache( dir.path(), Some( "2026-03-07" ), &[
    DayEntry { date: "2026-03-07", models: vec![ ( "claude-sonnet-4-6", 100 ) ] },
  ] );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "2026-03-01" ), "must show period start, got:\n{text}" );
  assert!( text.contains( "2026-03-07" ), "must show period end, got:\n{text}" );
}

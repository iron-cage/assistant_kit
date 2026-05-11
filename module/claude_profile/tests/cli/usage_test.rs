//! Integration tests: U (Usage).
//!
//! Tests the `.usage` command which reads `stats-cache.json` and reports
//! per-model 7-day token usage.
//!
//! ## Test Matrix
//!
//! ### U — Usage Command
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | u01 | `u01_usage_missing_stats_file_exits_2` | no stats file → exit 2 | N |
//! | u02 | `u02_usage_empty_stats_file_exits_2` | empty stats file → exit 2 | N |
//! | u03 | `u03_usage_no_daily_model_tokens_exits_2` | stats missing dailyModelTokens → exit 2 | N |
//! | u04 | `u04_usage_missing_last_computed_date_exits_2` | stats missing lastComputedDate → exit 2 | N |
//! | u05 | `u05_usage_home_unset_exits_2` | HOME unset → exit 2 | N |
//! | u06 | `u06_usage_empty_daily_array_shows_zero` | empty daily array → zero usage | P |
//! | u07 | `u07_usage_single_model_single_day` | one model, one day → correct output | P |
//! | u08 | `u08_usage_multiple_models_sorted_desc` | multiple models → sorted descending | P |
//! | u09 | `u09_usage_model_name_shortening` | long model names → shortened display | P |
//! | u10 | *(reserved — not assigned)* | — | — |
//! | u11 | `u11_usage_v1_default_table` | default → table format | P |
//! | u12 | *(reserved — not assigned)* | — | — |
//! | u13 | `u13_usage_json_valid` | format::json → valid JSON output | P |
//! | u14 | `u14_usage_filters_outside_7day_window` | entries outside 7-day window filtered | P |
//! | u15 | `u15_usage_month_boundary` | entries spanning month boundary | P |
//! | u16 | `u16_usage_year_boundary` | entries spanning year boundary | P |
//! | u17 | `u17_usage_leap_year_boundary` | entries spanning leap-year Feb boundary | P |
//! | u18 | *(reserved — not assigned)* | — | — |
//! | u19 | `u19_usage_multi_day_aggregation` | multi-day entries aggregated correctly | P |
//! | u20 | `u20_usage_malformed_entries_skipped` | malformed entries skipped gracefully | P |
//! | u21 | `u21_usage_json_empty_by_model` | no models in window → empty JSON | P |
//! | u22 | `u22_usage_single_model_100_percent` | single model → 100% share | P |
//! | u23 | `u23_usage_v1_comma_formatted_tokens` | large token counts → comma-formatted | P |
//! | u24 | `u24_usage_v1_shows_period` | default → period label shown | P |
//! | u25 | `u25_usage_stale_data_shows_warning` | lastComputedDate 30 days old → warning present | P |
//! | u26 | `u26_usage_fresh_data_no_warning` | lastComputedDate 5 days old → no warning | P |

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

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Total" ), "empty array must show Total row, got:\n{text}" );
  assert!( text.contains( "0\n" ) || text.ends_with( "0\n" ) || text.contains( "  0\n" ),
    "empty array total must be 0, got:\n{text}" );
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

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "5,000" ), "must show 5,000 tokens, got:\n{text}" );
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

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // haiku-4-5 (date stripped), not haiku-4-5-20251001
  assert!( text.contains( "haiku-4-5" ), "must strip date suffix, got:\n{text}" );
  assert!( !text.contains( "20251001" ), "date suffix must be removed, got:\n{text}" );
  // non-claude model unchanged
  assert!( text.contains( "glm-4.5-air" ), "non-claude model must be unchanged, got:\n{text}" );
}

// ── Output format ────────────────────────────────────────────────────────────

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
  assert!( text.contains( "Usage" ), "default output must have header, got:\n{text}" );
  assert!( text.contains( "Total" ), "default output must have Total row, got:\n{text}" );
  assert!( text.contains( '%' ), "default output must have percentage, got:\n{text}" );
  // Must NOT have Daily section.
  assert!( !text.contains( "Daily" ), "default output must not have Daily section, got:\n{text}" );
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

/// Default text output must show comma-separated token counts.
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
  assert!( text.contains( "1,234,567" ), "default output must show comma-formatted tokens, got:\n{text}" );
}

// ── Period display ──────────────────────────────────────────────────────────

/// Default text output header must show the date range.
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

// ── Staleness warning ────────────────────────────────────────────────────────

/// Compute the ISO date string for `n` days before today (UTC day boundary).
///
/// Uses Julian Day Number arithmetic for correct Gregorian calendar rollover —
/// mirrors the `days_since()` implementation so both sides of the check are
/// consistent with the same day-counting method.
fn date_n_days_ago( days : u32 ) -> String
{
  use std::time::{ SystemTime, UNIX_EPOCH };
  let now_secs    = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap().as_secs();
  let today_days  = now_secs / 86400;
  let target_days = today_days - u64::from( days );

  // Days-since-epoch → Julian Day Number → Gregorian calendar.
  // Reverse algorithm from Richards (2013) / Meeus "Astronomical Algorithms".
  let jdn = i64::try_from( target_days ).expect( "epoch days fit in i64" ) + 2_440_588_i64;
  let a   = jdn + 32044;
  let b   = ( 4 * a + 3 ) / 146_097;
  let c   = a - ( 146_097 * b ) / 4;
  let dv  = ( 4 * c + 3 ) / 1461;
  let e   = c - ( 1461 * dv ) / 4;
  let mv  = ( 5 * e + 2 ) / 153;
  let day   = e - ( 153 * mv + 2 ) / 5 + 1;
  let month = mv + 3 - 12 * ( mv / 10 );
  let year  = 100 * b + dv - 4800 + mv / 10;
  format!( "{year:04}-{month:02}-{day:02}" )
}

/// u25: `lastComputedDate` is 30 days before today → warning line present in stdout.
///
/// The interim `.usage` implementation uses `lastComputedDate` verbatim as the
/// window endpoint. Without a staleness guard a user running `clp .usage` months
/// after the cache was last written sees stale token data with no indication.
#[ test ]
fn u25_usage_stale_data_shows_warning()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let date = date_n_days_ago( 30 );
  // Build raw JSON so the dynamic date string can be injected (DayEntry uses &'static str).
  let json = format!(
    r#"{{"lastComputedDate":"{date}","dailyModelTokens":[{{"date":"{date}","tokensByModel":{{"claude-sonnet-4-6":1000}}}}]}}"#
  );
  write_stats_cache_raw( dir.path(), &json );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "⚠ Data last updated" ),
    "30-day-old cache must show staleness warning, got:\n{text}",
  );
  // Token data must still appear after the warning.
  assert!( text.contains( "Total" ), "warning must not replace token data, got:\n{text}" );
}

/// u26: `lastComputedDate` is 5 days before today → no warning line in stdout.
#[ test ]
fn u26_usage_fresh_data_no_warning()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let date = date_n_days_ago( 5 );
  let json = format!(
    r#"{{"lastComputedDate":"{date}","dailyModelTokens":[{{"date":"{date}","tokensByModel":{{"claude-sonnet-4-6":1000}}}}]}}"#
  );
  write_stats_cache_raw( dir.path(), &json );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "⚠" ),
    "5-day-old cache must not show staleness warning, got:\n{text}",
  );
  assert!( text.contains( "Total" ), "must show token data, got:\n{text}" );
}

//! Integration tests for the `clj` binary — EC-1 through EC-38.
//!
//! Each test writes fixture events via `JournalWriter`, runs the `clj` binary
//! against the temporary journal directory, and asserts on stdout/stderr/exit.
//!
//! EC-30 through EC-34 cover `.list sort::`/`reverse::` and `.tail format::`,
//! which were documented parameters with no implementation until they landed.
//! They share `write_sortable_events`, a fixture deliberately built so that no
//! sort field reproduces the order its events were appended in — against a
//! date-ordered fixture, a `sort::` that ignored its argument entirely would
//! satisfy every assertion.
//!
//! EC-36 and EC-37 walk the parameter tables in `docs/cli/type/08_boolean.md`
//! and `docs/cli/type/04_integer.md` rather than naming sites individually, so
//! a parameter added to a type page without a matching implementation is a
//! failing test rather than a quiet gap.

#![ allow( missing_docs ) ]
#![ cfg( unix ) ]

use claude_journal::{ EventRecord, EventType, JournalWriter };
use std::path::Path;
use std::process::{ Command, Stdio };

const CLJ : &str = env!( "CARGO_BIN_EXE_clj" );

fn assert_container()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: ./verb/test (from workspace root)\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

// ── Fixture helpers ───────────────────────────────────────────────────────────

/// Write a mix of events to `dir` using `JournalWriter`.
///
/// Creates 4 events: 2 Execution (one with `stdout` containing "rate limit"),
/// 1 Credential, 1 Retry. All with current-time timestamps so they pass any
/// reasonable `since::` filter.
fn write_fixture_events( dir : &Path )
{
  let writer = JournalWriter::new( dir.to_path_buf() );

  let mut ev1        = EventRecord::new( EventType::Execution );
  ev1.fields.command       = Some( "run".to_owned() );
  ev1.fields.model         = Some( "claude-sonnet-5".to_owned() );
  ev1.fields.exit_code     = Some( 0 );
  ev1.fields.duration_ms   = Some( 1_500 );
  ev1.fields.cost_usd      = Some( 0.012 );
  ev1.fields.input_tokens  = Some( 100 );
  ev1.fields.output_tokens = Some( 50 );
  ev1.fields.stdout        = Some( "Hello world rate limit".to_owned() );
  writer.append( &ev1 ).expect( "append ev1" );

  let mut ev2        = EventRecord::new( EventType::Credential );
  ev2.fields.command   = Some( "refresh".to_owned() );
  ev2.fields.exit_code = Some( 0 );
  ev2.fields.model     = Some( "claude-haiku-4-5-20251001".to_owned() );
  writer.append( &ev2 ).expect( "append ev2" );

  let mut ev3          = EventRecord::new( EventType::Retry );
  ev3.fields.error_class = Some( "Transient".to_owned() );
  ev3.fields.attempt     = Some( 1 );
  ev3.fields.delay_secs  = Some( 30 );
  writer.append( &ev3 ).expect( "append ev3" );

  let mut ev4        = EventRecord::new( EventType::Execution );
  ev4.fields.command       = Some( "ask".to_owned() );
  ev4.fields.model         = Some( "claude-haiku-4-5-20251001".to_owned() );
  ev4.fields.exit_code     = Some( 0 );
  ev4.fields.duration_ms   = Some( 500 );
  ev4.fields.cost_usd      = Some( 0.002 );
  ev4.fields.input_tokens  = Some( 40 );
  ev4.fields.output_tokens = Some( 20 );
  ev4.fields.stdout        = Some( "some output".to_owned() );
  writer.append( &ev4 ).expect( "append ev4" );
}

/// Run `clj` with the given args, always appending `journal_dir::<dir>`.
fn run_clj( args : &[ &str ], dir : &Path ) -> std::process::Output
{
  assert_container();
  Command::new( CLJ )
    .args( args )
    .arg( format!( "journal_dir::{}", dir.display() ) )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "NO_COLOR" )
    .output()
    .expect( "failed to run clj" )
}

/// Stdout as a `String`.
fn stdout_str( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).to_string()
}

/// Stderr as a `String`.
fn stderr_str( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).to_string()
}

// ── EC-1 : .list prints event table ───────────────────────────────────────────

#[ test ]
fn ec1_list_prints_table()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );

  let out = run_clj( &[ ".list" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );

  let stdout = stdout_str( &out );
  assert!( stdout.contains( "TIME" ),   "missing TIME header: {stdout}" );
  assert!( stdout.contains( "TYPE" ),   "missing TYPE header: {stdout}" );
  assert!( stdout.contains( "CMD" ),    "missing CMD header: {stdout}" );
  assert!( stdout.contains( "event(s)" ), "missing event count: {stdout}" );
  // At least one event row with "execution" type
  assert!( stdout.contains( "execution" ), "no execution event in output: {stdout}" );
}

// ── EC-2 : .list format::json outputs JSON array ──────────────────────────────

#[ test ]
fn ec2_list_format_json_outputs_array()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );

  let out = run_clj( &[ ".list", "format::json" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );

  let stdout = stdout_str( &out );
  let json : serde_json::Value = serde_json::from_str( stdout.trim() )
    .expect( "stdout is not valid JSON" );
  assert!( json.is_array(), "expected JSON array, got: {json}" );
  let arr = json.as_array().unwrap();
  assert!( !arr.is_empty(), "JSON array is empty" );
  // Each element should have a "type" field
  assert!(
    arr[ 0 ].get( "type" ).is_some(),
    "first element missing 'type' field"
  );
}

// ── EC-3 : .list type::bogus exits 1 ─────────────────────────────────────────

#[ test ]
fn ec3_list_invalid_type_exits_1()
{
  let dir = tempfile::TempDir::new().unwrap();
  let out = run_clj( &[ ".list", "type::bogus" ], dir.path() );
  assert!( !out.status.success(), "expected non-zero exit" );
  let stderr = stderr_str( &out );
  assert!( stderr.contains( "invalid type" ), "expected 'invalid type' in stderr: {stderr}" );
}

// ── EC-4 : .stats by::model shows aggregation ────────────────────────────────

#[ test ]
fn ec4_stats_by_model_shows_aggregation()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );

  // Use since::9999d to bypass the 7-day default window
  let out = run_clj( &[ ".stats", "by::model", "since::9999d" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );

  let stdout = stdout_str( &out );
  assert!( stdout.contains( "MODEL" ), "missing MODEL header: {stdout}" );
  assert!( stdout.contains( "COUNT" ), "missing COUNT header: {stdout}" );
  assert!( stdout.contains( "COST" ),  "missing COST header: {stdout}" );
  // Should show both models from fixture
  assert!( stdout.contains( "claude-sonnet-5" ), "missing sonnet model: {stdout}" );
  assert!( stdout.contains( "claude-haiku" ),      "missing haiku model: {stdout}" );
}

// ── EC-5 : .search pattern:: filters events ──────────────────────────────────

#[ test ]
fn ec5_search_pattern_filters_events()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );

  let out = run_clj( &[ ".search", "pattern::rate limit", "since::9999d" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );

  let stdout = stdout_str( &out );
  assert!( stdout.contains( "(matched)" ), "no matched events: {stdout}" );
  assert!( stdout.contains( "1 match" ),   "expected 1 match: {stdout}" );
}

// ── EC-6 : .prune dry_run::1 lists without deleting ──────────────────────────

#[ test ]
fn ec6_prune_dry_run_lists_without_deleting()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );

  let before : usize = std::fs::read_dir( dir.path() ).unwrap()
    .filter_map( core::result::Result::ok )
    .filter( | e | e.path().extension().and_then( | x | x.to_str() ) == Some( "jsonl" ) )
    .count();
  assert!( before > 0, "fixture should have created at least 1 JSONL file" );

  // keep::0s means cutoff = now, so everything before now is listed
  let out = run_clj( &[ ".prune", "keep::0s", "dry_run::1" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );

  // Files not deleted (dry run)
  let after : usize = std::fs::read_dir( dir.path() ).unwrap()
    .filter_map( core::result::Result::ok )
    .filter( | e | e.path().extension().and_then( | x | x.to_str() ) == Some( "jsonl" ) )
    .count();
  assert_eq!( before, after, "dry_run should not delete files" );

  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "Would delete" ) || stdout.contains( "Nothing to prune" ),
    "unexpected prune output: {stdout}"
  );
}

// ── EC-7 : .status shows health report ────────────────────────────────────────

#[ test ]
fn ec7_status_shows_health_report()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );

  let out = run_clj( &[ ".status" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );

  // The five lines `docs/cli/command/07_status.md` documents, in its wording —
  // including `Journal level:`, which backs the "and configuration" half of that
  // page's own one-line summary of what `.status` reports.
  let stdout = stdout_str( &out );
  for line in [ "Journal directory:", "Files:", "Total size:", "Date range:", "Journal level:" ]
  {
    assert!( stdout.contains( line ), "missing `{line}` line: {stdout}" );
  }
  assert!( stdout.contains( "Files: 1" ), "expected 1 file: {stdout}" );
}

// ── EC-8 : .export format::json creates file ─────────────────────────────────

#[ test ]
fn ec8_export_json_creates_file()
{
  let dir    = tempfile::TempDir::new().unwrap();
  let outdir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );

  let output_path = outdir.path().join( "export.json" );
  let out = run_clj(
    &[ ".export", "format::json", &format!( "output::{}", output_path.display() ), "since::9999d" ],
    dir.path(),
  );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );

  assert!( output_path.exists(), "export file not created" );
  let content = std::fs::read_to_string( &output_path ).unwrap();
  let json : serde_json::Value = serde_json::from_str( &content )
    .expect( "exported file is not valid JSON" );
  assert!( json.is_array(), "expected JSON array in exported file" );

  let stdout = stdout_str( &out );
  assert!( stdout.contains( "Exported" ), "missing export confirmation: {stdout}" );
}

// ── EC-9 : .list since::xyz exits 1 with "invalid duration" ──────────────────

#[ test ]
fn ec9_list_invalid_since_exits_1()
{
  let dir = tempfile::TempDir::new().unwrap();
  let out = run_clj( &[ ".list", "since::xyz" ], dir.path() );
  assert!( !out.status.success(), "expected non-zero exit" );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "invalid duration" ),
    "expected 'invalid duration' in stderr: {stderr}"
  );
}

// ── EC-10 : type validation at parse time ─────────────────────────────────────

#[ test ]
fn ec10_type_validation_at_parse_time()
{
  let dir = tempfile::TempDir::new().unwrap();

  // 1. invalid since (duration)
  let out = run_clj( &[ ".list", "since::bogus" ], dir.path() );
  assert!( !out.status.success(), "since::bogus should fail" );

  // 2. invalid until (duration)
  let out = run_clj( &[ ".list", "until::bogus" ], dir.path() );
  assert!( !out.status.success(), "until::bogus should fail" );

  // 3. invalid type (event_type)
  let out = run_clj( &[ ".list", "type::bogus" ], dir.path() );
  assert!( !out.status.success(), "type::bogus should fail" );

  // 4. invalid exit_code (i32)
  let out = run_clj( &[ ".list", "exit_code::notanint" ], dir.path() );
  assert!( !out.status.success(), "exit_code::notanint should fail" );

  // 5. invalid limit (usize)
  let out = run_clj( &[ ".list", "limit::negative" ], dir.path() );
  assert!( !out.status.success(), "limit::negative should fail" );

  // 6. invalid format (enum in .list)
  write_fixture_events( dir.path() );
  let out = run_clj( &[ ".list", "format::bogus" ], dir.path() );
  assert!( !out.status.success(), "format::bogus should fail" );

  // 7. invalid by (enum in .stats)
  let out = run_clj( &[ ".stats", "by::bogus", "since::9999d" ], dir.path() );
  assert!( !out.status.success(), "by::bogus should fail" );

  // 8. invalid dry_run (bool in .prune)
  let out = run_clj( &[ ".prune", "dry_run::bogus" ], dir.path() );
  assert!( !out.status.success(), "dry_run::bogus should fail" );

  // 9. invalid keep (duration in .prune)
  let out = run_clj( &[ ".prune", "keep::bogus" ], dir.path() );
  assert!( !out.status.success(), "keep::bogus should fail" );

  // 10. missing pattern in .search (required param)
  let out = run_clj( &[ ".search" ], dir.path() );
  assert!( !out.status.success(), "missing pattern should fail" );
  assert!( stderr_str( &out ).contains( "pattern" ), "error should mention pattern" );

  // 11. missing output in .export (required param)
  let out = run_clj( &[ ".export" ], dir.path() );
  assert!( !out.status.success(), "missing output should fail" );
  assert!( stderr_str( &out ).contains( "output" ), "error should mention output" );
}

// ── EC-11 : NO_COLOR=1 suppresses ANSI codes ─────────────────────────────────

#[ test ]
fn ec11_no_color_suppresses_ansi()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );

  // With NO_COLOR — no ANSI escape sequences
  let out_no_color = Command::new( CLJ )
    .args( [ ".list" ] )
    .arg( format!( "journal_dir::{}", dir.path().display() ) )
    .env( "NO_COLOR", "1" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .output()
    .expect( "failed to run clj" );
  assert!( out_no_color.status.success() );
  let stdout_nc = stdout_str( &out_no_color );
  assert!(
    !stdout_nc.contains( "\x1b[" ),
    "NO_COLOR=1 should suppress ANSI codes, got: {stdout_nc}"
  );

  // Without NO_COLOR — ANSI escape sequences present (bold header)
  let out_color = run_clj( &[ ".list" ], dir.path() );
  assert!( out_color.status.success() );
  let stdout_color = stdout_str( &out_color );
  assert!(
    stdout_color.contains( "\x1b[" ),
    "without NO_COLOR, ANSI codes should be present: {stdout_color}"
  );
}

// EC-12 (`.serve` HTTP GET `/`) moved to `serve_test.rs` as FT-2 — all `.serve`
// coverage shares one spawn/parse/connect harness there rather than two copies.

// ── EC-13 : .tail starts and can be killed ────────────────────────────────────

/// EC-13: `.tail` blocks (infinite iterator) waiting for new events.  This test
/// verifies the command starts without panicking and remains running until killed.
///
/// The command prints "Tailing journal — press Ctrl+C to stop" to stderr on
/// startup, then blocks.  We kill it after 300 ms — a graceful SIGKILL is the
/// expected termination mechanism for `.tail`.
#[ test ]
fn ec13_tail_starts_and_can_be_killed()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );

  let mut child = Command::new( CLJ )
    .args( [ ".tail" ] )
    .arg( format!( "journal_dir::{}", dir.path().display() ) )
    .env_remove( "CLR_JOURNAL_DIR" )
    .stdout( Stdio::null() )
    .stderr( Stdio::piped() )
    .spawn()
    .expect( "failed to spawn clj .tail" );

  // Brief delay — confirm the process starts without immediately panicking.
  std::thread::sleep( core::time::Duration::from_millis( 300 ) );

  // If the process exited prematurely, try_wait returns Some(_).
  assert!(
    child.try_wait().expect( "try_wait" ).is_none(),
    "clj .tail exited prematurely — expected it to remain running"
  );

  // Kill the infinite tail loop.
  child.kill().ok();
  child.wait().ok();
}

// ── EC-14 : .chart default invocation writes usage.svg in cwd ────────────────

#[ test ]
fn ec14_chart_default_writes_usage_svg_in_cwd()
{
  assert_container();
  let dir     = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let cwd_dir = tempfile::TempDir::new().unwrap();

  let out = Command::new( CLJ )
    .args( [ ".chart" ] )
    .arg( format!( "journal_dir::{}", dir.path().display() ) )
    .current_dir( cwd_dir.path() )
    .env_remove( "CLR_JOURNAL_DIR" )
    .output()
    .expect( "failed to run clj" );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );

  let expected = cwd_dir.path().join( "usage.svg" );
  assert!( expected.exists(), "expected usage.svg in cwd: {}", expected.display() );
  let svg = std::fs::read_to_string( &expected ).expect( "read usage.svg" );
  assert!( svg.starts_with( "<svg" ), "expected valid svg output: {svg}" );

  let stdout = stdout_str( &out );
  assert!( stdout.contains( "Chart written" ), "missing confirmation: {stdout}" );
  assert!( !stdout.to_lowercase().contains( "warning" ), "no open:: requested, should not warn: {stdout}" );
}

// ── EC-15 : .chart out::<path> writes to a custom path ────────────────────────

#[ test ]
fn ec15_chart_custom_out_path()
{
  let dir     = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let outdir  = tempfile::TempDir::new().unwrap();
  let out_path = outdir.path().join( "custom.svg" );

  let out = run_clj( &[ ".chart", &format!( "out::{}", out_path.display() ) ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );
  assert!( out_path.exists(), "expected custom.svg to exist at {}", out_path.display() );
}

// ── EC-16 : .chart open::1 — browser-open failure is non-fatal ───────────────

#[ test ]
fn ec16_chart_open_failure_is_non_fatal()
{
  let dir      = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let outdir   = tempfile::TempDir::new().unwrap();
  let out_path = outdir.path().join( "chart.svg" );

  // `open::1`, not `open::true` — `true` is no longer a documented boolean and
  // now exits 1, which would make this case pass for the wrong reason.
  let out = run_clj( &[ ".chart", &format!( "out::{}", out_path.display() ), "open::1" ], dir.path() );
  assert!(
    out.status.success(),
    "exit non-zero even though SVG write should succeed regardless of browser-open outcome: {}",
    stderr_str( &out )
  );
  assert!( out_path.exists(), "SVG should still be written even if browser-open fails" );
}

// ── EC-17 : .help lists .chart ─────────────────────────────────────────────────

#[ test ]
fn ec17_help_lists_chart()
{
  let dir = tempfile::TempDir::new().unwrap();
  let out = run_clj( &[ ".help" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );
  assert!( stdout.contains( ".chart" ), "expected .chart in help output: {stdout}" );
}

// ── EC-18 : .chart against an empty journal produces a placeholder SVG ───────

#[ test ]
fn ec18_chart_empty_journal_produces_placeholder()
{
  let dir      = tempfile::TempDir::new().unwrap(); // no fixture events written
  let outdir   = tempfile::TempDir::new().unwrap();
  let out_path = outdir.path().join( "chart.svg" );

  let out = run_clj( &[ ".chart", &format!( "out::{}", out_path.display() ) ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );
  let svg = std::fs::read_to_string( &out_path ).expect( "svg should exist" );
  assert!( svg.starts_with( "<svg" ), "expected valid svg output even for empty journal: {svg}" );
}

// ── EC-19 : .chart journal_dir:: resolves the same way as the other commands ──

#[ test ]
fn ec19_chart_journal_dir_param_resolution_nonexistent_dir_errors()
{
  assert_container();
  let base        = tempfile::TempDir::new().unwrap();
  let nonexistent = base.path().join( "does_not_exist" );
  let outdir      = tempfile::TempDir::new().unwrap();
  let out_path    = outdir.path().join( "chart.svg" );

  let out = Command::new( CLJ )
    .args( [ ".chart" ] )
    .arg( format!( "journal_dir::{}", nonexistent.display() ) )
    .arg( format!( "out::{}", out_path.display() ) )
    .env_remove( "CLR_JOURNAL_DIR" )
    .output()
    .expect( "failed to run clj" );

  assert!( !out.status.success(), "expected non-zero exit for a nonexistent journal dir" );
  assert!( !out_path.exists(), "output file must not be written when the journal dir is missing" );
}

// ── EC-20 : .prune is filename-date-based and never touches non-journal files ─

/// `.prune` deletes by `YYYY-MM-DD.jsonl` filename date: an old dated file goes,
/// while a non-date `.jsonl` file and today's file survive even at `keep::0s`
/// (sub-day durations floor to a 0-day window = keep only today).
#[ test ]
fn ec20_prune_filename_date_semantics()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() ); // creates today's YYYY-MM-DD.jsonl
  std::fs::write( dir.path().join( "2020-01-01.jsonl" ), "{}\n" ).unwrap();
  std::fs::write( dir.path().join( "notes.jsonl" ), "not a journal file\n" ).unwrap();

  let out = run_clj( &[ ".prune", "keep::0s" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );
  assert!( stdout.contains( "2020-01-01.jsonl" ), "old dated file must be reported: {stdout}" );
  assert!( !dir.path().join( "2020-01-01.jsonl" ).exists(), "old dated file must be deleted" );
  assert!(
    dir.path().join( "notes.jsonl" ).exists(),
    "non-date-pattern .jsonl must never be deleted by .prune"
  );
  assert!(
    dir.path().join( claude_journal::rotation::today_filename() ).exists(),
    "today's journal file must survive even keep::0s"
  );
}

// ── Fixture for grouping tests (task 543) ────────────────────────────────────

/// Write events with uneven `dir`/`agent_id` distributions plus one field-less
/// event: 3× alpha, 2× beta, 1× neither — so ranked output order is testable.
fn write_grouping_fixture( dir : &Path )
{
  let writer = JournalWriter::new( dir.to_path_buf() );
  let stamp = | d : &str |
  {
    let mut ev = EventRecord::new( EventType::Execution );
    ev.fields.command  = Some( "run".to_owned() );
    ev.fields.exit_code = Some( 0 );
    ev.fields.dir      = Some( format!( "/tmp/{d}" ) );
    ev.fields.agent_id = Some( format!( "tester@testhost/tmp/{d}/" ) );
    ev
  };
  for _ in 0..3 { writer.append( &stamp( "alpha" ) ).expect( "append alpha" ); }
  for _ in 0..2 { writer.append( &stamp( "beta" ) ).expect( "append beta" ); }
  let bare = EventRecord::new( EventType::Execution );
  writer.append( &bare ).expect( "append bare" );
}

// ── EC-21 : .stats by::dir ranks rows by descending count ────────────────────

#[ test ]
fn ec21_stats_by_dir_ranked_rows()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_grouping_fixture( dir.path() );

  let out = run_clj( &[ ".stats", "by::dir", "since::9999d" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );

  let stdout = stdout_str( &out );
  assert!( stdout.contains( "DIR" ), "missing DIR header: {stdout}" );
  let alpha = stdout.find( "/tmp/alpha" ).expect( "alpha row missing" );
  let beta  = stdout.find( "/tmp/beta" ).expect( "beta row missing" );
  let none  = stdout.find( "(no dir)" ).expect( "(no dir) row missing" );
  assert!( alpha < beta && beta < none, "rows not in descending count order: {stdout}" );
  assert!( stdout.contains( "Total: 6 event(s)" ), "wrong total: {stdout}" );
}

// ── EC-22 : .stats by::agent ranks rows by descending count ──────────────────

#[ test ]
fn ec22_stats_by_agent_ranked_rows()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_grouping_fixture( dir.path() );

  let out = run_clj( &[ ".stats", "by::agent", "since::9999d" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );

  let stdout = stdout_str( &out );
  assert!( stdout.contains( "AGENT" ), "missing AGENT header: {stdout}" );
  let alpha = stdout.find( "tester@testhost/tmp/alpha/" ).expect( "alpha agent row missing" );
  let beta  = stdout.find( "tester@testhost/tmp/beta/" ).expect( "beta agent row missing" );
  let none  = stdout.find( "(no agent)" ).expect( "(no agent) row missing" );
  assert!( alpha < beta && beta < none, "rows not in descending count order: {stdout}" );
}

// ── EC-23 : field-less events aggregate under visible buckets with counts ────

#[ test ]
fn ec23_stats_missing_field_buckets_carry_counts()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_grouping_fixture( dir.path() );

  let by_dir = stdout_str( &run_clj( &[ ".stats", "by::dir", "since::9999d" ], dir.path() ) );
  let bucket_line = by_dir.lines()
    .find( | l | l.contains( "(no dir)" ) )
    .expect( "(no dir) bucket missing" );
  assert!( bucket_line.contains( '1' ), "(no dir) bucket lacks count 1: {bucket_line}" );

  let by_agent = stdout_str( &run_clj( &[ ".stats", "by::agent", "since::9999d" ], dir.path() ) );
  let bucket_line = by_agent.lines()
    .find( | l | l.contains( "(no agent)" ) )
    .expect( "(no agent) bucket missing" );
  assert!( bucket_line.contains( '1' ), "(no agent) bucket lacks count 1: {bucket_line}" );
}

// ── EC-24 : .stats by::bogus error lists dir and agent as valid values ───────

#[ test ]
fn ec24_stats_by_bogus_lists_valid_values()
{
  let dir = tempfile::TempDir::new().unwrap();
  let out = run_clj( &[ ".stats", "by::bogus" ], dir.path() );
  assert!( !out.status.success(), "expected non-zero exit" );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "valid: day, model, dir, agent" ),
    "error must list all valid by values: {stderr}"
  );
}

// ── EC-25 : empty HOME never resolves the journal relative to cwd ────────────

/// EC-25 — `bug_reproducer(BUG-550)`
///
/// # Root Cause
/// `resolve_journal_dir`'s `/tmp` fallback used `unwrap_or_else`, which fires only on
/// `Err` (HOME genuinely unset). `HOME=""` returns `Ok("")`, and `PathBuf::from("")
/// .join(".clr")` is RELATIVE — so `clj` silently read from (and reported on) a journal
/// under the invocation cwd instead of the documented absolute default.
///
/// # Why Not Caught
/// Every other integration test passes an explicit `journal_dir::` (see `run_clj`),
/// which short-circuits resolution at the first tier — no test ever exercised the
/// HOME tier.
///
/// # Fix Applied
/// `resolve_journal_dir` filters an empty HOME into the same `/tmp` fallback as an unset
/// one (`src/output.rs`), matching the `is_empty()` guard the `CLR_JOURNAL_DIR` arm
/// directly above it already had.
///
/// # Prevention
/// The probe event is uniquely named and the test carries its own positive control, so
/// the absence assertion cannot pass vacuously through a broken fixture.
///
/// # Pitfall
/// `env::var` distinguishes unset (`Err`) from empty (`Ok("")`) — an empty path prefix
/// silently converts an absolute join into a relative one.
#[ test ]
fn ec25_empty_home_does_not_resolve_relative_journal()
{
  assert_container();

  let cwd              = tempfile::TempDir::new().unwrap();
  let relative_journal = cwd.path().join( ".clr" ).join( "journal" );
  std::fs::create_dir_all( &relative_journal ).unwrap();

  // A uniquely-named event that can only surface if the cwd-relative path was resolved.
  let writer = JournalWriter::new( relative_journal.clone() );
  let mut ev = EventRecord::new( EventType::Execution );
  ev.fields.command   = Some( "ec25_relative_probe".to_owned() );
  ev.fields.exit_code = Some( 0 );
  writer.append( &ev ).expect( "append probe event" );

  // Positive control: read the fixture explicitly via `journal_dir::`. The probe IS
  // visible here, so its later absence means the path was genuinely not resolved — not
  // that the fixture is unreadable or the marker misspelled.
  let control = run_clj( &[ ".list" ], &relative_journal );
  assert!(
    stdout_str( &control ).contains( "ec25_relative_probe" ),
    "positive control failed — probe unreadable via explicit journal_dir::: {}",
    stdout_str( &control ),
  );

  // The defect: no `journal_dir::`, empty HOME, `CLR_JOURNAL_DIR` removed, cwd = fixture's parent.
  let out = Command::new( CLJ )
    .arg( ".list" )
    .current_dir( cwd.path() )
    .env( "HOME", "" )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "NO_COLOR" )
    .output()
    .expect( "failed to run clj" );

  assert!(
    !stdout_str( &out ).contains( "ec25_relative_probe" ),
    "empty HOME resolved the journal relative to cwd (BUG-550): {}",
    stdout_str( &out ),
  );
}

// ── EC-26..EC-29 : parameter names mean what the docs say they mean ───────────

/// Three events that differ in exit code and working directory, so a filter that
/// is silently ignored produces a visibly different count than one that works.
fn write_param_fixture( dir : &Path )
{
  let writer = JournalWriter::new( dir.to_path_buf() );

  let mut ok       = EventRecord::new( EventType::Execution );
  ok.fields.command   = Some( "param_ok".to_owned() );
  ok.fields.exit_code = Some( 0 );
  ok.fields.dir       = Some( "/work/alpha".to_owned() );
  writer.append( &ok ).expect( "append ok" );

  let mut failed   = EventRecord::new( EventType::Execution );
  failed.fields.command   = Some( "param_failed".to_owned() );
  failed.fields.exit_code = Some( 2 );
  failed.fields.dir       = Some( "/work/alpha".to_owned() );
  writer.append( &failed ).expect( "append failed" );

  let mut other    = EventRecord::new( EventType::Execution );
  other.fields.command   = Some( "param_other".to_owned() );
  other.fields.exit_code = Some( 2 );
  other.fields.dir       = Some( "/work/beta".to_owned() );
  writer.append( &other ).expect( "append other" );
}

/// EC-26 — `exit::` is the key the filter actually reads.
///
/// # Root Cause
/// `build_filter` looked up `exit_code`, the JSON *field* name, while every doc and
/// help line printed `exit::` (`docs/cli/param/05_exit.md`). Nothing read `exit`.
///
/// # Why Not Caught
/// No test passed `exit::` at all — the two integration tests touching exit codes
/// asserted on unfiltered output, where an ignored filter is invisible.
///
/// # Fix Applied
/// `build_filter` reads `exit`; unknown keys are now rejected (EC-28), so the old
/// spelling fails loudly instead of silently widening the result set.
///
/// # Prevention
/// The fixture contains both a matching and a non-matching event, so an ignored
/// filter returns 3 rows where a working one returns 2 — the assertion cannot pass
/// vacuously.
///
/// # Pitfall
/// An ignored *filter* param fails open, not closed: it widens output rather than
/// erroring, which reads as success.
#[ test ]
fn ec26_exit_param_filters_by_exit_code()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_param_fixture( dir.path() );

  let out = run_clj( &[ ".list", "exit::2" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );

  assert!( stdout.contains( "param_failed" ), "exit::2 must keep the exit-2 event: {stdout}" );
  assert!( stdout.contains( "param_other" ),  "exit::2 must keep both exit-2 events: {stdout}" );
  assert!( !stdout.contains( "param_ok" ),    "exit::2 must drop the exit-0 event: {stdout}" );
}

/// EC-27 — `dir::` filters on the event's own working directory.
///
/// # Root Cause
/// `resolve_journal_dir` consumed `dir::` as the journal *location*, so a documented
/// directory filter silently repointed the reader at a path with no journal in it.
/// The result — "No events found." — is indistinguishable from a filter that matched
/// nothing.
///
/// # Why Not Caught
/// Every test used `dir::` for its own tempdir, which is exactly the wrong reading
/// working by coincidence: the path happened to be a real journal.
///
/// # Fix Applied
/// `journal_dir::` overrides the journal location; `dir::` populates
/// `JournalFilter::dir`, the field the library already reserved for it.
///
/// # Prevention
/// This test passes both keys at once with different values, so neither can be
/// standing in for the other.
///
/// # Pitfall
/// When a library type already reserves a field name, the CLI must not spend the
/// same name on an unrelated meaning.
#[ test ]
fn ec27_dir_param_filters_by_event_working_directory()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_param_fixture( dir.path() );

  // `journal_dir::` (supplied by run_clj) locates the journal; `dir::` filters within it.
  let out = run_clj( &[ ".list", "dir::/work/beta" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );

  assert!( stdout.contains( "param_other" ), "dir::/work/beta must keep its event: {stdout}" );
  assert!( !stdout.contains( "param_ok" ),     "dir:: must drop /work/alpha events: {stdout}" );
  assert!( !stdout.contains( "param_failed" ), "dir:: must drop /work/alpha events: {stdout}" );
}

/// EC-28 — an unrecognised parameter exits 1 instead of being ignored.
///
/// Also the behavioural pin for every *retraction*: a parameter that once had a
/// page, or was once accepted and applied to nothing, has to be rejected by the
/// binary and not merely absent from a table somewhere.
#[ test ]
fn ec28_unknown_param_exits_1()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_param_fixture( dir.path() );

  // The pre-fix spelling of EC-26's filter is now a hard error, not a silent no-op.
  let out = run_clj( &[ ".list", "exit_code::2" ], dir.path() );
  assert!( !out.status.success(), "unknown param must exit non-zero" );
  let stderr = stderr_str( &out );
  assert!( stderr.contains( "unknown parameter" ), "missing diagnostic: {stderr}" );
  assert!( stderr.contains( "exit_code" ), "diagnostic must name the offending key: {stderr}" );
  assert!( stderr.contains( "Accepted:" ), "diagnostic must list what is accepted: {stderr}" );

  // A param valid for another command is still rejected for this one — including
  // `output::`, whose silent acceptance would read as "wrote the file" while
  // `.list` printed to stdout and created nothing.
  for ( param, owner ) in [ ( "by::model", ".stats" ), ( "output::/tmp/should_not_be_used.txt", ".export" ) ]
  {
    let out = run_clj( &[ ".list", param ], dir.path() );
    assert!( !out.status.success(), "`{param}` belongs to {owner}, not .list" );
  }
  assert!(
    !std::path::Path::new( "/tmp/should_not_be_used.txt" ).exists(),
    "a rejected `output::` must not have written its file",
  );

  // An unknown *command* outranks an unknown param — the command is the real error.
  let out = run_clj( &[ ".bogus", "since::1d" ], dir.path() );
  assert!( !out.status.success() );
  assert!(
    stderr_str( &out ).contains( "unknown command" ),
    "unknown command must be reported ahead of its params: {}",
    stderr_str( &out ),
  );

  // A retracted param is an unknown one, with no separate diagnostic class.
  // `wide::` and `columns::` had parameter pages describing a renderer that was
  // never built; the pages are gone, so the only honest answer left is the list
  // of what `.list` does accept.
  for retracted in [ "wide::1", "columns::time,cost" ]
  {
    let out = run_clj( &[ ".list", retracted ], dir.path() );
    assert!( !out.status.success(), "`{retracted}` must exit non-zero" );
    let stderr = stderr_str( &out );
    assert!( stderr.contains( "unknown parameter" ), "wrong diagnostic for {retracted}: {stderr}" );
    assert!( stderr.contains( "Accepted:" ), "must list the accepted set for {retracted}: {stderr}" );
  }

  // `.stats verbosity::` was retracted alongside them, so it is unknown *there*
  // even though `.status` now implements the same key — the accepted set is
  // per-command, and this pins that it stayed that way.
  let out = run_clj( &[ ".stats", "verbosity::2" ], dir.path() );
  assert!( !out.status.success(), "`.stats verbosity::` was retracted" );
  assert!( stderr_str( &out ).contains( "unknown parameter" ), "{}", stderr_str( &out ) );

  // Fix(param-inert-accept): pin the two retractions that closed the gap between
  //   what the docs promised and what any command actually applies.
  // Root cause: a parameter can be inert in two different ways. `include_stdout`
  //   was wired to no command at all while a page, a type-table row, a group
  //   membership and two user-story recipes described it as widening `.search` —
  //   which already searches stdout unconditionally. `.tail since::`/`limit::`
  //   were the opposite: accepted, parsed, and then never consulted, because
  //   `TailIter` calls `event_matches` with `since_cutoff : None` and never reads
  //   `filter.limit`. The first exited 1 on a documented recipe; the second
  //   exited 0 having done nothing.
  // Pitfall: `tests/cli_doc_consistency.rs` proves the docs and the binary agree
  //   on what is accepted, but it cannot prove a rejection is still enforced —
  //   it reads that same accepted set to decide what to compare against. This is
  //   the behavioural half, and the two are not substitutes for each other.
  let out = run_clj( &[ ".search", "pattern::x", "include_stdout::1" ], dir.path() );
  assert!( !out.status.success(), "`include_stdout::` is superseded, not accepted" );
  assert!( stderr_str( &out ).contains( "unknown parameter" ), "{}", stderr_str( &out ) );

  // `.tail` is bounded on wall-clock time rather than asserted on directly: if
  // the rejection regresses, `.tail` starts following the journal and `output()`
  // would wait on it forever, so the regression would present as a silent hung
  // suite instead of as this failure.
  for retracted in [ "since::1h", "limit::5" ]
  {
    let mut child  = spawn_tail( &[ retracted ], dir.path() );
    let mut exited = None;
    for _ in 0..50
    {
      if let Some( status ) = child.try_wait().expect( "try_wait failed" ) { exited = Some( status ); break; }
      std::thread::sleep( core::time::Duration::from_millis( 100 ) );
    }
    let status = exited.unwrap_or_else( ||
    {
      kill_child( &mut child );
      panic!( "`.tail {retracted}` did not exit within 5s — it is being followed, not rejected" );
    } );
    assert!( !status.success(), "`.tail {retracted}` must exit non-zero" );
    let mut stderr = String::new();
    std::io::Read::read_to_string( child.stderr.as_mut().expect( "no stderr pipe" ), &mut stderr ).ok();
    assert!( stderr.contains( "unknown parameter" ), "wrong diagnostic for `.tail {retracted}`: {stderr}" );
  }
}

/// EC-29 — `no_color::1` suppresses ANSI, matching the `NO_COLOR` env var.
///
/// # Root Cause
/// `docs/cli/param/24_no_color.md` documents `no_color::1` with worked examples, but
/// only the `NO_COLOR` environment variable was ever read.
///
/// # Why Not Caught
/// EC-11 covered the env var only; no test passed the parameter form.
///
/// # Fix Applied
/// `output::force_no_color()` sets a process-wide override that `no_color()` consults
/// alongside the env var; `main` calls it when the param is `1`/`true`.
///
/// # Prevention
/// The test asserts the presence of ANSI without the param in the same run, so a
/// globally-colorless build cannot make it pass vacuously.
///
/// # Pitfall
/// A setting readable only from the environment cannot be driven by an argument —
/// documenting both forms requires implementing both.
#[ test ]
fn ec29_no_color_param_suppresses_ansi()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_param_fixture( dir.path() );

  let plain = run_clj( &[ ".list", "no_color::1" ], dir.path() );
  assert!( plain.status.success(), "exit non-zero: {}", stderr_str( &plain ) );
  assert!(
    !stdout_str( &plain ).contains( "\x1b[" ),
    "no_color::1 should suppress ANSI codes, got: {}",
    stdout_str( &plain ),
  );

  // Control: the same command without the param does emit ANSI.
  let colored = run_clj( &[ ".list" ], dir.path() );
  assert!(
    stdout_str( &colored ).contains( "\x1b[" ),
    "control failed — ANSI absent even without no_color::: {}",
    stdout_str( &colored ),
  );
}

// ── Sort / format fixtures and helpers ────────────────────────────────────────

/// Write three events whose every sortable field orders them differently from
/// the order they are appended in.
///
/// Deliberate: a fixture already in the target order lets a sort that does
/// nothing at all pass every assertion. Insertion order here is `run`, `ask`,
/// `build`, and no documented sort key (`cost`, `duration`, `exit`, `model`,
/// `command`) reproduces that order or its exact reverse.
fn write_sortable_events( dir : &Path )
{
  let writer = JournalWriter::new( dir.to_path_buf() );

  let mut first = EventRecord::new( EventType::Execution );
  first.fields.command     = Some( "run".to_owned() );
  first.fields.model       = Some( "sonnet".to_owned() );
  first.fields.exit_code   = Some( 0 );
  first.fields.duration_ms = Some( 1_500 );
  first.fields.cost_usd    = Some( 0.05 );
  writer.append( &first ).expect( "append first" );

  let mut second = EventRecord::new( EventType::Execution );
  second.fields.command     = Some( "ask".to_owned() );
  second.fields.model       = Some( "opus".to_owned() );
  second.fields.exit_code   = Some( 2 );
  second.fields.duration_ms = Some( 300 );
  second.fields.cost_usd    = Some( 0.90 );
  writer.append( &second ).expect( "append second" );

  let mut third = EventRecord::new( EventType::Execution );
  third.fields.command     = Some( "build".to_owned() );
  third.fields.model       = Some( "haiku".to_owned() );
  third.fields.exit_code   = Some( 1 );
  third.fields.duration_ms = Some( 9_000 );
  third.fields.cost_usd    = Some( 0.01 );
  writer.append( &third ).expect( "append third" );
}

/// Run `.list format::json` with `extra` args and return the parsed event array.
///
/// Panics with the command's own stderr on a non-zero exit, so a failing
/// assertion names the CLI's diagnostic rather than a JSON parse error.
fn list_json( extra : &[ &str ], dir : &Path ) -> Vec< serde_json::Value >
{
  let mut args = vec![ ".list", "format::json" ];
  args.extend_from_slice( extra );
  let out = run_clj( &args, dir );
  assert!( out.status.success(), "`.list {}` exited non-zero: {}", extra.join( " " ), stderr_str( &out ) );
  serde_json::from_str::< serde_json::Value >( stdout_str( &out ).trim() )
    .expect( "stdout is not valid JSON" )
    .as_array()
    .expect( "format::json must produce an array" )
    .clone()
}

/// Pull one field out of every event, as a string, for order comparison.
///
/// A missing field becomes `"-"` rather than being skipped: dropping it would
/// shorten the sequence and hide a mis-ordered event instead of surfacing it.
fn field_seq( events : &[ serde_json::Value ], field : &str ) -> Vec< String >
{
  events
    .iter()
    .map( | e | e.get( field ).map_or_else( || "-".to_owned(), ToString::to_string ) )
    .collect()
}

/// EC-30 — `sort::` orders by each documented `SortField`, `reverse::1` inverts it.
///
/// Covers the `.list` test plan's IT-4 and the `SortField` plan's TC-1 through TC-5,
/// none of which had an implementing test: `sort::` and `reverse::` were documented
/// parameters that exited 1 as unimplemented.
#[ test ]
fn ec30_sort_orders_by_every_documented_field()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_sortable_events( dir.path() );

  // ( sort key, json field, ascending order of that field's values )
  let cases : &[ ( &str, &str, [ &str; 3 ] ) ] =
  &[
    ( "cost",     "cost_usd",    [ "0.01", "0.05", "0.9" ] ),
    ( "duration", "duration_ms", [ "300", "1500", "9000" ] ),
    ( "exit",     "exit_code",   [ "0", "1", "2" ] ),
    ( "model",    "model",       [ "\"haiku\"", "\"opus\"", "\"sonnet\"" ] ),
    ( "command",  "command",     [ "\"ask\"", "\"build\"", "\"run\"" ] ),
  ];

  for ( key, field, ascending ) in cases
  {
    let sort_arg = format!( "sort::{key}" );

    let asc = field_seq( &list_json( &[ sort_arg.as_str() ], dir.path() ), field );
    assert_eq!( asc, ascending.to_vec(), "sort::{key} is not ascending by {field}" );

    let desc = field_seq( &list_json( &[ sort_arg.as_str(), "reverse::1" ], dir.path() ), field );
    let mut expected = ascending.to_vec();
    expected.reverse();
    assert_eq!( desc, expected, "sort::{key} reverse::1 is not descending by {field}" );
  }

  // `time` is the default, so an explicit `sort::time` and no sort at all agree —
  // and both differ from every ordering above, which is what makes those non-vacuous.
  let default_order  = field_seq( &list_json( &[], dir.path() ), "command" );
  let explicit_order = field_seq( &list_json( &[ "sort::time" ], dir.path() ), "command" );
  assert_eq!( default_order, explicit_order, "sort::time must match the default ordering" );
  assert_eq!(
    default_order,
    vec![ "\"run\"", "\"ask\"", "\"build\"" ],
    "default order must be append order, not any sorted order",
  );
}

/// EC-31 — `sort::` matches case-insensitively; bad `sort::`/`reverse::` values exit 1.
///
/// Covers the `SortField` plan's TC-6 and TC-7, and the `Boolean` type's documented
/// rejection message for `reverse::`.
#[ test ]
fn ec31_sort_case_insensitive_and_invalid_values_exit_1()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_sortable_events( dir.path() );

  // TC-6 — `COST` and `cost` are the same field.
  let upper = field_seq( &list_json( &[ "sort::COST" ], dir.path() ), "cost_usd" );
  let lower = field_seq( &list_json( &[ "sort::cost" ], dir.path() ), "cost_usd" );
  assert_eq!( upper, lower, "sort:: must match case-insensitively" );

  // TC-7 — an unknown field names all six valid ones rather than silently
  // falling back to the default, which would look like a sort that worked.
  let out = run_clj( &[ ".list", "sort::popularity" ], dir.path() );
  assert!( !out.status.success(), "sort::popularity must exit non-zero" );
  let stderr = stderr_str( &out );
  for valid in [ "time", "cost", "duration", "exit", "model", "command" ]
  {
    assert!( stderr.contains( valid ), "diagnostic must list '{valid}': {stderr}" );
  }

  // `reverse::` is a documented Boolean — only 0 and 1.
  let out = run_clj( &[ ".list", "reverse::2" ], dir.path() );
  assert!( !out.status.success(), "reverse::2 must exit non-zero" );
  let stderr = stderr_str( &out );
  assert!( stderr.contains( "reverse" ),  "diagnostic must name the parameter: {stderr}" );
  assert!( stderr.contains( "0 or 1" ),   "diagnostic must state the accepted values: {stderr}" );
}

/// EC-32 — `.list` non-table formats are byte-identical to `.export`'s.
///
/// `.list` renders `table` itself and hands every other format to the same
/// `build_export_content` `.export` uses. Asserting equality is what keeps that
/// delegation real: two independent implementations of `csv` would drift, and the
/// drift would only ever show up in whichever surface the reader was not using.
#[ test ]
fn ec32_list_non_table_formats_match_export()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_sortable_events( dir.path() );
  // Exports land outside the journal dir on purpose: an `-export.jsonl` written
  // *into* it would be picked up as a journal file by the next iteration's read.
  let out_dir = tempfile::TempDir::new().unwrap();

  for format in [ "json", "jsonl", "csv" ]
  {
    let format_arg = format!( "format::{format}" );

    let listed = run_clj( &[ ".list", format_arg.as_str() ], dir.path() );
    assert!( listed.status.success(), "`.list format::{format}` failed: {}", stderr_str( &listed ) );

    let exported_path = out_dir.path().join( format!( "export.{format}" ) );
    let output_arg    = format!( "output::{}", exported_path.display() );
    let exported = run_clj( &[ ".export", format_arg.as_str(), output_arg.as_str() ], dir.path() );
    assert!( exported.status.success(), "`.export format::{format}` failed: {}", stderr_str( &exported ) );

    let from_file = std::fs::read_to_string( &exported_path ).expect( "read exported file" );
    assert_eq!(
      stdout_str( &listed ).trim_end(),
      from_file.trim_end(),
      "`.list format::{format}` and `.export format::{format}` disagree",
    );
  }
}

/// EC-33 — `limit::` caps after sorting, and `limit::0` means unlimited.
///
/// # Root Cause
/// `limit` was handed to `JournalReader::query()`, which caps by *stopping early*.
/// The sort then only ever saw the oldest N events, so `sort::cost reverse::1
/// limit::1` reported the priciest of the first N — not of the journal.
/// Separately, `limit::0` reached `query()` as `Some( 0 )`, which stops before
/// collecting anything, so the documented "unlimited" returned nothing at all.
///
/// # Why Not Caught
/// `sort::` was unimplemented, so no test could combine it with `limit::`, and no
/// test passed `limit::0` at all.
///
/// # Fix Applied
/// `list_output` queries uncapped, sorts, then truncates; `build_filter` maps
/// `limit::0` to `None`.
///
/// # Prevention
/// The cheapest event is also the *last* appended, so a cap applied before the
/// sort keeps it out of a `limit::1` result and the assertion fails.
///
/// # Pitfall
/// "Cap" and "stop early" are the same thing only when the output order is the
/// order the source is read in. Any sort between the two breaks that equivalence.
#[ test ]
fn ec33_limit_applies_after_sort_and_zero_means_unlimited()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_sortable_events( dir.path() );

  // The single most expensive event across the whole journal — 0.90, appended
  // second. A cap applied inside the query would yield 0.05, the first appended.
  let top = list_json( &[ "sort::cost", "reverse::1", "limit::1" ], dir.path() );
  assert_eq!( top.len(), 1, "limit::1 must return exactly one event" );
  assert_eq!( field_seq( &top, "cost_usd" ), vec![ "0.9" ], "limit::1 must keep the sort's first event" );

  // The cheapest is the last appended, so the same check in the other direction
  // fails too if the cap runs first.
  let bottom = list_json( &[ "sort::cost", "limit::1" ], dir.path() );
  assert_eq!( field_seq( &bottom, "cost_usd" ), vec![ "0.01" ], "ascending limit::1 must keep the cheapest" );

  // `limit::0` is documented as unlimited, not as an empty result.
  assert_eq!( list_json( &[ "limit::0" ], dir.path() ).len(), 3, "limit::0 must return every event" );
}

/// Spawn `clj .tail <extra…>` against `dir`, with stdout piped.
///
/// The caller is responsible for killing the child — `.tail` never exits on its
/// own. Both helpers below do so via [`kill_child`].
fn spawn_tail( extra : &[ &str ], dir : &Path ) -> std::process::Child
{
  assert_container();
  Command::new( CLJ )
    .arg( ".tail" )
    .arg( format!( "journal_dir::{}", dir.display() ) )
    .args( extra )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "NO_COLOR" )
    .stdout( Stdio::piped() )
    .stderr( Stdio::piped() )
    .spawn()
    .expect( "failed to spawn clj .tail" )
}

/// Kill `child` and reap it, ignoring an already-exited process.
fn kill_child( child : &mut std::process::Child )
{
  child.kill().ok();
  child.wait().ok();
}

/// EC-34 — `.tail format::` renders each format, and rejects a bad one before blocking.
///
/// Both halves are bounded in wall-clock time on purpose. `.tail` blocks forever
/// by design, so the failure mode of every regression here is a hang, and a hung
/// test reports nothing at all — it just stalls the suite until the runner's own
/// timeout kills it, with no message naming what broke.
#[ test ]
fn ec34_tail_format_renders_and_rejects_before_blocking()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_sortable_events( dir.path() );

  // An invalid format must be caught *before* the follow loop starts. If it were
  // only caught when rendering the first event, a journal that never gets another
  // write would leave the user waiting on an error that was already knowable.
  let mut child   = spawn_tail( &[ "format::bogus" ], dir.path() );
  let mut exited  = None;
  for _ in 0..50
  {
    if let Some( status ) = child.try_wait().expect( "try_wait failed" ) { exited = Some( status ); break; }
    std::thread::sleep( core::time::Duration::from_millis( 100 ) );
  }
  let status = exited.unwrap_or_else( ||
  {
    kill_child( &mut child );
    panic!( "`.tail format::bogus` did not exit within 5s — the format is being validated too late, or not at all" );
  } );
  assert!( !status.success(), "`.tail format::bogus` must exit non-zero" );
  let mut stderr = String::new();
  std::io::Read::read_to_string( child.stderr.as_mut().expect( "no stderr pipe" ), &mut stderr ).ok();
  assert!( stderr.contains( "invalid format" ), "diagnostic must name the problem: {stderr}" );
  assert!( stderr.contains( "bogus" ),          "diagnostic must name the offending value: {stderr}" );

  // `tail()` replays the current day's file from its start, so the fixture events
  // arrive without anything being appended after the spawn.
  for format in [ "jsonl", "json", "csv" ]
  {
    let format_arg = format!( "format::{format}" );
    let mut child  = spawn_tail( &[ format_arg.as_str() ], dir.path() );
    let stdout     = child.stdout.take().expect( "no stdout pipe" );

    // Read on a worker thread: `read_line` on a pipe blocks, and the whole point
    // of this test is that a broken build must fail rather than block the suite.
    let ( tx, rx ) = std::sync::mpsc::channel();
    std::thread::spawn( move ||
    {
      let mut line = String::new();
      let read = std::io::BufRead::read_line( &mut std::io::BufReader::new( stdout ), &mut line );
      tx.send( read.map( | _ | line ) ).ok();
    } );

    let first = rx.recv_timeout( core::time::Duration::from_secs( 10 ) );
    kill_child( &mut child );
    let line = first
      .unwrap_or_else( | _ | panic!( "`.tail format::{format}` produced no line within 10s" ) )
      .expect( "reading .tail stdout failed" );

    if format == "csv"
    {
      assert!(
        line.starts_with( "ts,type,command" ),
        "`.tail format::csv` must lead with its header row, got: {line}",
      );
      continue;
    }

    // The claim under test for `json`: on a stream it is jsonl, so the first line
    // is a complete standalone object — not `[` opening an array that would never
    // be closed.
    let value : serde_json::Value = serde_json::from_str( line.trim() )
      .unwrap_or_else( | e | panic!( "`.tail format::{format}` line is not standalone JSON ({e}): {line}" ) );
    assert!( value.is_object(), "`.tail format::{format}` must emit one object per line, got: {value}" );
    assert!( value.get( "type" ).is_some(), "`.tail format::{format}` object missing 'type': {value}" );
  }
}

// ── EC-35 : .status verbosity:: renders three levels and clamps above 2 ───────

/// Write two dated journal files with exactly known, clearly distinct sizes.
///
/// Sizes are chosen so the two per-file figures and the total all render
/// differently — `128 B`, `5.0 KB`, `5.1 KB`. A fixture where any two collided
/// would let a breakdown that printed the total on every row pass.
///
/// Neither file is today's, so the reported date range is fixed rather than
/// drifting with the calendar.
fn write_sized_journal_files( dir : &Path )
{
  // 8 bytes per line, so a line count sets the byte count exactly.
  let line = "{\"v\":1}\n";
  std::fs::write( dir.join( "2020-01-01.jsonl" ), line.repeat( 16 ) ).unwrap();
  std::fs::write( dir.join( "2020-06-15.jsonl" ), line.repeat( 640 ) ).unwrap();
}

/// EC-35 — `.status verbosity::` selects among three documented detail levels.
#[ test ]
fn ec35_status_verbosity_levels_and_clamping()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_sized_journal_files( dir.path() );

  let run = | args : &[ &str ] |
  {
    let out = run_clj( args, dir.path() );
    assert!( out.status.success(), "`.status {}` failed: {}", args.join( " " ), stderr_str( &out ) );
    stdout_str( &out )
  };

  // Level 0 — one line, and specifically *not* the multi-line report.
  let compact = run( &[ ".status", "verbosity::0" ] );
  assert_eq!( compact.trim().lines().count(), 1, "verbosity::0 must be one line: {compact}" );
  assert!( compact.contains( "2 files" ),  "missing file count: {compact}" );
  assert!( compact.contains( "5.1 KB" ),   "missing total size: {compact}" );
  assert!( compact.contains( "2020-01-01 to 2020-06-15" ), "missing date range: {compact}" );
  assert!( !compact.contains( "Journal directory:" ), "verbosity::0 must not print the full report: {compact}" );

  // Level 1 is the documented default, so it must be exactly what bare `.status` prints.
  assert_eq!( run( &[ ".status", "verbosity::1" ] ), run( &[ ".status" ] ),
    "verbosity::1 must be identical to the default" );

  // Level 2 — the level-1 report plus one row per file, each with its own size.
  let detailed = run( &[ ".status", "verbosity::2" ] );
  assert!( detailed.contains( "Total size: 5.1 KB" ), "level 2 must keep the totals: {detailed}" );
  assert!( detailed.contains( "DATE" ) && detailed.contains( "SIZE" ), "missing breakdown header: {detailed}" );
  for ( date, size ) in [ ( "2020-01-01", "128 B" ), ( "2020-06-15", "5.0 KB" ) ]
  {
    let row = detailed.lines().find( | l | l.starts_with( date ) )
      .unwrap_or_else( || panic!( "no breakdown row for {date}: {detailed}" ) );
    assert!( row.contains( size ), "row for {date} must show its own size {size}, got: {row}" );
  }

  // Above the documented range clamps to 2 rather than erroring — asking for more
  // detail than exists is answered by the most detailed level there is.
  assert_eq!( run( &[ ".status", "verbosity::9" ] ), detailed, "verbosity::9 must clamp to level 2" );

  // Clamping applies to too-large values only. Garbage and negatives are typos,
  // and `docs/cli/type/04_integer.md` makes both exit 1 — without these two the
  // clamp above would be indistinguishable from "accept anything".
  for bad in [ "verbosity::abc", "verbosity::-1", "verbosity::1.5" ]
  {
    let out = run_clj( &[ ".status", bad ], dir.path() );
    assert!( !out.status.success(), "`.status {bad}` must exit non-zero" );
    let stderr = stderr_str( &out );
    assert!( stderr.contains( "invalid integer" ), "wrong diagnostic for {bad}: {stderr}" );
    assert!( stderr.contains( "verbosity" ), "diagnostic must name the param for {bad}: {stderr}" );
  }

  // An empty journal still reports at every level, and level 2 says so outright
  // rather than printing a `DATE`/`SIZE` header above zero rows.
  let empty = tempfile::TempDir::new().unwrap();
  let out   = run_clj( &[ ".status", "verbosity::2" ], empty.path() );
  assert!( out.status.success(), "`.status` on an empty journal must succeed: {}", stderr_str( &out ) );
  let empty_detailed = stdout_str( &out );
  assert!( empty_detailed.contains( "Files: 0" ),        "missing zero count: {empty_detailed}" );
  assert!( empty_detailed.contains( "Date range: no events" ), "an empty journal has no range: {empty_detailed}" );
  assert!( empty_detailed.contains( "(no journal files)" ),    "missing empty-breakdown notice: {empty_detailed}" );
  assert!( !empty_detailed.contains( "DATE" ), "no column header without rows: {empty_detailed}" );

  let empty_compact = stdout_str( &run_clj( &[ ".status", "verbosity::0" ], empty.path() ) );
  assert!( empty_compact.contains( "0 files" ) && empty_compact.contains( "no events" ),
    "level 0 must degrade cleanly on an empty journal: {empty_compact}" );
}

// ── EC-36 : every documented Boolean param takes 0/1 and nothing else ─────────

/// EC-36 — the `Boolean` contract holds at every site that claims it.
///
/// ## Root Cause
/// `docs/cli/type/08_boolean.md` has always said only `0` and `1` are accepted,
/// but four of the five read sites did not enforce it. Three of them
/// (`.serve open::`, `.chart open::`, `no_color::`) matched `"1" | "true"` and
/// treated everything else as `false`, so `open::banana` exited 0 having done
/// nothing. The fourth (`.prune dry_run::`) did reject, but on a wider grammar
/// (`true`/`false` too) and with a message of its own invention.
///
/// ## Why Not Caught
/// Each site had a case proving the *enabled* path worked — `open::1` opens,
/// `dry_run::1` previews. None passed a value outside the grammar, so the
/// silent-false branch was never once observed.
///
/// ## Prevention
/// Every parameter the type page lists is exercised with values outside the
/// grammar, asserting the documented sentence verbatim rather than a substring
/// — and with `0`/`1`, so a parser that rejects everything cannot pass either.
#[ test ]
fn ec36_boolean_params_accept_only_0_and_1()
{
  let probe = | command : &str, param : &str, value : &str | -> std::process::Output
  {
    let dir = tempfile::TempDir::new().unwrap();
    write_fixture_events( dir.path() );

    let mut args : Vec< String > = vec![ command.to_owned(), format!( "{param}::{value}" ) ];
    // `.chart` defaults to `usage.svg` in the process cwd; keep the artifact
    // inside the temp dir so a passing run leaves nothing behind.
    if command == ".chart" { args.push( format!( "out::{}", dir.path().join( "ec36.svg" ).display() ) ); }

    let refs : Vec< &str > = args.iter().map( String::as_str ).collect();
    run_clj( &refs, dir.path() )
  };

  // Each parameter in `docs/cli/type/08_boolean.md`'s Referenced Parameters
  // table, reached through a command that reads it. `open` also appears on
  // `.serve`, which needs a bound port — that side is serve_test's FT-14.
  for ( command, param ) in [ ( ".list", "reverse" ), ( ".prune", "dry_run" ), ( ".list", "no_color" ), ( ".chart", "open" ) ]
  {
    // `true`/`false` are in the list deliberately: they used to be accepted at
    // some sites and silently ignored at others, which is the divergence this
    // case pins shut.
    for bad in [ "true", "false", "yes", "banana", "2", "-1", "" ]
    {
      let out = probe( command, param, bad );
      assert_eq!(
        out.status.code(), Some( 1 ),
        "`{command} {param}::{bad}` must exit 1, got {:?}", out.status.code(),
      );
      let want = format!( "Error: invalid boolean '{bad}' for parameter '{param}' — expected 0 or 1" );
      assert!(
        stderr_str( &out ).contains( &want ),
        "`{command} {param}::{bad}` must print the documented message.\n  want: {want}\n  got:  {}",
        stderr_str( &out ).trim(),
      );
    }

    for good in [ "0", "1" ]
    {
      let out = probe( command, param, good );
      assert!(
        out.status.success(),
        "`{command} {param}::{good}` is inside the documented grammar and must succeed: {}",
        stderr_str( &out ).trim(),
      );
    }
  }
}

// ── EC-37 : every documented Integer param honours its documented range ──────

/// EC-37 — the `Integer` contract holds at every site that claims it.
///
/// ## Root Cause
/// `docs/cli/type/04_integer.md` specifies one message and a non-negative
/// domain, but `limit` and `exit` each carried ad-hoc wording, and `exit`
/// parsed as `i32` — so `exit::-1` was accepted and then matched nothing,
/// reading as "no failures" rather than "that is not an exit code".
///
/// ## Why Not Caught
/// The existing parse-time case asserted only `!status.success()` for one
/// non-numeric input per parameter. A wrong-but-present message and an
/// out-of-domain value that parses cleanly both satisfy that.
///
/// ## Prevention
/// Each parameter is checked against the documented sentence verbatim, and
/// `exit` additionally against its documented 0-255 ceiling — the bound that
/// made a negative value meaningful in the first place.
#[ test ]
fn ec37_integer_params_honour_documented_domain()
{
  let probe = | command : &str, param : &str, value : &str | -> std::process::Output
  {
    let dir = tempfile::TempDir::new().unwrap();
    write_fixture_events( dir.path() );
    run_clj( &[ command, &format!( "{param}::{value}" ) ], dir.path() )
  };

  // `refresh` is the fourth parameter on the type page; it is `.serve`-only and
  // is covered by serve_test's FT-11.
  for ( command, param ) in [ ( ".list", "exit" ), ( ".list", "limit" ), ( ".status", "verbosity" ) ]
  {
    for bad in [ "abc", "-1", "1.5", "" ]
    {
      let out = probe( command, param, bad );
      assert_eq!(
        out.status.code(), Some( 1 ),
        "`{command} {param}::{bad}` must exit 1, got {:?}", out.status.code(),
      );
      let want = format!( "Error: invalid integer '{bad}' for parameter '{param}'" );
      assert!(
        stderr_str( &out ).contains( &want ),
        "`{command} {param}::{bad}` must print the documented message.\n  want: {want}\n  got:  {}",
        stderr_str( &out ).trim(),
      );
    }

    for good in [ "0", "1" ]
    {
      let out = probe( command, param, good );
      assert!(
        out.status.success(),
        "`{command} {param}::{good}` is inside the documented domain and must succeed: {}",
        stderr_str( &out ).trim(),
      );
    }
  }

  // `exit`'s documented range is 0-255 — a Unix wait status is one byte, so 256
  // is not a smaller-than-expected result set, it is not an exit code at all.
  // `verbosity` deliberately clamps instead (EC-35); the two are not the same
  // rule, which is why the ceiling is asserted here and not in the loop above.
  for over in [ "256", "300" ]
  {
    let out = probe( ".list", "exit", over );
    assert_eq!(
      out.status.code(), Some( 1 ),
      "`.list exit::{over}` is above the documented 0-255 range and must exit 1",
    );
  }
  let out = probe( ".list", "exit", "255" );
  assert!( out.status.success(), "`.list exit::255` is the documented ceiling and must succeed: {}", stderr_str( &out ).trim() );
}

/// EC-38 — `.search` reads the prompt (`message`), and reads only the six
/// documented fields.
///
/// ## Root Cause
/// `search_output`'s match set was assembled from what the runner *captures*
/// (`stdout`, `stderr`, `error_message`) plus two identifiers (`model`,
/// `command`). `message` — the prompt the event was launched with, an
/// `EventFields` member since the schema was written and the explicit subject
/// of `feature/001_cli_viewing.md` AC-006 — was never added, so the one query a
/// caller is most likely to type reached no code path.
///
/// ## Why Not Caught
/// A missing field is invisible from outside. `.search` answered exit 0 with
/// `No events matching '<pattern>'` — character-for-character what it says for a
/// phrase genuinely absent from the journal. Every existing `.search` case put
/// its needle in `stdout` and passed.
///
/// ## Fix Applied
/// `message` added to the match set in `output.rs::search_output`, ahead of the
/// captured-output fields.
///
/// ## Prevention
/// The fixture places one phrase in `message` on one event and the *same*
/// phrase in `dir` — filterable, deliberately not searched — on another. The
/// single `1 match` assertion then fails in both directions: `0` if `message`
/// stops being read, `2` if the match set ever silently widens to `dir`. A
/// fixture with the phrase in only one place could not detect the second.
///
/// ## Pitfall
/// `.search` accepts `dir::` as a filter, which makes "`dir` is searchable" an
/// easy assumption. Filtering and matching are different sets: `dir::` narrows
/// which events are considered, and only the six fields above decide whether
/// `pattern` hit.
#[ test ]
fn ec38_search_reads_prompt_and_only_documented_fields()
{
  assert_container();
  let dir    = tempfile::TempDir::new().unwrap();
  let writer = JournalWriter::new( dir.path().to_path_buf() );

  let mut prompt_ev = EventRecord::new( EventType::Execution );
  prompt_ev.fields.command   = Some( "ask".to_owned() );
  prompt_ev.fields.exit_code = Some( 0 );
  prompt_ev.fields.message   = Some( "refactor the parser".to_owned() );
  prompt_ev.fields.stdout    = Some( "done".to_owned() );
  writer.append( &prompt_ev ).expect( "append prompt_ev" );

  let mut dir_ev = EventRecord::new( EventType::Execution );
  dir_ev.fields.command   = Some( "run".to_owned() );
  dir_ev.fields.exit_code = Some( 0 );
  dir_ev.fields.dir       = Some( "/w/refactor the parser".to_owned() );
  dir_ev.fields.stdout    = Some( "done".to_owned() );
  writer.append( &dir_ev ).expect( "append dir_ev" );

  let out = run_clj( &[ ".search", "pattern::refactor the parser" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "1 match" ),
    "expected exactly the prompt event — 0 means `message` went unread, \
     2 means `dir` leaked into the searched set: {stdout}",
  );
  assert!( stdout.contains( "ask" ), "the match must be the prompt event, not the dir event: {stdout}" );

  // The negative half stated on its own, so a regression names its own direction
  // instead of only moving the count above.
  let out = run_clj( &[ ".search", "pattern::/w/refactor" ], dir.path() );
  assert!( out.status.success(), "exit non-zero: {}", stderr_str( &out ) );
  assert!(
    stdout_str( &out ).contains( "No events matching" ),
    "`dir` is a filter, never a searched field: {}", stdout_str( &out ),
  );
}

//! Integration tests for the `clj` binary — EC-1 through EC-29.
//!
//! Each test writes fixture events via `JournalWriter`, runs the `clj` binary
//! against the temporary journal directory, and asserts on stdout/stderr/exit.

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

  let stdout = stdout_str( &out );
  assert!( stdout.contains( "dir:" ),    "missing dir: {stdout}" );
  assert!( stdout.contains( "files:" ),  "missing files: {stdout}" );
  assert!( stdout.contains( "size:" ),   "missing size: {stdout}" );
  assert!( stdout.contains( "oldest:" ), "missing oldest: {stdout}" );
  assert!( stdout.contains( "newest:" ), "missing newest: {stdout}" );
  // Should show at least 1 file
  assert!( stdout.contains( "files:  1" ), "expected 1 file: {stdout}" );
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

// ── EC-16 : .chart open::true — browser-open failure is non-fatal ────────────

#[ test ]
fn ec16_chart_open_true_failure_is_non_fatal()
{
  let dir      = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let outdir   = tempfile::TempDir::new().unwrap();
  let out_path = outdir.path().join( "chart.svg" );

  let out = run_clj( &[ ".chart", &format!( "out::{}", out_path.display() ), "open::true" ], dir.path() );
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

  // A param valid for another command is still rejected for this one.
  let out = run_clj( &[ ".list", "by::model" ], dir.path() );
  assert!( !out.status.success(), "`by::` belongs to .stats, not .list" );

  // An unknown *command* outranks an unknown param — the command is the real error.
  let out = run_clj( &[ ".bogus", "since::1d" ], dir.path() );
  assert!( !out.status.success() );
  assert!(
    stderr_str( &out ).contains( "unknown command" ),
    "unknown command must be reported ahead of its params: {}",
    stderr_str( &out ),
  );

  // A param the docs declare but no code reads gets its own diagnostic: the user
  // followed the documentation, so "unknown" would be a lie about the parameter
  // rather than the truth about the feature.
  let out = run_clj( &[ ".list", "sort::time" ], dir.path() );
  assert!( !out.status.success(), "an unimplemented param must exit non-zero" );
  let stderr = stderr_str( &out );
  assert!( stderr.contains( "not implemented" ), "wrong diagnostic class: {stderr}" );
  assert!( !stderr.contains( "unknown parameter" ), "must not also claim it is unknown: {stderr}" );

  // `.status` accepts the global params and nothing else — verbosity:: is declared
  // in its docs but unimplemented, while journal_dir:: alone succeeds.
  let out = run_clj( &[ ".status", "verbosity::2" ], dir.path() );
  assert!( !out.status.success() );
  assert!( stderr_str( &out ).contains( "not implemented" ), "{}", stderr_str( &out ) );
  assert!( run_clj( &[ ".status" ], dir.path() ).status.success(), "bare .status must still work" );
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

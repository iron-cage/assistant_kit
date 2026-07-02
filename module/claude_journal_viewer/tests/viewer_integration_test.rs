//! Integration tests for the `clj` binary — EC-1 through EC-12.
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

/// Run `clj` with the given args, always appending `dir::<dir>`.
fn run_clj( args : &[ &str ], dir : &Path ) -> std::process::Output
{
  assert_container();
  Command::new( CLJ )
    .args( args )
    .arg( format!( "dir::{}", dir.display() ) )
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
    .arg( format!( "dir::{}", dir.path().display() ) )
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

// ── EC-12 : .serve HTTP GET / returns 200 with text/html ──────────────────────

#[ test ]
fn ec12_serve_http_returns_html()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );

  let mut child = Command::new( CLJ )
    .args( [ ".serve", &format!( "dir::{}", dir.path().display() ), "port::0" ] )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLJ_PORT" )
    .stdout( Stdio::piped() )
    .stderr( Stdio::null() )
    .spawn()
    .expect( "failed to spawn clj .serve" );

  // Read the "Listening on http://localhost:PORT" line from stdout (explicitly
  // flushed by cmd_serve after println).
  use std::io::BufRead;
  let stdout = child.stdout.take().expect( "no stdout pipe" );
  let mut reader = std::io::BufReader::new( stdout );
  let mut line   = String::new();
  reader.read_line( &mut line ).expect( "failed to read server port line" );

  // Parse port from "Listening on http://localhost:PORT"
  let port : u16 = line
    .trim()
    .rsplit( ':' )
    .next()
    .and_then( | s | s.parse().ok() )
    .unwrap_or_else( || panic!( "could not parse port from: '{}'", line.trim() ) );

  // Connect with retries (server may not be ready instantly after printing)
  use std::io::{ Read, Write };
  let mut stream = None;
  for _ in 0..20
  {
    match std::net::TcpStream::connect( format!( "127.0.0.1:{port}" ) )
    {
      Ok( s )  => { stream = Some( s ); break; }
      Err( _ ) => std::thread::sleep( core::time::Duration::from_millis( 50 ) ),
    }
  }
  let mut stream = stream.unwrap_or_else( || panic!( "could not connect to server on port {port}" ) );

  // Send HTTP request
  stream.write_all( b"GET / HTTP/1.0\r\nHost: localhost\r\n\r\n" )
    .expect( "failed to write HTTP request" );
  let mut response = String::new();
  stream.read_to_string( &mut response ).expect( "failed to read HTTP response" );

  // Cleanup
  child.kill().ok();
  child.wait().ok();

  // Assertions
  assert!( response.contains( "200" ), "expected 200 in response:\n{response}" );
  assert!(
    response.to_lowercase().contains( "text/html" ),
    "expected text/html content-type in response:\n{response}"
  );
  assert!(
    response.contains( "CLR Journal" ),
    "expected HTML body with 'CLR Journal' title in response:\n{response}"
  );
}

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
    .arg( format!( "dir::{}", dir.path().display() ) )
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

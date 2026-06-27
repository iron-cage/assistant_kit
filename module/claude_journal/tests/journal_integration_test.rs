//! Integration tests for `claude_journal` — IT-1 through IT-7.
//!
//! Tests cover:
//! - IT-1: `JournalWriter::append()` creates the daily JSONL file on first call
//! - IT-2: `JournalReader::query()` returns all written events in order
//! - IT-3: `JournalFilter::since` trims events by timestamp (older events excluded)
//! - IT-4: Day rotation produces different filenames for different UTC dates
//! - IT-5: Corrupt/partial JSONL lines are skipped; valid lines are returned
//! - IT-6: Concurrent appends from two writers produce valid JSONL (no interleaved lines)
//! - IT-7: `EventRecord::v` equals `1` on all events (schema version invariant)

use claude_journal::{
  EventRecord, EventType,
  JournalFilter, JournalReader, JournalWriter,
};
use core::time::Duration;
use std::{ path::PathBuf, sync::Arc, thread };
use tempfile::TempDir;

// ── IT-1: JournalWriter creates daily file on first append ────────────────────

/// IT-1: `append()` creates the journal directory and the daily `.jsonl` file if absent.
///
/// **Root Cause Coverage:** AC-001 (file creation), AC-010 (dir auto-create)
#[ test ]
fn it1_writer_creates_file_on_first_append()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir = tmp.path().join( "journal" );

  // Directory does not exist yet.
  assert!( !dir.exists(), "journal dir must not exist before first append" );

  let writer = JournalWriter::new( dir.clone() );
  let ev = EventRecord::new( EventType::Execution );
  writer.append( &ev ).expect( "first append must succeed" );

  // Directory and at least one `.jsonl` file must now exist.
  assert!( dir.exists(), "journal dir must be created on first append" );
  let jsonl_files : Vec< _ > = std::fs::read_dir( &dir )
    .expect( "read_dir" )
    .filter_map( | e | e.ok() )
    .filter( | e | e.path().extension().and_then( | x | x.to_str() ) == Some( "jsonl" ) )
    .collect();
  assert!( !jsonl_files.is_empty(), "at least one .jsonl file must exist after first append" );
}

// ── IT-2: JournalReader returns all written events ────────────────────────────

/// IT-2: `query(default_filter)` returns every event written by `JournalWriter`.
///
/// **Root Cause Coverage:** Round-trip write→read for `EventRecord`.
#[ test ]
fn it2_reader_returns_all_written_events()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir = tmp.path().join( "journal" );
  let writer = JournalWriter::new( dir.clone() );

  // Write 5 events with distinct exit codes.
  for code in 0i32..5
  {
    let mut ev = EventRecord::new( EventType::Execution );
    ev.fields.exit_code = Some( code );
    writer.append( &ev ).expect( "append" );
  }

  let reader = JournalReader::open( dir );
  let filter = JournalFilter::default();
  let events = reader.query( &filter );

  assert_eq!( events.len(), 5, "must return all 5 events" );
  let codes : Vec< Option< i32 > > = events.iter().map( | e | e.fields.exit_code ).collect();
  let expected : Vec< Option< i32 > > = ( 0i32..5 ).map( Some ).collect();
  assert_eq!( codes, expected, "exit codes must match insertion order" );
}

// ── IT-3: JournalFilter::since trims old events ───────────────────────────────

/// IT-3: Events with timestamps before the `since` cutoff are excluded by `query()`.
///
/// **Root Cause Coverage:** `JournalFilter::since` filters old events.
#[ test ]
fn it3_filter_since_excludes_old_events()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir = tmp.path().join( "journal" );
  let writer = JournalWriter::new( dir.clone() );

  // Write one event with a timestamp 2 hours in the past.
  let mut old_ev = EventRecord::new( EventType::Execution );
  old_ev.ts = "2000-01-01T00:00:00.000Z".to_owned(); // very old timestamp
  old_ev.fields.exit_code = Some( 99 );
  writer.append( &old_ev ).expect( "append old" );

  // Write one event with a current timestamp.
  let mut new_ev = EventRecord::new( EventType::Execution );
  new_ev.fields.exit_code = Some( 0 );
  writer.append( &new_ev ).expect( "append new" );

  let reader = JournalReader::open( dir );

  // Filter: events from the last 5 minutes only.
  let filter = JournalFilter
  {
    since : Some( Duration::from_secs( 300 ) ),
    ..JournalFilter::default()
  };
  let events = reader.query( &filter );

  assert_eq!( events.len(), 1, "only the recent event must pass the since filter" );
  assert_eq!(
    events[ 0 ].fields.exit_code,
    Some( 0 ),
    "recent event must be returned"
  );
}

// ── IT-4: Day rotation produces correct distinct filenames ────────────────────

/// IT-4: `rotation::date_filename()` generates `YYYY-MM-DD.jsonl` and different
/// calendar dates produce different filenames.
///
/// **Root Cause Coverage:** `docs/feature/003_rotation.md` — daily file rotation.
#[ test ]
fn it4_rotation_date_filename_format()
{
  use claude_journal::rotation::{ date_filename, today_filename };

  let f = date_filename( 2026, 6, 27 );
  assert_eq!( f, "2026-06-27.jsonl", "date_filename must produce YYYY-MM-DD.jsonl" );

  let next_day = date_filename( 2026, 6, 28 );
  assert_ne!( f, next_day, "different days must produce different filenames" );

  // today_filename must produce a string matching YYYY-MM-DD.jsonl format.
  let today = today_filename();
  assert!(
    today.len() == "2026-06-27.jsonl".len(),
    "today_filename must have the correct length"
  );
  assert!( today.ends_with( ".jsonl" ), "today_filename must end with .jsonl" );
  // Verify date part has the right shape: digits and dashes at positions 0-9.
  let date_part = &today[ ..10 ];
  let parts : Vec< &str > = date_part.split( '-' ).collect();
  assert_eq!( parts.len(), 3, "date part must have 3 components separated by '-'" );
  assert_eq!( parts[ 0 ].len(), 4, "year component must be 4 digits" );
  assert_eq!( parts[ 1 ].len(), 2, "month component must be 2 digits" );
  assert_eq!( parts[ 2 ].len(), 2, "day component must be 2 digits" );
}

// ── IT-5: Corrupt JSONL lines are skipped ─────────────────────────────────────

/// IT-5: `query()` silently skips partial or malformed JSONL lines and returns
/// the surrounding valid events.
///
/// **Root Cause Coverage:** `docs/invariant/002_crash_safety.md` — skip-on-parse-failure.
#[ test ]
fn it5_corrupt_lines_are_skipped()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir = tmp.path().join( "journal" );
  std::fs::create_dir_all( &dir ).expect( "create dir" );

  // Create a JSONL file that mixes valid events with corrupt lines.
  let today = claude_journal::rotation::today_filename();
  let path = dir.join( today );

  // Valid event serialized manually.
  let ev1 = EventRecord::new( EventType::Execution );
  let valid1 = serde_json::to_string( &ev1 ).expect( "serialize" );

  let mut ev2 = EventRecord::new( EventType::Retry );
  ev2.fields.attempt = Some( 1 );
  let valid2 = serde_json::to_string( &ev2 ).expect( "serialize" );

  let content = format!( "{valid1}\n{{bad json\n\n{valid2}\n" );
  std::fs::write( &path, content ).expect( "write" );

  let reader = JournalReader::open( dir );
  let filter = JournalFilter::default();
  let events = reader.query( &filter );

  assert_eq!( events.len(), 2, "corrupt line must be skipped; valid events must be returned" );
  assert_eq!( events[ 0 ].event_type, EventType::Execution );
  assert_eq!( events[ 1 ].event_type, EventType::Retry );
}

// ── IT-6: Concurrent appends produce valid JSONL ──────────────────────────────

/// IT-6: Two threads appending concurrently via independent `JournalWriter` instances
/// each targeting the same directory produce a file with no interleaved lines.
/// All appended events must be recoverable by `query()`.
///
/// **Root Cause Coverage:** `docs/api/001_journal_writer.md` thread-safety contract.
#[ test ]
fn it6_concurrent_appends_produce_valid_jsonl()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir : Arc< PathBuf > = Arc::new( tmp.path().join( "journal" ) );

  let count_per_thread : usize = 50;

  let dir_a = Arc::clone( &dir );
  let thread_a = thread::spawn( move ||
  {
    let writer = JournalWriter::new( ( *dir_a ).clone() );
    for _ in 0..count_per_thread
    {
      writer.append( &EventRecord::new( EventType::Execution ) ).expect( "thread_a append" );
    }
  } );

  let dir_b = Arc::clone( &dir );
  let thread_b = thread::spawn( move ||
  {
    let writer = JournalWriter::new( ( *dir_b ).clone() );
    for _ in 0..count_per_thread
    {
      writer.append( &EventRecord::new( EventType::Retry ) ).expect( "thread_b append" );
    }
  } );

  thread_a.join().expect( "thread_a panicked" );
  thread_b.join().expect( "thread_b panicked" );

  // All events must be recoverable.
  let reader = JournalReader::open( ( *dir ).clone() );
  let filter = JournalFilter::default();
  let events = reader.query( &filter );

  let total = count_per_thread * 2;
  assert_eq!(
    events.len(),
    total,
    "all {total} events from both threads must be present; got {}",
    events.len()
  );

  let execution_count =
    events.iter().filter( | e | e.event_type == EventType::Execution ).count();
  let retry_count =
    events.iter().filter( | e | e.event_type == EventType::Retry ).count();
  assert_eq!( execution_count, count_per_thread, "execution event count must match" );
  assert_eq!( retry_count,     count_per_thread, "retry event count must match" );
}

// ── IT-7: Schema version field equals 1 ──────────────────────────────────────

/// IT-7: Every event appended by `JournalWriter` has `v == 1` in both the
/// deserialized struct and the raw JSONL bytes.
///
/// **Root Cause Coverage:** `docs/invariant/003_schema_version.md` — `v:1` invariant.
#[ test ]
fn it7_schema_version_is_one_on_all_events()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir = tmp.path().join( "journal" );
  let writer = JournalWriter::new( dir.clone() );

  // Write one of each event type.
  let types = [
    EventType::Execution,
    EventType::Credential,
    EventType::GateWait,
    EventType::Retry,
    EventType::Timeout,
    EventType::RunnerRetry,
    EventType::ValidationRetry,
    EventType::Interactive,
  ];

  for et in types
  {
    let ev = EventRecord::new( et );
    writer.append( &ev ).expect( "append" );
  }

  // Via deserialized struct.
  let reader = JournalReader::open( dir.clone() );
  let filter = JournalFilter::default();
  let events = reader.query( &filter );

  assert_eq!( events.len(), types.len(), "all event types must be written" );
  for ev in &events
  {
    assert_eq!( ev.v, 1, "schema version must be 1 for event {:?}", ev.event_type );
  }

  // Via raw JSONL bytes — every line must contain `"v":1`.
  let today = claude_journal::rotation::today_filename();
  let raw = std::fs::read_to_string( dir.join( today ) ).expect( "read raw" );
  for line in raw.lines().filter( | l | !l.trim().is_empty() )
  {
    assert!(
      line.contains( "\"v\":1" ),
      "raw JSONL line must contain \"v\":1: {line}"
    );
  }
}

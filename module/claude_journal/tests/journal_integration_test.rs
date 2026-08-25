//! Integration tests for `claude_journal` — IT-1 through IT-25.
//!
//! Tests cover:
//! - IT-1: `JournalWriter::append()` creates the daily JSONL file on first call
//! - IT-2: `JournalReader::query()` returns all written events in order
//! - IT-3: `JournalFilter::since` trims events by timestamp (older events excluded)
//! - IT-4: Day rotation produces different filenames for different UTC dates
//! - IT-5: Corrupt/partial JSONL lines are skipped; valid lines are returned
//! - IT-6: Concurrent appends from two writers produce valid JSONL (no interleaved lines)
//! - IT-7: `EventRecord::v` equals `1` on all events (schema version invariant)
//! - IT-8: `EventType::Command.as_str()` returns `"command"`
//! - IT-9: `EventType::parse("command")` returns `Some(EventType::Command)`
//! - IT-10: `EventType::parse("bogus")` returns `None` (regression guard)
//! - IT-11: `EventFields::default()` leaves `user`/`host`/`args` all `None`
//! - IT-12: `user`/`host`/`args` serialize with correct values when `Some`
//! - IT-13: `user`/`host`/`args` are omitted from JSON when `None`
//! - IT-14: Existing 8 `EventType` variants' `as_str()` strings are unchanged
//! - IT-15: `tail()` yields every event of a multi-line append batch, in order
//! - IT-16: `tail()` defers a torn (partially-written) line and delivers it once completed
//! - IT-17: Unparseable `ts` excluded under `since`/`until`, included when unbounded
//! - IT-18: `since: Duration::MAX` degrades to unbounded instead of panicking
//! - IT-19: `account`/`agent_id` serialize with correct values when `Some`
//! - IT-20: `account`/`agent_id` are omitted from JSON when `None`
//! - IT-21: Legacy JSONL line without `account`/`agent_id` still deserializes
//! - IT-22: `compose_agent_id()` produces the exact `{user}@{host}{abs_dir}/` format
//! - IT-23: `compose_agent_id()` never double-slashes an already-slashed dir
//! - IT-24: Metadata accessors reflect real journal state (AP-6)
//! - IT-25: `files()` is oldest-first and the scalar accessors agree with it (AP-7)

use claude_journal::{
  compose_agent_id,
  EventFields, EventRecord, EventType,
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
    .filter_map( core::result::Result::ok )
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
  assert!(
    std::path::Path::new( &today ).extension().is_some_and( | ext | ext.eq_ignore_ascii_case( "jsonl" ) ),
    "today_filename must end with .jsonl"
  );
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

// ── IT-8: Command variant → string ────────────────────────────────────────────

/// IT-8: `EventType::Command.as_str()` returns exactly `"command"`.
///
/// **Root Cause Coverage:** T01 — Command variant string mapping.
#[ test ]
fn it8_command_variant_as_str_returns_command()
{
  assert_eq!( EventType::Command.as_str(), "command", "Command variant must serialize as \"command\"" );
}

// ── IT-9: String → Command variant ────────────────────────────────────────────

/// IT-9: `EventType::parse("command")` returns `Some(EventType::Command)`.
///
/// **Root Cause Coverage:** T02 — Command variant parse round-trip.
#[ test ]
fn it9_parse_command_string_returns_command_variant()
{
  assert_eq!( EventType::parse( "command" ), Some( EventType::Command ), "\"command\" must parse to Some(EventType::Command)" );
}

// ── IT-10: Unknown string still rejected ──────────────────────────────────────

/// IT-10: `EventType::parse("bogus")` returns `None` (regression guard — unrecognized
/// strings must still be rejected after adding the `Command` variant).
///
/// **Root Cause Coverage:** T03 — `parse()` forward-compat rejection unaffected.
#[ test ]
fn it10_parse_unknown_string_returns_none()
{
  assert_eq!( EventType::parse( "bogus" ), None, "unrecognized strings must still parse to None" );
}

// ── IT-11: New fields default to None ─────────────────────────────────────────

/// IT-11: `EventFields::default()` leaves `user`, `host`, `args` all `None`.
///
/// **Root Cause Coverage:** T04 — new fields covered by existing `#[derive(Default)]`.
#[ test ]
fn it11_new_fields_default_to_none()
{
  let fields = EventFields::default();
  assert_eq!( fields.user, None, "user must default to None" );
  assert_eq!( fields.host, None, "host must default to None" );
  assert_eq!( fields.args, None, "args must default to None" );
}

// ── IT-12: New fields serialize when present ──────────────────────────────────

/// IT-12: `user`, `host`, `args` serialize with their correct values when `Some`.
///
/// **Root Cause Coverage:** T05 — new field serialization.
#[ test ]
fn it12_new_fields_serialize_when_present()
{
  let fields = EventFields
  {
    user : Some( "i4".to_owned() ),
    host : Some( "nodeB".to_owned() ),
    args : Some( vec![ "--foo".to_owned(), "bar".to_owned() ] ),
    ..EventFields::default()
  };
  let json = serde_json::to_value( &fields ).expect( "serialize" );

  assert_eq!( json[ "user" ], "i4", "user key must be present with correct value" );
  assert_eq!( json[ "host" ], "nodeB", "host key must be present with correct value" );
  assert_eq!( json[ "args" ], serde_json::json!( [ "--foo", "bar" ] ), "args key must be present with correct value" );
}

// ── IT-13: New fields omitted when None ───────────────────────────────────────

/// IT-13: `user`, `host`, `args` are omitted entirely (not serialized as `null`)
/// from JSON output when `None`, matching every other `Option` field's convention.
///
/// **Root Cause Coverage:** T06 — omit-when-None serialization for new fields.
#[ test ]
fn it13_new_fields_omitted_when_none()
{
  let fields = EventFields::default();
  let json = serde_json::to_value( &fields ).expect( "serialize" );

  assert!( json.get( "user" ).is_none(), "user key must be omitted when None" );
  assert!( json.get( "host" ).is_none(), "host key must be omitted when None" );
  assert!( json.get( "args" ).is_none(), "args key must be omitted when None" );
}

// ── IT-14: Existing 8 variants unaffected ─────────────────────────────────────

/// IT-14: All 8 pre-existing `EventType` variants keep their exact prior
/// `as_str()` strings after adding `Command` (regression guard).
///
/// **Root Cause Coverage:** T07 — existing variant strings unaffected.
#[ test ]
fn it14_existing_variants_as_str_unchanged()
{
  assert_eq!( EventType::Execution.as_str(),      "execution" );
  assert_eq!( EventType::Credential.as_str(),     "credential" );
  assert_eq!( EventType::GateWait.as_str(),       "gate_wait" );
  assert_eq!( EventType::Retry.as_str(),          "retry" );
  assert_eq!( EventType::Timeout.as_str(),        "timeout" );
  assert_eq!( EventType::RunnerRetry.as_str(),    "runner_retry" );
  assert_eq!( EventType::ValidationRetry.as_str(), "validation_retry" );
  assert_eq!( EventType::Interactive.as_str(),    "interactive" );
}

// ── IT-15: tail yields every event of a multi-line batch ──────────────────────

/// IT-15: `tail()` yields ALL events appended between polls, not just the first.
///
/// # Root Cause (audit-tail-data-loss)
///
/// `TailIter::next()` advanced `offset` to the file size before iterating the
/// read batch, then returned on the first matching event — every remaining line
/// of that same batch sat behind the already-advanced offset and was permanently
/// lost.
///
/// # Why Not Caught
///
/// `TailIter` had no tests at all; `query()` tests never exercise the polling
/// path, and single-event manual checks can't reveal a batch-local loss.
///
/// # Fix Applied
///
/// `offset` now advances only past lines actually consumed; a mid-batch return
/// leaves the rest of the batch unread, so subsequent `next()` calls deliver it.
///
/// # Prevention
///
/// Any cursor a reader persists must track what was consumed, not what was
/// observed — never advance past data that hasn't been handed to the caller.
///
/// # Pitfall
///
/// All 3 events must be in the file BEFORE the first poll so they land in one
/// read batch — appending after tail starts can split them across polls and
/// mask the bug.
#[ test ]
fn it15_tail_yields_all_events_of_a_batch()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir = tmp.path().join( "journal" );
  let writer = JournalWriter::new( dir.clone() );
  for code in 0i32..3
  {
    let mut ev = EventRecord::new( EventType::Execution );
    ev.fields.exit_code = Some( code );
    writer.append( &ev ).expect( "append" );
  }

  let ( tx, rx ) = std::sync::mpsc::channel();
  let handle = thread::spawn( move ||
  {
    let reader = JournalReader::open( dir );
    let filter = JournalFilter::default();
    for event in reader.tail( &filter ).take( 3 )
    {
      tx.send( event ).expect( "send" );
    }
  } );

  let mut codes = Vec::new();
  for _ in 0..3
  {
    let ev = rx.recv_timeout( Duration::from_secs( 10 ) )
      .expect( "tail must yield all 3 batch events (first-match-only loss if this times out)" );
    codes.push( ev.fields.exit_code );
  }
  handle.join().expect( "tail thread" );
  assert_eq!( codes, vec![ Some( 0 ), Some( 1 ), Some( 2 ) ], "batch events must arrive in order" );
}

// ── IT-16: tail defers a torn line until the writer completes it ──────────────

/// IT-16: a partially-written trailing line is not consumed; the event is
/// delivered intact once the writer finishes the line.
///
/// # Root Cause (audit-tail-data-loss)
///
/// `TailIter::next()` set `offset = size` after reading, even when the read
/// ended mid-line (writer between `write_all` chunks, or reader racing a
/// non-atomic append). The torn fragment failed to parse and the completed
/// line was never re-read — the event vanished.
///
/// # Why Not Caught
///
/// The race window is a few microseconds in production; only a deliberately
/// half-written file makes it deterministic. No such test existed.
///
/// # Fix Applied
///
/// Only complete (`\n`-terminated) lines are consumed; `offset` stays at the
/// start of a partial trailing line so the next poll re-reads it whole.
///
/// # Prevention
///
/// Treat "bytes read" and "records consumed" as different quantities in any
/// incremental parser; commit the cursor per record, not per read.
///
/// # Pitfall
///
/// The half-line must stay incomplete across at least one full poll interval
/// (~500 ms) before being finished — completing it immediately can land both
/// halves in the first read and never exercise the deferral path.
#[ test ]
fn it16_tail_defers_torn_line_until_completed()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir = tmp.path().join( "journal" );
  std::fs::create_dir_all( &dir ).expect( "create dir" );
  let path = dir.join( claude_journal::rotation::today_filename() );

  // One complete event + the first half of a second event (no trailing newline).
  let mut ev1 = EventRecord::new( EventType::Execution );
  ev1.fields.exit_code = Some( 1 );
  let line1 = serde_json::to_string( &ev1 ).expect( "serialize" );
  let mut ev2 = EventRecord::new( EventType::Retry );
  ev2.fields.exit_code = Some( 2 );
  let line2 = serde_json::to_string( &ev2 ).expect( "serialize" );
  let ( half_a, half_b ) = line2.split_at( line2.len() / 2 );
  std::fs::write( &path, format!( "{line1}\n{half_a}" ) ).expect( "write torn" );

  let ( tx, rx ) = std::sync::mpsc::channel();
  let dir_clone = dir.clone();
  let handle = thread::spawn( move ||
  {
    let reader = JournalReader::open( dir_clone );
    let filter = JournalFilter::default();
    for event in reader.tail( &filter ).take( 2 )
    {
      tx.send( event ).expect( "send" );
    }
  } );

  let first = rx.recv_timeout( Duration::from_secs( 10 ) )
    .expect( "complete first line must arrive" );
  assert_eq!( first.fields.exit_code, Some( 1 ) );

  // Let the tail loop poll at least once while the torn line is still incomplete,
  // then finish the line.
  thread::sleep( Duration::from_millis( 1200 ) );
  {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new().append( true ).open( &path ).expect( "reopen" );
    f.write_all( format!( "{half_b}\n" ).as_bytes() ).expect( "complete line" );
  }

  let second = rx.recv_timeout( Duration::from_secs( 10 ) )
    .expect( "completed torn line must be delivered (lost if offset advanced past it)" );
  assert_eq!( second.fields.exit_code, Some( 2 ), "the torn-then-completed event must arrive intact" );
  handle.join().expect( "tail thread" );
}

// ── IT-17: unparseable timestamp excluded under a time bound ──────────────────

/// IT-17: an event whose `ts` fails RFC 3339 parsing is EXCLUDED from any
/// time-bounded query (`since`/`until` set) — and still returned when no time
/// bound is active.
///
/// **Root Cause Coverage:** Fix(audit-ts-filter-bypass) — corrupt-ts events used
/// to skip the time check entirely and leak into "last N minutes" queries.
#[ test ]
fn it17_unparseable_ts_excluded_under_time_bound()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir = tmp.path().join( "journal" );
  let writer = JournalWriter::new( dir.clone() );

  let mut corrupt = EventRecord::new( EventType::Execution );
  corrupt.ts = "not-a-timestamp".to_owned();
  corrupt.fields.exit_code = Some( 77 );
  writer.append( &corrupt ).expect( "append corrupt-ts" );

  let mut ok = EventRecord::new( EventType::Execution );
  ok.fields.exit_code = Some( 0 );
  writer.append( &ok ).expect( "append valid" );

  let reader = JournalReader::open( dir );

  // Unbounded query: both events visible (corrupt ts is not a parse failure of the line).
  let all = reader.query( &JournalFilter::default() );
  assert_eq!( all.len(), 2, "without a time bound the corrupt-ts event must still be returned" );

  // Time-bounded query: the corrupt-ts event must be excluded.
  let filter = JournalFilter
  {
    since : Some( Duration::from_secs( 300 ) ),
    ..JournalFilter::default()
  };
  let bounded = reader.query( &filter );
  assert_eq!( bounded.len(), 1, "corrupt-ts event must not leak into a time-bounded query" );
  assert_eq!( bounded[ 0 ].fields.exit_code, Some( 0 ) );
}

// ── IT-18: huge `since` duration must not panic ───────────────────────────────

/// IT-18: `since` larger than the representable `SystemTime` range degrades to
/// "no lower bound" (all events returned) instead of panicking.
///
/// **Root Cause Coverage:** `now - d` panicked on underflow; now `checked_sub`.
#[ test ]
fn it18_huge_since_duration_does_not_panic()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir = tmp.path().join( "journal" );
  let writer = JournalWriter::new( dir.clone() );
  writer.append( &EventRecord::new( EventType::Execution ) ).expect( "append" );

  let reader = JournalReader::open( dir );
  let filter = JournalFilter
  {
    since : Some( Duration::MAX ),
    ..JournalFilter::default()
  };
  let events = reader.query( &filter );
  assert_eq!( events.len(), 1, "a since window larger than all of time must include every event" );
}

// ── IT-19: account/agent_id serialize when present ────────────────────────────

/// IT-19: `account` and `agent_id` serialize with their correct values when `Some`.
///
/// **Root Cause Coverage:** TSK-541 — attribution field serialization.
#[ test ]
fn it19_attribution_fields_serialize_when_present()
{
  let fields = EventFields
  {
    account  : Some( "alice@example.com".to_owned() ),
    agent_id : Some( "alice@devbox/a/b/".to_owned() ),
    ..EventFields::default()
  };
  let json = serde_json::to_value( &fields ).expect( "serialize" );

  assert_eq!( json[ "account" ], "alice@example.com", "account key must be present with correct value" );
  assert_eq!( json[ "agent_id" ], "alice@devbox/a/b/", "agent_id key must be present with correct value" );
}

// ── IT-20: account/agent_id omitted when None ─────────────────────────────────

/// IT-20: `account` and `agent_id` are omitted entirely (not serialized as
/// `null`) from JSON output when `None`, matching every other `Option` field.
///
/// **Root Cause Coverage:** TSK-541 — omit-when-None serialization.
#[ test ]
fn it20_attribution_fields_omitted_when_none()
{
  let fields = EventFields::default();
  let json = serde_json::to_value( &fields ).expect( "serialize" );

  assert!( json.get( "account" ).is_none(), "account key must be omitted when None" );
  assert!( json.get( "agent_id" ).is_none(), "agent_id key must be omitted when None" );
}

// ── IT-21: legacy line without attribution fields still parses ────────────────

/// IT-21: A pre-TSK-541 JSONL line (no `account`/`agent_id` keys) deserializes
/// into an `EventRecord` with both fields `None` — additive schema change,
/// backward compatible with every existing journal file.
///
/// **Root Cause Coverage:** TSK-541 — legacy-line backward compatibility.
#[ test ]
fn it21_legacy_line_without_attribution_fields_parses()
{
  let line = r#"{"v":1,"ts":"2026-08-19T12:00:00.000Z","type":"execution","exit_code":0}"#;
  let ev : EventRecord = serde_json::from_str( line ).expect( "legacy line must parse" );

  assert_eq!( ev.fields.account, None, "account must be None on a legacy line" );
  assert_eq!( ev.fields.agent_id, None, "agent_id must be None on a legacy line" );
  assert_eq!( ev.fields.exit_code, Some( 0 ), "legacy fields must survive unchanged" );
}

// ── IT-22: compose_agent_id exact format ──────────────────────────────────────

/// IT-22: `compose_agent_id()` produces exactly `{user}@{host}{abs_dir}/` —
/// no separator between host and dir, exactly one trailing slash.
///
/// **Root Cause Coverage:** TSK-541 — single format owner for agent identity.
#[ test ]
fn it22_compose_agent_id_exact_format()
{
  assert_eq!( compose_agent_id( "alice", "devbox", "/a/b" ), "alice@devbox/a/b/" );
  assert_eq!
  (
    compose_agent_id( "alice", "devbox", "/home/alice/assistant/module/claude_runner" ),
    "alice@devbox/home/alice/assistant/module/claude_runner/",
    "format must match the canonical AGENT_ID shape"
  );
}

// ── IT-23: compose_agent_id never double-slashes ──────────────────────────────

/// IT-23: A `dir` already carrying a trailing slash (or several) yields the
/// same result as its unslashed form — exactly one trailing slash, always.
///
/// **Root Cause Coverage:** TSK-541 — trailing-slash normalization.
#[ test ]
fn it23_compose_agent_id_never_double_slashes()
{
  assert_eq!( compose_agent_id( "alice", "devbox", "/a/b/" ), "alice@devbox/a/b/" );
  assert_eq!( compose_agent_id( "alice", "devbox", "/a/b//" ), "alice@devbox/a/b/" );
}

/// Write `dates` as `.jsonl` files in `dir`, each holding one line of `filler`
/// per index so the files differ in size. Returns the directory.
///
/// Creation order is the caller's, deliberately — IT-25 relies on being able to
/// create files in an order that is *not* their date order.
fn write_dated_files( dir : &std::path::Path, dates : &[ &str ] )
{
  std::fs::create_dir_all( dir ).expect( "create journal dir" );
  for ( i, date ) in dates.iter().enumerate()
  {
    let body = format!( "{{\"v\":1,\"ts\":\"{date}T00:00:00.000Z\",\"type\":\"execution\"}}\n" ).repeat( i + 1 );
    std::fs::write( dir.join( format!( "{date}.jsonl" ) ), body ).expect( "write dated file" );
  }
}

// ── IT-24: metadata accessors reflect real journal state ──────────────────────

/// IT-24 (AP-6): `file_count`/`total_bytes`/`oldest_date`/`newest_date` report
/// the directory's actual contents, and an absent directory degrades to
/// empty/`None`/`0` rather than erroring.
///
/// **Root Cause Coverage:** AP-6 was specified in `tests/docs/api/002_journal_reader.md`
/// but had no implementing test — all four accessors backing `clj .status` were
/// uncovered. Added alongside AP-7 so the shared-`files()` refactor of these
/// four is guarded against behavior drift.
#[ test ]
fn it24_metadata_accessors_reflect_journal_state()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir = tmp.path().join( "journal" );

  // Absent directory: every accessor degrades, none panics.
  let empty = JournalReader::open( dir.clone() );
  assert_eq!( empty.file_count(), 0, "absent dir must count 0 files" );
  assert_eq!( empty.total_bytes(), 0, "absent dir must total 0 bytes" );
  assert_eq!( empty.oldest_date(), None, "absent dir has no oldest date" );
  assert_eq!( empty.newest_date(), None, "absent dir has no newest date" );

  write_dated_files( &dir, &[ "2023-01-01", "2026-06-27" ] );
  let reader = JournalReader::open( dir );

  assert_eq!( reader.file_count(), 2, "must count both .jsonl files" );
  assert_eq!( reader.oldest_date(), Some( "2023-01-01".to_owned() ), "oldest is the earliest filename" );
  assert_eq!( reader.newest_date(), Some( "2026-06-27".to_owned() ), "newest is the latest filename" );
  assert!( reader.total_bytes() > 0, "non-empty files must total more than zero bytes" );
}

// ── IT-25: files() ordering and cross-accessor agreement ──────────────────────

/// IT-25 (AP-7): `files()` orders by date regardless of filesystem creation
/// order, and the four scalar accessors are all derivable from it.
///
/// **Root Cause Coverage:** `files()` backs `clj .status verbosity::2`. Ordering
/// cannot be left to `read_dir`, which returns entries in an arbitrary,
/// filesystem-dependent order — so the fixture creates files in an order that is
/// deliberately *not* chronological. A test that created them in date order
/// would pass even with the sort removed.
#[ test ]
fn it25_files_is_oldest_first_and_agrees_with_scalar_accessors()
{
  let tmp = TempDir::new().expect( "tempdir" );
  let dir = tmp.path().join( "journal" );

  assert!(
    JournalReader::open( dir.clone() ).files().is_empty(),
    "absent dir must list no files rather than erroring"
  );

  // Deliberately non-chronological creation order.
  write_dated_files( &dir, &[ "2026-06-27", "2023-01-01", "2024-03-15" ] );
  let reader = JournalReader::open( dir.clone() );
  let files  = reader.files();

  let dates : Vec< &str > = files.iter().map( | f | f.date.as_str() ).collect();
  assert_eq!(
    dates, [ "2023-01-01", "2024-03-15", "2026-06-27" ],
    "files() must sort by date, not by filesystem creation order"
  );

  // Every scalar accessor must be derivable from the same listing.
  assert_eq!( files.len(), reader.file_count(), "files().len() must equal file_count()" );
  assert_eq!(
    files.iter().map( | f | f.bytes ).sum::< u64 >(), reader.total_bytes(),
    "summed per-file bytes must equal total_bytes()"
  );
  assert_eq!( files.first().map( | f | f.date.clone() ), reader.oldest_date(), "first entry must be oldest_date()" );
  assert_eq!( files.last().map( | f | f.date.clone() ), reader.newest_date(), "last entry must be newest_date()" );

  // Reported sizes must be the real ones, not a placeholder.
  for f in &files
  {
    let actual = std::fs::metadata( dir.join( format!( "{}.jsonl", f.date ) ) ).expect( "stat" ).len();
    assert_eq!( f.bytes, actual, "{}: reported size must match the real file size", f.date );
  }
}

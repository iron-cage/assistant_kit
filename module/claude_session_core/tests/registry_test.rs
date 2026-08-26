//! Registry parsing and scanning tests.
//!
//! Every scan runs against a real directory of real files in a `TempDir` — never
//! the developer's `~/.claude/sessions/`, and never a mock filesystem.
//!
//! ## Specification References
//!
//! - `docs/feature/001_registry_scan.md` — the scan contract
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | reg01 | A complete record | Every field parsed |
//! | reg02 | `procStart` written as a number | `proc_start` is `None` |
//! | reg03 | Missing `pid` | `None` — unusable record |
//! | reg04 | Missing `sessionId` | `None` — unusable record |
//! | reg05 | A torn write (truncated JSON) | `None`, no panic |
//! | reg06 | Missing `status` | Defaults to `Idle` |
//! | reg07 | Unrecognized `status` | `Other`, value preserved |
//! | reg08 | `scan` of a missing directory | `Ok( [] )` |
//! | reg09 | Non-`.json` files present | Ignored |
//! | reg10 | One corrupt file among good ones | Good records still returned |
//! | reg11 | Several records | Sorted by pid |
//! | reg12 | `scan_live` with a dead pid present | Dead record filtered out |
//! | reg13 | Missing `cwd` | Empty path, not a parse failure |
//! | reg14 | `scan` pointed at a regular file | `Err( ReadDir )` |
//! | reg15 | `is_alive` for this test process | `true` |

use std::fs;
use std::path::Path;

use claude_session_core::{ scan, scan_live, proc_starttime, Error, SessionRecord, SessionStatus };

/// A registry file body with every field Claude Code writes.
fn full_record_json( pid : u32, session_id : &str ) -> String
{
  format!(
    r#"{{
      "pid": {pid},
      "sessionId": "{session_id}",
      "cwd": "/work/project",
      "procStart": "123456789",
      "version": "2.0.30",
      "kind": "interactive",
      "entrypoint": "cli",
      "name": "a session",
      "status": "busy",
      "updatedAt": 1735689600000
    }}"#
  )
}

/// Write `body` to `<dir>/<name>`.
fn write_file( dir : &Path, name : &str, body : &str )
{
  fs::write( dir.join( name ), body ).expect( "cannot write registry file" );
}

/// reg01: a complete record parses field for field.
#[ test ]
fn reg01_full_record_parses()
{
  let record = SessionRecord::parse( &full_record_json( 4242, "abc-123" ) )
    .expect( "a complete record failed to parse" );

  assert_eq!( record.pid, 4242 );
  assert_eq!( record.session_id, "abc-123" );
  assert_eq!( record.cwd, Path::new( "/work/project" ) );
  assert_eq!( record.proc_start, Some( 123_456_789 ) );
  assert_eq!( record.version.as_deref(), Some( "2.0.30" ) );
  assert_eq!( record.kind.as_deref(), Some( "interactive" ) );
  assert_eq!( record.entrypoint.as_deref(), Some( "cli" ) );
  assert_eq!( record.name.as_deref(), Some( "a session" ) );
  assert_eq!( record.status, SessionStatus::Busy );
  assert_eq!( record.updated_at, Some( 1_735_689_600_000 ) );
}

/// reg02: `procStart` is a JSON *string* on disk, not a number.
///
/// Reading it as a number is the mistake this test exists to catch: the record
/// would parse, `proc_start` would silently be `None`, and the incarnation clause
/// in `pid_alive` would go inert — a recycled PID would then read as the original
/// process. A number here is treated as a malformed value, not as a second
/// accepted spelling, so the mismatch surfaces instead of degrading.
#[ test ]
fn reg02_numeric_proc_start_is_not_accepted()
{
  let json = r#"{ "pid": 7, "sessionId": "s", "procStart": 123456789 }"#;
  let record = SessionRecord::parse( json ).expect( "record should still parse" );

  assert_eq!(
    record.proc_start, None,
    "a numeric procStart must not be read as the recorded start time",
  );
}

/// reg03, reg04: the two fields without which a record cannot be used.
#[ test ]
fn reg03_records_missing_identity_fields_are_rejected()
{
  assert!(
    SessionRecord::parse( r#"{ "sessionId": "s", "status": "idle" }"# ).is_none(),
    "a record without a pid was accepted",
  );
  assert!(
    SessionRecord::parse( r#"{ "pid": 7, "status": "idle" }"# ).is_none(),
    "a record without a sessionId was accepted",
  );
}

/// reg05: a torn write is skipped, not fatal.
///
/// Claude Code rewrites these files in place, so a reader can observe a file
/// mid-write. Panicking — or failing the whole scan — would make every other live
/// session invisible for the duration of one unrelated write.
#[ test ]
fn reg05_torn_write_is_skipped()
{
  for body in [ r#"{ "pid": 7, "sessionI"#, "", "not json at all", "[]", "null" ]
  {
    assert!( SessionRecord::parse( body ).is_none(), "accepted malformed body {body:?}" );
  }
}

/// reg06, reg07: status handling.
#[ test ]
fn reg06_status_defaults_and_unknown_values()
{
  let missing = SessionRecord::parse( r#"{ "pid": 7, "sessionId": "s" }"# )
    .expect( "record without status should parse" );
  assert_eq!( missing.status, SessionStatus::Idle, "missing status should default to Idle" );

  let unknown = SessionRecord::parse( r#"{ "pid": 7, "sessionId": "s", "status": "compacting" }"# )
    .expect( "record with unknown status should parse" );
  assert_eq!(
    unknown.status,
    SessionStatus::Other( "compacting".to_string() ),
    "an unmodelled status must be preserved, not collapsed into Idle",
  );
}

/// reg08: a registry directory that was never created is not an error.
#[ test ]
fn reg08_missing_directory_scans_empty()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let absent = dir.path().join( "never-created" );

  let records = scan( &absent ).expect( "a missing registry directory must not be an error" );
  assert!( records.is_empty(), "expected no records, got {records:?}" );
}

/// reg09: only `.json` files are considered.
#[ test ]
fn reg09_non_json_files_are_ignored()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  write_file( dir.path(), "100.json", &full_record_json( 100, "keep" ) );
  write_file( dir.path(), "notes.txt", &full_record_json( 200, "drop" ) );
  write_file( dir.path(), "200.json.tmp", &full_record_json( 300, "drop" ) );

  let records = scan( dir.path() ).expect( "scan failed" );
  assert_eq!( records.len(), 1, "expected only the .json record, got {records:?}" );
  assert_eq!( records[ 0 ].session_id, "keep" );
}

/// reg10: one corrupt file does not hide the rest.
#[ test ]
fn reg10_corrupt_file_does_not_hide_good_records()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  write_file( dir.path(), "1.json", &full_record_json( 1, "first" ) );
  write_file( dir.path(), "2.json", r#"{ "pid": 2, "sessi"# );
  write_file( dir.path(), "3.json", &full_record_json( 3, "third" ) );

  let records = scan( dir.path() ).expect( "scan failed" );
  let ids : Vec< &str > = records.iter().map( | r | r.session_id.as_str() ).collect();
  assert_eq!( ids, vec![ "first", "third" ], "corrupt file affected unrelated records" );
}

/// reg11: results are ordered by pid.
///
/// Directory iteration order is unspecified, so without the sort a caller's
/// output would reshuffle between runs on the same inputs.
#[ test ]
fn reg11_records_are_sorted_by_pid()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  for pid in [ 900_u32, 17, 42_000, 3 ]
  {
    write_file( dir.path(), &format!( "{pid}.json" ), &full_record_json( pid, "s" ) );
  }

  let records = scan( dir.path() ).expect( "scan failed" );
  let pids : Vec< u32 > = records.iter().map( | r | r.pid ).collect();
  assert_eq!( pids, vec![ 3, 17, 900, 42_000 ], "records are not sorted by pid" );
}

/// reg12: `scan_live` drops records whose process is gone.
///
/// The registry is reaped on clean exit, but a killed or crashed process leaves
/// its file behind — so the raw scan reports sessions that no longer exist. This
/// test's own pid, with its real start time, is the live control.
#[ test ]
fn reg12_scan_live_filters_dead_records()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let live_pid = std::process::id();
  let live_start = proc_starttime( live_pid ).expect( "cannot read this process's start time" );

  write_file(
    dir.path(),
    &format!( "{live_pid}.json" ),
    &format!( r#"{{ "pid": {live_pid}, "sessionId": "live", "procStart": "{live_start}" }}"# ),
  );
  // No `/proc` entry can exist for the highest representable pid number.
  write_file(
    dir.path(),
    "dead.json",
    r#"{ "pid": 4294967295, "sessionId": "dead", "procStart": "1" }"#,
  );

  let all = scan( dir.path() ).expect( "scan failed" );
  assert_eq!( all.len(), 2, "raw scan should report both records, got {all:?}" );

  let live = scan_live( dir.path() ).expect( "scan_live failed" );
  let ids : Vec< &str > = live.iter().map( | r | r.session_id.as_str() ).collect();
  assert_eq!( ids, vec![ "live" ], "scan_live did not filter the dead record" );
}

/// reg13: a record without `cwd` is usable.
#[ test ]
fn reg13_missing_cwd_yields_empty_path()
{
  let record = SessionRecord::parse( r#"{ "pid": 7, "sessionId": "s" }"# )
    .expect( "record without cwd should parse" );

  assert_eq!( record.cwd, Path::new( "" ), "missing cwd should be an empty path" );
}

/// reg14: a path that exists but is not a directory is a real error.
///
/// This is the one failure `scan` reports rather than absorbing: the caller named
/// something, it is there, and it cannot be enumerated. Silently returning an
/// empty list would make a misconfigured path indistinguishable from an idle
/// machine.
#[ test ]
fn reg14_non_directory_path_is_an_error()
{
  let dir = tempfile::tempdir().expect( "cannot create temp dir" );
  let file = dir.path().join( "regular-file" );
  fs::write( &file, "not a directory" ).expect( "cannot write file" );

  match scan( &file )
  {
    Err( Error::ReadDir { path, .. } ) => assert_eq!( path, file, "error names the wrong path" ),
    other => panic!( "expected ReadDir error, got {other:?}" ),
  }
}

/// reg15: `is_alive` agrees with reality for this very process.
#[ test ]
fn reg15_is_alive_for_this_process()
{
  let pid = std::process::id();
  let start = proc_starttime( pid ).expect( "cannot read this process's start time" );

  let live = SessionRecord::parse(
    &format!( r#"{{ "pid": {pid}, "sessionId": "self", "procStart": "{start}" }}"# ),
  )
  .expect( "self record failed to parse" );
  assert!( live.is_alive(), "this process reported itself dead" );

  let wrong_incarnation = SessionRecord::parse(
    &format!( r#"{{ "pid": {pid}, "sessionId": "self", "procStart": "{}" }}"#, start + 1 ),
  )
  .expect( "self record failed to parse" );
  assert!(
    !wrong_incarnation.is_alive(),
    "a mismatched start time was accepted as the same incarnation",
  );
}

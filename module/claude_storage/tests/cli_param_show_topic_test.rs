//! Edge case tests for the `show_topic::` parameter on `.projects`.
//!
//! ## Coverage
//!
//! EC-1 through EC-5 per `tests/docs/cli/param/28_show_topic.md` —
//! topic display from the first user message, newline flattening plus
//! 90-character truncation, omitted/`0` default (no topic), non-boolean
//! rejection, and combination with `since_days::`.
//!
//! ## Test Case Index
//!
//! | ID | Test Name | Category |
//! |----|-----------|----------|
//! | EC-1 | `show_topic::1` appends first user message text | Display Format |
//! | EC-2 | Topic flattens newlines and truncates at 90 chars | Display Format |
//! | EC-3 | Omitted or `0` shows no topic text | Default |
//! | EC-4 | Non-boolean value rejected | Type Validation |
//! | EC-5 | Combined `since_days::` window plus topic display | Filter Interaction |

mod common;

use tempfile::TempDir;

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr( out )
  );
}

/// Write a session whose FIRST entry is a user message with `first_msg` as
/// its content, followed by one assistant entry. The topic renderer reads
/// exactly this first user entry.
///
/// `first_msg` may contain `\n`, `"`, and `\` — they are JSON-escaped here.
///
/// Returns the encoded project ID.
fn write_session_with_first_message(
  root         : &std::path::Path,
  project_path : &std::path::Path,
  session_id   : &str,
  first_msg    : &str,
) -> String
{
  use std::io::Write as _;

  let encoded = claude_storage_core::encode_path( project_path )
    .expect( "encode project path" );
  let dir = root.join( "projects" ).join( &encoded );
  std::fs::create_dir_all( &dir ).expect( "create project dir" );
  let path = dir.join( format!( "{session_id}.jsonl" ) );
  let mut file = std::fs::File::create( &path ).expect( "create session file" );

  let escaped = first_msg
    .replace( '\\', "\\\\" )
    .replace( '"', "\\\"" )
    .replace( '\n', "\\n" );

  writeln!(
    file,
    r#"{{"type":"user","uuid":"test-uuid-000","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"{escaped}"}}}}"#
  )
  .expect( "write user entry" );
  writeln!(
    file,
    r#"{{"type":"assistant","uuid":"test-uuid-001","parentUuid":"test-uuid-000","timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req_test_001","message":{{"role":"assistant","model":"claude-test","id":"msg_test_001","content":[{{"type":"text","text":"ok"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
  )
  .expect( "write assistant entry" );

  encoded
}

/// Set a session file's modification time to `now - days` days.
fn set_mtime_days_ago( path : &std::path::Path, days : u64 )
{
  let t = std::time::SystemTime::now() - core::time::Duration::from_secs( days * 86_400 );
  let f = std::fs::OpenOptions::new().write( true ).open( path )
    .expect( "open session file for mtime update" );
  f.set_times( std::fs::FileTimes::new().set_modified( t ) )
    .expect( "set session file mtime" );
}

/// EC-1: `show_topic::1` appends first user message text.
///
/// ## Purpose
/// Validates that the session line gains the first user message as its
/// topic when `show_topic::1` is set.
///
/// ## Coverage
/// Topic text appears on the same line as the session's short ID.
///
/// ## Validation Strategy
/// Write a session with a known first message, run `.projects
/// scope::global show_topic::1`, assert the ID line carries the topic.
///
/// ## Related Requirements
/// `tests/docs/cli/param/28_show_topic.md` — EC-1
#[ test ]
fn ec_1_show_topic_appends_first_user_message()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj_topic" );

  write_session_with_first_message( &storage_root, &project, "topicaa1", "fix retry timeouts" );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "show_topic::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let line = s.lines().find( | l | l.contains( "topicaa1" ) )
    .expect( "session line with short ID must exist" );
  assert!(
    line.contains( "fix retry timeouts" ),
    "EC-1: topic must appear on the session line; got line: {line}\nfull:\n{s}"
  );
}

/// EC-2: Topic flattens newlines and truncates at 90 chars.
///
/// ## Purpose
/// Validates the two topic normalization rules: newlines become spaces,
/// and the flattened text is cut at 90 characters.
///
/// ## Coverage
/// A multi-line, 101-character first message renders as a single-line,
/// exactly-90-character topic.
///
/// ## Validation Strategy
/// Compute the expected flattened 90-char prefix and assert it appears
/// while the 91-char prefix does not.
///
/// ## Related Requirements
/// `tests/docs/cli/param/28_show_topic.md` — EC-2
#[ test ]
fn ec_2_show_topic_flattens_newlines_and_truncates()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj_trunc" );

  let long_msg = format!( "alpha\nbeta {}", "x".repeat( 90 ) );
  write_session_with_first_message( &storage_root, &project, "truncaa1", &long_msg );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "show_topic::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );

  let flat = long_msg.replace( '\n', " " );
  let expected : String = flat.chars().take( 90 ).collect();
  let over : String = flat.chars().take( 91 ).collect();

  assert!(
    s.contains( "alpha beta x" ),
    "EC-2: newline must flatten to a space; got:\n{s}"
  );
  assert!(
    s.contains( &expected ),
    "EC-2: the 90-char flattened prefix must appear; got:\n{s}"
  );
  assert!(
    !s.contains( &over ),
    "EC-2: the 91st character must be truncated away; got:\n{s}"
  );
}

/// EC-3: Omitted or `0` shows no topic text.
///
/// ## Purpose
/// Validates the baseline regression: without `show_topic::1`, session
/// lines carry no message text — bare output is unchanged.
///
/// ## Coverage
/// Topic text absent both when the parameter is omitted and when it is
/// explicitly `0`.
///
/// ## Validation Strategy
/// Same fixture as EC-1; run bare and with `show_topic::0`; assert the
/// message text never appears.
///
/// ## Related Requirements
/// `tests/docs/cli/param/28_show_topic.md` — EC-3
#[ test ]
fn ec_3_show_topic_off_shows_no_topic()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj_topic" );

  write_session_with_first_message( &storage_root, &project, "topicaa1", "fix retry timeouts" );

  for extra in [ None, Some( "show_topic::0" ) ]
  {
    let mut cmd = common::clg_cmd();
    cmd
      .env( "HOME", root.path().to_str().unwrap() )
      .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
      .arg( ".projects" )
      .arg( "scope::global" );
    if let Some( arg ) = extra
    {
      cmd.arg( arg );
    }
    let out = cmd.output().unwrap();

    assert_exit( &out, 0 );
    let s = stdout( &out );
    assert!( s.contains( "topicaa1" ), "EC-3: session must still be listed; got:\n{s}" );
    assert!(
      !s.contains( "fix retry timeouts" ),
      "EC-3: topic text must be absent with show_topic {} ; got:\n{s}",
      extra.unwrap_or( "omitted" )
    );
  }
}

/// EC-4: Non-boolean value rejected.
///
/// ## Purpose
/// Validates that `show_topic::abc` is rejected (not a valid boolean).
///
/// ## Coverage
/// Exit non-zero; coercion error on the `show_topic` argument.
///
/// ## Validation Strategy
/// Run `.projects show_topic::abc`. Assert exit non-zero.
///
/// ## Related Requirements
/// `tests/docs/cli/param/28_show_topic.md` — EC-4
#[ test ]
fn ec_4_show_topic_non_boolean_rejected()
{
  let out = common::clg_cmd()
    .arg( ".projects" )
    .arg( "show_topic::abc" )
    .output()
    .unwrap();

  assert_ne!(
    out.status.code().unwrap_or( -1 ),
    0,
    "EC-4: show_topic::abc should be rejected; stderr: {}",
    stderr( &out )
  );
}

/// EC-5: Combined `since_days::` window plus topic display.
///
/// ## Purpose
/// Validates the task's headline invocation: `scope::global
/// since_days::20 show_topic::1` windows the list AND shows topics —
/// the excluded session contributes neither a line nor a topic.
///
/// ## Coverage
/// Recent session listed with its topic; old session's ID and topic both
/// absent.
///
/// ## Validation Strategy
/// Recent custom-topic session (5 days old) plus standard fixture session
/// (25 days old, whose topic would be "entry 0"); assert only the recent
/// pair survives.
///
/// ## Related Requirements
/// `tests/docs/cli/param/28_show_topic.md` — EC-5
#[ test ]
fn ec_5_show_topic_combined_with_since_days()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "proj_combo" );

  let enc = write_session_with_first_message(
    &storage_root, &project, "freshaa1", "recent window topic"
  );
  common::write_path_project_session( &storage_root, &project, "older222", 2 );

  let dir = storage_root.join( "projects" ).join( &enc );
  set_mtime_days_ago( &dir.join( "freshaa1.jsonl" ), 5 );
  set_mtime_days_ago( &dir.join( "older222.jsonl" ), 25 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .arg( "since_days::20" )
    .arg( "show_topic::1" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "freshaa1" ), "EC-5: windowed session must appear; got:\n{s}" );
  assert!( s.contains( "recent window topic" ), "EC-5: its topic must appear; got:\n{s}" );
  assert!( !s.contains( "older222" ), "EC-5: out-of-window session must be excluded; got:\n{s}" );
  assert!( !s.contains( "entry 0" ), "EC-5: excluded session's topic must not leak; got:\n{s}" );
}

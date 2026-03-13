//! Session management parameter builder method tests (TSK-074)
//!
//! ## Purpose
//!
//! Verify the five session management `with_*()` methods add the correct CLI flags.
//!
//! ## Evidence
//!
//! - `with_resume(None)` adds `-r` with no following value
//! - `with_resume(Some("uuid"))` adds `-r uuid`
//! - `with_session_id("uuid")` adds `--session-id uuid`
//! - `with_fork_session(true)` adds `--fork-session`
//! - `with_fork_session(false)` adds nothing
//! - `with_no_session_persistence(true)` adds `--no-session-persistence`
//! - `with_no_session_persistence(false)` adds nothing
//! - `with_from_pr("42")` adds `--from-pr 42`
//!
//! ## Test Coverage Matrix
//!
//! | Method | flag present | flag absent | empty string |
//! |--------|-------------|-------------|--------------|
//! | with_resume(None) | ✅ | — | — |
//! | with_resume(Some) | ✅ | — | ✅ |
//! | with_session_id | ✅ | — | ✅ |
//! | with_fork_session | ✅ | ✅ | — |
//! | with_no_session_persistence | ✅ | ✅ | — |
//! | with_from_pr | ✅ | — | ✅ |

use claude_runner_core::ClaudeCommand;

fn args_of( cmd: &ClaudeCommand ) -> Vec<String> {
  let c = cmd.build_command_for_test();
  c.get_args().map( |a| a.to_string_lossy().to_string() ).collect()
}

// with_resume

#[test]
fn with_resume_none_adds_r_flag_only() {
  let cmd = ClaudeCommand::new().with_resume( None );
  let args = args_of( &cmd );
  assert!( args.contains( &"-r".to_string() ), "must contain -r: {args:?}" );
}

#[test]
fn with_resume_none_does_not_add_value() {
  let cmd = ClaudeCommand::new().with_resume( None );
  let args = args_of( &cmd );
  // -r must be present but must not be followed by a UUID-like string
  let r_pos = args.iter().position( |a| a == "-r" );
  assert!( r_pos.is_some(), "-r not found" );
  // Ensure the next arg (if any) is NOT a UUID value added by with_resume
  // (Other flags like env defaults may follow, but with_resume(None) must not push a second arg)
  // We verify by checking that calling with_resume(None) adds exactly 1 arg
  let cmd_baseline = ClaudeCommand::new();
  let baseline_count = args_of( &cmd_baseline ).len();
  let resume_count = args.len();
  assert_eq!( resume_count, baseline_count + 1, "with_resume(None) must add exactly 1 arg (-r)" );
}

#[test]
fn with_resume_some_adds_r_flag_and_value() {
  let cmd = ClaudeCommand::new().with_resume( Some( "abc-123" ) );
  let args = args_of( &cmd );
  assert!( args.contains( &"-r".to_string() ) );
  assert!( args.contains( &"abc-123".to_string() ) );
}

#[test]
fn with_resume_some_r_is_followed_by_value() {
  let cmd = ClaudeCommand::new().with_resume( Some( "my-session" ) );
  let args = args_of( &cmd );
  let r_pos = args.iter().position( |a| a == "-r" ).expect( "-r not found" );
  assert_eq!( args.get( r_pos + 1 ).map( String::as_str ), Some( "my-session" ) );
}

// with_session_id

#[test]
fn with_session_id_adds_flag_and_value() {
  let cmd = ClaudeCommand::new().with_session_id( "dead-beef-0000" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--session-id".to_string() ) );
  assert!( args.contains( &"dead-beef-0000".to_string() ) );
}

// with_fork_session

#[test]
fn with_fork_session_true_adds_flag() {
  let cmd = ClaudeCommand::new().with_fork_session( true );
  assert!( args_of( &cmd ).contains( &"--fork-session".to_string() ) );
}

#[test]
fn with_fork_session_false_adds_nothing() {
  let cmd = ClaudeCommand::new().with_fork_session( false );
  assert!( !args_of( &cmd ).contains( &"--fork-session".to_string() ) );
}

// with_no_session_persistence

#[test]
fn with_no_session_persistence_true_adds_flag() {
  let cmd = ClaudeCommand::new().with_no_session_persistence( true );
  assert!( args_of( &cmd ).contains( &"--no-session-persistence".to_string() ) );
}

#[test]
fn with_no_session_persistence_false_adds_nothing() {
  let cmd = ClaudeCommand::new().with_no_session_persistence( false );
  assert!( !args_of( &cmd ).contains( &"--no-session-persistence".to_string() ) );
}

// with_from_pr

#[test]
fn with_from_pr_adds_flag_and_value() {
  let cmd = ClaudeCommand::new().with_from_pr( "42" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--from-pr".to_string() ) );
  assert!( args.contains( &"42".to_string() ) );
}

// ── Option<&str> empty string edge cases ──────────────────────────────────────

#[test]
fn with_resume_some_empty_string_adds_r_and_empty_value() {
  // with_resume(Some("")) passes the empty string as the session ID value.
  // Builder does not filter or validate — empty string is passed through.
  let cmd = ClaudeCommand::new().with_resume( Some( "" ) );
  let args = args_of( &cmd );
  assert!( args.contains( &"-r".to_string() ), "-r must be present: {args:?}" );
  let r_pos = args.iter().position( |a| a == "-r" ).expect( "-r not found" );
  assert_eq!(
    args.get( r_pos + 1 ).map( String::as_str ), Some( "" ),
    "empty string value must follow -r: {args:?}"
  );
}

#[test]
fn with_session_id_empty_string_adds_flag_and_empty_value() {
  // with_session_id("") passes empty string — builder does not validate UUIDs
  let cmd = ClaudeCommand::new().with_session_id( "" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--session-id".to_string() ) );
  let pos = args.iter().position( |a| a == "--session-id" ).expect( "flag not found" );
  assert_eq!( args.get( pos + 1 ).map( String::as_str ), Some( "" ) );
}

#[test]
fn with_from_pr_empty_string_adds_flag_and_empty_value() {
  // with_from_pr("") passes empty string — builder does not validate PR IDs
  let cmd = ClaudeCommand::new().with_from_pr( "" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--from-pr".to_string() ) );
  let pos = args.iter().position( |a| a == "--from-pr" ).expect( "flag not found" );
  assert_eq!( args.get( pos + 1 ).map( String::as_str ), Some( "" ) );
}

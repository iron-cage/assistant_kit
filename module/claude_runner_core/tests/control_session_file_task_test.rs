//! Real-subprocess tests: file/session-state control methods — `rewindFiles`,
//! `getContextUsage`, `readFile`, `reloadPlugins`, `reloadSkills`, `seedReadState` (task 415
//! Test Matrix rows IT-10, IT-28, IT-29, IT-30, IT-31, IT-32).
//!
//! No mocking anywhere below — every test spawns a real `claude` subprocess via
//! [`claude_runner_core::ClaudeCommand::spawn_control_session`].

mod control_session_common;

/// IT-10: `rewindFiles(userMessageId, {dryRun})` returns a structured result rather than
/// erroring for an unknown/no-checkpoint message id — `canRewind: false` with an explanatory
/// `error`, confirmed against a real subprocess.
#[ test ]
fn it_10_rewind_files_reports_structured_result_for_dry_run()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let result = session.rewind_files( "clr-test-nonexistent-message-id", true )
    .expect( "rewind_files() must return a structured result, not error, for an unknown message id" );
  assert!( !result.can_rewind, "no real edit preceded this message id, so rewind must not be possible" );
  assert!( result.error.is_some(), "the reason rewind is impossible must be explained" );
}

/// IT-28: `getContextUsage()` is a real wire round trip returning a multi-category usage
/// breakdown.
#[ test ]
fn it_28_get_context_usage_is_a_real_wire_round_trip()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let usage = session.get_context_usage().expect( "get_context_usage() must succeed against a real session" );
  assert!( usage.get( "categories" ).is_some_and( serde_json::Value::is_array ), "categories must be a JSON array" );
  assert!( usage.get( "totalTokens" ).is_some(), "totalTokens field must be present" );
}

/// IT-29: `readFile(path, options?)` returns real file contents for an existing file, and
/// `Ok(None)` (not `Err`) for a missing one — confirmed empirically: the real CLI reports a
/// missing file as a wire-level control error, which this method translates to `None` per the
/// SDK's own `Promise<.. | null>` contract.
#[ test ]
fn it_29_read_file_returns_contents_for_an_existing_file_and_none_for_a_missing_one()
{
  let ( session, dir ) = control_session_common::spawn_session();
  let file_path = dir.path().join( "sample.txt" );
  std::fs::write( &file_path, "hello from task 415\n" ).expect( "failed to write scratch file" );

  let result = session.read_file( file_path.to_str().unwrap(), None, None )
    .expect( "read_file() must succeed for a real, existing, permitted file" )
    .expect( "read_file() must return Some(..) for an existing file" );
  assert_eq!( result.contents, "hello from task 415\n" );

  let missing = session.read_file( dir.path().join( "does-not-exist.txt" ).to_str().unwrap(), None, None )
    .expect( "a missing file must yield Ok(None), not Err" );
  assert!( missing.is_none(), "a missing file must yield None" );
}

/// IT-30: `reloadPlugins()` is a real wire round trip returning refreshed command/agent lists.
#[ test ]
fn it_30_reload_plugins_is_a_real_wire_round_trip()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let result = session.reload_plugins().expect( "reload_plugins() must succeed against a real session" );
  assert!( result.commands.is_array() );
  assert!( result.agents.is_array() );
}

/// IT-31: `reloadSkills()` is a real wire round trip returning a refreshed skill list.
#[ test ]
fn it_31_reload_skills_is_a_real_wire_round_trip()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let result = session.reload_skills().expect( "reload_skills() must succeed against a real session" );
  assert!( result.skills.is_array(), "skills field must be a JSON array" );
}

/// IT-32: `seedReadState(path, mtime)` succeeds live against a real session.
#[ test ]
fn it_32_seed_read_state_succeeds_live()
{
  let ( session, dir ) = control_session_common::spawn_session();
  let file_path = dir.path().join( "seed_target.txt" );
  std::fs::write( &file_path, "seed me\n" ).expect( "failed to write scratch file" );
  let mtime = std::fs::metadata( &file_path ).unwrap()
    .modified().unwrap()
    .duration_since( std::time::UNIX_EPOCH ).unwrap()
    .as_secs();

  session.seed_read_state( file_path.to_str().unwrap(), mtime )
    .expect( "seed_read_state() must succeed against a real session" );
}

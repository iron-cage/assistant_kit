//! Real-subprocess tests: Phase 0 capture integrity, and `ControlSession` lifecycle/fault
//! handling — `interrupt`, request timeout, subprocess crash, `stopTask`, `backgroundTasks`
//! (task 415 Test Matrix rows P0-1, IT-7, IT-8, IT-9, IT-26, IT-33).
//!
//! No mocking anywhere below — every test spawns a real `claude` subprocess via
//! [`claude_runner_core::ClaudeCommand::spawn_control_session`].

mod control_session_common;

use core::time::Duration;
use std::time::Instant;

/// P0-1: the Phase 0 capture artifact this crate's control-protocol design is evidenced
/// against is present, well-formed, and documents all 25 in-scope `Query` methods.
#[ test ]
fn p0_1_fixture_capture_is_well_formed_and_complete()
{
  let fixture_dir = std::path::Path::new( env!( "CARGO_MANIFEST_DIR" ) )
    .join( "tests/fixtures/sdk_control_capture" );

  let argv_raw = std::fs::read_to_string( fixture_dir.join( "argv.json" ) )
    .expect( "argv.json must exist and be readable" );
  let argv : serde_json::Value = serde_json::from_str( &argv_raw ).expect( "argv.json must be valid JSON" );
  assert!(
    argv.get( "args" ).and_then( serde_json::Value::as_array ).is_some_and( | a | !a.is_empty() ),
    "argv.json must record a non-empty args array"
  );

  for ndjson_name in [ "wire_stdin.ndjson", "wire_stdout.ndjson" ]
  {
    let raw = std::fs::read_to_string( fixture_dir.join( ndjson_name ) )
      .unwrap_or_else( | e | panic!( "{ndjson_name} must exist and be readable: {e}" ) );
    let mut line_count = 0;
    for line in raw.lines()
    {
      if line.trim().is_empty()
      {
        continue;
      }
      serde_json::from_str::< serde_json::Value >( line )
        .unwrap_or_else( | e | panic!( "{ndjson_name} line must be valid JSON: {e}\nline: {line}" ) );
      line_count += 1;
    }
    assert!( line_count > 0, "{ndjson_name} must contain at least one message" );
  }

  let results_raw = std::fs::read_to_string( fixture_dir.join( "method_results.json" ) )
    .expect( "method_results.json must exist and be readable" );
  let results : serde_json::Value = serde_json::from_str( &results_raw )
    .expect( "method_results.json must be valid JSON" );
  let methods = results.get( "results" ).and_then( serde_json::Value::as_object )
    .expect( "method_results.json must have a 'results' object" );

  const EXPECTED_METHODS : [ &str; 25 ] = [
    "interrupt", "rewindFiles", "setPermissionMode", "setMcpPermissionModeOverride", "setModel",
    "setMaxThinkingTokens", "applyFlagSettings", "initializationResult", "reinitialize",
    "supportedCommands", "supportedModels", "supportedAgents", "mcpServerStatus",
    "getContextUsage", "readFile", "reloadPlugins", "reloadSkills", "accountInfo",
    "seedReadState", "reconnectMcpServer", "toggleMcpServer", "setMcpServers", "streamInput",
    "stopTask", "backgroundTasks",
  ];
  for method in EXPECTED_METHODS
  {
    assert!( methods.contains_key( method ), "method_results.json missing entry for '{method}'" );
  }
  assert_eq!(
    methods.len(), EXPECTED_METHODS.len(),
    "method_results.json should record exactly the 25 in-scope methods, found: {:?}",
    methods.keys().collect::< Vec< _ > >()
  );
}

/// IT-7: `interrupt()` completes against a real subprocess without hanging.
#[ test ]
fn it_7_interrupt_completes_without_hanging()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let started = Instant::now();
  session.interrupt().expect( "interrupt() must succeed against a real idle session" );
  assert!(
    started.elapsed() < Duration::from_secs( 10 ),
    "interrupt() must return promptly, not hang toward the request timeout"
  );
}

/// IT-8: an expired per-request timeout surfaces a clear typed error instead of hanging —
/// exactly the scenario `DEFAULT_REQUEST_TIMEOUT`'s own doc comment cites.
#[ test ]
fn it_8_expired_timeout_surfaces_typed_error_without_hanging()
{
  let ( mut session, _dir ) = control_session_common::spawn_session();
  session.set_request_timeout( Duration::from_nanos( 1 ) );

  let started = Instant::now();
  let err = session.interrupt()
    .expect_err( "an effectively-zero timeout must surface Err, not a hang or panic" );
  let elapsed = started.elapsed();

  assert!( elapsed < Duration::from_secs( 5 ), "timeout error must surface almost immediately, got {elapsed:?}" );
  assert!( err.to_string().contains( "timed out" ), "error message must clearly identify a timeout, got: {err}" );
}

/// IT-9: killing the subprocess out-of-band (simulating a crash) fails any subsequent control
/// call promptly and cleanly, rather than hanging toward the full request timeout.
#[ test ]
fn it_9_subprocess_crash_fails_future_calls_cleanly()
{
  let ( mut session, _dir ) = control_session_common::spawn_session();
  // Warm up: one real round trip confirms the session is live before the crash simulation.
  session.interrupt().expect( "session must be responsive before the crash simulation" );

  let debug_repr = format!( "{session:?}" );
  let pid = extract_pid( &debug_repr )
    .unwrap_or_else( || panic!( "could not extract pid from Debug output: {debug_repr}" ) );

  let status = std::process::Command::new( "kill" )
    .args( [ "-9", &pid.to_string() ] )
    .status()
    .expect( "failed to invoke `kill`" );
  assert!( status.success(), "kill -9 {pid} must succeed" );

  std::thread::sleep( Duration::from_millis( 500 ) );
  session.set_request_timeout( Duration::from_secs( 3 ) );

  let started = Instant::now();
  let err = session.interrupt()
    .expect_err( "a call after the subprocess is killed must return Err, not hang" );
  assert!(
    started.elapsed() < Duration::from_secs( 5 ),
    "post-crash call must fail promptly, not hang toward the timeout"
  );
  let message = err.to_string();
  assert!(
    message.contains( "broken" ) || message.contains( "exited" ) || message.contains( "closed" ),
    "error message must clearly describe the dead session, got: {message}"
  );
}

fn extract_pid( debug_repr : &str ) -> Option< u32 >
{
  let after = debug_repr.split( "pid: " ).nth( 1 )?;
  let digits : String = after.chars().take_while( char::is_ascii_digit ).collect();
  digits.parse().ok()
}

/// IT-26: `stopTask(taskId)` completes cleanly even for an unknown task id.
#[ test ]
fn it_26_stop_task_completes_for_an_unknown_task_id()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  session.stop_task( "clr-test-nonexistent-task" )
    .expect( "stop_task() must complete without error even for an unknown task id (confirmed real-subprocess behavior)" );
}

/// IT-33: `backgroundTasks(toolUseId?)` completes and reports success against a real session.
/// The real wire ack is an empty object regardless of whether anything was actually
/// backgrounded (confirmed against both an idle session here and an active-task session in
/// the Phase 0 fixture) — so any successful ack is surfaced as `true`.
#[ test ]
fn it_33_background_tasks_returns_true_on_a_successful_ack()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let backgrounded = session.background_tasks( None )
    .expect( "background_tasks() must complete against a real idle session" );
  assert!( backgrounded, "a successful wire ack must surface as true — no per-call boolean signal exists on the wire" );
}

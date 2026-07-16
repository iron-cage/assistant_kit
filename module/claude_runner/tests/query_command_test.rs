//! Integration tests for `clr query` (task 418): PID-addressed control-session dispatch.
//!
//! Covers QT-1 through QT-32 from task 418's Test Matrix. Uses `fake_claude_control`
//! (tests/fixtures/fake_claude_control.rs) as the `claude` stand-in — a compiled ELF
//! binary (not a shell script) speaking the real bidirectional control-session wire
//! protocol, so `find_claude_processes()` can discover it and `query.rs`'s daemon can
//! complete a real `spawn_control_session()` handshake against it.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::*;

/// All 25 camelCase method tokens `clr query` must dispatch (task 418 In-Scope bullet 3).
const ALL_METHODS : &[ &str ] = &[
  "interrupt", "rewindFiles", "setPermissionMode", "setModel", "setMaxThinkingTokens",
  "applyFlagSettings", "initializationResult", "reinitialize", "supportedCommands",
  "supportedModels", "supportedAgents", "mcpServerStatus", "accountInfo",
  "reconnectMcpServer", "toggleMcpServer", "setMcpServers", "streamInput", "stopTask",
  "setMcpPermissionModeOverride", "getContextUsage", "readFile", "reloadPlugins",
  "reloadSkills", "seedReadState", "backgroundTasks",
];

/// Start a `clr query "<message>"` session against the fake control binary.
///
/// Returns `(claude-PATH TempDir, CLR_QUERY_DIR TempDir, printed PID string)`. Both
/// `TempDir`s must stay alive for the session's lifetime — dropping either can delete
/// the fake binary symlink or the socket directory out from under the running daemon.
fn start_query_session() -> ( tempfile::TempDir, tempfile::TempDir, String )
{
  let ( claude_dir, path_val ) = fake_claude_control_binary_dir();
  let query_dir = tempfile::TempDir::new().expect( "query dir" );
  let query_dir_val = query_dir.path().to_str().expect( "utf8 path" ).to_owned();

  let out = run_cli_with_env(
    &[ "query", "hello from query_command_test" ],
    &[ ( "PATH", path_val.as_str() ), ( "CLR_QUERY_DIR", query_dir_val.as_str() ) ],
  );
  assert_eq!( exit_code( &out ), 0, "start failed: stdout={} stderr={}", stdout_str( &out ), stderr_str( &out ) );
  let pid = stdout_str( &out ).trim().to_string();
  assert!( pid.parse::< u32 >().is_ok(), "expected a numeric PID on stdout, got: {pid:?}" );

  // Poll briefly for the socket file to exist — the daemon binds it before printing the
  // PID (see query.rs's run_query_daemon doc comment), but this process still needs a
  // moment after reading that stdout line to actually reach a `connect()` call itself.
  let socket_path = query_dir.path().join( format!( "{pid}.sock" ) );
  for _ in 0..50
  {
    if socket_path.exists() { break; }
    std::thread::sleep( std::time::Duration::from_millis( 20 ) );
  }
  assert!( socket_path.exists(), "query socket never appeared at {}", socket_path.display() );

  ( claude_dir, query_dir, pid )
}

/// Dispatch `clr query <pid> <method> [extra...]` with `CLR_QUERY_DIR` set to `query_dir_val`.
fn call( pid : &str, query_dir_val : &str, method : &str, extra : &[ &str ] ) -> std::process::Output
{
  let mut args = vec![ "query", pid, method ];
  args.extend_from_slice( extra );
  run_cli_with_env( &args, &[ ( "CLR_QUERY_DIR", query_dir_val ) ] )
}

/// Best-effort cleanup: terminate the session's claude PID via the real `clr kill` path.
fn cleanup( pid : &str )
{
  let _ = run_cli( &[ "kill", pid ] );
}

// QT-1: `clr query "<msg>"` prints a PID and exits 0.
#[ test ]
fn qt1_start_prints_pid_and_exits_0()
{
  let ( _claude_dir, _query_dir, pid ) = start_query_session();
  cleanup( &pid );
}

// QT-2: `clr ps` lists the query session distinguishably (Mode column == "query").
#[ test ]
fn qt2_ps_distinguishes_query_session()
{
  let ( _claude_dir, _query_dir, pid ) = start_query_session();

  let out = run_cli( &[ "ps", "--columns", "pid,mode", "--pid", &pid ] );
  assert_eq!( exit_code( &out ), 0, "clr ps failed: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );
  assert!( stdout.contains( &pid ), "expected PID {pid} in `clr ps` output:\n{stdout}" );
  assert!( stdout.contains( "query" ), "expected `query` mode marker in `clr ps` output:\n{stdout}" );

  cleanup( &pid );
}

// QT-3 / QT-4: `interrupt` normal case, then dispatch against a PID that never started
// (or has already been killed) mirrors `clr kill`'s existing not-found contract.
#[ test ]
fn qt3_qt4_interrupt_and_not_found_case()
{
  let ( _claude_dir, query_dir, pid ) = start_query_session();
  let query_dir_val = query_dir.path().to_str().expect( "utf8" ).to_owned();

  let out = call( &pid, &query_dir_val, "interrupt", &[] );
  assert_eq!( exit_code( &out ), 0, "interrupt failed: {}", stderr_str( &out ) );

  cleanup( &pid );

  // Same not-found contract `clr kill` already uses (kill.rs's exact error text).
  let missing_pid = "999999";
  let out = call( missing_pid, &query_dir_val, "interrupt", &[] );
  assert_eq!( exit_code( &out ), 1 );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( &format!( "PID {missing_pid} is not a running Claude Code session" ) ),
    "expected kill.rs's not-found contract, got: {stderr}"
  );
  assert!( stderr.contains( "clr ps" ), "expected 'Use clr ps...' hint, got: {stderr}" );
}

// QT-5: `clr kill` terminates a query session the same way as any other session.
#[ test ]
fn qt5_kill_terminates_query_session()
{
  let ( _claude_dir, _query_dir, pid ) = start_query_session();

  let out = run_cli( &[ "kill", &pid ] );
  assert_eq!( exit_code( &out ), 0, "clr kill failed: {}", stderr_str( &out ) );

  // Give the OS a moment to reap the signaled process before checking `clr ps`.
  std::thread::sleep( std::time::Duration::from_millis( 200 ) );
  let ps_out = run_cli( &[ "ps", "--pid", &pid ] );
  let stdout = stdout_str( &ps_out );
  assert!(
    !stdout.contains( &pid ) || stdout.to_lowercase().contains( "no active" ),
    "expected PID {pid} to be gone from `clr ps` after kill:\n{stdout}"
  );
}

// QT-6: `clr query --help` lists the query command and all 25 method names.
#[ test ]
fn qt6_help_lists_all_25_methods()
{
  let out = run_cli( &[ "query", "--help" ] );
  assert_eq!( exit_code( &out ), 0 );
  let stdout = stdout_str( &out );
  for method in ALL_METHODS
  {
    assert!( stdout.contains( method ), "expected method `{method}` in --help output:\n{stdout}" );
  }
}

// QT-7: an unknown method name exits 1 and lists all 25 valid names in stderr.
#[ test ]
fn qt7_unknown_method_lists_all_valid_names()
{
  let ( _claude_dir, query_dir, pid ) = start_query_session();
  let query_dir_val = query_dir.path().to_str().expect( "utf8" ).to_owned();

  let out = call( &pid, &query_dir_val, "notARealMethod", &[] );
  assert_eq!( exit_code( &out ), 1 );
  let stderr = stderr_str( &out );
  for method in ALL_METHODS
  {
    assert!( stderr.contains( method ), "expected `{method}` listed in unknown-method error:\n{stderr}" );
  }

  cleanup( &pid );
}

// QT-8: regression guard — adding `query` must not change `clr run`'s own behavior.
#[ test ]
fn qt8_run_subcommand_unaffected_by_query_addition()
{
  let out = run_cli( &[ "run", "--help" ] );
  assert_eq!( exit_code( &out ), 0 );
  let stdout = stdout_str( &out );
  assert!( stdout.to_lowercase().contains( "run" ), "expected `run` still documented:\n{stdout}" );

  // `--dry-run` still produces the existing preview output unaffected by the new subcommand.
  let out = run_cli( &[ "--dry-run", "a message" ] );
  assert_eq!( exit_code( &out ), 0, "clr --dry-run regressed: {}", stderr_str( &out ) );
}

// QT-9 through QT-32: dispatch every remaining method against one shared session.
// The fake fixture accepts any request shape and responds per-subtype (or `null` for
// subtypes it treats as fire-and-forget) — assertions here focus on the CLI-level
// contract (exit 0, some printed response) since exact wire-shape mapping is TSK-415's
// own test surface, not this task's.
#[ test ]
fn qt9_through_qt32_all_remaining_methods_dispatch_successfully()
{
  let ( _claude_dir, query_dir, pid ) = start_query_session();
  let query_dir_val = query_dir.path().to_str().expect( "utf8" ).to_owned();

  let cases : &[ ( &str, &[ &str ] ) ] = &[
    ( "rewindFiles", &[ "msg-id-1", "true" ] ),                 // QT-9
    ( "setPermissionMode", &[ "acceptEdits" ] ),                // QT-10
    ( "setModel", &[ "sonnet" ] ),                              // QT-11
    ( "setMaxThinkingTokens", &[ "1024" ] ),                    // QT-12
    ( "applyFlagSettings", &[ "{}" ] ),                         // QT-13
    ( "initializationResult", &[] ),                            // QT-14 (cached)
    ( "reinitialize", &[] ),                                    // QT-15
    ( "supportedCommands", &[] ),                               // QT-16 (cached)
    ( "supportedModels", &[] ),                                 // QT-17 (cached)
    ( "supportedAgents", &[] ),                                 // QT-18 (cached)
    ( "mcpServerStatus", &[] ),                                 // QT-19
    ( "accountInfo", &[] ),                                     // QT-20 (cached)
    ( "reconnectMcpServer", &[ "server-1" ] ),                  // QT-21
    ( "toggleMcpServer", &[ "server-1", "true" ] ),             // QT-22
    ( "setMcpServers", &[ "{}" ] ),                              // QT-23
    ( "streamInput", &[ "incremental input text" ] ),           // QT-24 (write-only)
    ( "stopTask", &[ "task-1" ] ),                              // QT-25
    ( "setMcpPermissionModeOverride", &[ "server-1", "auto" ] ),// QT-26
    ( "getContextUsage", &[] ),                                 // QT-27
    ( "readFile", &[ "/tmp/does-not-matter.txt" ] ),            // QT-28
    ( "reloadPlugins", &[] ),                                   // QT-29
    ( "reloadSkills", &[] ),                                    // QT-30
    ( "seedReadState", &[ "/tmp/does-not-matter.txt", "0" ] ),  // QT-31
    ( "backgroundTasks", &[] ),                                 // QT-32
  ];

  for ( method, extra ) in cases
  {
    let out = call( &pid, &query_dir_val, method, extra );
    assert_eq!(
      exit_code( &out ), 0,
      "method `{method}` failed: stdout={} stderr={}", stdout_str( &out ), stderr_str( &out )
    );
  }

  cleanup( &pid );
}

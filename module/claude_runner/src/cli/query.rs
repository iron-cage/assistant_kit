//! `clr query` — PID-addressed control-session dispatch (task 418).
//!
//! Two invocation forms:
//! - `clr query "<message>" [--dir <path>]` — starts a bidirectional control session,
//!   backgrounds it via a detached `__query_daemon` child process, prints the underlying
//!   `claude` subprocess's PID to stdout, and returns immediately — mirroring `clr run`'s
//!   existing backgrounded-by-default session model and `clr ps`/`clr kill`'s PID addressing.
//! - `clr query <pid> <method> [args...]` — dispatches one control method against a running
//!   session's daemon over a Unix domain socket keyed by PID.
//!
//! Method tokens are camelCase, matching the SDK's own spelling (see
//! `contract/sdk/docs/api/004_query_control_object.md`) rather than `ControlSession`'s Rust
//! snake_case method names — this module is the translation layer between the two.
//!
//! # Why a detached daemon, not a direct `ControlSession` hold
//!
//! `ClaudeCommand::spawn_control_session()` returns a `ControlSession` that owns a live
//! `std::process::Child` with piped stdin/stdout — it cannot be handed across process
//! boundaries. Each `clr query <pid> <method>` invocation is a separate, short-lived process,
//! so the `ControlSession` must be held by a long-running process instead: the detached
//! `__query_daemon` child spawned by the initial `clr query "<msg>"` call. The daemon holds
//! the session for its entire lifetime and relays method calls over a Unix socket keyed by
//! the underlying `claude` subprocess's PID — the same PID `clr ps`/`clr kill` already use.
//!
//! # Why PID-diffing, not a direct accessor
//!
//! `ControlSession` deliberately exposes no public PID accessor (its `child` field is
//! private). The daemon instead snapshots `find_claude_processes()` immediately before and
//! after `spawn_control_session()` and takes the new PID that appears — the same
//! "claude"-basename process discovery `clr ps`/`clr kill` already rely on.

use claude_core::process::find_claude_processes;
use claude_runner_core::
{
  ClaudeCommand, ControlSession, InitializeResult, InputFormat, McpPermissionOverrideMode,
  OutputFormat, PermissionMode,
};
use error_tools::{ Error, Result };
use std::io::{ BufRead, Write };
use std::os::unix::net::{ UnixListener, UnixStream };

/// All 25 `Query` control-method CLI tokens, camelCase per the SDK's own spelling
/// (task 418 In-Scope bullet 3). Order matches the Test Matrix (QT-3/4, then QT-9–32).
const QUERY_METHODS : &[ &str ] = &[
  "interrupt",
  "rewindFiles",
  "setPermissionMode",
  "setModel",
  "setMaxThinkingTokens",
  "applyFlagSettings",
  "initializationResult",
  "reinitialize",
  "supportedCommands",
  "supportedModels",
  "supportedAgents",
  "mcpServerStatus",
  "accountInfo",
  "reconnectMcpServer",
  "toggleMcpServer",
  "setMcpServers",
  "streamInput",
  "stopTask",
  "setMcpPermissionModeOverride",
  "getContextUsage",
  "readFile",
  "reloadPlugins",
  "reloadSkills",
  "seedReadState",
  "backgroundTasks",
];

/// Directory holding one `<pid>.sock` file per live query session.
///
/// Mirrors `gate.rs`'s `gate_dir()` convention: env override, else a temp-dir default.
fn query_dir() -> std::path::PathBuf
{
  if let Ok( dir ) = std::env::var( "CLR_QUERY_DIR" )
  {
    return std::path::PathBuf::from( dir );
  }
  std::env::temp_dir().join( "clr-query" )
}

fn socket_path_for( pid : u32 ) -> std::path::PathBuf
{
  query_dir().join( format!( "{pid}.sock" ) )
}

/// Dispatch `clr query ...` — routes to the start form or the PID+method dispatch form.
pub( crate ) fn dispatch_query( tokens : &[ String ] ) -> !
{
  let rest = &tokens[ 1.. ];

  if rest.iter().any( | t | t == "--help" || t == "-h" || t == "help" )
  {
    print_query_help();
  }

  let Some( first ) = rest.first() else
  {
    eprintln!(
      "Error: 'clr query' requires a message or a PID + method.\n\
       Usage: clr query \"<message>\" [--dir <path>] | clr query <pid> <method> [args...]\n\
       Run 'clr query --help' for usage."
    );
    std::process::exit( 1 );
  };

  if let Ok( pid ) = first.parse::< u32 >()
  {
    let Some( method ) = rest.get( 1 ) else
    {
      eprintln!(
        "Error: 'clr query {pid}' requires a method name.\n\
         Usage: clr query <pid> <method> [args...]\nRun 'clr query --help' for usage."
      );
      std::process::exit( 1 );
    };
    dispatch_call( pid, method, &rest[ 2.. ] );
  }

  let message = first.clone();
  let mut dir : Option< String > = None;
  let mut i = 1;
  while i < rest.len()
  {
    if rest[ i ] == "--dir" && i + 1 < rest.len()
    {
      dir = Some( rest[ i + 1 ].clone() );
      i += 2;
    }
    else
    {
      i += 1;
    }
  }

  dispatch_start( &message, dir.as_deref() );
}

/// Print help for the `query` subcommand and exit 0.
fn print_query_help() -> !
{
  println!( "clr query — Start or control a persistent, PID-addressed control session" );
  println!();
  println!( "USAGE:" );
  println!( "  clr query \"<MESSAGE>\" [--dir <PATH>]" );
  println!( "  clr query <PID> <METHOD> [ARGS...]" );
  println!();
  println!( "Starting a session backgrounds it immediately and prints the underlying" );
  println!( "claude subprocess's PID to stdout — it does not block until the session ends." );
  println!( "Use the printed PID with 'clr query <PID> <METHOD>' to dispatch a control" );
  println!( "method, 'clr ps' to list it, or 'clr kill <PID>' to terminate it." );
  println!();
  println!( "METHODS (camelCase, matching the SDK's own spelling):" );
  for method in QUERY_METHODS
  {
    println!( "  {method}" );
  }
  println!();
  println!( "EXIT CODES:" );
  println!( "  0    Success (PID printed, or method response printed)" );
  println!( "  1    Error (unknown method, PID not a running query session, session start failed)" );
  println!();
  println!( "  -h, --help                          Show this help" );
  std::process::exit( 0 );
}

/// Start form: `clr query "<message>" [--dir <path>]`.
///
/// Spawns a detached `__query_daemon` child, reads back exactly one stdout line (the
/// underlying claude subprocess's PID), prints it, and exits — the daemon keeps running
/// after this process exits (reparented to init, standard Unix backgrounding).
fn dispatch_start( message : &str, dir : Option< &str > ) -> !
{
  let exe = match std::env::current_exe()
  {
    Ok( p ) => p,
    Err( e ) =>
    {
      eprintln!( "Error: cannot resolve current executable: {e}" );
      std::process::exit( 1 );
    }
  };

  let mut cmd = std::process::Command::new( &exe );
  cmd.arg( "__query_daemon" ).arg( message );
  if let Some( d ) = dir
  {
    cmd.arg( "--dir" ).arg( d );
  }
  cmd.stdin( std::process::Stdio::null() );
  cmd.stdout( std::process::Stdio::piped() );
  cmd.stderr( std::process::Stdio::piped() );

  let mut child = match cmd.spawn()
  {
    Ok( c ) => c,
    Err( e ) =>
    {
      eprintln!( "Error: failed to start query daemon: {e}" );
      std::process::exit( 1 );
    }
  };

  let stdout = child.stdout.take().expect( "piped by dispatch_start" );
  let mut reader = std::io::BufReader::new( stdout );
  let mut line = String::new();

  match reader.read_line( &mut line )
  {
    Ok( n ) if n > 0 && !line.trim().is_empty() =>
    {
      println!( "{}", line.trim() );
      std::process::exit( 0 );
    }
    _ =>
    {
      let mut stderr_buf = String::new();
      if let Some( mut se ) = child.stderr.take()
      {
        let _ = std::io::Read::read_to_string( &mut se, &mut stderr_buf );
      }
      let detail = stderr_buf.trim();
      if detail.is_empty()
      {
        eprintln!( "Error: query daemon failed to start" );
      }
      else
      {
        eprintln!( "Error: query daemon failed to start: {detail}" );
      }
      std::process::exit( 1 );
    }
  }
}

/// PID+method dispatch form: `clr query <pid> <method> [args...]`.
fn dispatch_call( pid : u32, method : &str, args : &[ String ] ) -> !
{
  if !QUERY_METHODS.contains( &method )
  {
    eprintln!(
      "Error: unknown query method '{method}'.\nValid methods: {}",
      QUERY_METHODS.join( ", " )
    );
    std::process::exit( 1 );
  }

  let socket_path = socket_path_for( pid );
  let stream = match UnixStream::connect( &socket_path )
  {
    Ok( s ) => s,
    Err( _ ) =>
    {
      eprintln!(
        "Error: PID {pid} is not a running Claude Code session.\n\
         Use 'clr ps' to list active sessions."
      );
      std::process::exit( 1 );
    }
  };

  let request = serde_json::json!( { "method" : method, "args" : args } );
  let mut write_stream = match stream.try_clone()
  {
    Ok( s ) => s,
    Err( _ ) =>
    {
      eprintln!( "Error: failed to prepare request to PID {pid}" );
      std::process::exit( 1 );
    }
  };
  if writeln!( write_stream, "{request}" ).is_err() || write_stream.flush().is_err()
  {
    eprintln!( "Error: failed to send request to PID {pid}" );
    std::process::exit( 1 );
  }

  let mut reader = std::io::BufReader::new( stream );
  let mut line = String::new();
  if reader.read_line( &mut line ).unwrap_or( 0 ) == 0
  {
    eprintln!( "Error: no response from PID {pid}" );
    std::process::exit( 1 );
  }

  let Ok( resp ) = serde_json::from_str::< serde_json::Value >( line.trim() ) else
  {
    eprintln!( "Error: malformed response from PID {pid}" );
    std::process::exit( 1 );
  };

  if resp.get( "ok" ).and_then( serde_json::Value::as_bool ) == Some( true )
  {
    let result = resp.get( "result" ).cloned().unwrap_or( serde_json::Value::Null );
    println!( "{}", serde_json::to_string_pretty( &result ).unwrap_or_else( | _ | result.to_string() ) );
    std::process::exit( 0 );
  }

  let err = resp.get( "error" ).and_then( serde_json::Value::as_str ).unwrap_or( "unknown error" );
  eprintln!( "Error: {err}" );
  std::process::exit( 1 );
}

/// Hidden daemon entry point (`__query_daemon <message> [--dir <path>]`), dispatched from
/// `lib.rs::run_cli()` before subcommand validation — never invoked directly by a user.
///
/// Spawns the real control session, discovers its PID by diffing `find_claude_processes()`
/// before/after (see module doc comment), prints exactly that PID as its one stdout line,
/// then serves method requests over a PID-keyed Unix socket until the session's claude
/// subprocess exits.
pub( crate ) fn run_query_daemon( tokens : &[ String ] ) -> !
{
  let message = tokens.get( 1 ).cloned().unwrap_or_default();
  let mut dir : Option< String > = None;
  let mut i = 2;
  while i < tokens.len()
  {
    if tokens[ i ] == "--dir" && i + 1 < tokens.len()
    {
      dir = Some( tokens[ i + 1 ].clone() );
      i += 2;
    }
    else
    {
      i += 1;
    }
  }

  // Real captured argv (claude_runner_core/tests/fixtures/sdk_control_capture/argv.json,
  // task 415 Phase 0) confirmed these 4 flags for a working control session; --chrome is
  // suppressed (None) since it's absent from that capture and irrelevant to a
  // non-interactive control session (mirrors claude_version()'s same suppression).
  let mut builder = ClaudeCommand::new()
    .with_chrome( None )
    .with_output_format( OutputFormat::StreamJson )
    .with_verbose( true )
    .with_input_format( InputFormat::StreamJson )
    .with_permission_mode( PermissionMode::BypassPermissions )
    .with_message( message );
  if let Some( d ) = dir
  {
    builder = builder.with_working_directory( d );
  }

  let before : std::collections::HashSet< u32 > =
    find_claude_processes().into_iter().map( | p | p.pid ).collect();

  let session = match builder.spawn_control_session()
  {
    Ok( s ) => s,
    Err( e ) =>
    {
      eprintln!( "Error: failed to start control session: {e}" );
      std::process::exit( 1 );
    }
  };

  let Some( claude_pid ) = find_claude_processes()
    .into_iter()
    .map( | p | p.pid )
    .find( | pid | !before.contains( pid ) )
  else
  {
    eprintln!( "Error: spawned control session but could not determine its PID" );
    std::process::exit( 1 );
  };

  // Populate the initialize cache so cached accessors (initializationResult,
  // supportedCommands/Models/Agents, accountInfo) work immediately without requiring
  // a prior manual reinitialize call from the client.
  let _ = session.reinitialize();

  // Bind the socket BEFORE printing the PID: `dispatch_start()`'s caller treats the
  // printed PID as "ready to dispatch against" and may issue `clr query <pid> <method>`
  // immediately after reading it — printing first would race a client's connect attempt
  // against this bind call.
  let socket_path = socket_path_for( claude_pid );
  if let Some( parent ) = socket_path.parent()
  {
    let _ = std::fs::create_dir_all( parent );
  }
  let _ = std::fs::remove_file( &socket_path );

  let listener = match UnixListener::bind( &socket_path )
  {
    Ok( l ) => l,
    Err( e ) =>
    {
      eprintln!( "Error: failed to bind query socket: {e}" );
      std::process::exit( 1 );
    }
  };

  println!( "{claude_pid}" );
  if std::io::stdout().flush().is_err()
  {
    std::process::exit( 1 );
  }

  spawn_liveness_watchdog( claude_pid, socket_path );

  for stream in listener.incoming()
  {
    let Ok( stream ) = stream else { continue };
    handle_connection( &session, stream );
  }

  std::process::exit( 0 );
}

/// Background thread: once `claude_pid` is no longer a live process, remove the socket
/// file and exit the daemon. Polls rather than blocking so the accept loop stays simple
/// and single-threaded for request handling.
fn spawn_liveness_watchdog( claude_pid : u32, socket_path : std::path::PathBuf )
{
  std::thread::spawn( move ||
  {
    loop
    {
      std::thread::sleep( std::time::Duration::from_millis( 500 ) );
      if !find_claude_processes().into_iter().any( | p | p.pid == claude_pid )
      {
        let _ = std::fs::remove_file( &socket_path );
        std::process::exit( 0 );
      }
    }
  } );
}

/// Serve exactly one `{"method":...,"args":[...]}` request from `stream` and write back
/// exactly one `{"ok":bool,...}` response line.
fn handle_connection( session : &ControlSession, stream : UnixStream )
{
  let Ok( mut writer ) = stream.try_clone() else { return };
  let mut reader = std::io::BufReader::new( stream );
  let mut line = String::new();
  if reader.read_line( &mut line ).unwrap_or( 0 ) == 0
  {
    return;
  }

  let response = match serde_json::from_str::< serde_json::Value >( line.trim() )
  {
    Ok( req ) =>
    {
      let method = req.get( "method" ).and_then( serde_json::Value::as_str ).unwrap_or( "" );
      let args : Vec< String > = req.get( "args" )
        .and_then( serde_json::Value::as_array )
        .map( | a | a.iter().filter_map( | v | v.as_str().map( str::to_string ) ).collect() )
        .unwrap_or_default();

      match dispatch_method( session, method, &args )
      {
        Ok( result ) => serde_json::json!( { "ok" : true, "result" : result } ),
        Err( e ) => serde_json::json!( { "ok" : false, "error" : e.to_string() } ),
      }
    }
    Err( e ) => serde_json::json!( { "ok" : false, "error" : format!( "invalid request: {e}" ) } ),
  };

  let _ = writeln!( writer, "{response}" );
}

/// Dispatch one camelCase method token against `session`, translating to the matching
/// snake_case `ControlSession` method (task 418 In-Scope bullet 3) with minimal default
/// parameter values grounded in TSK-415's Phase 0/Phase 2 artifacts (C17) — exact per-method
/// argument shapes beyond the bare token are explicitly deferred by this task's own scope.
#[ allow( clippy::too_many_lines ) ] // mechanical one-arm-per-method dispatch table (mirrors ps.rs::dispatch_ps)
fn dispatch_method( session : &ControlSession, method : &str, args : &[ String ] ) -> Result< serde_json::Value >
{
  match method
  {
    "interrupt" =>
    {
      session.interrupt()?;
      Ok( serde_json::Value::Null )
    }
    "rewindFiles" =>
    {
      let user_message_id = args.first().map( String::as_str ).unwrap_or( "" );
      let dry_run = args.get( 1 ).map_or( true, | s | s != "false" );
      let r = session.rewind_files( user_message_id, dry_run )?;
      Ok( serde_json::json!( { "canRewind" : r.can_rewind, "error" : r.error } ) )
    }
    "setPermissionMode" =>
    {
      let mode = parse_permission_mode( args.first().map( String::as_str ).unwrap_or( "default" ) )?;
      session.set_permission_mode( mode )?;
      Ok( serde_json::Value::Null )
    }
    "setModel" =>
    {
      session.set_model( args.first().map( String::as_str ) )?;
      Ok( serde_json::Value::Null )
    }
    "setMaxThinkingTokens" =>
    {
      let tokens = args.first().and_then( | s | s.parse::< u64 >().ok() );
      session.set_max_thinking_tokens( tokens, None )?;
      Ok( serde_json::Value::Null )
    }
    "applyFlagSettings" =>
    {
      let settings = args.first()
        .and_then( | s | serde_json::from_str( s ).ok() )
        .unwrap_or( serde_json::Value::Null );
      session.apply_flag_settings( settings )?;
      Ok( serde_json::Value::Null )
    }
    "initializationResult" => Ok( initialize_to_json( session.initialization_result()? ) ),
    "reinitialize" => Ok( initialize_to_json( session.reinitialize()? ) ),
    "supportedCommands" => session.supported_commands(),
    "supportedModels" => session.supported_models(),
    "supportedAgents" => session.supported_agents(),
    "mcpServerStatus" =>
    {
      let entries = session.mcp_server_status()?;
      Ok( serde_json::Value::Array( entries.into_iter().map( | e | serde_json::json!(
        { "name" : e.name, "status" : e.status, "scope" : e.scope, "tools" : e.tools }
      ) ).collect() ) )
    }
    "getContextUsage" => session.get_context_usage(),
    "readFile" =>
    {
      let path = args.first().map( String::as_str ).unwrap_or( "" );
      let max_bytes = args.get( 1 ).and_then( | s | s.parse::< u64 >().ok() );
      match session.read_file( path, max_bytes, None )?
      {
        Some( r ) => Ok( serde_json::json!( { "contents" : r.contents, "absPath" : r.abs_path } ) ),
        None => Ok( serde_json::Value::Null ),
      }
    }
    "reloadPlugins" =>
    {
      let r = session.reload_plugins()?;
      Ok( serde_json::json!(
        {
          "commands" : r.commands, "agents" : r.agents, "plugins" : r.plugins,
          "mcpServers" : r.mcp_servers, "error_count" : r.error_count,
        }
      ) )
    }
    "reloadSkills" =>
    {
      let r = session.reload_skills()?;
      Ok( serde_json::json!( { "skills" : r.skills } ) )
    }
    "accountInfo" =>
    {
      let r = session.account_info()?;
      Ok( serde_json::json!(
        {
          "email" : r.email, "organization" : r.organization,
          "subscriptionType" : r.subscription_type, "apiProvider" : r.api_provider,
        }
      ) )
    }
    "seedReadState" =>
    {
      let path = args.first().map( String::as_str ).unwrap_or( "" );
      let mtime = args.get( 1 ).and_then( | s | s.parse::< u64 >().ok() ).unwrap_or( 0 );
      session.seed_read_state( path, mtime )?;
      Ok( serde_json::Value::Null )
    }
    "reconnectMcpServer" =>
    {
      session.reconnect_mcp_server( args.first().map( String::as_str ).unwrap_or( "" ) )?;
      Ok( serde_json::Value::Null )
    }
    "toggleMcpServer" =>
    {
      let name = args.first().map( String::as_str ).unwrap_or( "" );
      let enabled = args.get( 1 ).map_or( true, | s | s != "false" );
      session.toggle_mcp_server( name, enabled )?;
      Ok( serde_json::Value::Null )
    }
    "setMcpServers" =>
    {
      let servers = args.first()
        .and_then( | s | serde_json::from_str( s ).ok() )
        .unwrap_or( serde_json::Value::Null );
      let r = session.set_mcp_servers( servers )?;
      Ok( serde_json::json!( { "added" : r.added, "removed" : r.removed, "errors" : r.errors } ) )
    }
    "streamInput" =>
    {
      session.stream_input( args.first().map( String::as_str ).unwrap_or( "" ) )?;
      Ok( serde_json::Value::Null )
    }
    "stopTask" =>
    {
      session.stop_task( args.first().map( String::as_str ).unwrap_or( "" ) )?;
      Ok( serde_json::Value::Null )
    }
    "setMcpPermissionModeOverride" =>
    {
      let server_name = args.first().map( String::as_str ).unwrap_or( "" );
      let mode = match args.get( 1 ).map( String::as_str )
      {
        Some( "auto" ) => Some( McpPermissionOverrideMode::Auto ),
        Some( "default" ) => Some( McpPermissionOverrideMode::Default ),
        _ => None,
      };
      let r = session.set_mcp_permission_mode_override( server_name, mode )?;
      Ok( serde_json::json!( { "warning" : r.warning } ) )
    }
    "backgroundTasks" =>
    {
      let tool_use_id = args.first().map( String::as_str );
      Ok( serde_json::Value::Bool( session.background_tasks( tool_use_id )? ) )
    }
    other => Err( Error::msg( format!( "unknown query method: {other}" ) ) ),
  }
}

fn initialize_to_json( r : InitializeResult ) -> serde_json::Value
{
  serde_json::json!(
    {
      "commands" : r.commands, "agents" : r.agents, "output_style" : r.output_style,
      "available_output_styles" : r.available_output_styles, "models" : r.models,
      "account" : r.account, "pid" : r.pid, "feedback_survey_config" : r.feedback_survey_config,
    }
  )
}

fn parse_permission_mode( s : &str ) -> Result< PermissionMode >
{
  match s
  {
    "default" => Ok( PermissionMode::Default ),
    "acceptEdits" => Ok( PermissionMode::AcceptEdits ),
    "bypassPermissions" => Ok( PermissionMode::BypassPermissions ),
    other => Err( Error::msg( format!(
      "unknown permission mode: {other} — valid values: default, acceptEdits, bypassPermissions"
    ) ) ),
  }
}

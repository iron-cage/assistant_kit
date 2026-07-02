use claude_runner_core::{
  ClaudeCommand, EffortLevel, IsolatedModel, IsolatedRunResult, ISOLATED_CLAUDE_MD,
  REFRESH_DEFAULT_MODEL, RunnerError, run_isolated_ext, signal_exit_code,
};
use claude_journal::{ EventRecord, EventType, JournalWriter };

/// Emit trace/dry-run diagnostics for a credential-operation command (`isolated` or `refresh`).
///
/// Reconstructs the `ClaudeCommand` exactly as `run_isolated_ext()` would build it
/// (model flag prepended, then `with_home(&temp_dir)`, `with_compact_window(compact_window)`,
/// then `with_args(args)`) and prints `describe_env()` + `describe()`.
///
/// `to_stdout`: `true` for `--dry-run` (user-facing preview → stdout, like `handle_dry_run`);
/// `false` for `--trace` (diagnostic → stderr).
///
/// `args` must be the fully-assembled arg list that will be passed to `run_isolated_ext()`,
/// including all injected flags (`--effort`, `--no-session-persistence`,
/// `--dangerously-skip-permissions`, `--no-chrome`, `--print`, message, passthrough).
/// WYSIWYG: the reconstructed command here must match what `run_isolated_ext()` actually runs.
///
/// Pitfall: if `run_isolated_ext()` in `claude_runner_core` is updated to modify the
/// `ClaudeCommand` beyond prepending the model flag, `with_home()`, `with_compact_window()`,
/// and `with_args()`, this trace will diverge — update both together.
fn emit_credential_trace
(
  label          : &str,
  creds_path     : &str,
  model          : &IsolatedModel,
  args           : &[ String ],
  timeout_secs   : u64,
  compact_window : Option< u32 >,
  to_stdout      : bool,
)
{
  // Reproduce the exact temp dir path and arg list that run_isolated_ext() will create.
  let temp_dir = std::env::temp_dir()
    .join( format!( "claude_isolated_{}", std::process::id() ) );
  let mut full_args = Vec::with_capacity( args.len() + 2 );
  if let Some( id ) = model.model_id()
  {
    full_args.push( "--model".to_string() );
    full_args.push( id.to_string() );
  }
  full_args.extend_from_slice( args );
  let preview = ClaudeCommand::new()
    .with_home( &temp_dir )
    .with_compact_window( compact_window )
    .with_args( full_args.iter().cloned() );
  let env_out = preview.describe_env();
  let cmd_out = preview.describe();
  if to_stdout
  {
    if !env_out.is_empty() { println!( "{env_out}" ); }
    println!( "{cmd_out}" );
  }
  else
  {
    eprintln!( "# clr {label}" );
    eprintln!( "# creds: {creds_path}" );
    eprintln!( "# timeout: {timeout_secs}s" );
    if !env_out.is_empty() { eprintln!( "{env_out}" ); }
    eprintln!( "{cmd_out}" );
  }
}

/// Execute an `isolated` or `refresh` subprocess command.
///
/// Builds the full argument list with all injected defaults (effort, session
/// persistence suppression, permissions, chrome), emits trace if requested,
/// reads the credentials file, calls `run_isolated()`, and propagates the result.
///
/// Injected flags are prepended before `--print` and message so that passthrough
/// args come last and can override them via claude's last-wins flag semantics.
///
/// - **Success (`exit_code >= 0`):** propagates the subprocess exit code.
/// - **Success (`exit_code == -1`, creds refreshed before timeout kill):** exits 0.
/// - **`Err(Timeout)` / `Err(TimeoutWithOutput)`:** exits 2.
/// - **Other errors:** exits 1 with an error message.
///
/// This function never returns; it always calls `std::process::exit`.
#[ allow( clippy::too_many_arguments, clippy::fn_params_excessive_bools, clippy::too_many_lines ) ]
pub( super ) fn run_isolated_command
(
  label             : &str,
  creds_path        : &str,
  timeout_secs      : u64,
  trace             : bool,
  dry_run           : bool,
  no_compact_window : bool,
  model             : IsolatedModel,
  effort            : EffortLevel,
  message           : Option< &str >,
  passthrough_args  : &[ String ],
  skip_perms        : bool,
  no_chrome         : bool,
  file_path         : Option< &str >,
  expect            : Option< &str >,
  expect_strategy   : Option< &str >,
  journal           : Option< JournalWriter >,
  output_file       : Option< &str >,
  do_strip_fences   : bool,
  output_style      : Option< &str >,
  summary_fields    : Option< &str >,
) -> !
{
  let compact_window : Option< u32 > = if no_compact_window { None } else { Some( 200_000 ) };
  // Build the full arg list with all injected defaults prepended before --print.
  // Order: [--no-chrome?] --effort <level> --no-session-persistence
  //        [--dangerously-skip-permissions?] [--print <msg>] [passthrough]
  let mut args : Vec< String > = Vec::new();
  if no_chrome    { args.push( "--no-chrome".to_string() ); }
  args.push( "--effort".to_string() );
  args.push( effort.as_str().to_string() );
  args.push( "--no-session-persistence".to_string() );
  if skip_perms   { args.push( "--dangerously-skip-permissions".to_string() ); }
  if let Some( m ) = message
  {
    args.push( "--print".to_string() );
    args.push( m.to_string() );
  }
  args.extend_from_slice( passthrough_args );

  // Emit trace/dry-run diagnostics before any I/O so they fire even when the creds file is missing.
  // Fix(R5-2): dry_run uses the same code path as trace — WYSIWYG, not a separate branch.
  // dry_run → stdout (user-facing preview, matches handle_dry_run behaviour for `run`).
  // trace   → stderr (diagnostic).
  if trace    { emit_credential_trace( label, creds_path, &model, &args, timeout_secs, compact_window, false ); }
  if dry_run  { emit_credential_trace( label, creds_path, &model, &args, timeout_secs, compact_window, true ); std::process::exit( 0 ); }

  let creds_json = match std::fs::read_to_string( creds_path )
  {
    Ok( s )  => s,
    Err( e ) =>
    {
      eprintln!( "Error: cannot read credentials file '{creds_path}': {e}" );
      std::process::exit( 1 );
    }
  };
  let run_result = if let Some( fp ) = file_path
  {
    run_isolated_with_stdin_file( &creds_json, args, timeout_secs, model, fp, compact_window )
  }
  else
  {
    run_isolated_ext( &creds_json, args, timeout_secs, model, compact_window )
  };
  match run_result
  {
    Ok( result ) =>
    {
      // Write back refreshed credentials if Claude updated them before
      // the subprocess finished (or before the timeout killed it).
      if let Some( ref new_creds ) = result.credentials
      {
        if let Err( e ) = std::fs::write( creds_path, new_creds )
        {
          eprintln!( "Warning: could not write back refreshed credentials to '{creds_path}': {e}" );
        }
      }
      if !result.stderr.is_empty() { eprint!( "{}", result.stderr ); }
      // exit_code == -1: killed by timeout but creds already refreshed — exit 0.
      let exit_code = if result.exit_code == -1 { 0 } else { result.exit_code };
      // Apply --expect validation on success; non-zero exits skip validation.
      let stdout = if exit_code == 0
      {
        apply_isolated_expect( &result.stdout, expect, expect_strategy )
      }
      else
      {
        result.stdout
      };
      // Output processing chain: strip fences → render summary → print → write file.
      let stdout = if do_strip_fences
      {
        super::fence::strip_fences( &stdout )
      }
      else { stdout };
      let stdout = if output_style == Some( "summary" )
      {
        super::summary::render_summary( &stdout, summary_fields ).unwrap_or( stdout )
      }
      else { stdout };
      if !stdout.is_empty() { print!( "{stdout}" ); }
      if let Some( path ) = output_file
      {
        if let Err( e ) = std::fs::write( path, &stdout )
        {
          eprintln!( "Warning: could not write output file '{path}': {e}" );
        }
      }
      if let Some( ref w ) = journal
      {
        let mut ev             = EventRecord::new( EventType::Credential );
        ev.fields.command      = Some( label.to_string() );
        ev.fields.exit_code    = Some( exit_code );
        ev.fields.creds        = Some( creds_path.to_string() );
        ev.fields.timeout_secs = Some( u32::try_from( timeout_secs ).unwrap_or( u32::MAX ) );
        let _ = w.append( &ev );
      }
      std::process::exit( exit_code );
    }
    Err( RunnerError::Timeout { secs } | RunnerError::TimeoutWithOutput { secs, .. } ) =>
    {
      eprintln!( "Error: {label} subprocess timed out after {secs} seconds" );
      // WHY exit 2 and not exit 4: isolated/refresh timeout is semantically distinct from
      // run/ask timeout.  run/ask uses poll_timeout() in execution.rs → exit 4 (TSK-202).
      // isolated/refresh use RunnerError::Timeout from claude_runner_core::isolated — exit 2
      // signals "no credentials refreshed, no subprocess output" (see 001_design_decisions.md
      // line 138).  Do NOT change to exit 4; that would break the isolated/refresh contract.
      if let Some( ref w ) = journal
      {
        let mut ev             = EventRecord::new( EventType::Credential );
        ev.fields.command      = Some( label.to_string() );
        ev.fields.exit_code    = Some( 2 );
        ev.fields.creds        = Some( creds_path.to_string() );
        ev.fields.timeout_secs = Some( u32::try_from( secs ).unwrap_or( u32::MAX ) );
        let _ = w.append( &ev );
      }
      std::process::exit( 2 );
    }
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  }
}

/// Validate isolated subprocess stdout against `--expect`; apply strategy on mismatch.
///
/// Returns the original stdout when the pattern matches (or when `expect` is None).
/// Exits directly on mismatch:
/// - `fail` (or no strategy) → exit 3.
/// - `default:<V>` → print `<V>` to stdout, exit 0.
/// - `retry` → exit 1 (explicitly unsupported for isolated's one-shot semantics).
fn apply_isolated_expect( stdout : &str, expect : Option< &str >, strategy : Option< &str > ) -> String
{
  let Some( pattern ) = expect else { return stdout.to_string(); };
  let allowed : Vec< String > = pattern.split( '|' )
    .map( | s | s.trim().to_lowercase() )
    .collect();
  let trimmed = stdout.trim().to_lowercase();
  if allowed.iter().any( | v | v.as_str() == trimmed ) { return stdout.to_string(); }
  match strategy
  {
    Some( s ) if s.starts_with( "default:" ) =>
    {
      let fallback = s[ "default:".len() .. ].to_string();
      print!( "{fallback}" );
      std::process::exit( 0 );
    }
    Some( "retry" ) =>
    {
      eprintln!( "Error: --expect-strategy retry is not supported for isolated (one-shot semantics)" );
      std::process::exit( 1 );
    }
    _ =>
    {
      eprintln!(
        "Error: [Validation] expected \"{pattern}\", got \"{}\" (exit 3)",
        stdout.trim()
      );
      std::process::exit( 3 );
    }
  }
}

/// Run an isolated subprocess with a file piped as stdin.
///
/// Mirrors [`run_isolated`] exactly but calls `.with_stdin_file(file_path)` on the
/// `ClaudeCommand` so the given file is fed as stdin to the subprocess.
/// Pre-condition: `file_path` must exist; caller is responsible for existence check.
///
/// Pitfall: this function duplicates the temp-HOME lifecycle from `run_isolated()` in
/// `claude_runner_core` to avoid modifying out-of-scope code. If `run_isolated()` gains
/// new setup steps, update both together.
#[ allow( clippy::too_many_lines ) ]
fn run_isolated_with_stdin_file
(
  credentials_json : &str,
  args             : Vec< String >,
  timeout_secs     : u64,
  model            : IsolatedModel,
  file_path        : &str,
  compact_window   : Option< u32 >,
) -> Result< IsolatedRunResult, RunnerError >
{
  use core::time::Duration;

  let temp_dir   = std::env::temp_dir()
    .join( format!( "claude_isolated_{}", std::process::id() ) );
  let claude_dir = temp_dir.join( ".claude" );
  std::fs::create_dir_all( &claude_dir )
    .map_err( | e | RunnerError::TempDirFailed( e.to_string() ) )?;

  let creds_path = claude_dir.join( ".credentials.json" );
  std::fs::write( &creds_path, credentials_json )
    .map_err( | e | RunnerError::Io( e.to_string() ) )?;
  std::fs::write( claude_dir.join( "CLAUDE.md" ), ISOLATED_CLAUDE_MD )
    .map_err( | e | RunnerError::Io( e.to_string() ) )?;

  let mut full_args = Vec::with_capacity( args.len() + 2 );
  if let Some( id ) = model.model_id()
  {
    full_args.push( "--model".to_string() );
    full_args.push( id.to_string() );
  }
  full_args.extend( args );
  let cmd = ClaudeCommand::new()
    .with_home( &temp_dir )
    .with_home_isolation()
    .with_compact_window( compact_window )
    .with_stdin_file( std::path::PathBuf::from( file_path ) )
    .with_args( full_args );

  let mut child = cmd.spawn_piped().map_err( | e |
  {
    if e.kind() == std::io::ErrorKind::NotFound
    {
      RunnerError::ClaudeNotFound
    }
    else
    {
      RunnerError::Io( e.to_string() )
    }
  } )?;

  // Take pipe handles before the poll loop so background reader threads own them.
  // Without draining, a subprocess writing >64 KiB blocks on pipe write,
  // try_wait() never returns Some(_), and the operation hangs until timeout.
  let stdout_pipe = child.stdout.take().expect( "stdout piped by spawn_piped" );
  let stderr_pipe = child.stderr.take().expect( "stderr piped by spawn_piped" );

  let stdout_t = std::thread::spawn( move || -> Vec< u8 >
  {
    use std::io::Read as _;
    let mut buf = Vec::new();
    let _ = { let mut r = stdout_pipe; r.read_to_end( &mut buf ) };
    buf
  } );
  let stderr_t = std::thread::spawn( move || -> Vec< u8 >
  {
    use std::io::Read as _;
    let mut buf = Vec::new();
    let _ = { let mut r = stderr_pipe; r.read_to_end( &mut buf ) };
    buf
  } );

  let deadline : Option< std::time::Instant > = if timeout_secs > 0
  {
    Some( std::time::Instant::now() + Duration::from_secs( timeout_secs ) )
  }
  else
  {
    None
  };
  let mut timed_out = false;
  loop
  {
    match child.try_wait()
    {
      Ok( Some( _ ) ) => { break; }
      Ok( None ) =>
      {
        if deadline.is_some_and( | d | std::time::Instant::now() >= d )
        {
          timed_out = true;
          let _ = child.kill();
          break;
        }
        std::thread::sleep( Duration::from_millis( 50 ) );
      }
      Err( e ) =>
      {
        // Drop reader threads before returning — do NOT join (shell children may hold pipes).
        drop( stdout_t );
        drop( stderr_t );
        let _ = std::fs::remove_dir_all( &temp_dir );
        return Err( RunnerError::Io( e.to_string() ) );
      }
    }
  }

  let credentials = std::fs::read_to_string( &creds_path )
    .ok()
    .and_then( | new |
    {
      if new.as_bytes() == credentials_json.as_bytes() { None } else { Some( new ) }
    } );

  let _ = std::fs::remove_dir_all( &temp_dir );

  if timed_out
  {
    // Drop reader threads — do NOT join.  Shell child processes may inherit the pipe
    // write end; joining would block until they exit too (same as execute_print_attempt fix).
    drop( stdout_t );
    drop( stderr_t );
    if credentials.is_some()
    {
      return Ok( IsolatedRunResult
      {
        exit_code   : -1,
        stdout      : String::new(),
        stderr      : String::new(),
        credentials,
      } );
    }
    return Err( RunnerError::TimeoutWithOutput
    {
      secs           : timeout_secs,
      partial_stdout : String::new(),
    } );
  }

  // Normal exit: join reader threads to collect all buffered data.
  let stdout_bytes = stdout_t.join().unwrap_or_default();
  let stderr_bytes = stderr_t.join().unwrap_or_default();
  let stdout = String::from_utf8_lossy( &stdout_bytes ).to_string();
  let stderr = String::from_utf8_lossy( &stderr_bytes ).to_string();
  let exit_code = signal_exit_code( &child.wait()
    .map_err( | e | RunnerError::Io( e.to_string() ) )? );
  Ok( IsolatedRunResult { exit_code, stdout, stderr, credentials } )
}

/// Execute the `refresh` subcommand.
///
/// Spawns `claude --print "."` inside an isolated temp HOME so the Claude binary
/// performs its OAuth token refresh at startup. Writes the refreshed credentials
/// back to `creds_path` if the subprocess updated them.
///
/// Injected defaults for refresh: `--no-chrome` (HTTP-only OAuth exchange;
/// no browser context needed), `--effort low` (trivial ping), `--no-session-persistence`
/// (temp HOME discarded after run). No `--dangerously-skip-permissions` (no tool use).
///
/// This function never returns; it always calls `std::process::exit`.
pub( super ) fn run_refresh_command
(
  creds_path        : &str,
  timeout_secs      : u64,
  trace             : bool,
  dry_run           : bool,
  no_compact_window : bool,
  journal           : Option< JournalWriter >,
) -> !
{
  run_isolated_command(
    "refresh",
    creds_path,
    timeout_secs,
    trace,
    dry_run,
    no_compact_window,
    IsolatedModel::Specific( REFRESH_DEFAULT_MODEL.to_string() ),
    EffortLevel::Low,
    Some( "." ),
    &[],
    false,  // no skip-perms: refresh is HTTP-only, invokes no tools
    true,   // no-chrome: OAuth token exchange is pure HTTP; suppress browser context
    None,   // no stdin file for refresh
    None,   // no expect pattern for refresh
    None,   // no expect strategy for refresh
    journal,
    None,   // no output file for refresh
    false,  // no fence stripping for refresh
    None,   // no output style for refresh
    None,   // no summary fields for refresh
  );
}

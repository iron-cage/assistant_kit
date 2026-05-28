use claude_runner_core::{ ClaudeCommand, IsolatedModel, RunnerError, run_isolated };

/// Emit trace diagnostics for a credential-operation command (`isolated` or `refresh`).
///
/// Reconstructs the `ClaudeCommand` exactly as `run_isolated()` would build it
/// (model flag prepended, then `with_home(&temp_dir)`, then `with_args(args)`) and
/// prints `describe_env()` + `describe()` to stderr, matching the format of `run` trace.
///
/// Pitfall: if `run_isolated()` in `claude_runner_core` is updated to modify the
/// `ClaudeCommand` beyond prepending the model flag, `with_home()`, and `with_args()`,
/// this trace will diverge — update both together.
fn emit_credential_trace
(
  label        : &str,
  creds_path   : &str,
  model        : &IsolatedModel,
  args         : &[ String ],
  timeout_secs : u64,
)
{
  // Reproduce the exact temp dir path and arg list that run_isolated() will create.
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
    .with_args( full_args.iter().cloned() );
  let env_out = preview.describe_env();
  let cmd_out = preview.describe();
  eprintln!( "# clr {label}" );
  eprintln!( "# creds: {creds_path}" );
  eprintln!( "# timeout: {timeout_secs}s" );
  if !env_out.is_empty() { eprintln!( "{env_out}" ); }
  eprintln!( "{cmd_out}" );
}

/// Execute the `isolated` subcommand.
///
/// Reads the credentials file at `creds_path`, builds the argument list for
/// `run_isolated`, then handles the result:
///
/// - **Success (`exit_code >= 0`):** propagates the subprocess exit code.
/// - **Success (`exit_code == -1`, creds refreshed at startup before timeout):**
///   writes back updated credentials and exits 0.
/// - **`Err(Timeout)`:** subprocess exceeded the deadline without refreshing
///   credentials — exits 2.
/// - **Other errors:** exits 1 with an error message.
///
/// This function never returns; it always calls `std::process::exit`.
pub( super ) fn run_isolated_command
(
  creds_path       : &str,
  timeout_secs     : u64,
  trace            : bool,
  model            : IsolatedModel,
  message          : Option< &str >,
  passthrough_args : &[ String ],
) -> !
{
  // Build args first so trace can emit them before any I/O that may exit early.
  // This ensures trace fires even when the creds file is missing (matching
  // `run_refresh_command` which also emits trace before any validation).
  let mut args : Vec< String > = message
    .map( | m | vec![ "--print".to_string(), m.to_string() ] )
    .unwrap_or_default();
  args.extend_from_slice( passthrough_args );
  if trace { emit_credential_trace( "isolated", creds_path, &model, &args, timeout_secs ); }
  let creds_json = match std::fs::read_to_string( creds_path )
  {
    Ok( s )  => s,
    Err( e ) =>
    {
      eprintln!( "Error: cannot read credentials file '{creds_path}': {e}" );
      std::process::exit( 1 );
    }
  };
  match run_isolated( &creds_json, args, timeout_secs, model )
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
      if !result.stdout.is_empty() { print!( "{}", result.stdout ); }
      // exit_code == -1: subprocess was killed by timeout BUT credentials were
      // refreshed before the kill — per spec, exit 0 and write-back already done.
      let exit_code = if result.exit_code == -1 { 0 } else { result.exit_code };
      std::process::exit( exit_code );
    }
    Err( RunnerError::Timeout { secs } ) =>
    {
      eprintln!( "Error: isolated subprocess timed out after {secs} seconds" );
      std::process::exit( 2 );
    }
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  }
}

/// Execute the `refresh` subcommand.
///
/// Spawns `claude --print "."` inside an isolated temp HOME so the Claude binary
/// performs its OAuth token refresh at startup. Writes the refreshed credentials
/// back to `creds_path` if the subprocess updated them.
///
/// This function never returns; it always calls `std::process::exit`.
pub( super ) fn run_refresh_command
(
  creds_path   : &str,
  timeout_secs : u64,
  trace        : bool,
) -> !
{
  // Fixed args: trigger Claude's startup token refresh with a trivial prompt.
  let fixed_args = vec![ "--print".to_string(), ".".to_string() ];
  if trace { emit_credential_trace( "refresh", creds_path, &IsolatedModel::Default, &fixed_args, timeout_secs ); }
  // Pass trace=false: refresh already emitted its own trace above.
  run_isolated_command( creds_path, timeout_secs, false, IsolatedModel::Default, None, &fixed_args );
}

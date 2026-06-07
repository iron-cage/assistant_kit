use claude_runner_core::{ ClaudeCommand, EffortLevel, IsolatedModel, REFRESH_DEFAULT_MODEL, RunnerError, run_isolated };

/// Emit trace diagnostics for a credential-operation command (`isolated` or `refresh`).
///
/// Reconstructs the `ClaudeCommand` exactly as `run_isolated()` would build it
/// (model flag prepended, then `with_home(&temp_dir)`, then `with_args(args)`) and
/// prints `describe_env()` + `describe()` to stderr, matching the format of `run` trace.
///
/// `args` must be the fully-assembled arg list that will be passed to `run_isolated()`,
/// including all injected flags (`--effort`, `--no-session-persistence`,
/// `--dangerously-skip-permissions`, `--no-chrome`, `--print`, message, passthrough).
/// WYSIWYG: the reconstructed command here must match what `run_isolated()` actually runs.
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
pub( super ) fn run_isolated_command
(
  label            : &str,
  creds_path       : &str,
  timeout_secs     : u64,
  trace            : bool,
  model            : IsolatedModel,
  effort           : EffortLevel,
  message          : Option< &str >,
  passthrough_args : &[ String ],
  skip_perms       : bool,
  no_chrome        : bool,
) -> !
{
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

  // Emit trace before any I/O so it fires even when the creds file is missing.
  if trace { emit_credential_trace( label, creds_path, &model, &args, timeout_secs ); }

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
      // exit_code == -1: killed by timeout but creds already refreshed — exit 0.
      let exit_code = if result.exit_code == -1 { 0 } else { result.exit_code };
      std::process::exit( exit_code );
    }
    Err( RunnerError::Timeout { secs } | RunnerError::TimeoutWithOutput { secs, .. } ) =>
    {
      eprintln!( "Error: {label} subprocess timed out after {secs} seconds" );
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
/// Injected defaults for refresh: `--no-chrome` (HTTP-only OAuth exchange;
/// no browser context needed), `--effort low` (trivial ping), `--no-session-persistence`
/// (temp HOME discarded after run). No `--dangerously-skip-permissions` (no tool use).
///
/// This function never returns; it always calls `std::process::exit`.
pub( super ) fn run_refresh_command
(
  creds_path   : &str,
  timeout_secs : u64,
  trace        : bool,
) -> !
{
  run_isolated_command(
    "refresh",
    creds_path,
    timeout_secs,
    trace,
    IsolatedModel::Specific( REFRESH_DEFAULT_MODEL.to_string() ),
    EffortLevel::Low,
    Some( "." ),
    &[],
    false, // no skip-perms: refresh is HTTP-only, invokes no tools
    true,  // no-chrome: OAuth token exchange is pure HTTP; suppress browser context
  );
}

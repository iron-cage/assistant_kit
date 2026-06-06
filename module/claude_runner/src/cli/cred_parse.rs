//! Credential subcommand argument types and parsers.
//!
//! Contains `IsolatedArgs`, `RefreshArgs`, their parsers, and env-var fallback application
//! for the `isolated` and `refresh` subcommands.

use claude_core::paths::ClaudePaths;
use error_tools::{ Error, Result };
use super::parse::{ next_value, env_str, env_bool };

/// Parsed arguments for the `isolated` subcommand.
#[ derive( Default ) ]
pub( super ) struct IsolatedArgs
{
  pub( super ) creds_path       : String,
  pub( super ) timeout_secs     : u64,
  pub( super ) trace            : bool,
  pub( super ) message          : Option< String >,
  pub( super ) passthrough_args : Vec< String >,
}

/// Parsed arguments for the `refresh` subcommand.
pub( super ) struct RefreshArgs
{
  pub( super ) creds_path   : String,
  pub( super ) timeout_secs : u64,
  pub( super ) trace        : bool,
}

/// Parse a raw string as a `u64` timeout in seconds.
///
/// Rejects negative numbers (which start with `-` and fail `u64` parsing)
/// and non-numeric strings with a clear error message.
fn parse_timeout( raw : &str ) -> Result< u64 >
{
  raw.parse::< u64 >().map_err( | _ |
    Error::msg( format!(
      "invalid --timeout value: {raw}\n\
       Expected non-negative integer"
    ) )
  )
}

/// Apply `CLR_CREDS`, `CLR_TIMEOUT`, and `CLR_TRACE` env var fallbacks for credential subcommands.
///
/// Only updates each field when it is still at its default value — CLI flags always win.
/// `default_timeout` is the sentinel that signals "not set by CLI"; an explicit `--timeout N`
/// that happens to equal the default is indistinguishable (accepted limitation).
fn apply_cred_env_vars(
  creds_path      : &mut String,
  timeout_secs    : &mut u64,
  default_timeout : u64,
  trace           : &mut bool,
)
{
  if creds_path.is_empty()
  {
    *creds_path = env_str( "CLR_CREDS" ).unwrap_or_default();
  }
  if creds_path.is_empty()
  {
    if let Some( paths ) = ClaudePaths::new()
    {
      *creds_path = paths.credentials_file().to_string_lossy().into_owned();
    }
  }
  if *timeout_secs == default_timeout
  {
    if let Some( v ) = env_str( "CLR_TIMEOUT" )
    {
      if let Ok( secs ) = v.parse::< u64 >() { *timeout_secs = secs; }
    }
  }
  if !*trace { *trace = env_bool( "CLR_TRACE" ); }
}

/// Parse `tokens` as arguments to the `isolated` subcommand.
///
/// Recognises `--creds <FILE>`, `--timeout <SECS>`, a positional `[MESSAGE]`,
/// and `-- <PASSTHROUGH...>`. Everything after `--` is collected verbatim.
/// Unknown flags (before `--`) are rejected with an error.
pub( super ) fn parse_isolated_args( tokens : &[ String ] ) -> Result< IsolatedArgs >
{
  let mut creds_path       : Option< String > = None;
  let mut timeout_secs     : u64              = 30;
  let mut trace            : bool             = false;
  let mut message_parts    : Vec< String >    = Vec::new();
  let mut passthrough_args : Vec< String >    = Vec::new();
  let mut i = 0;
  while i < tokens.len()
  {
    let token = tokens[ i ].as_str();
    match token
    {
      "--" =>
      {
        passthrough_args.extend( tokens[ i + 1 .. ].iter().cloned() );
        break;
      }
      "--creds" =>
      {
        creds_path = Some( next_value( tokens, i + 1, "--creds" )?.to_string() );
        i += 1;
      }
      "--timeout" =>
      {
        let raw      = next_value( tokens, i + 1, "--timeout" )?;
        timeout_secs = parse_timeout( raw )?;
        i += 1;
      }
      "--trace" =>
      {
        trace = true;
      }
      // Fix(BUG-222): parse_isolated_args fell through --help to the
      // starts_with('-') catch-all, returning Err("unknown option: --help") → exit 1.
      // Root cause: no explicit --help arm in parse_isolated_args; global parse_args has
      // one but parse_isolated_args was written without it.
      // Pitfall: any catch-all for unknown flags silently swallows --help and -h;
      // always add an explicit --help arm before the catch-all in every subcommand parser.
      "-h" | "--help" => { super::help::print_isolated_help(); }
      s if s.starts_with( '-' ) =>
      {
        return Err( Error::msg( format!(
          "unknown option: {s}\nRun with --help for usage."
        ) ) );
      }
      _ =>
      {
        if !tokens[ i ].is_empty() { message_parts.push( tokens[ i ].clone() ); }
      }
    }
    i += 1;
  }
  // Note: creds_path validation is deferred to after apply_isolated_env_vars() is called
  // in run_cli() so that CLR_CREDS env var can supply the value before the check.
  let creds_path = creds_path.unwrap_or_default();
  let message    = if message_parts.is_empty() { None } else { Some( message_parts.join( " " ) ) };
  Ok( IsolatedArgs { creds_path, timeout_secs, trace, message, passthrough_args } )
}

/// Apply `CLR_CREDS`, `CLR_TIMEOUT`, and `CLR_TRACE` env var fallbacks for `isolated`.
///
/// Delegates to [`apply_cred_env_vars`] with isolated's timeout sentinel (30).
pub( super ) fn apply_isolated_env_vars( parsed : &mut IsolatedArgs )
{
  apply_cred_env_vars( &mut parsed.creds_path, &mut parsed.timeout_secs, 30, &mut parsed.trace );
}

/// Parse `tokens` as arguments to the `refresh` subcommand.
///
/// Recognises `--creds <FILE>`, `--timeout <SECS>`, and `--trace`.
/// The `refresh` command takes no positional arguments — only credential options.
pub( super ) fn parse_refresh_args( tokens : &[ String ] ) -> Result< RefreshArgs >
{
  let mut creds_path   : Option< String > = None;
  let mut timeout_secs : u64              = 45;
  let mut trace        : bool             = false;
  let mut i = 0;
  while i < tokens.len()
  {
    let token = tokens[ i ].as_str();
    match token
    {
      "--creds" =>
      {
        creds_path = Some( next_value( tokens, i + 1, "--creds" )?.to_string() );
        i += 1;
      }
      "--timeout" =>
      {
        let raw      = next_value( tokens, i + 1, "--timeout" )?;
        timeout_secs = parse_timeout( raw )?;
        i += 1;
      }
      "--trace" =>
      {
        trace = true;
      }
      "-h" | "--help" => { super::help::print_refresh_help(); }
      s if s.starts_with( '-' ) =>
      {
        return Err( Error::msg( format!(
          "unknown option: {s}\nRun with --help for usage."
        ) ) );
      }
      _ => {} // refresh accepts no positional arguments
    }
    i += 1;
  }
  let creds_path = creds_path.unwrap_or_default();
  Ok( RefreshArgs { creds_path, timeout_secs, trace } )
}

/// Apply `CLR_CREDS`, `CLR_TIMEOUT`, and `CLR_TRACE` env var fallbacks for `refresh`.
///
/// Delegates to [`apply_cred_env_vars`] with refresh's timeout sentinel (45).
pub( super ) fn apply_refresh_env_vars( parsed : &mut RefreshArgs )
{
  apply_cred_env_vars( &mut parsed.creds_path, &mut parsed.timeout_secs, 45, &mut parsed.trace );
}

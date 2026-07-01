//! Credential subcommand argument types and parsers.
//!
//! Contains `IsolatedArgs`, `RefreshArgs`, their parsers, and env-var fallback application
//! for the `isolated` and `refresh` subcommands.
//!
//! # Type Note: u64 vs u32 for `timeout_secs`
//!
//! `IsolatedArgs` and `RefreshArgs` store `timeout_secs` as `u64`, while the main
//! `CliArgs` in `parse.rs` uses `u32`. The divergence is intentional: isolated/refresh
//! subprocesses may run longer-lived operations where the wider type provides headroom.
//! `parse_timeout_secs` here returns `u64`; the one in `parse.rs` returns `u32`. Do not
//! unify the types without updating all struct fields and callers.

use claude_core::paths::ClaudePaths;
use error_tools::{ Error, Result };
use super::parse::next_value;
use super::env::{ env_bool, env_str };

/// Parsed arguments for the `isolated` subcommand.
#[ derive( Default ) ]
pub( super ) struct IsolatedArgs
{
  pub( super ) creds_path       : String,
  pub( super ) timeout_secs     : u64,
  pub( super ) trace            : bool,
  pub( super ) dry_run          : bool,
  pub( super ) message          : Option< String >,
  pub( super ) passthrough_args : Vec< String >,
  pub( super ) dir              : Option< String >,
  pub( super ) add_dirs         : Vec< String >,
  pub( super ) file             : Option< String >,
  pub( super ) expect           : Option< String >,
  pub( super ) expect_strategy  : Option< String >,
  pub( super ) journal          : Option< String >,
  pub( super ) journal_dir      : Option< String >,
  pub( super ) output_file      : Option< String >,
  pub( super ) strip_fences     : bool,
  pub( super ) output_style     : Option< String >,
  pub( super ) summary_fields   : Option< String >,
  pub( super ) no_compact_window : bool,
}

/// Parsed arguments for the `refresh` subcommand.
pub( super ) struct RefreshArgs
{
  pub( super ) creds_path       : String,
  pub( super ) timeout_secs     : u64,
  pub( super ) trace            : bool,
  pub( super ) dry_run          : bool,
  pub( super ) no_compact_window : bool,
  pub( super ) journal          : Option< String >,
  pub( super ) journal_dir      : Option< String >,
}

/// Parse a raw string as a `u64` timeout in seconds.
///
/// Rejects negative numbers (which start with `-` and fail `u64` parsing)
/// and non-numeric strings with a clear error message.
fn parse_timeout_secs( raw : &str ) -> Result< u64 >
{
  raw.parse::< u64 >().map_err( | _ |
    Error::msg( format!(
      "invalid --timeout value: {raw}\n\
       Expected non-negative integer"
    ) )
  )
}

/// Validate and return a journal level string (`full`, `meta`, or `off`).
fn validate_journal_level( v : &str ) -> Result< String >
{
  if matches!( v, "full" | "meta" | "off" )
  {
    Ok( v.to_string() )
  }
  else
  {
    Err( Error::msg( format!(
      "invalid --journal value '{v}' — expected: full, meta, off"
    ) ) )
  }
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
  let mut args          = IsolatedArgs { timeout_secs : 30, ..Default::default() };
  let mut creds_path_raw : Option< String > = None;
  let mut message_parts  : Vec< String >    = Vec::new();
  let mut i = 0;
  while i < tokens.len()
  {
    let token = tokens[ i ].as_str();
    match token
    {
      "--" =>
      {
        args.passthrough_args.extend( tokens[ i + 1 .. ].iter().cloned() );
        break;
      }
      "--creds" =>
      {
        creds_path_raw = Some( next_value( tokens, i + 1, "--creds" )?.to_string() );
        i += 1;
      }
      "--timeout" =>
      {
        args.timeout_secs = parse_timeout_secs( next_value( tokens, i + 1, "--timeout" )? )?;
        i += 1;
      }
      "--trace"            => { args.trace            = true; }
      "--dry-run"          => { args.dry_run          = true; }
      "--no-compact-window" => { args.no_compact_window = true; }
      "--dir" =>
      {
        args.dir = Some( next_value( tokens, i + 1, "--dir" )?.to_string() );
        i += 1;
      }
      "--add-dir" =>
      {
        args.add_dirs.push( next_value( tokens, i + 1, "--add-dir" )?.to_string() );
        i += 1;
      }
      "--file" =>
      {
        args.file = Some( next_value( tokens, i + 1, "--file" )?.to_string() );
        i += 1;
      }
      "--expect" =>
      {
        args.expect = Some( next_value( tokens, i + 1, "--expect" )?.to_string() );
        i += 1;
      }
      "--expect-strategy" =>
      {
        args.expect_strategy = Some( next_value( tokens, i + 1, "--expect-strategy" )?.to_string() );
        i += 1;
      }
      "--journal" =>
      {
        args.journal = Some( validate_journal_level( next_value( tokens, i + 1, "--journal" )? )? );
        i += 1;
      }
      "--journal-dir" =>
      {
        args.journal_dir = Some( next_value( tokens, i + 1, "--journal-dir" )?.to_string() );
        i += 1;
      }
      "--output-file" =>
      {
        args.output_file = Some( next_value( tokens, i + 1, "--output-file" )?.to_string() );
        i += 1;
      }
      "--strip-fences" => { args.strip_fences = true; }
      "--output-style" =>
      {
        args.output_style = Some( next_value( tokens, i + 1, "--output-style" )?.to_string() );
        i += 1;
      }
      "--summary-fields" =>
      {
        args.summary_fields = Some( next_value( tokens, i + 1, "--summary-fields" )?.to_string() );
        i += 1;
      }
      // Fix(BUG-222): explicit --help arm prevents catch-all from swallowing help flags.
      // Root cause: no --help arm in subcommand parser; catch-all returned Err.
      // Pitfall: always add --help before the starts_with('-') catch-all.
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
  // creds_path validation deferred to after apply_isolated_env_vars() so CLR_CREDS can supply it.
  args.creds_path = creds_path_raw.unwrap_or_default();
  args.message    = if message_parts.is_empty() { None } else { Some( message_parts.join( " " ) ) };
  Ok( args )
}

/// Apply `CLR_CREDS`, `CLR_TIMEOUT`, `CLR_TRACE`, `CLR_DIR`, `CLR_ADD_DIR`,
/// `CLR_JOURNAL`, `CLR_JOURNAL_DIR`, `CLR_OUTPUT_FILE`, `CLR_STRIP_FENCES`,
/// `CLR_OUTPUT_STYLE`, and `CLR_SUMMARY_FIELDS` env var fallbacks for `isolated`.
///
/// Delegates to [`apply_cred_env_vars`] with isolated's timeout sentinel (30).
pub( super ) fn apply_isolated_env_vars( parsed : &mut IsolatedArgs ) -> Result< () >
{
  apply_cred_env_vars( &mut parsed.creds_path, &mut parsed.timeout_secs, 30, &mut parsed.trace );
  if parsed.dir.is_none()       { parsed.dir     = env_str( "CLR_DIR" ); }
  if parsed.add_dirs.is_empty()
  {
    if let Some( v ) = env_str( "CLR_ADD_DIR" ) { parsed.add_dirs.push( v ); }
  }
  if parsed.journal.is_none()
  {
    if let Some( v ) = env_str( "CLR_JOURNAL" )
    {
      if !matches!( v.as_str(), "full" | "meta" | "off" )
      {
        return Err( Error::msg( format!(
          "CLR_JOURNAL: invalid value '{v}' — expected: full, meta, off"
        ) ) );
      }
      parsed.journal = Some( v );
    }
  }
  if parsed.journal_dir.is_none()  { parsed.journal_dir  = env_str( "CLR_JOURNAL_DIR" ); }
  if parsed.output_file.is_none()  { parsed.output_file  = env_str( "CLR_OUTPUT_FILE" ); }
  if !parsed.strip_fences          { parsed.strip_fences = env_bool( "CLR_STRIP_FENCES" ); }
  if parsed.output_style.is_none() { parsed.output_style = env_str( "CLR_OUTPUT_STYLE" ); }
  if parsed.summary_fields.is_none() { parsed.summary_fields = env_str( "CLR_SUMMARY_FIELDS" ); }
  if !parsed.no_compact_window { parsed.no_compact_window = env_bool( "CLR_NO_COMPACT_WINDOW" ); }
  Ok( () )
}

/// Parse `tokens` as arguments to the `refresh` subcommand.
///
/// Recognises `--creds <FILE>`, `--timeout <SECS>`, and `--trace`.
/// The `refresh` command takes no positional arguments — only credential options.
pub( super ) fn parse_refresh_args( tokens : &[ String ] ) -> Result< RefreshArgs >
{
  let mut creds_path        : Option< String > = None;
  let mut timeout_secs      : u64              = 45;
  let mut trace             : bool             = false;
  let mut dry_run           : bool             = false;
  let mut no_compact_window : bool             = false;
  let mut journal           : Option< String > = None;
  let mut journal_dir       : Option< String > = None;
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
        timeout_secs = parse_timeout_secs( raw )?;
        i += 1;
      }
      "--trace"             => { trace             = true; }
      "--dry-run"           => { dry_run           = true; }
      "--no-compact-window" => { no_compact_window = true; }
      "--journal" =>
      {
        journal = Some( validate_journal_level( next_value( tokens, i + 1, "--journal" )? )? );
        i += 1;
      }
      "--journal-dir" =>
      {
        journal_dir = Some( next_value( tokens, i + 1, "--journal-dir" )?.to_string() );
        i += 1;
      }
      "-h" | "--help" => { super::help::print_refresh_help(); }
      s if s.starts_with( '-' ) =>
      {
        return Err( Error::msg( format!(
          "unknown option: {s}\nRun with --help for usage."
        ) ) );
      }
      _ =>
      {
        return Err( Error::msg( format!(
          "unexpected argument: {token}\n`clr refresh` accepts no positional arguments.\nRun with --help for usage."
        ) ) );
      }
    }
    i += 1;
  }
  let creds_path = creds_path.unwrap_or_default();
  Ok( RefreshArgs { creds_path, timeout_secs, trace, dry_run, no_compact_window, journal, journal_dir } )
}

/// Apply `CLR_CREDS`, `CLR_TIMEOUT`, `CLR_TRACE`, `CLR_JOURNAL`, and `CLR_JOURNAL_DIR`
/// env var fallbacks for `refresh`.
///
/// Delegates to [`apply_cred_env_vars`] with refresh's timeout sentinel (45).
pub( super ) fn apply_refresh_env_vars( parsed : &mut RefreshArgs ) -> Result< () >
{
  apply_cred_env_vars( &mut parsed.creds_path, &mut parsed.timeout_secs, 45, &mut parsed.trace );
  if parsed.journal.is_none()
  {
    if let Some( v ) = env_str( "CLR_JOURNAL" )
    {
      if !matches!( v.as_str(), "full" | "meta" | "off" )
      {
        return Err( Error::msg( format!(
          "CLR_JOURNAL: invalid value '{v}' — expected: full, meta, off"
        ) ) );
      }
      parsed.journal = Some( v );
    }
  }
  if parsed.journal_dir.is_none()    { parsed.journal_dir    = env_str( "CLR_JOURNAL_DIR" ); }
  if !parsed.dry_run                 { parsed.dry_run         = env_bool( "CLR_DRY_RUN" ); }
  if !parsed.no_compact_window       { parsed.no_compact_window = env_bool( "CLR_NO_COMPACT_WINDOW" ); }
  Ok( () )
}

//! `.status` — installation state, process count, and active account.

use unilang::data::{ ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;

use crate::output::{ OutputFormat, OutputOptions, json_escape };
use claude_runner_core::process::find_claude_processes;
use claude_version_core::version::{
  extract_semver, get_claude_version_raw, get_installed_version, read_preferred_version,
};

/// Read the active account name from the credential store `_active` marker.
///
/// Checks `$HOME/.persistent/claude/credential/_active`.
fn get_active_account() -> Option< String >
{
  let root = std::env::var_os( "HOME" ).filter( | v | !v.is_empty() )?;
  let marker = std::path::Path::new( &root )
    .join( ".persistent" ).join( "claude" ).join( "credential" ).join( "_active" );
  std::fs::read_to_string( marker )
  .ok()
  .map( | s | s.trim().to_string() )
  .filter( | s | !s.is_empty() )
}

/// `.status` — show installation state, process count, and active account.
///
/// # Errors
///
/// Returns `Err` only when `format::` has an unrecognised value.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn status_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts     = OutputOptions::from_cmd( &cmd )?;
  let version  = get_installed_version()
    .or_else( || get_claude_version_raw().map( | r | extract_semver( &r ).to_string() ) )
    .unwrap_or_else( || "not found".to_string() );
  let processes = find_claude_processes().len();
  let account  = get_active_account().unwrap_or_else( || "unknown".to_string() );
  let pref     = read_preferred_version();

  let content = match ( opts.format, opts.verbosity )
  {
    ( OutputFormat::Json, _ ) =>
    {
      let v = json_escape( &version );
      let a = json_escape( &account );
      match &pref
      {
        Some( ( spec, resolved ) ) =>
        {
          let ps = json_escape( spec );
          let pr = resolved.as_deref().map_or( "null".to_string(), | r | format!( "\"{}\"", json_escape( r ) ) );
          format!( "{{\"version\":\"{v}\",\"processes\":{processes},\"account\":\"{a}\",\"preferred\":{{\"spec\":\"{ps}\",\"resolved\":{pr}}}}}\n" )
        }
        None => format!( "{{\"version\":\"{v}\",\"processes\":{processes},\"account\":\"{a}\"}}\n" ),
      }
    }
    ( OutputFormat::Text, 0 ) =>
    {
      match &pref
      {
        Some( ( spec, _ ) ) => format!( "{version}\n{processes}\n{account}\n{spec}\n" ),
        None => format!( "{version}\n{processes}\n{account}\n" ),
      }
    }
    ( OutputFormat::Text, v ) =>
    {
      // "Processes:" is 10 chars; pad shorter labels to align values at column 12.
      let base = format!( "Version:   {version}\nProcesses: {processes}\nAccount:   {account}" );
      match &pref
      {
        Some( ( spec, resolved ) ) =>
        {
          let pref_str = match resolved
          {
            Some( r ) => format!( "{spec} (v{r})" ),
            None => spec.clone(),
          };
          if v >= 2
          {
            format!( "{base}\nPreferred: {pref_str}  (settings.json \u{2192} preferredVersionSpec)\n" )
          }
          else
          {
            format!( "{base}\nPreferred: {pref_str}\n" )
          }
        }
        None => format!( "{base}\n" ),
      }
    }
  };

  Ok( OutputData::new( content, "text" ) )
}

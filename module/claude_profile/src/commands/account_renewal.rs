//! `.account.renewal` command handler.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use super::shared::{ require_nonempty_string_arg, is_dry, require_credential_store, io_err_to_error_data, resolve_account_name };
use claude_profile_core::account::trace_ts;

/// `.account.renewal` — set or clear a billing renewal timestamp override for one or more accounts.
///
/// Writes `_renewal_at` (ISO-8601 UTC string) to `{name}.json` via read-merge, or removes
/// it when `clear::1` is passed. Existing `oauthAccount` and other top-level keys are preserved.
///
/// # Errors
///
/// - Exit 1: conflicting params, no operation param provided, `from_now::` parse error.
/// - Exit 2 (via `process::exit`): named account not found.
#[ inline ]
#[ allow( clippy::too_many_lines ) ]
pub fn account_renewal_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let dry      = is_dry( &cmd );
  let trace    = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let raw_name = require_nonempty_string_arg( &cmd, "name" )?;

  let at_val = match cmd.arguments.get( "at" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    _ => None,
  };
  let from_now_val = match cmd.arguments.get( "from_now" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    _ => None,
  };
  let clear_val = crate::output::parse_int_flag( &cmd, "clear", 0 )? != 0;

  // Validate mutual exclusion.
  if at_val.is_some() && from_now_val.is_some()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      "at:: and from_now:: are mutually exclusive — provide only one".to_string(),
    ) );
  }
  if at_val.is_some() && clear_val
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      "at:: and clear:: are mutually exclusive — provide only one".to_string(),
    ) );
  }
  if from_now_val.is_some() && clear_val
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      "from_now:: and clear:: are mutually exclusive — provide only one".to_string(),
    ) );
  }
  if at_val.is_none() && from_now_val.is_none() && !clear_val
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      "must provide one of: at::, from_now::, clear::".to_string(),
    ) );
  }

  // Build the renewal operation.
  let op = if let Some( ts ) = at_val
  {
    crate::account::RenewalOperation::At( ts )
  }
  else if let Some( delta_str ) = from_now_val
  {
    let delta_secs = crate::account::parse_from_now_delta( &delta_str )
      .map_err( |e| ErrorData::new( ErrorCode::ArgumentTypeMismatch, e ) )?;
    use std::time::{ SystemTime, UNIX_EPOCH };
    let now_secs = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .unwrap_or_default()
      .as_secs();
    let target_secs = if delta_secs >= 0
    {
      now_secs.saturating_add( u64::try_from( delta_secs ).unwrap_or( 0 ) )
    }
    else
    {
      now_secs.saturating_sub( u64::try_from( -delta_secs ).unwrap_or( 0 ) )
    };
    crate::account::RenewalOperation::At( crate::account::secs_to_iso8601( target_secs ) )
  }
  else
  {
    crate::account::RenewalOperation::Clear
  };

  let credential_store = require_credential_store()?;
  if trace { eprintln!( "{}account.renewal  store: {}", trace_ts(), credential_store.display() ) }

  // Resolve target account name(s).
  let names : Vec< String > = if raw_name == "all"
  {
    crate::account::list( &credential_store )
      .map_err( |e| io_err_to_error_data( &e, "account renewal" ) )?
      .into_iter()
      .map( |a| a.name )
      .collect()
  }
  else if raw_name.contains( ',' )
  {
    raw_name.split( ',' )
      .map( | s | s.trim().to_string() )
      .filter( | s | !s.is_empty() )
      .collect()
  }
  else
  {
    vec![ resolve_account_name( &raw_name, &credential_store )? ]
  };

  if trace { eprintln!( "{}account.renewal  targets: {names:?}", trace_ts() ) }

  let mut output     = String::new();
  let mut had_error  = false;
  let mut error_code = 0_i32;

  // Fix(BUG-267): comma-list tokens were used as raw strings without
  //   prefix resolution, causing `name::i9,i11` to fail when i9@host, i11@host are saved.
  // Root cause: the comma-list branch collected raw tokens; only the single-name branch
  //   called resolve_account_name(). Full emails pass through the @-fast-path unchanged.
  // Pitfall: resolve_account_name() returns ErrorData (not io::Error); match on ErrorCode
  //   to get the correct exit code (ArgumentTypeMismatch=ambiguous→1, InternalError=not found→2).
  for raw in &names
  {
    let name = match resolve_account_name( raw, &credential_store )
    {
      Ok( n )  => n,
      Err( e ) =>
      {
        had_error  = true;
        error_code = if e.code == ErrorCode::ArgumentTypeMismatch { 1 } else { 2 };
        eprintln!( "account renewal error: {raw}: {e}" );
        continue;
      }
    };
    match crate::account::account_renewal( &name, &credential_store, &op, dry )
    {
      Ok( line ) => output.push_str( &line ),
      Err( e )   =>
      {
        had_error  = true;
        error_code = if e.kind() == std::io::ErrorKind::NotFound { 2 } else { 1 };
        eprintln!( "account renewal error: {name}: {e}" );
      }
    }
  }

  if had_error
  {
    if !output.is_empty() { print!( "{output}" ); }
    std::process::exit( error_code );
  }

  Ok( OutputData::new( output, "text" ) )
}

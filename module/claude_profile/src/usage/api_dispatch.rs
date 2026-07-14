//! Mutation-dispatch helper for the `.usage` command.
//!
//! Extracted from `api.rs` to keep `usage_routine` under the 500-line ceiling.
//! All mutation-only params (`assignee::`, `owner::`, and their `REMOVED_TOGGLE`
//! predecessors) are handled here; the caller falls through on `Ok(None)`.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use crate::commands::cmd_args::{ is_dry, resolve_account_name, io_err_to_error_data };
use claude_profile_core::account::trace_ts;

// ── Private helpers ────────────────────────────────────────────────────────────

/// Inner body of the `assignee::` dispatch — sentinel expansion done by caller.
///
/// `av` is the sanitised, sentinel-expanded `USER@MACHINE` string.
fn dispatch_assignee_param(
  av               : String,
  cmd              : &VerifiedCommand,
  trace            : bool,
  credential_store : &std::path::Path,
) -> Result< Option< OutputData >, ErrorData >
{
  let san = | s : &str | -> String
  {
    s.chars()
      .map( | c | if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' } )
      .collect()
  };
  let ( usr_raw, mch_raw ) = av.split_once( '@' ).ok_or_else( || ErrorData::new(
    ErrorCode::ArgumentTypeMismatch,
    "assignee:: must be USER@MACHINE format (no '@' found) — or use assignee::0 for current machine".to_string(),
  ) )?;
  if usr_raw.is_empty()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "assignee:: user component (left of '@') must not be empty".to_string(),
    ) );
  }
  if mch_raw.is_empty()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "assignee:: machine component (right of '@') must not be empty".to_string(),
    ) );
  }
  let su      = san( usr_raw );
  let sm      = san( mch_raw );
  let marker  = format!( "_active_{sm}_{su}" );
  let display = format!( "{su}@{sm}" );

  let raw_name = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _ => String::new(),
  };
  let name_arg = if raw_name.is_empty()
  {
    raw_name.clone()
  }
  else
  {
    resolve_account_name( &raw_name, credential_store )?
  };

  if !name_arg.is_empty()
  {
    // Assign: write marker pointing to name_arg.
    let cred_path = credential_store.join( format!( "{name_arg}.credentials.json" ) );
    if !cred_path.exists()
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "account '{name_arg}' not found in credential store" ),
      ) );
    }
    // G9: Claim-lock guard — locked accounts cannot become the assignee:: target.
    // force::1 bypasses the guard (Feature 070 AC-04).
    let force = crate::output::parse_int_flag( cmd, "force", 0 )? != 0;
    if !force && claude_profile_core::account::read_claim_lock( credential_store, &name_arg )
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "claim-lock violation: {name_arg} is claim-locked" ),
      ) );
    }
    if is_dry( cmd )
    {
      return Ok( Some( OutputData::new(
        format!( "[dry-run] would assign {name_arg} for {display}  \u{2192}  {marker}\n" ),
        "text",
      ) ) );
    }
    std::fs::write( credential_store.join( &marker ), name_arg.as_bytes() )
      .map_err( | e | io_err_to_error_data( &e, "usage assignee" ) )?;
    if trace { eprintln!( "{}usage assignee  write marker: {marker}  →  {name_arg}", trace_ts() ) }
    return Ok( Some( OutputData::new(
      format!( "assigned {name_arg} for {display}  \u{2192}  {marker}\n" ),
      "text",
    ) ) );
  }
  // Unassign: clear the marker file.
  if is_dry( cmd )
  {
    return Ok( Some( OutputData::new(
      format!( "[dry-run] would unassign {display}  \u{2192}  {marker} cleared\n" ),
      "text",
    ) ) );
  }
  let marker_path = credential_store.join( &marker );
  if marker_path.exists()
  {
    std::fs::remove_file( &marker_path )
      .map_err( | e | io_err_to_error_data( &e, "usage assignee unassign" ) )?;
  }
  if trace { eprintln!( "{}usage assignee  cleared marker: {marker}", trace_ts() ) }
  Ok( Some( OutputData::new(
    format!( "unassigned {display}  \u{2192}  {marker} cleared\n" ),
    "text",
  ) ) )
}

// ── Public interface ───────────────────────────────────────────────────────────

/// Handle all mutation-only params in `.usage`.
///
/// Returns:
/// - `Ok( Some( output ) )` — mutation completed; caller must return immediately.
/// - `Ok( None )` — no mutation param present; caller continues normally.
/// - `Err( e )` — validation or I/O error; caller propagates.
pub( crate ) fn handle_mutation_dispatch(
  cmd              : &VerifiedCommand,
  trace            : bool,
  force            : bool,
  credential_store : &std::path::Path,
) -> Result< Option< OutputData >, ErrorData >
{
  // REMOVED_TOGGLE checks: assign, unclaim, for, active → migration messages (Feature 064/065).
  if cmd.arguments.contains_key( "assign" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "assign:: REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)".to_string(),
    ) );
  }
  if cmd.arguments.contains_key( "unclaim" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "unclaim:: REMOVED — use owner::0 name::X instead (or owner::0 alone to batch-clear)".to_string(),
    ) );
  }
  if cmd.arguments.contains_key( "for" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "for:: REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)".to_string(),
    ) );
  }
  if cmd.arguments.contains_key( "active" )
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "active:: REMOVED — use assignee::USER@MACHINE name::X (or assignee::0 name::X for current machine)".to_string(),
    ) );
  }

  if cmd.arguments.contains_key( "assignee" )
  {
    return dispatch_assignee( cmd, trace, credential_store );
  }

  // owner:: param — explicit ownership assignment/release (Feature 063 + 064).
  let owner_value = match cmd.arguments.get( "owner" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    Some( Value::String( _ ) ) =>
      return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch,
        "owner:: value must be non-empty — use owner::0 to clear ownership".into() ) ),
    _ => None,
  };

  if let Some( ref ov ) = owner_value
  {
    return dispatch_owner( cmd, trace, force, credential_store, ov );
  }

  // lock:: / reserve:: — ungated boolean field dispatch (Feature 070 AC-02).
  for ( param_name, writer ) in
  [
    ( "lock",    claude_profile_core::account::write_claim_lock as fn( &str, &std::path::Path, bool ) -> Result< (), std::io::Error > ),
    ( "reserve", claude_profile_core::account::write_reserve    as fn( &str, &std::path::Path, bool ) -> Result< (), std::io::Error > ),
  ]
  {
    if let Some( Value::String( v ) ) = cmd.arguments.get( param_name )
    {
      return dispatch_bool_field( cmd, trace, credential_store, param_name, v == "1", writer ).map( Some );
    }
  }

  Ok( None )
}

// ── assignee:: dispatch (Feature 065) ────────────────────────────────────────

#[ allow( clippy::too_many_lines ) ]
fn dispatch_assignee(
  cmd              : &VerifiedCommand,
  trace            : bool,
  credential_store : &std::path::Path,
) -> Result< Option< OutputData >, ErrorData >
{
  if let Some( Value::String( av ) ) = cmd.arguments.get( "assignee" )
  {
    // Sentinel "0" expands to current machine identity ($USER@$HOSTNAME).
    let av = if av == "0" { claude_profile_core::account::current_identity() } else { av.clone() };
    return dispatch_assignee_param( av, cmd, trace, credential_store );
  }

  Ok( None )
}

// ── owner:: dispatch (Feature 063 + 064) ──────────────────────────────────────

fn dispatch_owner(
  cmd              : &VerifiedCommand,
  trace            : bool,
  force            : bool,
  credential_store : &std::path::Path,
  ov               : &str,
) -> Result< Option< OutputData >, ErrorData >
{
  let is_sentinel = ov == "0";
  let is_dry_run  = is_dry( cmd );

  let raw_name = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _ => String::new(),
  };
  let name_arg = if raw_name.is_empty() || raw_name.contains( ',' )
  {
    // Comma-list — defer per-component resolution to dispatch below.
    raw_name.clone()
  }
  else
  {
    resolve_account_name( &raw_name, credential_store )?
  };

  if raw_name.is_empty()
  {
    if !is_sentinel
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "owner::USER@MACHINE requires name:: to specify the target account".to_string(),
      ) );
    }
    let all_accounts = crate::account::list( credential_store )
      .map_err( |e| ErrorData::new(
        ErrorCode::InternalError,
        format!( "cannot read credential store: {e}" ),
      ) )?;
    return crate::owner_dispatch::owner_batch_clear( trace, force, is_dry_run, &all_accounts, credential_store, "usage" ).map( Some );
  }

  crate::owner_dispatch::owner_named_dispatch( trace, force, is_dry_run, is_sentinel, ov, &raw_name, &name_arg, credential_store, "usage" ).map( Some )
}

// ── lock:: / reserve:: dispatch (Feature 070) ─────────────────────────────────

fn dispatch_bool_field(
  cmd              : &VerifiedCommand,
  trace            : bool,
  credential_store : &std::path::Path,
  field_name       : &str,
  value            : bool,
  writer           : fn( &str, &std::path::Path, bool ) -> Result< (), std::io::Error >,
) -> Result< OutputData, ErrorData >
{
  let is_dry_run = is_dry( cmd );

  let raw_name = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _ => String::new(),
  };
  let name_arg = if raw_name.is_empty() || raw_name.contains( ',' )
  {
    raw_name.clone()
  }
  else
  {
    resolve_account_name( &raw_name, credential_store )?
  };

  if raw_name.is_empty()
  {
    let all_accounts = crate::account::list( credential_store )
      .map_err( |e| ErrorData::new(
        ErrorCode::InternalError,
        format!( "cannot read credential store: {e}" ),
      ) )?;
    return crate::owner_dispatch::bool_field_batch_set( trace, is_dry_run, value, &all_accounts, credential_store, "usage", field_name, writer );
  }

  crate::owner_dispatch::bool_field_named_dispatch( trace, is_dry_run, value, &raw_name, &name_arg, credential_store, "usage", field_name, writer )
}

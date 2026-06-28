//! Mutation-dispatch helper for the `.usage` command.
//!
//! Extracted from `api.rs` to keep `usage_routine` under the 500-line ceiling.
//! All mutation-only params (`assignee::`, `owner::`, and their REMOVED_TOGGLE
//! predecessors) are handled here; the caller falls through on `Ok(None)`.

use core::fmt::Write as _;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use crate::commands::shared::{ is_dry, resolve_account_name, io_err_to_error_data };
use claude_profile_core::account::trace_ts;

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

  // ── assignee:: dispatch (Feature 065) ────────────────────────────────────
  if let Some( Value::String( av ) ) = cmd.arguments.get( "assignee" )
  {
    let av = if av == "0"
    {
      // Sentinel "0" expands to current machine identity ($USER@$HOSTNAME).
      claude_profile_core::account::current_identity()
    }
    else
    {
      av.clone()
    };
    let san = | s : &str | -> String
    {
      s.chars().map( | c | if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' } ).collect()
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
    return Ok( Some( OutputData::new(
      format!( "unassigned {display}  \u{2192}  {marker} cleared\n" ),
      "text",
    ) ) );
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

  // ── owner:: explicit ownership assignment/release (Feature 063 + 064) ──────
  if let Some( ref ov ) = owner_value
  {
    let is_sentinel = ov.as_str() == "0";
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
      // No name:: → batch-clear (owner::0 only; owner::VALUE requires name::).
      if !is_sentinel
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          "owner::USER@MACHINE requires name:: to specify the target account".to_string(),
        ) );
      }
      // Batch-clear all accounts currently owned by this identity.
      // Unowned and foreign-owned accounts are skipped with a "skip" message (AC-09).
      let all_accounts = crate::account::list( credential_store )
        .map_err( |e| ErrorData::new(
          ErrorCode::InternalError,
          format!( "cannot read credential store: {e}" ),
        ) )?;
      let mut out = String::new();
      for acct in &all_accounts
      {
        let json_path = credential_store.join( format!( "{}.json", acct.name ) );
        // No metadata file → silently skip (no ownership info to act on).
        if !json_path.exists() { continue; }
        let acct_owner = crate::account::read_owner( credential_store, &acct.name );
        if acct_owner.is_empty()
        {
          // Unowned — nothing to clear; skip with message (AC-09).
          writeln!( out, "skip {}", acct.name ).unwrap();
          continue;
        }
        if !force && !crate::account::is_owned( &acct_owner )
        {
          // Owned by another identity — skip with message (AC-09).
          if trace { eprintln!( "{}usage owner  batch-skip (foreign owner): {}  owner={acct_owner}", trace_ts(), acct.name ) }
          writeln!( out, "skip {}", acct.name ).unwrap();
          continue;
        }
        if is_dry_run
        {
          writeln!( out, "[dry-run] would clear owner of {}", acct.name ).unwrap();
          continue;
        }
        crate::account::write_owner( &acct.name, credential_store, "" )
          .map_err( |e| io_err_to_error_data( &e, "usage owner batch-clear" ) )?;
        if trace { eprintln!( "{}usage owner  cleared: {}  was={acct_owner}", trace_ts(), acct.name ) }
        writeln!( out, "unclaimed {}", acct.name ).unwrap();
      }
      return Ok( Some( OutputData::new( out, "text" ) ) );
    }

    // name:: present — resolve each component (comma-list supported for owner:: ops).
    let target_names : Vec< String > = if raw_name.contains( ',' )
    {
      raw_name.split( ',' )
        .map( | part | resolve_account_name( part.trim(), credential_store ) )
        .collect::< Result< Vec< _ >, _ > >()?
    }
    else
    {
      vec![ name_arg ]
    };

    let mut out = String::new();
    for name in &target_names
    {
      let json_path = credential_store.join( format!( "{name}.json" ) );
      if !json_path.exists()
      {
        return Err( ErrorData::new(
          ErrorCode::InternalError,
          format!( "account not found: {name}" ),
        ) );
      }
      // G8 ownership gate — evaluated per account, even in dry-run (AC-16/AC-17).
      let acct_owner = crate::account::read_owner( credential_store, name );
      if !force && !crate::account::is_owned( &acct_owner )
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          format!( "ownership violation: {name} is owned by {acct_owner}" ),
        ) );
      }
      if is_dry_run
      {
        if is_sentinel
        {
          writeln!( out, "[dry-run] would clear owner of {name}" ).unwrap();
        }
        else
        {
          writeln!( out, "[dry-run] would set owner of {name} to {ov}" ).unwrap();
        }
        continue;
      }
      let new_owner = if is_sentinel { "" } else { ov.as_str() };
      crate::account::write_owner( name, credential_store, new_owner )
        .map_err( |e| io_err_to_error_data( &e, "usage owner" ) )?;
      if trace
      {
        eprintln!( "{}usage owner  write_owner: OK  name={name} identity={}", trace_ts(), if is_sentinel { "(cleared)" } else { ov } );
      }
      if is_sentinel
      {
        writeln!( out, "unclaimed {name}" ).unwrap();
      }
      else
      {
        writeln!( out, "owned {name} by {ov}" ).unwrap();
      }
    }
    return Ok( Some( OutputData::new( out, "text" ) ) );
  }

  Ok( None )
}

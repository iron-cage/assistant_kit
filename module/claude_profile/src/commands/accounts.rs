//! `.accounts` command handler.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use crate::output::{ OutputFormat, OutputOptions };
use super::cmd_args::{ io_err_to_error_data, resolve_account_name };
use super::cmd_context::require_credential_store;
use super::accounts_render::{ IdentityCols, render_accounts_text, render_accounts_json, render_accounts_table };
use claude_profile_core::account::trace_ts;

// ── Single-consumer helpers ───────────────────────────────────────────────────

/// Detect which saved account matches the live session token.
///
/// Reads `accessToken` from `live_creds_path` (graceful degradation: returns `None`
/// on any I/O or parse error). Compares by string equality against each saved account's
/// stored `accessToken` in `credential_store`; returns `Some(name)` on the first match,
/// `None` if no match.
fn detect_current_account(
  accounts         : &[ crate::account::Account ],
  live_creds_path  : &std::path::Path,
  credential_store : &std::path::Path,
) -> Option< String >
{
  let content    = std::fs::read_to_string( live_creds_path ).ok()?;
  let live_token = crate::account::parse_string_field( &content, "accessToken" )?;
  for acct in accounts
  {
    let path    = credential_store.join( format!( "{}.credentials.json", acct.name ) );
    let Ok( s ) = std::fs::read_to_string( &path ) else { continue };
    if let Some( token ) = crate::account::parse_string_field( &s, "accessToken" )
    {
      if token == live_token
      {
        return Some( acct.name.clone() );
      }
    }
  }
  None
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.accounts` — list all saved accounts with field-presence control.
///
/// Without `name::`: lists every account in the credential store as an indented
/// key-value block, with a blank line between accounts when any field is shown.
/// With `name::EMAIL`: shows that single account's block only.
/// Field-presence params (`active`, `sub`, `tier`, `expires`, `email`) are all default-on.
/// `format::json` always includes all fields regardless of presence params.
///
/// # Errors
///
/// Returns `ErrorData` if `name::` is invalid (exit 1),
/// the named account is not found (exit 2), or the credential store is unreadable.
///
/// Storage unavailable (HOME/PRO unset) returns advisory "(no accounts configured)"
/// with exit 0 — same graceful behavior as an empty credential store.
// Fix(BUG-268):
// Root cause: require_credential_store()?; propagated Err (exit 2) when HOME and PRO are
//   both unset. .accounts is a graceful-read command; storage unavailability means the same
//   thing as an empty store — show advisory, not an error.
// Pitfall: require_credential_store() failing is NOT the same as list() returning [] —
//   they are different code paths. The graceful fallback must be at require_credential_store()
//   level, not at list() level.
#[ inline ]
#[ allow( clippy::too_many_lines ) ]
pub fn accounts_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts             = OutputOptions::from_cmd( &cmd )?;
  let trace            = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let Ok( credential_store ) = require_credential_store() else
  {
    if trace { eprintln!( "{}accounts  credential store: not found", trace_ts() ) }
    let content = match opts.format
    {
      OutputFormat::Json  => "[]\n".to_string(),
      OutputFormat::Text
      | OutputFormat::Table => "(no accounts configured)\n".to_string(),
    };
    return Ok( OutputData::new( content, "text" ) );
  };
  if trace { eprintln!( "{}accounts  reading store: {}", trace_ts(), credential_store.display() ) }

  let raw_name = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => String::new(),
  };
  let name_arg = if raw_name.is_empty()
  {
    raw_name.clone()
  }
  else if raw_name.contains( ',' )
  {
    // Comma-list for batch owner:: ops — defer per-component resolution to dispatch.
    raw_name.clone()
  }
  else
  {
    resolve_account_name( &raw_name, &credential_store )?
  };

  let all_accounts = crate::account::list( &credential_store )
    .map_err( |e| io_err_to_error_data( &e, "accounts" ) )?;

  let accounts : Vec< _ > = if name_arg.is_empty() || name_arg.contains( ',' ) || cmd.arguments.contains_key( "assignee" )
  {
    // Comma-list and assignee:: dispatch handle their own account filtering/validation.
    all_accounts.iter().collect()
  }
  else
  {
    let found : Vec< _ > = all_accounts.iter().filter( |a| a.name == name_arg ).collect();
    if found.is_empty()
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "account '{name_arg}' not found" ),
      ) );
    }
    found
  };

  // ── Mutation dispatch ──────────────────────────────────────────────────────
  use super::cmd_args::is_dry;

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

  // ── assignee:: dispatch (Feature 065) ──────────────────────────────────────
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
      let force = crate::output::parse_int_flag( &cmd, "force", 0 )? != 0;
      if !force && crate::account::read_claim_lock( &credential_store, &name_arg )
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          format!( "claim-lock violation: {name_arg} is claim-locked" ),
        ) );
      }
      if is_dry( &cmd )
      {
        return Ok( OutputData::new(
          format!( "[dry-run] would assign {name_arg} for {display}  \u{2192}  {marker}\n" ),
          "text",
        ) );
      }
      std::fs::write( credential_store.join( &marker ), name_arg.as_bytes() )
        .map_err( | e | io_err_to_error_data( &e, "accounts assignee" ) )?;
      if trace { eprintln!( "{}accounts assignee  write marker: {marker}  →  {name_arg}", trace_ts() ) }
      return Ok( OutputData::new(
        format!( "assigned {name_arg} for {display}  \u{2192}  {marker}\n" ),
        "text",
      ) );
    }
    // Unassign: clear the marker file.
    if is_dry( &cmd )
    {
      return Ok( OutputData::new(
        format!( "[dry-run] would unassign {display}  \u{2192}  {marker} cleared\n" ),
        "text",
      ) );
    }
    let marker_path = credential_store.join( &marker );
    if marker_path.exists()
    {
      std::fs::remove_file( &marker_path )
        .map_err( | e | io_err_to_error_data( &e, "accounts assignee unassign" ) )?;
    }
    if trace { eprintln!( "{}accounts assignee  cleared marker: {marker}", trace_ts() ) }
    return Ok( OutputData::new(
      format!( "unassigned {display}  \u{2192}  {marker} cleared\n" ),
      "text",
    ) );
  }

  // owner:: param — explicit ownership assignment (Feature 063 + 064).
  let owner_value = match cmd.arguments.get( "owner" )
  {
    Some( unilang::types::Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    Some( unilang::types::Value::String( _ ) ) =>
      return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch,
        "owner:: value must be non-empty — use owner::0 to clear ownership".into() ) ),
    _ => None,
  };

  // ── owner:: explicit ownership assignment/release (Feature 063 + 064) ────────
  if let Some( ref ov ) = owner_value
  {
    let is_sentinel = ov.as_str() == "0";
    let force       = crate::output::parse_int_flag( &cmd, "force", 0 )? != 0;
    let is_dry_run  = is_dry( &cmd );

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
      return crate::owner_dispatch::owner_batch_clear( trace, force, is_dry_run, &all_accounts, &credential_store, "accounts" );
    }

    return crate::owner_dispatch::owner_named_dispatch( trace, force, is_dry_run, is_sentinel, ov, &raw_name, &name_arg, &credential_store, "accounts" );
  }

  // ── lock:: / reserve:: boolean field dispatch (Feature 070) ───────────────────
  // Ungated (AC-02) — unlike owner::, both batch directions are valid (Test Matrix T04).
  for ( param_name, writer ) in
  [
    ( "lock",    crate::account::write_claim_lock as fn( &str, &std::path::Path, bool ) -> Result< (), std::io::Error > ),
    ( "reserve", crate::account::write_reserve    as fn( &str, &std::path::Path, bool ) -> Result< (), std::io::Error > ),
  ]
  {
    if let Some( Value::String( v ) ) = cmd.arguments.get( param_name )
    {
      let value      = v == "1";
      let is_dry_run = is_dry( &cmd );
      if raw_name.is_empty()
      {
        return crate::owner_dispatch::bool_field_batch_set( trace, is_dry_run, value, &all_accounts, &credential_store, "accounts", param_name, writer );
      }
      return crate::owner_dispatch::bool_field_named_dispatch( trace, is_dry_run, value, &raw_name, &name_arg, &credential_store, "accounts", param_name, writer );
    }
  }

  // ── Legacy field-toggle rejection (Feature 037 — removed; use cols:: instead) ─
  // Params remain registered so the framework routes to this routine; the routine
  // rejects them explicitly to provide a helpful migration message.
  const REMOVED_TOGGLES : &[ ( &str, &str ) ] = &[
    ( "current",      "-current" ),
    ( "sub",          "-sub" ),
    ( "tier",         "-tier" ),
    ( "expires",      "-expires" ),
    ( "email",        "-email" ),
    ( "display_name", "+display_name" ),
    ( "host",         "+host" ),
    ( "role",         "+role" ),
    ( "billing",      "+billing" ),
    ( "model",        "+model" ),
    ( "uuid",         "+uuid" ),
    ( "capabilities", "+capabilities" ),
    ( "org_uuid",     "+org_uuid" ),
    ( "org_name",     "+org_name" ),
  ];
  for ( param, suggestion ) in REMOVED_TOGGLES
  {
    if cmd.arguments.contains_key( *param )
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "parameter '{param}' removed — use 'cols::{suggestion}' instead" ),
      ) );
    }
  }

  // ── Parse cols:: modifier string into IdentityCols ────────────────────────────
  let cols_raw = match cmd.arguments.get( "cols" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => s.clone(),
    _ => String::new(),
  };
  let cols = if cols_raw.is_empty()
  {
    IdentityCols::default_set()
  }
  else
  {
    IdentityCols::parse( &cols_raw )?
  };

  // Compute owner strings for display (skipped when cols.owner is false to avoid I/O).
  let owners : Vec< String > = if cols.owner
  {
    accounts.iter().map( |a| crate::account::read_owner( &credential_store, &a.name ) ).collect()
  }
  else
  {
    Vec::new()
  };

  // Detect which account matches the live session token (graceful: None when creds absent).
  let live_creds = crate::ClaudePaths::new()
    .map_or_else( || std::path::PathBuf::from( "/dev/null" ), |p| p.credentials_file() );
  let current_name = detect_current_account( &all_accounts, &live_creds, &credential_store );

  let content = match opts.format
  {
    OutputFormat::Json => render_accounts_json( &accounts, current_name.as_deref() ),
    OutputFormat::Text =>
    {
      render_accounts_text( &accounts, &owners, &cols, current_name.as_deref() )
    }
    OutputFormat::Table =>
    {
      render_accounts_table( &accounts, &owners, &cols, current_name.as_deref() )
    }
  };
  Ok( OutputData::new( content, "text" ) )
}

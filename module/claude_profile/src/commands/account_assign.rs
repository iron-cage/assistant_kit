//! `.account.assign` command handler.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use super::shared::{ is_dry, require_credential_store, io_err_to_error_data, resolve_account_name };

/// Build the live usage block emitted when `.account.assign` is called without `name::`.
///
/// Extracted from `account_assign_routine` to keep that function under clippy's 100-line limit.
/// `user` and `machine` must be the *sanitized* values (same char-filter as the marker filename),
/// not the raw env values — otherwise the displayed `for::` target would diverge from the written
/// filename. When `active` is `"(none)"` the `Ready to copy:` section is omitted; including
/// `(none)` as the account name in copy-paste examples would produce an invalid command.
fn account_assign_usage_block( user : &str, machine : &str, marker : &str, active : &str ) -> String
{
  let ready_section = if active == "(none)"
  {
    String::new()
  }
  else
  {
    format!(
      "\nReady to copy:\n  clp .account.assign name::{active}\n  clp .account.assign name::{active} for::{user}@{machine}\n  clp .account.assign name::{active} for::otheruser@othermachine dry::1\n"
    )
  };
  format!(
    ".account.assign \u{2014} write the active-account marker for any machine without credential rotation.\n\n  name::   account to assign (required)\n  for::    USER@MACHINE to target  (default: current machine)\n  dry::1   preview without writing\n\nCurrent machine:  {user}@{machine}  (\u{2192} {marker})\nActive account:   {active}\n{ready_section}"
  )
}

/// `.account.assign` — write the per-machine active-account marker for any host+user pair without credential rotation.
///
/// Marker-only write: no `~/.claude.*` side effects. When `name::` is absent, emits a live
/// usage block showing the current machine identity and active account, then exits 0.
///
/// # Errors
///
/// - Exit 1: `for::` value missing `@`, or either component is empty after split.
/// - Exit 2: account not found.
#[ inline ]
#[ allow( clippy::too_many_lines ) ]
pub fn account_assign_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let trace    = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;

  // When name:: is absent, emit a live usage block showing current machine identity.
  let raw_name = match cmd.arguments.get( "name" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => s.clone(),
    _ =>
    {
      let user    = std::env::var( "USER" )
        .or_else( |_| std::env::var( "USERNAME" ) )
        .unwrap_or_else( |_| "user".to_string() );
      let machine = crate::account::resolve_hostname();
      let san     = | s : &str | -> String
      {
        s.chars().map( | c | if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' } ).collect()
      };
      let marker  = format!( "_active_{}_{}", san( &machine ), san( &user ) );
      let active  = require_credential_store()
        .ok()
        .and_then( | store | std::fs::read_to_string( store.join( &marker ) ).ok() )
        .map( | s | s.trim().to_string() )
        .filter( | s | !s.is_empty() )
        .unwrap_or_else( || "(none)".to_string() );
      return Ok( OutputData::new( account_assign_usage_block( &user, &machine, &marker, &active ), "text" ) );
    }
  };

  let credential_store = require_credential_store()?;
  if trace { eprintln!( "[trace] account.assign  store: {}", credential_store.display() ) }
  let name             = resolve_account_name( &raw_name, &credential_store )?;

  // Fix(BUG-247): resolve_account_name passes @-containing names through without checking
  //   the credential store — it short-circuits on '@' to avoid treating email addresses as
  //   bare prefixes, but this skips the existence lookup entirely for full email names.
  // Root cause: the '@' fast-path in resolve_account_name trades store validation for
  //   unambiguous input handling; callers accepting email-format names must add their own guard.
  // Pitfall: bare-prefix resolution already validates existence (list() → 0 matches → exit 2),
  //   so only the @-path was missing the check. The fix mirrors what list-based resolution does.
  let cred_path = credential_store.join( format!( "{name}.credentials.json" ) );
  if !cred_path.exists()
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "account '{name}' not found in credential store" ),
    ) );
  }

  // Derive marker filename and display target from for:: (split on first '@') or current machine identity.
  let ( marker, for_display ) = match cmd.arguments.get( "for" )
  {
    Some( Value::String( s ) ) if !s.is_empty() =>
    {
      let ( usr, mch ) = s.split_once( '@' ).ok_or_else( || ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        "for:: must be USER@MACHINE format (no '@' found)".to_string(),
      ) )?;
      if usr.is_empty()
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          "for:: user component (left of '@') must not be empty".to_string(),
        ) );
      }
      if mch.is_empty()
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          "for:: machine component (right of '@') must not be empty".to_string(),
        ) );
      }
      let san = | s : &str | -> String
      {
        s.chars().map( | c | if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' } ).collect()
      };
      let su = san( usr );
      let sm = san( mch );
      ( format!( "_active_{sm}_{su}" ), format!( "{su}@{sm}" ) )
    }
    _ =>
    {
      let user    = std::env::var( "USER" )
        .or_else( |_| std::env::var( "USERNAME" ) )
        .unwrap_or_else( |_| "user".to_string() );
      let machine = crate::account::resolve_hostname();
      let san     = | s : &str | -> String
      {
        s.chars().map( | c | if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '_' } ).collect()
      };
      let su = san( &user );
      let sm = san( &machine );
      ( format!( "_active_{sm}_{su}" ), format!( "{su}@{sm}" ) )
    }
  };
  if trace { eprintln!( "[trace] account.assign  marker: {marker}" ) }

  if is_dry( &cmd )
  {
    return Ok( OutputData::new(
      format!( "[dry-run] would assign {name} for {for_display}  →  {marker}\n" ),
      "text",
    ) );
  }

  std::fs::write( credential_store.join( &marker ), name.as_bytes() )
    .map_err( | e | io_err_to_error_data( &e, "account assign" ) )?;

  Ok( OutputData::new( format!( "Assigned {name} for {for_display}  →  {marker}\n" ), "text" ) )
}

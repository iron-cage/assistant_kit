//! `.account.tag` — add, remove, or replace tags on one or more saved accounts.
//!
//! Feature 075: exactly one of `add::`/`remove::`/`tags::` per invocation;
//! batch `name::a,b` resolves and existence-checks every component before any
//! write (all-resolve-before-any-mutate); ungated (no ownership checks — tags
//! are shared pool metadata, same trust model as `lock::`/`reserve::`).

use core::fmt::Write as _;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use super::cmd_args::{ require_nonempty_string_arg, is_dry, io_err_to_error_data, resolve_account_name };
use super::cmd_context::require_credential_store;
use claude_profile_core::account::trace_ts;

/// Render a stored tag set for display: `"(no tags)"` when empty, else `", "`-joined.
fn render_set( tags : &[ String ] ) -> String
{
  if tags.is_empty() { "(no tags)".to_string() } else { tags.join( ", " ) }
}

/// `.account.tag` — mutate the stored tag set of one or more accounts.
///
/// # Errors
///
/// Returns `ErrorData` if `name::` is missing/empty or resolution is ambiguous
/// (exit 1), not exactly one of `add::`/`remove::`/`tags::` is given (exit 1),
/// an operand tag is invalid (exit 1), or any named account does not exist
/// (exit 2 — checked for every component before any write).
#[ inline ]
pub fn account_tag_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let trace    = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let raw_name = require_nonempty_string_arg( &cmd, "name" )?;

  // Exactly one operation param. Empty-string values still count as "given" —
  // they fail tag validation loudly below instead of silently no-oping.
  let opt_str = | key : &str | -> Option< String >
  {
    match cmd.arguments.get( key )
    {
      Some( Value::String( s ) ) => Some( s.clone() ),
      _                          => None,
    }
  };
  let add_raw    = opt_str( "add" );
  let remove_raw = opt_str( "remove" );
  let tags_raw   = opt_str( "tags" );
  let given = usize::from( add_raw.is_some() )
    + usize::from( remove_raw.is_some() )
    + usize::from( tags_raw.is_some() );
  if given != 1
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "exactly one of add::, remove::, tags:: is required".to_string(),
    ) );
  }

  let split = | s : String | -> Vec< String > { s.split( ',' ).map( str::to_string ).collect() };
  let op = if let Some( v ) = add_raw { crate::account::TagOp::Add( split( v ) ) }
    else if let Some( v ) = remove_raw { crate::account::TagOp::Remove( split( v ) ) }
    else { crate::account::TagOp::Replace( split( tags_raw.unwrap_or_default() ) ) };

  // Validate operand tags once, before any resolution or write (fast-fail, exit 1).
  // write_tags() re-validates internally; this keeps batch runs all-or-nothing.
  let ( crate::account::TagOp::Add( ref op_tags )
    | crate::account::TagOp::Remove( ref op_tags )
    | crate::account::TagOp::Replace( ref op_tags ) ) = op;
  crate::account::normalize_tag_set( op_tags )
    .map_err( | e | io_err_to_error_data( &e, "account tag" ) )?;

  let credential_store = require_credential_store()?;

  // Resolve and existence-check ALL names before ANY write (batch atomicity gate).
  // resolve_account_name() passes '@'-containing names through unchecked, so the
  // credentials-file existence check here is what makes a ghost name exit 2.
  let mut resolved : Vec< String > = Vec::new();
  for raw in raw_name.split( ',' ).map( str::trim ).filter( | s | !s.is_empty() )
  {
    let name = resolve_account_name( raw, &credential_store )?;
    if !credential_store.join( format!( "{name}.credentials.json" ) ).exists()
    {
      return Err( ErrorData::new(
        ErrorCode::InternalError,
        format!( "account '{name}' not found" ),
      ) );
    }
    resolved.push( name );
  }
  if resolved.is_empty()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      "name:: must contain at least one account name".to_string(),
    ) );
  }

  let mut out = String::new();
  if is_dry( &cmd )
  {
    for name in &resolved
    {
      let preview = crate::account::preview_tags( name, &credential_store, &op )
        .map_err( | e | io_err_to_error_data( &e, "account tag" ) )?;
      let _ = writeln!( out, "[dry-run] {name}: {}", render_set( &preview ) );
    }
    return Ok( OutputData::new( out, "text" ) );
  }

  for name in &resolved
  {
    let stored = crate::account::write_tags( name, &credential_store, &op )
      .map_err( | e | io_err_to_error_data( &e, "account tag" ) )?;
    if trace { eprintln!( "{}account.tag  write: OK  {name}", trace_ts() ) }
    let _ = writeln!( out, "{name}: {}", render_set( &stored ) );
  }
  Ok( OutputData::new( out, "text" ) )
}

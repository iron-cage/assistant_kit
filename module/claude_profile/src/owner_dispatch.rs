//! Shared owner batch-clear/named-dispatch logic, plus generic boolean-field dispatch.
//!
//! Used by both `.accounts` and `.usage` command handlers to avoid duplicating
//! the ownership gate and write logic. The `label` parameter (`"accounts"` or
//! `"usage"`) distinguishes the caller in trace messages and error context strings.
//!
//! `bool_field_batch_set`/`bool_field_named_dispatch` generalize the same sharing
//! pattern for `claim_lock`/`reserve` (Feature 070) — parameterized by a `writer`
//! function pointer and `field_name` label so both fields and both callers reuse
//! one implementation instead of four near-duplicates.

use core::fmt::Write as _;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use claude_profile_core::account::trace_ts;
use crate::commands::cmd_args::{ io_err_to_error_data, resolve_account_name };

/// Batch-clear ownership for all accounts in `all_accounts`.
///
/// Accounts unowned or owned by another identity are skipped with a `"skip"` line (AC-09).
/// The caller is responsible for validating that `is_sentinel` is true before calling
/// (non-sentinel with no `name::` is an argument error — not this function's concern).
#[ allow( clippy::fn_params_excessive_bools ) ]
pub( crate ) fn owner_batch_clear(
  trace            : bool,
  force            : bool,
  is_dry_run       : bool,
  all_accounts     : &[ crate::account::Account ],
  credential_store : &std::path::Path,
  label            : &str,
) -> Result< OutputData, ErrorData >
{
  let mut out = String::new();
  for acct in all_accounts
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
      if trace { eprintln!( "{}{label} owner  batch-skip (foreign owner): {}  owner={acct_owner}", trace_ts(), acct.name ) }
      writeln!( out, "skip {}", acct.name ).unwrap();
      continue;
    }
    if is_dry_run
    {
      writeln!( out, "[dry-run] would clear owner of {}", acct.name ).unwrap();
      continue;
    }
    crate::account::write_owner( &acct.name, credential_store, "" )
      .map_err( |e| io_err_to_error_data( &e, &format!( "{label} owner batch-clear" ) ) )?;
    if trace { eprintln!( "{}{label} owner  cleared: {}  was={acct_owner}", trace_ts(), acct.name ) }
    writeln!( out, "unclaimed {}", acct.name ).unwrap();
  }
  Ok( OutputData::new( out, "text" ) )
}

/// Assign or clear ownership for one or more named accounts.
///
/// `raw_name` may be a comma-list; each component is resolved independently.
/// `is_sentinel` true means clear ownership (`owner::0`); false means set to `ov`.
/// The G8 ownership gate is evaluated per account before any write (AC-16/AC-17).
#[ allow( clippy::too_many_arguments ) ]
#[ allow( clippy::fn_params_excessive_bools ) ]
pub( crate ) fn owner_named_dispatch(
  trace            : bool,
  force            : bool,
  is_dry_run       : bool,
  is_sentinel      : bool,
  ov               : &str,
  raw_name         : &str,
  name_arg         : &str,
  credential_store : &std::path::Path,
  label            : &str,
) -> Result< OutputData, ErrorData >
{
  // name:: present — resolve each component (comma-list supported for owner:: ops).
  let target_names : Vec< String > = if raw_name.contains( ',' )
  {
    raw_name.split( ',' )
      .map( | part | resolve_account_name( part.trim(), credential_store ) )
      .collect::< Result< Vec< _ >, _ > >()?
  }
  else
  {
    vec![ name_arg.to_owned() ]
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
    let new_owner = if is_sentinel { "" } else { ov };
    crate::account::write_owner( name, credential_store, new_owner )
      .map_err( |e| io_err_to_error_data( &e, &format!( "{label} owner" ) ) )?;
    if trace
    {
      eprintln!(
        "{}{label} owner  write_owner: OK  name={name} identity={}",
        trace_ts(),
        if is_sentinel { "(cleared)" } else { ov },
      );
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
  Ok( OutputData::new( out, "text" ) )
}

/// Batch-set a boolean account field (`claim_lock` / `reserve`) for every account.
///
/// Unlike ownership, these fields are ungated — no ownership check (Feature 070 AC-02).
/// Batch-mode supports both boolean directions (Test Matrix T04) — a deliberate divergence
/// from `owner_batch_clear`, which only supports the sentinel-clear case.
#[ allow( clippy::too_many_arguments ) ]
pub( crate ) fn bool_field_batch_set(
  trace            : bool,
  is_dry_run       : bool,
  value            : bool,
  all_accounts     : &[ crate::account::Account ],
  credential_store : &std::path::Path,
  label            : &str,
  field_name       : &str,
  writer           : fn( &str, &std::path::Path, bool ) -> Result< (), std::io::Error >,
) -> Result< OutputData, ErrorData >
{
  let mut out = String::new();
  for acct in all_accounts
  {
    if is_dry_run
    {
      writeln!( out, "[dry-run] would set {field_name} of {} to {value}", acct.name ).unwrap();
      continue;
    }
    writer( &acct.name, credential_store, value )
      .map_err( |e| io_err_to_error_data( &e, &format!( "{label} {field_name} batch-set" ) ) )?;
    if trace { eprintln!( "{}{label} {field_name}  set: {}  value={value}", trace_ts(), acct.name ) }
    writeln!( out, "{field_name}={value} {}", acct.name ).unwrap();
  }
  Ok( OutputData::new( out, "text" ) )
}

/// Set a boolean account field (`claim_lock` / `reserve`) for one or more named accounts.
///
/// `raw_name` may be a comma-list; each component is resolved independently.
/// Ungated — no ownership check (Feature 070 AC-02; Out of Scope excludes ownership-gating).
#[ allow( clippy::too_many_arguments ) ]
pub( crate ) fn bool_field_named_dispatch(
  trace            : bool,
  is_dry_run       : bool,
  value            : bool,
  raw_name         : &str,
  name_arg         : &str,
  credential_store : &std::path::Path,
  label            : &str,
  field_name       : &str,
  writer           : fn( &str, &std::path::Path, bool ) -> Result< (), std::io::Error >,
) -> Result< OutputData, ErrorData >
{
  let target_names : Vec< String > = if raw_name.contains( ',' )
  {
    raw_name.split( ',' )
      .map( | part | resolve_account_name( part.trim(), credential_store ) )
      .collect::< Result< Vec< _ >, _ > >()?
  }
  else
  {
    vec![ name_arg.to_owned() ]
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
    if is_dry_run
    {
      writeln!( out, "[dry-run] would set {field_name} of {name} to {value}" ).unwrap();
      continue;
    }
    writer( name, credential_store, value )
      .map_err( |e| io_err_to_error_data( &e, &format!( "{label} {field_name}" ) ) )?;
    if trace { eprintln!( "{}{label} {field_name}  set: {name}  value={value}", trace_ts() ) }
    writeln!( out, "{field_name}={value} {name}" ).unwrap();
  }
  Ok( OutputData::new( out, "text" ) )
}

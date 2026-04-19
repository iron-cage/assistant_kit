//! Command handlers for `claude_assets` CLI.
//!
//! Each routine receives a `VerifiedCommand` + `ExecutionContext` and returns
//! `Result<OutputData, ErrorData>`. The four commands are:
//! - `.list`      — survey available and installed artifacts
//! - `.install`   — symlink an artifact from `$PRO_CLAUDE` into `.claude/<kind>/`
//! - `.uninstall` — remove an installed symlink
//! - `.kinds`     — print all supported artifact kinds with path mappings

use core::fmt::Write as _;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use claude_assets_core::artifact::ArtifactKind;
use claude_assets_core::error::{ AssetError, AssetPathsError };
use claude_assets_core::install::{ InstallOutcome, UninstallOutcome, install, uninstall };
use claude_assets_core::paths::AssetPaths;
use claude_assets_core::registry::{ InstallStatus, list_all };

// ── Error mapping ─────────────────────────────────────────────────────────────

fn asset_err_to_error_data( e : &AssetError ) -> ErrorData
{
  ErrorData::new( ErrorCode::InternalError, e.to_string() )
}

fn paths_err_to_error_data( e : &AssetPathsError ) -> ErrorData
{
  ErrorData::new( ErrorCode::InternalError, e.to_string() )
}

// ── Argument helpers ──────────────────────────────────────────────────────────

/// Extract an optional string argument; returns `None` if missing/unset.
fn opt_str( cmd : &VerifiedCommand, name : &str ) -> Option< String >
{
  match cmd.arguments.get( name )
  {
    Some( Value::String( s ) ) => Some( s.clone() ),
    _                          => None,
  }
}

/// Parse `kind::` to `ArtifactKind`; returns `ArgumentTypeMismatch` for unknown strings.
fn parse_kind( raw : &str ) -> Result< ArtifactKind, ErrorData >
{
  ArtifactKind::from_name( raw ).ok_or_else( ||
  {
    let valid : Vec< &str > = ArtifactKind::all().iter().map( |k| k.as_str() ).collect();
    ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "unknown kind '{raw}': valid kinds are {}", valid.join( ", " ) ),
    )
  } )
}

/// Extract a required (non-empty) string argument.
fn require_str( cmd : &VerifiedCommand, name : &str ) -> Result< String, ErrorData >
{
  match opt_str( cmd, name )
  {
    Some( s ) if !s.is_empty() => Ok( s ),
    Some( _ ) => Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      format!( "{name}:: must not be empty" ),
    ) ),
    None => Err( ErrorData::new(
      ErrorCode::ArgumentMissing,
      format!( "{name}:: is required" ),
    ) ),
  }
}

// ── Routines ─────────────────────────────────────────────────────────────────

/// `.list` — survey available and installed artifacts.
///
/// Optional `kind::` filters to one artifact kind; without it, all kinds are shown.
/// Optional `installed::1` restricts output to installed artifacts only.
///
/// # Errors
///
/// Returns `ErrorData` with `InternalError` if `$PRO_CLAUDE` is unset or
/// if the source or target directory cannot be read.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn list_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let paths = AssetPaths::from_env().map_err( |e| paths_err_to_error_data( &e ) )?;

  let raw_kind       = opt_str( &cmd, "kind" ).unwrap_or_default();
  let installed_only = matches!( cmd.arguments.get( "installed" ), Some( Value::Boolean( true ) ) );

  let kinds : Vec< ArtifactKind > = if raw_kind.is_empty()
  {
    ArtifactKind::all().to_vec()
  }
  else
  {
    vec![ parse_kind( &raw_kind )? ]
  };

  let mut out = String::new();

  for kind in kinds
  {
    let entries = list_all( &paths, kind ).map_err( |e| asset_err_to_error_data( &e ) )?;
    if entries.is_empty()
    {
      continue;
    }
    for ( name, status ) in &entries
    {
      if installed_only && *status != InstallStatus::Installed
      {
        continue;
      }
      let marker   = if *status == InstallStatus::Installed { '●' } else { '○' };
      let kind_str = kind.as_str();
      writeln!( out, "{marker} {kind_str}/{name}" ).expect( "string write" );
    }
  }

  if out.is_empty()
  {
    out.push_str( "No artifacts found.\n" );
  }

  Ok( OutputData::new( out, "text" ) )
}

/// `.install` — symlink a named artifact into `.claude/<kind>/`.
///
/// # Errors
///
/// Returns `ErrorData` with `ArgumentMissing` (exit 1) if `kind::` or `name::` is absent.
/// Returns `ErrorData` with `ArgumentTypeMismatch` (exit 1) if `kind::` is unknown.
/// Returns `ErrorData` with `InternalError` (exit 2) if the source artifact is missing,
/// the target is a non-symlink, or a filesystem error occurs.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn install_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let paths = AssetPaths::from_env().map_err( |e| paths_err_to_error_data( &e ) )?;

  let raw_kind = require_str( &cmd, "kind" )?;
  let name     = require_str( &cmd, "name" )?;
  let kind     = parse_kind( &raw_kind )?;

  let report   = install( &paths, kind, &name ).map_err( |e| asset_err_to_error_data( &e ) )?;
  let kind_str = kind.as_str();

  let msg = match report.action
  {
    InstallOutcome::Installed   => format!( "Installed {kind_str}/{name}\n" ),
    InstallOutcome::Reinstalled => format!( "Reinstalled {kind_str}/{name}\n" ),
  };

  Ok( OutputData::new( msg, "text" ) )
}

/// `.uninstall` — remove an installed artifact symlink.
///
/// # Errors
///
/// Returns `ErrorData` with `ArgumentMissing` (exit 1) if `kind::` or `name::` is absent.
/// Returns `ErrorData` with `ArgumentTypeMismatch` (exit 1) if `kind::` is unknown.
/// Returns `ErrorData` with `InternalError` (exit 2) if the target is a non-symlink
/// regular file (data-loss guard) or a filesystem error occurs.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn uninstall_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let paths = AssetPaths::from_env().map_err( |e| paths_err_to_error_data( &e ) )?;

  let raw_kind = require_str( &cmd, "kind" )?;
  let name     = require_str( &cmd, "name" )?;
  let kind     = parse_kind( &raw_kind )?;

  let report   = uninstall( &paths, kind, &name ).map_err( |e| asset_err_to_error_data( &e ) )?;
  let kind_str = kind.as_str();

  let msg = match report.action
  {
    UninstallOutcome::Uninstalled  => format!( "Uninstalled {kind_str}/{name}\n" ),
    UninstallOutcome::NotInstalled => format!( "Not installed: {kind_str}/{name}\n" ),
  };

  Ok( OutputData::new( msg, "text" ) )
}

/// `.kinds` — show all supported artifact kinds with their path mappings.
///
/// # Errors
///
/// This routine never returns `Err` — it degrades gracefully when `$PRO_CLAUDE` is unset.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn kinds_routine( _cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  // Resolve source root for display; fall back gracefully if env not set.
  let source_root = match AssetPaths::from_env()
  {
    Ok( p )  => p.source_root().to_string_lossy().into_owned(),
    Err( _ ) => "$PRO_CLAUDE".to_string(),
  };

  let mut out = String::new();
  for kind in ArtifactKind::all()
  {
    let kind_str = kind.as_str();
    let src      = kind.source_subdir();
    let tgt      = kind.target_subdir();
    writeln!( out, "{kind_str:<10}  {source_root}/{src}  →  .claude/{tgt}" ).expect( "string write" );
  }

  Ok( OutputData::new( out, "text" ) )
}

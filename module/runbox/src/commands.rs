//! Command handlers for `runbox` CLI.
//!
//! The single command is:
//! - `.init` — scaffold container runner integration files in the current directory

use std::fs;

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use crate::templates::{ Ecosystem, dockerfile, runbox_yml, wrapper_script };

// ── Argument helpers ──────────────────────────────────────────────────────────

/// Extract an optional string argument; returns `None` if missing or absent.
fn opt_str( cmd : &VerifiedCommand, name : &str ) -> Option< String >
{
  match cmd.arguments.get( name )
  {
    Some( Value::String( s ) ) => Some( s.clone() ),
    _                          => None,
  }
}

/// Extract a required non-empty string argument.
fn require_str( cmd : &VerifiedCommand, name : &str ) -> Result< String, ErrorData >
{
  let missing = || ErrorData::new(
    ErrorCode::ArgumentMissing,
    format!( "missing required argument: {name}::" ),
  );
  match opt_str( cmd, name )
  {
    Some( s ) if !s.is_empty() => Ok( s ),
    _                          => Err( missing() ),
  }
}

// ── Routines ─────────────────────────────────────────────────────────────────

/// `.init` — scaffold container runner integration files in the current directory.
///
/// Creates `runbox/runbox`, `runbox/runbox.yml`, and `runbox/runbox.dockerfile`
/// under the current working directory.
///
/// # Errors
///
/// Returns `ErrorData` with `ArgumentMissing` (exit 1) if `image::` is absent.
/// Returns `ErrorData` with `ArgumentTypeMismatch` (exit 1) if `ecosystem::` is unknown.
/// Returns `ErrorData` with `ArgumentTypeMismatch` (exit 1) if `runbox/` already exists.
/// Returns `ErrorData` with `InternalError` (exit 2) for filesystem failures.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn init_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let image = require_str( &cmd, "image" )?;

  let eco_raw = opt_str( &cmd, "ecosystem" ).unwrap_or_default();
  let eco = if eco_raw.is_empty()
  {
    Ecosystem::None
  }
  else
  {
    Ecosystem::from_name( &eco_raw ).ok_or_else( || ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "unknown ecosystem: {eco_raw}" ),
    ) )?
  };

  let test_script = opt_str( &cmd, "test_script" )
  .filter( |s| !s.is_empty() )
  .unwrap_or_else( || "verb/test.d/l1".to_string() );

  let cwd = std::env::current_dir().map_err( |e| ErrorData::new(
    ErrorCode::InternalError,
    format!( "cannot determine current directory: {e}" ),
  ) )?;

  let runbox_dir = cwd.join( "runbox" );

  if runbox_dir.exists()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "runbox/ already exists".to_string(),
    ) );
  }

  fs::create_dir( &runbox_dir ).map_err( |e| ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to create runbox/: {e}" ),
  ) )?;

  // Write runbox/runbox wrapper script and make it executable.
  let wrapper_path = runbox_dir.join( "runbox" );
  fs::write( &wrapper_path, wrapper_script() ).map_err( |e| ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to write runbox/runbox: {e}" ),
  ) )?;

  // Fix: set executable bit on Unix; on Windows shell scripts don't use Unix permissions.
  #[ cfg( unix ) ]
  {
    use std::os::unix::fs::PermissionsExt as _;
    let mut perms = fs::metadata( &wrapper_path ).map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "failed to read runbox/runbox metadata: {e}" ),
    ) )?.permissions();
    perms.set_mode( 0o755 );
    fs::set_permissions( &wrapper_path, perms ).map_err( |e| ErrorData::new(
      ErrorCode::InternalError,
      format!( "failed to set permissions on runbox/runbox: {e}" ),
    ) )?;
  }

  // Write runbox/runbox.yml
  fs::write( runbox_dir.join( "runbox.yml" ), runbox_yml( &image, &eco, &test_script ) )
  .map_err( |e| ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to write runbox/runbox.yml: {e}" ),
  ) )?;

  // Write runbox/runbox.dockerfile
  fs::write( runbox_dir.join( "runbox.dockerfile" ), dockerfile( &eco ) )
  .map_err( |e| ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to write runbox/runbox.dockerfile: {e}" ),
  ) )?;

  let msg = format!(
    "Initialized runbox/ for image '{image}' (ecosystem: {})\n  runbox/runbox\n  runbox/runbox.yml\n  runbox/runbox.dockerfile\n",
    eco.as_str(),
  );

  Ok( OutputData::new( msg, "text" ) )
}

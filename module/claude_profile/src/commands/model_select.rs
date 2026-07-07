//! `.model.select` command handler — pin or clear the subprocess model preference.
//!
//! Manages `subprocess_model` in `~/.clr/prefs.json` (Schema 008).
//! Three modes: get (no `id::`, no `reset::`), set (`id::VALUE`), reset (`reset::1`).

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use claude_core::settings_io::{ get_setting, remove_setting, set_setting };
use crate::output::{ OutputFormat, OutputOptions };

const PREFS_KEY : &str = "subprocess_model";

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.model.select` — get or set the clr subprocess model preference.
///
/// **Get mode** (no `id::`, no `reset::1`): prints `model.select: VALUE` or
/// `model.select: (unset)`. Exit 0.
///
/// **Set mode** (`id::VALUE`): writes `subprocess_model` to `~/.clr/prefs.json`,
/// creates the file and parent directory when absent. Prints
/// `model.select: VALUE (pinned)`. Exit 0.
///
/// **Reset mode** (`reset::1`): removes `subprocess_model` key; preserves other
/// keys. Prints `model.select: (reset to default)`. Exit 0. Idempotent when
/// file is absent.
///
/// `id::` and `reset::1` together → exit 1 with `mutually exclusive` in stderr.
/// `id::` with empty value → exit 1.
///
/// # Errors
///
/// Returns `Err(ErrorData)` with `ArgumentTypeMismatch` when `id::` and `reset::1` are both set,
/// `ArgumentMissing` when `id::` is empty, or `InternalError` on file I/O failure.
#[ inline ]
pub fn model_select_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts     = OutputOptions::from_cmd( &cmd )?;
  let id_val   = match cmd.arguments.get( "id" )
  {
    Some( Value::String( s ) ) => Some( s.clone() ),
    _                          => None,
  };
  let reset_val = matches!( cmd.arguments.get( "reset" ), Some( Value::Integer( 1 ) ) );

  // Mutual exclusion
  if id_val.is_some() && reset_val
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "model.select: id:: and reset::1 are mutually exclusive".to_string(),
    ) );
  }

  // Validate non-empty id
  if let Some( ref id ) = id_val
  {
    if id.is_empty()
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentMissing,
        "model.select: id:: must be non-empty — pass a full model ID (e.g. claude-opus-4-8)".to_string(),
      ) );
    }
  }

  let prefs_path = resolve_prefs_path()?;

  if let Some( ref model_id ) = id_val
  {
    // Set mode
    set_prefs_model( &prefs_path, model_id )?;
    Ok( OutputData::new( format!( "model.select: {model_id} (pinned)\n" ), "text" ) )
  }
  else if reset_val
  {
    // Reset mode
    remove_prefs_model( &prefs_path )?;
    Ok( OutputData::new( "model.select: (reset to default)\n".to_string(), "text" ) )
  }
  else
  {
    // Get mode
    let current = read_prefs_model( &prefs_path );
    let text = match opts.format
    {
      OutputFormat::Json =>
      {
        match &current
        {
          Some( m ) => format!( "{{\"subprocess_model\":\"{m}\"}}\n" ),
          None      => "{\"subprocess_model\":null}\n".to_string(),
        }
      }
      OutputFormat::Text | OutputFormat::Table =>
      {
        match &current
        {
          Some( m ) => format!( "model.select: {m}\n" ),
          None      => "model.select: (unset)\n".to_string(),
        }
      }
    };
    Ok( OutputData::new( text, "text" ) )
  }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Resolve `~/.clr/prefs.json` path.
fn resolve_prefs_path() -> Result< std::path::PathBuf, ErrorData >
{
  let home = std::env::var( "HOME" )
    .map_err( |_| ErrorData::new( ErrorCode::InternalError, "HOME environment variable not set".to_string() ) )?;
  Ok( std::path::PathBuf::from( home ).join( ".clr" ).join( "prefs.json" ) )
}

/// Read `subprocess_model` from `prefs.json`; `None` when absent or file missing.
fn read_prefs_model( path : &std::path::Path ) -> Option< String >
{
  get_setting( path, PREFS_KEY ).ok().flatten()
}

/// Write or update `subprocess_model` in `prefs.json`, creating dir + file as needed.
fn set_prefs_model( path : &std::path::Path, model_id : &str ) -> Result< (), ErrorData >
{
  if let Some( parent ) = path.parent()
  {
    std::fs::create_dir_all( parent ).map_err( | e | ErrorData::new(
      ErrorCode::InternalError,
      format!( "failed to create .clr directory: {e}" ),
    ) )?;
  }
  set_setting( path, PREFS_KEY, model_id ).map( | _ | () ).map_err( | e | ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to write prefs.json: {e}" ),
  ) )
}

/// Remove `subprocess_model` from `prefs.json`; no-op if file absent.
fn remove_prefs_model( path : &std::path::Path ) -> Result< (), ErrorData >
{
  remove_setting( path, PREFS_KEY ).map_err( | e | ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to write prefs.json: {e}" ),
  ) )
}

//! `.model.select` command handler — pin or clear the subprocess model preference.
//!
//! Manages the `model` key in `~/.clr/config.toml`'s user tier — the same
//! key `--model`'s own config-file resolution tier reads (task 410). The
//! `format::json` output shape is unchanged: still keyed `subprocess_model`,
//! this command's own CLI-visible JSON contract, independent of the backing
//! store's key name.
//! Three modes: get (no `id::`, no `reset::`), set (`id::VALUE`), reset (`reset::1`).

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use claude_core::toml_io::{ get_tiered, remove_user_tier, set_user_tier };
use crate::output::{ OutputFormat, OutputOptions };

const MODEL_KEY : &str = "model";

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.model.select` — get or set the clr subprocess model preference.
///
/// **Get mode** (no `id::`, no `reset::1`): prints `model.select: VALUE` or
/// `model.select: (unset)`. Exit 0.
///
/// **Set mode** (`id::VALUE`): writes `model` to `~/.clr/config.toml`'s user
/// tier, creates the file and parent directory when absent. Prints
/// `model.select: VALUE (pinned)`. Exit 0.
///
/// **Reset mode** (`reset::1`): removes the `model` key; preserves other
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

  let config_path = resolve_config_path()?;

  if let Some( ref model_id ) = id_val
  {
    // Set mode
    set_config_model( &config_path, model_id )?;
    Ok( OutputData::new( format!( "model.select: {model_id} (pinned)\n" ), "text" ) )
  }
  else if reset_val
  {
    // Reset mode
    remove_config_model( &config_path )?;
    Ok( OutputData::new( "model.select: (reset to default)\n".to_string(), "text" ) )
  }
  else
  {
    // Get mode
    let current = read_config_model( &config_path );
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

/// Resolve `~/.clr/config.toml` path (user tier; no project-tier merge for
/// this command's get/set/reset semantics).
fn resolve_config_path() -> Result< std::path::PathBuf, ErrorData >
{
  let home = std::env::var( "HOME" )
    .map_err( |_| ErrorData::new( ErrorCode::InternalError, "HOME environment variable not set".to_string() ) )?;
  Ok( std::path::PathBuf::from( home ).join( ".clr" ).join( "config.toml" ) )
}

/// Read `model` from `config.toml`'s user tier; `None` when absent or file missing.
fn read_config_model( path : &std::path::Path ) -> Option< String >
{
  get_tiered( None, path, MODEL_KEY )
}

/// Write or update `model` in `config.toml`'s user tier, creating dir + file as needed.
fn set_config_model( path : &std::path::Path, model_id : &str ) -> Result< (), ErrorData >
{
  if let Some( parent ) = path.parent()
  {
    std::fs::create_dir_all( parent ).map_err( | e | ErrorData::new(
      ErrorCode::InternalError,
      format!( "failed to create .clr directory: {e}" ),
    ) )?;
  }
  set_user_tier( path, MODEL_KEY, model_id ).map_err( | e | ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to write config.toml: {e}" ),
  ) )
}

/// Remove `model` from `config.toml`'s user tier; no-op if file absent.
fn remove_config_model( path : &std::path::Path ) -> Result< (), ErrorData >
{
  remove_user_tier( path, MODEL_KEY ).map_err( | e | ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to write config.toml: {e}" ),
  ) )
}

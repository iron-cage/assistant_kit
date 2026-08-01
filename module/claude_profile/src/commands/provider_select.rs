//! `.provider.select` command handler — get, pin, or reset the global inference
//! provider selection.
//!
//! Manages the `provider` key in `~/.clr/config.toml`'s user tier — the
//! **sole write path** for the selected-provider config value anywhere in
//! this workspace. No other command or code path may set or infer `provider`
//! (no fallback chain, no auto-detection — the value is a plain global
//! config setting, changed only by explicit user action here).
//! Three modes: get (no `id::`, no `reset::`), set (`id::VALUE`), reset (`reset::1`).

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use claude_core::toml_io::{ get_tiered, remove_user_tier, set_user_tier };
use crate::output::{ OutputFormat, OutputOptions };

const PROVIDER_KEY     : &str = "provider";
const DEFAULT_PROVIDER : &str = "anthropic";

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.provider.select` — get, pin, or reset the global inference provider selection.
///
/// **Get mode** (no `id::`, no `reset::1`): prints `provider.select: VALUE`,
/// defaulting to `provider.select: anthropic` when never explicitly set —
/// never an `(unset)`-style sentinel. Exit 0.
///
/// **Set mode** (`id::VALUE`): writes `provider` to `~/.clr/config.toml`'s
/// user tier, creates the file and parent directory when absent. Prints
/// `provider.select: VALUE (selected)`. Exit 0.
///
/// **Reset mode** (`reset::1`): removes the `provider` key; preserves other
/// keys. Prints `provider.select: anthropic (reset to default)`. Exit 0.
/// Idempotent when file is absent.
///
/// `id::` and `reset::1` together → exit 1 with `mutually exclusive` in stderr.
/// `id::` with empty value → exit 1.
///
/// # Errors
///
/// Returns `Err(ErrorData)` with `ArgumentTypeMismatch` when `id::` and `reset::1` are both set,
/// `ArgumentMissing` when `id::` is empty, or `InternalError` on file I/O failure.
#[ inline ]
pub fn provider_select_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts      = OutputOptions::from_cmd( &cmd )?;
  let id_val    = match cmd.arguments.get( "id" )
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
      "provider.select: id:: and reset::1 are mutually exclusive".to_string(),
    ) );
  }

  // Validate non-empty id
  if let Some( ref id ) = id_val
  {
    if id.is_empty()
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentMissing,
        "provider.select: id:: must be a non-empty provider name".to_string(),
      ) );
    }
  }

  let config_path = resolve_config_path()?;

  if let Some( ref provider_id ) = id_val
  {
    // Set mode
    set_config_provider( &config_path, provider_id )?;
    Ok( OutputData::new( format!( "provider.select: {provider_id} (selected)\n" ), "text" ) )
  }
  else if reset_val
  {
    // Reset mode
    remove_config_provider( &config_path )?;
    Ok( OutputData::new( format!( "provider.select: {DEFAULT_PROVIDER} (reset to default)\n" ), "text" ) )
  }
  else
  {
    // Get mode
    let current = read_config_provider( &config_path ).unwrap_or_else( || DEFAULT_PROVIDER.to_string() );
    let text = match opts.format
    {
      OutputFormat::Json => format!( "{{\"provider\":\"{current}\"}}\n" ),
      OutputFormat::Text | OutputFormat::Table => format!( "provider.select: {current}\n" ),
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

/// Read `provider` from `config.toml`'s user tier; `None` when absent or file missing.
fn read_config_provider( path : &std::path::Path ) -> Option< String >
{
  get_tiered( None, path, PROVIDER_KEY )
}

/// Write or update `provider` in `config.toml`'s user tier, creating dir + file as needed.
fn set_config_provider( path : &std::path::Path, provider_id : &str ) -> Result< (), ErrorData >
{
  if let Some( parent ) = path.parent()
  {
    std::fs::create_dir_all( parent ).map_err( | e | ErrorData::new(
      ErrorCode::InternalError,
      format!( "failed to create .clr directory: {e}" ),
    ) )?;
  }
  set_user_tier( path, PROVIDER_KEY, provider_id ).map_err( | e | ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to write config.toml: {e}" ),
  ) )
}

/// Remove `provider` from `config.toml`'s user tier; no-op if file absent.
fn remove_config_provider( path : &std::path::Path ) -> Result< (), ErrorData >
{
  remove_user_tier( path, PROVIDER_KEY ).map_err( | e | ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to write config.toml: {e}" ),
  ) )
}

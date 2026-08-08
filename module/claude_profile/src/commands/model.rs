//! `.model` command handler — unified session + subprocess model and effort management.
//!
//! Routes on `scope::` between two persisted stores: the Claude Code interactive
//! session (`~/.claude/settings.json`, `scope::session`, default) and the clr
//! subprocess-execution preference (`~/.clr/config.toml` user tier, `scope::subprocess`).
//! Absorbs the former `.model.select` command (Feature 035) — see `model_select.rs`
//! for its retirement stub.

use core::fmt::Write;
use std::path::PathBuf;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use claude_core::toml_io::{ get_tiered, remove_user_tier, set_user_tier };
use crate::output::{ OutputFormat, OutputOptions };
use super::cmd_context::require_claude_paths;
use crate::usage::map_model_shorthand;

const SESSION_EFFORT_VALUES    : &[ &str ] = &[ "low", "normal", "high", "max" ];
const SUBPROCESS_EFFORT_VALUES : &[ &str ] = &[ "low", "medium", "high", "max" ];

// ── Argument parsing ─────────────────────────────────────────────────────────

/// Parsed `.model` arguments, scope-independent.
struct ModelArgs
{
  scope              : String,
  model              : Option< String >,
  effort_level       : Option< String >,
  reset_model        : bool,
  reset_effort_level : bool,
}

impl ModelArgs
{
  fn from_cmd( cmd : &VerifiedCommand ) -> Self
  {
    let scope = match cmd.arguments.get( "scope" )
    {
      Some( Value::String( s ) ) => s.clone(),
      _                          => "session".to_string(),
    };
    let model = match cmd.arguments.get( "model" )
    {
      Some( Value::String( s ) ) => Some( s.clone() ),
      _                          => None,
    };
    let effort_level = match cmd.arguments.get( "effort_level" )
    {
      Some( Value::String( s ) ) => Some( s.clone() ),
      _                          => None,
    };
    let reset_model        = matches!( cmd.arguments.get( "reset_model" ),        Some( Value::Integer( 1 ) ) );
    let reset_effort_level = matches!( cmd.arguments.get( "reset_effort_level" ), Some( Value::Integer( 1 ) ) );
    Self { scope, model, effort_level, reset_model, reset_effort_level }
  }

  /// True when any write-triggering argument is present (write mode); false → get mode.
  fn is_write( &self ) -> bool
  {
    self.model.is_some() || self.effort_level.is_some() || self.reset_model || self.reset_effort_level
  }
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.model` — get or set model + effort level for `scope::session` or `scope::subprocess`.
///
/// **Get mode** (no `model::`/`effort_level::`/`reset_model::1`/`reset_effort_level::1`):
/// prints `scope`, resolved absolute `path`, `model`, `effort_level` together.
/// `format::json` returns `{"scope":...,"path":...,"model":...,"effort_level":...}`.
///
/// **Write mode**: applies each present action independently against the selected
/// scope's store; actions may combine freely across the model/effort concepts (never
/// within the same concept's set+reset pair). Prints one confirmation line per applied
/// action.
///
/// Absorbs the former `.model.select` command (Feature 035) — see `model_select.rs`
/// for its retirement stub.
///
/// # Errors
///
/// Returns `ErrorData` on: unknown `scope::` value; `model::`+`reset_model::1` or
/// `effort_level::`+`reset_effort_level::1` together; unknown `model::`/`effort_level::`
/// value for the selected scope; empty `model::` on `scope::subprocess`; HOME unset.
#[ inline ]
pub fn model_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts = OutputOptions::from_cmd( &cmd )?;
  if opts.is_table()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "format::table is not supported by .model".to_string(),
    ) );
  }
  let args = ModelArgs::from_cmd( &cmd );
  validate_scope( &args.scope )?;

  if args.is_write()
  {
    model_write( &args )
  }
  else
  {
    model_get( &args.scope, opts.format )
  }
}

// ── Validation ────────────────────────────────────────────────────────────────

/// Reject any `scope::` value other than `session`/`subprocess`.
fn validate_scope( scope : &str ) -> Result< (), ErrorData >
{
  if scope != "session" && scope != "subprocess"
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      format!( "scope:: must be one of: session, subprocess; got {scope:?}" ),
    ) );
  }
  Ok( () )
}

/// Reject `model::`+`reset_model::1` or `effort_level::`+`reset_effort_level::1` together.
fn validate_mutual_exclusion( args : &ModelArgs ) -> Result< (), ErrorData >
{
  if args.model.is_some() && args.reset_model
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "model:: and reset_model::1 are mutually exclusive".to_string(),
    ) );
  }
  if args.effort_level.is_some() && args.reset_effort_level
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "effort_level:: and reset_effort_level::1 are mutually exclusive".to_string(),
    ) );
  }
  Ok( () )
}

// ── Path resolution ───────────────────────────────────────────────────────────

/// Resolve `~/.clr/config.toml` path (user tier; no project-tier merge).
fn resolve_subprocess_config_path() -> Result< PathBuf, ErrorData >
{
  let home = std::env::var( "HOME" )
    .map_err( |_| ErrorData::new( ErrorCode::InternalError, "HOME environment variable not set".to_string() ) )?;
  Ok( PathBuf::from( home ).join( ".clr" ).join( "config.toml" ) )
}

// ── Get mode ──────────────────────────────────────────────────────────────────

/// Get mode: read model + effort for `scope`, render together with resolved path.
fn model_get( scope : &str, format : OutputFormat ) -> Result< OutputData, ErrorData >
{
  let ( path, model, effort ) = if scope == "session"
  {
    let paths  = require_claude_paths()?;
    let model  = claude_profile_core::account::get_session_model( &paths );
    let effort = claude_profile_core::account::get_session_effort( &paths );
    ( paths.settings_file(), model, effort )
  }
  else
  {
    let path   = resolve_subprocess_config_path()?;
    let model  = get_tiered( None, &path, "model" );
    let effort = get_tiered( None, &path, "effort" );
    ( path, model, effort )
  };

  let path_str = path.display().to_string();
  let text = match format
  {
    OutputFormat::Json =>
    {
      let m = model.as_deref().map_or( "null".to_string(), | v | format!( "\"{v}\"" ) );
      let e = effort.as_deref().map_or( "null".to_string(), | v | format!( "\"{v}\"" ) );
      format!( "{{\"scope\":\"{scope}\",\"path\":\"{path_str}\",\"model\":{m},\"effort_level\":{e}}}\n" )
    }
    OutputFormat::Text | OutputFormat::Table =>
    {
      format!(
        "scope: {scope} ({path_str})\nmodel: {}\neffort_level: {}\n",
        model.as_deref().unwrap_or( "(unset)" ),
        effort.as_deref().unwrap_or( "(unset)" ),
      )
    }
  };
  Ok( OutputData::new( text, "text" ) )
}

// ── Write mode ────────────────────────────────────────────────────────────────

/// Write mode: validate mutual exclusion, apply actions for the selected scope.
fn model_write( args : &ModelArgs ) -> Result< OutputData, ErrorData >
{
  validate_mutual_exclusion( args )?;
  let mut lines = String::new();
  if args.scope == "session"
  {
    session_apply( args, &mut lines )?;
  }
  else
  {
    subprocess_apply( args, &mut lines )?;
  }
  Ok( OutputData::new( lines, "text" ) )
}

/// Apply write actions against `scope::session` (`~/.claude/settings.json`).
fn session_apply( args : &ModelArgs, out : &mut String ) -> Result< (), ErrorData >
{
  let paths    = require_claude_paths()?;
  let path_str = paths.settings_file().display().to_string();

  if let Some( ref val ) = args.model
  {
    let model_id = map_model_shorthand( val )
      .ok_or_else( || ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "model:: must be one of: opus, sonnet, haiku, default; got {val:?}" ),
      ) )?;
    claude_profile_core::account::set_session_model( &paths, model_id );
    let _ = writeln!( out, "model: {val}  →  {path_str} (session)" );
  }
  if args.reset_model
  {
    claude_profile_core::account::set_session_model( &paths, None );
    let _ = writeln!( out, "model: (reset)  →  {path_str} (session)" );
  }
  if let Some( ref val ) = args.effort_level
  {
    if !SESSION_EFFORT_VALUES.contains( &val.as_str() )
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "effort_level:: must be one of: low, normal, high, max; got {val:?}" ),
      ) );
    }
    claude_profile_core::account::set_session_effort( &paths, val );
    let _ = writeln!( out, "effort_level: {val}  →  {path_str} (session)" );
  }
  if args.reset_effort_level
  {
    claude_profile_core::account::remove_session_effort( &paths );
    let _ = writeln!( out, "effort_level: (reset)  →  {path_str} (session)" );
  }
  Ok( () )
}

/// Apply write actions against `scope::subprocess` (`~/.clr/config.toml` user tier).
fn subprocess_apply( args : &ModelArgs, out : &mut String ) -> Result< (), ErrorData >
{
  let path = resolve_subprocess_config_path()?;
  if let Some( parent ) = path.parent()
  {
    std::fs::create_dir_all( parent ).map_err( | e | ErrorData::new(
      ErrorCode::InternalError, format!( "failed to create .clr directory: {e}" ),
    ) )?;
  }
  subprocess_apply_model( args, &path, out )?;
  subprocess_apply_effort( args, &path, out )?;
  Ok( () )
}

/// Apply `model::`/`reset_model::1` against the subprocess `config.toml`.
fn subprocess_apply_model( args : &ModelArgs, path : &std::path::Path, out : &mut String ) -> Result< (), ErrorData >
{
  let path_str = path.display().to_string();
  if let Some( ref val ) = args.model
  {
    if val.is_empty()
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentMissing,
        "model:: must be non-empty on scope::subprocess — pass a full model ID (e.g. claude-opus-4-8)".to_string(),
      ) );
    }
    set_user_tier( path, "model", val ).map_err( | e | ErrorData::new(
      ErrorCode::InternalError, format!( "failed to write config.toml: {e}" ),
    ) )?;
    let _ = writeln!( out, "model: {val}  →  {path_str} (subprocess)" );
  }
  if args.reset_model
  {
    remove_user_tier( path, "model" ).map_err( | e | ErrorData::new(
      ErrorCode::InternalError, format!( "failed to write config.toml: {e}" ),
    ) )?;
    let _ = writeln!( out, "model: (reset)  →  {path_str} (subprocess)" );
  }
  Ok( () )
}

/// Apply `effort_level::`/`reset_effort_level::1` against the subprocess `config.toml`.
fn subprocess_apply_effort( args : &ModelArgs, path : &std::path::Path, out : &mut String ) -> Result< (), ErrorData >
{
  let path_str = path.display().to_string();
  if let Some( ref val ) = args.effort_level
  {
    if !SUBPROCESS_EFFORT_VALUES.contains( &val.as_str() )
    {
      return Err( ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "effort_level:: must be one of: low, medium, high, max; got {val:?}" ),
      ) );
    }
    set_user_tier( path, "effort", val ).map_err( | e | ErrorData::new(
      ErrorCode::InternalError, format!( "failed to write config.toml: {e}" ),
    ) )?;
    let _ = writeln!( out, "effort_level: {val}  →  {path_str} (subprocess)" );
  }
  if args.reset_effort_level
  {
    remove_user_tier( path, "effort" ).map_err( | e | ErrorData::new(
      ErrorCode::InternalError, format!( "failed to write config.toml: {e}" ),
    ) )?;
    let _ = writeln!( out, "effort_level: (reset)  →  {path_str} (subprocess)" );
  }
  Ok( () )
}

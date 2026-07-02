//! Command handlers: one function per `claude_version` subcommand.
//!
//! Each handler receives a `VerifiedCommand` and `ExecutionContext` and returns
//! `Result<OutputData, ErrorData>`. Handlers are registered via `register_commands()`
//! in `lib.rs`.
//!
//! # Architectural Boundary
//!
//! Account management commands (`.account.*`) are **not** implemented here.
//! They live in `claude_profile` (Layer 2 peer crate). `claude_version` has
//! zero dependency on `claude_profile_core` — account CLI code moved there
//! in plan 005 to fix a layering violation where `claude_profile_core`
//! (Layer 1 pure domain) had pulled in CLI dependencies.
//!
//! # Note on `needless_pass_by_value`
//!
//! Every handler takes `VerifiedCommand` by value because the `CommandRoutine`
//! type alias requires ownership.

mod config;
mod history;
mod params;
mod process;
mod runtime_files;
mod settings;
mod status;
mod version;

pub use config::config_routine;
pub use history::version_history_routine;
pub use params::params_routine;
pub use runtime_files::runtime_files_routine;
pub use process::{ processes_kill_routine, processes_routine };
pub use settings::{ settings_get_routine, settings_set_routine, settings_show_routine };
pub use status::status_routine;
pub use version::{
  version_guard_routine, version_install_routine,
  version_list_routine, version_show_routine,
};

use unilang::data::{ ErrorCode, ErrorData };
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

// ── Shared private helpers ────────────────────────────────────────────────────

/// Require a non-empty string argument from the command's argument map.
fn require_nonempty_string_arg( cmd : &VerifiedCommand, name : &str ) -> Result< String, ErrorData >
{
  let val = match cmd.arguments.get( name )
  {
    Some( Value::String( s ) ) => s.clone(),
    _ => return Err( ErrorData::new( ErrorCode::ArgumentMissing, format!( "{name}:: is required" ) ) ),
  };
  if val.is_empty()
  {
    return Err( ErrorData::new( ErrorCode::ArgumentMissing, format!( "{name}:: value cannot be empty" ) ) );
  }
  Ok( val )
}

/// Return `true` when the command has `dry::1`.
#[ inline ]
fn is_dry( cmd : &VerifiedCommand ) -> bool
{
  matches!( cmd.arguments.get( "dry" ), Some( Value::Boolean( true ) ) )
}

/// Return `true` when the command has `force::1`.
#[ inline ]
fn is_force( cmd : &VerifiedCommand ) -> bool
{
  matches!( cmd.arguments.get( "force" ), Some( Value::Boolean( true ) ) )
}

/// Validate HOME is non-empty and return a `ClaudePaths`.
fn require_claude_paths() -> Result< claude_core::ClaudePaths, ErrorData >
{
  match std::env::var( "HOME" )
  {
    Ok( home ) if !home.is_empty() =>
    {
      claude_core::ClaudePaths::new().ok_or_else( || ErrorData::new(
        ErrorCode::InternalError,
        "could not resolve Claude configuration paths (HOME is set but path resolution failed)".to_string(),
      ) )
    }
    Ok( _ ) => Err( ErrorData::new( ErrorCode::InternalError, "HOME environment variable is empty".to_string() ) ),
    Err( _ ) => Err( ErrorData::new( ErrorCode::InternalError, "HOME environment variable not set".to_string() ) ),
  }
}

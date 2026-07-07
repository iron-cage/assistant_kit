//! Config-file parameter tier: `~/.clr/config.toml` (or `$CLR_CONFIG_DIR/config.toml`)
//! and project-level `.clr.toml`, applied between `CLR_*` env vars and hardcoded
//! defaults in the 5-level CLI parameter precedence chain (CLI > `--args-file`
//! JSON > `CLR_*` env > this tier > hardcoded default).

use crate::cli::parse::CliArgs;
use claude_runner_core::EffortLevel;
use error_tools::{ Error, Result };
use std::path::PathBuf;

/// Config-file-supplied defaults for CLI parameters — one field per eligible
/// parameter (`docs/cli/config_param.md`), mirroring `CliArgs` field names and
/// types exactly. `bool` fields (not `Option<bool>`) so an absent key and an
/// explicit `false` are indistinguishable at the struct level; project-vs-user
/// precedence correctness instead comes from merging raw TOML tables before this
/// struct is deserialized (see `load_config`).
#[ allow( clippy::struct_excessive_bools ) ]
#[ derive( Default, serde::Deserialize ) ]
#[ serde( default ) ]
pub( crate ) struct ConfigDefaults
{
  pub( crate ) model                : Option< String >,
  pub( crate ) max_tokens           : Option< u32 >,
  pub( crate ) effort               : Option< String >,
  pub( crate ) no_effort_max        : bool,
  pub( crate ) max_sessions         : Option< u32 >,
  pub( crate ) retry_on_transient   : Option< u8 >,
  pub( crate ) transient_delay      : Option< u32 >,
  pub( crate ) retry_on_account     : Option< u8 >,
  pub( crate ) account_delay        : Option< u32 >,
  pub( crate ) retry_on_auth        : Option< u8 >,
  pub( crate ) auth_delay           : Option< u32 >,
  pub( crate ) retry_on_service     : Option< u8 >,
  pub( crate ) service_delay        : Option< u32 >,
  pub( crate ) retry_on_process     : Option< u8 >,
  pub( crate ) process_delay        : Option< u32 >,
  pub( crate ) retry_on_validation  : Option< u8 >,
  pub( crate ) validation_delay     : Option< u32 >,
  pub( crate ) retry_on_runner      : Option< u8 >,
  pub( crate ) runner_delay         : Option< u32 >,
  pub( crate ) retry_on_unknown     : Option< u8 >,
  pub( crate ) unknown_delay        : Option< u32 >,
  pub( crate ) retry_override       : Option< u8 >,
  pub( crate ) retry_override_delay : Option< u32 >,
  pub( crate ) retry_default        : Option< u8 >,
  pub( crate ) retry_default_delay  : Option< u32 >,
  pub( crate ) timeout              : Option< u32 >,
  pub( crate ) output_style         : Option< String >,
  pub( crate ) summary_fields       : Option< String >,
  pub( crate ) journal              : Option< String >,
  pub( crate ) journal_dir          : Option< String >,
  pub( crate ) quiet                : bool,
  pub( crate ) no_chrome            : bool,
  pub( crate ) no_persist           : bool,
  pub( crate ) no_compact_window    : bool,
  pub( crate ) allowed_tools        : Option< String >,
  pub( crate ) disallowed_tools     : Option< String >,
  pub( crate ) max_budget_usd       : Option< String >,
  pub( crate ) fallback_model       : Option< String >,
}

/// Resolve the user-level config directory: `$CLR_CONFIG_DIR` if set and
/// non-empty, else `$HOME/.clr`, else `.clr` (mirrors `resolve_journal_dir`'s
/// `HOME`-fallback style in `mod.rs`).
fn user_config_dir() -> PathBuf
{
  if let Ok( v ) = std::env::var( "CLR_CONFIG_DIR" )
  {
    if !v.is_empty() { return PathBuf::from( v ); }
  }
  std::env::var( "HOME" )
    .map_or_else( | _ | PathBuf::from( ".clr" ), | h | PathBuf::from( h ).join( ".clr" ) )
}

/// Discover the project-level and user-level config file paths.
///
/// Returns `(project_path, user_path)` — each `Some` only if the file actually
/// exists on disk. Project-level is `.clr.toml` in the current directory;
/// user-level is `config.toml` under `user_config_dir()`.
pub( crate ) fn discover_config_paths() -> ( Option< PathBuf >, Option< PathBuf > )
{
  let project = PathBuf::from( ".clr.toml" );
  let project = if project.is_file() { Some( project ) } else { None };

  let user = user_config_dir().join( "config.toml" );
  let user = if user.is_file() { Some( user ) } else { None };

  ( project, user )
}

/// Read a TOML file into a raw table, naming the file path on failure.
fn read_toml_table( path : &std::path::Path ) -> Result< toml::Table >
{
  let content = std::fs::read_to_string( path )
    .map_err( | e | Error::msg( format!( "failed to read config file {}: {e}", path.display() ) ) )?;
  content.parse::< toml::Table >()
    .map_err( | e | Error::msg( format!( "invalid TOML in config file {}: {e}", path.display() ) ) )
}

/// Load and merge the config-file tier into a single `ConfigDefaults`.
///
/// Project-level values take precedence over user-level values on the same key.
/// Merging happens on the raw `toml::Table`s (before the final typed deserialize)
/// so an absent key in the project file is never confused with an explicit
/// override — required because `bool` fields in `ConfigDefaults` are plain
/// `bool`, not `Option<bool>`. Unknown keys are silently ignored (forward
/// compatibility) — `serde`'s default struct deserialization does not reject
/// them. A missing file is treated as an empty table, not an error; only a
/// present-but-malformed file is an error.
pub( crate ) fn load_config() -> Result< ConfigDefaults >
{
  let ( project_path, user_path ) = discover_config_paths();

  let mut merged = toml::Table::new();
  if let Some( path ) = &user_path
  {
    for ( k, v ) in read_toml_table( path )? { merged.insert( k, v ); }
  }
  if let Some( path ) = &project_path
  {
    for ( k, v ) in read_toml_table( path )? { merged.insert( k, v ); }
  }

  let merged_str = toml::to_string( &merged )
    .map_err( | e | Error::msg( format!( "failed to serialize merged config: {e}" ) ) )?;
  toml::from_str::< ConfigDefaults >( &merged_str )
    .map_err( | e | Error::msg( format!( "invalid config file structure: {e}" ) ) )
}

/// Fill any currently-unset `CliArgs` fields from `config` — the lowest-precedence
/// tier above hardcoded defaults. Mirrors `apply_env_vars`'s and
/// `apply_json_config`'s fill-only-if-unset guard: never overwrites a value
/// already set by a higher tier (CLI flag, `--args-file`, or `CLR_*` env var).
/// Validates `output_style`/`summary_fields`/`journal` exactly as `apply_env_vars`
/// does, returning `Err` on an unrecognized value — config-file input is no less
/// trusted than an env var, so it must be rejected with the same rigor.
#[ allow( clippy::too_many_lines ) ] // config-field mapping is inherently wide — one branch per field, mirrors apply_env_vars.
pub( crate ) fn apply_config_defaults( parsed : &mut CliArgs, config : &ConfigDefaults ) -> Result< () >
{
  if parsed.model.is_none() { parsed.model.clone_from( &config.model ); }
  if parsed.max_tokens.is_none() { parsed.max_tokens = config.max_tokens; }
  if parsed.effort.is_none()
  {
    if let Some( s ) = &config.effort
    {
      if let Ok( e ) = s.parse::< EffortLevel >() { parsed.effort = Some( e ); }
    }
  }
  if !parsed.no_effort_max { parsed.no_effort_max = config.no_effort_max; }
  if parsed.max_sessions.is_none() { parsed.max_sessions = config.max_sessions; }
  if parsed.retry_on_transient.is_none() { parsed.retry_on_transient = config.retry_on_transient; }
  if parsed.transient_delay.is_none() { parsed.transient_delay = config.transient_delay; }
  if parsed.retry_on_account.is_none() { parsed.retry_on_account = config.retry_on_account; }
  if parsed.account_delay.is_none() { parsed.account_delay = config.account_delay; }
  if parsed.retry_on_auth.is_none() { parsed.retry_on_auth = config.retry_on_auth; }
  if parsed.auth_delay.is_none() { parsed.auth_delay = config.auth_delay; }
  if parsed.retry_on_service.is_none() { parsed.retry_on_service = config.retry_on_service; }
  if parsed.service_delay.is_none() { parsed.service_delay = config.service_delay; }
  if parsed.retry_on_process.is_none() { parsed.retry_on_process = config.retry_on_process; }
  if parsed.process_delay.is_none() { parsed.process_delay = config.process_delay; }
  if parsed.retry_on_validation.is_none() { parsed.retry_on_validation = config.retry_on_validation; }
  if parsed.validation_delay.is_none() { parsed.validation_delay = config.validation_delay; }
  if parsed.retry_on_runner.is_none() { parsed.retry_on_runner = config.retry_on_runner; }
  if parsed.runner_delay.is_none() { parsed.runner_delay = config.runner_delay; }
  if parsed.retry_on_unknown.is_none() { parsed.retry_on_unknown = config.retry_on_unknown; }
  if parsed.unknown_delay.is_none() { parsed.unknown_delay = config.unknown_delay; }
  if parsed.retry_override.is_none() { parsed.retry_override = config.retry_override; }
  if parsed.retry_override_delay.is_none() { parsed.retry_override_delay = config.retry_override_delay; }
  if parsed.retry_default.is_none() { parsed.retry_default = config.retry_default; }
  if parsed.retry_default_delay.is_none() { parsed.retry_default_delay = config.retry_default_delay; }
  if parsed.timeout.is_none() { parsed.timeout = config.timeout; }
  if parsed.output_style.is_none()
  {
    if let Some( v ) = &config.output_style
    {
      if !matches!( v.as_str(), "summary" | "raw" )
      {
        return Err( Error::msg( format!(
          "config file: invalid output_style '{v}' — expected: summary, raw"
        ) ) );
      }
      parsed.output_style = Some( v.clone() );
    }
  }
  if parsed.summary_fields.is_none()
  {
    if let Some( v ) = &config.summary_fields
    {
      if super::summary::resolve_fields( v ).is_err()
      {
        return Err( Error::msg( format!( "config file: invalid summary_fields '{v}'" ) ) );
      }
      parsed.summary_fields = Some( v.clone() );
    }
  }
  if parsed.journal.is_none()
  {
    if let Some( v ) = &config.journal
    {
      if !matches!( v.as_str(), "full" | "meta" | "off" )
      {
        return Err( Error::msg( format!(
          "config file: invalid journal '{v}' — expected: full, meta, off"
        ) ) );
      }
      parsed.journal = Some( v.clone() );
    }
  }
  if parsed.journal_dir.is_none() { parsed.journal_dir.clone_from( &config.journal_dir ); }
  if !parsed.quiet { parsed.quiet = config.quiet; }
  if !parsed.no_chrome { parsed.no_chrome = config.no_chrome; }
  if !parsed.no_persist { parsed.no_persist = config.no_persist; }
  if !parsed.no_compact_window { parsed.no_compact_window = config.no_compact_window; }
  if parsed.allowed_tools.is_none() { parsed.allowed_tools.clone_from( &config.allowed_tools ); }
  if parsed.disallowed_tools.is_none() { parsed.disallowed_tools.clone_from( &config.disallowed_tools ); }
  if parsed.max_budget_usd.is_none() { parsed.max_budget_usd.clone_from( &config.max_budget_usd ); }
  if parsed.fallback_model.is_none() { parsed.fallback_model.clone_from( &config.fallback_model ); }
  Ok( () )
}

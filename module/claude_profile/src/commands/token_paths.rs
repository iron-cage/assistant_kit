//! `.token.status` and `.paths` command handlers.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use crate::output::{ OutputFormat, OutputOptions, json_escape };
use super::cmd_args::io_err_to_error_data;
use super::cmd_context::{ require_claude_paths, require_credential_store };
use claude_profile_core::account::trace_ts;

/// `.token.status` — show active OAuth token expiry classification.
///
/// **CRITICAL:** Uses `status_with_threshold()`, NEVER bare function that
/// matches the responsibility test grep pattern. See P1 in the plan.
///
/// # Errors
///
/// Returns `ErrorData` if HOME is unset, credentials are missing,
/// or the `expiresAt` field is unparseable.
#[ inline ]
pub fn token_status_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts  = OutputOptions::from_cmd( &cmd )?;
  if opts.is_table()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "format::table is only supported by .accounts".to_string(),
    ) );
  }
  let trace = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  let paths = require_claude_paths()?;
  if trace { eprintln!( "{}token.status  reading {}", trace_ts(), paths.credentials_file().display() ) }

  let threshold_secs = match cmd.arguments.get( "threshold" )
  {
    Some( Value::Integer( n ) ) => u64::try_from( *n ).unwrap_or( crate::token::WARNING_THRESHOLD_SECS ),
    _ => crate::token::WARNING_THRESHOLD_SECS,
  };

  let token_result = crate::token::status_with_threshold( threshold_secs )
    .map_err( |e| io_err_to_error_data( &e, "token status" ) )?;

  let content = match opts.format
  {
    OutputFormat::Json =>
    {
      match &token_result
      {
        crate::token::TokenStatus::Valid { expires_in } =>
          format!( "{{\"status\":\"valid\",\"expires_in_secs\":{}}}\n", expires_in.as_secs() ),
        crate::token::TokenStatus::ExpiringSoon { expires_in } =>
          format!( "{{\"status\":\"expiring_soon\",\"expires_in_secs\":{}}}\n", expires_in.as_secs() ),
        crate::token::TokenStatus::Expired =>
          "{\"status\":\"expired\"}\n".to_string(),
      }
    }
    OutputFormat::Text =>
    {
      match &token_result
      {
        crate::token::TokenStatus::Valid { expires_in } =>
          format!( "valid — {}m remaining\n", expires_in.as_secs() / 60 ),
        crate::token::TokenStatus::ExpiringSoon { expires_in } =>
          format!( "expiring soon — {}m remaining\n", expires_in.as_secs() / 60 ),
        crate::token::TokenStatus::Expired =>
          "expired\n".to_string(),
      }
    }
    // Table rejected above via is_table() guard; unreachable.
    OutputFormat::Table => String::new(),
  };

  Ok( OutputData::new( content, "text" ) )
}

/// `.paths` — show all resolved `~/.claude/` canonical file paths.
///
/// # Errors
///
/// Returns `ErrorData` if HOME is unset or empty.
#[ inline ]
pub fn paths_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let trace = crate::output::parse_int_flag( &cmd, "trace", 0 )? != 0;
  if let Some( Value::String( field ) ) = cmd.arguments.get( "field" )
  {
    if !field.is_empty()
    {
      let paths            = require_claude_paths()?;
      let credential_store = require_credential_store()?;
      let raw = match field.as_str()
      {
        "base"             => paths.base().display().to_string(),
        "credentials"      => paths.credentials_file().display().to_string(),
        "credential_store" => credential_store.display().to_string(),
        "projects"         => paths.projects_dir().display().to_string(),
        "stats"            => paths.stats_file().display().to_string(),
        "settings"         => paths.settings_file().display().to_string(),
        "session_env"      => paths.session_env_dir().display().to_string(),
        "sessions"         => paths.sessions_dir().display().to_string(),
        other =>
        {
          return Err( ErrorData::new(
            ErrorCode::ArgumentTypeMismatch,
            format!( "unknown field '{other}'; valid: base, credentials, credential_store, projects, stats, settings, session_env, sessions" ),
          ) );
        }
      };
      return Ok( OutputData::new( format!( "{raw}\n" ), "text" ) );
    }
  }
  let opts             = OutputOptions::from_cmd( &cmd )?;
  if opts.is_table()
  {
    return Err( ErrorData::new(
      ErrorCode::ArgumentTypeMismatch,
      "format::table is only supported by .accounts".to_string(),
    ) );
  }
  let paths            = require_claude_paths()?;
  if trace { eprintln!( "{}paths  base: {}", trace_ts(), paths.base().display() ) }
  let credential_store = require_credential_store()?;

  let content = match opts.format
  {
    OutputFormat::Json =>
    {
      format!(
        concat!(
          "{{\"base\":\"{}\",",
          "\"credentials\":\"{}\",",
          "\"credential_store\":\"{}\",",
          "\"projects\":\"{}\",",
          "\"stats\":\"{}\",",
          "\"settings\":\"{}\",",
          "\"session_env\":\"{}\",",
          "\"sessions\":\"{}\"}}\n",
        ),
        json_escape( &paths.base().display().to_string() ),
        json_escape( &paths.credentials_file().display().to_string() ),
        json_escape( &credential_store.display().to_string() ),
        json_escape( &paths.projects_dir().display().to_string() ),
        json_escape( &paths.stats_file().display().to_string() ),
        json_escape( &paths.settings_file().display().to_string() ),
        json_escape( &paths.session_env_dir().display().to_string() ),
        json_escape( &paths.sessions_dir().display().to_string() ),
      )
    }
    OutputFormat::Text =>
    {
      format!(
        "credentials:      {}\ncredential_store: {}\nprojects:         {}\nstats:            {}\nsettings:         {}\nsession-env:      {}\nsessions:         {}\n",
        paths.credentials_file().display(),
        credential_store.display(),
        paths.projects_dir().display(),
        paths.stats_file().display(),
        paths.settings_file().display(),
        paths.session_env_dir().display(),
        paths.sessions_dir().display(),
      )
    }
    // Table rejected above via is_table() guard; unreachable.
    OutputFormat::Table => String::new(),
  };

  Ok( OutputData::new( content, "text" ) )
}

//! `.model` command handler — read or write `~/.claude/settings.json` model key.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use crate::output::{ OutputFormat, OutputOptions };
use super::cmd_context::require_claude_paths;
use crate::usage::map_model_shorthand;

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.model` — read or write the Claude Code session model in `~/.claude/settings.json`.
///
/// **Get mode** (no `set::` param): prints `model: VALUE` or `model: (unset)`. Exit 0.
/// `format::json` returns `{"model":"VALUE"}` or `{"model":null}`.
///
/// **Set mode** (`set::VALUE`): validates the shorthand, writes via `set_session_model()`,
/// prints `model set: VALUE`. Exit 0. Unknown value → exit 1.
///
/// # Errors
///
/// Returns `ErrorData` if HOME is unset (exit 2) or `set::` value is unknown (exit 1).
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
  let paths = require_claude_paths()?;

  let set_val = match cmd.arguments.get( "set" )
  {
    Some( Value::String( s ) ) => Some( s.clone() ),
    _                          => None,
  };

  if let Some( ref val ) = set_val
  {
    // Set mode: validate shorthand, write, report.
    let model_id = map_model_shorthand( val )
      .ok_or_else( || ErrorData::new(
        ErrorCode::ArgumentTypeMismatch,
        format!( "set:: must be one of: opus, sonnet, haiku, default; got {val:?}" ),
      ) )?;
    claude_profile_core::account::set_session_model( &paths, model_id );
    Ok( OutputData::new( format!( "model set: {val}\n" ), "text" ) )
  }
  else
  {
    // Get mode: read and format.
    let model = claude_profile_core::account::get_session_model( &paths );
    let text = match opts.format
    {
      OutputFormat::Json =>
      {
        match &model
        {
          Some( m ) => format!( "{{\"model\":\"{m}\"}}\n" ),
          None      => "{\"model\":null}\n".to_string(),
        }
      }
      OutputFormat::Text | OutputFormat::Table =>
      {
        match &model
        {
          Some( m ) => format!( "model: {m}\n" ),
          None      => "model: (unset)\n".to_string(),
        }
      }
    };
    Ok( OutputData::new( text, "text" ) )
  }
}

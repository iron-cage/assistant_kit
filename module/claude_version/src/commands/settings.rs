//! `.settings.*` — read and write Claude Code settings.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;

use crate::output::{ OutputFormat, OutputOptions, json_escape };
use claude_version_core::settings_io::{ StoredAs, get_setting, infer_type, read_all_settings, set_setting };

/// `.settings.show` — print all entries from `~/.claude/settings.json`.
///
/// # Errors
///
/// Returns `Err(InternalError)` when HOME is missing or settings unreadable.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn settings_show_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let paths = super::require_claude_paths()?;
  let pairs = read_all_settings( &paths.settings_file() ).map_err( | e |
    ErrorData::new( ErrorCode::InternalError, format!( "failed to read settings: {e}" ) )
  )?;

  let content = match opts.format
  {
    OutputFormat::Json =>
    {
      let body : String = pairs.iter()
      .map( | ( k, v ) |
      {
        let json_val = match infer_type( v )
        {
          StoredAs::Bool | StoredAs::Number | StoredAs::Raw => v.clone(),
          StoredAs::Str => format!( "\"{}\"", json_escape( v ) ),
        };
        format!( "  \"{}\":{json_val}", json_escape( k ) )
      } )
      .collect::< Vec< _ > >()
      .join( ",\n" );
      if body.is_empty()
      {
        "{}\n".to_string()
      }
      else
      {
        format!( "{{\n{body}\n}}\n" )
      }
    }
    OutputFormat::Text =>
    {
      let lines : Vec< String > = pairs.iter().map( | ( k, v ) |
        match opts.verbosity
        {
          0 => format!( "{k}={v}" ),
          _ => format!( "{k}: {v}" ),
        }
      ).collect();
      if lines.is_empty() { String::new() } else { format!( "{}\n", lines.join( "\n" ) ) }
    }
  };

  Ok( OutputData::new( content, "text" ) )
}

/// `.settings.get` — read and print the value of a single setting by key.
///
/// # Errors
///
/// Returns `Err(ArgumentMissing)` when `key::` is missing.
/// Returns `Err(InternalError)` when HOME is missing or key not found.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn settings_get_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let key   = super::require_nonempty_string_arg( &cmd, "key" )?;
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let paths = super::require_claude_paths()?;

  let value = get_setting( &paths.settings_file(), &key )
  .map_err( | e |
    ErrorData::new( ErrorCode::InternalError, format!( "failed to read settings: {e}" ) )
  )?
  .ok_or_else( || ErrorData::new(
    ErrorCode::InternalError,
    format!( "key '{key}' not found in settings" ),
  ) )?;

  let content = match ( opts.format, opts.verbosity )
  {
    ( OutputFormat::Json, _ ) =>
    {
      let k = json_escape( &key );
      let json_val = match infer_type( &value )
      {
        StoredAs::Bool | StoredAs::Number | StoredAs::Raw => value.clone(),
        StoredAs::Str => format!( "\"{}\"", json_escape( &value ) ),
      };
      format!( "{{\"key\":\"{k}\",\"value\":{json_val}}}\n" )
    }
    ( OutputFormat::Text, 0 ) => format!( "{value}\n" ),
    ( OutputFormat::Text, _ ) => format!( "{key}: {value}\n" ),
  };

  Ok( OutputData::new( content, "text" ) )
}

/// `.settings.set` — write a new value for a single setting atomically.
///
/// # Errors
///
/// Returns `Err(ArgumentMissing)` when `key::` or `value::` is missing or empty.
/// Returns `Err(InternalError)` when HOME is missing or write fails.
///
/// Fix(issue-settings-set-empty-value): `value::` (empty) was accepted and stored `""` in JSON.
/// Root cause: used `require_string_arg` (allows empty) instead of `require_nonempty_string_arg`
///   for the `value::` parameter, silently bypassing the FR-04 empty-value rejection.
/// Pitfall: `cm .settings.set key::k value::` appeared to succeed but wrote a meaningless
///   empty-string entry — indistinguishable from "key not set" when read back via `.settings.get`.
#[ allow( clippy::needless_pass_by_value, clippy::missing_inline_in_public_items ) ]
pub fn settings_set_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let key   = super::require_nonempty_string_arg( &cmd, "key" )?;
  let value = super::require_nonempty_string_arg( &cmd, "value" )?;

  let stored_as = infer_type( &value );

  if super::is_dry( &cmd )
  {
    let type_label = match stored_as
    {
      StoredAs::Bool   => "bool",
      StoredAs::Number => "number",
      StoredAs::Str    => "string",
      StoredAs::Raw    => "object",
    };
    return Ok( OutputData::new(
      format!( "[dry-run] would set {key} = {value}  ({type_label})\n" ),
      "text",
    ) );
  }

  let paths = super::require_claude_paths()?;
  let settings_file = paths.settings_file();

  if let Some( parent ) = settings_file.parent()
  {
    std::fs::create_dir_all( parent ).map_err( | e |
      ErrorData::new( ErrorCode::InternalError, format!( "failed to create settings directory: {e}" ) )
    )?;
  }

  set_setting( &settings_file, &key, &value ).map_err( | e |
    ErrorData::new( ErrorCode::InternalError, format!( "failed to write settings: {e}" ) )
  )?;

  Ok( OutputData::new( format!( "set {key} = {value}\n" ), "text" ) )
}

//! `.config` — 4-layer configuration resolution, show, get, set, and unset.

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use crate::output::{ OutputFormat, OutputOptions, json_escape };
use claude_version_core::config_catalog::catalog;
use claude_version_core::config_resolve::{ resolve, resolve_all, ResolvedValue };
use claude_version_core::settings_io::{ StoredAs, infer_type, remove_setting, set_setting };

/// `.config` — show, get, set, or unset Claude Code configuration settings
/// with 4-layer effective-value resolution.
///
/// # Mode dispatch
///
/// | `key::` | `value::` | `unset::` | Mode    |
/// |-------|---------|---------|---------|
/// | absent | absent | absent  | show-all |
/// | present | absent | absent | get      |
/// | present | present | absent | set      |
/// | present | absent | true   | unset    |
///
/// # Errors
///
/// Returns `Err(ArgumentMissing)` for invalid parameter combinations.
/// Returns `Err(ArgumentTypeMismatch)` for an unrecognised `scope::` value.
/// Returns `Err(InternalError)` when HOME is missing or file I/O fails.
///
/// # Panics
///
/// Panics only on internal invariant violations (unreachable code paths that
/// indicate a programming error, not user input error).
#[ allow(
  clippy::needless_pass_by_value,
  clippy::missing_inline_in_public_items,
  clippy::too_many_lines,
) ]
pub fn config_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  // Reject explicitly-provided but empty key:: value before treating it as absent.
  if let Some( Value::String( s ) ) = cmd.arguments.get( "key" )
  {
    if s.is_empty()
    {
      return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch,
        "key:: value cannot be empty".to_string() ) );
    }
  }
  let key_opt = match cmd.arguments.get( "key" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    _                                            => None,
  };
  let value_opt = match cmd.arguments.get( "value" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    _                                            => None,
  };
  let is_unset  = matches!( cmd.arguments.get( "unset" ), Some( Value::Boolean( true ) ) );
  let scope_explicit = cmd.arguments.contains_key( "scope" );
  let scope_str = match cmd.arguments.get( "scope" )
  {
    Some( Value::String( s ) ) => s.clone(),
    _                          => "user".to_string(),
  };
  let opts = OutputOptions::from_cmd( &cmd )?;

  // ── Validate invalid parameter combinations ───────────────────────────────
  if value_opt.is_some() && key_opt.is_none()
  {
    return Err( ErrorData::new( ErrorCode::ArgumentMissing,
      "key:: is required when value:: is provided".to_string() ) );
  }
  if is_unset && key_opt.is_none()
  {
    return Err( ErrorData::new( ErrorCode::ArgumentMissing,
      "key:: is required when unset::1 is provided".to_string() ) );
  }
  if value_opt.is_some() && is_unset
  {
    return Err( ErrorData::new( ErrorCode::ArgumentMissing,
      "value:: and unset::1 are mutually exclusive".to_string() ) );
  }
  // scope:: is only meaningful for write operations (set or unset).
  // Passing scope:: in show-all or get mode is a user error.
  let is_write = key_opt.is_some() && ( value_opt.is_some() || is_unset );
  if scope_explicit && !is_write
  {
    return Err( ErrorData::new( ErrorCode::ArgumentMissing,
      "scope:: applies to write operations only; provide key:: with value:: or unset::1".to_string() ) );
  }

  // ── Validate scope value ──────────────────────────────────────────────────
  match scope_str.as_str()
  {
    "user" | "project" => {}
    other => return Err( ErrorData::new( ErrorCode::ArgumentTypeMismatch,
      format!( "unknown scope '{other}': expected user or project" ) ) ),
  }
  let scope_explicit = cmd.arguments.contains_key( "scope" );
  let is_write       = key_opt.is_some() && ( value_opt.is_some() || is_unset );
  if scope_explicit && !is_write
  {
    return Err( ErrorData::new( ErrorCode::ArgumentMissing,
      "scope:: applies to write operations only (key:: + value:: or key:: + unset::1)".to_string() ) );
  }

  // ── Mode dispatch ─────────────────────────────────────────────────────────
  match ( key_opt.as_deref(), value_opt.as_deref(), is_unset )
  {
    ( None, None, false ) =>
    {
      let ( home_dir, cwd ) = config_resolve_context()?;
      let all     = resolve_all( &home_dir, &cwd, catalog() );
      let content = render_config_show_all( &all, &opts );
      Ok( OutputData::new( content, "text" ) )
    }

    ( Some( key ), None, false ) =>
    {
      let ( home_dir, cwd ) = config_resolve_context()?;
      let resolved = resolve( key, &home_dir, &cwd, catalog() );
      let content  = render_config_get( key, &resolved, &opts );
      Ok( OutputData::new( content, "text" ) )
    }

    ( Some( key ), Some( value ), false ) =>
    {
      // Set mode: write the value to the target scope.
      let stored_as = infer_type( value );

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

      let target = config_scope_target( &scope_str )?;
      if let Some( parent ) = target.parent()
      {
        std::fs::create_dir_all( parent ).map_err( | e |
          ErrorData::new( ErrorCode::InternalError,
            format!( "failed to create settings directory: {e}" ) )
        )?;
      }
      set_setting( &target, key, value ).map_err( | e |
        ErrorData::new( ErrorCode::InternalError, format!( "failed to write settings: {e}" ) )
      )?;
      Ok( OutputData::new( format!( "set {key} = {value}\n" ), "text" ) )
    }

    ( Some( key ), None, true ) =>
    {
      // Unset mode: remove the key from the target scope.
      if super::is_dry( &cmd )
      {
        return Ok( OutputData::new(
          format!( "[dry-run] would unset {key}\n" ),
          "text",
        ) );
      }

      let target = config_scope_target( &scope_str )?;
      remove_setting( &target, key ).map_err( | e |
        ErrorData::new( ErrorCode::InternalError, format!( "failed to unset setting: {e}" ) )
      )?;
      Ok( OutputData::new( format!( "unset {key}\n" ), "text" ) )
    }

    _ => unreachable!( "all invalid combinations rejected above" ),
  }
}

/// Resolve home directory and cwd for config resolution operations.
///
/// Returns `(home_dir, cwd)`. Falls back to home dir if `current_dir` fails.
fn config_resolve_context() -> Result< ( std::path::PathBuf, std::path::PathBuf ), ErrorData >
{
  let paths    = super::require_claude_paths()?;
  let home_dir = paths.base().parent()
    .expect( "ClaudePaths base always has HOME as parent" )
    .to_path_buf();
  let cwd = std::env::current_dir().unwrap_or_else( | _ | home_dir.clone() );
  Ok( ( home_dir, cwd ) )
}

/// Resolve the settings file path for a given scope string.
///
/// `"user"` → `~/.claude/settings.json`; `"project"` → `{cwd}/.claude/settings.json`.
fn config_scope_target( scope : &str ) -> Result< std::path::PathBuf, ErrorData >
{
  match scope
  {
    "user" =>
    {
      let paths = super::require_claude_paths()?;
      Ok( paths.settings_file() )
    }
    "project" =>
    {
      let cwd = std::env::current_dir().map_err( | e |
        ErrorData::new( ErrorCode::InternalError, format!( "failed to get current directory: {e}" ) )
      )?;
      Ok( cwd.join( ".claude" ).join( "settings.json" ) )
    }
    // Caller already validated scope before this point.
    _ => unreachable!( "scope validated before config_scope_target call" ),
  }
}

/// Render the show-all output for `.config` (no `key::` param).
fn render_config_show_all(
  all  : &[ ( String, ResolvedValue ) ],
  opts : &OutputOptions,
) -> String
{
  match opts.format
  {
    OutputFormat::Json =>
    {
      let entries : Vec< String > = all.iter().map( | ( k, rv ) |
      {
        let src      = rv.source.to_string();
        let val_json = config_value_to_json( rv.value.as_deref() );
        format!( "  \"{}\": {{\"value\": {val_json}, \"source\": \"{src}\"}}",
          json_escape( k ) )
      } ).collect();
      if entries.is_empty() { "{}\n".to_string() }
      else { format!( "{{\n{}\n}}\n", entries.join( ",\n" ) ) }
    }
    OutputFormat::Text =>
    {
      let lines : Vec< String > = all.iter().map( | ( k, rv ) |
      {
        match &rv.value
        {
          None    => format!( "{k}: ({})", rv.source ),
          Some( v ) => format!( "{k}: {v} ({})", rv.source ),
        }
      } ).collect();
      if lines.is_empty() { String::new() }
      else { format!( "{}\n", lines.join( "\n" ) ) }
    }
  }
}

/// Render the get output for `.config key::K`.
fn render_config_get(
  key  : &str,
  rv   : &ResolvedValue,
  opts : &OutputOptions,
) -> String
{
  match opts.format
  {
    OutputFormat::Json =>
    {
      let src      = rv.source.to_string();
      let val_json = config_value_to_json( rv.value.as_deref() );
      format!( "{{\"key\": \"{}\", \"value\": {val_json}, \"source\": \"{src}\"}}\n",
        json_escape( key ) )
    }
    OutputFormat::Text =>
    {
      match ( opts.verbosity, &rv.value )
      {
        ( 0, None )       => "(absent)\n".to_string(),
        ( 0, Some( v ) )  => format!( "{v} ({})\n", rv.source ),
        ( _, None )       => format!( "{key}: (absent)\n" ),
        ( _, Some( v ) )  => format!( "{key}: {v} ({})\n", rv.source ),
      }
    }
  }
}

/// Serialize an optional settings value to a JSON value literal.
///
/// `None` → `null`; bool/number/raw → bare; strings → quoted and escaped.
fn config_value_to_json( raw : Option< &str > ) -> String
{
  match raw
  {
    None    => "null".to_string(),
    Some( v ) => match infer_type( v )
    {
      StoredAs::Bool | StoredAs::Number | StoredAs::Raw => v.to_string(),
      StoredAs::Str  => format!( "\"{}\"", json_escape( v ) ),
    },
  }
}

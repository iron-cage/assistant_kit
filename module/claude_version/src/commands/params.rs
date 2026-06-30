//! `.params` — inspect the observable Claude Code parameter catalog.
//!
//! Shows each parameter's CLI flag, env-var, and config-key forms alongside
//! the current observable values. Read-only: never modifies state.
//!
//! # Mode dispatch
//!
//! | `key::` | Mode |
//! |---------|------|
//! | absent  | show-all (optionally filtered by `kind::`) |
//! | present | single-param deep-dive |

use core::fmt::Write;
use std::path::Path;

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;

use crate::output::{ OutputFormat, OutputOptions, json_escape };
use claude_version_core::params_catalog::{ ParamDef, lookup, params_catalog };
use claude_version_core::settings_io::get_setting;

/// `.params` — show params catalog with forms, current values, and source.
///
/// # Errors
///
/// Returns `Err(ArgumentTypeMismatch)` for an unrecognised `kind::` or `format::` value (exit 1).
/// Returns `Err(InternalError)` when `key::` is specified but not in the catalog (exit 2).
#[ allow(
  clippy::needless_pass_by_value,
  clippy::missing_inline_in_public_items,
) ]
pub fn params_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let key_opt = match cmd.arguments.get( "key" )
  {
    Some( Value::String( s ) ) if !s.is_empty() => Some( s.clone() ),
    _                                            => None,
  };

  // Empty kind:: string is passed through so validation catches it (exit 1), not silently ignored.
  let kind_opt = match cmd.arguments.get( "kind" )
  {
    Some( Value::String( s ) ) => Some( s.clone() ),
    _                          => None,
  };

  // Validate kind:: early so it exits 1, not conflated with format errors.
  if let Some( ref k ) = kind_opt
  {
    match k.as_str()
    {
      "config" | "env" => {}
      other =>
      {
        return Err( ErrorData::new(
          ErrorCode::ArgumentTypeMismatch,
          format!( "unknown kind '{other}': expected config or env" ),
        ) );
      }
    }
  }

  // format:: and verbosity — exits 1 for unknown format.
  let opts = OutputOptions::from_cmd( &cmd )?;

  // Resolve user settings path; soft fail — .params works even without HOME.
  let user_settings = std::env::var( "HOME" )
    .ok()
    .filter( | h | !h.is_empty() )
    .map( | h | std::path::PathBuf::from( h ).join( ".claude" ).join( "settings.json" ) );

  match key_opt.as_deref()
  {
    None =>
    {
      let catalog  = params_catalog();
      let filtered : Vec< &ParamDef > = catalog.iter().filter( | p |
      {
        match kind_opt.as_deref()
        {
          Some( "config" ) => p.has_config(),
          Some( "env" )    => p.has_env(),
          _                => true,
        }
      } ).collect();

      let content = render_show_all( &filtered, &opts, user_settings.as_deref() );
      Ok( OutputData::new( content, "text" ) )
    }

    Some( key ) =>
    {
      let param = lookup( key ).ok_or_else( || ErrorData::new(
        ErrorCode::InternalError,
        format!( "unknown param key '{key}': not in params catalog" ),
      ) )?;

      let content = render_single_param( param, &opts, user_settings.as_deref() );
      Ok( OutputData::new( content, "text" ) )
    }
  }
}

// ── Catalog value resolution ──────────────────────────────────────────────────

/// Read the env var for this param (None if absent or empty).
fn read_env_val( param : &ParamDef ) -> Option< String >
{
  param.env_var.and_then( | ev |
  {
    match std::env::var( ev )
    {
      Ok( v ) if !v.is_empty() => Some( v ),
      _                        => None,
    }
  } )
}

/// Read the settings.json value for the param's config key (None if absent).
fn read_config_val( param : &ParamDef, user_settings : Option< &Path > ) -> Option< String >
{
  let ck   = param.config_key?;
  let path = user_settings?;
  get_setting( path, ck ).ok().flatten().filter( | v | !v.is_empty() )
}

/// Resolve (`effective_value`, `source_tag`) for a param using env → config → default.
///
/// Returns source tag: `"env"`, `"user"`, `"default"`, or `"absent"`.
fn resolve_effective( param : &ParamDef, user_settings : Option< &Path > ) -> ( String, &'static str )
{
  if let Some( v ) = read_env_val( param )
  {
    return ( v, "env" );
  }
  if let Some( v ) = read_config_val( param, user_settings )
  {
    return ( v, "user" );
  }
  if let Some( d ) = param.default
  {
    return ( d.to_string(), "default" );
  }
  ( "absent".to_string(), "absent" )
}

// ── Forms string builder ──────────────────────────────────────────────────────

/// Build the human-readable forms string (e.g. `"CLI --model  |  env CLAUDE_MODEL  |  config model"`).
fn build_forms( param : &ParamDef ) -> String
{
  let mut parts : Vec< String > = Vec::new();
  if let Some( cli ) = param.cli_flag   { parts.push( format!( "CLI {cli}"    ) ); }
  if let Some( ev  ) = param.env_var    { parts.push( format!( "env {ev}"     ) ); }
  if let Some( ck  ) = param.config_key { parts.push( format!( "config {ck}" ) ); }
  parts.join( "  |  " )
}

// ── Show-all rendering ────────────────────────────────────────────────────────

fn render_show_all(
  params        : &[ &ParamDef ],
  opts          : &OutputOptions,
  user_settings : Option< &Path >,
) -> String
{
  match opts.format
  {
    OutputFormat::Json => render_show_all_json( params, user_settings ),
    OutputFormat::Text => render_show_all_text( params, opts.verbosity, user_settings ),
  }
}

fn render_show_all_text(
  params        : &[ &ParamDef ],
  verbosity     : u8,
  user_settings : Option< &Path >,
) -> String
{
  let mut out = String::new();
  for param in params
  {
    if verbosity == 0
    {
      // Compact: one line, value only, no annotations.
      let ( val, _ ) = resolve_effective( param, user_settings );
      let _ = writeln!( out, "{}: {val}", param.name );
    }
    else
    {
      // Block format: name (non-indented), then forms + annotated value.
      let forms = build_forms( param );
      out.push_str( param.name );
      out.push( '\n' );
      let _ = writeln!( out, "  Forms:   {forms}" );

      if param.is_cli_only()
      {
        out.push_str( "  Value:   (CLI-only)\n" );
      }
      else
      {
        let ( val, source ) = resolve_effective( param, user_settings );
        let display = match source
        {
          "absent"  => "(absent)".to_string(),
          src       => format!( "{val} ({src})" ),
        };
        let _ = writeln!( out, "  Value:   {display}" );
      }
      out.push( '\n' );
    }
  }
  out
}

fn render_show_all_json(
  params        : &[ &ParamDef ],
  user_settings : Option< &Path >,
) -> String
{
  let entries : Vec< String > = params.iter().map( | p |
  {
    let ( eff_val, source ) = resolve_effective( p, user_settings );
    let cli_json = opt_json_str( p.cli_flag );
    let env_json = opt_json_str( p.env_var );
    let cfg_json = opt_json_str( p.config_key );
    let val_json = if source == "absent"
    {
      "null".to_string()
    }
    else
    {
      format!( "\"{}\"", json_escape( &eff_val ) )
    };
    format!(
      "  {{\"name\": \"{}\", \"cli\": {cli_json}, \"env\": {env_json}, \"config\": {cfg_json}, \"effective_value\": {val_json}, \"source\": \"{}\"}}",
      json_escape( p.name ), source
    )
  } ).collect();

  if entries.is_empty()
  {
    "[]\n".to_string()
  }
  else
  {
    format!( "[\n{}\n]\n", entries.join( ",\n" ) )
  }
}

// ── Single-param rendering ────────────────────────────────────────────────────

fn render_single_param(
  param         : &ParamDef,
  opts          : &OutputOptions,
  user_settings : Option< &Path >,
) -> String
{
  match opts.format
  {
    OutputFormat::Json => render_single_param_json( param, user_settings ),
    OutputFormat::Text => render_single_param_text( param, user_settings ),
  }
}

fn render_single_param_text( param : &ParamDef, user_settings : Option< &Path > ) -> String
{
  let mut out   = String::new();
  let forms_str = build_forms( param );

  out.push_str( param.name );
  out.push( '\n' );
  let _ = writeln!( out, "  Forms:   {forms_str}" );

  if param.is_cli_only()
  {
    out.push_str(
      "  Value:   (CLI-only \u{2014} unobservable from clv; passed directly to claude at invocation)\n"
    );
    if let Some( d ) = param.default
    {
      let _ = writeln!( out, "  Default: {d}" );
    }
  }
  else
  {
    // Env var line
    if let Some( ev ) = param.env_var
    {
      match read_env_val( param )
      {
        Some( ref v ) => { let _ = writeln!( out, "  Env:     {ev} \u{2192} {v} (set)" ); }
        None          => { let _ = writeln!( out, "  Env:     {ev} \u{2192} unset" ); }
      }
    }

    // Config key line
    if let Some( ck ) = param.config_key
    {
      match read_config_val( param, user_settings )
      {
        Some( ref v ) => { let _ = writeln!( out, "  Config:  {ck} = {v}" ); }
        None          => { let _ = writeln!( out, "  Config:  {ck} = (absent in user config)" ); }
      }
    }

    // Default line
    if let Some( d ) = param.default
    {
      let _ = writeln!( out, "  Default: {d}" );
    }

    // Effective value (only for params with at least one observable form)
    let ( eff_val, source ) = resolve_effective( param, user_settings );
    let _ = writeln!( out, "  {}", "\u{2500}".repeat( 50 ) );
    let _ = writeln!( out, "  Effective: {eff_val} ({source})" );
  }

  out
}

fn render_single_param_json( param : &ParamDef, user_settings : Option< &Path > ) -> String
{
  let ( eff_val, source ) = resolve_effective( param, user_settings );
  let cli_json    = opt_json_str( param.cli_flag );
  let env_json    = opt_json_str( param.env_var );
  let cfg_json    = opt_json_str( param.config_key );
  let def_json    = opt_json_str( param.default );
  let val_json    = if source == "absent"
  {
    "null".to_string()
  }
  else
  {
    format!( "\"{}\"", json_escape( &eff_val ) )
  };

  format!(
    "{{\"name\": \"{}\", \"cli\": {cli_json}, \"env\": {env_json}, \"config\": {cfg_json}, \
     \"default\": {def_json}, \"effective_value\": {val_json}, \"source\": \"{source}\"}}\n",
    json_escape( param.name )
  )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn opt_json_str( s : Option< &str > ) -> String
{
  match s
  {
    None    => "null".to_string(),
    Some( v ) => format!( "\"{}\"", json_escape( v ) ),
  }
}

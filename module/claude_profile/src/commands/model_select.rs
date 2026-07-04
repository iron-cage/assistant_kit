//! `.model.select` command handler — pin or clear the subprocess model preference.
//!
//! Manages `subprocess_model` in `~/.clr/prefs.json` (Schema 008).
//! Three modes: get (no `id::`, no `reset::`), set (`id::VALUE`), reset (`reset::1`).

use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;
use unilang::types::Value;
use crate::output::{ OutputFormat, OutputOptions };

const PREFS_KEY : &str = "subprocess_model";

// ── Handler ───────────────────────────────────────────────────────────────────

/// `.model.select` — get or set the clr subprocess model preference.
///
/// **Get mode** (no `id::`, no `reset::1`): prints `model.select: VALUE` or
/// `model.select: (unset)`. Exit 0.
///
/// **Set mode** (`id::VALUE`): writes `subprocess_model` to `~/.clr/prefs.json`,
/// creates the file and parent directory when absent. Prints
/// `model.select: VALUE (pinned)`. Exit 0.
///
/// **Reset mode** (`reset::1`): removes `subprocess_model` key; preserves other
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

  let prefs_path = resolve_prefs_path()?;

  if let Some( ref model_id ) = id_val
  {
    // Set mode
    set_prefs_model( &prefs_path, model_id )?;
    Ok( OutputData::new( format!( "model.select: {model_id} (pinned)\n" ), "text" ) )
  }
  else if reset_val
  {
    // Reset mode
    remove_prefs_model( &prefs_path )?;
    Ok( OutputData::new( "model.select: (reset to default)\n".to_string(), "text" ) )
  }
  else
  {
    // Get mode
    let current = read_prefs_model( &prefs_path );
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

/// Resolve `~/.clr/prefs.json` path.
fn resolve_prefs_path() -> Result< std::path::PathBuf, ErrorData >
{
  let home = std::env::var( "HOME" )
    .map_err( |_| ErrorData::new( ErrorCode::InternalError, "HOME environment variable not set".to_string() ) )?;
  Ok( std::path::PathBuf::from( home ).join( ".clr" ).join( "prefs.json" ) )
}

/// Read `subprocess_model` from `prefs.json`; `None` when absent or file missing.
fn read_prefs_model( path : &std::path::Path ) -> Option< String >
{
  let content = std::fs::read_to_string( path ).ok()?;
  extract_json_str( &content, PREFS_KEY )
}

/// Write or update `subprocess_model` in `prefs.json`, creating dir + file as needed.
fn set_prefs_model( path : &std::path::Path, model_id : &str ) -> Result< (), ErrorData >
{
  let existing = std::fs::read_to_string( path ).unwrap_or_else( |_| "{}".to_string() );
  let updated  = upsert_json_str( &existing, PREFS_KEY, model_id );
  if let Some( parent ) = path.parent()
  {
    std::fs::create_dir_all( parent ).map_err( | e | ErrorData::new(
      ErrorCode::InternalError,
      format!( "failed to create .clr directory: {e}" ),
    ) )?;
  }
  std::fs::write( path, updated ).map_err( | e | ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to write prefs.json: {e}" ),
  ) )
}

/// Remove `subprocess_model` from `prefs.json`; no-op if file absent.
fn remove_prefs_model( path : &std::path::Path ) -> Result< (), ErrorData >
{
  let Ok( content ) = std::fs::read_to_string( path ) else
  {
    return Ok( () ); // file absent — idempotent
  };
  let updated = remove_json_key( &content, PREFS_KEY );
  std::fs::write( path, updated ).map_err( | e | ErrorData::new(
    ErrorCode::InternalError,
    format!( "failed to write prefs.json: {e}" ),
  ) )
}

// ── JSON manipulation (no serde_json — flat string-valued object only) ────────

/// Find the byte position of `needle` that occurs at a JSON key boundary.
///
/// A key boundary is a position where the opening `"` of the key is immediately
/// preceded (ignoring trailing whitespace) by `{` or `,`. This prevents false
/// matches when the same pattern appears inside a string value.
fn find_json_key( json : &str, needle : &str ) -> Option< usize >
{
  let mut start = 0usize;
  loop
  {
    let rel = json[ start.. ].find( needle )?;
    let abs = start + rel;
    if json[ ..abs ].trim_end().ends_with( ['{', ','] )
    {
      return Some( abs );
    }
    start = abs + needle.len();
    if start >= json.len() { return None; }
  }
}

/// Extract a JSON string value from a flat JSON object by key name.
///
/// Handles only string values (`"key":"value"` pairs). Returns `None` for
/// null, integer, or absent values.
fn extract_json_str( json : &str, key : &str ) -> Option< String >
{
  let needle   = format!( "\"{key}\":\"" );
  let key_pos  = find_json_key( json, &needle )?;
  let inner    = &json[ key_pos + needle.len() .. ];
  let end      = inner.find( '"' )?;
  let value    = &inner[ ..end ];
  if value.is_empty() { None } else { Some( value.to_string() ) }
}

/// Insert or replace a string-valued key in a flat JSON object.
///
/// If the key already exists with a quoted string value, replaces the value
/// in-place. If absent, appends before the closing `}`.
fn upsert_json_str( json : &str, key : &str, value : &str ) -> String
{
  let needle = format!( "\"{key}\":\"" );
  if let Some( key_pos ) = find_json_key( json, &needle )
  {
    let val_start = key_pos + needle.len();
    if let Some( rel_end ) = json[ val_start .. ].find( '"' )
    {
      let val_end = val_start + rel_end;
      return format!( "{}{value}{}", &json[ ..val_start ], &json[ val_end .. ] );
    }
  }
  // Key not found — insert before closing `}`
  let trimmed = json.trim();
  if let Some( close ) = trimmed.rfind( '}' )
  {
    let before  = trimmed[ ..close ].trim_end();
    // Check if the object body is empty (only whitespace/`{` before `}`)
    let body    = before.trim_start_matches( '{' ).trim();
    if body.is_empty()
    {
      return format!( "{{\"{key}\":\"{value}\"}}" );
    }
    return format!( "{before},\"{key}\":\"{value}\"}}" );
  }
  // Malformed JSON — create fresh
  format!( "{{\"{key}\":\"{value}\"}}" )
}

/// Remove a string-valued key from a flat JSON object.
///
/// Handles leading or trailing comma. If the key is absent, returns `json`
/// unchanged.
fn remove_json_key( json : &str, key : &str ) -> String
{
  let needle = format!( "\"{key}\":\"" );
  let Some( pos ) = find_json_key( json, &needle ) else { return json.to_string() };
  let val_start = pos + needle.len();
  let Some( rel_end ) = json[ val_start .. ].find( '"' ) else
  {
    return json.to_string();
  };
  let pair_end = val_start + rel_end + 1; // includes closing "

  let before = &json[ ..pos ];
  let after  = &json[ pair_end .. ];

  let before_trimmed = before.trim_end();
  let after_trimmed  = after.trim_start();

  if let Some( cut ) = before_trimmed.strip_suffix( ',' )
  {
    // Remove trailing comma from before: `{..,`  → `{..`
    format!( "{cut}{after}" )
  }
  else if let Some( stripped ) = after_trimmed.strip_prefix( ',' )
  {
    // Remove leading comma from after: `,"key":..` → `..`
    format!( "{before}{stripped}" )
  }
  else
  {
    format!( "{before}{after}" )
  }
}

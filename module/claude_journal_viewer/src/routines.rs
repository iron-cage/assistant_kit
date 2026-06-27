//! Unilang command routines for `ast .journal.*` integration.
//!
//! Each routine extracts parameters from a `VerifiedCommand`, delegates to the
//! corresponding function in [`crate::output`], and wraps the result as
//! `OutputData`.  Long-running or interactive commands (`.tail`, `.serve`) are
//! not supportable in the super-app context and return a guidance message.

use crate::output;
use std::collections::HashMap;
use unilang::{ ErrorCode, ErrorData, ExecutionContext, OutputData, VerifiedCommand };
use unilang::types::Value;

/// Build a `HashMap<String, String>` from the `arguments` map of a `VerifiedCommand`.
///
/// All argument types are converted to their string representation.
/// Empty string values are excluded so that optional params with empty defaults
/// don't shadow the output module's defaults.
fn extract_params( cmd : &VerifiedCommand ) -> HashMap< String, String >
{
  let mut map = HashMap::new();
  for ( name, value ) in &cmd.arguments
  {
    let s = match value
    {
      Value::String( s )  => s.clone(),
      Value::Integer( n ) => n.to_string(),
      Value::Boolean( b ) => b.to_string(),
      Value::Float( f )   => f.to_string(),
      _                   => continue,
    };
    if !s.is_empty() { map.insert( name.clone(), s ); }
  }
  map
}

/// Map an error string to an `ErrorData` with `InternalError` code.
fn err( msg : impl Into< String > ) -> ErrorData
{
  ErrorData::new( ErrorCode::InternalError, msg.into() )
}

// ── Command routines ──────────────────────────────────────────────────────────

/// `.journal.list` — display a filtered event table.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn list_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let params = extract_params( &cmd );
  let dir    = output::resolve_journal_dir( &params );
  output::list_output( &params, dir )
    .map( | s | OutputData::new( s + "\n", "text" ) )
    .map_err( | e | err( format!( "Error: {e}" ) ) )
}

/// `.journal.stats` — aggregate statistics by day or model.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn stats_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let params = extract_params( &cmd );
  let dir    = output::resolve_journal_dir( &params );
  output::stats_output( &params, dir )
    .map( | s | OutputData::new( s + "\n", "text" ) )
    .map_err( | e | err( format!( "Error: {e}" ) ) )
}

/// `.journal.search` — search events by pattern.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn search_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let params = extract_params( &cmd );
  let dir    = output::resolve_journal_dir( &params );
  output::search_output( &params, dir )
    .map( | s | OutputData::new( s + "\n", "text" ) )
    .map_err( | e | err( format!( "Error: {e}" ) ) )
}

/// `.journal.status` — show journal health report.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn status_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let params = extract_params( &cmd );
  let dir    = output::resolve_journal_dir( &params );
  Ok( OutputData::new( output::status_output( dir ) + "\n", "text" ) )
}

/// `.journal.export` — export events to a file.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn export_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let params = extract_params( &cmd );
  let dir    = output::resolve_journal_dir( &params );
  output::export_output( &params, dir )
    .map( | s | OutputData::new( s + "\n", "text" ) )
    .map_err( | e | err( format!( "Error: {e}" ) ) )
}

/// `.journal.prune` — delete old journal files.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn prune_routine( cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  let params = extract_params( &cmd );
  let dir    = output::resolve_journal_dir( &params );
  output::prune_output( &params, dir )
    .map( | s | OutputData::new( s + "\n", "text" ) )
    .map_err( | e | err( format!( "Error: {e}" ) ) )
}

/// `.journal.tail` — not supported in the super-app context.
///
/// `.tail` is a long-running blocking command; invoke `clj .tail` directly.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn tail_routine( _cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  Ok( OutputData::new(
    "For real-time tail, use clj .tail directly.\n".to_owned(),
    "text",
  ) )
}

/// `.journal.serve` — not supported in the super-app context.
///
/// `.serve` starts a blocking HTTP server; invoke `clj .serve` directly.
#[ allow( clippy::needless_pass_by_value ) ]
#[ inline ]
pub fn serve_routine( _cmd : VerifiedCommand, _ctx : ExecutionContext )
  -> core::result::Result< OutputData, ErrorData >
{
  Ok( OutputData::new(
    "For the web viewer, use clj .serve directly.\n".to_owned(),
    "text",
  ) )
}

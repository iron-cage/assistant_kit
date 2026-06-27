//! Shared output logic for the `clj` journal viewer.
//!
//! Each command handler returns a `String` (or `Result<String, String>` for
//! commands that validate input) so that the same logic can be used from both
//! the `clj` binary (`cli_main.rs`) and the unilang assistant routines
//! (`routines.rs`).

use claude_journal::{ EventRecord, EventType, JournalFilter, JournalReader };
use core::time::Duration;
use std::{ collections::HashMap, path::PathBuf, time::SystemTime };

// ── Directory resolution ──────────────────────────────────────────────────────

/// Resolve the journal directory from `dir::` param, `CLR_JOURNAL_DIR` env,
/// or the default `~/.clr/journal/`.
#[ must_use ]
pub fn resolve_journal_dir( params : &HashMap< String, String > ) -> PathBuf
{
  if let Some( d ) = params.get( "dir" )
  {
    return PathBuf::from( d );
  }
  if let Ok( d ) = std::env::var( "CLR_JOURNAL_DIR" )
  {
    if !d.is_empty() { return PathBuf::from( d ); }
  }
  let home = std::env::var( "HOME" ).unwrap_or_else( | _ | "/tmp".to_owned() );
  PathBuf::from( home ).join( ".clr" ).join( "journal" )
}

// ── Argument parsing helpers ──────────────────────────────────────────────────

/// Parse a human-readable duration string (e.g. `1h`, `30d`, `2w`) into a `Duration`.
///
/// Supported units: `s` (seconds), `m` (minutes), `h` (hours), `d` (days), `w` (weeks).
/// Returns `Err` with a descriptive message on invalid input.
///
/// # Errors
///
/// Returns an error when the input is not a valid `<number><unit>` pair.
pub fn parse_duration( s : &str ) -> Result< Duration, String >
{
  let err = || format!( "invalid duration '{s}' (expected e.g. 30s, 5m, 1h, 7d, 2w)" );
  let unit_char = s.chars().last().ok_or_else( err )?;
  let num_str   = &s[ ..s.len() - unit_char.len_utf8() ];
  let n : u64   = num_str.parse().map_err( | _ | err() )?;
  let secs = match unit_char
  {
    's' => n,
    'm' => n * 60,
    'h' => n * 3_600,
    'd' => n * 86_400,
    'w' => n * 86_400 * 7,
    _   => return Err( err() ),
  };
  Ok( Duration::from_secs( secs ) )
}

/// Parse an event type discriminator string into an `EventType`.
///
/// # Errors
///
/// Returns an error listing all valid type names when the input is not recognised.
pub fn parse_event_type( s : &str ) -> Result< EventType, String >
{
  EventType::parse( s ).ok_or_else( || format!(
    "invalid type '{s}' (valid: execution, credential, gate_wait, retry, \
     timeout, runner_retry, validation_retry, interactive)"
  ) )
}

/// Build a `JournalFilter` from the parsed param map.
///
/// # Errors
///
/// Returns a descriptive error string when any typed param (`since`, `until`,
/// `type`, `exit_code`, `limit`) fails to parse.
pub fn build_filter( params : &HashMap< String, String > ) -> Result< JournalFilter, String >
{
  let mut f = JournalFilter::default();
  if let Some( s ) = params.get( "since" )
  {
    f.since = Some( parse_duration( s )? );
  }
  if let Some( s ) = params.get( "until" )
  {
    f.until = SystemTime::now().checked_sub( parse_duration( s )? );
  }
  if let Some( s ) = params.get( "type" )    { f.event_type = Some( parse_event_type( s )? ); }
  if let Some( s ) = params.get( "command" ) { f.command = Some( s.clone() ); }
  if let Some( s ) = params.get( "exit_code" )
  {
    f.exit_code = Some(
      s.parse::< i32 >()
        .map_err( | _ | format!( "invalid exit_code '{s}' (expected integer)" ) )?
    );
  }
  if let Some( s ) = params.get( "model" )   { f.model = Some( s.clone() ); }
  if let Some( s ) = params.get( "creds" )   { f.creds = Some( s.clone() ); }
  if let Some( s ) = params.get( "limit" )
  {
    f.limit = Some(
      s.parse::< usize >()
        .map_err( | _ | format!( "invalid limit '{s}' (expected non-negative integer)" ) )?
    );
  }
  Ok( f )
}

// ── Formatting helpers ────────────────────────────────────────────────────────

/// Returns `true` when `NO_COLOR` is set in the environment.
#[ must_use ]
pub fn no_color() -> bool
{
  std::env::var_os( "NO_COLOR" ).is_some()
}

/// Wrap `s` in ANSI bold codes unless `NO_COLOR` is set.
#[ must_use ]
pub fn bold( s : &str ) -> String
{
  if no_color() { s.to_owned() }
  else { format!( "\x1b[1m{s}\x1b[0m" ) }
}

/// Format a millisecond duration as a human-readable string.
#[ must_use ]
pub fn format_ms( ms : u64 ) -> String
{
  if ms < 1_000 { format!( "{ms}ms" ) }
  else if ms < 60_000 { format!( "{:.1}s", ms as f64 / 1_000.0 ) }
  else { format!( "{:.1}m", ms as f64 / 60_000.0 ) }
}

/// Return the event table header line.
#[ must_use ]
pub fn event_header() -> String
{
  format!(
    "{:<16}  {:<18}  {:<10}  {:<22}  {:<4}  {:<10}  {:<8}  {:<8}  DUR",
    "TIME", "TYPE", "CMD", "MODEL", "EXIT", "COST", "IN", "OUT"
  )
}

/// Format one event record as a compact table row string.
#[ must_use ]
pub fn format_event_row( ev : &EventRecord ) -> String
{
  let ts     = ev.ts.get( ..16 ).unwrap_or( &ev.ts );
  let etype  = ev.event_type.as_str();
  let exit   = ev.fields.exit_code.map_or_else( || "-".to_owned(), | c | c.to_string() );
  let dur    = ev.fields.duration_ms.map_or_else( || "-".to_owned(), format_ms );
  let cost   = ev.fields.cost_usd.map_or_else( || "-".to_owned(), | c | format!( "${c:.4}" ) );
  let model  = ev.fields.model.as_deref().unwrap_or( "-" );
  let cmd    = ev.fields.command.as_deref().unwrap_or( "-" );
  let intok  = ev.fields.input_tokens.map_or_else( || "-".to_owned(), | t | t.to_string() );
  let outtok = ev.fields.output_tokens.map_or_else( || "-".to_owned(), | t | t.to_string() );
  format!( "{ts}  {etype:<18}  {cmd:<10}  {model:<22}  {exit:<4}  {cost:<10}  {intok:<8}  {outtok:<8}  {dur}" )
}

// ── Command output functions ──────────────────────────────────────────────────

/// `.list` — return a formatted event table or JSON array.
///
/// # Errors
///
/// Returns `Err` when any filter param is invalid or when the format is not
/// `"table"` or `"json"`.
pub fn list_output( params : &HashMap< String, String >, dir : PathBuf ) -> Result< String, String >
{
  let mut filter = build_filter( params )?;
  if filter.limit.is_none() { filter.limit = Some( 50 ); }

  let events = JournalReader::open( dir ).query( &filter );
  let format = params.get( "format" ).map_or( "table", String::as_str );
  match format
  {
    "json" => Ok(
      serde_json::to_string_pretty( &events )
        .map_err( | e | e.to_string() )?
    ),
    "table" =>
    {
      if events.is_empty() { return Ok( "No events found.".to_owned() ); }
      let mut out = String::new();
      out.push_str( &bold( &event_header() ) );
      out.push( '\n' );
      for ev in &events
      {
        out.push_str( &format_event_row( ev ) );
        out.push( '\n' );
      }
      out.push( '\n' );
      out.push_str( &format!( "{} event(s)", events.len() ) );
      Ok( out )
    }
    other => Err( format!( "invalid format '{other}' (valid: table, json)" ) ),
  }
}

/// `.stats` — return a stats table aggregated by `by` (day or model).
///
/// # Errors
///
/// Returns `Err` when any filter param is invalid or `by` is not `"day"` or
/// `"model"`.
pub fn stats_output( params : &HashMap< String, String >, dir : PathBuf ) -> Result< String, String >
{
  let mut filter = build_filter( params )?;
  if filter.since.is_none() { filter.since = Some( Duration::from_secs( 7 * 86_400 ) ); }

  let events = JournalReader::open( dir ).query( &filter );
  let by     = params.get( "by" ).map_or( "day", String::as_str );
  let mut out = String::new();
  match by
  {
    "day"   =>
    {
      out.push_str( &stats_table( &events, | ev | ev.ts.get( ..10 ).unwrap_or( "unknown" ).to_owned(), "DATE" ) );
    }
    "model" =>
    {
      out.push_str( &stats_table(
        &events,
        | ev | ev.fields.model.clone().unwrap_or_else( || "unknown".to_owned() ),
        "MODEL",
      ) );
    }
    other => return Err( format!( "invalid by '{other}' (valid: day, model)" ) ),
  }
  out.push( '\n' );
  out.push_str( &format!( "\nTotal: {} event(s)", events.len() ) );
  Ok( out )
}

/// Build a stats table string grouped by the key returned by `key_fn`.
fn stats_table< F >( events : &[ EventRecord ], key_fn : F, col_label : &str ) -> String
where
  F : Fn( &EventRecord ) -> String,
{
  let mut buckets : std::collections::HashMap< String, ( f64, u64 ) > = std::collections::HashMap::new();
  for ev in events
  {
    let entry = buckets.entry( key_fn( ev ) ).or_insert( ( 0.0, 0 ) );
    entry.0 += ev.fields.cost_usd.unwrap_or( 0.0 );
    entry.1 += 1;
  }
  let mut out = String::new();
  out.push_str( &bold( &format!( "{col_label:<24}  COUNT     COST" ) ) );
  out.push( '\n' );
  let mut rows : Vec< _ > = buckets.into_iter().collect();
  rows.sort_by( | a, b | a.0.cmp( &b.0 ) );
  for ( key, ( cost, count ) ) in &rows
  {
    out.push_str( &format!( "{key:<24}  {count:<8}  ${cost:.4}\n" ) );
  }
  out
}

/// `.search` — return events matching the pattern.
///
/// # Errors
///
/// Returns `Err` when any filter param is invalid or the required `pattern::`
/// param is absent.
pub fn search_output( params : &HashMap< String, String >, dir : PathBuf ) -> Result< String, String >
{
  let pattern = params.get( "pattern" )
    .cloned()
    .ok_or_else( || "pattern:: parameter required".to_owned() )?;
  let filter  = build_filter( params )?;
  let events  = JournalReader::open( dir ).query( &filter );
  let pat     = pattern.as_str();
  let matches : Vec< &EventRecord > = events.iter().filter( | ev |
  {
    ev.fields.stdout.as_deref().unwrap_or( "" ).contains( pat )
      || ev.fields.stderr.as_deref().unwrap_or( "" ).contains( pat )
      || ev.fields.error_message.as_deref().unwrap_or( "" ).contains( pat )
      || ev.fields.model.as_deref().unwrap_or( "" ).contains( pat )
      || ev.fields.command.as_deref().unwrap_or( "" ).contains( pat )
  } ).collect();

  if matches.is_empty()
  {
    return Ok( format!( "No events matching '{pattern}'." ) );
  }
  let mut out = String::new();
  out.push_str( &bold( &format!( "{:<16}  {:<18}  {:<10}  MATCH", "TIME", "TYPE", "CMD" ) ) );
  out.push( '\n' );
  for ev in &matches
  {
    let ts    = ev.ts.get( ..16 ).unwrap_or( &ev.ts );
    let etype = ev.event_type.as_str();
    let cmd   = ev.fields.command.as_deref().unwrap_or( "-" );
    out.push_str( &format!( "{ts}  {etype:<18}  {cmd:<10}  (matched)\n" ) );
  }
  out.push( '\n' );
  out.push_str( &format!( "{} match(es)", matches.len() ) );
  Ok( out )
}

/// `.status` — return a journal health string.
#[ must_use ]
pub fn status_output( dir : PathBuf ) -> String
{
  let reader = JournalReader::open( dir.clone() );
  let count  = reader.file_count();
  let bytes  = reader.total_bytes();
  let oldest = reader.oldest_date().unwrap_or_else( || "(none)".to_owned() );
  let newest = reader.newest_date().unwrap_or_else( || "(none)".to_owned() );
  format!(
    "{}\ndir:    {}\nfiles:  {count}\nsize:   {bytes} bytes\noldest: {oldest}\nnewest: {newest}",
    bold( "Journal Status" ),
    dir.display(),
  )
}

/// `.prune` — delete old journal files; return a description of what was done.
///
/// This function has filesystem side effects when `dry_run` is `false`.
///
/// # Errors
///
/// Returns `Err` when `keep::` or `dry_run::` params are invalid.
pub fn prune_output( params : &HashMap< String, String >, dir : PathBuf ) -> Result< String, String >
{
  let keep_str = params.get( "keep" ).map_or( "30d", String::as_str );
  let keep_dur = parse_duration( keep_str )?;
  let dry_raw  = params.get( "dry_run" ).map_or( "0", String::as_str );
  let dry_run  = match dry_raw
  {
    "0" | "false" => false,
    "1" | "true"  => true,
    other => return Err(
      format!( "invalid dry_run '{other}' (valid: 0, 1, true, false)" )
    ),
  };
  let cutoff  = SystemTime::now().checked_sub( keep_dur ).unwrap_or( SystemTime::UNIX_EPOCH );
  let Ok( entries ) = std::fs::read_dir( &dir ) else
  {
    return Ok( format!( "Journal dir {} not found or empty.", dir.display() ) );
  };
  let mut lines = Vec::new();
  let mut count = 0_u32;
  for entry in entries.flatten()
  {
    let path = entry.path();
    if path.extension().and_then( | e | e.to_str() ) != Some( "jsonl" ) { continue; }
    let mtime = entry.metadata().and_then( | m | m.modified() ).unwrap_or( SystemTime::UNIX_EPOCH );
    if mtime >= cutoff { continue; }
    if dry_run { lines.push( format!( "Would delete: {}", path.display() ) ); }
    else
    {
      match std::fs::remove_file( &path )
      {
        Ok( () ) => lines.push( format!( "Deleted: {}", path.display() ) ),
        Err( e ) => lines.push( format!( "Warning: could not delete {}: {e}", path.display() ) ),
      }
    }
    count += 1;
  }
  if count == 0 { return Ok( "Nothing to prune (all files within keep window).".to_owned() ); }
  let mut out = lines.join( "\n" );
  out.push( '\n' );
  out.push( '\n' );
  out.push_str( if dry_run
  {
    &format!( "{count} file(s) would be pruned." )
  }
  else
  {
    &format!( "{count} file(s) pruned." )
  } );
  Ok( out )
}

/// Build export file content for `events` in the given `format`.
///
/// # Errors
///
/// Returns `Err` for unknown format names.
pub fn build_export_content( events : &[ EventRecord ], format : &str ) -> Result< String, String >
{
  match format
  {
    "json" => Ok(
      serde_json::to_string_pretty( events ).unwrap_or_else( | _ | "[]".to_owned() )
    ),
    "jsonl" => Ok(
      events.iter()
        .filter_map( | ev | serde_json::to_string( ev ).ok() )
        .collect::< Vec< _ > >()
        .join( "\n" )
    ),
    "csv" =>
    {
      let mut rows = vec![ "ts,type,command,model,exit_code,cost_usd,duration_ms".to_owned() ];
      for ev in events
      {
        rows.push( format!(
          "{},{},{},{},{},{},{}",
          ev.ts,
          ev.event_type.as_str(),
          ev.fields.command.as_deref().unwrap_or( "" ),
          ev.fields.model.as_deref().unwrap_or( "" ),
          ev.fields.exit_code.map_or_else( String::new, | c | c.to_string() ),
          ev.fields.cost_usd.map_or_else( String::new, | c | format!( "{c:.6}" ) ),
          ev.fields.duration_ms.map_or_else( String::new, | d | d.to_string() ),
        ) );
      }
      Ok( rows.join( "\n" ) )
    }
    "table" =>
    {
      let mut rows = vec![
        format!(
          "{:<16}  {:<18}  {:<10}  {:<22}  EXIT  COST",
          "TIME", "TYPE", "CMD", "MODEL"
        )
      ];
      for ev in events
      {
        let ts    = ev.ts.get( ..16 ).unwrap_or( &ev.ts );
        let etype = ev.event_type.as_str();
        let cmd   = ev.fields.command.as_deref().unwrap_or( "-" );
        let model = ev.fields.model.as_deref().unwrap_or( "-" );
        let exit  = ev.fields.exit_code.map_or_else( || "-".to_owned(), | c | c.to_string() );
        let cost  = ev.fields.cost_usd.map_or_else( || "-".to_owned(), | c | format!( "${c:.4}" ) );
        rows.push( format!( "{ts:<16}  {etype:<18}  {cmd:<10}  {model:<22}  {exit:<4}  {cost}" ) );
      }
      Ok( rows.join( "\n" ) )
    }
    other => Err( format!( "invalid format '{other}' (valid: json, jsonl, csv, table)" ) ),
  }
}

/// `.export` — write events to a file; return a confirmation message.
///
/// # Errors
///
/// Returns `Err` when `output::` is missing, any filter param is invalid,
/// the format is unknown, or the file cannot be written.
pub fn export_output( params : &HashMap< String, String >, dir : PathBuf ) -> Result< String, String >
{
  let output  = params.get( "output" )
    .cloned()
    .ok_or_else( || "output:: parameter required".to_owned() )?;
  let format  = params.get( "format" ).map_or( "json", String::as_str );
  let filter  = build_filter( params )?;
  let events  = JournalReader::open( dir ).query( &filter );
  let content = build_export_content( &events, format )?;
  std::fs::write( &output, &content )
    .map_err( | e | format!( "could not write to '{output}': {e}" ) )?;
  Ok( format!( "Exported {} event(s) to {output}", events.len() ) )
}

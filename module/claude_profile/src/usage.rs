//! `.usage` command — 7-day token usage history from `stats-cache.json`.
//!
//! Reads `~/.claude/stats-cache.json` (written by Claude Code) and reports
//! per-model token totals for the 7-day window ending at `lastComputedDate`.
//!
//! # Data source
//!
//! `stats-cache.json` → `dailyModelTokens[].tokensByModel`.
//! Tokens per entry are the sum of input + output + cache tokens for that day
//! and model. The `lastComputedDate` field tells us when Claude Code last
//! recomputed the cache; data may be stale if Claude Code hasn't run recently.
//!
//! # What this command CANNOT show
//!
//! The live 5-hour and 7-day utilization percentages are server-side only —
//! Claude Code receives them via `anthropic-ratelimit-unified-*` response
//! headers at runtime but never persists them to disk. A future `.quota`
//! command would make a minimal API call to retrieve that data.

use core::fmt::Write as FmtWrite;
use std::collections::HashMap;

use serde_json::Value;
use unilang::data::{ ErrorCode, ErrorData, OutputData };
use unilang::interpreter::ExecutionContext;
use unilang::semantic::VerifiedCommand;

use crate::output::{ OutputFormat, OutputOptions, json_escape };

// ── Date helpers ──────────────────────────────────────────────────────────────

/// Days in a given month, accounting for leap years.
fn days_in_month( year : u32, month : u32 ) -> u32
{
  match month
  {
    2 =>
    {
      if year % 4 == 0 && ( year % 100 != 0 || year % 400 == 0 )
      {
        29
      }
      else
      {
        28
      }
    }
    4 | 6 | 9 | 11 => 30,
    _ => 31,
  }
}

/// Subtract `n` days from a `YYYY-MM-DD` date string.
///
/// Returns `None` if the input is malformed. Handles month and year
/// boundaries correctly for the small offsets we use (≤ 6 days).
fn subtract_days( date : &str, n : u32 ) -> Option< String >
{
  let parts : Vec< &str > = date.splitn( 3, '-' ).collect();
  if parts.len() < 3 { return None; }
  let mut year  : u32 = parts[ 0 ].parse().ok()?;
  let mut month : u32 = parts[ 1 ].parse().ok()?;
  let mut day : i64 = i64::from( parts[ 2 ].parse::<u32>().ok()? ) - i64::from( n );

  while day <= 0
  {
    if month == 1
    {
      month = 12;
      year -= 1;
    }
    else
    {
      month -= 1;
    }
    day += i64::from( days_in_month( year, month ) );
  }

  Some( format!( "{year:04}-{month:02}-{day:02}" ) )
}

// ── Model name helpers ────────────────────────────────────────────────────────

/// Shorten a full API model name to a compact display form.
///
/// - `claude-sonnet-4-6`         → `sonnet-4-6`
/// - `claude-haiku-4-5-20251001` → `haiku-4-5`   (trailing 8-digit date stripped)
/// - `glm-4.5-air`               → `glm-4.5-air`  (non-claude, unchanged)
fn model_short( model : &str ) -> String
{
  let name = model.strip_prefix( "claude-" ).unwrap_or( model );
  let parts : Vec< &str > = name.split( '-' ).collect();
  let has_date_suffix = parts.last()
    .is_some_and( |p| p.len() == 8 && p.bytes().all( |b| b.is_ascii_digit() ) );
  if has_date_suffix
  {
    parts[ ..parts.len() - 1 ].join( "-" )
  }
  else
  {
    name.to_owned()
  }
}

// ── Token formatting ──────────────────────────────────────────────────────────

/// Format tokens as a human-readable compact string.
///
/// - < 1 000          → `"999"`
/// - < 1 000 000      → `"42.3K"`
/// - ≥ 1 000 000      → `"17.3M"`
fn fmt_tokens_compact( n : u64 ) -> String
{
  // Boundaries account for {:.1} rounding: 999_950 / 1000 = 999.95 → "1000.0K"
  // so we promote to M at 999_950 instead of 1_000_000.
  if n < 1_000
  {
    format!( "{n}" )
  }
  else if n < 999_950
  {
    format!( "{:.1}K", n as f64 / 1_000.0 )
  }
  else
  {
    format!( "{:.1}M", n as f64 / 1_000_000.0 )
  }
}

/// Format tokens as a comma-separated integer: `17,282,815`.
fn fmt_tokens_full( n : u64 ) -> String
{
  let s = n.to_string();
  let mut out = String::with_capacity( s.len() + s.len() / 3 );
  for ( i, c ) in s.chars().enumerate()
  {
    if i > 0 && ( s.len() - i ) % 3 == 0 { out.push( ',' ); }
    out.push( c );
  }
  out
}

// ── Parsed usage data ─────────────────────────────────────────────────────────

struct UsageData
{
  /// ISO date of the most recent day in the 7-day window.
  period_end    : String,
  /// ISO date of the first day in the 7-day window.
  period_start  : String,
  /// Sum of all tokens across all models in the window.
  total         : u64,
  /// Per-model totals, sorted descending by token count.
  by_model      : Vec< ( String, u64 ) >,
  /// Daily entries (newest first), each with per-model breakdown.
  daily         : Vec< ( String, Vec< ( String, u64 ) > ) >,
}

/// Load and compute `UsageData` from `stats-cache.json`.
///
/// # Errors
///
/// Returns `ErrorData` if HOME is unset, the file is missing or malformed.
fn load_usage( paths : &crate::ClaudePaths ) -> Result< UsageData, ErrorData >
{
  let raw = std::fs::read_to_string( paths.stats_file() ).map_err( |e| ErrorData::new(
    ErrorCode::InternalError,
    format!( "cannot read stats-cache.json: {e}" ),
  ) )?;

  let json : Value = serde_json::from_str( &raw ).map_err( |e| ErrorData::new(
    ErrorCode::InternalError,
    format!( "malformed stats-cache.json: {e}" ),
  ) )?;

  let period_end = json[ "lastComputedDate" ]
    .as_str()
    .ok_or_else( || ErrorData::new(
      ErrorCode::InternalError,
      "stats-cache.json: lastComputedDate missing or not a string".to_string(),
    ) )?
    .to_owned();

  let period_start = subtract_days( &period_end, 6 )
    .unwrap_or_else( || period_end.clone() );

  let dmt = json[ "dailyModelTokens" ].as_array().ok_or_else( || ErrorData::new(
    ErrorCode::InternalError,
    "stats-cache.json: dailyModelTokens missing or not an array".to_string(),
  ) )?;

  let mut totals : HashMap< String, u64 > = HashMap::new();
  let mut daily  : Vec< ( String, Vec< ( String, u64 ) > ) > = Vec::new();

  for entry in dmt
  {
    let date = match entry[ "date" ].as_str()
    {
      Some( d ) => d.to_owned(),
      None => continue,
    };

    // Keep only dates inside the [period_start, period_end] window.
    // ISO-8601 strings sort lexicographically, so string comparison is correct.
    if date.as_str() < period_start.as_str() || date.as_str() > period_end.as_str()
    {
      continue;
    }

    let Some( tbm ) = entry[ "tokensByModel" ].as_object() else { continue };

    let mut day_models : Vec< ( String, u64 ) > = tbm
      .iter()
      .map( |( model, val ) | ( model_short( model ), val.as_u64().unwrap_or( 0 ) ) )
      .collect();
    day_models.sort_by( |a, b| b.1.cmp( &a.1 ) );

    for ( short, tokens ) in &day_models
    {
      *totals.entry( short.clone() ).or_insert( 0 ) += tokens;
    }

    daily.push( ( date, day_models ) );
  }

  // Newest first.
  daily.sort_by( |a, b| b.0.cmp( &a.0 ) );

  let mut by_model : Vec< ( String, u64 ) > = totals.into_iter().collect();
  by_model.sort_by( |a, b| b.1.cmp( &a.1 ) );

  let total : u64 = by_model.iter().map( |( _, t ) | t ).sum();

  Ok( UsageData { period_end, period_start, total, by_model, daily } )
}

// ── Output formatters ─────────────────────────────────────────────────────────

/// `v::0` — single compact summary line.
fn text_v0( data : &UsageData ) -> String
{
  let mut parts = vec![ format!( "{} total", fmt_tokens_compact( data.total ) ) ];
  for ( model, tokens ) in &data.by_model
  {
    parts.push( format!( "{model}: {}", fmt_tokens_compact( *tokens ) ) );
  }
  format!( "{}\n", parts.join( " · " ) )
}

/// `v::1` — labelled summary table (default).
fn text_v1( data : &UsageData ) -> String
{
  // Column widths: model name padded to 12, token count right-aligned to 14.
  let mut out = format!(
    "Usage — last 7 days ({} → {})\n\n",
    data.period_start, data.period_end
  );

  let total_str = fmt_tokens_full( data.total );
  let _ = writeln!( out, "  {:<12}  {:>14}", "Total", total_str );

  for ( model, tokens ) in &data.by_model
  {
    let pct = if data.total > 0 { *tokens as f64 / data.total as f64 * 100.0 } else { 0.0 };
    let _ = writeln!(
      out,
      "  {:<12}  {:>14}   {:4.1}%",
      model,
      fmt_tokens_full( *tokens ),
      pct
    );
  }

  out
}

/// `v::2` — summary + daily breakdown (newest first).
fn text_v2( data : &UsageData ) -> String
{
  let mut out = text_v1( data );

  if data.daily.is_empty() { return out; }

  out.push( '\n' );
  out.push_str( "  Daily:\n" );

  for ( date, models ) in &data.daily
  {
    let day_total : u64 = models.iter().map( |( _, t ) | t ).sum();
    let mut line = format!( "  {}  {:>12}", date, fmt_tokens_full( day_total ) );

    // Show each model contribution on the same line.
    for ( model, tokens ) in models
    {
      // Use first segment of model name (e.g. "sonnet" from "sonnet-4-6").
      let family = model.split( '-' ).next().unwrap_or( model );
      let _ = write!( line, "   {family}: {:>10}", fmt_tokens_full( *tokens ) );
    }
    out.push_str( &line );
    out.push( '\n' );
  }

  out
}

/// JSON output.
fn text_json( data : &UsageData ) -> String
{
  let mut models_json = String::new();
  let last = data.by_model.len().saturating_sub( 1 );
  for ( i, ( model, tokens ) ) in data.by_model.iter().enumerate()
  {
    let pct = if data.total > 0 { *tokens as f64 / data.total as f64 * 100.0 } else { 0.0 };
    let comma = if i < last { "," } else { "" };
    let _ = writeln!(
      models_json,
      "    {{\"model\":\"{}\",\"tokens\":{},\"pct\":{:.1}}}{}",
      json_escape( model ), tokens, pct, comma
    );
  }

  format!(
    "{{\
\"period_days\":7,\
\"period_start\":\"{}\",\
\"period_end\":\"{}\",\
\"total_tokens\":{},\
\"by_model\":[\n{}]}}\n",
    json_escape( &data.period_start ),
    json_escape( &data.period_end ),
    data.total,
    models_json,
  )
}

// ── Command handler ───────────────────────────────────────────────────────────

/// `.usage` — show 7-day token usage from `stats-cache.json`.
///
/// # Errors
///
/// Returns `ErrorData` if HOME is unset, `stats-cache.json` is missing
/// or malformed.
#[ inline ]
pub fn usage_routine( cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  let opts  = OutputOptions::from_cmd( &cmd )?;
  let paths = crate::ClaudePaths::new().ok_or_else( || ErrorData::new(
    ErrorCode::InternalError,
    "HOME environment variable not set".to_string(),
  ) )?;

  let data = load_usage( &paths )?;

  let content = match opts.format
  {
    OutputFormat::Json => text_json( &data ),
    OutputFormat::Text => match opts.verbosity
    {
      0 => text_v0( &data ),
      1 => text_v1( &data ),
      _ => text_v2( &data ),
    },
  };

  Ok( OutputData::new( content, "text" ) )
}

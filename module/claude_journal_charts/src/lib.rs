//! Aggregates `claude_journal` `Command` events into a daily-usage SVG bar chart.
//!
//! Exposes exactly one entry point, [`generate_usage_chart`], which a caller
//! invokes explicitly — this crate never generates a chart on its own initiative.

use core::fmt;
use std::collections::BTreeMap;
use std::path::{ Path, PathBuf };

use claude_journal::{ EventType, JournalFilter, JournalReader };
use svg_chart::{ ChartKind, ChartSpec, Series };

/// Error type returned by [`generate_usage_chart`].
#[ derive( Debug ) ]
pub enum ClaudeJournalChartsError
{
  /// `journal_dir` does not exist or is not a directory.
  JournalDirNotFound( PathBuf ),
  /// Underlying SVG chart rendering failed.
  Chart( svg_chart::SvgChartError ),
}

impl fmt::Display for ClaudeJournalChartsError
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::JournalDirNotFound( path ) => write!( f, "journal directory not found: {}", path.display() ),
      Self::Chart( e ) => write!( f, "chart render error: {e}" ),
    }
  }
}

impl std::error::Error for ClaudeJournalChartsError {}

/// Reads `Command` events from `journal_dir`, aggregates them into daily
/// invocation counts, and renders the result as an SVG bar chart at `out_path`.
///
/// An empty or `Command`-event-free journal produces a placeholder chart
/// (delegating to `svg_chart`'s own empty-series handling), not an error.
/// Non-`Command` events are excluded from the count.
///
/// # Errors
///
/// Returns `Err(ClaudeJournalChartsError::JournalDirNotFound)` if `journal_dir`
/// does not exist. Returns `Err(ClaudeJournalChartsError::Chart)` if the
/// underlying SVG rendering fails.
#[ inline ]
pub fn generate_usage_chart( journal_dir : &Path, out_path : &Path ) -> Result< (), ClaudeJournalChartsError >
{
  if !journal_dir.is_dir()
  {
    return Err( ClaudeJournalChartsError::JournalDirNotFound( journal_dir.to_path_buf() ) );
  }

  let reader = JournalReader::open( journal_dir.to_path_buf() );
  let filter = JournalFilter { event_type : Some( EventType::Command ), ..Default::default() };
  let events = reader.query( &filter );

  let mut counts : BTreeMap< String, u64 > = BTreeMap::new();
  for event in &events
  {
    let day = event.ts.get( 0..10 ).unwrap_or( "unknown" ).to_string();
    *counts.entry( day ).or_default() += 1;
  }

  let points : Vec< ( f64, f64 ) > = counts.values()
  .enumerate()
  .map( | ( i, &count ) | ( i as f64, count as f64 ) )
  .collect();

  let spec = ChartSpec
  {
    title : "Daily Command Invocations".to_string(),
    x_label : "day".to_string(),
    y_label : "count".to_string(),
    kind : ChartKind::Bar,
    series : vec![ Series { name : "invocations".to_string(), points } ],
  };

  svg_chart::render_to_file( &spec, out_path ).map_err( ClaudeJournalChartsError::Chart )
}

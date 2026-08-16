//! `.ps`-style table rendering for a [`ProcessInfo`] slice.
//!
//! Feature-gated behind `ps_table` so plain `claude_runner_core` consumers
//! never pull in `data_fmt`. Pure rendering only — no `/proc` scanning (see
//! [`crate::process`]) and no CLI wiring (see `claude_version`, Task 463).

use crate::process::ProcessInfo;
use crate::types::OutputFormat;
use data_fmt::{ Format, RowBuilder, TableConfig, TableFormatter };

/// Render a list of Claude Code processes as a table (`text`, `v >= 1`), a
/// compact PID+cwd listing (`text`, `v == 0`), or a JSON array (`json`).
///
/// Columns: PID, working directory, state. `state` is always `"running"` —
/// every entry is expected to originate from [`crate::process::find_claude_processes`],
/// which enumerates only processes currently present in `/proc`; this module
/// has no independent means of observing a non-running state.
///
/// Empty input renders `"no active processes"` in text mode, or `[]` in JSON mode.
///
/// # Examples
///
/// ```
/// use claude_runner_core::ps_table::render_ps_table;
/// use claude_runner_core::OutputFormat;
///
/// assert_eq!( render_ps_table( &[], OutputFormat::Text, 1 ), "no active processes" );
/// assert_eq!( render_ps_table( &[], OutputFormat::Json, 1 ), "[]" );
/// ```
#[ allow( clippy::missing_inline_in_public_items ) ]
#[ must_use ]
pub fn render_ps_table( processes : &[ ProcessInfo ], format : OutputFormat, verbosity : u8 ) -> String
{
  match format
  {
    OutputFormat::Json => render_json( processes ),
    OutputFormat::Text | OutputFormat::StreamJson => render_text( processes, verbosity ),
  }
}

/// Escape a string for embedding in a hand-built JSON string literal.
fn json_escape( s : &str ) -> String
{
  s.replace( '\\', "\\\\" ).replace( '"', "\\\"" )
}

fn render_json( processes : &[ ProcessInfo ] ) -> String
{
  let entries : Vec< String > = processes.iter().map( | p |
  {
    let cwd = json_escape( &p.cwd.to_string_lossy() );
    format!( "{{\"pid\":{},\"cwd\":\"{cwd}\",\"state\":\"running\"}}", p.pid )
  } ).collect();
  format!( "[{}]", entries.join( "," ) )
}

fn render_text( processes : &[ ProcessInfo ], verbosity : u8 ) -> String
{
  if processes.is_empty()
  {
    return "no active processes".to_string();
  }

  if verbosity == 0
  {
    // Compact: no column headers, mirrors `.ps v::0` (pid + cwd, one per line).
    let lines : Vec< String > = processes.iter()
      .map( | p | format!( "{} {}", p.pid, p.cwd.display() ) )
      .collect();
    return lines.join( "\n" );
  }

  let headers = vec![ "PID".to_string(), "Working Directory".to_string(), "State".to_string() ];
  let mut builder = RowBuilder::new( headers );
  for p in processes
  {
    let row : Vec< String > = vec![ p.pid.to_string(), p.cwd.display().to_string(), "running".to_string() ];
    builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
  }

  let config = TableConfig::plain().with_auto_wrap( false ).with_auto_fold( false );
  Format::format( &TableFormatter::with_config( config ), &builder.build_view() ).unwrap_or_default()
}

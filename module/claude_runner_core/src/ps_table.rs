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

#[ cfg( test ) ]
mod tests
{
  use super::*;
  use std::path::PathBuf;

  fn sample_process( pid : u32, cwd : &str ) -> ProcessInfo
  {
    ProcessInfo
    {
      pid,
      cmdline : format!( "claude --pid {pid}" ),
      cwd     : PathBuf::from( cwd ),
      args    : vec![ "claude".to_string() ],
    }
  }

  // T01: render_ps_table(&[], Text, 1) -> "no active processes"
  #[ test ]
  fn t01_empty_text_v1_no_active_processes()
  {
    assert_eq!( render_ps_table( &[], OutputFormat::Text, 1 ), "no active processes" );
  }

  // T02: render_ps_table(&[one_entry], Text, 1) -> contains PID and dir values
  #[ test ]
  fn t02_one_entry_text_v1_contains_pid_and_dir()
  {
    let p = sample_process( 12345, "/home/user/project" );
    let out = render_ps_table( &[ p ], OutputFormat::Text, 1 );
    assert!( out.contains( "12345" ), "must contain pid: {out}" );
    assert!( out.contains( "/home/user/project" ), "must contain cwd: {out}" );
  }

  // T03: render_ps_table(&[entry], Text, 1) -> contains PID column header
  #[ test ]
  fn t03_entry_text_v1_contains_pid_header()
  {
    let p = sample_process( 12345, "/home/user/project" );
    let out = render_ps_table( &[ p ], OutputFormat::Text, 1 );
    assert!( out.contains( "PID" ), "must contain PID column header: {out}" );
  }

  // T04: render_ps_table(&[entry], Json, 1) -> valid JSON string
  #[ test ]
  fn t04_entry_json_valid()
  {
    let p = sample_process( 12345, "/home/user/project" );
    let out = render_ps_table( &[ p ], OutputFormat::Json, 1 );
    let trimmed = out.trim();
    assert!( trimmed.starts_with( '[' ) && trimmed.ends_with( ']' ), "must be a JSON array: {out}" );
    assert!( out.contains( "\"pid\":12345" ), "must contain pid field: {out}" );
    assert!( out.contains( "\"cwd\":\"/home/user/project\"" ), "must contain cwd field: {out}" );
    assert!( out.contains( "\"state\":\"running\"" ), "must contain state field: {out}" );
  }

  // T05: render_ps_table(&[entry], Text, 0) -> compact output (fewer/no column headers)
  #[ test ]
  fn t05_entry_text_v0_compact_no_headers()
  {
    let p = sample_process( 12345, "/home/user/project" );
    let out = render_ps_table( &[ p ], OutputFormat::Text, 0 );
    assert!( out.contains( "12345" ), "must still contain pid: {out}" );
    assert!( !out.contains( "Working Directory" ), "v::0 must omit column headers: {out}" );
    assert!( !out.contains( "State" ), "v::0 must omit column headers: {out}" );
  }

  // Empty-slice JSON case (AC: "handles empty-slice case").
  #[ test ]
  fn empty_json_renders_empty_array()
  {
    assert_eq!( render_ps_table( &[], OutputFormat::Json, 1 ), "[]" );
  }

  // Multiple entries: each row must be present independently (no truncation/merge).
  #[ test ]
  fn multiple_entries_all_present()
  {
    let procs = vec![
      sample_process( 111, "/a" ),
      sample_process( 222, "/b" ),
    ];
    let out = render_ps_table( &procs, OutputFormat::Text, 1 );
    assert!( out.contains( "111" ) && out.contains( "/a" ), "first row missing: {out}" );
    assert!( out.contains( "222" ) && out.contains( "/b" ), "second row missing: {out}" );
  }
}

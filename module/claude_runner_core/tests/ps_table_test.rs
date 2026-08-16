//! Unit tests for `ps_table::render_ps_table` (feature `ps_table`).
//!
//! # Test Matrix
//!
//! | ID  | Scenario                                | Expectation                          |
//! |-----|------------------------------------------|--------------------------------------|
//! | T01 | empty slice, Text, v=1                   | "no active processes"                |
//! | T02 | one entry, Text, v=1                     | contains pid and cwd                 |
//! | T03 | one entry, Text, v=1                     | contains PID column header           |
//! | T04 | one entry, Json, v=1                     | valid JSON array with all 3 fields   |
//! | T05 | one entry, Text, v=0                     | compact — no column headers          |
//! | T06 | empty slice, Json, v=1                   | "[]"                                 |
//! | T07 | two entries, Text, v=1                   | both rows present independently      |
//!
//! Moved from an in-src `#[cfg(test)]` module in `src/ps_table.rs` — all tests
//! live in `tests/` per this project's test-location convention.

#![ cfg( feature = "ps_table" ) ]

use claude_runner_core::ps_table::render_ps_table;
use claude_runner_core::process::ProcessInfo;
use claude_runner_core::OutputFormat;
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

// T06: empty-slice JSON case (AC: "handles empty-slice case").
#[ test ]
fn t06_empty_json_renders_empty_array()
{
  assert_eq!( render_ps_table( &[], OutputFormat::Json, 1 ), "[]" );
}

// T07: multiple entries — each row must be present independently (no truncation/merge).
#[ test ]
fn t07_multiple_entries_all_present()
{
  let procs = vec![
    sample_process( 111, "/a" ),
    sample_process( 222, "/b" ),
  ];
  let out = render_ps_table( &procs, OutputFormat::Text, 1 );
  assert!( out.contains( "111" ) && out.contains( "/a" ), "first row missing: {out}" );
  assert!( out.contains( "222" ) && out.contains( "/b" ), "second row missing: {out}" );
}

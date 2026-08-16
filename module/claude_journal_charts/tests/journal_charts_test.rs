//! Integration tests for `claude_journal_charts`'s public API — Test Matrix T01-T06.

use claude_journal::{ EventFields, EventRecord, EventType, JournalWriter };
use claude_journal_charts::{ generate_usage_chart, ClaudeJournalChartsError };

fn write_event( writer : &JournalWriter, event_type : EventType, ts : &str )
{
  let event = EventRecord
  {
    v : 1,
    ts : ts.to_string(),
    event_type,
    fields : EventFields::default(),
  };
  writer.append( &event ).expect( "fixture append should succeed" );
}

#[ test ]
fn t01_empty_journal_produces_placeholder_not_err()
{
  let journal_dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let out_dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let out_path = out_dir.path().join( "chart.svg" );

  generate_usage_chart( journal_dir.path(), &out_path ).expect( "empty journal must not return Err" );

  let svg = std::fs::read_to_string( &out_path ).expect( "output file should exist and be readable" );
  assert!( svg.contains( "No data" ), "expected placeholder text in output: {svg}" );
}

#[ test ]
fn t02_events_across_three_days_produce_three_bars()
{
  let journal_dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let out_dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let out_path = out_dir.path().join( "chart.svg" );
  let writer = JournalWriter::new( journal_dir.path().to_path_buf() );

  for _ in 0..2 { write_event( &writer, EventType::Command, "2026-01-01T10:00:00.000Z" ); }
  for _ in 0..5 { write_event( &writer, EventType::Command, "2026-01-02T10:00:00.000Z" ); }
  write_event( &writer, EventType::Command, "2026-01-03T10:00:00.000Z" );

  generate_usage_chart( journal_dir.path(), &out_path ).expect( "generate_usage_chart should succeed" );

  let svg = std::fs::read_to_string( &out_path ).expect( "output file should exist and be readable" );
  let rect_count = svg.matches( "<rect" ).count();
  assert!( rect_count >= 3, "expected at least 3 <rect elements (one per day), found {rect_count} in: {svg}" );
}

#[ test ]
fn t03_same_day_events_produce_single_bar()
{
  let journal_dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let out_dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let out_path = out_dir.path().join( "chart.svg" );
  let writer = JournalWriter::new( journal_dir.path().to_path_buf() );

  for i in 0..4
  {
    write_event( &writer, EventType::Command, &format!( "2026-01-01T10:0{i}:00.000Z" ) );
  }

  generate_usage_chart( journal_dir.path(), &out_path ).expect( "generate_usage_chart should succeed" );

  let svg = std::fs::read_to_string( &out_path ).expect( "output file should exist and be readable" );
  let rect_count = svg.matches( "<rect" ).count();
  assert!( rect_count >= 1, "expected at least 1 <rect element (one bucket), found {rect_count} in: {svg}" );
}

#[ test ]
fn t04_non_command_events_are_excluded_from_the_count()
{
  let command_only_dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let mixed_dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let out_dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let command_only_out = out_dir.path().join( "command_only.svg" );
  let mixed_out = out_dir.path().join( "mixed.svg" );

  let command_only_writer = JournalWriter::new( command_only_dir.path().to_path_buf() );
  for _ in 0..3 { write_event( &command_only_writer, EventType::Command, "2026-01-01T10:00:00.000Z" ); }

  let mixed_writer = JournalWriter::new( mixed_dir.path().to_path_buf() );
  for _ in 0..3 { write_event( &mixed_writer, EventType::Command, "2026-01-01T10:00:00.000Z" ); }
  for _ in 0..2 { write_event( &mixed_writer, EventType::Execution, "2026-01-02T10:00:00.000Z" ); }
  write_event( &mixed_writer, EventType::Credential, "2026-01-03T10:00:00.000Z" );

  generate_usage_chart( command_only_dir.path(), &command_only_out ).expect( "generate_usage_chart should succeed" );
  generate_usage_chart( mixed_dir.path(), &mixed_out ).expect( "generate_usage_chart should succeed" );

  let command_only_svg = std::fs::read_to_string( &command_only_out ).expect( "output file should exist" );
  let mixed_svg = std::fs::read_to_string( &mixed_out ).expect( "output file should exist" );
  let command_only_rects = command_only_svg.matches( "<rect" ).count();
  let mixed_rects = mixed_svg.matches( "<rect" ).count();
  assert!( command_only_rects >= 1, "expected the command-only baseline to draw at least 1 bar, found {command_only_rects} <rect> elements" );
  assert_eq!(
    command_only_rects, mixed_rects,
    "non-Command events on separate days must not add bars: command_only had {command_only_rects} <rect> elements, mixed had {mixed_rects}"
  );
}

#[ test ]
fn t05_output_file_is_valid_svg()
{
  let journal_dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let out_dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let out_path = out_dir.path().join( "chart.svg" );
  let writer = JournalWriter::new( journal_dir.path().to_path_buf() );
  write_event( &writer, EventType::Command, "2026-01-01T10:00:00.000Z" );

  generate_usage_chart( journal_dir.path(), &out_path ).expect( "generate_usage_chart should succeed" );

  assert!( out_path.exists(), "output file should exist" );
  let content = std::fs::read_to_string( &out_path ).expect( "output file should be readable" );
  assert!( !content.is_empty(), "output file should be non-empty" );
  assert!( content.starts_with( "<svg" ), "output should start with <svg, got: {content}" );
  assert!( content.contains( "</svg>" ), "output should contain a closing </svg> tag" );
}

#[ test ]
fn t06_nonexistent_journal_directory_returns_err()
{
  let base = tempfile::tempdir().expect( "tempdir should be creatable" );
  let journal_dir = base.path().join( "does_not_exist" );
  let out_path = base.path().join( "chart.svg" );

  let result = generate_usage_chart( &journal_dir, &out_path );

  match result
  {
    Err( ClaudeJournalChartsError::JournalDirNotFound( path ) ) => assert_eq!( path, journal_dir ),
    other => panic!( "expected Err(JournalDirNotFound(_)), got {other:?}" ),
  }
  assert!( !out_path.exists(), "output file must not be written when journal_dir is missing" );
}

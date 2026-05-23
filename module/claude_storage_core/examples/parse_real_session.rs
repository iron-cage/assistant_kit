//! Parse a real Claude Code session to validate format compatibility

use std::path::PathBuf;
use claude_storage_core::Session;

fn main() -> Result< (), Box< dyn core::error::Error > >
{
  // Direct session file path
  let session_path = PathBuf::from( "/home/alice/.claude/projects/-home-alice-projects-my-app--default-topic/00000000-0000-0000-0000-000000000000.jsonl" );

  println!( "=== Real Session Parse Test ===" );
  println!( "Session file: {}", session_path.display() );

  let mut session = Session::load( &session_path )?;

  println!( "Session ID: {}", session.id() );

  let stats = session.stats()?;

  println!( "\nSession Statistics:" );
  println!( "  Total entries: {}", stats.total_entries );
  println!( "  User entries: {}", stats.user_entries );
  println!( "  Assistant entries: {}", stats.assistant_entries );
  println!( "  Total input tokens: {}", stats.total_input_tokens );
  println!( "  Total output tokens: {}", stats.total_output_tokens );

  if let Some( first ) = &stats.first_timestamp
  {
    println!( "  First message: {first}" );
  }

  if let Some( last ) = &stats.last_timestamp
  {
    println!( "  Last message: {last}" );
  }

  println!( "\n✅ Successfully parsed Claude Code session!" );
  println!( "✅ Graceful skip working (loaded {} conversation entries)", stats.total_entries );
  println!( "✅ Parser handled {} user + {} assistant entries", stats.user_entries, stats.assistant_entries );

  Ok( () )
}

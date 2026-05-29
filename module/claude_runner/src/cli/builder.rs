//! Command building: session continuity check and `ClaudeCommand` construction.

use super::parse::CliArgs;
use claude_runner_core::{ ClaudeCommand, EffortLevel };

/// Returns `true` when the resolved session directory exists and contains at least one entry.
///
/// When `session_dir` is `None`, falls back to `$HOME/.claude/` (the claude default).
/// Returns `false` on any I/O error, missing directory, or empty directory.
fn session_exists( session_dir : Option< &std::path::Path > ) -> bool
{
  let path = if let Some( p ) = session_dir
  {
    p.to_path_buf()
  }
  else
  {
    let Ok( home ) = std::env::var( "HOME" ) else { return false; };
    std::path::PathBuf::from( home ).join( ".claude" )
  };
  std::fs::read_dir( &path )
    .is_ok_and( | mut entries | entries.next().is_some() )
}

/// Translate parsed CLI args into a `ClaudeCommand` builder.
///
/// Session continuation (`-c`) is applied by default unless `--new-session` is set
/// or no prior session exists in the configured storage directory.
pub( crate ) fn build_claude_command( cli : &CliArgs ) -> ClaudeCommand
{
  let mut builder = ClaudeCommand::new();

  if let Some( ref dir ) = cli.dir
  {
    builder = builder.with_working_directory( dir.clone() );
  }
  if let Some( n ) = cli.max_tokens
  {
    builder = builder.with_max_output_tokens( n );
  }
  // Fix(BUG-214): inject -c only when a prior session exists in storage
  // Root cause: unconditional -c causes claude binary to exit on first use with no session
  // Pitfall: resumption flags (-c, --continue) require state to resume; guard with existence check
  if !cli.new_session && session_exists( cli.session_dir.as_deref().map( std::path::Path::new ) )
  {
    builder = builder.with_continue_conversation( true );
  }
  if !cli.no_skip_permissions
  {
    builder = builder.with_skip_permissions( true );
  }
  if !cli.no_effort_max
  {
    builder = builder.with_effort(
      cli.effort.unwrap_or( EffortLevel::Max )
    );
  }
  if cli.no_chrome
  {
    builder = builder.with_chrome( None );
  }
  if cli.no_persist
  {
    builder = builder.with_no_session_persistence( true );
  }
  if let Some( ref schema ) = cli.json_schema
  {
    builder = builder.with_json_schema( schema.as_str() );
  }
  if !cli.mcp_config.is_empty()
  {
    builder = builder.with_mcp_config( cli.mcp_config.iter().map( String::as_str ) );
  }
  if let Some( ref path ) = cli.file
  {
    builder = builder.with_stdin_file( std::path::PathBuf::from( path ) );
  }
  if cli.keep_claudecode
  {
    builder = builder.with_unset_claudecode( false );
  }
  if cli.verbose
  {
    builder = builder.with_verbose( true );
  }
  if let Some( ref model ) = cli.model
  {
    builder = builder.with_model( model.clone() );
  }
  if let Some( ref sd ) = cli.session_dir
  {
    builder = builder.with_session_dir( sd.clone() );
  }
  if let Some( ref sp ) = cli.system_prompt
  {
    builder = builder.with_system_prompt( sp.clone() );
  }
  if let Some( ref asp ) = cli.append_system_prompt
  {
    builder = builder.with_append_system_prompt( asp.clone() );
  }
  // Auto-add --print when a message is given and interactive mode is not explicitly requested.
  // Fix(BUG-227): message without -p was silently using TTY passthrough,
  // producing raw TUI escape codes instead of clean text output in scripted contexts.
  // Root cause: print mode was only enabled by explicit -p/--print; no auto-detection.
  // Pitfall: `--interactive` must suppress this auto-addition to allow prompted REPL sessions.
  let use_print = cli.print_mode || ( cli.message.is_some() && !cli.interactive );
  if use_print
  {
    builder = builder.with_arg( "--print" );
  }
  if let Some( ref msg ) = cli.message
  {
    // Fix(BUG-224): inject as suffix not prefix so the user task
    //   comes first in Claude's context window — earlier tokens carry more weight.
    // Root cause: original format!("ultrathink {msg}") buried the task description
    //   under the directive; suffix form preserves natural "state task, then direct thinking"
    //   order that matches Claude's conversational expectations.
    // Pitfall: idempotent guard must use trim_end().ends_with not starts_with —
    //   suffix anchors at the end; starts_with would miss re-injection on existing suffixes.
    let effective_msg = if cli.no_ultrathink || msg.trim_end().ends_with( "ultrathink" )
    {
      msg.clone()
    }
    else
    {
      format!( "{msg}\n\nultrathink" )
    };
    builder = builder.with_message( effective_msg );
  }

  builder
}

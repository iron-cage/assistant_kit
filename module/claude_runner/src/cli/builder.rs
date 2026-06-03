//! Command building: session continuity check and `ClaudeCommand` construction.

use super::parse::CliArgs;
use claude_runner_core::{ ClaudeCommand, EffortLevel };

/// Returns `true` when there is prior conversation history for the resolved session directory.
///
/// Fix(BUG-214-reopen): use project-specific storage path when no `--session-dir` is given.
/// Root cause: the previous fallback checked `$HOME/.claude/` (always non-empty — holds
/// credentials, projects/ dir, etc.) so `-c` was injected even for fresh project directories.
/// Pitfall: `$HOME/.claude/` is Claude's global config dir, not per-project session storage;
/// actual project sessions live at `$HOME/.claude/projects/{encoded(cwd)}/`.
///
/// - With `--session-dir <dir>`: sessions are stored directly in `<dir>`; check its entries.
/// - Without `--session-dir`: sessions are in `$HOME/.claude/projects/{encoded(effective_dir)}/`;
///   use `claude_storage_core::continuation::check_continuation` which encodes the path correctly.
fn session_exists
(
  session_dir  : Option< &std::path::Path >,
  effective_dir : Option< &std::path::Path >,
) -> bool
{
  if let Some( dir ) = session_dir
  {
    // Custom --session-dir: claude stores sessions directly inside this directory.
    std::fs::read_dir( dir ).is_ok_and( | mut entries | entries.next().is_some() )
  }
  else
  {
    // Default: project sessions live at $HOME/.claude/projects/{encoded(cwd)}/
    let cwd = effective_dir.map_or_else(
      || std::env::current_dir().unwrap_or_else( | _ | std::path::PathBuf::from( "." ) ),
      std::path::Path::to_path_buf,
    );
    claude_storage_core::continuation::check_continuation( &cwd )
  }
}

/// Resolve the effective working directory from `--dir` and `--subdir` args.
///
/// Fix(BUG-229): guard empty string — `--subdir ""` must be identity, not degenerate `/-`
/// Root cause: only `"."` was checked; empty string passed the guard and produced bare-hyphen dir
/// Pitfall: `env_str` already filters empty, but CLI path can deliver `""` via `--subdir ""`
///
/// Fix(BUG-231): skip `create_dir_all` in dry-run — dry-run must be side-effect-free
/// Root cause: `build_claude_command` runs before the dry-run branch; mkdir executed unconditionally
/// Pitfall: builder computes the path for display; only the run path needs the physical directory
fn resolve_effective_dir( cli : &CliArgs ) -> Option< std::path::PathBuf >
{
  let base_dir = cli.dir.as_deref().map( std::path::PathBuf::from );
  match cli.subdir.as_deref()
  {
    Some( sub ) if sub != "." && !sub.is_empty() =>
    {
      let base = base_dir.unwrap_or_else( ||
        std::env::current_dir().unwrap_or_else( | _ | std::path::PathBuf::from( "." ) )
      );
      let effective = base.join( format!( "-{sub}" ) );
      if !cli.dry_run
      {
        let _ = std::fs::create_dir_all( &effective );
      }
      Some( effective )
    }
    _ => base_dir,
  }
}

/// Translate parsed CLI args into a `ClaudeCommand` builder.
///
/// Session continuation (`-c`) is applied by default unless `--new-session` is set
/// or no prior session exists in the configured storage directory.
pub( crate ) fn build_claude_command( cli : &CliArgs ) -> ClaudeCommand
{
  let mut builder = ClaudeCommand::new();

  let effective_working_dir = resolve_effective_dir( cli );
  if let Some( ref dir ) = effective_working_dir
  {
    builder = builder.with_working_directory( dir.to_string_lossy().into_owned() );
  }
  if let Some( n ) = cli.max_tokens
  {
    builder = builder.with_max_output_tokens( n );
  }
  // Fix(BUG-214): inject -c only when a prior session exists in storage
  // Root cause: unconditional -c causes claude binary to exit on first use with no session
  // Pitfall: resumption flags (-c, --continue) require state to resume; guard with existence check
  if !cli.new_session && session_exists(
    cli.session_dir.as_deref().map( std::path::Path::new ),
    effective_working_dir.as_deref(),
  )
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

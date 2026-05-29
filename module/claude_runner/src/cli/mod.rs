mod parse;
mod credential;

use super::VerbosityLevel;
use claude_runner_core::{ ClaudeCommand, EffortLevel, IsolatedModel };
use parse::{
  CliArgs,
  parse_isolated_args, parse_refresh_args,
  apply_isolated_env_vars, apply_refresh_env_vars,
};
use credential::{ run_isolated_command, run_refresh_command };

pub( super ) use parse::{ parse_args, apply_env_vars };

pub( super ) fn print_help()
{
  println!( "clr — Execute Claude Code with configurable parameters" );
  println!();
  println!( "USAGE:" );
  println!( "  clr [OPTIONS] [MESSAGE]" );
  println!( "  clr run      [OPTIONS] [MESSAGE]" );
  println!( "  clr ask      [OPTIONS] [QUESTION]" );
  println!( "  clr isolated --creds <FILE> [--timeout <SECS>] [--trace] [MESSAGE]" );
  println!( "  clr refresh  --creds <FILE> [--timeout <SECS>] [--trace]" );
  println!( "  clr help" );
  println!();
  println!( "COMMANDS:" );
  // Fix(BUG-212): `run` was absent from COMMANDS despite being a valid explicit subcommand.
  // Root cause: print_help() only listed ask/isolated/refresh/help; discoverability AC violated.
  // Pitfall: `clr run` must strip the leading token before reaching the parser — see lib.rs.
  println!( "  run                                Execute Claude Code with configurable parameters (default mode)" );
  println!( "  ask                                Q&A mode with lightweight defaults (effort high, no -c)" );
  println!( "  isolated                           Run Claude with credential-isolated temp HOME" );
  println!( "  refresh                            Refresh OAuth credentials without running a task" );
  println!( "  help                               Print usage information and exit" );
  println!();
  println!( "ARGUMENTS:" );
  println!( "  [MESSAGE]                          Prompt message for Claude" );
  println!();
  println!( "OPTIONS:" );
  println!( "  -p, --print                        Non-interactive mode (capture and print output)" );
  println!( "  --interactive                      Force interactive mode even when a message is given" );
  println!( "  --new-session                      Start a new session (default: continues previous)" );
  println!( "  --model <MODEL>                    Model to use" );
  println!( "  --verbose                          Enable verbose output" );
  println!( "  --no-skip-permissions              Disable automatic permission bypass (on by default)" );
  println!( "  --max-tokens <N>                   Max output tokens (default: 200000)" );
  println!( "  --session-dir <PATH>               Session storage directory" );
  println!( "  --dir <PATH>                       Working directory" );
  println!( "  --dry-run                          Print command without executing" );
  println!( "  --trace                            Print command to stderr then execute (like set -x)" );
  println!( "  --system-prompt <TEXT>             Set system prompt (replaces the default)" );
  println!( "  --append-system-prompt <TEXT>      Append text to the default system prompt" );
  println!( "  --no-ultrathink                    Disable automatic \"\\n\\nultrathink\" message suffix" );
  println!( "  --effort <LEVEL>                   Reasoning effort: low, medium, high, max (default: max)" );
  println!( "  --no-effort-max                    Suppress default --effort max injection" );
  println!( "  --no-chrome                        Suppress default --chrome injection" );
  println!( "  --no-persist                       Disable session persistence (--no-session-persistence)" );
  println!( "  --json-schema <SCHEMA>             JSON schema for structured output" );
  println!( "  --mcp-config <PATH>                MCP server config file (repeatable)" );
  println!( "  --file <PATH>                      Pipe file content to subprocess stdin" );
  println!( "  --strip-fences                     Strip outermost markdown code fences from stdout" );
  println!( "  --keep-claudecode                  Preserve CLAUDECODE env var in subprocess (default: removed)" );
  println!( "  --verbosity <0-5>                  Runner output verbosity level (default: 3)" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "CREDENTIAL OPTIONS (isolated, refresh):" );
  println!( "  --creds <FILE>                     Credentials JSON file (required)" );
  println!( "  --timeout <SECS>                   Max seconds to wait (default: 30 isolated, 45 refresh)" );
  println!( "  --trace                            Print creds path, timeout, and claude invocation to stderr" );
}

/// Print help for the `isolated` subcommand and exit 0.
///
/// Called when `parse_isolated_args` encounters `-h` or `--help`.
/// Terminates the process via `std::process::exit(0)` so the caller
/// never needs to handle a return value.
fn print_isolated_help() -> !
{
  println!( "clr isolated — Run Claude Code with credential-isolated temp HOME" );
  println!();
  println!( "USAGE:" );
  println!( "  clr isolated --creds <FILE> [--timeout <SECS>] [MESSAGE] [-- PASSTHROUGH...]" );
  println!();
  println!( "ARGUMENTS:" );
  println!( "  [MESSAGE]                          Prompt message for Claude" );
  println!();
  println!( "CREDENTIAL OPTIONS:" );
  println!( "  --creds <FILE>                     Credentials JSON file (required)" );
  println!( "  --timeout <SECS>                   Max seconds to wait for subprocess (default: 30)" );
  println!( "  --trace                            Print underlying call details to stderr" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "EXIT CODES:" );
  println!( "  0    Success" );
  println!( "  1    Error (bad arguments, subprocess failure)" );
  println!( "  2    Timeout — subprocess did not finish within --timeout seconds" );
  std::process::exit( 0 );
}

/// Print help for the `refresh` subcommand and exit 0.
fn print_refresh_help() -> !
{
  println!( "clr refresh — Refresh OAuth credentials without running a task" );
  println!();
  println!( "USAGE:" );
  println!( "  clr refresh --creds <FILE> [--timeout <SECS>] [--trace]" );
  println!();
  println!( "CREDENTIAL OPTIONS:" );
  println!( "  --creds <FILE>                     Credentials JSON file (required)" );
  println!( "  --timeout <SECS>                   Max seconds to wait for refresh (default: 45)" );
  println!( "  --trace                            Print underlying call details to stderr" );
  println!( "  -h, --help                         Show this help" );
  println!();
  println!( "EXIT CODES:" );
  println!( "  0    Credentials were refreshed and written back" );
  println!( "  1    Error (bad arguments, no refresh occurred, subprocess failure)" );
  println!( "  2    Timeout — subprocess did not finish within --timeout seconds" );
  std::process::exit( 0 );
}

/// Print help for the `ask` subcommand and exit 0.
///
/// Called when `--help` or `-h` is detected immediately after parsing in `dispatch_ask`.
fn print_ask_help() -> !
{
  println!( "clr ask — Single-turn Q&A with lightweight defaults" );
  println!();
  println!( "USAGE:" );
  println!( "  clr ask [OPTIONS] [QUESTION]" );
  println!();
  println!( "ARGUMENTS:" );
  println!( "  [QUESTION]                         Question to ask Claude" );
  println!();
  println!( "DEFAULTS (differ from `clr run`):" );
  println!( "  --effort high                      Reasoning effort (ask default; run uses max)" );
  println!( "  --max-tokens 16384                 Max output tokens (ask default; run uses 200000)" );
  println!( "  --new-session                      Always start fresh (no session continuation)" );
  println!( "  --no-skip-permissions              No auto permission bypass" );
  println!( "  --no-ultrathink                    No ultrathink suffix" );
  println!( "  --no-chrome                        Chrome suppressed" );
  println!( "  --no-persist                       No session persistence" );
  println!();
  println!( "OPTIONS (same as `clr run`, overridable):" );
  println!( "  --effort <LEVEL>                   Override: low, medium, high, max" );
  println!( "  --max-tokens <N>                   Override max output tokens" );
  println!( "  --model <MODEL>                    Model to use" );
  println!( "  --dry-run                          Print command without executing" );
  println!( "  --trace                            Print command to stderr then execute" );
  println!( "  --system-prompt <TEXT>             Set system prompt" );
  println!( "  --append-system-prompt <TEXT>      Append to default system prompt" );
  println!( "  --dir <PATH>                       Working directory" );
  println!( "  --session-dir <PATH>               Session storage directory" );
  println!( "  --verbosity <0-5>                  Runner output verbosity level" );
  println!( "  --json-schema <SCHEMA>             JSON schema for structured output" );
  println!( "  --mcp-config <PATH>                MCP server config file (repeatable)" );
  println!( "  --file <PATH>                      Pipe file content to subprocess stdin" );
  println!( "  --strip-fences                     Strip outermost markdown code fences" );
  println!( "  -h, --help                         Show this help" );
  std::process::exit( 0 );
}

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
pub( super ) fn build_claude_command( cli : &CliArgs ) -> ClaudeCommand
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
  // Fix(issue-default-print): message without -p was silently using TTY passthrough,
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
    // Fix(issue-ultrathink-suffix): inject as suffix not prefix so the user task
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

/// Handle dry-run mode: print command preview and exit.
///
/// Always emits output regardless of verbosity level. Verbosity controls runner
/// diagnostics only; `--dry-run` output is core functionality the user explicitly requested.
// Fix(issue-dry-run-verbosity-gate): always emit; verbosity must not suppress --dry-run output
// Root cause: prior version gated on shows_progress() (≥3); --verbosity 0–2 produced silent exit
// Pitfall: Verbosity gates runner diagnostics only, never core feature output like --dry-run
pub( super ) fn handle_dry_run( builder : &ClaudeCommand )
{
  let env = builder.describe_env();
  let command = builder.describe();
  if !env.is_empty() { println!( "{env}" ); }
  println!( "{command}" );
}

// Fix(issue-unknown-subcommand): Guard against typos/truncations of known subcommand names.
// Root cause: `run_cli()` dispatched subcommands by exact string match only — any
//   non-matching first token silently fell through to `parse_args()`.
// Pitfall: Bare string comparison only guards exact matches; typos pass silently
//   unless a prefix-match guard is also placed before the main argument parser.
pub( super ) fn guard_unknown_subcommand( tokens : &[ String ] )
{
  // Fix(BUG-212): `run` was absent from KNOWN; typing `clr running` produced no helpful error.
  // Root cause: KNOWN list was never updated when `run` became an explicit subcommand.
  // Pitfall: `clr run` (len=3) bypasses is_identifier guard (requires len>=4), so it reaches
  //   the run_cli dispatch before guard is called — that is correct and expected.
  const KNOWN : &[ &str ] = &[ "run", "ask", "isolated", "refresh", "help" ];
  if let Some( first ) = tokens.first()
  {
    let is_identifier = !first.starts_with( '-' )
      && first.len() >= 4
      && first.chars().all( | c | c.is_alphanumeric() || c == '_' || c == '-' );
    if is_identifier
    {
      for &sub in KNOWN
      {
        if first != sub
          && ( sub.starts_with( first.as_str() ) || first.starts_with( sub ) )
        {
          eprintln!(
            "Error: unknown subcommand: {first}. Did you mean '{sub}'?\nRun with --help for usage."
          );
          std::process::exit( 1 );
        }
      }
    }
  }
}

/// Strip the outermost markdown code fence pair from `stdout`.
///
/// Finds the first and last lines starting with ` ``` ` (after optional leading whitespace).
/// If both exist and are distinct lines, returns the content between them (preserving
/// the original trailing-newline state). If fewer than two fences exist, returns `stdout`
/// unchanged.
fn strip_fences( stdout : &str ) -> String
{
  let lines : Vec< &str > = stdout.lines().collect();
  let first_fence = lines.iter().position( | l | l.trim_start().starts_with( "```" ) );
  let last_fence  = lines.iter().rposition( | l | l.trim_start().starts_with( "```" ) );
  match ( first_fence, last_fence )
  {
    ( Some( f ), Some( l ) ) if f < l =>
    {
      let body = lines[ f + 1 .. l ].join( "\n" );
      if stdout.ends_with( '\n' ) { format!( "{body}\n" ) } else { body }
    }
    _ => stdout.to_string(),
  }
}

/// Execute in non-interactive print mode (captures output).
///
/// Both `--print` (passed to claude) and `execute()` (captures stdout) are required:
/// `--print` tells claude to run single-shot with clean text output (no TUI);
/// `execute()` captures that output into memory for programmatic use.
/// Without `--print`, captured output would be TUI escape codes.
/// Without `execute()`, clean output would go straight to terminal uncaptured.
pub( super ) fn run_print_mode(
  builder           : &ClaudeCommand,
  verbosity         : VerbosityLevel,
  strip_fences_flag : bool,
)
{
  let output = match builder.execute()
  {
    Ok( o )  => o,
    Err( e ) =>
    {
      if verbosity.shows_errors()
      {
        eprintln!( "Error: {e}" );
      }
      std::process::exit( 1 );
    }
  };

  if !output.stderr.is_empty() { eprint!( "{}", output.stderr ); }

  if output.exit_code != 0
  {
    if verbosity.shows_errors()
    {
      eprintln!( "Error: Claude exited with code {}", output.exit_code );
    }
    std::process::exit( 1 );
  }

  let out = if strip_fences_flag { strip_fences( &output.stdout ) } else { output.stdout };
  print!( "{out}" );
}

/// Execute in interactive mode (TTY passthrough).
pub( super ) fn run_interactive( builder : &ClaudeCommand, verbosity : VerbosityLevel )
{
  let status = match builder.execute_interactive()
  {
    Ok( s )  => s,
    Err( e ) =>
    {
      if verbosity.shows_errors()
      {
        eprintln!( "Error: {e}" );
      }
      std::process::exit( 1 );
    }
  };

  if !status.success()
  {
    std::process::exit( status.code().unwrap_or( 1 ) );
  }
}

/// Parse, validate, and execute the `ask` subcommand.  Never returns.
///
/// `ask` is a facade of `run` with Q&A-optimised defaults:
///
/// **Unconditional** (cannot be overridden): `no_skip_permissions`, `new_session`,
///   `no_chrome`, `no_persist`, `no_ultrathink`.
///
/// **Soft** (CLI wins if explicitly provided, otherwise applied as default):
///   `effort = High`, `max_tokens = 16384`.
///
/// Priority chain: CLI flag > CLR_* env var > ask default.
pub( super ) fn dispatch_ask( tokens : &[ String ] ) -> !
{
  let mut cli = match parse_args( &tokens[ 1 .. ] )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  // Help wins before any default is applied.
  if cli.help { print_ask_help(); }
  // Unconditional ask pre-sets — override whatever CLI provided.
  cli.no_skip_permissions = true;
  cli.new_session         = true;
  cli.no_chrome           = true;
  cli.no_persist          = true;
  cli.no_ultrathink       = true;
  // Soft defaults — only applied when CLI did not explicitly set the field.
  cli.effort     = cli.effort.or( Some( EffortLevel::High ) );
  cli.max_tokens = cli.max_tokens.or( Some( 16384 ) );
  // Apply CLR_* env var fallbacks (CLI already took precedence during parse_args).
  apply_env_vars( &mut cli );

  let builder = build_claude_command( &cli );

  if cli.dry_run
  {
    handle_dry_run( &builder );
    std::process::exit( 0 );
  }

  let verbosity = cli.verbosity.unwrap_or_default();

  if cli.trace || verbosity.shows_verbose_detail()
  {
    let env     = builder.describe_env();
    let command = builder.describe();
    let mut preview = String::new();
    if !env.is_empty() { preview.push_str( &env ); preview.push( '\n' ); }
    preview.push_str( &command );
    eprintln!( "{preview}" );
  }

  if cli.print_mode || ( cli.message.is_some() && !cli.interactive )
  {
    run_print_mode( &builder, verbosity, cli.strip_fences );
  }
  else
  {
    run_interactive( &builder, verbosity );
  }
  std::process::exit( 0 );
}

/// Parse, validate, and execute the `isolated` subcommand.  Never returns.
pub( super ) fn dispatch_isolated( tokens : &[ String ] ) -> !
{
  let mut cli = match parse_isolated_args( &tokens[ 1 .. ] )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  apply_isolated_env_vars( &mut cli );
  if cli.creds_path.is_empty()
  {
    eprintln!( "Error: missing required argument: --creds\nRun with --help for usage." );
    std::process::exit( 1 );
  }
  run_isolated_command( &cli.creds_path, cli.timeout_secs, cli.trace, IsolatedModel::Default, cli.message.as_deref(), &cli.passthrough_args )
}

/// Parse, validate, and execute the `refresh` subcommand.  Never returns.
pub( super ) fn dispatch_refresh( tokens : &[ String ] ) -> !
{
  let mut cli = match parse_refresh_args( &tokens[ 1 .. ] )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  apply_refresh_env_vars( &mut cli );
  if cli.creds_path.is_empty()
  {
    eprintln!( "Error: missing required argument: --creds\nRun with --help for usage." );
    std::process::exit( 1 );
  }
  run_refresh_command( &cli.creds_path, cli.timeout_secs, cli.trace )
}

#[ cfg( test ) ]
mod tests
{
  use super::strip_fences;

  #[ test ]
  fn sf01_basic_fence_pair_stripped() { assert_eq!( strip_fences( "```\nhello\n```\n" ), "hello\n" ); }
  #[ test ]
  fn sf02_language_tagged_fence_stripped() { assert_eq!( strip_fences( "```rust\nfn f(){}\n```\n" ), "fn f(){}\n" ); }
  #[ test ]
  fn sf03_no_fences_pass_through() { assert_eq!( strip_fences( "plain text\n" ), "plain text\n" ); }
  #[ test ]
  fn sf04_single_fence_unchanged() { assert_eq!( strip_fences( "```\n" ), "```\n" ); }
  #[ test ]
  fn sf05_empty_string_unchanged() { assert_eq!( strip_fences( "" ), "" ); }
  #[ test ]
  fn sf06_inner_fences_preserved() { assert_eq!( strip_fences( "```\n```inner\n```\n```\n" ), "```inner\n```\n" ); }
  #[ test ]
  fn sf07_no_trailing_newline_preserved() { assert_eq!( strip_fences( "```\ncontent\n```" ), "content" ); }
  #[ test ]
  fn sf08_trailing_newline_preserved() { assert_eq!( strip_fences( "```\ncontent\n```\n" ), "content\n" ); }
}

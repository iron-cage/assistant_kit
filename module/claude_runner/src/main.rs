//! Claude Runner CLI
//!
//! Command-line interface for executing Claude Code with configurable parameters.
//! Mirrors Claude Code's native `--flag value` CLI syntax.
//!
//! # Architecture
//!
//! ```text
//! User argv (--flag value style)
//!     ↓ parse_args()
//! CliArgs struct
//!     ↓ build_claude_command()
//! ClaudeCommand builder (via claude_runner_core)
//!     ↓ execute_interactive() or execute()
//! subprocess: claude
//! ```
//!
//! # Session Continuity
//!
//! Session continuation (`-c`) is applied automatically to every invocation.
//! Use `--new-session` to start a fresh session when switching to an unrelated task.
//!
//! # Modes
//!
//! - **Interactive REPL** (default, no message): `clr`
//!   TTY passthrough via `execute_interactive()`. Continues previous session.
//! - **Print** (default when message given): `clr "Fix bug"` or `clr -p "Fix bug"`
//!   Non-interactive, adds `--print` automatically when a message is provided.
//!   Captures stdout via `execute()`. Continues previous session.
//! - **Explicit interactive with message**: `clr --interactive "Fix bug"`
//!   Suppresses the auto-`--print` default; runs TTY passthrough with a prompt.
//! - **Dry run** (`--dry-run`): shows the command without executing (includes `-c`).
//! - **Trace** (`--trace`): prints command to stderr then executes (like shell `set -x`).
//! - **New session** (`--new-session`): disables default continuation.
//!
//! # Usage
//!
//! ```sh
//! clr                                        # interactive REPL (continues session)
//! clr "Fix the bug"                          # print mode (auto-adds --print)
//! clr --interactive "Fix the bug"            # interactive with prompt (no --print)
//! clr -p "Explain this" --model sonnet       # explicit non-interactive print
//! clr --dry-run "test"                       # show command (includes -c by default)
//! clr --trace "Fix the bug"                  # print command to stderr then execute
//! clr --new-session "Start fresh analysis"   # new session
//! clr --system-prompt "You are a Rust expert" "Explain lifetimes"   # override system prompt
//! clr --append-system-prompt "Be concise." "Fix the bug"            # extend system prompt
//! clr --help
//! ```

use claude_runner::VerbosityLevel;
use claude_runner_core::ClaudeCommand;
use error_tools::{ Error, Result };

/// Parsed CLI arguments.
#[allow( clippy::struct_excessive_bools )]
#[derive( Default )]

struct CliArgs
{
  message : Option< String >,
  print_mode : bool,
  interactive : bool,
  new_session : bool,
  model : Option< String >,
  verbose : bool,
  no_skip_permissions : bool,
  max_tokens : Option< u32 >,
  session_dir : Option< String >,
  dir : Option< String >,
  dry_run : bool,
  trace : bool,
  verbosity : VerbosityLevel,
  help : bool,
  system_prompt : Option< String >,
  append_system_prompt : Option< String >,
}

fn print_help()
{
  println!( "clr — Execute Claude Code with configurable parameters" );
  println!();
  println!( "USAGE:" );
  println!( "  clr [OPTIONS] [MESSAGE]" );
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
  println!( "  --verbosity <0-5>                  Runner output verbosity level (default: 3)" );
  println!( "  -h, --help                         Show this help" );
}

/// Consume the next argv element as a flag's value.
fn next_value<'a>( tokens : &'a [ String ], idx : usize, flag : &str ) -> Result< &'a str >
{
  tokens.get( idx ).map( String::as_str ).ok_or_else( ||
    Error::msg( format!( "{flag} requires a value" ) )
  )
}

/// Parse a raw string as a u32 token limit with a clear error message.
///
/// Extracted from `parse_args()` to keep that function under clippy's
/// `too_many_lines` limit (100). Each new value-flag match arm costs ~5 lines;
/// pull additional multi-line logic here rather than into `parse_args` directly.
fn parse_token_limit( raw : &str ) -> Result< u32 >
{
  raw.parse::< u32 >().map_err( | _ |
    Error::msg( format!(
      "invalid --max-tokens value: {raw}\n\
       Expected unsigned integer 0–4294967295"
    ) )
  )
}

/// Parse argv into structured CLI arguments.
///
/// Mirrors Claude Code's native `--flag value` syntax.
/// Positional (non-flag) arguments are joined with space to form the message.
fn parse_args( tokens : &[ String ] ) -> Result< CliArgs >
{
  let mut parsed = CliArgs::default();
  let mut positional : Vec< String > = Vec::new();
  let mut i = 0;

  while i < tokens.len()
  {
    let token = tokens[ i ].as_str();
    match token
    {
      "-h" | "--help" =>
      {
        parsed.help = true;
      }
      "-p" | "--print" =>
      {
        parsed.print_mode = true;
      }
      "--interactive" =>
      {
        parsed.interactive = true;
      }
      "--new-session" =>
      {
        parsed.new_session = true;
      }
      "--verbose" =>
      {
        parsed.verbose = true;
      }
      "--no-skip-permissions" =>
      {
        parsed.no_skip_permissions = true;
      }
      "--dry-run" =>
      {
        parsed.dry_run = true;
      }
      "--trace" =>
      {
        parsed.trace = true;
      }
      "--system-prompt" =>
      {
        i += 1;
        parsed.system_prompt = Some( next_value( tokens, i, "--system-prompt" )?.to_string() );
      }
      "--append-system-prompt" =>
      {
        i += 1;
        parsed.append_system_prompt = Some( next_value( tokens, i, "--append-system-prompt" )?.to_string() );
      }
      "--model" =>
      {
        i += 1;
        parsed.model = Some( next_value( tokens, i, "--model" )?.to_string() );
      }
      "--max-tokens" =>
      {
        i += 1;
        parsed.max_tokens = Some( parse_token_limit( next_value( tokens, i, "--max-tokens" )? )? );
      }
      "--session-dir" =>
      {
        i += 1;
        parsed.session_dir = Some( next_value( tokens, i, "--session-dir" )?.to_string() );
      }
      "--dir" =>
      {
        i += 1;
        parsed.dir = Some( next_value( tokens, i, "--dir" )?.to_string() );
      }
      "--verbosity" =>
      {
        i += 1;
        let raw = next_value( tokens, i, "--verbosity" )?;
        parsed.verbosity = raw.parse::< VerbosityLevel >().map_err( Error::msg )?;
      }
      "--" =>
      {
        // Everything after `--` is positional.
        positional.extend( tokens[ i + 1 .. ].iter().cloned() );
        break;
      }
      s if s.starts_with( '-' ) =>
      {
        return Err( Error::msg( format!( "unknown option: {s}\nRun with --help for usage." ) ) );
      }
      _ =>
      {
        positional.push( tokens[ i ].clone() );
      }
    }
    i += 1;
  }

  if !positional.is_empty()
  {
    parsed.message = Some( positional.join( " " ) );
  }

  Ok( parsed )
}

/// Translate parsed CLI args into a `ClaudeCommand` builder.
///
/// Session continuation (`-c`) is applied by default unless `--new-session` is set.
fn build_claude_command( cli : &CliArgs ) -> ClaudeCommand
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
  if !cli.new_session
  {
    builder = builder.with_continue_conversation( true );
  }
  if !cli.no_skip_permissions
  {
    builder = builder.with_skip_permissions( true );
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
    builder = builder.with_message( msg.clone() );
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
fn handle_dry_run( builder : &ClaudeCommand )
{
  let env = builder.describe_env();
  let command = builder.describe();
  if !env.is_empty() { println!( "{env}" ); }
  println!( "{command}" );
}

/// Execute in non-interactive print mode (captures output).
///
/// Both `--print` (passed to claude) and `execute()` (captures stdout) are required:
/// `--print` tells claude to run single-shot with clean text output (no TUI);
/// `execute()` captures that output into memory for programmatic use.
/// Without `--print`, captured output would be TUI escape codes.
/// Without `execute()`, clean output would go straight to terminal uncaptured.
fn run_print_mode( builder : &ClaudeCommand, verbosity : VerbosityLevel )
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

  print!( "{}", output.stdout );
}

/// Execute in interactive mode (TTY passthrough).
fn run_interactive( builder : &ClaudeCommand, verbosity : VerbosityLevel )
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

fn main()
{
  let tokens : Vec< String > = std::env::args().skip( 1 ).collect();

  let cli = match parse_args( &tokens )
  {
    Ok( c )  => c,
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  };

  if cli.help
  {
    print_help();
    return;
  }

  if cli.print_mode && cli.message.is_none()
  {
    eprintln!( "Error: --print requires a message argument" );
    eprintln!( "Run with --help for usage." );
    std::process::exit( 1 );
  }

  let builder = build_claude_command( &cli );

  if cli.dry_run
  {
    handle_dry_run( &builder );
    return;
  }

  // Trace / verbose detail preview: show command to stderr before executing.
  if cli.trace || cli.verbosity.shows_verbose_detail()
  {
    let env = builder.describe_env();
    let command = builder.describe();
    let mut preview = String::new();
    if !env.is_empty() { preview.push_str( &env ); preview.push( '\n' ); }
    preview.push_str( &command );
    eprintln!( "{preview}" );
  }

  // Dispatch to print mode when message given (default) or -p/--print explicit.
  // --interactive overrides the message-default back to TTY passthrough.
  if cli.print_mode || ( cli.message.is_some() && !cli.interactive )
  {
    run_print_mode( &builder, cli.verbosity );
  }
  else
  {
    run_interactive( &builder, cli.verbosity );
  }
}

//! `claude_runner` crate / `clr` binary — Claude Code CLI + command schema constants.
//!
//! This crate has two roles:
//!
//! 1. **Library** — exports [`COMMANDS_YAML`], the path to the `.claude` command schema,
//!    for use by YAML consumers at compile time or runtime.
//!
//! 2. **Binary** (`clr`) — Standalone CLI that mirrors Claude Code's native
//!    `--flag value` syntax and executes via `claude_runner_core`.
//!    Session continuation (`-c`) is applied by default; use `--new-session` to start fresh.
//!
//! ## Two roles, two consumers
//!
//! ```text
//! clr binary (standalone CLI)
//!   invoked directly: clr "Fix bug" --dir /path --model sonnet
//!     → parse_args() → ClaudeCommand builder → claude subprocess (with -c by default)
//!   message given → print mode (default); bare clr → interactive REPL
//!
//! YAML consumers (e.g. consumer workspace's CLI, build.rs)
//!   aggregate: claude_runner::COMMANDS_YAML → registers .claude + .claude.help in PHF map
//! ```
//!
//! This lib has **zero consumer workspace dependencies**. Without `enabled`, it is a pure constants +
//! types crate. With `enabled`, it also exposes [`register_commands`] (API consistency shim).
//!
//! ## Registering commands in other binaries
//!
//! **Build-time (PHF):** Point `build.rs` at [`COMMANDS_YAML`].
//!
//! **Runtime:** Use `MultiYamlAggregator` with [`COMMANDS_YAML`].

pub mod verbosity;
pub use verbosity::VerbosityLevel;

/// Absolute path to this crate's command definitions YAML.
///
/// Use in `build.rs` for compile-time aggregation or at runtime for dynamic registration.
pub const COMMANDS_YAML : &str = concat!( env!( "CARGO_MANIFEST_DIR" ), "/claude.commands.yaml" );

#[ cfg( feature = "enabled" ) ]
/// Register `claude_runner` commands into an existing registry.
///
/// `claude_runner` commands are defined in [`COMMANDS_YAML`] for compile-time aggregation
/// (used by `assistant/build.rs`). This function is provided for API consistency with
/// other Layer 2 crates; the body is intentionally empty because runtime registration of
/// `.claude` commands is handled by the build-time YAML aggregation path in `assistant`.
#[ inline ]
pub fn register_commands( _registry : &mut unilang::registry::CommandRegistry ) {}

#[ cfg( feature = "enabled" ) ]
mod cli
{
  use super::VerbosityLevel;
  use claude_runner_core::{ ClaudeCommand, EffortLevel };
  use error_tools::{ Error, Result };

  /// Parsed CLI arguments.
  #[ allow( clippy::struct_excessive_bools ) ]
  #[ derive( Default ) ]
  pub( super ) struct CliArgs
  {
    pub( super ) message              : Option< String >,
    pub( super ) print_mode           : bool,
    pub( super ) interactive          : bool,
    pub( super ) new_session          : bool,
    pub( super ) model                : Option< String >,
    pub( super ) verbose              : bool,
    pub( super ) no_skip_permissions  : bool,
    pub( super ) max_tokens           : Option< u32 >,
    pub( super ) session_dir          : Option< String >,
    pub( super ) dir                  : Option< String >,
    pub( super ) dry_run              : bool,
    pub( super ) trace                : bool,
    pub( super ) verbosity            : VerbosityLevel,
    pub( super ) help                 : bool,
    pub( super ) system_prompt        : Option< String >,
    pub( super ) append_system_prompt : Option< String >,
    pub( super ) no_ultrathink        : bool,
    pub( super ) effort               : Option< EffortLevel >,
    pub( super ) no_effort_max        : bool,
  }

  pub( super ) fn print_help()
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
    println!( "  --no-ultrathink                    Disable automatic \"\\n\\nultrathink\" message suffix" );
    println!( "  --effort <LEVEL>                   Reasoning effort: low, medium, high, max (default: max)" );
    println!( "  --no-effort-max                    Suppress default --effort max injection" );
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

  /// Parse a raw string as an `EffortLevel` with a clear error message.
  ///
  /// Extracted from `parse_args()` to keep that function under clippy's
  /// `too_many_lines` limit. Delegates to `EffortLevel::from_str`.
  fn parse_effort_level( raw : &str ) -> Result< EffortLevel >
  {
    raw.parse::< EffortLevel >().map_err( Error::msg )
  }

  /// Parse argv into structured CLI arguments.
  ///
  /// Mirrors Claude Code's native `--flag value` syntax.
  /// Positional (non-flag) arguments are joined with space to form the message.
  ///
  /// `--help`/`-h` wins regardless of other flags or unknown tokens: if either appears
  /// anywhere in `tokens`, parsing short-circuits and returns `CliArgs { help: true, .. }`.
  #[ allow( clippy::too_many_lines ) ]
  pub( super ) fn parse_args( tokens : &[ String ] ) -> Result< CliArgs >
  {
    // --help/-h always wins — return early before any other token is parsed.
    // This ensures help is shown even when unknown flags or other errors are present.
    // Fix(issue-help-loses-to-unknown): parse_args returned Err on the first unknown flag,
    // so main() never reached the cli.help check even when --help was present in argv.
    // Root cause: early Err return on unknown flags prevented the help check from firing.
    // Pitfall: checking cli.help after parse_args completes is insufficient — the Err path
    // in main() exits before any field of CliArgs is consulted.
    if tokens.iter().any( | t | t == "--help" || t == "-h" )
    {
      return Ok( CliArgs { help : true, ..CliArgs::default() } );
    }

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
        "--no-ultrathink" =>
        {
          parsed.no_ultrathink = true;
        }
        "--effort" =>
        {
          i += 1;
          parsed.effort = Some(
            parse_effort_level( next_value( tokens, i, "--effort" )? )?
          );
        }
        "--no-effort-max" =>
        {
          parsed.no_effort_max = true;
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
          // Fix(issue-empty-msg-double-dash): filter empty tokens here too — `clr -- ""`
          // must behave like bare `clr`, not forward a degenerate "\n\nultrathink" message.
          // Root cause: positional.extend() copies all tokens verbatim; the empty-token
          // guard in the `_` arm does not apply to the `--` code path.
          // Pitfall: filter at the individual-token level (not the joined string) so that
          // whitespace-only strings like " " are still valid messages and pass through.
          positional.extend( tokens[ i + 1 .. ].iter().filter( | t | !t.is_empty() ).cloned() );
          break;
        }
        s if s.starts_with( '-' ) =>
        {
          return Err( Error::msg( format!( "unknown option: {s}\nRun with --help for usage." ) ) );
        }
        _ =>
        {
          // Fix(issue-empty-msg-ultrathink): skip empty tokens so `clr ""` behaves like
          // bare `clr` (no message, no --print, no degenerate "\n\nultrathink" forwarded).
          // Root cause: empty string was pushed to positional, joined to message=Some(""),
          // then the ultrathink suffix produced "\n\nultrathink" for an empty payload.
          // Pitfall: filter individual empty tokens, not the joined string — whitespace-only
          // strings like " " are valid non-empty messages and must not be filtered out.
          if !tokens[ i ].is_empty()
          {
            positional.push( tokens[ i ].clone() );
          }
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
    if !cli.new_session
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

  /// Execute in non-interactive print mode (captures output).
  ///
  /// Both `--print` (passed to claude) and `execute()` (captures stdout) are required:
  /// `--print` tells claude to run single-shot with clean text output (no TUI);
  /// `execute()` captures that output into memory for programmatic use.
  /// Without `--print`, captured output would be TUI escape codes.
  /// Without `execute()`, clean output would go straight to terminal uncaptured.
  pub( super ) fn run_print_mode( builder : &ClaudeCommand, verbosity : VerbosityLevel )
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
}

#[ cfg( feature = "enabled" ) ]
/// Run the `clr`/`claude_runner` CLI.
///
/// Entry point shared by the `clr` and `claude_runner` binary targets.
#[ inline ]
pub fn run_cli()
{
  use cli::{ parse_args, build_claude_command, handle_dry_run, print_help, run_print_mode, run_interactive };

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

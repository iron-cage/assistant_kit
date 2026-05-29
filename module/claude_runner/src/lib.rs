//! `claude_runner` crate / `clr` binary — Claude Code CLI + command schema constants.
//!
//! This crate has two roles:
//!
//! 1. **Library** — exports [`COMMANDS_YAML`], the path to the `.claude` command schema,
//!    for use by YAML consumers at compile time or runtime.
//!
//! 2. **Binary** (`clr`) — Standalone CLI that mirrors Claude Code's native
//!    `--flag value` syntax and executes via `claude_runner_core`.
//!    Session continuation (`-c`) is applied by default when a prior session exists; use `--new-session` to start fresh.
//!
//! ## Two roles, two consumers
//!
//! ```text
//! clr binary (standalone CLI)
//!   invoked directly: clr "Fix bug" --dir /path --model sonnet
//!     → parse_args() → ClaudeCommand builder → claude subprocess (with -c when session exists)
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
mod cli;

#[ cfg( feature = "enabled" ) ]
#[ doc( hidden ) ]
pub use cli::strip_fences;

#[ cfg( feature = "enabled" ) ]
/// Run the `clr`/`claude_runner` CLI.
///
/// Entry point shared by the `clr` and `claude_runner` binary targets.
#[ inline ]
pub fn run_cli()
{
  use cli::{
    parse_args, build_claude_command, handle_dry_run,
    print_help, run_built_command,
    dispatch_ask, dispatch_isolated, dispatch_refresh,
    apply_env_vars, guard_unknown_subcommand,
  };

  let tokens : Vec< String > = std::env::args().skip( 1 ).collect();

  // Dispatch `help` subcommand before everything else.
  if tokens.first().map( String::as_str ) == Some( "help" )
  {
    print_help();
    return;
  }

  // Fix(BUG-212): `run` is the default mode expressed as an explicit subcommand.
  // Root cause: `clr run msg` treated "run" as the message argument — silent wrong behavior.
  // Pitfall: strip only the leading "run" token; remaining args are passed normally.
  let tokens : Vec< String > = if tokens.first().map( String::as_str ) == Some( "run" )
  {
    tokens[ 1.. ].to_vec()
  }
  else
  {
    tokens
  };

  // Fix(BUG-215): re-check `help` after stripping `run` — `clr run help` must print help.
  // Root cause: the `help` dispatch above fires before the `run` strip; after stripping,
  // remaining `["help"]` fell through to parse_args which treated "help" as a message,
  // causing claude to be invoked with "help\n\nultrathink" as the prompt.
  // Pitfall: any subcommand dispatch that precedes a token-strip must be re-checked after.
  if tokens.first().map( String::as_str ) == Some( "help" )
  {
    print_help();
    return;
  }

  // Dispatch subcommands — these functions never return.
  if tokens.first().map( String::as_str ) == Some( "ask" )      { dispatch_ask( &tokens ); }
  if tokens.first().map( String::as_str ) == Some( "isolated" ) { dispatch_isolated( &tokens ); }
  if tokens.first().map( String::as_str ) == Some( "refresh" )  { dispatch_refresh( &tokens ); }

  guard_unknown_subcommand( &tokens );

  let mut cli = match parse_args( &tokens )
  {
    Ok( c )  => c,
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  };
  apply_env_vars( &mut cli );

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

  run_built_command( &builder, &cli );
}

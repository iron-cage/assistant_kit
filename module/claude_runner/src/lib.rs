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

// render_summary and resolve_fields are used by tests/summary_unit_test.rs.
// The allow(unused_imports) suppresses the false-positive lint that fires because
// the lib-crate compiler cannot see external test consumers at library compile time.
#[ cfg( feature = "enabled" ) ]
#[ doc( hidden ) ]
#[ allow( unused_imports ) ]
pub use cli::{ render_summary, resolve_fields };

#[ cfg( feature = "enabled" ) ]
/// Run the `clr`/`claude_runner` CLI.
///
/// Entry point shared by the `clr` and `claude_runner` binary targets.
#[ inline ]
pub fn run_cli()
{
  use cli::{
    print_help, dispatch_run,
    dispatch_ask, dispatch_isolated, dispatch_refresh, dispatch_ps, dispatch_kill,
    dispatch_tools, guard_unknown_subcommand,
  };

  let tokens : Vec< String > = std::env::args().skip( 1 ).collect();

  // Fix(BUG-212): strip the leading "run" token before any subcommand checks so that
  //   `clr run msg` does not treat "run" as the message argument.
  // Root cause: `clr run msg` treated "run" as the message argument — silent wrong behavior.
  // Pitfall: strip only the leading "run" token; remaining args are passed normally.
  //
  // Fix(BUG-215): stripping first also means the single help check below covers both
  //   `clr help` and `clr run help` — no need for two separate checks.
  // Root cause: `clr run help` bypassed the help dispatcher because `tokens[0]` was "run",
  //   not "help" — the help check never fired for the `clr run help` form.
  // Pitfall: the help check must run after stripping, not before; reversing the order re-breaks both bugs.
  let tokens : Vec< String > = if tokens.first().map( String::as_str ) == Some( "run" )
  {
    tokens[ 1.. ].to_vec()
  }
  else
  {
    tokens
  };

  // Single help check — covers both `clr help` and `clr run help` (post-strip above).
  if tokens.first().map( String::as_str ) == Some( "help" )
  {
    print_help();
    return;
  }

  // Dispatch subcommands.  All arms are -> ! (process exits inside the handler).
  // Also update KNOWN_SUBCOMMANDS in cli/mod.rs when adding a subcommand.
  match tokens.first().map( String::as_str )
  {
    Some( "ask" )      => dispatch_ask( &tokens ),
    Some( "isolated" ) => dispatch_isolated( &tokens ),
    Some( "refresh" )  => dispatch_refresh( &tokens ),
    Some( "ps" )       => dispatch_ps( &tokens ),
    Some( "kill" )     => dispatch_kill( &tokens ),
    Some( "tools" )    => dispatch_tools( &tokens ),
    _                  => {}
  }

  guard_unknown_subcommand( &tokens );

  dispatch_run( &tokens );
}

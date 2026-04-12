//! Binary entry point logic shared by the `claude_storage` and `clg` targets.
//!
//! This module exists to give each `[[bin]]` target a unique source file
//! (eliminating the Cargo "same file in multiple targets" warning) while
//! keeping the REPL and one-shot pipeline in a single compiled location.

// The generated static command registry has no doc comments — allow that
// for this module only; the outer lib enforces missing_docs everywhere else.
#![ allow( missing_docs ) ]

use std::{ env, io::{ self, Write }, process };
use crate::cli;
use unilang::prelude::*;
use unilang::phf;

// Include compile-time generated static commands (produced by build.rs).
include!( concat!( env!( "OUT_DIR" ), "/static_commands.rs" ) );

/// Build a `CommandRegistry` wired to all `claude_storage` routines.
fn build_command_registry() -> CommandRegistry
{
  type RoutineFn = fn( VerifiedCommand, ExecutionContext ) -> Result< OutputData, ErrorData >;

  let routines : phf::Map< &'static str, RoutineFn > = phf::phf_map!
  {
    ".status"         => cli::status_routine,
    ".list"           => cli::list_routine,
    ".show"           => cli::show_routine,
    ".count"          => cli::count_routine,
    ".search"         => cli::search_routine,
    ".export"         => cli::export_routine,
    ".projects"       => cli::projects_routine,
    ".path"           => cli::path_routine,
    ".exists"         => cli::exists_routine,
    ".session.dir"    => cli::session_dir_routine,
    ".session.ensure" => cli::session_ensure_routine,
  };

  #[ allow( deprecated ) ]
  let mut registry = CommandRegistry::new();

  for ( name, static_cmd ) in AGGREGATED_COMMANDS.entries()
  {
    if let Some( &routine ) = routines.get( *name )
    {
      let cmd : CommandDefinition = ( *static_cmd ).into();
      #[ allow( deprecated ) ]
      if let Err( e ) = registry.command_add_runtime( &cmd, Box::new( routine ) )
      {
        eprintln!( "WARNING: Failed to register routine for {name}: {e}" );
      }
    }
  }

  registry
}

/// Run REPL (Read-Eval-Print Loop) mode.
fn run_repl( registry : CommandRegistry )
{
  println!( "Claude Code Storage CLI" );
  println!( "Type 'help' for available commands, 'exit' to quit.\n" );

  let pipeline = Pipeline::new( registry );
  let mut command_buffer = String::new();

  loop
  {
    print!( "> " );
    io::stdout().flush().unwrap();

    command_buffer.clear();
    if let Err( e ) = io::stdin().read_line( &mut command_buffer )
    {
      eprintln!( "Error reading input: {e}" );
      continue;
    }

    let input = command_buffer.trim();

    if input.is_empty() { continue; }

    if input == "exit" || input == "quit" || input == "q"
    {
      println!( "Goodbye!" );
      break;
    }

    let result = pipeline.process_command_simple( input );

    if result.success
    {
      if let Some( output ) = result.outputs.first()
      {
        println!( "{}", output.content );
      }
    }
    else if let Some( error ) = result.error
    {
      eprintln!( "Error: {error}" );
    }
  }
}

/// Extract the user-visible message from a unilang pipeline error string.
///
/// The pipeline wraps handler `ErrorData` with multi-level prefixes:
///   `"Execution error: Execution Error: {message}\n"` (execution path)
///   `"Semantic analysis error: Execution Error: {message}\n"` (analysis path)
///
/// In one-shot scripting mode callers should see only `{message}` — the text
/// the handler authored — without framework noise. Spec-defined commands like
/// `.exists` rely on exact stderr content (e.g. `"no sessions"`) for shell
/// conditional use; the wrapping would break `stderr == "no sessions"` checks.
///
/// Parse errors are NOT stripped because the context they carry (`"Parse error:
/// Syntax(...) at StrSpan {...}"`) is the full useful message.
///
/// Pitfall: `ErrorData::Display` uses `writeln!` so the message already ends with
/// `\n` inside the error string. `trim()` is required to remove that trailing
/// newline before printing; otherwise `eprintln!` adds a second `\n`.
fn extract_user_message( error : &str ) -> String
{
  let trimmed = error.trim_end();
  for prefix in &[
    "Execution error: Execution Error: ",
    "Semantic analysis error: Execution Error: ",
  ]
  {
    if let Some( rest ) = trimmed.strip_prefix( prefix )
    {
      return rest.trim().to_string();
    }
  }
  trimmed.to_string()
}

/// Run one-shot command mode.
///
/// # Output contract
///
/// This function calls `println!("{}", output.content)`, which appends `\n`.
/// Handlers must therefore return `OutputData` whose `content` does NOT end
/// with `\n`; otherwise the binary emits a blank trailing line (`\n\n`).
/// This bites tests using exact `assert_eq!(stdout, "…\n")` checks and
/// any shell caller that splits on newlines.
///
/// Errors are printed via `extract_user_message` which strips the pipeline
/// wrapping (`"Execution error: Execution Error: "`) so handlers receive
/// clean user-visible messages on stderr (e.g. `"no sessions"` not
/// `"Error: Execution error: Execution Error: no sessions"`).
#[ allow( clippy::needless_pass_by_value ) ]
fn execute_oneshot( registry : CommandRegistry, args : Vec< String > ) -> !
{
  let pipeline    = Pipeline::new( registry );
  let command_line = args[ 1.. ].join( " " );
  let result      = pipeline.process_command_simple( &command_line );

  if result.success
  {
    if let Some( output ) = result.outputs.first()
    {
      println!( "{}", output.content );
    }
    process::exit( 0 );
  }
  else
  {
    if let Some( error ) = result.error
    {
      eprintln!( "{}", extract_user_message( &error ) );
    }
    process::exit( 1 );
  }
}

/// Run the `claude_storage` CLI.
///
/// Dispatches to REPL mode (no args) or one-shot mode (args provided).
/// Entry point shared by the `claude_storage` and `clg` binary targets.
#[ inline ]
pub fn run()
{
  let registry = build_command_registry();
  let args : Vec< String > = env::args().collect();

  if args.len() == 1
  {
    run_repl( registry );
  }
  else
  {
    execute_oneshot( registry, args );
  }
}

//! Binary entry point logic shared by the `claude_storage` and `clg` targets.
//!
//! This module exists to give each `[[bin]]` target a unique source file
//! (eliminating the Cargo "same file in multiple targets" warning) while
//! keeping the REPL and one-shot pipeline in a single compiled location.

use std::{ env, io::{ self, Write }, process };
use crate::cli;
use unilang::prelude::*;
use unilang::phf;

// Include compile-time generated static commands (produced by build.rs).
// Lint suppression is scoped to the generated module — file-wide attrs are forbidden
// by dep/l1_imp.rulebook.md § Strict Workspace Lint Inheritance.
mod generated
{
  #![ allow( missing_docs ) ]
  #![ allow( clippy::unreadable_literal ) ]
  include!( concat!( env!( "OUT_DIR" ), "/static_commands.rs" ) );
}
use generated::AGGREGATED_COMMANDS;

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
    ".project.path"   => cli::project_path_routine,
    ".project.exists" => cli::project_exists_routine,
    ".session.dir"    => cli::session_dir_routine,
    ".session.ensure" => cli::session_ensure_routine,
  };

  let mut registry = CommandRegistry::new();

  for ( name, static_cmd ) in AGGREGATED_COMMANDS.entries()
  {
    if let Some( &routine ) = routines.get( *name )
    {
      let cmd : CommandDefinition = ( *static_cmd ).into();
      if let Err( e ) = registry.register_with_routine( &cmd, Box::new( routine ) )
      {
        eprintln!( "WARNING: Failed to register routine for {name}: {e}" );
      }
    }
  }

  registry
}

/// Render grouped help output via `cli_fmt::CliHelpTemplate`.
///
/// Displays 4 command groups (Status, Session, Project, Query),
/// shared parameters, and usage examples.
fn print_usage( binary : &str )
{
  use cli_fmt::help::*;

  let mut data    = CliHelpData::default();
  data.binary     = binary.to_string();
  data.tagline    = "Claude Code storage explorer: query conversations, sessions, and projects.".to_string();
  data.groups     = vec!
  [
    CommandGroup
    {
      name    : "Status".to_string(),
      entries : vec!
      [
        CommandEntry { name : ".status".to_string(), desc : "Show storage summary (projects, sessions, entries)".to_string() },
      ],
    },
    CommandGroup
    {
      name    : "Session".to_string(),
      entries : vec!
      [
        CommandEntry { name : ".session.dir".to_string(),    desc : "Print the filesystem path of a session directory".to_string() },
        CommandEntry { name : ".session.ensure".to_string(), desc : "Ensure a session directory exists (create if missing)".to_string() },
      ],
    },
    CommandGroup
    {
      name    : "Project".to_string(),
      entries : vec!
      [
        CommandEntry { name : ".projects".to_string(),       desc : "List all known projects with session counts".to_string() },
        CommandEntry { name : ".project.path".to_string(),   desc : "Print the filesystem path of a project directory".to_string() },
        CommandEntry { name : ".project.exists".to_string(), desc : "Check whether a project has any sessions".to_string() },
      ],
    },
    CommandGroup
    {
      name    : "Query".to_string(),
      entries : vec!
      [
        CommandEntry { name : ".list".to_string(),   desc : "List sessions with filtering and sorting".to_string() },
        CommandEntry { name : ".show".to_string(),   desc : "Display entries from a specific session".to_string() },
        CommandEntry { name : ".count".to_string(),  desc : "Count sessions or entries matching criteria".to_string() },
        CommandEntry { name : ".search".to_string(), desc : "Search conversation content across sessions".to_string() },
        CommandEntry { name : ".export".to_string(), desc : "Export session data in various formats".to_string() },
      ],
    },
  ];
  data.options    = vec!
  [
    OptionEntry { name : "project::ID".to_string(),     desc : "Filter by project identifier".to_string() },
    OptionEntry { name : "session::ID".to_string(),     desc : "Target a specific session".to_string() },
    OptionEntry { name : "scope::VALUE".to_string(),    desc : "Scope filter (all, cli, web, ide)".to_string() },
    OptionEntry { name : "format::FMT".to_string(),     desc : "Output format (text, json, markdown)".to_string() },
    OptionEntry { name : "limit::N".to_string(),        desc : "Maximum entries to return".to_string() },
    OptionEntry { name : "query::TEXT".to_string(),     desc : "Search query string".to_string() },
  ];
  data.examples   = vec!
  [
    ExampleEntry { invocation : format!( "{binary} .status" ),                          desc : None },
    ExampleEntry { invocation : format!( "{binary} .list scope::cli limit::10" ),       desc : None },
    ExampleEntry { invocation : format!( "{binary} .search query::\"error handling\"" ), desc : None },
    ExampleEntry { invocation : format!( "{binary} --repl" ),                           desc : Some( "Enter interactive REPL mode".to_string() ) },
  ];
  print!( "{}", CliHelpTemplate::new( CliHelpStyle::default(), data ).render() );
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
/// `.project.exists` rely on exact stderr content (e.g. `"no sessions"`) for shell
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
  // Fix(issue-030): Quote parameter values that contain spaces before joining argv into
  // a REPL-style command line string.
  //
  // Root cause: `args[1..].join(" ")` destroys arg boundaries — a space inside a single
  // argv element (e.g., `query::session management`) is indistinguishable from the space
  // between two separate args after joining. The REPL parser then splits on all spaces,
  // causing `management` to become an unknown positional token.
  //
  // Pitfall: Any `name::value` parameter where the value contains a space will silently
  // lose the second word unless the value is quoted before joining. Always quote `::` values
  // that contain spaces; the REPL parser strips the surrounding `"..."` during parsing.
  let command_line = args[ 1.. ]
    .iter()
    .map( | arg |
    {
      if let Some( sep ) = arg.find( "::" )
      {
        let key   = &arg[ ..sep + 2 ];
        let value = &arg[ sep + 2.. ];
        if value.contains( ' ' )
        {
          return format!( "{}\"{}\"", key, value.replace( '"', "\\\"" ) );
        }
      }
      arg.clone()
    } )
    .collect::< Vec< _ > >()
    .join( " " );
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
/// Three invocation modes:
/// - Help: empty argv, `.help`, `--help`, `-h` → grouped help via `cli_fmt`
/// - REPL: `--repl` → interactive read-eval-print loop
/// - One-shot: any other args → execute command and exit
///
/// Entry point shared by the `claude_storage` and `clg` binary targets.
#[ inline ]
pub fn run()
{
  let args : Vec< String > = env::args().collect();

  let binary = args.first()
  .and_then( | p | std::path::Path::new( p ).file_name() )
  .and_then( | n | n.to_str() )
  .unwrap_or( "clg" )
  .to_owned();

  if args.len() == 1
  {
    print_usage( &binary );
    process::exit( 0 );
  }

  let first = &args[ 1 ];

  if first == ".help" || first == "--help" || first == "-h" || first == "help"
  {
    print_usage( &binary );
    process::exit( 0 );
  }

  if first == "--repl"
  {
    let registry = build_command_registry();
    run_repl( registry );
    return;
  }

  let registry = build_command_registry();
  execute_oneshot( registry, args );
}

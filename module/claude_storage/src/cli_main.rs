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
    ".tail"           => cli::tail_routine,
    ".usage"          => cli::usage_routine,
    ".rollup"         => cli::rollup_routine,
    ".cost"           => cli::cost_routine,
    ".count"          => cli::count_routine,
    ".search"         => cli::search_routine,
    ".export"         => cli::export_routine,
    ".projects"       => cli::projects_routine,
    ".project.path"   => cli::project_path_routine,
    ".project.exists" => cli::project_exists_routine,
    ".session.dir"    => cli::session_dir_routine,
    ".session.ensure" => cli::session_ensure_routine,
    ".session.path"   => cli::session_path_routine,
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
        CommandEntry { name : ".session.path".to_string(),   desc : "Print a session .jsonl file path (latest, session:: UUID, or fork topic::)".to_string() },
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
    // Fix(help-listing-drift)
    // Root cause: this Vec is a hand-maintained copy of the command list also
    // declared in `unilang.commands.yaml` and wired in `build_command_registry()`
    // (below). `.tail`, `.usage`, and `.rollup` were added to both of those but
    // never added here, so they worked perfectly yet stayed invisible in `clg .`.
    // Pitfall: adding a routine to `build_command_registry()` does not surface it
    // in the grouped help listing — this Vec must be updated in the same change.
    // `tests/help_command_coverage_test.rs` now asserts every command declared in
    // `unilang.commands.yaml` appears in `clg .` output, so a future drift fails
    // the test instead of shipping silently.
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
        CommandEntry { name : ".tail".to_string(),   desc : "Print last N conversation turns for current directory".to_string() },
        CommandEntry { name : ".usage".to_string(),  desc : "Per-session usage table \u{2014} turns, tokens, cache, duration, dir".to_string() },
        CommandEntry { name : ".rollup".to_string(), desc : "Grouped/filtered/sorted/projected token-usage table".to_string() },
        CommandEntry { name : ".cost".to_string(),   desc : "Per-conversation cost table with agent sessions folded in".to_string() },
      ],
    },
  ];
  data.options    = vec!
  [
    OptionEntry { name : "project::ID".to_string(),     desc : "Filter by project identifier".to_string() },
    OptionEntry { name : "session::ID".to_string(),     desc : "Target a specific session".to_string() },
    OptionEntry { name : "scope::VALUE".to_string(),    desc : "Scope filter (relevant|local|under|global|around)".to_string() },
    OptionEntry { name : "format::FMT".to_string(),     desc : "Output format (text, json, markdown)".to_string() },
    OptionEntry { name : "limit::N".to_string(),        desc : "Maximum entries to return".to_string() },
    OptionEntry { name : "query::TEXT".to_string(),     desc : "Search query string".to_string() },
  ];
  data.examples   = vec!
  [
    ExampleEntry { invocation : format!( "{binary} .status" ),                          desc : None },
    ExampleEntry { invocation : format!( "{binary} .list limit::10" ),                  desc : None },
    ExampleEntry { invocation : format!( "{binary} .search query::\"error handling\"" ), desc : None },
    ExampleEntry { invocation : format!( "{binary} --repl" ),                           desc : Some( "Enter interactive REPL mode".to_string() ) },
  ];
  print!( "{}", CliHelpTemplate::new( CliHelpStyle::default(), data ).render() );
}

/// Returns `true` when `token` requests the full grouped command listing.
///
/// Covers every alias unilang and this binary recognize as a global-help
/// trigger: bare argv (handled separately), `.`, `.help`, `--help`, `-h`, `help`.
fn is_global_help_token( token : &str ) -> bool
{
  matches!( token, "." | ".help" | "--help" | "-h" | "help" )
}

/// Render single-command detail help via `cli_fmt::CliHelpTemplate`.
///
/// unilang's own auto-generated `<command>.help` routine (`format_command_help()`
/// inside the `unilang` crate) prints a plain-text dump with no `cli_fmt` styling.
/// This renders the same content — description, arguments, examples — through
/// the same styled renderer `print_usage` uses for the grouped listing, so every
/// help-producing entry point stays visually consistent.
fn print_command_help( binary : &str, cmd : &CommandDefinition )
{
  use cli_fmt::help::*;

  let name = cmd.name().as_str();

  let mut data     = CliHelpData::default();
  data.binary      = binary.to_string();
  data.tagline     = cmd.description().to_string();
  data.usage_lines = vec![ format!( "Usage: {binary} {name}" ) ];
  data.arguments   = cmd.arguments().iter().map( | arg |
  {
    let suffix = match ( arg.attributes.optional, &arg.attributes.default )
    {
      ( true, Some( default ) ) => format!( "optional, default: {default}" ),
      ( true, None )            => "optional".to_string(),
      ( false, _ )              => "required".to_string(),
    };
    OptionEntry
    {
      name : format!( "{}::{}", arg.name, arg.kind ),
      desc : format!( "{} ({suffix})", arg.description ),
    }
  } ).collect();
  data.examples    = cmd.examples().iter().map( | example |
    ExampleEntry { invocation : format!( "{binary} {example}" ), desc : None }
  ).collect();

  print!( "{}", CliHelpTemplate::new( CliHelpStyle::default(), data ).render() );
}

/// Detects a `<command>.help` token and, when `<command>` is registered,
/// renders its detail via `print_command_help` instead of letting unilang's
/// own auto-generated `<command>.help` routine execute and print its
/// plain-text form.
///
/// Returns `true` when `token` was handled; the caller should skip normal
/// dispatch in that case.
fn try_command_help( binary : &str, registry : &CommandRegistry, token : &str ) -> bool
{
  let Some( base ) = token.strip_suffix( ".help" ) else { return false };
  let Some( cmd ) = registry.command( base ) else { return false };
  print_command_help( binary, &cmd );
  true
}

/// Detects the space-separated `<command> help` two-token form and, when
/// `<command>` is registered, renders its detail via `print_command_help` —
/// the same rendering `try_command_help` uses for the dot-suffix form.
///
/// A sibling to `is_global_help_token()` and `try_command_help()` — a third
/// hand-rolled interceptor of the same shape, mirroring the pattern
/// established in commit `4230527e`, not a shared abstraction with them. If
/// a 4th help-interception form is ever needed, consolidate the `try_*`/
/// `is_global_help_token` predicates into an ordered (matcher, renderer)
/// list rather than adding another hand-rolled `if` block.
///
/// Returns `true` when `tokens` was handled; the caller should skip normal
/// dispatch in that case.
///
/// `help` is a reserved second token for every registered command,
/// unconditionally — including a command like `.search` whose first
/// positional argument is an unconstrained string where "help" could
/// otherwise be a literal value (AGG-01, resolved as an accepted trade-off
/// matching npm/git/kubectl's own reserved-word convention: `.search help`
/// renders help rather than searching; `.search query::help`, the
/// named-parameter form, is unaffected and still searches). See
/// `t09_search_help_one_shot_reserved_word_tradeoff` /
/// `t10_search_query_help_named_param_unaffected`.
///
/// Fix(BUG-005)
/// Root cause: unilang binds a trailing bare token positionally when it
/// isn't recognized as a flag or as one of the existing dot-suffix/global
/// help special cases, so `<command> help` (space-separated) fell through to
/// ordinary argument parsing instead of being recognized as a help request.
/// Pitfall: this match is intentionally exact two-token equality on trimmed
/// content (`tokens.len() == 2 && tokens[ 1 ].trim() == "help"`) — do not
/// loosen it to a prefix/contains check (would swallow `.list helpme`) or
/// make it case-insensitive (would swallow `.list HELP`, currently a
/// distinct parse-error path); both are locked in as regression boundaries
/// by `t06_list_help_uppercase_unchanged`/`t07_list_helpme_content_near_miss_unchanged`.
///
/// Fix(BUG-005 gap 2, MAAV Tier 5 Round 3 G1)
/// Root cause: the `args.len() == 3` call site (two separate argv elements)
/// passed its tokens through unnormalized, unlike the REPL path
/// (`input.trim()` then `split_whitespace()`) and the `args.len() == 2`
/// fallback (`split_whitespace()`) — `clg ".list " "help"` or
/// `clg ".list" "help "` (leading/trailing whitespace baked into one argv
/// element) both reproduced the original BUG-005 symptom, confirmed live
/// against the compiled binary.
/// Pitfall: fix whitespace tolerance inside this shared matcher, not per
/// call site — every current and future caller gets the same tolerance
/// uniformly, rather than relying on each call site happening to
/// pre-normalize its own tokens correctly (2 of 3 did, by accident of using
/// `split_whitespace()` for an unrelated reason; the 3rd didn't).
fn try_command_help_space_form( binary : &str, registry : &CommandRegistry, tokens : &[ &str ] ) -> bool
{
  if tokens.len() != 2 || tokens[ 1 ].trim() != "help" { return false; }
  let Some( cmd ) = registry.command( tokens[ 0 ].trim() ) else { return false };
  print_command_help( binary, &cmd );
  true
}

/// Run REPL (Read-Eval-Print Loop) mode.
///
/// Intercepts global-help tokens (`.`, `.help`, `help`, ...) and
/// `<command>.help` tokens before pipeline dispatch, so both render via
/// `cli_fmt` instead of unilang's plain-text formatters.
fn run_repl( registry : CommandRegistry, binary : &str )
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
    // Fix(task-482)
    // Root cause: EOF is signaled in-band as `Ok(0)` — the previous
    // `if let Err` form treated it as empty input and `continue`d, and
    // reads at EOF return instantly, so the loop became a CPU-pegging
    // busy-spin escapable only by SIGINT/SIGKILL.
    // Pitfall: `read_line` never reports EOF through `Err`; every REPL
    // input loop needs an explicit `Ok(0)` exit arm.
    match io::stdin().read_line( &mut command_buffer )
    {
      Ok( 0 ) =>
      {
        // EOF (empty-line Ctrl+D or exhausted pipe): close the
        // unterminated `> ` prompt line, then exit like `exit` does.
        println!();
        println!( "Goodbye!" );
        break;
      }
      Ok( _ ) => {}
      Err( e ) =>
      {
        eprintln!( "Error reading input: {e}" );
        continue;
      }
    }

    let input = command_buffer.trim();

    if input.is_empty() { continue; }

    if input == "exit" || input == "quit" || input == "q"
    {
      println!( "Goodbye!" );
      break;
    }

    if is_global_help_token( input )
    {
      print_usage( binary );
      continue;
    }

    if try_command_help( binary, pipeline.registry(), input )
    {
      continue;
    }

    let tokens : Vec< &str > = input.split_whitespace().collect();
    if try_command_help_space_form( binary, pipeline.registry(), &tokens )
    {
      continue;
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
///
/// A single `<command>.help` argument is intercepted before the pipeline is
/// built, rendering via `cli_fmt` instead of unilang's auto-generated
/// `<command>.help` routine — see `try_command_help`.
#[ allow( clippy::needless_pass_by_value ) ]
fn execute_oneshot( registry : CommandRegistry, args : Vec< String >, binary : &str ) -> !
{
  if args.len() == 2
  {
    if try_command_help( binary, &registry, &args[ 1 ] )
    {
      process::exit( 0 );
    }
    // Fix(BUG-005 gap 1, MAAV Tier 5 Round 1 G1)
    // Root cause: a single quoted argv element (e.g. `clg ".list help"`,
    // args.len() == 2) carries both tokens joined by whitespace inside
    // args[ 1 ]; the dot-suffix check above doesn't match it, and the
    // separate-argv branch below is never reached (args.len() == 2, not 3).
    // Pitfall: tokenize the same way run_repl() tokenizes its input line
    // (split_whitespace) — don't assume args.len() == 2 only ever means
    // "one bare token".
    let tokens : Vec< &str > = args[ 1 ].split_whitespace().collect();
    if try_command_help_space_form( binary, &registry, &tokens )
    {
      process::exit( 0 );
    }
  }

  if args.len() == 3 && try_command_help_space_form( binary, &registry, &[ args[ 1 ].as_str(), args[ 2 ].as_str() ] )
  {
    process::exit( 0 );
  }

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
/// - Help: empty argv, `.`, `.help`, `--help`, `-h`, `help` → grouped help via `cli_fmt`
/// - REPL: `--repl` → interactive read-eval-print loop
/// - One-shot: any other args → execute command and exit
///
/// A single `<command>.help` argument (one-shot or REPL) renders that
/// command's detail via `cli_fmt` as well — see `try_command_help`.
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

  if is_global_help_token( first )
  {
    print_usage( &binary );
    process::exit( 0 );
  }

  if first == "--repl"
  {
    let registry = build_command_registry();
    run_repl( registry, &binary );
    return;
  }

  let registry = build_command_registry();
  execute_oneshot( registry, args, &binary );
}

//! `claude_profile` binary entry point.
//!
//! Implements the 5-phase unilang pipeline for account credential management:
//! Adapter → Parser → `SemanticAnalyzer` → Interpreter → stdout / stderr
//!
//! # Exit codes
//!
//! | Code | Meaning | Triggered by |
//! |------|---------|-------------|
//! | 0 | Success | Normal completion |
//! | 1 | Usage error | Unknown command, unknown param, invalid argument |
//! | 2 | Runtime error | `InternalError`, `CommandNotImplemented` |

use claude_profile::adapter::argv_to_unilang_tokens;
use claude_profile::commands::dot_routine;
use unilang::data::{ CommandDefinition, ErrorCode };
use unilang::interpreter::{ ExecutionContext, Interpreter };
use unilang::parser::{ Parser, UnilangParserOptions };
use unilang::registry::CommandRegistry;
use unilang::semantic::SemanticAnalyzer;

/// Map a unilang error to the appropriate exit code.
///
/// Usage errors (invalid input from the user) → 1.
/// Runtime errors (system failures during execution) → 2.
fn exit_code_for( e : &unilang::error::Error ) -> i32
{
  if let unilang::error::Error::Execution( ref data ) = e
  {
    match data.code
    {
      ErrorCode::InternalError | ErrorCode::CommandNotImplemented => 2,
      _ => 1,
    }
  }
  else
  {
    1
  }
}

/// Register all `claude_profile` commands with their argument definitions and routines.
///
/// Delegates 9 shared commands to `claude_profile::register_commands()` and
/// adds the `.` (dot) hidden command inline (binary-specific).
fn build_registry() -> CommandRegistry
{
  let mut registry = CommandRegistry::new();

  // `.` is hidden from the listing (adapter routes `.` → `.help`).
  {
    let def = CommandDefinition::former()
    .name( "." )
    .description( "Show help (alias for .help)" )
    .arguments( vec![] )
    .hidden_from_list( true )
    .end();
    registry
    .command_add_runtime( &def, Box::new( dot_routine ) )
    .expect( "internal error: failed to register ." );
  }

  // `.help` is pre-registered by CommandRegistry::new() — do not register again.

  // Register 9 shared commands (credentials, account, token, paths, usage).
  claude_profile::register_commands( &mut registry );

  registry
}

/// Print Claude Code-style structured usage to stdout.
///
/// Mirrors the format Claude Code itself uses: header, description, Commands
/// table, Options table, and Examples block — binary name is detected at
/// runtime so both `claude_profile` and `clp` show the correct invocation.
fn print_usage( binary : &str )
{
  // Column layout (measured from line start, 0-based):
  //   col  0-1  : 2-space indent
  //   col  2-18 : command name, padded to 17 chars
  //   col 19-46 : parameters  (28 chars for read cmds, 24 for write cmds)
  //   col 47-49 : gap (3+ spaces, making descriptions land on col 50)
  //   col 50+   : description
  //
  // Options column: names padded to 17 chars → descriptions at col 20.
  println!( "Usage: {binary} [command] [key::value ...]" );
  println!();
  println!( "Manage Claude Code account credentials and token state." );
  println!();
  println!( "Commands:" );
  println!( "  .account.list        [v::0-2] [format::text|json]   List all saved accounts" );
  println!( "  .account.status      [v::0-2] [format::text|json]   Show active account and token state" );
  println!( "  .account.save        name::STRING [dry::bool]       Save current credentials as named account" );
  println!( "  .account.switch      name::STRING [dry::bool]       Switch active account" );
  println!( "  .account.delete      name::STRING [dry::bool]       Delete a saved account" );
  println!( "  .token.status        [v::0-2] [format::text|json]   Show OAuth token expiry status" );
  println!( "  .paths               [v::0-2] [format::text|json]   Show all ~/.claude/ canonical paths" );
  println!( "  .usage               [v::0-2] [format::text|json]   Show 7-day token usage summary" );
  println!( "  .credentials.status  [v::0-2] [format::text|json]   Show live credentials (no account store needed)" );
  println!();
  println!( "Options:" );
  println!( "  v::0-2              Verbosity level (default: 1)" );
  println!( "  format::text|json   Output format (default: text)" );
  println!( "  dry::bool           Preview without applying" );
  println!( "  name::STRING        Account name" );
  println!();
  println!( "Examples:" );
  println!( "  {binary} .account.list" );
  println!( "  {binary} .account.list v::2" );
  println!( "  {binary} .account.switch name::work" );
  println!( "  {binary} .account.switch name::work dry::true" );
  println!( "  {binary} .token.status format::json" );
  println!( "  {binary} .paths v::2" );
  println!( "  {binary} .usage" );
  println!( "  {binary} .usage v::2" );
  println!( "  {binary} .credentials.status" );
}

fn main()
{
  // Detect the invoked binary name for usage messages (`claude_profile` or `clp`).
  let binary = std::env::args()
  .next()
  .as_deref()
  .and_then( | p | std::path::Path::new( p ).file_name() )
  .and_then( | n | n.to_str() )
  .unwrap_or( "clp" )
  .to_owned();

  let argv : Vec< String > = std::env::args().skip( 1 ).collect();

  // Phase 1: adapter — convert argv to unilang tokens.
  let ( tokens, needs_help ) = match argv_to_unilang_tokens( &argv )
  {
    Ok( r )  => r,
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      eprintln!( "Run '{binary} --help' for usage." );
      std::process::exit( 1 );
    }
  };

  // Intercept help requests before entering the unilang pipeline.
  // Triggered by: no args, `.`, `--help`, `-h`.
  // Explicit `.help` does NOT set needs_help, so it still goes through unilang.
  if needs_help
  {
    print_usage( &binary );
    return;
  }

  let registry = build_registry();

  // Phase 2: parse — convert token vec to GenericInstruction.
  let parser = Parser::new( UnilangParserOptions::default() );
  let instruction = match parser.parse_from_argv( &tokens )
  {
    Ok( i )  => i,
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  };

  // Phase 3: semantic analysis — validate instruction against registered commands.
  let instructions = [ instruction ];
  let analyzer     = SemanticAnalyzer::new( &instructions, &registry );
  let commands = match analyzer.analyze()
  {
    Ok( cmds ) => cmds,
    Err( e )   =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( exit_code_for( &e ) );
    }
  };

  // Phase 4: execute — run command routines.
  let interpreter = Interpreter::new( &commands, &registry );
  let mut context = ExecutionContext::default();
  match interpreter.run( &mut context )
  {
    Ok( outputs ) =>
    {
      for out in outputs
      {
        print!( "{}", out.content );
      }
    }
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( exit_code_for( &e ) );
    }
  }
}

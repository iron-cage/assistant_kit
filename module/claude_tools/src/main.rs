//! `clt` — Claude Tools super-app binary.
//!
//! Aggregates commands from all four Layer 2 `claude_*` crates into a single CLI.
//!
//! Commands are registered from:
//! - `claude_manager::register_commands()` — `.status`, `.version.*`, `.processes.*`, `.settings.*`
//! - `claude_profile::register_commands()` — all `.account.*`, `.token.status`, `.paths`, `.usage`
//! - `static_commands.rs` (build.rs YAML aggregation) — `.claude.*` and storage commands
//!
//! # Exit codes
//!
//! | Code | Meaning |
//! |------|---------|
//! | 0 | Success |
//! | 1 | Usage error |
//! | 2 | Runtime error |

// allow: binary entry point; generated static_commands.rs lacks doc comments on pub statics
#![ allow( missing_docs ) ]

use claude_manager::adapter::argv_to_unilang_tokens;
use unilang::data::{ CommandDefinition, ErrorCode, ErrorData, OutputData };
use unilang::interpreter::{ ExecutionContext, Interpreter };
use unilang::parser::{ Parser, UnilangParserOptions };
use unilang::phf;
use unilang::registry::CommandRegistry;
use unilang::semantic::{ SemanticAnalyzer, VerifiedCommand };

// Include compile-time generated static commands (runner + storage YAML)
include!( concat!( env!( "OUT_DIR" ), "/static_commands.rs" ) );

/// Map a unilang error to the appropriate exit code.
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

/// Stub for `.claude` and `.claude.help` — subprocess routing is out of scope for `clt`.
#[ allow( clippy::unnecessary_wraps ) ]
fn claude_stub_routine( _cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
{
  Ok( OutputData::new( "For Claude Code execution, use clr directly.\n".to_string(), "text" ) )
}

/// Register static YAML-sourced commands (runner + storage) into the registry.
///
/// Skips duplicates silently — e.g., `.status` from storage is already
/// registered by `claude_manager::register_commands()` and the manager
/// version takes precedence.  After the `.sessions` → `.processes` rename
/// in manager, storage's `.sessions` (scope-based session listing) is now
/// registered without collision.
fn register_static_commands( registry : &mut CommandRegistry )
{
  type RoutineFn = fn( VerifiedCommand, ExecutionContext ) -> Result< OutputData, ErrorData >;

  let routines : phf::Map< &'static str, RoutineFn > = phf::phf_map!
  {
    ".claude"       => claude_stub_routine,
    ".claude.help"  => claude_stub_routine,
    ".status"       => claude_storage::cli::status_routine,
    ".list"         => claude_storage::cli::list_routine,
    ".show"         => claude_storage::cli::show_routine,
    ".show.project" => claude_storage::cli::show_project_routine,
    ".count"        => claude_storage::cli::count_routine,
    ".search"       => claude_storage::cli::search_routine,
    ".export"       => claude_storage::cli::export_routine,
    ".session"      => claude_storage::cli::session_routine,
    ".sessions"     => claude_storage::cli::sessions_routine,
  };

  for ( name, static_cmd ) in AGGREGATED_COMMANDS.entries()
  {
    if let Some( &routine ) = routines.get( *name )
    {
      let cmd : CommandDefinition = ( *static_cmd ).into();
      #[ allow( deprecated ) ]
      // Silently skip duplicates (e.g., .status already registered by claude_manager)
      let _ = registry.command_add_runtime( &cmd, Box::new( routine ) );
    }
  }
}

fn build_registry() -> CommandRegistry
{
  let mut registry = CommandRegistry::new();
  // Registration order determines first-wins precedence for overlapping command names.
  // After plan-005: account commands live exclusively in claude_profile; no overlap with claude_manager.
  claude_assets::register_commands( &mut registry );
  claude_manager::register_commands( &mut registry );
  claude_profile::register_commands( &mut registry );
  claude_runner::register_commands( &mut registry );
  claude_storage::register_commands( &mut registry );
  register_static_commands( &mut registry );
  registry
}

fn main()
{
  let argv : Vec< String > = std::env::args().skip( 1 ).collect();

  let ( tokens, _needs_help ) = match argv_to_unilang_tokens( &argv )
  {
    Ok( r )  => r,
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  };

  let registry = build_registry();

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

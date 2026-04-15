//! `claude_assets` — Claude Code artifact installer CLI (`cla`).
//!
//! Provides symlink-based install/uninstall of rules, commands, agents,
//! skills, plugins, and hooks from `$PRO_CLAUDE` into `.claude/<kind>/`.
//!
//! # Feature Gate
//!
//! All modules require the `enabled` feature. Without it the crate compiles
//! to an empty shell — the intended behaviour for library crates in this workspace.

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]
#![ warn( missing_docs ) ]
#![ warn( missing_debug_implementations ) ]

/// Path to the YAML command definitions for this crate.
///
/// Used by `assistant/build.rs` for `rerun-if-changed` watching.
/// Commands are registered programmatically via [`register_commands()`].
pub const COMMANDS_YAML : &str = concat!( env!( "CARGO_MANIFEST_DIR" ), "/unilang.commands.yaml" );

#[ cfg( feature = "enabled" ) ]
mod adapter;

#[ cfg( feature = "enabled" ) ]
/// Command handler routines for each CLI subcommand.
pub mod commands;

#[ cfg( feature = "enabled" ) ]
fn reg_arg_opt( name : &str, kind : unilang::data::Kind ) -> unilang::data::ArgumentDefinition
{
  unilang::data::ArgumentDefinition::new( name, kind ).with_optional( None::< String > )
}

#[ cfg( feature = "enabled" ) ]
fn reg_cmd(
  registry : &mut unilang::registry::CommandRegistry,
  name     : &str,
  desc     : &str,
  args     : Vec< unilang::data::ArgumentDefinition >,
  routine  : unilang::registry::CommandRoutine,
)
{
  let def = unilang::data::CommandDefinition::former()
  .name( name )
  .description( desc )
  .arguments( args )
  .end();
  registry
  .command_add_runtime( &def, routine )
  .expect( "internal error: failed to register command" );
}

#[ cfg( feature = "enabled" ) ]
/// Register all `claude_assets` commands into an existing registry.
///
/// Used by both the standalone binary (`run_cli`) and by `assistant`
/// to aggregate commands from multiple crates into one shared registry.
///
/// # Panics
///
/// Panics if a command fails to register (duplicate name — programming error).
#[ inline ]
pub fn register_commands( registry : &mut unilang::registry::CommandRegistry )
{
  use unilang::data::Kind;
  use commands::{ list_routine, install_routine, uninstall_routine, kinds_routine };

  let kind      = || reg_arg_opt( "kind",      Kind::String  );
  let name      = || reg_arg_opt( "name",      Kind::String  );
  let installed = || reg_arg_opt( "installed", Kind::Boolean );
  let v         = || reg_arg_opt( "verbosity", Kind::Integer );

  reg_cmd( registry, ".list",      "List available and installed Claude Code artifacts",         vec![ kind(), installed(), v() ],  Box::new( list_routine      ) );
  reg_cmd( registry, ".install",   "Install a named artifact via symlink into .claude/<kind>/",  vec![ kind(), name() ],            Box::new( install_routine   ) );
  reg_cmd( registry, ".uninstall", "Remove an installed artifact symlink from .claude/<kind>/",  vec![ kind(), name() ],            Box::new( uninstall_routine ) );
  reg_cmd( registry, ".kinds",     "Show all supported artifact kinds with source and target path mappings",       vec![],                            Box::new( kinds_routine     ) );
}

#[ cfg( feature = "enabled" ) ]
/// Run the `claude_assets` CLI — 5-phase unilang pipeline.
///
/// Entry point shared by both the `claude_assets` and `cla` binaries so
/// the pipeline is compiled once rather than twice.
///
/// # Exit codes
///
/// | Code | Meaning |
/// |------|---------|
/// | 0    | Success |
/// | 1    | Usage / input error |
/// | 2    | Runtime error |
#[ inline ]
pub fn run_cli()
{
  use adapter::argv_to_unilang_tokens;
  use unilang::data::ErrorCode;
  use unilang::interpreter::{ ExecutionContext, Interpreter };
  use unilang::parser::{ Parser, UnilangParserOptions };
  use unilang::registry::CommandRegistry;
  use unilang::semantic::SemanticAnalyzer;

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

  let mut registry = CommandRegistry::new();
  register_commands( &mut registry );

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

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
  .register_with_routine( &def, routine )
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

  reg_cmd( registry, ".list",      "List available and installed Claude Code artifacts",         vec![ kind(), installed() ],       Box::new( list_routine      ) );
  reg_cmd( registry, ".install",   "Install a named artifact via symlink into .claude/<kind>/",  vec![ kind(), name() ],            Box::new( install_routine   ) );
  reg_cmd( registry, ".uninstall", "Remove an installed artifact symlink from .claude/<kind>/",  vec![ kind(), name() ],            Box::new( uninstall_routine ) );
  reg_cmd( registry, ".kinds",     "Show all supported artifact kinds with source and target path mappings",       vec![],                            Box::new( kinds_routine     ) );
}

/// Render grouped help output via `cli_fmt::CliHelpTemplate`.
///
/// Displays 1 command group (Asset Management), shared parameters,
/// and usage examples.
#[ cfg( feature = "enabled" ) ]
fn print_usage( binary : &str )
{
  use cli_fmt::help::*;

  let mut data    = CliHelpData::default();
  data.binary     = binary.to_string();
  data.tagline    = "Claude Code artifact installer: manage rules, skills, commands, and hooks via symlinks.".to_string();
  data.groups     = vec!
  [
    CommandGroup
    {
      name    : "Asset Management".to_string(),
      entries : vec!
      [
        CommandEntry { name : ".list".to_string(),      desc : "List available and installed Claude Code artifacts".to_string() },
        CommandEntry { name : ".install".to_string(),   desc : "Install a named artifact via symlink into .claude/<kind>/".to_string() },
        CommandEntry { name : ".uninstall".to_string(), desc : "Remove an installed artifact symlink from .claude/<kind>/".to_string() },
        CommandEntry { name : ".kinds".to_string(),     desc : "Show all artifact kinds with source and target path mappings".to_string() },
      ],
    },
  ];
  data.options    = vec!
  [
    OptionEntry { name : "kind::KIND".to_string(),     desc : "Filter by artifact kind (e.g. rule, skill, command)".to_string() },
    OptionEntry { name : "name::NAME".to_string(),     desc : "Artifact name to install or uninstall".to_string() },
    OptionEntry { name : "installed::0|1".to_string(), desc : "Show only installed (1) or uninstalled (0)".to_string() },
  ];
  data.examples   = vec!
  [
    ExampleEntry { invocation : format!( "{binary} .list" ),                             desc : None },
    ExampleEntry { invocation : format!( "{binary} .install kind::rule name::my_rule" ), desc : None },
    ExampleEntry { invocation : format!( "{binary} .kinds" ),                            desc : None },
  ];
  print!( "{}", CliHelpTemplate::new( CliHelpStyle::default(), data ).render() );
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

  let binary = std::env::args()
  .next()
  .as_deref()
  .and_then( | p | std::path::Path::new( p ).file_name() )
  .and_then( | n | n.to_str() )
  .unwrap_or( "cla" )
  .to_owned();

  let argv : Vec< String > = std::env::args().skip( 1 ).collect();

  let ( tokens, needs_help ) = match argv_to_unilang_tokens( &argv )
  {
    Ok( r )  => r,
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  };

  if needs_help
  {
    print_usage( &binary );
    std::process::exit( 0 );
  }

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

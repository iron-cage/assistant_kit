//! `runbox` — container runner scaffold CLI (`crb`).
//!
//! Generates `runbox/runbox`, `runbox/runbox.yml`, and `runbox/runbox.dockerfile`
//! for integrating a project with the container runner system.
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
/// Used by consumer `build.rs` for `rerun-if-changed` watching.
/// Commands are registered programmatically via [`register_commands()`].
pub const COMMANDS_YAML : &str = concat!( env!( "CARGO_MANIFEST_DIR" ), "/unilang.commands.yaml" );

#[ cfg( feature = "enabled" ) ]
mod adapter;

#[ cfg( feature = "enabled" ) ]
/// Command handler routines for each CLI subcommand.
pub mod commands;

#[ cfg( feature = "enabled" ) ]
/// File content templates for generated container runner files.
pub mod templates;

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
/// Register all `runbox` commands into an existing registry.
///
/// Used by the standalone binaries (`run_cli`).
///
/// # Panics
///
/// Panics if a command fails to register (duplicate name — programming error).
#[ inline ]
pub fn register_commands( registry : &mut unilang::registry::CommandRegistry )
{
  use unilang::data::Kind;
  use commands::init_routine;

  let image       = || reg_arg_opt( "image",       Kind::String );
  let ecosystem   = || reg_arg_opt( "ecosystem",   Kind::String );
  let test_script = || reg_arg_opt( "test_script", Kind::String );

  reg_cmd(
    registry,
    ".init",
    "Scaffold container runner integration files in the current directory",
    vec![ image(), ecosystem(), test_script() ],
    Box::new( init_routine ),
  );
}

#[ cfg( feature = "enabled" ) ]
fn print_usage( binary : &str )
{
  use cli_fmt::help::*;

  let data = CliHelpData
  {
    binary  : binary.to_string(),
    tagline : "Container runner scaffold: generate integration files for a project.".to_string(),
    groups  : vec!
    [
      CommandGroup
      {
        name    : "Scaffold".to_string(),
        entries : vec!
        [
          CommandEntry
          {
            name : ".init".to_string(),
            desc : "Scaffold container runner integration files in the current directory".to_string(),
          },
        ],
      },
    ],
    options : vec!
    [
      OptionEntry
      {
        name : "image::IMAGE".to_string(),
        desc : "Docker image tag for this project (required)".to_string(),
      },
      OptionEntry
      {
        name : "ecosystem::ECOSYSTEM".to_string(),
        desc : "Project ecosystem: rust, nodejs, python, none (default: none)".to_string(),
      },
      OptionEntry
      {
        name : "test_script::PATH".to_string(),
        desc : "Test script path inside container (default: verb/test.d/l1)".to_string(),
      },
    ],
    examples : vec!
    [
      ExampleEntry { invocation : format!( "{binary} .init image::my_project" ),                                         desc : None },
      ExampleEntry { invocation : format!( "{binary} .init image::my_project ecosystem::rust" ),                         desc : None },
      ExampleEntry { invocation : format!( "{binary} .init image::my_project ecosystem::python test_script::verb/test" ), desc : None },
    ],
  };
  print!( "{}", CliHelpTemplate::new( CliHelpStyle::default(), data ).render() );
}

#[ cfg( feature = "enabled" ) ]
/// Run the `runbox` CLI — 5-phase unilang pipeline.
///
/// Entry point shared by both the `runbox` and `crb` binaries so
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
  .unwrap_or( "crb" )
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

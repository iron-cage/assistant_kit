//! `claude_manager` — manage Claude Code installation, versions, and process lifecycle.
//!
//! See [`spec.md`](../spec.md) for complete requirements and architecture.
//!
//! # Feature Gate
//!
//! All modules require the `enabled` feature. Without it the crate compiles to an empty
//! shell, which is the intended behaviour for library crates in this workspace.

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]
#![ warn( missing_docs ) ]
#![ warn( missing_debug_implementations ) ]

/// Path to the YAML command definitions for this crate.
///
/// Used by `claude_tools/build.rs` for metadata-only export. Manager commands
/// are registered programmatically via [`register_commands()`], not via YAML aggregation.
pub const COMMANDS_YAML : &str = concat!( env!( "CARGO_MANIFEST_DIR" ), "/unilang.commands.yaml" );

#[ cfg( feature = "enabled" ) ]
pub mod adapter;

#[ cfg( feature = "enabled" ) ]
pub use claude_manager_core::settings_io;

#[ cfg( feature = "enabled" ) ]
pub mod output;

#[ cfg( feature = "enabled" ) ]
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
/// Register all `claude_manager` commands into an existing registry.
///
/// Used by both the standalone binary (`run_cli`) and by `claude_tools` (Layer 3)
/// to aggregate commands from multiple crates into one shared registry.
///
/// # Panics
///
/// Panics if a command fails to register (would indicate a duplicate name,
/// which is a programming error).
#[ inline ]
pub fn register_commands( registry : &mut unilang::registry::CommandRegistry )
{
  use unilang::data::Kind;
  use commands::
  {
    status_routine, version_show_routine, version_install_routine,
    version_guard_routine, version_list_routine, version_history_routine,
    processes_routine, processes_kill_routine,
    settings_show_routine, settings_get_routine, settings_set_routine,
  };
  let v   = || reg_arg_opt( "verbosity", Kind::Integer );
  let fmt = || reg_arg_opt( "format",    Kind::String  );
  let dry = || reg_arg_opt( "dry",       Kind::Boolean );
  let frc = || reg_arg_opt( "force",     Kind::Boolean );
  let ver = || reg_arg_opt( "version",   Kind::String  );
  let key = || reg_arg_opt( "key",       Kind::String  );
  let val = || reg_arg_opt( "value",     Kind::String  );
  let itv = || reg_arg_opt( "interval",  Kind::Integer );
  let cnt = || reg_arg_opt( "count",     Kind::Integer );

  reg_cmd( registry, ".status",          "Show installation state, process count, and active account", vec![ v(), fmt() ],                      Box::new( status_routine          ) );
  reg_cmd( registry, ".version.show",    "Print the currently installed Claude Code version",          vec![ v(), fmt() ],                      Box::new( version_show_routine    ) );
  reg_cmd( registry, ".version.install", "Download and install a Claude Code version via installer",   vec![ ver(), dry(), frc(), v(), fmt() ], Box::new( version_install_routine ) );
  reg_cmd( registry, ".version.guard",   "Check for version drift and restore preferred version",      vec![ ver(), dry(), frc(), itv(), v() ], Box::new( version_guard_routine   ) );
  reg_cmd( registry, ".version.list",    "List all named version aliases",                             vec![ v(), fmt() ],                      Box::new( version_list_routine    ) );
  reg_cmd( registry, ".version.history", "Show release history with changelogs from GitHub",           vec![ cnt(), v(), fmt() ],               Box::new( version_history_routine ) );
  reg_cmd( registry, ".processes",       "List all running Claude Code processes",                     vec![ v(), fmt() ],                      Box::new( processes_routine       ) );
  reg_cmd( registry, ".processes.kill",  "Terminate all Claude Code processes",                        vec![ dry(), frc() ],                    Box::new( processes_kill_routine  ) );
  reg_cmd( registry, ".settings.show",   "Print all settings from ~/.claude/settings.json",            vec![ v(), fmt() ],                      Box::new( settings_show_routine   ) );
  reg_cmd( registry, ".settings.get",    "Read a single setting by key",                               vec![ key(), v(), fmt() ],               Box::new( settings_get_routine    ) );
  reg_cmd( registry, ".settings.set",    "Write a single setting atomically",                          vec![ key(), val(), dry() ],             Box::new( settings_set_routine    ) );
}

#[ cfg( feature = "enabled" ) ]
/// Run the `claude_manager` CLI — 5-phase unilang pipeline.
///
/// Entry point shared by both the `claude_manager` and `clman` binaries so
/// the pipeline is compiled once rather than twice (avoids the Cargo
/// "same file in multiple `[[bin]]` targets" warning).
///
/// # Exit codes
///
/// | Code | Meaning |
/// |------|---------|
/// | 0    | Success |
/// | 1    | Usage / input error |
/// | 2    | Runtime error (`InternalError`, `CommandNotImplemented`) |
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

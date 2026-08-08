//! `claude_version` — Claude Code version manager: install, upgrade, and session lifecycle.
//!
//! See `docs/feature/`, `docs/pattern/`, and `docs/algorithm/` for requirements and architecture.
//!
//! # Feature Gate
//!
//! All modules require the `enabled` feature. Without it the crate compiles to an empty
//! shell, which is the intended behaviour for library crates in this workspace.

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]
#![ warn( missing_docs ) ]
#![ warn( missing_debug_implementations ) ]
#![ doc( html_root_url = "https://docs.rs/claude_version/" ) ]

/// Path to the YAML command definitions for this crate.
///
/// Used by `assistant/build.rs` for metadata-only export. Manager commands
/// are registered programmatically via [`register_commands()`], not via YAML aggregation.
pub const COMMANDS_YAML : &str = concat!( env!( "CARGO_MANIFEST_DIR" ), "/unilang.commands.yaml" );

#[ cfg( feature = "enabled" ) ]
pub mod adapter;

#[ cfg( feature = "enabled" ) ]
pub use claude_core::settings_io;

#[ cfg( feature = "enabled" ) ]
pub mod output;

#[ cfg( feature = "enabled" ) ]
pub mod commands;

#[ cfg( feature = "enabled" ) ]
fn reg_arg_opt(
  name    : &str,
  kind    : unilang::data::Kind,
  desc    : &str,
  default : Option< &str >,
) -> unilang::data::ArgumentDefinition
{
  unilang::data::ArgumentDefinition::new( name, kind )
  .with_description( desc )
  .with_optional( default )
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

/// Register all `claude_version` commands into an existing registry.
///
/// Used by both the standalone binary (`run_cli`) and by `assistant` (Layer 3)
/// to aggregate commands from multiple crates into one shared registry.
///
/// # Panics
///
/// Panics if a command fails to register (would indicate a duplicate name,
/// which is a programming error).
#[ cfg( feature = "enabled" ) ]
#[ inline ]
pub fn register_commands( registry : &mut unilang::registry::CommandRegistry )
{
  use unilang::data::Kind;
  use commands::
  {
    status_routine, version_show_routine, version_install_routine,
    version_guard_routine, version_list_routine, version_mark_routine,
    ps_routine, ps_kill_routine,
    settings_show_routine, settings_get_routine, settings_set_routine,
    config_routine, params_routine, runtime_files_routine, paths_routine,
  };
  let v   = || reg_arg_opt( "verbosity",   Kind::Integer, "Output detail level (0=quiet, 1=normal, 2=verbose).",          Some( "1"       ) );
  let fmt = || reg_arg_opt( "format",      Kind::String,  "Output format (text or json).",                                 Some( "text"    ) );
  let dry = || reg_arg_opt( "dry",         Kind::Boolean, "Preview the action without executing side effects.",            Some( "0"       ) );
  let frc = || reg_arg_opt( "force",       Kind::Boolean, "Bypass safety guards and idempotency checks.",                 Some( "0"       ) );
  let rec = || reg_arg_opt( "record_only", Kind::Boolean, "Persist version preference without installing.",               Some( "0"       ) );
  let ver = || reg_arg_opt( "version",     Kind::String,  "Version alias or semver to install or guard against.",         None             );
  let key = || reg_arg_opt( "key",         Kind::String,  "Settings key to read, write, or remove.",                      None             );
  let val = || reg_arg_opt( "value",       Kind::String,  "Settings value to write.",                                     None             );
  let itv = || reg_arg_opt( "interval",    Kind::Integer, "Guard check frequency in seconds; 0 = one-shot.",              Some( "0"       ) );
  let cnt = || reg_arg_opt( "count",       Kind::Integer, "Maximum history entries to show.",                             Some( "10"      ) );
  let scp = || reg_arg_opt( "scope",       Kind::String,  "Configuration write scope (user or project).",                 None             );
  let nme = || reg_arg_opt( "name",        Kind::String,  "Custom marker name for .version.mark.",                        None             );
  let dsc = || reg_arg_opt( "description", Kind::String,  "Human-readable description for a custom marker.",              None             );
  let uns = || reg_arg_opt( "unset",       Kind::Boolean, "Remove the specified key from the target scope.",              Some( "0"       ) );
  let knd = || reg_arg_opt( "kind",        Kind::String,  "Parameter kind filter for .params (config, env, or absent=all).", None          );
  let md  = || reg_arg_opt( "mode",        Kind::String,  "Operation mode for .version.list (aliases or history).",      Some( "aliases" ) );
  let pid = || reg_arg_opt( "pid",         Kind::Integer, "Target a single process by pid (bulk-kill all when absent).", None             );

  reg_cmd( registry, ".status",          "Show installation state, process count, and active account", vec![ v(), fmt() ],                      Box::new( status_routine          ) );
  reg_cmd( registry, ".version.show",    "Print the currently installed Claude Code version",          vec![ v(), fmt() ],                      Box::new( version_show_routine    ) );
  reg_cmd( registry, ".version.install", "Download and install a Claude Code version via installer",   vec![ ver(), dry(), frc(), rec(), v(), fmt() ], Box::new( version_install_routine ) );
  reg_cmd( registry, ".version.guard",   "Check for version drift and restore preferred version",      vec![ ver(), dry(), frc(), itv(), v(), fmt() ], Box::new( version_guard_routine   ) );
  reg_cmd( registry, ".version.list",    "List named version aliases (mode::aliases) or recent release history (mode::history)", vec![ md(), cnt(), v(), fmt() ], Box::new( version_list_routine    ) );
  reg_cmd( registry, ".version.mark",    "Create, update, or remove a custom version alias marker",                              vec![ nme(), ver(), dsc(), uns(), dry(), v(), fmt() ], Box::new( version_mark_routine ) );
  reg_cmd( registry, ".ps",              "List all running Claude Code processes",                     vec![ v(), fmt() ],                      Box::new( ps_routine              ) );
  reg_cmd( registry, ".ps.kill",         "Terminate all Claude Code processes, or one via pid::",       vec![ pid(), dry(), frc(), v(), fmt() ], Box::new( ps_kill_routine         ) );
  reg_cmd( registry, ".settings.show",   "Print all settings from ~/.claude/settings.json",            vec![ v(), fmt() ],                      Box::new( settings_show_routine   ) );
  reg_cmd( registry, ".settings.get",    "Read a single setting by key",                               vec![ key(), v(), fmt() ],               Box::new( settings_get_routine    ) );
  reg_cmd( registry, ".settings.set",    "Write a single setting atomically",                          vec![ key(), val(), dry() ],             Box::new( settings_set_routine    ) );
  reg_cmd( registry, ".config",          "Show, get, set, or unset settings with 4-layer resolution",  vec![ key(), val(), scp(), uns(), dry(), v(), fmt() ], Box::new( config_routine ) );
  reg_cmd( registry, ".params",          "Inspect Claude Code params: forms, current values, defaults", vec![ key(), knd(), v(), fmt() ],       Box::new( params_routine          ) );
  reg_cmd( registry, ".runtime_files",   "List all paths managed by clv at runtime",                   vec![],                                  Box::new( runtime_files_routine   ) );
  reg_cmd( registry, ".version.paths",           "Report filesystem paths clv reads from or writes to",        vec![ key(), fmt(), v() ],               Box::new( paths_routine           ) );
}

/// Render grouped help output via `cli_fmt::CliHelpTemplate`.
///
/// Displays 4 command groups (Version Management, Settings & Config,
/// Process Lifecycle, Status), shared parameters, and usage examples.
#[ cfg( feature = "enabled" ) ]
fn print_usage( binary : &str )
{
  use cli_fmt::help::*;

  let mut data    = CliHelpData::default();
  data.binary     = binary.to_string();
  data.tagline    = "Claude Code version manager: install, upgrade, and session lifecycle.".to_string();
  data.groups     = vec!
  [
    CommandGroup
    {
      name    : "Version Management".to_string(),
      entries : vec!
      [
        CommandEntry { name : ".version.show".to_string(),    desc : "Print the currently installed Claude Code version".to_string() },
        CommandEntry { name : ".version.install".to_string(), desc : "Download and install a Claude Code version via installer".to_string() },
        CommandEntry { name : ".version.guard".to_string(),   desc : "Check for version drift and restore preferred version".to_string() },
        CommandEntry { name : ".version.list".to_string(),    desc : "Lists aliases or release history relevant to the installed version".to_string() },
        CommandEntry { name : ".version.mark".to_string(),    desc : "Create, update, or remove a custom version alias marker".to_string() },
      ],
    },
    CommandGroup
    {
      name    : "Settings & Config".to_string(),
      entries : vec!
      [
        CommandEntry { name : ".settings.show".to_string(), desc : "Print all settings from ~/.claude/settings.json".to_string() },
        CommandEntry { name : ".settings.get".to_string(),  desc : "Read a single setting by key".to_string() },
        CommandEntry { name : ".settings.set".to_string(),  desc : "Write a single setting atomically".to_string() },
        CommandEntry { name : ".config".to_string(),        desc : "Show, get, set, or unset settings with 4-layer resolution".to_string() },
        CommandEntry { name : ".params".to_string(),        desc : "Inspect Claude Code params: forms, current values, defaults".to_string() },
      ],
    },
    CommandGroup
    {
      name    : "Process Lifecycle".to_string(),
      entries : vec!
      [
        CommandEntry { name : ".ps".to_string(),      desc : "List all running Claude Code processes".to_string() },
        CommandEntry { name : ".ps.kill".to_string(), desc : "Terminate all Claude Code processes, or one via pid::".to_string() },
      ],
    },
    CommandGroup
    {
      name    : "Status".to_string(),
      entries : vec!
      [
        CommandEntry { name : ".status".to_string(),        desc : "Show installation state, process count, and active account".to_string() },
        CommandEntry { name : ".runtime_files".to_string(), desc : "List all paths managed by clv at runtime".to_string() },
        CommandEntry { name : ".version.paths".to_string(),         desc : "Report filesystem paths clv reads from or writes to".to_string() },
      ],
    },
  ];
  data.options    = vec!
  [
    OptionEntry { name : "v::0|1|2".to_string(),        desc : "Verbosity level (default: 1)".to_string() },
    OptionEntry { name : "format::text|json".to_string(), desc : "Output format (default: text)".to_string() },
    OptionEntry { name : "dry::bool".to_string(),       desc : "Dry-run preview (no changes)".to_string() },
    OptionEntry { name : "force::bool".to_string(),     desc : "Force operation (skip confirmations)".to_string() },
    OptionEntry { name : "record_only::bool".to_string(), desc : "Record preference only, skip the actual install (.version.install)".to_string() },
    OptionEntry { name : "key::KEY".to_string(),        desc : "Setting key".to_string() },
    OptionEntry { name : "value::VALUE".to_string(),    desc : "Setting value".to_string() },
    OptionEntry { name : "interval::N".to_string(),     desc : "Polling interval in seconds".to_string() },
    OptionEntry { name : "count::N".to_string(),        desc : "Maximum entries to show".to_string() },
  ];
  data.examples   = vec!
  [
    ExampleEntry { invocation : format!( "{binary} .status" ),                    desc : None },
    ExampleEntry { invocation : format!( "{binary} .version.install" ),            desc : None },
    ExampleEntry { invocation : format!( "{binary} .settings.get key::model" ),    desc : None },
    ExampleEntry { invocation : format!( "{binary} .ps" ),                        desc : None },
    ExampleEntry { invocation : format!( "{binary} .config key::model" ),          desc : None },
  ];
  print!( "{}", CliHelpTemplate::new( CliHelpStyle::default(), data ).render() );
}

/// Render per-command help for a single registered command via `cli_fmt`.
///
/// Looks up `name` in `registry`, builds a `CliHelpData` with the command's
/// description and all its arguments (description + default suffix), then
/// prints the rendered output.  Prints an error to stderr if the command is
/// not found and returns without printing usage.
#[ cfg( feature = "enabled" ) ]
#[ inline ]
pub fn print_command_help( name : &str, registry : &unilang::registry::CommandRegistry )
{
  use cli_fmt::help::*;
  let Some( def ) = registry.command( name ) else
  {
    eprintln!( "unknown command: {name}" );
    return;
  };
  let mut data = CliHelpData::default();
  data.binary  = name.to_string();
  data.tagline = def.description().to_string();
  data.options = def.arguments().iter().map( | arg |
  {
    let default_part = arg.attributes.default
    .as_deref()
    .map( | d | format!( " (default: {d})" ) )
    .unwrap_or_default();
    OptionEntry
    {
      name : arg.name.clone(),
      desc : format!( "{}{}", arg.description, default_part ),
    }
  } ).collect();
  print!( "{}", CliHelpTemplate::new( CliHelpStyle::default(), data ).render() );
}

/// Run the `claude_version` CLI — 5-phase unilang pipeline.
///
/// Entry point shared by both the `claude_version` and `clv` binaries so
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
#[ cfg( feature = "enabled" ) ]
#[ inline ]
pub fn run_cli()
{
  use adapter::{ argv_to_unilang_tokens, HelpMode };
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
  .unwrap_or( "clv" )
  .to_owned();

  let argv : Vec< String > = std::env::args().skip( 1 ).collect();

  let ( tokens, help_mode ) = match argv_to_unilang_tokens( &argv )
  {
    Ok( r )  => r,
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  };

  match help_mode
  {
    HelpMode::Global =>
    {
      print_usage( &binary );
      std::process::exit( 0 );
    }
    HelpMode::Command( name ) =>
    {
      let mut registry = CommandRegistry::new();
      register_commands( &mut registry );
      print_command_help( &name, &registry );
      std::process::exit( 0 );
    }
    HelpMode::None => {}
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

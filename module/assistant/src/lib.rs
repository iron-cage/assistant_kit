//! `assistant` — agent-agnostic super-app aggregating all coding agent CLI tools.
//!
//! Aggregates commands from all Layer 2 crates:
//! - [`claude_version`]: version management, settings, session, account commands
//! - [`claude_runner`]: `.claude` AI-assistance command (YAML-based)
//! - [`claude_storage`]: storage exploration commands (YAML-based)
//!
//! # Feature Gate
//!
//! All modules require the `enabled` feature. Without it the crate compiles to an empty
//! shell, which is the intended behaviour for library crates in this workspace.

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]
#![ warn( missing_docs ) ]
#![ warn( missing_debug_implementations ) ]

#[ cfg( feature = "enabled" ) ]
mod generated
{
  #![ allow( missing_docs ) ]
  #![ allow( clippy::unreadable_literal ) ]
  include!( concat!( env!( "OUT_DIR" ), "/static_commands.rs" ) );
}
#[ cfg( feature = "enabled" ) ]
use generated::AGGREGATED_COMMANDS;

#[ cfg( feature = "enabled" ) ]
mod cli
{
  use super::AGGREGATED_COMMANDS;
  use claude_version::adapter::argv_to_unilang_tokens;
  use unilang::data::{ CommandDefinition, ErrorCode, ErrorData, OutputData };
  use unilang::interpreter::{ ExecutionContext, Interpreter };
  use unilang::parser::{ Parser, UnilangParserOptions };
  use unilang::phf;
  use unilang::registry::CommandRegistry;
  use unilang::semantic::{ SemanticAnalyzer, VerifiedCommand };

  /// Map a unilang error to the appropriate exit code.
  pub( super ) fn exit_code_for( e : &unilang::error::Error ) -> i32
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

  /// Stub for `.claude` and `.claude.help` — subprocess routing is out of scope for `ast`.
  #[ allow( clippy::unnecessary_wraps ) ]
  fn claude_stub_routine( _cmd : VerifiedCommand, _ctx : ExecutionContext ) -> Result< OutputData, ErrorData >
  {
    Ok( OutputData::new( "For Claude Code execution, use clr directly.\n".to_string(), "text" ) )
  }

  /// Register static YAML-sourced commands (runner + storage) into the registry.
  ///
  /// Skips duplicates silently — e.g., `.status` from storage is already
  /// registered by `claude_version::register_commands()` and the manager
  /// version takes precedence.
  fn register_static_commands( registry : &mut CommandRegistry )
  {
    type RoutineFn = fn( VerifiedCommand, ExecutionContext ) -> Result< OutputData, ErrorData >;

    let routines : phf::Map< &'static str, RoutineFn > = phf::phf_map!
    {
      ".claude"          => claude_stub_routine,
      ".claude.help"     => claude_stub_routine,
      ".status"          => claude_storage::cli::status_routine,
      ".list"            => claude_storage::cli::list_routine,
      ".show"            => claude_storage::cli::show_routine,
      ".projects"        => claude_storage::cli::projects_routine,
      ".count"           => claude_storage::cli::count_routine,
      ".search"          => claude_storage::cli::search_routine,
      ".export"          => claude_storage::cli::export_routine,
      ".path"            => claude_storage::cli::project_path_routine,
      ".exists"          => claude_storage::cli::project_exists_routine,
      ".session.dir"     => claude_storage::cli::session_dir_routine,
      ".session.ensure"  => claude_storage::cli::session_ensure_routine,
    };

    for ( name, static_cmd ) in AGGREGATED_COMMANDS.entries()
    {
      if let Some( &routine ) = routines.get( *name )
      {
        let cmd : CommandDefinition = ( *static_cmd ).into();
        // Silently skip duplicates (e.g., .status already registered by claude_version)
        let _ = registry.register_with_routine( &cmd, Box::new( routine ) );
      }
    }
  }

  pub( super ) fn build_registry() -> CommandRegistry
  {
    let mut registry = CommandRegistry::new();
    // Registration order determines first-wins precedence for overlapping command names.
    // After plan-005: account commands live exclusively in claude_profile; no overlap with claude_version.
    claude_assets::register_commands( &mut registry );
    claude_version::register_commands( &mut registry );
    claude_profile::register_commands( &mut registry );
    claude_runner::register_commands( &mut registry );
    claude_storage::register_commands( &mut registry );
    register_static_commands( &mut registry );
    registry
  }

  // 41 commands across 8 groups — data construction cannot be split further.
  #[ allow( clippy::too_many_lines ) ]
  fn print_usage( binary : &str )
  {
    use cli_fmt::help::*;

    let mut data    = CliHelpData::default();
    data.binary     = binary.to_string();
    data.tagline    = "Claude Code super-app: manage versions, accounts, assets, storage, and processes.".to_string();
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
        CommandGroup
        {
          name    : "Version Management".to_string(),
          entries : vec!
          [
            CommandEntry { name : ".version.show".to_string(),    desc : "Show current Claude Code version".to_string() },
            CommandEntry { name : ".version.install".to_string(), desc : "Install a specific Claude Code version".to_string() },
            CommandEntry { name : ".version.guard".to_string(),   desc : "Guard minimum version requirement".to_string() },
            CommandEntry { name : ".version.list".to_string(),    desc : "List available Claude Code versions".to_string() },
            CommandEntry { name : ".version.history".to_string(), desc : "Show version installation history".to_string() },
          ],
        },
        CommandGroup
        {
          name    : "Settings & Config".to_string(),
          entries : vec!
          [
            CommandEntry { name : ".settings.show".to_string(), desc : "Display current settings".to_string() },
            CommandEntry { name : ".settings.get".to_string(),  desc : "Get a specific setting value".to_string() },
            CommandEntry { name : ".settings.set".to_string(),  desc : "Set a specific setting value".to_string() },
            CommandEntry { name : ".config".to_string(),        desc : "Show configuration paths and state".to_string() },
          ],
        },
        CommandGroup
        {
          name    : "Process Lifecycle".to_string(),
          entries : vec!
          [
            CommandEntry { name : ".status".to_string(),         desc : "Show overall system status".to_string() },
            CommandEntry { name : ".processes".to_string(),      desc : "List running Claude processes".to_string() },
            CommandEntry { name : ".processes.kill".to_string(), desc : "Kill running Claude processes".to_string() },
          ],
        },
        CommandGroup
        {
          name    : "Account Management".to_string(),
          entries : vec!
          [
            CommandEntry { name : ".credentials.status".to_string(), desc : "Show credential status".to_string() },
            CommandEntry { name : ".accounts".to_string(),           desc : "List all configured accounts".to_string() },
            CommandEntry { name : ".account.limits".to_string(),     desc : "Show account rate limits".to_string() },
            CommandEntry { name : ".account.save".to_string(),       desc : "Save account credentials".to_string() },
            CommandEntry { name : ".account.use".to_string(),        desc : "Switch active account".to_string() },
            CommandEntry { name : ".account.delete".to_string(),     desc : "Remove an account".to_string() },
            CommandEntry { name : ".account.relogin".to_string(),    desc : "Re-authenticate an account".to_string() },
            CommandEntry { name : ".account.renewal".to_string(),    desc : "Show account renewal info".to_string() },
            CommandEntry { name : ".account.rotate".to_string(),     desc : "Rotate credentials (DEPRECATED)".to_string() },
            CommandEntry { name : ".account.inspect".to_string(),    desc : "Inspect account details".to_string() },
          ],
        },
        CommandGroup
        {
          name    : "Token & Model".to_string(),
          entries : vec!
          [
            CommandEntry { name : ".model".to_string(),        desc : "Show or set active model".to_string() },
            CommandEntry { name : ".token.status".to_string(), desc : "Show token/usage status".to_string() },
            CommandEntry { name : ".paths".to_string(),        desc : "Show profile filesystem paths".to_string() },
            CommandEntry { name : ".usage".to_string(),        desc : "Show usage statistics".to_string() },
          ],
        },
        CommandGroup
        {
          name    : "Storage Query".to_string(),
          entries : vec!
          [
            CommandEntry { name : ".show".to_string(),           desc : "Display entries from a specific session".to_string() },
            CommandEntry { name : ".count".to_string(),          desc : "Count sessions or entries matching criteria".to_string() },
            CommandEntry { name : ".search".to_string(),         desc : "Search conversation content across sessions".to_string() },
            CommandEntry { name : ".export".to_string(),         desc : "Export session data in various formats".to_string() },
            CommandEntry { name : ".projects".to_string(),       desc : "List all known projects with session counts".to_string() },
            CommandEntry { name : ".path".to_string(),           desc : "Print filesystem path of a project directory".to_string() },
            CommandEntry { name : ".exists".to_string(),         desc : "Check whether a project has any sessions".to_string() },
            CommandEntry { name : ".session.dir".to_string(),    desc : "Print the filesystem path of a session directory".to_string() },
            CommandEntry { name : ".session.ensure".to_string(), desc : "Ensure a session directory exists (create if missing)".to_string() },
          ],
        },
        CommandGroup
        {
          name    : "System".to_string(),
          entries : vec!
          [
            CommandEntry { name : ".claude".to_string(),      desc : "Show Claude Code runtime info".to_string() },
            CommandEntry { name : ".claude.help".to_string(), desc : "Show Claude Code built-in help".to_string() },
          ],
        },
      ];
    data.options = vec!
      [
        OptionEntry { name : "kind::KIND".to_string(),   desc : "Artifact kind filter".to_string() },
        OptionEntry { name : "name::NAME".to_string(),   desc : "Artifact or account name".to_string() },
        OptionEntry { name : "format::FMT".to_string(),  desc : "Output format (text, json)".to_string() },
        OptionEntry { name : "session::ID".to_string(),  desc : "Target session identifier".to_string() },
        OptionEntry { name : "project::ID".to_string(),  desc : "Target project identifier".to_string() },
        OptionEntry { name : "limit::N".to_string(),     desc : "Maximum entries to return".to_string() },
        OptionEntry { name : "query::TEXT".to_string(),  desc : "Search query string".to_string() },
      ];
    data.examples = vec!
      [
        ExampleEntry { invocation : format!( "{binary} .status" ),                            desc : None },
        ExampleEntry { invocation : format!( "{binary} .version.show" ),                      desc : None },
        ExampleEntry { invocation : format!( "{binary} .accounts" ),                          desc : None },
        ExampleEntry { invocation : format!( "{binary} .list kind::rule" ),                   desc : None },
        ExampleEntry { invocation : format!( "{binary} .search query::\"error handling\"" ),  desc : None },
      ];
    print!( "{}", CliHelpTemplate::new( CliHelpStyle::default(), data ).render() );
  }

  pub( super ) fn run( argv : &[ String ] )
  {
    let binary = std::env::args()
    .next()
    .as_deref()
    .and_then( | p | std::path::Path::new( p ).file_name() )
    .and_then( | n | n.to_str() )
    .unwrap_or( "ast" )
    .to_owned();

    let ( tokens, needs_help ) = match argv_to_unilang_tokens( argv )
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
}

#[ cfg( feature = "enabled" ) ]
/// Run the `ast`/`assistant` CLI.
///
/// Entry point shared by the `ast` and `assistant` binary targets.
#[ inline ]
pub fn run_cli()
{
  let argv : Vec< String > = std::env::args().skip( 1 ).collect();
  cli::run( &argv );
}

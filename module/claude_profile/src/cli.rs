//! CLI pipeline: adapter → parser → semantic analysis → execution.

use crate::adapter::argv_to_unilang_tokens;
use crate::commands::dot_routine;
use unilang::data::{ CommandDefinition, ErrorCode };
use unilang::interpreter::{ ExecutionContext, Interpreter };
use unilang::parser::{ Parser, UnilangParserOptions };
use unilang::registry::CommandRegistry;
use unilang::semantic::SemanticAnalyzer;

/// Map a unilang error to the appropriate exit code.
///
/// Usage errors (invalid input from the user) → 1.
/// Runtime errors (system failures during execution) → 2.
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

/// Register all `claude_profile` commands with their argument definitions and routines.
///
/// Delegates 16 shared commands to `claude_profile::register_commands()` and
/// adds the `.` (dot) hidden command inline (binary-specific).
pub( super ) fn build_registry() -> CommandRegistry
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
    .register_with_routine( &def, Box::new( dot_routine ) )
    .expect( "internal error: failed to register ." );
  }

  // `.help` is pre-registered by CommandRegistry::new() — do not register again.

  // Register 16 shared commands (credentials, account, token, paths, usage).
  crate::register_commands( &mut registry );

  registry
}

/// Print structured usage to stdout with ANSI colours on TTYs.
///
/// Renders help via `cli_fmt::CliHelpTemplate` with two command groups,
/// three options, and four examples. Colour is suppressed when stdout is
/// not a terminal (TTY detection delegated to `CliHelpStyle::default()`).
///
/// Entry order within each group is defined by the const ordering arrays;
/// descriptions are single-sourced from the command registry. Commands added
/// to the registry that are absent from an ordering array appear automatically
/// in the appropriate group (`.account*` → "Account management"; others →
/// "Status & info"), sorted alphabetically within the overflow section.
pub( super ) fn print_usage( binary : &str )
{
  use cli_fmt::help::*;

  fn entries_for
  (
    names : &[ &str ],
    cmds  : &std::collections::HashMap< String, CommandDefinition >,
  ) -> Vec< cli_fmt::help::CommandEntry >
  {
    names.iter()
    .filter_map( |&name|
      cmds.get( name ).map( |cmd|
        cli_fmt::help::CommandEntry { name : name.to_string(), desc : cmd.description().to_string() }
      )
    )
    .collect()
  }

  const ACCOUNT_MGMT : &[ &str ] = &[
    ".accounts", ".account.save", ".account.use", ".account.delete",
    ".account.limits", ".account.relogin", ".account.rotate",
    ".account.renewal", ".account.inspect",
  ];
  const STATUS_INFO : &[ &str ] = &[
    ".credentials.status", ".token.status", ".paths", ".usage", ".model",
  ];

  let registry = build_registry();
  let cmds     = registry.commands();

  let in_ordered = |name : &str|
    ACCOUNT_MGMT.iter().chain( STATUS_INFO.iter() ).any( |&n| n == name );

  let mut acct_extra : Vec< cli_fmt::help::CommandEntry > = cmds.iter()
  .filter( |( name, cmd )| !cmd.hidden_from_list() && !in_ordered( name ) && name.starts_with( ".account" ) )
  .map( |( name, cmd )| cli_fmt::help::CommandEntry { name : name.clone(), desc : cmd.description().to_string() } )
  .collect();
  let mut info_extra : Vec< cli_fmt::help::CommandEntry > = cmds.iter()
  .filter( |( name, cmd )| !cmd.hidden_from_list() && !in_ordered( name ) && !name.starts_with( ".account" ) )
  .map( |( name, cmd )| cli_fmt::help::CommandEntry { name : name.clone(), desc : cmd.description().to_string() } )
  .collect();
  acct_extra.sort_by( |a, b| a.name.cmp( &b.name ) );
  info_extra.sort_by( |a, b| a.name.cmp( &b.name ) );

  let mut acct_entries = entries_for( ACCOUNT_MGMT, &cmds );
  let mut info_entries = entries_for( STATUS_INFO,  &cmds );
  acct_entries.append( &mut acct_extra );
  info_entries.append( &mut info_extra );

  let mut data    = CliHelpData::default();
  data.binary     = binary.to_string();
  data.tagline    = "Manage Claude Code account credentials and token state.".to_string();
  data.groups     = vec!
  [
    CommandGroup { name : "Account management".to_string(), entries : acct_entries },
    CommandGroup { name : "Status & info".to_string(),      entries : info_entries },
  ];
  data.options    = vec!
  [
    OptionEntry { name : "format::text|json".to_string(), desc : "Output format (default: text)".to_string() },
    OptionEntry { name : "dry::bool".to_string(),         desc : "Dry-run preview (no changes)".to_string()  },
    OptionEntry { name : "name::EMAIL".to_string(),       desc : "Account name".to_string()                  },
  ];
  data.examples   = vec!
  [
    ExampleEntry { invocation : format!( "{binary} .accounts" ),                   desc : None },
    ExampleEntry { invocation : format!( "{binary} .account.use alice@acme.com" ), desc : None },
    ExampleEntry { invocation : format!( "{binary} .usage" ),                      desc : None },
    ExampleEntry { invocation : format!( "{binary} .credentials.status" ),         desc : None },
  ];
  print!( "{}", CliHelpTemplate::new( CliHelpStyle::default(), data ).render() );
}

/// Run the full unilang pipeline for the given argv.
pub( super ) fn run( binary : &str, argv : &[ String ] )
{
  // Phase 1: adapter — convert argv to unilang tokens.
  let ( tokens, needs_help ) = match argv_to_unilang_tokens( argv )
  {
    Ok( r )  => r,
    Err( e ) =>
    {
      eprintln!( "Error: {e}" );
      eprintln!( "Run '{binary} .help' for usage." );
      std::process::exit( 1 );
    }
  };

  // Intercept help requests before entering the unilang pipeline.
  // Triggered by: empty args, `.`, `.help` anywhere in argv, or bare `help`.
  if needs_help
  {
    print_usage( binary );
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

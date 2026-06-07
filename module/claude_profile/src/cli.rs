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
/// Delegates 14 shared commands to `claude_profile::register_commands()` and
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
    .command_add_runtime( &def, Box::new( dot_routine ) )
    .expect( "internal error: failed to register ." );
  }

  // `.help` is pre-registered by CommandRegistry::new() — do not register again.

  // Register 14 shared commands (credentials, account, token, paths, usage).
  crate::register_commands( &mut registry );

  registry
}

/// Print structured usage to stdout with ANSI colours on TTYs.
///
/// Renders help via `cli_fmt::CliHelpTemplate` with two command groups,
/// three options, and four examples. Colour is suppressed when stdout is
/// not a terminal (TTY detection delegated to `CliHelpStyle::default()`).
pub( super ) fn print_usage( binary : &str )
{
  use cli_fmt::help::*;
  let data = CliHelpData
  {
    binary  : binary.to_string(),
    tagline : "Manage Claude Code account credentials and token state.".to_string(),
    groups  : vec!
    [
      CommandGroup
      {
        name    : "Account management".to_string(),
        entries : vec!
        [
          CommandEntry { name : ".accounts".to_string(),       desc : "List all saved accounts".to_string()                    },
          CommandEntry { name : ".account.save".to_string(),   desc : "Save current credentials as a named profile".to_string() },
          CommandEntry { name : ".account.use".to_string(),    desc : "Switch the active account".to_string()                  },
          CommandEntry { name : ".account.delete".to_string(),  desc : "Delete a saved account".to_string()                    },
          CommandEntry { name : ".account.limits".to_string(),  desc : "Show rate-limit utilization (one account)".to_string()  },
          CommandEntry { name : ".account.relogin".to_string(), desc : "Re-authenticate via browser login".to_string()          },
          CommandEntry { name : ".account.rotate".to_string(),  desc : "Auto-rotate to the best inactive account".to_string()   },
        ],
      },
      CommandGroup
      {
        name    : "Status & info".to_string(),
        entries : vec!
        [
          CommandEntry { name : ".credentials.status".to_string(), desc : "Show live credential metadata".to_string()          },
          CommandEntry { name : ".token.status".to_string(),       desc : "Show OAuth token expiry classification".to_string() },
          CommandEntry { name : ".paths".to_string(),              desc : "Show all resolved ~/.claude/ paths".to_string()     },
          CommandEntry { name : ".usage".to_string(),              desc : "Show live quota for all saved accounts".to_string() },
        ],
      },
    ],
    options  : vec!
    [
      OptionEntry { name : "format::text|json".to_string(), desc : "Output format (default: text)".to_string() },
      OptionEntry { name : "dry::bool".to_string(),         desc : "Dry-run preview (no changes)".to_string()  },
      OptionEntry { name : "name::EMAIL".to_string(),       desc : "Account name".to_string()                  },
    ],
    examples : vec!
    [
      ExampleEntry { invocation : format!( "{binary} .accounts" ),                   desc : None },
      ExampleEntry { invocation : format!( "{binary} .account.use alice@acme.com" ), desc : None },
      ExampleEntry { invocation : format!( "{binary} .usage" ),                      desc : None },
      ExampleEntry { invocation : format!( "{binary} .credentials.status" ),         desc : None },
    ],
  };
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
  // Triggered by: no args, `.`.
  // Explicit `.help` does NOT set needs_help, so it still goes through unilang.
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

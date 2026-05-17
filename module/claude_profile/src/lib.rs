#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ doc( html_root_url = "https://docs.rs/claude_profile" ) ]

//! Claude Code account credential management.
//!
//! Manages multiple Claude Code credential sets stored under `.persistent/claude/credential/`
//! for account rotation when usage limits are reached.
//!
//! # Modules
//!
//! - [`paths`]: [`ClaudePaths`] — all `~/.claude/` canonical paths from `HOME`
//! - [`account`]: Named credential storage and rotation
//! - [`token`]: OAuth token expiry status detection
//! - [`persist`]: [`PersistPaths`] — persistent user storage path from `$PRO`/`$HOME` (FR-15)
//!
//! # Account Management Examples
//!
//! ## Check Token Status
//!
//! ```no_run
//! use claude_profile::token;
//!
//! match token::status().expect( "failed to read credentials" )
//! {
//!   token::TokenStatus::Valid { expires_in } =>
//!     println!( "ok — {}m remaining", expires_in.as_secs() / 60 ),
//!   token::TokenStatus::ExpiringSoon { expires_in } =>
//!     eprintln!( "expires in {}m", expires_in.as_secs() / 60 ),
//!   token::TokenStatus::Expired =>
//!     eprintln!( "token expired — run: claude auth login" ),
//! }
//! ```
//!
//! ## Rotate Account
//!
//! ```no_run
//! use std::path::Path;
//! use claude_profile::{ account, ClaudePaths, PersistPaths };
//!
//! let persist = PersistPaths::new().expect( "PRO or HOME must be set" );
//! let credential_store = persist.credential_store();
//! let paths = ClaudePaths::new().expect( "HOME must be set" );
//! // One-liner: pick the inactive account with the highest expiry and switch
//! let switched_to = account::auto_rotate( &credential_store, &paths ).expect( "no inactive account available" );
//! println!( "switched to {switched_to}" );
//! ```
//!
//! ## Inspect and Switch Manually
//!
//! ```no_run
//! use claude_profile::{ account, ClaudePaths, PersistPaths };
//!
//! let persist = PersistPaths::new().expect( "PRO or HOME must be set" );
//! let credential_store = persist.credential_store();
//! let paths = ClaudePaths::new().expect( "HOME must be set" );
//!
//! // See what's available
//! for acct in account::list( &credential_store ).expect( "list failed" )
//! {
//!   let active = if acct.is_active { " ← active" } else { "" };
//!   println!( "{}{} ({})", acct.name, active, acct.subscription_type );
//! }
//!
//! // Switch to a specific account
//! account::switch_account( "alice@home.com", &credential_store, &paths ).expect( "switch failed" );
//! ```

/// Path to the YAML command definitions for this crate.
///
/// Used by `assistant/build.rs` for metadata-only export. Profile commands
/// are registered programmatically via [`register_commands()`], not via YAML aggregation.
pub const COMMANDS_YAML : &str = concat!( env!( "CARGO_MANIFEST_DIR" ), "/unilang.commands.yaml" );

pub mod paths;
pub mod token;
pub mod account;
pub mod persist;

#[ cfg( feature = "enabled" ) ]
pub mod adapter;
#[ cfg( feature = "enabled" ) ]
pub mod output;
#[ cfg( feature = "enabled" ) ]
pub mod commands;
#[ cfg( feature = "enabled" ) ]
pub mod usage;

pub use paths::ClaudePaths;
pub use persist::PersistPaths;

#[ cfg( feature = "enabled" ) ]
/// Register all `claude_profile` commands into an existing registry.
///
/// Registers 9 commands (credentials status, account management including limits, token status, paths, usage).
/// The `.` (dot) hidden command and `.help` are binary-specific — they are NOT
/// included here.
///
/// # Panics
///
/// Panics if a command fails to register (duplicate name = programming error).
#[ inline ]
pub fn register_commands( registry : &mut unilang::registry::CommandRegistry )
{
  use unilang::data::Kind;
  use commands::
  {
    credentials_status_routine,
    accounts_routine,
    account_limits_routine,
    account_save_routine,
    account_use_routine,
    account_delete_routine,
    token_status_routine,
    paths_routine,
    usage_routine,
  };

  let fmt = || reg_arg_opt( "format",    Kind::String  );
  let dry = || reg_arg_opt( "dry",       Kind::Boolean );
  let nam = || reg_arg_opt( "name",      Kind::String  );
  let thr = || reg_arg_opt( "threshold", Kind::Integer );
  let bfd = | nm : &'static str, desc : &'static str |
    reg_arg_opt( nm, Kind::Boolean ).with_description( desc );

  reg_cmd( registry, ".credentials.status", "Show live credential metadata without account store dependency",
    vec![
      reg_arg_opt( "format", Kind::String ).with_description( "Output format: `text` (default) or `json`" ),
      bfd( "account", "Show account name from `_active` marker (default on)"   ),
      bfd( "sub",     "Show subscription type from credentials (default on)"    ),
      bfd( "tier",    "Show rate-limit tier from credentials (default on)"      ),
      bfd( "token",   "Show OAuth token validity state (default on)"            ),
      bfd( "expires", "Show token expiry time (default on)"                     ),
      bfd( "email",   "Show email address from `.claude.json` (default on)"     ),
      bfd( "file",         "Show path to `.credentials.json` file (opt-in)"                         ),
      bfd( "saved",        "Show count of saved accounts in credential store (opt-in)"               ),
      bfd( "display_name", "Show display name from `~/.claude.json` oauthAccount (opt-in)"           ),
      bfd( "role",         "Show organisation role from `~/.claude.json` oauthAccount (opt-in)"      ),
      bfd( "billing",      "Show billing type from `~/.claude.json` oauthAccount (opt-in)"           ),
      bfd( "model",        "Show active model from `~/.claude/settings.json` (opt-in)"               ),
    ],
    Box::new( credentials_status_routine ) );
  reg_cmd( registry, ".accounts",       "List all saved accounts with field-presence control",
    vec![
      nam(),
      bfd( "active",       "Show active/inactive status per account (default on)"                      ),
      bfd( "current",      "Show current (live) session match per account (default on)"                ),
      bfd( "sub",          "Show subscription type per account (default on)"                           ),
      bfd( "tier",         "Show rate-limit tier per account (default on)"                             ),
      bfd( "expires",      "Show token expiry duration per account (default on)"                       ),
      bfd( "email",        "Show email address per account (default on)"                               ),
      bfd( "display_name", "Show display name from saved `{name}.claude.json` snapshot (opt-in)"       ),
      bfd( "role",         "Show organisation role from saved `{name}.claude.json` snapshot (opt-in)"  ),
      bfd( "billing",      "Show billing type from saved `{name}.claude.json` snapshot (opt-in)"       ),
      bfd( "model",        "Show active model from saved `{name}.settings.json` snapshot (opt-in)"     ),
      fmt(),
    ],
    Box::new( accounts_routine ) );
  reg_cmd( registry, ".account.limits", "Show rate-limit utilization for the selected account (FR-18)", vec![ nam(), fmt() ],      Box::new( account_limits_routine ) );
  reg_cmd( registry, ".account.save",   "Save current credentials as a named account profile",            vec![ nam(), dry() ],      Box::new( account_save_routine   ) );
  reg_cmd( registry, ".account.use",    "Switch active account by name with atomic credential rotation",  vec![ nam(), dry() ],      Box::new( account_use_routine    ) );
  reg_cmd( registry, ".account.delete", "Delete a saved account from the account store",                  vec![ nam(), dry() ],      Box::new( account_delete_routine ) );
  reg_cmd( registry, ".token.status",   "Show active OAuth token expiry classification",                  vec![ fmt(), thr() ],      Box::new( token_status_routine   ) );
  reg_cmd( registry, ".paths",          "Show all resolved ~/.claude/ canonical file paths",              vec![ fmt() ],             Box::new( paths_routine          ) );
  reg_cmd( registry, ".usage",          "Show live rate-limit quota for all saved accounts",
    vec![
      fmt(),
      reg_arg_opt( "refresh",   Kind::Integer ).with_description( "Retry once on 401/403/429 by refreshing OAuth token via isolated subprocess (0 = disabled; 1 = enabled)" ),
      reg_arg_opt( "live",      Kind::Integer ).with_description( "Continuous monitor mode (0 = off, default; 1 = on)" ),
      reg_arg_opt( "interval",  Kind::Integer ).with_description( "Seconds between refreshes (minimum 30, default 30)" ),
      reg_arg_opt( "jitter",    Kind::Integer ).with_description( "Max random seconds added to interval (0 = none, default)" ),
      reg_arg_opt( "trace",     Kind::Integer ).with_description( "Print [trace] lines to stderr showing each credential read, API call, and refresh step (0 = off; 1 = on)" ),
    ],
    Box::new( usage_routine          ) );
}

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
mod cli
{
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
  /// Delegates 9 shared commands to `claude_profile::register_commands()` and
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

    // Register 9 shared commands (credentials, account, token, paths, usage).
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
            CommandEntry { name : ".account.delete".to_string(), desc : "Delete a saved account".to_string()                    },
            CommandEntry { name : ".account.limits".to_string(), desc : "Show rate-limit utilization (one account)".to_string()  },
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
}

#[ cfg( feature = "enabled" ) ]
/// Run the `clp`/`claude_profile` CLI.
///
/// Entry point shared by the `clp` and `claude_profile` binary targets.
#[ inline ]
pub fn run_cli()
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

  cli::run( &binary, &argv );
}

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
/// Registers 10 commands (credentials status, account management including limits, token status, paths, usage).
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
    account_list_routine,
    account_limits_routine,
    account_status_routine,
    account_save_routine,
    account_switch_routine,
    account_delete_routine,
    token_status_routine,
    paths_routine,
    usage_routine,
  };

  let v   = || reg_arg_opt( "verbosity", Kind::Integer );
  let fmt = || reg_arg_opt( "format",    Kind::String  );
  let dry = || reg_arg_opt( "dry",       Kind::Boolean );
  let nam = || reg_arg_opt( "name",      Kind::String  );
  let thr = || reg_arg_opt( "threshold", Kind::Integer );
  let bf  = | nm : &'static str | reg_arg_opt( nm, Kind::Boolean );
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
      bfd( "org",     "Show organisation name from `.claude.json` (default on)" ),
      bfd( "file",    "Show path to `.credentials.json` file (opt-in)"          ),
      bfd( "saved",   "Show count of saved accounts in credential store (opt-in)" ),
    ],
    Box::new( credentials_status_routine ) );
  reg_cmd( registry, ".account.list",   "List all saved accounts; or show a single named account", vec![ nam(), v(), fmt() ], Box::new( account_list_routine   ) );
  reg_cmd( registry, ".account.limits", "Show rate-limit utilization for the selected account (FR-18)", vec![ nam(), v(), fmt() ],   Box::new( account_limits_routine ) );
  reg_cmd( registry, ".account.status", "Show active account name and token state; optionally query a named account", vec![ nam(), v(), fmt() ], Box::new( account_status_routine ) );
  reg_cmd( registry, ".account.save",   "Save current credentials as a named account profile",            vec![ nam(), dry() ],      Box::new( account_save_routine   ) );
  reg_cmd( registry, ".account.switch", "Switch active account by name with atomic credential rotation",  vec![ nam(), dry() ],      Box::new( account_switch_routine ) );
  reg_cmd( registry, ".account.delete", "Delete a saved account from the account store",                  vec![ nam(), dry() ],      Box::new( account_delete_routine ) );
  reg_cmd( registry, ".token.status",   "Show active OAuth token expiry classification",                  vec![ v(), fmt(), thr() ], Box::new( token_status_routine   ) );
  reg_cmd( registry, ".paths",          "Show all resolved ~/.claude/ canonical file paths",              vec![ v(), fmt() ],        Box::new( paths_routine          ) );
  reg_cmd( registry, ".usage",          "Show 7-day token usage from stats-cache.json",                   vec![ v(), fmt() ],        Box::new( usage_routine          ) );
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
  /// Delegates 10 shared commands to `claude_profile::register_commands()` and
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

    // Register 10 shared commands (credentials, account, token, paths, usage).
    crate::register_commands( &mut registry );

    registry
  }

  /// Print Claude Code-style structured usage to stdout.
  ///
  /// Mirrors the format Claude Code itself uses: header, description, Commands
  /// table, Options table, and Examples block — binary name is detected at
  /// runtime so both `claude_profile` and `clp` show the correct invocation.
  pub( super ) fn print_usage( binary : &str )
  {
    // Column layout (measured from line start, 0-based):
    //   col  0-1  : 2-space indent
    //   col  2-18 : command name, padded to 17 chars
    //   col 19-46 : parameters  (28 chars for read cmds, 24 for write cmds)
    //   col 47-49 : gap (3+ spaces, making descriptions land on col 50)
    //   col 50+   : description
    //
    // Options column: names padded to 17 chars → descriptions at col 20.
    println!( "Usage: {binary} [command] [key::value ...]" );
    println!();
    println!( "Manage Claude Code account credentials and token state." );
    println!();
    println!( "Commands:" );
    println!( "  .account.list        [name::EMAIL] [v::0-2] [format::text|json]   List all accounts; single with name::" );
    println!( "  .account.status      [name::EMAIL] [v::0-2] [format::text|json]   Show active or named account status" );
    println!( "  .account.save        name::EMAIL [dry::bool]       Save current credentials as named account" );
    println!( "  .account.switch      name::EMAIL [dry::bool]       Switch active account" );
    println!( "  .account.delete      name::EMAIL [dry::bool]       Delete a saved account" );
    println!( "  .token.status        [v::0-2] [format::text|json]   Show OAuth token expiry status" );
    println!( "  .paths               [v::0-2] [format::text|json]   Show all ~/.claude/ canonical paths" );
    println!( "  .usage               [v::0-2] [format::text|json]   Show 7-day token usage summary" );
    println!( "  .credentials.status  [format::text|json] [field::0|1] ...  Show live credentials (no account store needed)" );
    println!();
    println!( "Options:" );
    println!( "  v::0-2              Verbosity level (default: 1)" );
    println!( "  format::text|json   Output format (default: text)" );
    println!( "  dry::bool           Preview without applying" );
    println!( "  name::EMAIL        Account name" );
    println!();
    println!( "Examples:" );
    println!( "  {binary} .account.list" );
    println!( "  {binary} .account.list v::2" );
    println!( "  {binary} .account.switch name::alice@acme.com" );
    println!( "  {binary} .account.switch name::alice@acme.com dry::true" );
    println!( "  {binary} .token.status format::json" );
    println!( "  {binary} .paths v::2" );
    println!( "  {binary} .usage" );
    println!( "  {binary} .usage v::2" );
    println!( "  {binary} .credentials.status" );
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

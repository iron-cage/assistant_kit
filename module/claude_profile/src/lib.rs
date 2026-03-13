#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! Claude Code account credential management.
//!
//! Manages multiple Claude Code credential sets stored under `~/.claude/accounts/`
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
//! use claude_profile::account;
//!
//! // One-liner: pick the inactive account with the highest expiry and switch
//! let switched_to = account::auto_rotate().expect( "no inactive account available" );
//! println!( "switched to {switched_to}" );
//! ```
//!
//! ## Inspect and Switch Manually
//!
//! ```no_run
//! use claude_profile::account;
//!
//! // See what's available
//! for acct in account::list().expect( "list failed" )
//! {
//!   let active = if acct.is_active { " ← active" } else { "" };
//!   println!( "{}{} ({})", acct.name, active, acct.subscription_type );
//! }
//!
//! // Switch to a specific account
//! account::switch_account( "personal" ).expect( "switch failed" );
//! ```

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]
#![ warn( missing_docs ) ]

/// Path to the YAML command definitions for this crate.
///
/// Used by `claude_tools/build.rs` for metadata-only export. Profile commands
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

  reg_cmd( registry, ".credentials.status", "Show live credential metadata without account store dependency", vec![ v(), fmt() ],   Box::new( credentials_status_routine ) );
  reg_cmd( registry, ".account.list",   "List all saved accounts with subscription type and token state", vec![ v(), fmt() ],        Box::new( account_list_routine   ) );
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

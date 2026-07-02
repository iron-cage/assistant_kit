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
//! - [`registry`]: Command registration helpers (feature `enabled`)
//! - [`output`]: Output formatting, `parse_int_flag`, JWT utilities (feature `enabled`)
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
#[ cfg( feature = "enabled" ) ]
pub mod registry;
#[ cfg( feature = "enabled" ) ]
pub( crate ) mod owner_dispatch;
#[ cfg( feature = "enabled" ) ]
mod cli;

pub use paths::ClaudePaths;
pub use persist::PersistPaths;
#[ cfg( feature = "enabled" ) ]
pub use registry::register_commands;

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

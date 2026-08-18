//! Named credential storage and account rotation.
//!
//! # Account Store Layout
//!
//! ```text
//! $PRO/.persistent/claude/credential/
//!   alice@acme.com.credentials.json   ← OAuth credentials (tokens, expiry)
//!   alice@acme.com.json               ← account metadata (identity, model, roles, profile)
//!   alice@home.com.credentials.json
//!   alice@home.com.json
//!   _active_w003_user1                ← text: name of active account (per-machine)
//!   _filter_w003_user1                ← JSON: per-identity include/exclude tag filter
//! ```
//!
//! The active marker filename is `_active_{hostname}_{user}` (see [`active_marker_filename`]).
//! Each machine maintains its own marker independently; add `_active_*` to `.gitignore`.
//!
//! The tag filter filename is `_filter_{hostname}_{user}` — the same slug (see
//! [`filter_filename`]). Unlike the marker it is deliberately NOT gitignored: the
//! filter syncs with the store so it can be administered centrally (Feature 076).
//!
//! # Examples
//!
//! ```no_run
//! use claude_profile_core::account;
//! use claude_core::ClaudePaths;
//! use std::path::Path;
//!
//! let paths = ClaudePaths::new().expect( "HOME must be set" );
//! let credential_store = Path::new( "/pro/.persistent/claude/credential" );
//!
//! // List stored accounts
//! for acct in account::list( credential_store ).expect( "failed to list accounts" )
//! {
//!   let active = if acct.is_active { " ← active" } else { "" };
//!   println!( "{}{} ({}) email={}", acct.name, active, acct.subscription_type, acct.email );
//! }
//!
//! // Save current credentials as "alice@acme.com"
//! account::save(
//!   "alice@acme.com", credential_store, &paths, true, None, None, None, None,
//!   account::AccountBackend::Anthropic, None, None, None, None,
//! ).expect( "failed to save" );
//!
//! // Switch to "alice@home.com"
//! account::switch_account( "alice@home.com", credential_store, &paths ).expect( "failed to switch" );
//!
//! // Delete an old entry
//! account::delete( "alice@oldco.com", credential_store ).expect( "failed to delete" );
//! ```

mod types;
mod store;
mod validate;
mod switch;
mod session_settings;
mod refresh;
mod ownership;
mod renewal;
mod json_field;
mod quota_cache;
mod history;
mod tags;
mod filter;

pub use types::*;
pub use store::*;
pub use validate::*;
pub use switch::*;
pub use session_settings::*;
pub use refresh::*;
pub use ownership::*;
pub use renewal::*;
pub use json_field::*;
pub use quota_cache::*;
pub use history::*;
pub use tags::*;
pub use filter::*;

// Canonical implementations live in claude_core::time (identical bodies were
// previously duplicated here); re-exported to preserve the public paths
// `account::chrono_now_utc` / `account::trace_ts` that sibling crates import.
// trace_ts carries Fix(BUG-338)'s "UTC" marker format — the behavioral test
// `trace_ts_returns_utc_marked_timestamp` pins it regardless of which crate hosts it.
pub use claude_core::{ chrono_now_utc, trace_ts };

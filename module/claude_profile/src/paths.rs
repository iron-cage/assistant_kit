//! Canonical paths for all `~/.claude/` filesystem locations.
//!
//! Re-exported from `claude_core` — the authoritative implementation lives there.
//!
//! # Examples
//!
//! ```no_run
//! use claude_profile::ClaudePaths;
//!
//! let p = ClaudePaths::new().expect( "HOME must be set" );
//! println!( "accounts: {}", p.accounts_dir().display() );
//! ```

pub use claude_core::ClaudePaths;

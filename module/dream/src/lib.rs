#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! `dream` — agent-agnostic, feature-gated facade re-exporting AI agent core crates.
//!
//! Enable individual domain modules via Cargo features:
//!
//! | Feature | Module | Activates |
//! |---------|--------|-----------|
//! | `common` | [`common`] | `claude_core` — `ClaudePaths`, process utilities |
//! | `storage` | [`storage`] | `claude_storage_core` — JSONL parser, session types |
//! | `profile` | [`profile`] | `claude_profile_core` — token status, account management |
//! | `runner` | [`runner`] | `claude_runner_core` — `ClaudeCommand` builder + `execute()` |
//! | `version` | [`version`] | `claude_version_core` — version detection, settings I/O helpers |
//! | `assets` | [`assets`] | `claude_assets_core` — symlink-based artifact installer |
//! | `full` | all | All six domain modules |
//! | `enabled` | all | Alias for `full` |
//!
//! # Usage
//!
//! ```toml
//! [dependencies]
//! dream = { version = "~1.0", features = ["profile", "runner"] }
//! ```
//!
//! ```rust,no_run
//! # #[cfg(all(feature = "profile", feature = "runner"))]
//! # {
//! use dream::runner::ClaudeCommand;
//! # }
//! ```

#[ cfg( feature = "common" ) ]
pub mod common
{
  //! Re-exports from [`claude_core`]: `ClaudePaths`, process utilities.
  pub use claude_core::*;
}

#[ cfg( feature = "storage" ) ]
pub mod storage
{
  //! Re-exports from [`claude_storage_core`]: `Storage`, session and project types.
  pub use claude_storage_core::*;
}

#[ cfg( feature = "profile" ) ]
pub mod profile
{
  //! Re-exports from [`claude_profile_core`]: token status, account management.
  pub use claude_profile_core::*;
}

#[ cfg( feature = "runner" ) ]
pub mod runner
{
  //! Re-exports from [`claude_runner_core`]: `ClaudeCommand` builder + `execute()`.
  pub use claude_runner_core::*;
}

#[ cfg( feature = "version" ) ]
pub mod version
{
  //! Re-exports from [`claude_version_core`]: version detection, settings I/O.
  pub use claude_version_core::*;
}

#[ cfg( feature = "assets" ) ]
pub mod assets
{
  //! Re-exports from [`claude_assets_core`]: symlink-based artifact installer.
  pub use claude_assets_core::*;
}

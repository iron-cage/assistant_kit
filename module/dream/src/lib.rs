#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! `dream` — feature-gated facade re-exporting coding agent core crates.

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
  //! Re-exports from [`claude_version_core`]: version detection, settings I/O, install helpers.
  pub use claude_version_core::*;
}

#[ cfg( feature = "assets" ) ]
pub mod assets
{
  //! Re-exports from [`claude_assets_core`]: symlink-based artifact installer.
  pub use claude_assets_core::*;
}

#[ cfg( feature = "quota" ) ]
pub mod quota
{
  //! Re-exports from [`claude_quota`]: rate-limit utilization data and HTTP transport.
  pub use claude_quota::*;
}

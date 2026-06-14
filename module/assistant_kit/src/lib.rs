#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! `assistant_kit` — Layer 3 library facade re-exporting all Layer 2 full-featured crates.
//!
//! Unlike [`dream`] (which re-exports only `*_core` crates), `assistant_kit` re-exports
//! the full-featured Layer 2 crates — giving library consumers access to the complete
//! CLI command surface without depending on a binary.
//!
//! # Feature Flags
//!
//! | Feature   | Activates                        | Description                                   |
//! |-----------|----------------------------------|-----------------------------------------------|
//! | `profile` | `claude_profile/enabled`         | Account management, token status, CLI surface |
//! | `runner`  | `claude_runner/enabled`          | `ClaudeCommand` builder + CLI surface         |
//! | `version` | `claude_version/enabled`         | Version detection, settings I/O, CLI surface  |
//! | `assets`  | `claude_assets/enabled`          | Symlink-based artifact installer CLI surface  |
//! | `storage` | `claude_storage/cli`             | Storage exploration CLI surface               |
//! | `full`    | all five above                   | Everything                                    |
//! | `enabled` | `full`                           | Alias for `full`                              |

#[ cfg( feature = "profile" ) ]
pub mod profile
{
  //! Re-exports from [`claude_profile`]: account management, token status, CLI command surface.
  pub use claude_profile::*;
}

#[ cfg( feature = "runner" ) ]
pub mod runner
{
  //! Re-exports from [`claude_runner`]: `ClaudeCommand` builder, verbosity, CLI command surface.
  pub use claude_runner::*;
}

#[ cfg( feature = "version" ) ]
pub mod version
{
  //! Re-exports from [`claude_version`]: version detection, settings I/O, CLI command surface.
  pub use claude_version::*;
}

#[ cfg( feature = "assets" ) ]
pub mod assets
{
  //! Re-exports from [`claude_assets`]: symlink-based artifact installer CLI command surface.
  pub use claude_assets::*;
}

#[ cfg( feature = "storage" ) ]
pub mod storage
{
  //! Re-exports from [`claude_storage`]: storage exploration CLI command surface.
  pub use claude_storage::*;
}

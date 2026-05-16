//! CLI output formatting utilities for `claude_tools`.
//!
//! Local copy of the `cli_fmt` crate from the `wtools` workspace.
//! Only the `cli_help_template` feature is included here.
//! See the `help` module for `CliHelpTemplate`, `CliHelpStyle`, and `CliHelpData`.

/// CLI help text rendering.
#[ cfg( feature = "cli_help_template" ) ]
pub mod help;

/// Own namespace of the module.
#[ doc( inline ) ]
#[ allow( unused_imports ) ]
pub use own::*;

/// Own namespace of the module.
#[ allow( unused_imports ) ]
pub mod own
{
  #[ allow( unused_imports ) ]
  use super::*;
  #[ cfg( feature = "cli_help_template" ) ]
  pub use super::help::orphan::*;
}

/// Prelude to use essentials: `use cli_fmt::prelude::*`.
#[ allow( unused_imports ) ]
pub mod prelude
{
  #[ allow( unused_imports ) ]
  use super::*;
  #[ cfg( feature = "cli_help_template" ) ]
  pub use super::help::orphan::*;
}

//! Tests for `claude_runner::register_commands()`.
//!
//! # Test Matrix
//!
//! | TC | Description | P/N |
//! |----|-------------|-----|
//! | TC-001 | `register_commands()` is callable | P |

#[ cfg( feature = "enabled" ) ]
mod enabled
{
  use unilang::registry::CommandRegistry;

  #[ test ]
  fn tc001_register_commands_callable()
  {
    let mut registry = CommandRegistry::new();
    claude_runner::register_commands( &mut registry );
    // claude_runner has no runtime commands; registry unchanged but callable is what matters.
    let _ = registry;
  }
}

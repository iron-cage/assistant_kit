//! Tests for `claude_journal_viewer::register_commands()`.
//!
//! # Test Matrix
//!
//! | TC | Description | P/N |
//! |----|-------------|-----|
//! | TC-001 | `register_commands()` is callable and leaves the registry unchanged (T03) | P |

#[ cfg( feature = "routines" ) ]
mod routines
{
  use unilang::registry::CommandRegistry;

  #[ test ]
  fn tc001_register_commands_is_true_noop()
  {
    let mut registry = CommandRegistry::new();
    let before = registry.commands().len();
    claude_journal_viewer::register_commands( &mut registry );
    let after = registry.commands().len();
    assert_eq!( before, after, "register_commands() must not add or remove any command" );
  }
}

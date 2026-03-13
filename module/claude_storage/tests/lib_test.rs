//! Tests for `claude_storage::COMMANDS_YAML` and `register_commands()`.
//!
//! # Test Matrix
//!
//! | TC | Description | P/N |
//! |----|-------------|-----|
//! | TC-001 | `COMMANDS_YAML` points to an existing file | P |
//! | TC-002 | `register_commands()` is callable | P |

#[ test ]
fn tc001_commands_yaml_exists()
{
  assert!(
    std::path::Path::new( claude_storage::COMMANDS_YAML ).exists(),
    "COMMANDS_YAML must point to an existing file: {}",
    claude_storage::COMMANDS_YAML
  );
}

#[ cfg( feature = "cli" ) ]
mod cli_tests
{
  use unilang::registry::CommandRegistry;

  #[ test ]
  fn tc002_register_commands_callable()
  {
    let mut registry = CommandRegistry::new();
    claude_storage::register_commands( &mut registry );
    // Storage commands registered via YAML; callable is what matters.
    let _ = registry;
  }
}

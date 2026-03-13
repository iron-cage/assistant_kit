//! Tests for `claude_manager::register_commands()`.
//!
//! # Test Matrix
//!
//! | TC | Description | P/N |
//! |----|-------------|-----|
//! | TC-001 | `register_commands()` is callable and adds commands to a registry | P |
//! | TC-002 | all 11 explicitly-registered commands present (`.help` auto-registered = 12 total) | P |

#[ cfg( feature = "enabled" ) ]
mod enabled
{
  use unilang::registry::CommandRegistry;

  #[ test ]
  fn tc001_register_commands_callable()
  {
    let mut registry = CommandRegistry::new();
    claude_manager::register_commands( &mut registry );
    assert!( registry.command( ".status" ).is_some(), ".status must be registered" );
    assert!( registry.command( ".processes" ).is_some(), ".processes must be registered" );
    assert!( registry.command( ".settings.get" ).is_some(), ".settings.get must be registered" );
  }

  #[ test ]
  fn tc002_all_visible_commands_registered()
  {
    let mut registry = CommandRegistry::new();
    claude_manager::register_commands( &mut registry );
    let expected = [
      ".status",
      ".version.show",
      ".version.install",
      ".version.guard",
      ".version.list",
      ".version.history",
      ".processes",
      ".processes.kill",
      ".settings.show",
      ".settings.get",
      ".settings.set",
    ];
    for name in &expected
    {
      assert!(
        registry.command( name ).is_some(),
        "command {name} must be registered"
      );
    }
  }
}

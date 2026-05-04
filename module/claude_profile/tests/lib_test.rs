//! Tests for `claude_profile` library-level exports.
//!
//! # Test Matrix
//!
//! | TC | Description | P/N |
//! |----|-------------|-----|
//! | TC-500 | `COMMANDS_YAML` is a non-empty path string ending in `.yaml` | P |
//! | TC-501 | `register_commands()` is callable and adds commands to a registry | P |
//! | TC-502 | all 9 explicitly-registered commands present (`.help` auto-registered = 10 total) | P |

#[ cfg( feature = "enabled" ) ]
mod enabled
{
  use unilang::registry::CommandRegistry;

  #[ test ]
  fn tc500_commands_yaml_is_valid_path()
  {
    let path = claude_profile::COMMANDS_YAML;
    assert!( !path.is_empty(), "COMMANDS_YAML must not be empty" );
    // Use Path::extension() rather than str::ends_with(".yaml") — the latter triggers
    // clippy::case_sensitive_file_extension_comparisons on paths that may arrive with
    // uppercase extensions on case-insensitive file systems.
    assert!(
      std::path::Path::new( path ).extension().is_some_and( | ext | ext.eq_ignore_ascii_case( "yaml" ) ),
      "COMMANDS_YAML must end with .yaml: {path}"
    );
  }

  #[ test ]
  fn tc501_register_commands_callable()
  {
    let mut registry = CommandRegistry::new();
    claude_profile::register_commands( &mut registry );
    assert!( registry.command( ".accounts" ).is_some(), ".accounts must be registered" );
    assert!( registry.command( ".usage" ).is_some(), ".usage must be registered" );
    assert!( registry.command( ".paths" ).is_some(), ".paths must be registered" );
  }

  #[ test ]
  fn tc502_all_shared_commands_registered()
  {
    let mut registry = CommandRegistry::new();
    claude_profile::register_commands( &mut registry );
    let expected = [
      ".accounts",
      ".account.limits",
      ".account.save",
      ".account.switch",
      ".account.delete",
      ".credentials.status",
      ".token.status",
      ".paths",
      ".usage",
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

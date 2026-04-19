//! Tests for `claude_version::register_commands()` and YAML/registration consistency.
//!
//! # Test Matrix
//!
//! | TC | Description | P/N |
//! |----|-------------|-----|
//! | TC-001 | `register_commands()` is callable and adds commands to a registry | P |
//! | TC-002 | all 11 explicitly-registered commands present (`.help` auto-registered = 12 total) | P |
//! | TC-003 | `unilang.commands.yaml` file exists at `COMMANDS_YAML` path | P |
//! | TC-004 | YAML contains exactly the 11 expected command names | P |
//! | TC-005 | all 11 YAML command names are also registered programmatically (no drift) | P |

/// Canonical command list — single source of truth for TC-002, TC-004, TC-005.
const EXPECTED_COMMANDS : &[ &str ] = &[
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

#[ cfg( feature = "enabled" ) ]
mod enabled
{
  use super::EXPECTED_COMMANDS;
  use unilang::registry::CommandRegistry;

  #[ test ]
  fn tc001_register_commands_callable()
  {
    let mut registry = CommandRegistry::new();
    claude_version::register_commands( &mut registry );
    assert!( registry.command( ".status" ).is_some(), ".status must be registered" );
    assert!( registry.command( ".processes" ).is_some(), ".processes must be registered" );
    assert!( registry.command( ".settings.get" ).is_some(), ".settings.get must be registered" );
  }

  #[ test ]
  fn tc002_all_visible_commands_registered()
  {
    let mut registry = CommandRegistry::new();
    claude_version::register_commands( &mut registry );
    for name in EXPECTED_COMMANDS
    {
      assert!(
        registry.command( name ).is_some(),
        "command {name} must be registered"
      );
    }
  }

  // TC-003: YAML file exists at the path declared by the COMMANDS_YAML constant.
  #[ test ]
  fn tc003_commands_yaml_file_exists()
  {
    let path = std::path::Path::new( claude_version::COMMANDS_YAML );
    assert!(
      path.exists(),
      "COMMANDS_YAML points to non-existent file: {}",
      claude_version::COMMANDS_YAML
    );
  }

  // TC-004: YAML contains all 11 expected command names (drift detection: YAML side).
  #[ test ]
  fn tc004_yaml_contains_all_expected_commands()
  {
    let content = std::fs::read_to_string( claude_version::COMMANDS_YAML )
      .expect( "failed to read unilang.commands.yaml" );
    for name in EXPECTED_COMMANDS
    {
      let entry = format!( "- name: \"{name}\"" );
      assert!(
        content.contains( &entry ),
        "YAML missing command entry '{name}'\n\
         Expected: {entry}\n\
         Fix: add or restore the command block in unilang.commands.yaml"
      );
    }
  }

  // TC-005: every command present in YAML is also registered programmatically (drift detection: registry side).
  #[ test ]
  fn tc005_yaml_names_match_programmatic_registration()
  {
    let content = std::fs::read_to_string( claude_version::COMMANDS_YAML )
      .expect( "failed to read unilang.commands.yaml" );

    // Extract all `- name: "..."` entries from the YAML.
    let yaml_names : Vec< String > = content
      .lines()
      .filter_map( | line |
      {
        let t = line.trim();
        t.strip_prefix( "- name: \"" )?.strip_suffix( '"' ).map( | n | n.to_string() )
      } )
      .collect();

    let mut registry = CommandRegistry::new();
    claude_version::register_commands( &mut registry );

    // Every YAML name (excluding .help which is auto-registered) must be programmatically registered.
    for name in &yaml_names
    {
      assert!(
        registry.command( name ).is_some(),
        "YAML lists '{name}' but it is not registered programmatically — update register_commands() or remove from YAML"
      );
    }

    // Every programmatically registered expected command must appear in YAML.
    for name in EXPECTED_COMMANDS
    {
      let entry = format!( "- name: \"{name}\"" );
      assert!(
        content.contains( &entry ),
        "Command '{name}' registered programmatically but absent from YAML — add to unilang.commands.yaml"
      );
    }
  }
}

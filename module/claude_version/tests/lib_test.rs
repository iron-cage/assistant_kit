//! Tests for `claude_version::register_commands()` and YAML/registration consistency.
//!
//! # Test Matrix
//!
//! | TC | Description | P/N |
//! |----|-------------|-----|
//! | TC-001 | `register_commands()` is callable and adds commands to a registry | P |
//! | TC-002 | all 14 explicitly-registered commands present (`.help` auto-registered = 15 total) | P |
//! | TC-003 | `unilang.commands.yaml` file exists at `COMMANDS_YAML` path | P |
//! | TC-004 | YAML contains exactly the 14 expected command names | P |
//! | TC-005 | all 14 YAML command names are also registered programmatically (no drift) | P |
//! | TC-006 | all registered args carry a non-empty description | P |
//! | TC-007 | `verbosity` arg default is `"1"` | P |
//! | TC-008 | `format` arg default is `"text"` | P |
//! | TC-009 | `mode` arg on `.version.list` default is `"aliases"` | P |
//! | TC-010 | `count` arg on `.version.list` default is `"10"` | P |
//! | TC-011 | `key` arg on `.settings.get` has no default | P |

/// Canonical command list — single source of truth for TC-002, TC-004, TC-005.
const EXPECTED_COMMANDS : &[ &str ] = &[
  ".status",
  ".version.show",
  ".version.install",
  ".version.guard",
  ".version.list",
  ".processes",
  ".processes.kill",
  ".settings.show",
  ".settings.get",
  ".settings.set",
  ".config",
  ".params",
  ".runtime_files",
  ".version.paths",
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

  // TC-004: YAML contains all 13 expected command names (drift detection: YAML side).
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

    // Extract command `- name: "..."` entries from the YAML (names starting with `.`).
    // Parameter entries like `- name: "verbosity"` share the same YAML syntax but
    // are not commands — filtering by dot prefix distinguishes them correctly.
    let yaml_names : Vec< String > = content
      .lines()
      .filter_map( | line |
      {
        let t = line.trim();
        let name = t.strip_prefix( "- name: \"" )?.strip_suffix( '"' )?;
        name.starts_with( '.' ).then( || name.to_string() )
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

  // TC-006: every argument on every registered command has a non-empty description.
  #[ test ]
  fn tc006_all_args_have_non_empty_descriptions()
  {
    let mut registry = CommandRegistry::new();
    claude_version::register_commands( &mut registry );
    for name in EXPECTED_COMMANDS
    {
      let cmd = registry.command( name ).expect( "command must be registered" );
      for arg in cmd.arguments()
      {
        assert!(
          !arg.description.is_empty(),
          "arg '{}' of '{name}' has empty description — add it in reg_arg_opt call",
          arg.name
        );
      }
    }
  }

  // TC-007: verbosity default = "1".
  #[ test ]
  fn tc007_verbosity_default_is_1()
  {
    let mut registry = CommandRegistry::new();
    claude_version::register_commands( &mut registry );
    let cmd = registry.command( ".status" ).expect( ".status must be registered" );
    let arg = cmd.arguments().iter()
    .find( |a| a.name == "verbosity" )
    .expect( "verbosity must be an arg of .status" );
    assert_eq!(
      arg.attributes.default.as_deref(), Some( "1" ),
      "verbosity default must be \"1\", got {:?}", arg.attributes.default
    );
  }

  // TC-008: format default = "text".
  #[ test ]
  fn tc008_format_default_is_text()
  {
    let mut registry = CommandRegistry::new();
    claude_version::register_commands( &mut registry );
    let cmd = registry.command( ".status" ).expect( ".status must be registered" );
    let arg = cmd.arguments().iter()
    .find( |a| a.name == "format" )
    .expect( "format must be an arg of .status" );
    assert_eq!(
      arg.attributes.default.as_deref(), Some( "text" ),
      "format default must be \"text\", got {:?}", arg.attributes.default
    );
  }

  // TC-009: mode on .version.list default = "aliases".
  #[ test ]
  fn tc009_mode_default_is_aliases()
  {
    let mut registry = CommandRegistry::new();
    claude_version::register_commands( &mut registry );
    let cmd = registry.command( ".version.list" ).expect( ".version.list must be registered" );
    let arg = cmd.arguments().iter()
    .find( |a| a.name == "mode" )
    .expect( "mode must be an arg of .version.list" );
    assert_eq!(
      arg.attributes.default.as_deref(), Some( "aliases" ),
      "mode default must be \"aliases\", got {:?}", arg.attributes.default
    );
  }

  // TC-010: count on .version.list default = "10".
  #[ test ]
  fn tc010_count_default_is_10()
  {
    let mut registry = CommandRegistry::new();
    claude_version::register_commands( &mut registry );
    let cmd = registry.command( ".version.list" ).expect( ".version.list must be registered" );
    let arg = cmd.arguments().iter()
    .find( |a| a.name == "count" )
    .expect( "count must be an arg of .version.list" );
    assert_eq!(
      arg.attributes.default.as_deref(), Some( "10" ),
      "count default must be \"10\", got {:?}", arg.attributes.default
    );
  }

  // TC-011: key on .settings.get has no documented default (required arg).
  #[ test ]
  fn tc011_key_has_no_default()
  {
    let mut registry = CommandRegistry::new();
    claude_version::register_commands( &mut registry );
    let cmd = registry.command( ".settings.get" ).expect( ".settings.get must be registered" );
    let arg = cmd.arguments().iter()
    .find( |a| a.name == "key" )
    .expect( "key must be an arg of .settings.get" );
    assert!(
      arg.attributes.default.is_none(),
      "key must have no default, got {:?}", arg.attributes.default
    );
  }
}

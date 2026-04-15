//! Build script for `assistant`.
//!
//! Aggregates command YAML from Layer 2 crates that expose static YAML definitions:
//!   - `claude_runner`  — `claude_runner::COMMANDS_YAML`  (`.claude`, `.claude.help`)
//!   - `claude_storage` — `claude_storage::COMMANDS_YAML`  (9 storage commands)
//!
//! Manager and profile use programmatic registration via `register_commands()`
//! (hybrid approach — their YAML is metadata-only, not aggregated here).
//!
//! Generates `static_commands.rs` into `$OUT_DIR` for O(1) command lookup of
//! YAML-sourced commands at runtime.

use std::{ collections::HashMap, env, path::PathBuf };

fn main()
{
  println!( "cargo:rerun-if-changed=build.rs" );
  println!( "cargo:rerun-if-changed={}", claude_assets::COMMANDS_YAML );
  println!( "cargo:rerun-if-changed={}", claude_runner::COMMANDS_YAML );
  println!( "cargo:rerun-if-changed={}", claude_storage::COMMANDS_YAML );

  if env::var( "CARGO_FEATURE_ENABLED" ).is_ok()
  {
    generate_static_commands();
  }
}

/// Transform a simplified YAML argument (flat `optional`/`default`) into
/// unilang's nested `attributes` structure.
///
/// Mirrors the same transformation in `claude_storage/build.rs`.
fn transform_argument_attributes( arg_map : &mut serde_yaml::Mapping )
{
  let optional = arg_map
    .remove( serde_yaml::Value::String( "optional".to_string() ) )
    .and_then( | v | v.as_bool() )
    .unwrap_or( false );

  let default_val = arg_map
    .remove( serde_yaml::Value::String( "default".to_string() ) )
    .and_then( | v | match v
    {
      serde_yaml::Value::String( s )  => Some( s ),
      serde_yaml::Value::Number( n )  => Some( n.to_string() ),
      serde_yaml::Value::Bool( b )    => Some( b.to_string() ),
      _                               => None,
    } );

  let mut attributes = serde_yaml::Mapping::new();
  attributes.insert( serde_yaml::Value::String( "optional".to_string() ),    serde_yaml::Value::Bool( optional ) );
  attributes.insert( serde_yaml::Value::String( "sensitive".to_string() ),   serde_yaml::Value::Bool( false ) );
  attributes.insert( serde_yaml::Value::String( "interactive".to_string() ), serde_yaml::Value::Bool( false ) );
  attributes.insert( serde_yaml::Value::String( "multiple".to_string() ),    serde_yaml::Value::Bool( false ) );

  if let Some( default_str ) = default_val
  {
    attributes.insert(
      serde_yaml::Value::String( "default".to_string() ),
      serde_yaml::Value::String( default_str ),
    );
  }

  arg_map.insert(
    serde_yaml::Value::String( "attributes".to_string() ),
    serde_yaml::Value::Mapping( attributes ),
  );
}

/// Read a YAML file at `src_path`, transform its argument attributes, and write
/// the result to `$OUT_DIR/<out_name>`. Returns the relative filename.
fn transform_yaml( src_path : &str, out_name : &str, out_dir : &str ) -> String
{
  let content = std::fs::read_to_string( src_path )
    .unwrap_or_else( | e | panic!( "Failed to read {src_path}: {e}" ) );

  let mut commands : Vec< serde_yaml::Value > = serde_yaml::from_str( &content )
    .unwrap_or_else( | e | panic!( "Failed to parse {src_path}: {e}" ) );

  for command in &mut commands
  {
    if let Some( args ) = command.get_mut( "arguments" ).and_then( | v | v.as_sequence_mut() )
    {
      for arg in args
      {
        if let Some( map ) = arg.as_mapping_mut()
        {
          transform_argument_attributes( map );
        }
      }
    }
  }

  let temp_path = PathBuf::from( out_dir ).join( out_name );
  let yaml = serde_yaml::to_string( &commands )
    .unwrap_or_else( | e | panic!( "Failed to serialize {out_name}: {e}" ) );
  std::fs::write( &temp_path, yaml )
    .unwrap_or_else( | e | panic!( "Failed to write {out_name}: {e}" ) );

  out_name.to_string()
}

fn generate_static_commands()
{
  let out_dir = env::var( "OUT_DIR" ).expect( "OUT_DIR not set" );

  // Transform YAML from Layer 2 YAML-based crates.
  // claude_version uses programmatic registration (no YAML file).
  let runner_yaml  = transform_yaml( claude_runner::COMMANDS_YAML,  "claude_runner.yaml",  &out_dir );
  let storage_yaml = transform_yaml( claude_storage::COMMANDS_YAML, "claude_storage.yaml", &out_dir );

  let config = unilang::multi_yaml::AggregationConfig
  {
    base_dir            : PathBuf::from( &out_dir ),
    modules             : vec!
    [
      unilang::multi_yaml::ModuleConfig
      {
        name      : "claude_runner".to_string(),
        yaml_path : runner_yaml,
        prefix    : None,
        enabled   : true,
      },
      unilang::multi_yaml::ModuleConfig
      {
        name      : "claude_storage".to_string(),
        yaml_path : storage_yaml,
        prefix    : None,
        enabled   : true,
      },
    ],
    global_prefix       : None,
    detect_conflicts    : true,
    env_overrides       : HashMap::new(),
    conflict_resolution : unilang::multi_yaml::ConflictResolutionStrategy::Fail,
    auto_discovery      : false,
    discovery_patterns  : vec![],
    namespace_isolation : unilang::multi_yaml::NamespaceIsolation
    {
      enabled     : false,
      separator   : ".".to_string(),
      strict_mode : false,
    },
  };

  let mut aggregator = unilang::multi_yaml::MultiYamlAggregator::new( config );

  match aggregator.aggregate()
  {
    Ok( () ) =>
    {
      let mut source = aggregator.generate_static_registry_source();

      // Fix(issue-unilang-show-version): inject missing field as in claude_storage/build.rs.
      if !source.contains( "show_version_in_help:" )
      {
        source = source.replace(
          "  auto_help_enabled: true,\n",
          "  auto_help_enabled: true,\n  show_version_in_help: true,\n",
        );
        source = source.replace(
          "  auto_help_enabled: false,\n",
          "  auto_help_enabled: false,\n  show_version_in_help: true,\n",
        );
      }

      let output_path = PathBuf::from( &out_dir ).join( "static_commands.rs" );
      std::fs::write( &output_path, &source )
        .expect( "Failed to write static_commands.rs" );

      println!( "cargo:warning=assistant: generated static command registry" );
    }
    Err( e ) => panic!( "Failed to aggregate commands: {e}" ),
  }
}

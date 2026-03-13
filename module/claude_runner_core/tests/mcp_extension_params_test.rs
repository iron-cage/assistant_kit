//! MCP and extension parameter builder method tests (TSK-077)
//!
//! ## Purpose
//!
//! Verify the seven MCP/extension `with_*()` methods add the correct CLI flags.
//!
//! ## Evidence
//!
//! - `with_mcp_config(["a","b"])` adds two `--mcp-config` pairs (Pattern F)
//! - `with_strict_mcp_config(true)` adds `--strict-mcp-config`
//! - `with_strict_mcp_config(false)` adds nothing
//! - `with_settings("path")` adds `--settings path`
//! - `with_setting_sources("local")` adds `--setting-sources local`
//! - `with_agent("reviewer")` adds `--agent reviewer`
//! - `with_agents(json)` adds `--agents <json>` (single JSON string, NOT an iterator)
//! - `with_plugin_dir(["p1","p2"])` adds two `--plugin-dir` pairs (Pattern F)
//!
//! ## Test Coverage Matrix
//!
//! | Method | flag present | flag absent | repeated |
//! |--------|-------------|-------------|----------|
//! | with_mcp_config | ✅ | — | ✅ |
//! | with_strict_mcp_config | ✅ | ✅ | — |
//! | with_settings | ✅ | — | — |
//! | with_setting_sources | ✅ | — | — |
//! | with_agent | ✅ | — | — |
//! | with_agents | ✅ | — | — |
//! | with_plugin_dir | ✅ | — | ✅ |

use claude_runner_core::ClaudeCommand;

fn args_of( cmd: &ClaudeCommand ) -> Vec<String> {
  let c = cmd.build_command_for_test();
  c.get_args().map( |a| a.to_string_lossy().to_string() ).collect()
}

// with_mcp_config (Pattern F: repeated-flag)

#[test]
fn with_mcp_config_single_adds_flag_and_path() {
  let cmd = ClaudeCommand::new().with_mcp_config( [ "/path/mcp.json" ] );
  let args = args_of( &cmd );
  assert!( args.contains( &"--mcp-config".to_string() ) );
  assert!( args.contains( &"/path/mcp.json".to_string() ) );
}

#[test]
fn with_mcp_config_two_values_produces_two_pairs() {
  let cmd = ClaudeCommand::new().with_mcp_config( [ "/a.json", "/b.json" ] );
  let args = args_of( &cmd );
  let count = args.iter().filter( |a| *a == "--mcp-config" ).count();
  assert_eq!( count, 2, "--mcp-config must appear twice: {args:?}" );
  assert!( args.contains( &"/a.json".to_string() ) );
  assert!( args.contains( &"/b.json".to_string() ) );
}

// with_strict_mcp_config

#[test]
fn with_strict_mcp_config_true_adds_flag() {
  let cmd = ClaudeCommand::new().with_strict_mcp_config( true );
  assert!( args_of( &cmd ).contains( &"--strict-mcp-config".to_string() ) );
}

#[test]
fn with_strict_mcp_config_false_adds_nothing() {
  let cmd = ClaudeCommand::new().with_strict_mcp_config( false );
  assert!( !args_of( &cmd ).contains( &"--strict-mcp-config".to_string() ) );
}

// with_settings

#[test]
fn with_settings_adds_flag_and_value() {
  let cmd = ClaudeCommand::new().with_settings( "/path/settings.json" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--settings".to_string() ) );
  assert!( args.contains( &"/path/settings.json".to_string() ) );
}

// with_setting_sources

#[test]
fn with_setting_sources_adds_flag_and_value() {
  let cmd = ClaudeCommand::new().with_setting_sources( "local" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--setting-sources".to_string() ) );
  assert!( args.contains( &"local".to_string() ) );
}

// with_agent

#[test]
fn with_agent_adds_flag_and_value() {
  let cmd = ClaudeCommand::new().with_agent( "reviewer" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--agent".to_string() ) );
  assert!( args.contains( &"reviewer".to_string() ) );
}

// with_agents (SINGLE JSON string, not an iterator)

#[test]
fn with_agents_adds_flag_and_json_string() {
  let json = r#"[{"name":"bot","model":"claude-opus-4-6"}]"#;
  let cmd = ClaudeCommand::new().with_agents( json );
  let args = args_of( &cmd );
  assert!( args.contains( &"--agents".to_string() ) );
  assert!( args.contains( &json.to_string() ), "json string must be present verbatim: {args:?}" );
}

#[test]
fn with_agents_takes_single_string_not_iterator() {
  // with_agents takes a single JSON string — it is NOT iterated
  // Calling it with a JSON array string should produce exactly one --agents flag
  let json = r#"[{"name":"a"},{"name":"b"}]"#;
  let cmd = ClaudeCommand::new().with_agents( json );
  let args = args_of( &cmd );
  let count = args.iter().filter( |a| *a == "--agents" ).count();
  assert_eq!( count, 1, "--agents must appear exactly once (single JSON string): {args:?}" );
}

// with_plugin_dir (Pattern F: repeated-flag)

#[test]
fn with_plugin_dir_single_adds_flag_and_path() {
  let cmd = ClaudeCommand::new().with_plugin_dir( [ "/plugins" ] );
  let args = args_of( &cmd );
  assert!( args.contains( &"--plugin-dir".to_string() ) );
  assert!( args.contains( &"/plugins".to_string() ) );
}

#[test]
fn with_plugin_dir_two_values_produces_two_pairs() {
  let cmd = ClaudeCommand::new().with_plugin_dir( [ "/p1", "/p2" ] );
  let args = args_of( &cmd );
  let count = args.iter().filter( |a| *a == "--plugin-dir" ).count();
  assert_eq!( count, 2, "--plugin-dir must appear twice: {args:?}" );
  assert!( args.contains( &"/p1".to_string() ) );
  assert!( args.contains( &"/p2".to_string() ) );
}

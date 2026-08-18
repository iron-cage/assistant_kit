//! Environment variable setting tests
//!
//! ## Purpose
//!
//! Verify `build_command()` sets correct environment variables for all parameters.
//!
//! ## Evidence
//!
//! - Each parameter sets its corresponding environment variable
//! - Environment variable names match Claude Code expectations
//! - Values are correctly formatted (strings, booleans, numbers)

use std::ffi::OsStr;
use claude_runner_core::{ ClaudeCommand, ActionMode, LogLevel };

#[test]
fn bash_timeout_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_bash_timeout_ms(3_600_000);

  let cmd = cmd_builder.build_command_for_test();

  // Verify CLAUDE_CODE_BASH_TIMEOUT is set
  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_BASH_TIMEOUT" ), "Missing CLAUDE_CODE_BASH_TIMEOUT env var" );
  assert!( debug.contains( "3600000" ), "Incorrect timeout value" );
}

#[test]
fn bash_max_timeout_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_bash_max_timeout_ms(7_200_000);

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_BASH_MAX_TIMEOUT" ), "Missing CLAUDE_CODE_BASH_MAX_TIMEOUT env var" );
  assert!( debug.contains( "7200000" ), "Incorrect max timeout value" );
}

#[test]
fn auto_continue_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_auto_continue(true);

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_AUTO_CONTINUE" ), "Missing CLAUDE_CODE_AUTO_CONTINUE env var" );
  assert!( debug.contains( "true" ), "Incorrect auto_continue value" );
}

#[test]
fn telemetry_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_telemetry(false);

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_TELEMETRY" ), "Missing CLAUDE_CODE_TELEMETRY env var" );
  assert!( debug.contains( "false" ), "Incorrect telemetry value" );
}

#[test]
fn auto_approve_tools_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_auto_approve_tools(true);

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_AUTO_APPROVE_TOOLS" ), "Missing CLAUDE_CODE_AUTO_APPROVE_TOOLS env var" );
  assert!( debug.contains( "true" ), "Incorrect auto_approve_tools value" );
}

#[test]
fn action_mode_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_action_mode(ActionMode::Allow);

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_ACTION_MODE" ), "Missing CLAUDE_CODE_ACTION_MODE env var" );
  assert!( debug.contains( "allow" ), "Incorrect action_mode value" );
}

#[test]
fn log_level_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_log_level(LogLevel::Debug);

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_LOG_LEVEL" ), "Missing CLAUDE_CODE_LOG_LEVEL env var" );
  assert!( debug.contains( "debug" ), "Incorrect log_level value" );
}

#[test]
fn temperature_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_temperature(0.7);

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_TEMPERATURE" ), "Missing CLAUDE_CODE_TEMPERATURE env var" );
  assert!( debug.contains( "0.7" ), "Incorrect temperature value" );
}

#[test]
fn sandbox_mode_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_sandbox_mode(false);

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_SANDBOX_MODE" ), "Missing CLAUDE_CODE_SANDBOX_MODE env var" );
  assert!( debug.contains( "false" ), "Incorrect sandbox_mode value" );
}

#[test]
fn session_dir_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_session_dir("/tmp/sessions");

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_SESSION_DIR" ), "Missing CLAUDE_CODE_SESSION_DIR env var" );
  assert!( debug.contains( "/tmp/sessions" ), "Incorrect session_dir value" );
}

#[test]
fn top_p_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_top_p(0.9);

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_TOP_P" ), "Missing CLAUDE_CODE_TOP_P env var" );
  assert!( debug.contains( "0.9" ), "Incorrect top_p value" );
}

#[test]
fn top_k_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_top_k(40);

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_TOP_K" ), "Missing CLAUDE_CODE_TOP_K env var" );
  assert!( debug.contains( "40" ), "Incorrect top_k value" );
}

#[test]
fn print_bg_wait_ceiling_ms_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_print_bg_wait_ceiling_ms(600_000);

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS" ), "Missing CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS env var" );
  assert!( debug.contains( "600000" ), "Incorrect print_bg_wait_ceiling_ms value" );
}

#[test]
fn compact_window_sets_env_var() {
  let cmd_builder = ClaudeCommand::new()
    .with_compact_window( Some( 500_000 ) );

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!( debug.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ), "Missing CLAUDE_CODE_AUTO_COMPACT_WINDOW env var" );
  assert!( debug.contains( "500000" ), "Incorrect compact_window value" );
}

#[test]
fn compact_window_none_suppresses_env_var() {
  // compact_window is the only Tier 1 field whose builder takes Option<u32> directly —
  // None fully suppresses the env var (deferring to the model-native window), exercising
  // the `if let Some( window )` branch of env_pairs() that no bare-value sibling covers
  let cmd_builder = ClaudeCommand::new()
    .with_compact_window( None );

  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );
  assert!(
    !debug.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ),
    "compact_window None must fully suppress the env var, got: {debug}"
  );
}

#[test]
fn defaults_set_tier1_env_vars() {
  // Verify Tier 1 defaults are set (different from standard)
  let cmd_builder = ClaudeCommand::new();
  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );

  // Tier 1 should have env vars set
  assert!( debug.contains( "CLAUDE_CODE_BASH_TIMEOUT" ), "Default bash_timeout not set" );
  assert!( debug.contains( "3600000" ), "Incorrect default bash_timeout" );

  assert!( debug.contains( "CLAUDE_CODE_BASH_MAX_TIMEOUT" ), "Default bash_max_timeout not set" );
  assert!( debug.contains( "7200000" ), "Incorrect default bash_max_timeout" );

  assert!( debug.contains( "CLAUDE_CODE_AUTO_CONTINUE" ), "Default auto_continue not set" );
  assert!( debug.contains( "true" ), "Incorrect default auto_continue" );

  assert!( debug.contains( "CLAUDE_CODE_TELEMETRY" ), "Default telemetry not set" );
  assert!( debug.contains( "false" ), "Incorrect default telemetry" );

  assert!( debug.contains( "CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS" ), "Default print_bg_wait_ceiling_ms not set" );
  assert!( debug.contains( "CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS=\"0\"" ), "Incorrect default print_bg_wait_ceiling_ms: expected 0" );

  assert!( debug.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW" ), "Default compact_window not set" );
  assert!( debug.contains( "CLAUDE_CODE_AUTO_COMPACT_WINDOW=\"300000\"" ), "Incorrect default compact_window: expected 300000" );
}

// ── CLAUDE_CODE_CHILD_SESSION removal ────────────────────────────────────────

/// `CLAUDE_CODE_CHILD_SESSION` is stripped unconditionally from the subprocess env.
///
/// `clr` is always a top-level launcher — the marker must never propagate into the Claude
/// Code session it spawns or that session emits a spurious "inherited marker" warning.
/// Removal must happen even when `CLAUDECODE` is kept via `--keep-claudecode`.
#[test]
fn child_session_marker_always_removed_by_default()
{
  let built = ClaudeCommand::new().build_command_for_test();
  let removed : Vec< &str > = built.get_envs()
    .filter_map( | ( k, v ) | if v.is_none() { k.to_str() } else { None } )
    .collect();
  assert!(
    removed.contains( &"CLAUDE_CODE_CHILD_SESSION" ),
    "CLAUDE_CODE_CHILD_SESSION must be stripped by default; removal list: {removed:?}"
  );
}

#[test]
fn child_session_marker_removed_even_when_claudecode_kept()
{
  let built = ClaudeCommand::new()
    .with_unset_claudecode( false )
    .build_command_for_test();
  let removed : Vec< &str > = built.get_envs()
    .filter_map( | ( k, v ) | if v.is_none() { k.to_str() } else { None } )
    .collect();
  assert!(
    removed.contains( &"CLAUDE_CODE_CHILD_SESSION" ),
    "CLAUDE_CODE_CHILD_SESSION must be removed even when unset_claudecode=false; got: {removed:?}"
  );
  assert!(
    !removed.contains( &"CLAUDECODE" ),
    "CLAUDECODE must NOT be in removal list when unset_claudecode=false; got: {removed:?}"
  );
}

#[test]
fn claudecode_not_removed_when_unset_claudecode_false()
{
  let built = ClaudeCommand::new()
    .with_unset_claudecode( false )
    .build_command_for_test();
  let removed_keys : Vec< _ > = built.get_envs()
    .filter_map( | ( k, v ) | if v.is_none() { Some( k.to_owned() ) } else { None } )
    .collect();
  assert!(
    !removed_keys.iter().any( | k | k == OsStr::new( "CLAUDECODE" ) ),
    "CLAUDECODE must NOT be removed when unset_claudecode=false"
  );
}

// ─────────────────────────────────────────────────────────────────────────────

/// `with_home_isolation()` strips every credential/endpoint override var from the
/// subprocess environment.
///
/// # Root Cause (audit-isolated-env-leak)
///
/// The isolated subprocess inherited `ANTHROPIC_API_KEY`, `CLAUDE_CODE_OAUTH_TOKEN`,
/// `CLAUDE_CONFIG_DIR`, etc. from the parent. Any of them makes the `claude` binary
/// authenticate as the parent's identity — or read the parent's real config dir —
/// silently bypassing the isolated `HOME`'s credentials file the caller supplied.
///
/// # Why Not Caught
///
/// All isolation tests asserted what the subprocess *receives* (HOME override, args);
/// none asserted what it must *not* receive. Inherited-env leaks are invisible unless
/// a test enumerates the removal list.
///
/// # Fix Applied
///
/// `with_home_isolation()` now sets a `home_isolation` flag; `removed_vars()` — the
/// single source of truth shared by `build_command()` and `describe()` — appends
/// `ISOLATION_REMOVED_VARS` when the flag is set.
///
/// # Prevention
///
/// When adding an env var the `claude` binary honors for auth/endpoint selection,
/// add it to `ISOLATION_REMOVED_VARS` and extend this test's list.
///
/// # Pitfall
///
/// Removals must flow through `removed_vars()` — an ad-hoc `env_remove()` call at one
/// spawn site would leave `describe()`/dry-run output diverging from real execution.
#[test]
fn home_isolation_strips_credential_override_vars()
{
  let built = ClaudeCommand::new()
    .with_home_isolation()
    .build_command_for_test();
  let removed : Vec< &str > = built.get_envs()
    .filter_map( | ( k, v ) | if v.is_none() { k.to_str() } else { None } )
    .collect();
  for var in [
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "ANTHROPIC_CUSTOM_HEADERS",
    "CLAUDE_CODE_OAUTH_TOKEN",
    "CLAUDE_CODE_USE_BEDROCK",
    "CLAUDE_CODE_USE_VERTEX",
    "CLAUDE_CONFIG_DIR",
  ]
  {
    assert!(
      removed.contains( &var ),
      "home isolation must strip {var}; removal list: {removed:?}"
    );
  }
}

/// Isolation removals surface in `describe()` as `-u NAME` tokens (display parity with
/// real execution, via the shared `removed_vars()` list).
#[test]
fn home_isolation_removals_visible_in_describe()
{
  let desc = ClaudeCommand::new().with_home_isolation().describe();
  assert!( desc.contains( "-u ANTHROPIC_API_KEY" ), "describe must show the strip: {desc}" );
  assert!( desc.contains( "-u CLAUDE_CONFIG_DIR" ), "describe must show the strip: {desc}" );
}

/// Counterpart: WITHOUT home isolation the credential vars are inherited untouched —
/// the scrub is strictly opt-in, so ordinary (non-isolated) invocations keep honoring
/// the caller's environment-based configuration.
#[test]
fn no_isolation_keeps_credential_vars()
{
  let built = ClaudeCommand::new().build_command_for_test();
  let removed : Vec< &str > = built.get_envs()
    .filter_map( | ( k, v ) | if v.is_none() { k.to_str() } else { None } )
    .collect();
  assert!(
    !removed.contains( &"ANTHROPIC_API_KEY" ),
    "ANTHROPIC_API_KEY must NOT be stripped without home isolation; got: {removed:?}"
  );
  assert!(
    !removed.contains( &"CLAUDE_CONFIG_DIR" ),
    "CLAUDE_CONFIG_DIR must NOT be stripped without home isolation; got: {removed:?}"
  );
}

// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn defaults_do_not_set_tier2_tier3_env_vars() {
  // Verify Tier 2 & 3 defaults are NOT set (inherit standard)
  let cmd_builder = ClaudeCommand::new();
  let cmd = cmd_builder.build_command_for_test();

  let debug = format!( "{cmd:?}" );

  // Tier 2 & 3 should NOT have env vars set (inherit standard defaults)
  assert!( !debug.contains( "CLAUDE_CODE_AUTO_APPROVE_TOOLS" ), "Tier 2 var incorrectly set" );
  assert!( !debug.contains( "CLAUDE_CODE_ACTION_MODE" ), "Tier 2 var incorrectly set" );
  assert!( !debug.contains( "CLAUDE_CODE_LOG_LEVEL" ), "Tier 2 var incorrectly set" );
  assert!( !debug.contains( "CLAUDE_CODE_TEMPERATURE" ), "Tier 2 var incorrectly set" );
  assert!( !debug.contains( "CLAUDE_CODE_SANDBOX_MODE" ), "Tier 3 var incorrectly set" );
  assert!( !debug.contains( "CLAUDE_CODE_SESSION_DIR" ), "Tier 3 var incorrectly set" );
  assert!( !debug.contains( "CLAUDE_CODE_TOP_P" ), "Tier 3 var incorrectly set" );
  assert!( !debug.contains( "CLAUDE_CODE_TOP_K" ), "Tier 3 var incorrectly set" );
}

//! Prompt and permission parameter builder method tests (TSK-075)
//!
//! ## Purpose
//!
//! Verify the three prompt/permission `with_*()` methods add the correct CLI flags.
//!
//! ## Evidence
//!
//! - `with_append_system_prompt(s)` adds `--append-system-prompt <s>`
//! - `with_permission_mode(PermissionMode::AcceptEdits)` adds `--permission-mode acceptEdits`
//! - `with_permission_mode(PermissionMode::BypassPermissions)` adds `--permission-mode bypassPermissions`
//! - `with_allow_dangerously_skip_permissions(true)` adds `--allow-dangerously-skip-permissions`
//! - `with_allow_dangerously_skip_permissions(false)` adds nothing
//! - `with_system_prompt` and `with_append_system_prompt` are independent (both can coexist)
//!
//! ## Test Coverage Matrix
//!
//! | Method | flag present | flag absent |
//! |--------|-------------|-------------|
//! | with_append_system_prompt | ✅ | — |
//! | with_permission_mode (AcceptEdits) | ✅ | — |
//! | with_permission_mode (BypassPermissions) | ✅ | — |
//! | with_allow_dangerously_skip_permissions | ✅ | ✅ |

use claude_runner_core::{ ClaudeCommand, PermissionMode };

fn args_of( cmd: &ClaudeCommand ) -> Vec<String> {
  let c = cmd.build_command_for_test();
  c.get_args().map( |a| a.to_string_lossy().to_string() ).collect()
}

// with_append_system_prompt

#[test]
fn with_append_system_prompt_adds_flag_and_value() {
  let cmd = ClaudeCommand::new().with_append_system_prompt( "You are cautious" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--append-system-prompt".to_string() ) );
  assert!( args.contains( &"You are cautious".to_string() ) );
}

#[test]
fn with_append_system_prompt_and_system_prompt_are_independent() {
  // Fix(issue-system-prompt-independence): append-system-prompt and system-prompt are independent flags
  let cmd = ClaudeCommand::new()
    .with_system_prompt( "Base prompt" )
    .with_append_system_prompt( "Appended" );
  let args = args_of( &cmd );
  assert!( args.contains( &"--system-prompt".to_string() ), "must contain --system-prompt" );
  assert!( args.contains( &"--append-system-prompt".to_string() ), "must contain --append-system-prompt" );
}

// with_permission_mode

#[test]
fn with_permission_mode_default_adds_flag() {
  let cmd = ClaudeCommand::new().with_permission_mode( PermissionMode::Default );
  let args = args_of( &cmd );
  assert!( args.contains( &"--permission-mode".to_string() ) );
  assert!( args.contains( &"default".to_string() ) );
}

#[test]
fn with_permission_mode_accept_edits_uses_camel_case() {
  // Fix(issue-permission-mode-camelcase): acceptEdits is camelCase, not lowercase
  let cmd = ClaudeCommand::new().with_permission_mode( PermissionMode::AcceptEdits );
  let args = args_of( &cmd );
  assert!( args.contains( &"--permission-mode".to_string() ) );
  assert!( args.contains( &"acceptEdits".to_string() ), "must use camelCase: {args:?}" );
  assert!( !args.contains( &"acceptedits".to_string() ), "must NOT use lowercase" );
}

#[test]
fn with_permission_mode_bypass_permissions_uses_camel_case() {
  // Fix(issue-permission-mode-camelcase): bypassPermissions is camelCase, not lowercase
  let cmd = ClaudeCommand::new().with_permission_mode( PermissionMode::BypassPermissions );
  let args = args_of( &cmd );
  assert!( args.contains( &"--permission-mode".to_string() ) );
  assert!( args.contains( &"bypassPermissions".to_string() ), "must use camelCase: {args:?}" );
  assert!( !args.contains( &"bypasspermissions".to_string() ), "must NOT use lowercase" );
}

// with_allow_dangerously_skip_permissions

#[test]
fn with_allow_dangerously_skip_permissions_true_adds_flag() {
  let cmd = ClaudeCommand::new().with_allow_dangerously_skip_permissions( true );
  assert!( args_of( &cmd ).contains( &"--allow-dangerously-skip-permissions".to_string() ) );
}

#[test]
fn with_allow_dangerously_skip_permissions_false_adds_nothing() {
  let cmd = ClaudeCommand::new().with_allow_dangerously_skip_permissions( false );
  assert!( !args_of( &cmd ).contains( &"--allow-dangerously-skip-permissions".to_string() ) );
}

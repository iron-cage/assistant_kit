//! Terminal and IDE integration parameter builder method tests (TSK-079)
//!
//! ## Purpose
//!
//! Verify the four terminal/IDE `with_*()` methods add the correct CLI flags.
//!
//! ## Evidence
//!
//! - `with_worktree(None)` adds `-w` with no name
//! - `with_worktree(Some("feature"))` adds `-w feature`
//! - `with_tmux(true)` adds `--tmux`; `with_tmux(false)` adds nothing
//! - `with_ide(true)` adds `--ide`; `with_ide(false)` adds nothing
//! - `ClaudeCommand::new()` emits `--chrome` by default (builder default: `Some(true)`)
//! - `with_chrome(Some(true))` adds `--chrome` (Pattern G tri-state)
//! - `with_chrome(Some(false))` adds `--no-chrome` (Pattern G tri-state)
//! - `with_chrome(None)` adds nothing — explicitly overrides the `Some(true)` default
//!
//! ## Test Coverage Matrix
//!
//! | Method | Some(true) | Some(false) | None | empty string |
//! |--------|-----------|------------|------|--------------|
//! | with_worktree(None) | ✅ | — | — | — |
//! | with_worktree(Some) | ✅ | — | — | ✅ |
//! | with_tmux | ✅ | ✅ | — | — |
//! | with_ide | ✅ | ✅ | — | — |
//! | with_chrome | ✅ | ✅ | ✅ (overrides default) | — |
//! | new() chrome default | — | — | — (emits --chrome) | — |

use claude_runner_core::ClaudeCommand;

fn args_of( cmd: &ClaudeCommand ) -> Vec<String> {
  let c = cmd.build_command_for_test();
  c.get_args().map( |a| a.to_string_lossy().to_string() ).collect()
}

// with_worktree

#[test]
fn with_worktree_none_adds_w_flag_only() {
  let cmd = ClaudeCommand::new().with_worktree( None );
  let args = args_of( &cmd );
  assert!( args.contains( &"-w".to_string() ), "must contain -w: {args:?}" );
}

#[test]
fn with_worktree_none_adds_exactly_one_extra_arg() {
  let baseline = args_of( &ClaudeCommand::new() ).len();
  let with_w = args_of( &ClaudeCommand::new().with_worktree( None ) ).len();
  assert_eq!( with_w, baseline + 1, "with_worktree(None) must add exactly 1 arg (-w)" );
}

#[test]
fn with_worktree_some_adds_w_and_name() {
  let cmd = ClaudeCommand::new().with_worktree( Some( "feature" ) );
  let args = args_of( &cmd );
  assert!( args.contains( &"-w".to_string() ) );
  assert!( args.contains( &"feature".to_string() ) );
}

#[test]
fn with_worktree_some_w_is_followed_by_name() {
  let cmd = ClaudeCommand::new().with_worktree( Some( "my-tree" ) );
  let args = args_of( &cmd );
  let w_pos = args.iter().position( |a| a == "-w" ).expect( "-w not found" );
  assert_eq!( args.get( w_pos + 1 ).map( String::as_str ), Some( "my-tree" ) );
}

// with_tmux

#[test]
fn with_tmux_true_adds_flag() {
  let cmd = ClaudeCommand::new().with_tmux( true );
  assert!( args_of( &cmd ).contains( &"--tmux".to_string() ) );
}

#[test]
fn with_tmux_false_adds_nothing() {
  let cmd = ClaudeCommand::new().with_tmux( false );
  assert!( !args_of( &cmd ).contains( &"--tmux".to_string() ) );
}

// with_ide

#[test]
fn with_ide_true_adds_flag() {
  let cmd = ClaudeCommand::new().with_ide( true );
  assert!( args_of( &cmd ).contains( &"--ide".to_string() ) );
}

#[test]
fn with_ide_false_adds_nothing() {
  let cmd = ClaudeCommand::new().with_ide( false );
  assert!( !args_of( &cmd ).contains( &"--ide".to_string() ) );
}

// with_chrome (Pattern G: tri-state)

#[test]
fn new_emits_chrome_by_default() {
  // Fix(issue-chrome-default): ClaudeCommand::new() defaults chrome to Some(true) → --chrome
  // Builder always emits --chrome unless explicitly overridden with with_chrome()
  let cmd = ClaudeCommand::new();
  let args = args_of( &cmd );
  assert!( args.contains( &"--chrome".to_string() ), "new() must emit --chrome by default: {args:?}" );
  assert!( !args.contains( &"--no-chrome".to_string() ) );
}

#[test]
fn with_chrome_some_true_adds_chrome_flag() {
  // Fix(issue-chrome-tristate): Some(true) → --chrome, Some(false) → --no-chrome, None → omit
  let cmd = ClaudeCommand::new().with_chrome( Some( true ) );
  let args = args_of( &cmd );
  assert!( args.contains( &"--chrome".to_string() ), "Some(true) must add --chrome: {args:?}" );
  assert!( !args.contains( &"--no-chrome".to_string() ) );
}

#[test]
fn with_chrome_some_false_adds_no_chrome_flag() {
  // Fix(issue-chrome-tristate): Some(false) must emit --no-chrome, not just omit
  let cmd = ClaudeCommand::new().with_chrome( Some( false ) );
  let args = args_of( &cmd );
  assert!( args.contains( &"--no-chrome".to_string() ), "Some(false) must add --no-chrome: {args:?}" );
  assert!( !args.contains( &"--chrome".to_string() ) );
}

#[test]
fn with_chrome_none_adds_nothing() {
  // with_chrome(None) explicitly overrides the Some(true) builder default to None
  // → no flag emitted; defers entirely to Claude's own config (which defaults to off)
  let cmd = ClaudeCommand::new().with_chrome( None );
  let args = args_of( &cmd );
  assert!( !args.contains( &"--chrome".to_string() ) );
  assert!( !args.contains( &"--no-chrome".to_string() ) );
}

#[test]
fn with_worktree_and_tmux_coexist() {
  // Verify worktree + tmux can be used together (from AF2 anti-faking check)
  let cmd = ClaudeCommand::new()
    .with_worktree( Some( "feat" ) )
    .with_tmux( true );
  let args = args_of( &cmd );
  assert!( args.contains( &"-w".to_string() ) );
  assert!( args.contains( &"feat".to_string() ) );
  assert!( args.contains( &"--tmux".to_string() ) );
}

// ── with_worktree(Some("")) — empty name edge case ────────────────────────────

#[test]
fn with_worktree_some_empty_string_adds_w_and_empty_name() {
  // with_worktree(Some("")) passes empty string as worktree name.
  // Builder does not validate names; empty string is passed through.
  let cmd = ClaudeCommand::new().with_worktree( Some( "" ) );
  let args = args_of( &cmd );
  assert!( args.contains( &"-w".to_string() ), "-w must be present: {args:?}" );
  let w_pos = args.iter().position( |a| a == "-w" ).expect( "-w not found" );
  assert_eq!(
    args.get( w_pos + 1 ).map( String::as_str ), Some( "" ),
    "empty name must follow -w: {args:?}"
  );
}

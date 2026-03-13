//! `claude_manager` binary entry point.
//!
//! Thin wrapper that delegates to [`claude_manager::run_cli()`].
//! The full pipeline implementation lives in the library so it is compiled
//! once and shared by the `claude_manager` and `clman` binary targets.

fn main()
{
  claude_manager::run_cli();
}

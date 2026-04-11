//! `claude_assets` binary entry point.
//!
//! Thin wrapper that delegates to [`claude_assets::run_cli()`].
//! The full pipeline implementation lives in the library so it is compiled
//! once and shared by the `claude_assets` and `cla` binary targets.

fn main()
{
  claude_assets::run_cli();
}

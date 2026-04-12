//! `claude_tools` binary entry point.
//!
//! Thin wrapper that delegates to [`claude_tools::run_cli()`].
//! The full pipeline implementation lives in the library so it is compiled
//! once and shared by the `claude_tools` and `clt` binary targets.

fn main()
{
  claude_tools::run_cli();
}

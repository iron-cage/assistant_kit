//! `claude_runner` binary entry point.
//!
//! Thin wrapper that delegates to [`claude_runner::run_cli()`].
//! The full pipeline implementation lives in the library so it is compiled
//! once and shared by the `claude_runner` and `clr` binary targets.

fn main()
{
  claude_runner::run_cli();
}

//! `claude_version` binary entry point.
//!
//! Thin wrapper that delegates to [`claude_version::run_cli()`].
//! The full pipeline implementation lives in the library so it is compiled
//! once and shared by the `claude_version` and `clv` binary targets.

fn main()
{
  claude_version::run_cli();
}

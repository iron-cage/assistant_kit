//! `claude_profile` binary entry point.
//!
//! Thin wrapper that delegates to [`claude_profile::run_cli()`].
//! The full pipeline implementation lives in the library so it is compiled
//! once and shared by the `claude_profile` and `clp` binary targets.

fn main()
{
  claude_profile::run_cli();
}

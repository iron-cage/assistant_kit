//! `claude_storage` binary entry point.
//!
//! Thin wrapper that delegates to [`claude_storage::cli_main::run()`].
//! The full REPL and one-shot pipeline lives in `cli_main` so it is compiled
//! once and shared by the `claude_storage` and `clg` binary targets.

fn main()
{
  claude_storage::cli_main::run();
}

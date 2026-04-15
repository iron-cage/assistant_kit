//! `assistant` binary entry point.
//!
//! Thin wrapper that delegates to [`assistant::run_cli()`].
//! The full pipeline implementation lives in the library so it is compiled
//! once and shared by the `assistant` and `clt` binary targets.

fn main()
{
  assistant::run_cli();
}

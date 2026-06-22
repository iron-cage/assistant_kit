//! `runbox` binary entry point.
//!
//! Thin wrapper that delegates to [`runbox::run_cli()`].
//! The full pipeline implementation lives in the library so it is compiled
//! once and shared by the `runbox` and `crb` binary targets.

fn main()
{
  runbox::run_cli();
}

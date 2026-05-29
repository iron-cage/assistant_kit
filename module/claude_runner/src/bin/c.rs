//! `c` binary entry point.
//!
//! Ultra-short alias — identical to the `clr` and `claude_runner` binaries.
//! Separate file gives Cargo a unique compilation unit per `[[bin]]` target,
//! eliminating the "same file in multiple targets" warning.

fn main()
{
  claude_runner::run_cli();
}

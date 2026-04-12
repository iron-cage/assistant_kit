//! `clp` binary entry point.
//!
//! Short alias — identical to the `claude_profile` binary.
//! This separate file gives Cargo a unique compilation unit per `[[bin]]`
//! target, eliminating the "same file in multiple targets" warning.

fn main()
{
  claude_profile::run_cli();
}

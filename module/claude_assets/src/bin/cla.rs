//! `cla` alias binary entry point.
//!
//! Thin wrapper — identical to the `claude_assets` binary.
//! This separate file gives Cargo a unique compilation unit per `[[bin]]`
//! target, eliminating the "same file in multiple targets" warning.

fn main()
{
  claude_assets::run_cli();
}

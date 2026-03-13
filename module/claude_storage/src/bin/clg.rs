//! `clg` alias binary entry point.
//!
//! Thin wrapper — identical to the `claude_storage` binary.
//! This separate file gives Cargo a unique compilation unit per `[[bin]]`
//! target, eliminating the "same file in multiple targets" warning.

fn main()
{
  claude_storage::cli_main::run();
}

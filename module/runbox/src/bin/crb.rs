//! `crb` alias binary entry point.
//!
//! Thin wrapper — identical to the `runbox` binary.
//! This separate file gives Cargo a unique compilation unit per `[[bin]]`
//! target, eliminating the "same file in multiple targets" warning.

fn main()
{
  runbox::run_cli();
}

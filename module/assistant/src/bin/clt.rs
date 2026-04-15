//! `clt` binary entry point.
//!
//! Short alias — identical to the `assistant` binary.
//! This separate file gives Cargo a unique compilation unit per `[[bin]]`
//! target, eliminating the "same file in multiple targets" warning.

fn main()
{
  assistant::run_cli();
}

//! Hand-rolled ANSI color helpers for `.show`/`.tail` output — no external
//! color crate dependency (see `docs/cli/readme.md` § Local Style Conventions).
//! Colorizing auto-disables when `NO_COLOR` is set or stdout is not a
//! terminal, so every piped/redirected invocation — including every
//! integration test, which spawns the binary via `std::process::Command` —
//! renders plain text.

use std::io::IsTerminal;

const RESET  : &str = "\u{1b}[0m";
const CYAN   : &str = "\u{1b}[36m";
const YELLOW : &str = "\u{1b}[33m";
const RED    : &str = "\u{1b}[31m";
const GREEN  : &str = "\u{1b}[32m";
const DIM    : &str = "\u{1b}[2m";

fn enabled() -> bool
{
  std::env::var_os( "NO_COLOR" ).is_none() && std::io::stdout().is_terminal()
}

fn wrap( code : &str, text : &str ) -> String
{
  if enabled()
  {
    format!( "{code}{text}{RESET}" )
  }
  else
  {
    text.to_string()
  }
}

/// Colorize a role label (e.g. `User:`, `Assistant`).
pub( super ) fn role( text : &str ) -> String
{
  wrap( CYAN, text )
}

/// Colorize a field-name label (e.g. `timestamp`, `content.text`).
pub( super ) fn field_name( text : &str ) -> String
{
  wrap( YELLOW, text )
}

/// Colorize an error marker (e.g. `Tool error`).
pub( super ) fn error_marker( text : &str ) -> String
{
  wrap( RED, text )
}

/// Colorize a role label by who is speaking — the two roles get distinct hues
/// so a turn's author is readable at a glance without reading the word.
pub( super ) fn speaker( entry_type : claude_storage_core::EntryType, text : &str ) -> String
{
  match entry_type
  {
    claude_storage_core::EntryType::User => wrap( GREEN, text ),
    claude_storage_core::EntryType::Assistant => wrap( CYAN, text ),
  }
}

/// Colorize secondary chrome (rule lines, relative times, continuation hints).
pub( super ) fn muted( text : &str ) -> String
{
  wrap( DIM, text )
}

/// Colorize a tool invocation line.
pub( super ) fn tool( text : &str ) -> String
{
  wrap( YELLOW, text )
}

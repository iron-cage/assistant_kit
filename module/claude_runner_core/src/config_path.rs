//! User-tier `config.toml` location — the single resolver every clr config
//! reader and writer shares (`clr run`/`ask`/`topic`'s config tier, `clr isolated`'s
//! default-model lookup, clp's `.model scope::subprocess`, `.provider.select`, and
//! `.usage` Gate 10).
//!
//! Resolution, first match wins:
//! 1. `$CLR_CONFIG_DIR/config.toml` when `CLR_CONFIG_DIR` is set and non-empty;
//! 2. `$HOME/.clr/config.toml` when `HOME` is set and non-empty;
//! 3. no user tier at all (`None`).
//!
//! Step 3 is deliberately not a cwd-relative `.clr/config.toml`: with neither
//! variable set there is no user to scope the file to, and a relative path would
//! read (or, for clp's writers, create) a `.clr/` directory wherever the process
//! happened to start.

use std::ffi::OsStr;
use std::path::PathBuf;

/// Pure form of [`user_config_path`]: resolves from explicit variable values
/// instead of reading the process environment, so callers and tests can pin
/// every combination without mutating global state.
///
/// An empty value counts as unset for both variables.
#[ must_use ]
#[ inline ]
pub fn user_config_path_from( clr_config_dir : Option< &OsStr >, home : Option< &OsStr > ) -> Option< PathBuf >
{
  if let Some( dir ) = clr_config_dir.filter( | v | !v.is_empty() )
  {
    return Some( PathBuf::from( dir ).join( "config.toml" ) );
  }
  home
    .filter( | v | !v.is_empty() )
    .map( | h | PathBuf::from( h ).join( ".clr" ).join( "config.toml" ) )
}

/// Resolve the user-tier `config.toml` path from the process environment
/// (`CLR_CONFIG_DIR`, then `HOME`). `None` means there is no user tier — readers
/// treat it as "nothing set", writers must refuse rather than guess a location.
#[ must_use ]
#[ inline ]
pub fn user_config_path() -> Option< PathBuf >
{
  user_config_path_from
  (
    std::env::var_os( "CLR_CONFIG_DIR" ).as_deref(),
    std::env::var_os( "HOME" ).as_deref(),
  )
}

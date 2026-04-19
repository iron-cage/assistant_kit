//! `VerbosityLevel` — CLI output verbosity control.
//!
//! Newtype wrapping `u8` with range 0–5. Controls how much diagnostic output
//! `claude_runner` emits to stdout/stderr. Does not affect Claude Code output.
//!
//! Level semantics:
//! - 0 — silent; runner emits nothing
//! - 1 — errors only; runner only prints fatal errors
//! - 2 — warnings; runner prints errors and warnings
//! - 3 — normal (default); runner prints progress and status
//! - 4 — verbose; runner prints detailed step-by-step progress
//! - 5 — debug; runner prints internal state, timing, paths

use core::fmt;
use core::str::FromStr;

/// Verbosity level for `claude_runner` diagnostic output.
///
/// Range: 0 (silent) to 5 (debug). Default is 3 (normal).
/// Controls only runner-side output — Claude Code subprocess output is unaffected.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord ) ]

pub struct VerbosityLevel( u8 );

impl VerbosityLevel
{
  /// Returns the inner verbosity level as `u8`.
  #[ must_use ]
  #[ inline ]
  pub fn get( self ) -> u8 { self.0 }

  /// Level ≥ 1: error messages are shown.
  #[ must_use ]
  #[ inline ]
  pub fn shows_errors( self ) -> bool { self.0 >= 1 }

  /// Level ≥ 2: warnings are shown.
  #[ must_use ]
  #[ inline ]
  pub fn shows_warnings( self ) -> bool { self.0 >= 2 }

  /// Level ≥ 3: progress and status output is shown (default behavior).
  #[ must_use ]
  #[ inline ]
  pub fn shows_progress( self ) -> bool { self.0 >= 3 }

  /// Level ≥ 4: verbose detail (command preview, step-by-step) is shown.
  #[ must_use ]
  #[ inline ]
  pub fn shows_verbose_detail( self ) -> bool { self.0 >= 4 }

  /// Level ≥ 5: debug output (paths, timing, internal state) is shown.
  #[ must_use ]
  #[ inline ]
  pub fn shows_debug( self ) -> bool { self.0 >= 5 }
}

impl Default for VerbosityLevel
{
  /// Default verbosity is 3 (normal).
  #[ inline ]
  fn default() -> Self { VerbosityLevel( 3 ) }
}

impl fmt::Display for VerbosityLevel
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    write!( f, "{}", self.0 )
  }
}

impl FromStr for VerbosityLevel
{
  type Err = String;

  #[ inline ]
  fn from_str( s : &str ) -> Result< Self, Self::Err >
  {
    let n : u8 = s.parse().map_err( | _ |
      format!( "invalid verbosity level: {s}\nExpected integer 0–5" )
    )?;
    if n > 5
    {
      return Err( format!( "verbosity level out of range: {n}\nExpected 0–5" ) );
    }
    Ok( VerbosityLevel( n ) )
  }
}

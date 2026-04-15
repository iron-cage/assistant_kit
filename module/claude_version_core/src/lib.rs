#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! Layer 1 domain helpers for Claude Code version management and settings.
//!
//! Depends only on [`claude_core`] — no CLI framework dependencies.
//!
//! # Modules
//!
//! - [`settings_io`]: Read and write `~/.claude/settings.json`
//! - [`version`]: Detect, install, resolve, and validate Claude Code versions
//!
//! # Error Handling
//!
//! Layer 1 functions use [`CoreError`] instead of unilang's `ErrorData`.
//! Layer 2 adapts at call sites: `.map_err(|e| ErrorData::new(code, e.to_string()))`.

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]
#![ warn( missing_docs ) ]
#![ warn( missing_debug_implementations ) ]

pub mod settings_io;
pub mod version;

/// Domain-level error type for Layer 1 operations.
///
/// Layer 2 adapts this to `ErrorData` at call sites via
/// `.map_err(|e| ErrorData::new(code, e.to_string()))`.
#[ derive( Debug ) ]
pub enum CoreError
{
  /// An I/O operation failed.
  IoError( std::io::Error ),
  /// A parse or validation error with a human-readable message.
  ParseError( String ),
  /// A subprocess or process-level error with a human-readable message.
  ProcessError( String ),
}

impl core::fmt::Display for CoreError
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    match self
    {
      Self::IoError( e )      => write!( f, "io: {e}" ),
      Self::ParseError( s )   => write!( f, "parse: {s}" ),
      Self::ProcessError( s ) => write!( f, "process: {s}" ),
    }
  }
}

impl core::error::Error for CoreError
{
  #[ inline ]
  fn source( &self ) -> Option< &( dyn core::error::Error + 'static ) >
  {
    match self
    {
      Self::IoError( e ) => Some( e ),
      _                  => None,
    }
  }
}

impl From< std::io::Error > for CoreError
{
  #[ inline ]
  fn from( e : std::io::Error ) -> Self
  {
    Self::IoError( e )
  }
}

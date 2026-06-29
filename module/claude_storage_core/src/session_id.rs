//! `SessionId` — typed wrapper for the UUID stem of a `.jsonl` session filename.
//!
//! Claude Code stores sessions as `<uuid>.jsonl` files inside
//! `~/.claude/projects/{encoded-path}/`.  A `SessionId` holds exactly the
//! stem — the UUID without the `.jsonl` extension — and prevents arbitrary
//! strings from being passed where a concrete session UUID is expected.

/// Typed wrapper for the UUID stem of a `.jsonl` session filename.
///
/// # Examples
///
/// ```
/// use claude_storage_core::SessionId;
///
/// let id = SessionId::new( "abc-123" );
/// assert_eq!( id.as_str(), "abc-123" );
/// assert_eq!( id.to_string(), "abc-123" );
///
/// let from_string : SessionId = String::from( "xyz" ).into();
/// let from_str    : SessionId = "xyz".into();
/// assert_eq!( from_string, from_str );
/// ```
#[ derive( Debug, Clone, PartialEq, Eq, Hash ) ]
pub struct SessionId( String );

impl SessionId
{
  /// Construct a `SessionId` from any string-like value.
  #[ must_use ]
  #[ inline ]
  pub fn new( id : impl Into< String > ) -> Self
  {
    Self( id.into() )
  }

  /// Return the inner UUID as a `&str`.
  #[ must_use ]
  #[ inline ]
  pub fn as_str( &self ) -> &str
  {
    &self.0
  }
}

impl core::fmt::Display for SessionId
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    f.write_str( &self.0 )
  }
}

impl AsRef< str > for SessionId
{
  #[ inline ]
  fn as_ref( &self ) -> &str { &self.0 }
}

impl From< String > for SessionId
{
  #[ inline ]
  fn from( s : String ) -> Self { Self( s ) }
}

impl From< &str > for SessionId
{
  #[ inline ]
  fn from( s : &str ) -> Self { Self( s.to_owned() ) }
}

//! The envelope every answer travels in.
//!
//! One JSON object per line. Generalized from `claude_runner/src/cli/query.rs`,
//! which established the `{ "ok": true, "result": … }` / `{ "ok": false, "error":
//! … }` shape against a per-PID socket — the shape carries no assumption about
//! what a request means or what `result` holds, so it moved here unchanged.

use serde::{ Deserialize, Serialize };

/// What a daemon answers, independent of what it was asked.
///
/// Serialized with an explicit `ok` discriminant rather than an externally
/// tagged enum, so a client written against the older `query.rs` shape reads it
/// unchanged.
#[ derive( Debug, Clone, PartialEq, Eq, Serialize, Deserialize ) ]
#[ serde( untagged ) ]
pub enum Response
{
  /// The request succeeded.
  Ok
  {
    /// Always `true`. Present so the two variants are distinguishable by a
    /// client that does not know this enum.
    ok : OkTrue,
    /// Method-specific payload.
    result : serde_json::Value,
  },
  /// The request failed.
  Err
  {
    /// Always `false`.
    ok : OkFalse,
    /// Human-readable failure description.
    error : String,
  },
}

impl Response
{
  /// Build a success response carrying `result`.
  #[ inline ]
  #[ must_use ]
  pub const fn ok( result : serde_json::Value ) -> Self
  {
    Self::Ok { ok : OkTrue, result }
  }

  /// Build a failure response carrying `error`.
  #[ inline ]
  #[ must_use ]
  pub fn err( error : impl Into< String > ) -> Self
  {
    Self::Err { ok : OkFalse, error : error.into() }
  }
}

/// The literal `true` in a successful [`Response`].
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub struct OkTrue;

/// The literal `false` in a failed [`Response`].
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub struct OkFalse;

impl Serialize for OkTrue
{
  #[ inline ]
  fn serialize< S : serde::Serializer >( &self, s : S ) -> core::result::Result< S::Ok, S::Error >
  {
    s.serialize_bool( true )
  }
}

impl Serialize for OkFalse
{
  #[ inline ]
  fn serialize< S : serde::Serializer >( &self, s : S ) -> core::result::Result< S::Ok, S::Error >
  {
    s.serialize_bool( false )
  }
}

impl< 'de > Deserialize< 'de > for OkTrue
{
  #[ inline ]
  fn deserialize< D : serde::Deserializer< 'de > >( d : D ) -> core::result::Result< Self, D::Error >
  {
    if bool::deserialize( d )?
    {
      Ok( Self )
    }
    else
    {
      Err( serde::de::Error::custom( "expected ok:true" ) )
    }
  }
}

impl< 'de > Deserialize< 'de > for OkFalse
{
  #[ inline ]
  fn deserialize< D : serde::Deserializer< 'de > >( d : D ) -> core::result::Result< Self, D::Error >
  {
    if bool::deserialize( d )?
    {
      Err( serde::de::Error::custom( "expected ok:false" ) )
    }
    else
    {
      Ok( Self )
    }
  }
}

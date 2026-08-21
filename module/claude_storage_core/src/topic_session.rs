//! Deterministic topic-name → session-UUID rule for fork-based topics.
//!
//! # Design
//!
//! A fork-based topic never gets its own working directory: it lives as an
//! ordinary session file inside its base directory's Claude storage, named by a
//! UUID computed deterministically from `(canonical base path, topic name)`.
//! Because the identity is a pure function, any consumer — `clr`'s topic
//! machinery, `claude_storage`'s path queries, a shell one-liner — resolves the
//! same topic to the same session file with zero coordination and zero disk
//! access.
//!
//! # The Rule
//!
//! ```text
//! topic session UUID = UUIDv5( TOPIC_NAMESPACE, "<canonical base path>\0<topic name>" )
//! session file       = <storage dir of base>/<uuid>.jsonl
//! ```
//!
//! - `TOPIC_NAMESPACE` is itself `UUIDv5` of the RFC 4122 DNS namespace and the
//!   name `clr.topic` — `f2b5cc6a-c186-5cc7-99db-3075d9c705f8`, reproducible
//!   with `uuidgen --sha1 --namespace @dns --name clr.topic`.
//! - The base path MUST be in canonical physical form
//!   ([`physical_abs`][crate::physical_abs]) — the same form Claude Code's own
//!   storage encoding keys on. Hashing a non-canonical spelling would split one
//!   topic into distinct sessions depending on how the caller spelled the path.
//! - `\0` (NUL) separates path from name. NUL can appear in neither a Unix path
//!   nor a CLI-supplied topic name, so the concatenation is unambiguous —
//!   `("/a", "bc")` and `("/a/b", "c")` can never collide.
//!
//! `UUIDv5` is one-way: the topic name cannot be recovered from the session file
//! name. Listing fork topics by name therefore needs a consumer-side registry;
//! this module deliberately stays a pure name→UUID function.
//!
//! # Verification
//!
//! Any vector below can be cross-checked against independent implementations:
//!
//! ```sh
//! uuidgen --sha1 --namespace @dns --name clr.topic
//! python3 -c 'import uuid; ns = uuid.uuid5( uuid.NAMESPACE_DNS, "clr.topic" ); \
//!   print( uuid.uuid5( ns, "/home/user1/pro\0review" ) )'
//! ```

use std::path::{ Path, PathBuf };
use crate::{ Error, Result, SessionId };
use crate::continuation::to_storage_path_for;

/// `UUIDv5` namespace for topic-session identities:
/// `UUIDv5( DNS namespace, "clr.topic" )` = `f2b5cc6a-c186-5cc7-99db-3075d9c705f8`.
///
/// Derived (rather than randomly generated) so the constant is auditable from
/// public inputs: `uuidgen --sha1 --namespace @dns --name clr.topic`.
const TOPIC_NAMESPACE : [ u8; 16 ] =
[
  0xf2, 0xb5, 0xcc, 0x6a, 0xc1, 0x86, 0x5c, 0xc7,
  0x99, 0xdb, 0x30, 0x75, 0xd9, 0xc7, 0x05, 0xf8,
];

/// Compute the deterministic session UUID for `topic` under `canonical_base`.
///
/// `canonical_base` MUST already be in canonical physical absolute form (see
/// [`physical_abs`][crate::physical_abs]); this function hashes the bytes it is
/// given and cannot detect a non-canonical spelling.
///
/// # Errors
///
/// Returns [`Error::PathEncoding`][crate::Error] when `canonical_base` is not
/// valid UTF-8 — the same restriction [`encode_path`][crate::encode_path]
/// already places on every storage-key computation.
///
/// # Examples
///
/// ```
/// use claude_storage_core::topic_session_id;
/// use std::path::Path;
///
/// let id = topic_session_id( Path::new( "/home/user1/pro" ), "review" )?;
/// assert_eq!( id.as_str(), "e36d752a-341e-5db1-94c5-c8b91cccbfff" );
/// # Ok::<(), claude_storage_core::Error>(())
/// ```
#[ inline ]
pub fn topic_session_id( canonical_base : &Path, topic : &str ) -> Result< SessionId >
{
  let base_str = canonical_base.to_str().ok_or_else( || Error::path_encoding
  (
    format!( "{}", canonical_base.display() ),
    "path contains invalid UTF-8".to_string(),
  ))?;

  let mut name = Vec::with_capacity( base_str.len() + 1 + topic.len() );
  name.extend_from_slice( base_str.as_bytes() );
  name.push( 0 );
  name.extend_from_slice( topic.as_bytes() );

  Ok( SessionId::new( uuid_v5( &TOPIC_NAMESPACE, &name ) ) )
}

/// Compute the absolute session file path for `topic` under `canonical_base`:
/// `<storage dir of base>/<uuid>.jsonl`.
///
/// Pure computation — the file need not exist; this answers "where would this
/// topic's session live?" identically whether or not it has ever been used.
///
/// Returns `None` when the storage dir cannot be resolved (neither
/// `CLAUDE_HOME` nor `HOME` set, or the base path cannot be encoded) — the same
/// contract as [`to_storage_path_for`][crate::to_storage_path_for], which
/// supplies the directory half of the join.
///
/// # Examples
///
/// ```no_run
/// use claude_storage_core::topic_session_file;
/// use std::path::Path;
///
/// let file = topic_session_file( Path::new( "/home/user1/pro" ), "review" ).unwrap();
/// assert!( file.to_str().unwrap().ends_with( "e36d752a-341e-5db1-94c5-c8b91cccbfff.jsonl" ) );
/// ```
#[ inline ]
#[ must_use ]
pub fn topic_session_file( canonical_base : &Path, topic : &str ) -> Option< PathBuf >
{
  let id = topic_session_id( canonical_base, topic ).ok()?;
  let storage = to_storage_path_for( canonical_base )?;
  Some( storage.join( format!( "{}.jsonl", id.as_str() ) ) )
}

/// Assemble an RFC 4122 version-5 UUID from a namespace and a name, formatted
/// as lowercase hyphenated hex (`8-4-4-4-12`).
///
/// Version 5 = SHA-1 of `namespace bytes ++ name bytes`, truncated to 16
/// octets, with the version nibble forced to `5` and the variant bits to
/// `10xx` (RFC 4122 § 4.3).
fn uuid_v5( namespace : &[ u8; 16 ], name : &[ u8 ] ) -> String
{
  let mut input = Vec::with_capacity( 16 + name.len() );
  input.extend_from_slice( namespace );
  input.extend_from_slice( name );
  let hash = sha1( &input );

  let mut octets = [ 0u8; 16 ];
  octets.copy_from_slice( &hash[ ..16 ] );
  octets[ 6 ] = ( octets[ 6 ] & 0x0F ) | 0x50; // version 5
  octets[ 8 ] = ( octets[ 8 ] & 0x3F ) | 0x80; // variant RFC 4122

  let mut out = String::with_capacity( 36 );
  for ( i, byte ) in octets.iter().enumerate()
  {
    if i == 4 || i == 6 || i == 8 || i == 10 { out.push( '-' ); }
    let hi = byte >> 4;
    let lo = byte & 0x0F;
    out.push( char::from_digit( u32::from( hi ), 16 ).expect( "nibble < 16" ) );
    out.push( char::from_digit( u32::from( lo ), 16 ).expect( "nibble < 16" ) );
  }
  out
}

/// SHA-1 digest (FIPS 180-4), hand-written against the zero-dependency core
/// guarantee — same precedent as `path.rs`'s djb2 hash. Used only for `UUIDv5`
/// name hashing (a deterministic naming scheme, not a security boundary), where
/// SHA-1 is what RFC 4122 specifies.
#[ allow( clippy::many_single_char_names ) ] // a-f/h/k/w are FIPS 180-4's own variable names
fn sha1( data : &[ u8 ] ) -> [ u8; 20 ]
{
  let mut h : [ u32; 5 ] =
  [ 0x6745_2301, 0xEFCD_AB89, 0x98BA_DCFE, 0x1032_5476, 0xC3D2_E1F0 ];

  // Padding: 0x80, zeros to 56 mod 64, then the 64-bit big-endian bit length.
  let mut message = data.to_vec();
  let bit_len = ( data.len() as u64 ).wrapping_mul( 8 );
  message.push( 0x80 );
  while message.len() % 64 != 56 { message.push( 0 ); }
  message.extend_from_slice( &bit_len.to_be_bytes() );

  for chunk in message.chunks_exact( 64 )
  {
    let mut w = [ 0u32; 80 ];
    for ( i, word ) in chunk.chunks_exact( 4 ).enumerate()
    {
      w[ i ] = u32::from_be_bytes( [ word[ 0 ], word[ 1 ], word[ 2 ], word[ 3 ] ] );
    }
    for i in 16 .. 80
    {
      w[ i ] = ( w[ i - 3 ] ^ w[ i - 8 ] ^ w[ i - 14 ] ^ w[ i - 16 ] ).rotate_left( 1 );
    }

    let ( mut a, mut b, mut c, mut d, mut e ) = ( h[ 0 ], h[ 1 ], h[ 2 ], h[ 3 ], h[ 4 ] );

    for ( i, &word ) in w.iter().enumerate()
    {
      let ( f, k ) = match i
      {
        0 ..= 19  => ( ( b & c ) | ( !b & d ),           0x5A82_7999 ),
        20 ..= 39 => ( b ^ c ^ d,                        0x6ED9_EBA1 ),
        40 ..= 59 => ( ( b & c ) | ( b & d ) | ( c & d ), 0x8F1B_BCDC ),
        _         => ( b ^ c ^ d,                        0xCA62_C1D6 ),
      };
      let temp = a.rotate_left( 5 )
        .wrapping_add( f )
        .wrapping_add( e )
        .wrapping_add( k )
        .wrapping_add( word );
      e = d;
      d = c;
      c = b.rotate_left( 30 );
      b = a;
      a = temp;
    }

    h[ 0 ] = h[ 0 ].wrapping_add( a );
    h[ 1 ] = h[ 1 ].wrapping_add( b );
    h[ 2 ] = h[ 2 ].wrapping_add( c );
    h[ 3 ] = h[ 3 ].wrapping_add( d );
    h[ 4 ] = h[ 4 ].wrapping_add( e );
  }

  let mut out = [ 0u8; 20 ];
  for ( i, word ) in h.iter().enumerate()
  {
    out[ i * 4 .. i * 4 + 4 ].copy_from_slice( &word.to_be_bytes() );
  }
  out
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  fn hex( bytes : &[ u8 ] ) -> String
  {
    use core::fmt::Write as _;
    bytes.iter().fold( String::new(), | mut acc, b |
    {
      write!( acc, "{b:02x}" ).expect( "String write! is infallible" );
      acc
    } )
  }

  #[ test ]
  fn sha1_empty_vector()
  {
    assert_eq!( hex( &sha1( b"" ) ), "da39a3ee5e6b4b0d3255bfef95601890afd80709" );
  }

  #[ test ]
  fn sha1_abc_vector()
  {
    assert_eq!( hex( &sha1( b"abc" ) ), "a9993e364706816aba3e25717850c26c9cd0d89d" );
  }

  #[ test ]
  fn sha1_multi_block_vector()
  {
    // 43 bytes crosses no block boundary; repeat to force a second 64-byte block.
    assert_eq!
    (
      hex( &sha1( b"The quick brown fox jumps over the lazy dog" ) ),
      "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12"
    );
    let long : Vec< u8 > = b"0123456789".repeat( 20 );
    assert_eq!( long.len(), 200 ); // > 3 blocks with padding
    assert_eq!( hex( &sha1( &long ) ), "efeeb70467c1ca141619c954c2a0e699bd1f16a5" );
  }
}

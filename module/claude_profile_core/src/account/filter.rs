//! Per-identity tag filter: `_filter_{hostname}_{user}` file IO and the
//! rotation eligibility predicate (Feature 076).
//!
//! One JSON file per identity in the credential store — exactly two keys,
//! `{"include":[...],"exclude":[...]}`, both sorted/deduplicated
//! (`docs/schema/009_identity_filter_json.md`). Absent file ≡ permit-all;
//! store-resident (NOT gitignored) so filters sync across machines. Binds
//! automatic selection only (Gate 11) — explicit `.account.use name::X` is
//! never filtered.

use std::path::Path;
use claude_core::file_io::atomic_write;
use super::ownership::host_user_slug;
use super::tags::normalize_tag_set;

/// A per-identity include/exclude tag set pair gating automatic account
/// selection (`docs/type/004_tag_filter.md`). Eligibility over an account's
/// tag set `T`: `T ⊇ include ∧ T ∩ exclude = ∅` — see [`eligible`].
#[ derive( Debug, Clone, PartialEq, Eq, Default ) ]
pub struct TagFilter
{
  /// Tags an eligible account must ALL carry; empty = no requirement.
  pub include : Vec< String >,
  /// Tags an eligible account must carry NONE of; empty = nothing blocked.
  pub exclude : Vec< String >,
}

/// This identity's filter filename — `_filter_` + the exact slug the
/// `_active_*` marker uses (same `host_user_slug()` sanitization, AC-08).
#[ inline ]
#[ must_use ]
pub fn filter_filename() -> String
{
  format!( "_filter_{}", host_user_slug() )
}

/// Extract a string-array key from a parsed filter object; absent key reads
/// as empty (forward tolerance — unknown keys are ignored, missing sides are
/// no-requirement).
fn string_array( obj : &serde_json::Map< String, serde_json::Value >, key : &str ) -> Vec< String >
{
  obj
    .get( key )
    .and_then( | v | v.as_array() )
    .map( | a | a.iter().filter_map( | v | v.as_str().map( str::to_string ) ).collect() )
    .unwrap_or_default()
}

/// Read this identity's tag filter from `credential_store`.
///
/// An absent file is the permit-all default (both sets empty) — the
/// zero-migration adoption path. Malformed content is a LOUD error, never a
/// silent permit-all (Feature 076/AC-16).
///
/// # Errors
///
/// Returns `InvalidData` naming the file when its content is not a JSON
/// object, or the underlying IO error for any read failure other than
/// not-found.
pub fn read_filter( credential_store : &Path ) -> Result< TagFilter, std::io::Error >
{
  let path = credential_store.join( filter_filename() );
  let text = match std::fs::read_to_string( &path )
  {
    Ok( t ) => t,
    Err( e ) if e.kind() == std::io::ErrorKind::NotFound => return Ok( TagFilter::default() ),
    Err( e ) => return Err( e ),
  };
  let value : serde_json::Value = serde_json::from_str( &text ).map_err( | e |
    std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      format!( "malformed filter file '{}': {e}", path.display() ),
    ) )?;
  let Some( obj ) = value.as_object() else
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      format!( "malformed filter file '{}': not a JSON object", path.display() ),
    ) );
  };
  Ok( TagFilter
  {
    include : string_array( obj, "include" ),
    exclude : string_array( obj, "exclude" ),
  } )
}

/// Write this identity's tag filter: each given side fully REPLACES that
/// side; an omitted (`None`) side is preserved from the existing file. Both
/// sides are normalized (lowercased, validated, deduplicated, sorted) via
/// `tags.rs`, and a non-empty `include ∩ exclude` is rejected — against the
/// post-normalization sets — before any write (Feature 076/AC-05). The stored
/// file carries exactly the two schema keys. Returns the stored filter.
///
/// # Errors
///
/// Returns `InvalidInput` naming the overlap for a contradictory filter, the
/// [`normalize_tag_set`] error for an invalid tag on either side, a
/// [`read_filter`] error when a preserved side must be loaded from a
/// malformed file, or an IO error when the file cannot be written.
pub fn write_filter(
  credential_store : &Path,
  include : Option< &[ String ] >,
  exclude : Option< &[ String ] >,
) -> Result< TagFilter, std::io::Error >
{
  // Both sides given → full overwrite, no merge read needed (also the repair
  // path for a malformed existing file, which read_filter() rejects loudly).
  let current = if include.is_some() && exclude.is_some()
  { TagFilter::default() }
  else
  { read_filter( credential_store )? };

  let next = TagFilter
  {
    include : normalize_tag_set( include.unwrap_or( &current.include ) )?,
    exclude : normalize_tag_set( exclude.unwrap_or( &current.exclude ) )?,
  };

  let overlap : Vec< String > = next.include.iter()
    .filter( | t | next.exclude.contains( t ) )
    .cloned()
    .collect();
  if !overlap.is_empty()
  {
    return Err( std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      format!( "include ∩ exclude must be empty — overlapping: {}", overlap.join( ", " ) ),
    ) );
  }

  let json = serde_json::json!( { "include" : next.include, "exclude" : next.exclude } );
  atomic_write(
    &credential_store.join( filter_filename() ),
    &serde_json::to_string_pretty( &json ).map( | s | s + "\n" ).unwrap_or_default(),
  )?;
  Ok( next )
}

/// Rotation eligibility predicate (Gate 11): `tags ⊇ include ∧ tags ∩ exclude = ∅`.
///
/// Pure set arithmetic over plain arguments — no config or file read — so the
/// rotation loop can call it per-candidate without per-candidate IO (C8). The
/// untagged corner follows directly: an empty tag set fails any non-empty
/// `include` and trivially passes an exclude-only filter.
#[ inline ]
#[ must_use ]
pub fn eligible( tags : &[ String ], filter : &TagFilter ) -> bool
{
  filter.include.iter().all( | t | tags.contains( t ) )
    && filter.exclude.iter().all( | t | !tags.contains( t ) )
}

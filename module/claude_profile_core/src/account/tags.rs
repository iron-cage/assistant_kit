//! Account tag set: normalization, mutation ops on `{name}.json`, and the lazy
//! `role` migration (Feature 075).
//!
//! Tags are flat lowercase labels — charset `[a-z0-9_-]`, 1–64 chars — stored as
//! a sorted, deduplicated JSON string array under `tags` in `{name}.json`
//! (`docs/type/003_tag.md`). The legacy free-form `role` field is superseded:
//! the first tag write of ANY variant (including a pure remove) migrates a
//! non-empty `role` into the tag set and deletes the key in the same write.

use std::path::Path;
use claude_core::file_io::atomic_write;

/// `true` for characters the tag charset `[a-z0-9_-]` permits.
fn tag_char_ok( c : char ) -> bool
{
  c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-'
}

/// Build the `InvalidInput` error every tag-validation failure returns.
// core::io::ErrorKind requires the unstable `core_io` feature (rust-lang/rust#154046) — not usable on stable.
#[ allow( clippy::std_instead_of_core ) ]
fn invalid( msg : String ) -> std::io::Error
{
  std::io::Error::new( std::io::ErrorKind::InvalidInput, msg )
}

/// Normalize one raw tag: lowercase FIRST, then validate charset `[a-z0-9_-]`
/// and length 1–64 — so `CI` is accepted as `ci`, never rejected for case
/// alone (Feature 075/AC-03). The violating tag is named in every error.
///
/// # Errors
///
/// Returns `InvalidInput` when the lowercased tag is empty, exceeds 64
/// characters, or contains a character outside `[a-z0-9_-]`.
#[ inline ]
pub fn normalize_tag( raw : &str ) -> Result< String, std::io::Error >
{
  let tag = raw.to_lowercase();
  if tag.is_empty()
  {
    return Err( invalid( "tag is empty — a tag needs 1-64 chars from [a-z0-9_-]".to_string() ) );
  }
  if tag.len() > 64
  {
    return Err( invalid( format!( "tag '{tag}' is {} chars — the maximum is 64", tag.len() ) ) );
  }
  if let Some( bad ) = tag.chars().find( | c | !tag_char_ok( *c ) )
  {
    return Err( invalid( format!( "tag '{tag}' has invalid character '{bad}' — allowed: [a-z0-9_-]" ) ) );
  }
  Ok( tag )
}

/// Normalize a whole tag list: [`normalize_tag`] each entry, then deduplicate
/// and sort — the canonical form every write path stores (Feature 075/AC-01).
///
/// # Errors
///
/// Returns the first entry's [`normalize_tag`] error, unchanged.
#[ inline ]
pub fn normalize_tag_set( raws : &[ String ] ) -> Result< Vec< String >, std::io::Error >
{
  let mut out = Vec::with_capacity( raws.len() );
  for raw in raws
  {
    out.push( normalize_tag( raw )? );
  }
  out.sort();
  out.dedup();
  Ok( out )
}

/// Coerce a legacy `role` value into the tag charset for the lazy migration:
/// lowercase, map anything outside `[a-z0-9_-]` to `_`, truncate to 64 chars.
/// `None` when nothing survives (empty role) — migration then adds no entry.
///
/// Deliberately coercing rather than rejecting: a weird legacy `role` must not
/// make an account's FIRST tag write fail (`docs/type/003_tag.md` § Relationships).
fn sanitize_tag( raw : &str ) -> Option< String >
{
  let cleaned : String = raw
    .to_lowercase()
    .chars()
    .map( | c | if tag_char_ok( c ) { c } else { '_' } )
    .take( 64 )
    .collect();
  if cleaned.is_empty() { None } else { Some( cleaned ) }
}

/// A tag-set mutation — the three write variants of Feature 075. Carries raw
/// (pre-normalization) tags; every consumer normalizes before touching state.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub enum TagOp
{
  /// Union the given tags into the stored set.
  Add( Vec< String > ),
  /// Remove the given tags from the stored set; absent tags are a no-op.
  Remove( Vec< String > ),
  /// Overwrite the stored set with the given tags.
  Replace( Vec< String > ),
}

/// Read `{name}.json` into a JSON object map — absent, unreadable, or
/// non-object content yields an empty map (same tolerance as the
/// `write_owner()` read-merge precedent in `ownership.rs`).
fn read_meta_object( path : &Path ) -> serde_json::Map< String, serde_json::Value >
{
  std::fs::read_to_string( path )
    .ok()
    .and_then( | s | serde_json::from_str::< serde_json::Value >( &s ).ok() )
    .and_then( | v | v.as_object().cloned() )
    .unwrap_or_default()
}

/// Apply a tag write to an already-parsed `{name}.json` object, including the
/// lazy `role` migration (Feature 075/AC-09). Shared by [`write_tags`],
/// [`preview_tags`], and `store.rs`'s `save()` tags parameter — the single
/// implementation of the set arithmetic.
///
/// Migration: any `role` key is deleted; a non-empty value is sanitized into
/// the tag set — joined to the op's BASE for add/remove (so a first-write
/// remove naming the migrated tag still drops it), and to the REPLACEMENT set
/// for replace (the value survives an overwrite, per feature 075 AC-09).
///
/// The op's tags are normalized before the object is touched, so a rejection
/// leaves `obj` unmodified. Returns the final sorted, deduplicated set now
/// present under `tags`.
pub( super ) fn apply_tag_write(
  obj : &mut serde_json::Map< String, serde_json::Value >,
  op : &TagOp,
) -> Result< Vec< String >, std::io::Error >
{
  let given = normalize_tag_set( match op
  {
    TagOp::Add( raw ) | TagOp::Remove( raw ) | TagOp::Replace( raw ) => raw,
  } )?;

  let stored : Vec< String > = obj
    .get( "tags" )
    .and_then( | v | v.as_array() )
    .map( | a | a.iter().filter_map( | v | v.as_str().map( str::to_string ) ).collect() )
    .unwrap_or_default();
  let migrated = obj
    .remove( "role" )
    .and_then( | v | v.as_str().map( str::to_string ) )
    .and_then( | r | sanitize_tag( &r ) );

  let mut result = match op
  {
    TagOp::Add( _ ) =>
    {
      let mut base = stored;
      base.extend( migrated );
      base.extend( given );
      base
    }
    TagOp::Remove( _ ) =>
    {
      let mut base = stored;
      base.extend( migrated );
      base.retain( | t | !given.contains( t ) );
      base
    }
    TagOp::Replace( _ ) =>
    {
      let mut base = given;
      base.extend( migrated );
      base
    }
  };
  result.sort();
  result.dedup();
  obj.insert(
    "tags".to_string(),
    serde_json::Value::Array( result.iter().map( | t | serde_json::Value::String( t.clone() ) ).collect() ),
  );
  Ok( result )
}

/// Apply `op` to the stored tag set of `name` via read-merge on `{name}.json`
/// — every other field is preserved; the file is created if absent. Returns
/// the final stored set.
///
/// # Errors
///
/// Returns `InvalidInput` (nothing written, file byte-identical) when `op`
/// carries an invalid tag, or an IO error when the merged file cannot be
/// written.
#[ inline ]
pub fn write_tags( name : &str, credential_store : &Path, op : &TagOp ) -> Result< Vec< String >, std::io::Error >
{
  let path = credential_store.join( format!( "{name}.json" ) );
  let mut obj = read_meta_object( &path );
  let result = apply_tag_write( &mut obj, op )?;
  let json = serde_json::to_string_pretty( &serde_json::Value::Object( obj ) )
    .map( | s | s + "\n" )
    .unwrap_or_default();
  atomic_write( &path, &json )?;
  Ok( result )
}

/// Compute the set [`write_tags`] would store for `name` and `op` — same
/// normalization and `role` migration — without writing anything (dry-run
/// support for `.account.tag dry::1`).
///
/// # Errors
///
/// Returns `InvalidInput` when `op` carries an invalid tag.
#[ inline ]
pub fn preview_tags( name : &str, credential_store : &Path, op : &TagOp ) -> Result< Vec< String >, std::io::Error >
{
  let mut obj = read_meta_object( &credential_store.join( format!( "{name}.json" ) ) );
  apply_tag_write( &mut obj, op )
}

//! Unit tests for `physical_abs` — canonical physical path resolution.
//!
//! ## Purpose
//!
//! Every storage-key computation hashes or encodes the canonical physical form
//! of a path; a divergence between how two callers resolve the same spelling
//! silently splits one storage identity into two. These tests pin the three
//! resolution behaviors `physical_abs` guarantees: existing paths resolve
//! through symlinks, nonexistent tails append literally under a canonicalized
//! existing prefix, and `..`/`.` components normalize against the resolved
//! prefix (Fix(BUG-543)) rather than surviving lexically.

use std::path::Path;
use claude_storage_core::physical_abs;

/// An existing path resolves via full canonicalization (symlinks resolved).
#[ cfg( unix ) ]
#[ test ]
fn existing_symlink_resolves_to_target()
{
  let tmp = tempfile::TempDir::new().unwrap();
  let real = tmp.path().join( "real" );
  std::fs::create_dir( &real ).unwrap();
  let link = tmp.path().join( "link" );
  std::os::unix::fs::symlink( &real, &link ).unwrap();

  let canonical_real = std::fs::canonicalize( &real ).unwrap();
  assert_eq!( physical_abs( &link ), canonical_real );
}

/// A nonexistent tail under an existing prefix: the prefix canonicalizes, the
/// tail appends literally — matching what a later `create_dir_all` +
/// `canonicalize` would produce.
#[ test ]
fn nonexistent_tail_appends_literally()
{
  let tmp = tempfile::TempDir::new().unwrap();
  let canonical_tmp = std::fs::canonicalize( tmp.path() ).unwrap();
  let probe = tmp.path().join( "does-not-exist" ).join( "leaf" );
  assert_eq!( physical_abs( &probe ), canonical_tmp.join( "does-not-exist" ).join( "leaf" ) );
}

/// Fix(BUG-543): a symlinked ancestor of a nonexistent leaf still resolves —
/// the pre-creation probe and the post-creation run must agree on one key.
#[ cfg( unix ) ]
#[ test ]
fn symlinked_ancestor_of_nonexistent_leaf_resolves()
{
  let tmp = tempfile::TempDir::new().unwrap();
  let real = tmp.path().join( "real" );
  std::fs::create_dir( &real ).unwrap();
  let link = tmp.path().join( "link" );
  std::os::unix::fs::symlink( &real, &link ).unwrap();

  let canonical_real = std::fs::canonicalize( &real ).unwrap();
  let probed = physical_abs( &link.join( "-new-topic" ) );
  assert_eq!( probed, canonical_real.join( "-new-topic" ) );
}

/// `.` and `..` components normalize against the canonicalized prefix instead
/// of surviving lexically into the storage key.
#[ test ]
fn dot_and_dotdot_normalize()
{
  let tmp = tempfile::TempDir::new().unwrap();
  let canonical_tmp = std::fs::canonicalize( tmp.path() ).unwrap();
  let sub = tmp.path().join( "sub" );
  std::fs::create_dir( &sub ).unwrap();

  let probe = sub.join( ".." ).join( "." ).join( "leaf" );
  assert_eq!( physical_abs( &probe ), canonical_tmp.join( "leaf" ) );
}

/// A relative path joins against the current working directory before
/// resolution — the result is always absolute.
#[ test ]
fn relative_path_becomes_absolute()
{
  let resolved = physical_abs( Path::new( "some-relative-leaf-xyz" ) );
  assert!( resolved.is_absolute() );
  let cwd = std::fs::canonicalize( std::env::current_dir().unwrap() ).unwrap();
  assert_eq!( resolved, cwd.join( "some-relative-leaf-xyz" ) );
}

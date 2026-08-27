//! Fork-topic name registry: which topic names exist for which base directory.
//!
//! A fork-mode topic's session identity is `UUIDv5( canonical base, name )` — a
//! one-way function, so the name cannot be recovered from the session file that
//! embodies the topic. Listing fork topics by name therefore needs a side channel;
//! this module is that channel.
//!
//! Layout: one plain-text file per base directory under the registry root, named
//! by the base's storage encoding ([`claude_storage_core::encode_path`] — the same
//! flattening Claude's own `projects/` dir uses), holding one topic name per line.
//! Root resolution: `CLR_TOPIC_REGISTRY_DIR` > `~/.clr/topics/`.
//!
//! **The registry is a convenience index, never an authority.** Recording is
//! append-if-missing and warn-never-fatal (a failed write must not break the run
//! that triggered it), and listing tolerates a missing file (no fork topics
//! recorded yet). The authoritative existence signal for a fork topic is its
//! session file; [`crate::enumerate`] combines both.

use std::path::{ Path, PathBuf };

/// Resolve the registry root: `CLR_TOPIC_REGISTRY_DIR` (non-empty) >
/// `~/.clr/topics/` > relative `.clr/topics` when `HOME` is unset.
fn registry_root() -> PathBuf
{
  if let Ok( v ) = std::env::var( "CLR_TOPIC_REGISTRY_DIR" )
  {
    if !v.is_empty() { return PathBuf::from( v ); }
  }
  std::env::var( "HOME" )
    .map_or_else(
      | _ | PathBuf::from( ".clr/topics" ),
      | h | PathBuf::from( h ).join( ".clr" ).join( "topics" ),
    )
}

/// Registry file for one base directory, or `None` when the base cannot be encoded
/// (non-UTF-8 path — the same restriction every storage-key computation already has).
fn registry_file( canonical_base : &Path ) -> Option< PathBuf >
{
  let encoded = claude_storage_core::encode_path( canonical_base ).ok()?;
  Some( registry_root().join( encoded ) )
}

/// Record `topic` as a fork topic of `canonical_base`: append-if-missing.
///
/// Warn-never-fatal: any failure (unencodable base, unwritable root) prints a
/// `[Runner] warning:` and returns — the fork run itself already succeeded or
/// failed on its own terms; a listing-index write must never change that.
///
/// A name containing a newline is refused, because it would corrupt the
/// one-name-per-line format. Such a topic still works as a session; it just
/// cannot be listed by name, and therefore cannot be reached by any command that
/// enumerates topics rather than being handed one.
#[ inline ]
pub fn record( canonical_base : &Path, topic : &str )
{
  if topic.contains( '\n' )
  {
    eprintln!( "[Runner] warning: topic name contains a newline; not recorded in the topics registry" );
    return;
  }
  let Some( file ) = registry_file( canonical_base ) else
  {
    eprintln!
    (
      "[Runner] warning: cannot encode base path for the topics registry: {}",
      canonical_base.display()
    );
    return;
  };
  let existing = std::fs::read_to_string( &file ).unwrap_or_default();
  if existing.lines().any( | line | line == topic )
  {
    return;
  }
  if let Some( parent ) = file.parent()
  {
    if let Err( e ) = std::fs::create_dir_all( parent )
    {
      eprintln!
      (
        "[Runner] warning: cannot create topics registry dir {}: {e}",
        parent.display()
      );
      return;
    }
  }
  use std::io::Write as _;
  let result = std::fs::OpenOptions::new()
    .create( true )
    .append( true )
    .open( &file )
    .and_then( | mut f | writeln!( f, "{topic}" ) );
  if let Err( e ) = result
  {
    eprintln!
    (
      "[Runner] warning: cannot record topic in registry {}: {e}",
      file.display()
    );
  }
}

/// List the fork topic names recorded for `canonical_base`, in first-recorded order.
///
/// Missing or unreadable file (or unencodable base) yields an empty list — no fork
/// topics recorded is an ordinary state, not an error.
#[ inline ]
#[ must_use ]
pub fn list( canonical_base : &Path ) -> Vec< String >
{
  let Some( file ) = registry_file( canonical_base ) else { return Vec::new(); };
  let Ok( content ) = std::fs::read_to_string( &file ) else { return Vec::new(); };
  content.lines()
    .filter( | line | !line.is_empty() )
    .map( str::to_string )
    .collect()
}

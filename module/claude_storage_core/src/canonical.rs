//! Canonical physical path resolution shared by every storage-key computation.
//!
//! Claude Code derives storage names from its physical getcwd, so any path that
//! feeds [`encode_path`][crate::encode_path] — a working directory, a `--from`
//! source, a topic base — must first resolve to the same physical absolute form
//! or the encoded name silently misses the real storage dir (`./src` would
//! encode as `---src`).
//!
//! Extracted from `claude_runner`'s own builder (Fix(BUG-543)) so every consumer
//! of the storage encoding — `clr` and `claude_storage` alike — resolves paths
//! through one shared rule. Two independent resolutions of the same path must
//! never drift apart: the deterministic topic-session identity
//! ([`topic_session_id`][crate::topic_session_id]) hashes this canonical form,
//! so a byte of divergence between consumers would silently split one topic
//! into two unrelated sessions.

use std::path::{ Path, PathBuf };

/// Resolve `raw` to its physical absolute form: `canonicalize` when the path exists,
/// else a cwd-join whose deepest EXISTING prefix is still canonicalized component-wise
/// (Fix(BUG-543)) — only the nonexistent tail is appended literally.
///
/// # Examples
///
/// ```
/// use claude_storage_core::physical_abs;
/// use std::path::Path;
///
/// let abs = physical_abs( Path::new( "/" ) );
/// assert!( abs.is_absolute() );
///
/// // A nonexistent tail under an existing prefix is appended literally.
/// let probed = physical_abs( Path::new( "/no-such-dir-xyz/tail" ) );
/// assert_eq!( probed, Path::new( "/no-such-dir-xyz/tail" ) );
/// ```
#[ inline ]
#[ must_use ]
pub fn physical_abs( raw : &Path ) -> PathBuf
{
  std::fs::canonicalize( raw ).unwrap_or_else( | _ |
  {
    let joined = if raw.is_absolute() { raw.to_path_buf() }
    else
    {
      std::env::current_dir()
        .map_or_else( | _ | raw.to_path_buf(), | cwd | cwd.join( raw ) )
    };
    canonicalize_deepest_prefix( &joined )
  } )
}

/// Fix(BUG-543): component-wise fallback for a path whose leaf does not yet exist
/// (`physical_abs`'s own `canonicalize` already failed, which is expected for any
/// pre-creation probe — an auto-name freshness check, `--dry-run` planning).
///
/// Walks `joined` from the root, re-canonicalizing the accumulated prefix after every
/// component while it exists on disk — resolving any symlinked ancestor along the way —
/// so only the genuinely nonexistent tail is appended literally. `.` is skipped
/// throughout; `..` pops the last pushed component against the accumulated prefix, and
/// a pop that lands back inside existing territory re-canonicalizes there.
///
/// This mirrors what `create_dir_all` + a later `canonicalize` will yield once the
/// nonexistent tail is actually created, since a fresh `mkdir` cannot introduce
/// symlinks of its own — so a pre-creation probe and the real post-creation run agree
/// on the same storage key even when the base path traverses a symlink or carries an
/// unnormalized `..`.
///
/// Root cause: the old fallback (`joined.components().collect()`) was purely lexical —
/// symlinked ancestors and `..` survived verbatim, so pre-creation probes
/// (auto-name freshness checks, dry-run's own target storage) encoded a different
/// storage name than the real, post-`create_dir_all` run used, silently re-opening
/// BUG-542's orphaned-history resume under symlinked/`..` bases.
/// Pitfall: `..` must be popped against the already-canonical prefix (kernel
/// semantics: a symlinked ancestor resolves FIRST, then `..` applies to its target) —
/// never left in the raw lexical text — and the canonicalize re-attempt must not stop
/// permanently at the first miss: a later `..` can pop back into existing space where
/// a symlink still needs resolving for both runs to agree.
fn canonicalize_deepest_prefix( joined : &Path ) -> PathBuf
{
  use std::path::Component;

  let mut canonical = PathBuf::new();

  for component in joined.components()
  {
    match component
    {
      Component::CurDir => {}
      Component::ParentDir => { canonical.pop(); }
      other => canonical.push( other.as_os_str() ),
    }
    if let Ok( resolved ) = std::fs::canonicalize( &canonical )
    {
      canonical = resolved;
    }
  }
  canonical
}

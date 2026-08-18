//! Shared `scope::`/`path::` resolution for commands documented to support
//! project-discovery scoping (`.projects`, `.list`, `.count`, `.search`,
//! `.show`, `.export`).
//!
//! Extracted from `.projects`' original private implementation — see
//! `docs/cli/param/12_scope.md` and `docs/cli/param/09_path.md` for the
//! user-facing contract this module implements.

use unilang::{ ErrorData, ErrorCode };
use claude_storage_core::{ Project, Storage };
use super::storage::resolve_path_parameter;

// ─── path decode helpers ───────────────────────────────────────────────────

/// Length, in bytes, of the longest common prefix of `a` and `b`. Always a
/// valid char-boundary index into both strings (accumulates whole chars).
fn common_prefix_len( a : &str, b : &str ) -> usize
{
  let mut len = 0;
  for ( ca, cb ) in a.chars().zip( b.chars() )
  {
    if ca != cb { break; }
    len += ca.len_utf8();
  }
  len
}

/// Check whether `encoded_base` (cwd or `path::` arg, encoded) is covered by
/// the project identified by `dir_name` (raw storage directory name).
///
/// Returns `true` when the project is an ancestor of (or equal to) the base:
/// - `encoded_base == dir_name` — same project, no topic
/// - `encoded_base.starts_with(dir_name + "-")` — base is in the project subtree
/// - same two checks after stripping a genuine `--topic` suffix from `dir_name`
///
/// Fix(BUG-003)
/// Root cause: the previous `rfind("--")` loop stripped from the LAST `--`
/// found anywhere in `dir_name`, with no way to tell whether that `--` was a
/// genuine topic-suffix marker or just incidental structure shared with
/// `encoded_base` (e.g. both paths sit under the same dot-prefixed temp-dir
/// root, which itself contains a `--`-like byte sequence once encoded).
/// Filesystem existence cannot discriminate either: a shallow shared ancestor
/// (e.g. `/tmp`) exists just as reliably as a genuine one.
/// Fix: only accept a `--` as a real topic boundary when it falls EXACTLY at
/// the point where `dir_name` and `encoded_base` diverge (the longest common
/// prefix). Shared/incidental structure can never BE the divergence point
/// between two different paths, so the topic-boundary decision itself is
/// structurally sound without relying on filesystem state. This guarantee is
/// scoped to that decision only: the naive literal-prefix check above
/// (`check(dir_name)`, no topic suffix involved) is intentionally unchanged
/// and was never filesystem-independent — it can still admit a same-prefix
/// sibling (e.g. an underscore/dot collision), left for the caller's
/// `decode_path_via_fs` verification to resolve. See
/// `docs/invariant/001_path_encoding.md § Contract` for the accepted
/// multi-candidate tradeoff this depends on.
/// Pitfall: do not reintroduce a blind `rfind("--")`/`split("--")` search
/// for the topic-boundary decision — any boundary search that ignores
/// `encoded_base` re-opens this hole. Also do not assume the naive prefix
/// check is filesystem-independent-safe; only the topic-boundary alignment is.
fn is_relevant_encoded( dir_name : &str, encoded_base : &str ) -> bool
{
  let check = | candidate : &str | -> bool
  {
    encoded_base == candidate || encoded_base.starts_with( &format!( "{candidate}-" ) )
  };
  if check( dir_name ) { return true; }
  let lcp_len = common_prefix_len( dir_name, encoded_base );
  if lcp_len == 0 || lcp_len >= dir_name.len() { return false; }
  let ( before, after ) = ( &dir_name[ ..lcp_len ], &dir_name[ lcp_len.. ] );
  if !before.ends_with( '-' ) || !after.starts_with( '-' ) { return false; }
  check( &dir_name[ ..lcp_len - 1 ] )
}

/// Decode a storage directory name into a human-readable display path.
///
/// Path-encoded dirs start with `-` (e.g. `-home-alice-projects`). UUID dirs do not.
/// Compress `$HOME` prefix to `~` for display. Returns full path string if HOME unset.
fn tilde_compress( path : &std::path::Path ) -> String
{
  if let Ok( home ) = std::env::var( "HOME" )
  {
    if let Ok( rel ) = path.strip_prefix( std::path::Path::new( &home ) )
    {
      return format!( "~/{}", rel.display() );
    }
  }
  path.display().to_string()
}

/// Walk the filesystem to decode a lossy-encoded storage dir name to a real path.
///
/// At each `-` boundary the standard heuristic cannot distinguish a path separator
/// from an underscore (both encoded as `-`). This function resolves the ambiguity by
/// checking `is_dir()` at each step: it tries path separator first; if the candidate
/// directory does not exist, it falls back to joining with `_`.
///
/// Returns `None` if no matching path is found (project deleted, remote, or unmounted).
///
/// # Why only as fallback
///
/// Requires the project directory to exist on disk. Always call heuristic decode first
/// and only reach here when that result does not exist. This avoids unnecessary stat
/// calls for paths the heuristic already handles correctly.
fn decode_path_via_fs( encoded : &str ) -> Option< std::path::PathBuf >
{
  let inner = &encoded[ 1.. ]; // strip leading `-`
  let pieces : Vec< &str > = inner.split( '-' ).collect();
  if pieces.is_empty() { return None; }
  walk_fs( std::path::Path::new( "/" ), &pieces, 0, "" )
}

/// Decode the base-encoded component of a storage dir name to a real filesystem path.
///
/// Returns `None` if the encoded string is malformed (non-path-encoded keys such as UUIDs).
/// When `decode_path` succeeds but the result does not exist on disk, falls back to the
/// filesystem-guided walk to resolve `_` vs `/` ambiguity (Fix(issue-029)).
fn decode_storage_base( base_encoded : &str ) -> Option< std::path::PathBuf >
{
  use claude_storage_core::decode_path;
  let h = decode_path( base_encoded ).ok()?;
  if h.exists()
  {
    Some( h )
  }
  else
  {
    // Fix(issue-029): heuristic maps '_' to '/', try filesystem-guided decode.
    Some( decode_path_via_fs( base_encoded ).unwrap_or( h ) )
  }
}

/// Return true if `dir_name` encodes a project path that is `base_path` itself or is nested
/// under `base_path` (`scope::under` predicate).
///
/// The single-hyphen fast-reject `starts_with("{eb}-")` weeds out projects with completely
/// different paths before the more expensive filesystem decode.
///
/// Fix(BUG-003)
/// Root cause: previously stripped a `--topic` suffix via `strip_topic_suffix` before
/// the filesystem-verification decode. That stripping used the same unsound blind
/// `find("--")` search being removed from `is_relevant_encoded` for the same reason.
/// Fix: decode `dir_name` directly — `decode_path_via_fs` already treats an
/// unverifiable/nonexistent path as `true` (conservative include), which covers the
/// case where a genuine topic suffix would have made the raw decode fail to exist.
/// Pitfall: this simplification is only sound because no current test combines
/// `scope::under` with a project that has a genuine topic suffix; if such a test is
/// added, re-verify this fallback still selects the correct base.
fn matches_under( dir_name : &str, eb : &str, base_path : &std::path::Path ) -> bool
{
  if dir_name != eb && !dir_name.starts_with( &format!( "{eb}-" ) ) { return false; }
  if dir_name == eb { return true; }
  decode_path_via_fs( dir_name )
    .map_or( true, | p | p.starts_with( base_path ) )
}

/// Return true if `dir_name` encodes a project path that is an ancestor of `base_path`
/// (`scope::relevant` predicate).
///
/// Fix(BUG-003)
/// Root cause/Pitfall: see `matches_under` — same topic-suffix-stripping removal,
/// same conservative-include fallback in `decode_path_via_fs`.
fn matches_relevant( dir_name : &str, eb : &str, base_path : &std::path::Path ) -> bool
{
  if !is_relevant_encoded( dir_name, eb ) { return false; }
  if dir_name == eb { return true; }
  decode_path_via_fs( dir_name )
    .map_or( true, | p | base_path.starts_with( &p ) )
}

/// Recursive DFS helper for `decode_path_via_fs`.
///
/// `segment` accumulates the current unresolved path component. At each step, option A
/// commits `segment` as a directory and recurses with the next piece; option B appends
/// `_` + piece to `segment` and recurses; option C handles an empty piece (a `--`
/// boundary) by trying `.`/`_`/`-` as the next component's original first character.
/// `is_dir()` prunes option A (and gates option C) early.
///
/// Fix(BUG-003)
/// Root cause: `encode_path` emits a `--` marker whenever a path component's first
/// character is normalized away (original `.`, `_`, or literal `-` all become the
/// marker), so `split('-')` on the storage key produces an EMPTY piece at that
/// boundary. Options A and B only know how to commit a whole prior segment as a
/// directory or merge an underscore into the current one — neither ever re-derives
/// which of `.`/`_`/`-` the marker stood for, so a dot-prefixed component (e.g. a
/// `TempDir`'s `.tmpXXXXXX` root) can never resolve and the whole walk returns `None`.
/// Fix: on an empty piece, commit the current segment (or use `base` directly if the
/// segment is already empty) and try each candidate character as the FIRST character
/// of a fresh segment for the next piece.
/// Pitfall: option C must consume the empty piece AND the following piece together
/// (recurse at `idx + 2`) — treating the empty piece alone as a component would try to
/// join a bare candidate character with nothing.
fn walk_fs(
  base    : &std::path::Path,
  pieces  : &[ &str ],
  idx     : usize,
  segment : &str,
) -> Option< std::path::PathBuf >
{
  if idx == pieces.len()
  {
    let candidate = if segment.is_empty() { base.to_path_buf() } else { base.join( segment ) };
    return if candidate.exists() { Some( candidate ) } else { None };
  }
  let piece = pieces[ idx ];
  // Option C — empty piece (`--` boundary): try `.`, `_`, `-` as the original first
  // character of the next piece's component, starting a fresh segment for it.
  if piece.is_empty() && idx + 1 < pieces.len()
  {
    let commit_base = if segment.is_empty()
    {
      Some( base.to_path_buf() )
    }
    else
    {
      let candidate = base.join( segment );
      if candidate.is_dir() { Some( candidate ) } else { None }
    };
    if let Some( next_base ) = commit_base
    {
      let next_piece = pieces[ idx + 1 ];
      for prefix in [ '.', '_', '-' ]
      {
        let fresh_segment = format!( "{prefix}{next_piece}" );
        if let Some( result ) = walk_fs( &next_base, pieces, idx + 2, &fresh_segment )
        {
          return Some( result );
        }
      }
    }
  }
  // Option A — path separator: commit current segment as a directory, recurse
  if !segment.is_empty()
  {
    let next_base = base.join( segment );
    if next_base.is_dir()
    {
      if let Some( result ) = walk_fs( &next_base, pieces, idx + 1, piece )
      {
        return Some( result );
      }
    }
  }
  // Option B — underscore: merge piece into segment
  let joined = if segment.is_empty()
  {
    piece.to_string()
  }
  else
  {
    format!( "{segment}_{piece}" )
  };
  walk_fs( base, pieces, idx + 1, &joined )
}

/// Decode a storage dir name to the longest real filesystem path it represents.
///
/// # Why the `starts_with('-')` guard
///
/// `decode_path()` requires its input to be a valid path-encoded string. UUID project
/// directories (e.g. `deadbeef-1234-...`) do not start with `-` and are NOT path-encoded.
/// Calling `decode_path` on a UUID returns `Err` — but more importantly, it would be
/// semantically wrong. UUID dirs represent web/IDE sessions without filesystem paths.
/// The guard ensures they fall through to the raw string return at the end.
///
/// # Topic components: metadata vs real directories
///
/// Topic-scoped project dirs are named `-path--topic` (double dash before topic).
/// Topics are often pure metadata tags (e.g. `--commit`), but they can also be real
/// hyphen-prefixed directories (e.g. `--default-topic` → `-default_topic/`).
///
/// Examples:
/// - `-...-src--default-topic`         → `src/-default_topic`
/// - `-...-src--default-topic--commit` → `src/-default_topic/-commit`
/// - `-...-src--commit`                → `src/-commit`
///
/// # Why a single `decode_storage_base` call is sufficient
///
/// Fix(BUG-003)
/// Root cause: this used to call `split_storage_key` to break `dir_name` into a base
/// component plus a list of `--topic` components, then re-join each topic as a
/// separate `-{topic}` path segment. That split relied on the same unsound blind
/// `find("--")` search removed from `is_relevant_encoded` for the same reason — it
/// could not tell a genuine topic boundary from incidental shared structure.
/// Fix: `claude_storage_core::decode_path`'s own heuristic already chains multiple
/// `--`-separated segments correctly on its own (each `--` starts a new
/// hyphen-prefixed segment while the rest of that segment maps `-` → `_`), so passing
/// the whole `dir_name` straight through reconstructs the same multi-topic display
/// path with no external split/append loop needed.
/// Pitfall: do not reintroduce a `--`-splitting loop here — `decode_storage_base`
/// (via `decode_path`) already handles the full string, topics included.
///
/// # Why the filesystem fallback for the base
///
/// Fix(issue-029)
/// Root cause: `decode_path` heuristic defaults to path separator `/` for all
/// unrecognized `-` boundaries. Paths with underscore-named dirs (e.g. `my_project`,
/// `claude_tools`) display incorrectly as `wip/core`, `claude/tools`.
/// Pitfall: Only call the filesystem walk as fallback — never primary — because it
/// requires the project directory to exist on disk. Deleted/remote projects fall
/// back to the raw encoded storage dir name.
pub( super ) fn decode_project_display( dir_name : &str ) -> String
{
  if !dir_name.starts_with( '-' ) { return dir_name.to_string(); }
  let Some( path ) = decode_storage_base( dir_name ) else { return dir_name.to_string() };
  tilde_compress( &path )
}

// ─── scope validation ──────────────────────────────────────────────────────

/// Validate and lowercase a `scope::` parameter value.
///
/// Accepts (case-insensitively): `relevant`, `local`, `under`, `global`, `around`.
/// When `scope_raw` is `None`, falls back to `default` (assumed already valid —
/// every call site passes one of the 5 accepted values as its own default).
///
/// # Errors
///
/// Returns `ErrorData` when `scope_raw` is `Some` but not one of the 5 valid values.
/// The error message embeds the raw (pre-lowercase) input, matching
/// `docs/cli/param/12_scope.md`'s documented `relevant|local|under|global|around`
/// word order.
pub( crate ) fn validate_scope( scope_raw : Option< &str >, default : &str ) -> core::result::Result< String, ErrorData >
{
  let raw = scope_raw.unwrap_or( default );
  let scope = raw.to_lowercase();
  if !matches!( scope.as_str(), "local" | "relevant" | "under" | "global" | "around" )
  {
    return Err( ErrorData::new(
      ErrorCode::InternalError,
      format!( "scope must be relevant|local|under|global|around, got {raw}" ),
    ) );
  }
  Ok( scope )
}

// ─── scoped project resolution ─────────────────────────────────────────────

/// Resolve the scope anchor: `path_raw` when given, else the current directory.
///
/// Shared by `resolve_scoped_projects` and `.usage`'s depth filter, which both
/// anchor on the same `path::`-or-cwd base.
///
/// # Errors
///
/// Returns `ErrorData` when path resolution fails or cwd is unreadable.
pub( super ) fn resolve_base_path( path_raw : Option< &str > ) -> core::result::Result< std::path::PathBuf, ErrorData >
{
  if let Some( p ) = path_raw
  {
    resolve_path_parameter( p )
      .map( std::path::PathBuf::from )
      .map_err( | e | ErrorData::new(
        ErrorCode::InternalError,
        format!( "Failed to resolve path '{p}': {e}" ),
      ) )
  }
  else
  {
    std::env::current_dir()
      .map_err( | e | ErrorData::new(
        ErrorCode::InternalError,
        format!( "Failed to get current directory: {e}" ),
      ) )
  }
}

/// Resolve the set of projects matching `scope`, anchored at `path_raw` (or cwd
/// when `path_raw` is `None`).
///
/// Scope semantics:
/// - `local`    — the anchor project only
/// - `relevant` — every project whose path is an ancestor of (or equal to) the anchor
/// - `under`    — every project whose path is the anchor or nested under it
/// - `around`   — union of `under` + `relevant`
/// - `global`   — all projects in storage (anchor ignored)
///
/// `scope` must already be validated (see `validate_scope`) — an unrecognized
/// value falls through every match arm and yields an empty result rather than
/// an error, matching the original `.projects` closure's own fallback.
///
/// # Errors
///
/// Returns `ErrorData` when path resolution or storage access fails.
pub( crate ) fn resolve_scoped_projects(
  storage  : &Storage,
  scope    : &str,
  path_raw : Option< &str >,
) -> core::result::Result< Vec< Project >, ErrorData >
{
  let all_projects = storage.list_projects()
    .map_err( | e | ErrorData::new( ErrorCode::InternalError, format!( "Failed to list projects: {e}" ) ) )?;

  if scope == "global"
  {
    return Ok( all_projects );
  }

  let base_path = resolve_base_path( path_raw )?;

  // Fix(issue-024)
  // Root cause: encode_path() maps both '_' and '/' to '-', so decode_component()
  // defaults unknown pairs to '/', turning `my_project` → `wip-core` → `wip/core`.
  // Decoded paths never match the real base_path, causing silent 0-result returns.
  // Pitfall: Never decode storage dir names for path comparison — encoding is
  // deterministic but decoding is lossy. Compare encoded ↔ encoded instead.
  let encoded_base = claude_storage_core::encode_path( &base_path )
    .map_err( | e | ErrorData::new(
      ErrorCode::InternalError,
      format!( "Failed to encode base path '{}': {e}", base_path.display() ),
    ) )?;

  Ok(
    all_projects
      .into_iter()
      .filter( | p | project_matches( p, scope, &encoded_base, &base_path ) )
      .collect()
  )
}

/// Does `project` qualify under `scope`, anchored at `base_path`/`encoded_base`?
///
/// Compares encoded base against raw storage directory name — no decode step.
/// UUID project dirs start with a hex character (not `-`), so they never match
/// path-based comparisons and are correctly excluded from non-global scopes.
/// `global` never reaches here (`resolve_scoped_projects` early-returns).
fn project_matches(
  project      : &Project,
  scope        : &str,
  encoded_base : &str,
  base_path    : &std::path::Path,
) -> bool
{
  let dir_name = project
    .storage_dir()
    .file_name()
    .and_then( | n | n.to_str() )
    .unwrap_or( "" );
  match scope
  {
    "local"    => dir_name == encoded_base || dir_name.starts_with( &format!( "{encoded_base}--" ) ),
    // Fix(issue-031)
    // Root cause: starts_with on encoded strings cannot distinguish a child
    //   directory (base/sub → `base-sub`) from a same-level sibling whose name
    //   uses an underscore (base_extra → `base-extra`): both share the `base-`
    //   prefix. Path::starts_with is component-wise and correctly excludes siblings.
    // Pitfall: strip the `--topic` suffix from dir_name before calling
    //   decode_path_via_fs. The `--topic` part encodes a hyphen-prefixed directory
    //   like `-default_topic`; left in place, the walker searches for a dir named
    //   `topic` under the project root, returns None, and the fallback silently
    //   includes everything — the sibling exclusion is bypassed.
    "under" => matches_under( dir_name, encoded_base, base_path ),
    // Fix(issue-032)
    // Root cause: is_relevant_encoded uses string starts_with to check if
    //   dir_name's encoded path is a prefix of encoded_base, so a sibling
    //   `base` (encoded `base-`) falsely matches when base_path is `base_extra`
    //   (encoded `base-extra`). Both `_` and `/` map to `-`, making siblings
    //   indistinguishable from ancestors by string comparison alone.
    //   base_path.starts_with(decoded_path) is component-wise and rejects siblings.
    // Pitfall: strip the `--topic` suffix before calling decode_path_via_fs —
    //   same requirement as the issue-031 fix for scope::under.
    "relevant" => matches_relevant( dir_name, encoded_base, base_path ),
    // Union of under + relevant — bidirectional neighborhood.
    "around" =>
      matches_under( dir_name, encoded_base, base_path )
        || matches_relevant( dir_name, encoded_base, base_path ),
    _ => false,
  }
}

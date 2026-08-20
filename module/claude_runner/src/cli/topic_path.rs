//! Topic directory path computation — the single source of truth for `<base>/-<name>`.
//!
//! Three callers need the identical formula and must never drift apart:
//! `builder.rs::resolve_effective_dir` (where the subprocess actually runs),
//! `topic.rs::disambiguate_slug` (which probes for a free auto-generated name), and
//! `topics.rs` (which lists and resolves topics without running anything). Before this
//! module existed the formula was written out twice, with a comment in `topic.rs`
//! asking the reader to keep it in sync with `builder.rs` by hand.
//!
//! Every function here is pure: no filesystem access, no process state beyond the
//! environment read in `topic_home`. That is what lets `clr topics --path NAME` be a
//! deterministic name-to-path resolver — the same name always yields the same path,
//! whether or not anything exists on disk.

/// Directory name appended to the system temp dir when `CLR_TOPIC_HOME` is unset.
const GLOBAL_TOPIC_DIRNAME : &str = "clr-topic";

/// Prefix every topic directory carries, making topics sort together and match the
/// workspace-wide `-*` convention for generated/ignored directories.
const TOPIC_PREFIX : char = '-';

/// Resolve the global topic home — the base used when `--global` is given.
///
/// `$CLR_TOPIC_HOME` when set to a non-empty value, otherwise
/// `<system temp dir>/clr-topic`. On most systems the temp dir is cleared on reboot;
/// set `CLR_TOPIC_HOME` explicitly for topics that must outlive one.
pub( crate ) fn topic_home() -> std::path::PathBuf
{
  match std::env::var( "CLR_TOPIC_HOME" )
  {
    Ok( v ) if !v.is_empty() => std::path::PathBuf::from( v ),
    _ => std::env::temp_dir().join( GLOBAL_TOPIC_DIRNAME ),
  }
}

/// Resolve the base directory that topic directories are created under.
///
/// Precedence, highest first:
/// 1. `dir` — an explicit `--dir <PATH>`; an explicit path always beats a named default,
///    so `--dir` wins even when `--global` is also given.
/// 2. `global` — `--global`, resolving to [`topic_home`].
/// 3. The current working directory.
pub( crate ) fn topic_base( dir : Option< &str >, global : bool ) -> std::path::PathBuf
{
  if let Some( d ) = dir
  {
    return std::path::PathBuf::from( d );
  }
  if global
  {
    return topic_home();
  }
  std::env::current_dir().unwrap_or_else( | _ | std::path::PathBuf::from( "." ) )
}

/// Join a topic name onto a base directory: `<base>/-<name>`.
///
/// The hyphen prefix is unconditional — it is what distinguishes a topic directory
/// from an ordinary sibling directory, and what [`topic_name_of`] keys on when reading
/// the name back out.
pub( crate ) fn topic_dir( base : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  base.join( format!( "{TOPIC_PREFIX}{name}" ) )
}

/// Recover a topic name from a directory entry name, or `None` when the entry is not a
/// topic directory.
///
/// Exact inverse of [`topic_dir`]'s naming half: strips the leading hyphen. A bare `-`
/// yields `None` rather than an empty topic name, since `topic_dir( base, "" )` would
/// not round-trip to it meaningfully.
pub( crate ) fn topic_name_of( entry_name : &str ) -> Option< &str >
{
  entry_name.strip_prefix( TOPIC_PREFIX ).filter( | n | !n.is_empty() )
}

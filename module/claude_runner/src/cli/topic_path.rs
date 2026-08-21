//! Topic directory path computation — the single source of truth for `<base>/-<name>`.
//!
//! Three callers need the identical formula and must never drift apart:
//! `builder.rs::resolve_effective_dir` (where the subprocess actually runs),
//! `topic.rs::disambiguate_slug` (which probes for a free auto-generated name), and
//! `topics.rs` (which lists and resolves topics without running anything). Before this
//! module existed the formula was written out twice, with a comment in `topic.rs`
//! asking the reader to keep it in sync with `builder.rs` by hand.
//!
//! Every path-computing function here is pure: no filesystem access, no process state
//! beyond the environment read in `topic_home`. That is what lets `clr topics --path NAME`
//! be a deterministic name-to-path resolver — the same name always yields the same path,
//! whether or not anything exists on disk. The one deliberate exception is
//! `effective_topic_mode`, whose legacy-coexistence rule requires a single directory
//! existence probe (rule 4) — mode selection, unlike path resolution, is defined in
//! terms of what already exists.

/// How a `--topic` value maps onto a Claude session.
///
/// `Fork` is the default for a NEW topic: no working directory is created — the
/// topic lives as a deterministically-named session file (`UUIDv5` of canonical
/// base path + topic name, `claude_storage_core::topic_session_id`) inside the
/// base directory's own storage, created by forking the base's most recent
/// session. Staying in the base directory keeps the prompt-cache prefix
/// byte-identical, so the fork reuses the base session's cache instead of
/// re-priming the entire history (the measured cost of a directory change is
/// ~77% of a cold prime; a same-directory fork is ~5%).
///
/// `Dir` is the legacy mechanism: a `<base>/-<name>` working directory plus a
/// physical session-file transplant (`builder.rs::SessionTransplant`).
///
/// Selection between them is `effective_topic_mode`'s job; an explicit
/// `--topic-mode` (or `CLR_TOPIC_MODE`, or json `topic-mode`) overrides it.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub( crate ) enum TopicMode
{
  /// Same-directory session fork named by the deterministic topic UUID.
  Fork,
  /// Legacy `<base>/-<name>` working directory + session transplant.
  Dir,
}

impl core::str::FromStr for TopicMode
{
  type Err = String;
  fn from_str( s : &str ) -> core::result::Result< Self, Self::Err >
  {
    match s
    {
      "fork" => Ok( TopicMode::Fork ),
      "dir"  => Ok( TopicMode::Dir ),
      _ => Err( format!( "invalid topic mode: {s}\nExpected: fork or dir" ) ),
    }
  }
}

/// Decide the effective `TopicMode` for a topic invocation.
///
/// Precedence, highest first:
/// 1. An explicit `explicit` override (`--topic-mode`/`CLR_TOPIC_MODE`/json).
/// 2. `--global` → `Dir` — global topics are shared across arbitrary callers'
///    working directories, so the same-directory cache-identity premise of fork
///    mode never holds for them.
/// 3. A non-empty `--from` → `Dir` — an explicit cross-directory source needs
///    the transplant machinery; a cross-directory prefix can't cache-hit anyway.
/// 4. An existing `<base>/-<name>` directory → `Dir` — a topic created by the
///    legacy mechanism keeps its accumulated directory-based history forever;
///    fork mode silently starting a parallel same-name session would orphan it.
/// 5. Otherwise → `Fork` — the default for every new topic.
pub( crate ) fn effective_topic_mode
(
  explicit : Option< TopicMode >,
  global : bool,
  from : Option< &str >,
  dir : Option< &str >,
  topic : &str,
) -> TopicMode
{
  if let Some( mode ) = explicit
  {
    return mode;
  }
  if global
  {
    return TopicMode::Dir;
  }
  if from.is_some_and( | f | !f.is_empty() )
  {
    return TopicMode::Dir;
  }
  let base = topic_base( dir, false );
  if topic_dir( &base, topic ).exists()
  {
    return TopicMode::Dir;
  }
  TopicMode::Fork
}

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

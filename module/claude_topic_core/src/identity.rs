//! Topic identity: how a `--topic` name maps onto a base directory, a mechanism,
//! and a session.
//!
//! Every path-computing function here is pure — no filesystem access, no process
//! state beyond the environment read in [`topic_home`]. That is what lets a name
//! be resolved to a path deterministically, whether or not anything exists on
//! disk. The one deliberate exception is [`effective_topic_mode`], whose
//! legacy-coexistence rule requires a single directory existence probe (rule 4):
//! mode selection, unlike path resolution, is defined in terms of what exists.
//!
//! Before this crate, the `<base>/-<name>` formula lived in the `clr` binary and
//! three call sites had to be kept in sync by a comment asking the reader not to
//! let them drift. The formula is now behind a crate boundary instead.

use std::path::{ Path, PathBuf };

/// Directory name appended to the system temp dir when `CLR_TOPIC_HOME` is unset.
const GLOBAL_TOPIC_DIRNAME : &str = "clr-topic";

/// Prefix every topic directory carries, making topics sort together and match the
/// workspace-wide `-*` convention for generated/ignored directories.
const TOPIC_PREFIX : char = '-';

/// How a `--topic` value maps onto a Claude session.
///
/// [`TopicMode::Fork`] is the default for a NEW topic: no working directory is
/// created — the topic lives as a deterministically-named session file (`UUIDv5`
/// of canonical base path + topic name, [`claude_storage_core::topic_session_id`])
/// inside the base directory's own storage, created by forking the base's most
/// recent session. Staying in the base directory keeps the prompt-cache prefix
/// byte-identical, so the fork reuses the base session's cache instead of
/// re-priming the entire history (the measured cost of a directory change is ~77%
/// of a cold prime; a same-directory fork is ~5%).
///
/// [`TopicMode::Dir`] is the legacy mechanism: a `<base>/-<name>` working
/// directory plus a physical session-file transplant.
///
/// Selection between them is [`effective_topic_mode`]'s job; an explicit
/// `--topic-mode` (or `CLR_TOPIC_MODE`, or json `topic-mode`) overrides it.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash ) ]
pub enum TopicMode
{
  /// Same-directory session fork named by the deterministic topic UUID.
  Fork,
  /// Legacy `<base>/-<name>` working directory + session transplant.
  Dir,
}

impl TopicMode
{
  /// The lowercase wire name — `"fork"` or `"dir"` — as it appears in
  /// `--topic-mode`, `CLR_TOPIC_MODE`, json config, and listing output.
  ///
  /// Exact inverse of the [`core::str::FromStr`] impl, so a mode that round-trips
  /// through a command line comes back unchanged.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Fork => "fork",
      Self::Dir  => "dir",
    }
  }
}

impl core::fmt::Display for TopicMode
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    f.write_str( self.as_str() )
  }
}

impl core::str::FromStr for TopicMode
{
  type Err = String;

  #[ inline ]
  fn from_str( s : &str ) -> core::result::Result< Self, Self::Err >
  {
    match s
    {
      "fork" => Ok( Self::Fork ),
      "dir"  => Ok( Self::Dir ),
      _ => Err( format!( "invalid topic mode: {s}\nExpected: fork or dir" ) ),
    }
  }
}

/// Decide the effective [`TopicMode`] for a topic invocation.
///
/// Precedence, highest first:
/// 1. An explicit `explicit` override (`--topic-mode`/`CLR_TOPIC_MODE`/json).
/// 2. `global` → [`TopicMode::Dir`] — global topics are shared across arbitrary
///    callers' working directories, so the same-directory cache-identity premise
///    of fork mode never holds for them.
/// 3. A non-empty `from` → [`TopicMode::Dir`] — an explicit cross-directory source
///    needs the transplant machinery; a cross-directory prefix can't cache-hit anyway.
/// 4. An existing `<base>/-<name>` directory → [`TopicMode::Dir`] — a topic created
///    by the legacy mechanism keeps its accumulated directory-based history forever;
///    fork mode silently starting a parallel same-name session would orphan it.
/// 5. Otherwise → [`TopicMode::Fork`] — the default for every new topic.
#[ inline ]
#[ must_use ]
pub fn effective_topic_mode
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

/// Resolve the global topic home — the base used when `--global` is given.
///
/// `$CLR_TOPIC_HOME` when set to a non-empty value, otherwise
/// `<system temp dir>/clr-topic`. On most systems the temp dir is cleared on
/// reboot; set `CLR_TOPIC_HOME` explicitly for topics that must outlive one.
#[ inline ]
#[ must_use ]
pub fn topic_home() -> PathBuf
{
  match std::env::var( "CLR_TOPIC_HOME" )
  {
    Ok( v ) if !v.is_empty() => PathBuf::from( v ),
    _ => std::env::temp_dir().join( GLOBAL_TOPIC_DIRNAME ),
  }
}

/// Resolve the base directory that topic directories are created under.
///
/// Precedence, highest first:
/// 1. `dir` — an explicit `--dir <PATH>`; an explicit path always beats a named
///    default, so `--dir` wins even when `global` is also set.
/// 2. `global` — resolving to [`topic_home`].
/// 3. The current working directory.
#[ inline ]
#[ must_use ]
pub fn topic_base( dir : Option< &str >, global : bool ) -> PathBuf
{
  if let Some( d ) = dir
  {
    return PathBuf::from( d );
  }
  if global
  {
    return topic_home();
  }
  std::env::current_dir().unwrap_or_else( | _ | PathBuf::from( "." ) )
}

/// Join a topic name onto a base directory: `<base>/-<name>`.
///
/// The hyphen prefix is unconditional — it is what distinguishes a topic directory
/// from an ordinary sibling directory, and what [`topic_name_of`] keys on when
/// reading the name back out.
#[ inline ]
#[ must_use ]
pub fn topic_dir( base : &Path, name : &str ) -> PathBuf
{
  base.join( format!( "{TOPIC_PREFIX}{name}" ) )
}

/// Recover a topic name from a directory entry name, or `None` when the entry is
/// not a topic directory.
///
/// Exact inverse of [`topic_dir`]'s naming half: strips the leading hyphen. A bare
/// `-` yields `None` rather than an empty topic name, since `topic_dir( base, "" )`
/// would not round-trip to it meaningfully.
#[ inline ]
#[ must_use ]
pub fn topic_name_of( entry_name : &str ) -> Option< &str >
{
  entry_name.strip_prefix( TOPIC_PREFIX ).filter( | n | !n.is_empty() )
}

/// The session file a fork-mode topic of `name` occupies under `base`.
///
/// `base` is canonicalised here via [`claude_storage_core::physical_abs`], so a
/// symlinked or `..`-carrying base resolves to the same identity Claude Code
/// itself would derive. `None` only when the storage root cannot be resolved
/// (no `HOME`, no `CLAUDE_HOME`) or the path is not UTF-8.
///
/// Pure: the file need not exist.
#[ inline ]
#[ must_use ]
pub fn fork_session_file( base : &Path, name : &str ) -> Option< PathBuf >
{
  claude_storage_core::topic_session_file( &claude_storage_core::physical_abs( base ), name )
}

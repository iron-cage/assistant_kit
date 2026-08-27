//! Discovering which topics exist under a base directory.
//!
//! Two disjoint populations have to be merged, because the two mechanisms leave
//! completely different traces:
//!
//! - **Dir mode** leaves a `<base>/-<name>` directory, found by scanning the base.
//! - **Fork mode** leaves nothing in the base at all — only a `UUIDv5`-named
//!   session file in the base's own storage, whose name is unrecoverable from the
//!   file. Those come from [`crate::registry`].
//!
//! The same name can legitimately exist once per mode, so the unit of enumeration
//! is `( name, mode )` and never the name alone. A caller that collapses the two
//! into one row will address one topic and silently miss the other — and, because
//! [`crate::identity::effective_topic_mode`]'s rule 4 lets an existing directory
//! outrank fork mode, the one it misses is always the fork.
//!
//! # Why `sessions == 0` is worth filtering on
//!
//! [`enumerate`] reports everything it finds. [`enumerate_live`] keeps only topics
//! that hold at least one session, and that filter does two jobs at once:
//!
//! 1. **It is the difference between continuing a conversation and starting one.**
//!    A registry entry whose session file was deleted, or a `-name/` directory
//!    never entered, has no conversation to continue — addressing it *creates* one
//!    by forking the base. For a command that fans a prompt out over "my topics",
//!    silently minting new conversations is the wrong reading of the request.
//! 2. **It keeps fan-out out of non-topic directories.** [`crate::identity::topic_name_of`]
//!    accepts any `-`-prefixed directory name, and this workspace's own convention
//!    marks generated/ignored directories the same way — `-daemon/`, `-gate/`, and
//!    every `./-NNNN_*` scratch directory look exactly like dir-mode topics from
//!    the base's point of view. They have no session storage, so this filter drops
//!    them. Treat that as a strong heuristic and not a guarantee: a scratch
//!    directory someone did once run `claude` inside genuinely does have storage,
//!    and genuinely will be enumerated.

use std::path::{ Path, PathBuf };

use crate::identity::{ fork_session_file, topic_name_of, TopicMode };

/// One topic found under a base directory.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub struct Topic
{
  /// Topic name as `--topic` takes it — no `-` prefix, no path separators.
  pub name : String,
  /// Which mechanism this topic uses. Addressing it requires passing this back
  /// as `--topic-mode`; see the module docs.
  pub mode : TopicMode,
  /// Dir mode: the `<base>/-<name>` working directory. Fork mode: the `UUIDv5`
  /// session file. Both are computed, so both are meaningful before they exist.
  pub path : PathBuf,
  /// How many sessions this topic holds. Dir mode counts `*.jsonl` in the
  /// directory's own storage; fork mode is 0 or 1, since a fork topic *is* one
  /// session file.
  pub sessions : usize,
}

impl Topic
{
  /// The deterministic conversation id backing a fork topic, or `None` for a dir
  /// topic (whose sessions are ordinary Claude-generated ids, not derived from
  /// the name).
  ///
  /// This is the id `claude --resume` takes, and the key
  /// [`crate::lock`] and [`crate::select`] use to recognise the topic in a live
  /// process's argv.
  #[ inline ]
  #[ must_use ]
  pub fn session_id( &self ) -> Option< String >
  {
    match self.mode
    {
      TopicMode::Fork => self.path.file_stem()?.to_str().map( str::to_owned ),
      TopicMode::Dir  => None,
    }
  }
}

/// Count `*.jsonl` session files in `dir`'s own Claude Code session storage.
///
/// Returns 0 for a topic directory that exists but has never been entered — the
/// session directory is created by Claude Code on first run, not by whoever made
/// the topic directory.
#[ inline ]
#[ must_use ]
pub fn session_count( dir : &Path ) -> usize
{
  let scope = claude_storage_core::scope_for( dir );
  let Ok( entries ) = std::fs::read_dir( &scope.claude_session_dir ) else { return 0; };
  entries
    .filter_map( Result::ok )
    .filter( | e | e.path().extension().is_some_and( | x | x == "jsonl" ) )
    .count()
}

/// Every dir-mode topic directly under `base`, unsorted.
///
/// A non-existent or unreadable base yields an empty list rather than an error:
/// the global topic home legitimately does not exist until the first global topic
/// is created.
fn collect_dir_topics( base : &Path ) -> Vec< Topic >
{
  let Ok( entries ) = std::fs::read_dir( base ) else { return Vec::new(); };
  entries
    .filter_map( Result::ok )
    .filter( | e | e.path().is_dir() )
    .filter_map( | e |
    {
      let file_name = e.file_name();
      let name = topic_name_of( file_name.to_str()? )?.to_owned();
      let path = e.path();
      let sessions = session_count( &path );
      Some( Topic { name, mode : TopicMode::Dir, path, sessions } )
    } )
    .collect()
}

/// Every fork-mode topic recorded for `base` in the registry, unsorted.
///
/// Path and existence are resolved through the shared `UUIDv5` rule; the registry
/// contributes only the names. `sessions` is 1 when the session file exists
/// non-empty, 0 otherwise — a registry entry whose file was deleted stays listed
/// with 0, because its name is still reserved for auto-naming purposes even
/// though there is no conversation behind it.
fn collect_fork_topics( base : &Path ) -> Vec< Topic >
{
  let canonical_base = claude_storage_core::physical_abs( base );
  crate::registry::list( &canonical_base )
    .into_iter()
    .filter_map( | name |
    {
      let path = fork_session_file( base, &name )?;
      let sessions = usize::from(
        std::fs::metadata( &path ).is_ok_and( | meta | meta.len() > 0 ) );
      Some( Topic { name, mode : TopicMode::Fork, path, sessions } )
    } )
    .collect()
}

/// Every topic under `base`, both mechanisms merged, sorted by name then mode.
///
/// Includes topics with no sessions. For a caller that intends to *address* the
/// results rather than list them, [`enumerate_live`] is almost always the one
/// wanted — see the module docs.
#[ inline ]
#[ must_use ]
pub fn enumerate( base : &Path ) -> Vec< Topic >
{
  let mut topics = collect_dir_topics( base );
  topics.extend( collect_fork_topics( base ) );
  topics.sort_by( | a, b | a.name.cmp( &b.name ).then( a.mode.as_str().cmp( b.mode.as_str() ) ) );
  topics
}

/// Every topic under `base` that holds at least one session, sorted by name then mode.
///
/// The addressable subset: each of these has a conversation that a forwarded
/// prompt continues, rather than a name that a forwarded prompt would bring into
/// existence.
#[ inline ]
#[ must_use ]
pub fn enumerate_live( base : &Path ) -> Vec< Topic >
{
  let mut topics = enumerate( base );
  topics.retain( | t | t.sessions > 0 );
  topics
}

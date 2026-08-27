#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]

//! # `claude_topic_core` — what topics exist, and which one to use
//!
//! A *topic* is a named, isolated Claude Code conversation belonging to a base
//! directory. This crate owns the four questions that answer requires, and
//! nothing else:
//!
//! - [`identity`] — what a name resolves to: a base, a mechanism, a session.
//! - [`enumerate`] — which topics exist under a base, both mechanisms merged.
//! - [`select`] — which one to hand a prompt to.
//! - [`pool`] — what to call topics that exist only to be somewhere to work.
//! - [`lock`] — keeping two writers off one conversation.
//!
//! It runs nothing. Creating or continuing a topic means invoking Claude Code,
//! which belongs to the layer above; everything here is computation over paths,
//! a registry file, and a process list.
//!
//! ## The one thing to know before using it
//!
//! **A topic is a `( name, mode )` pair, never a name.** The two mechanisms —
//! [`identity::TopicMode::Fork`] and [`identity::TopicMode::Dir`] — leave
//! different traces, can hold the same name simultaneously, and are not
//! interchangeable. [`identity::effective_topic_mode`]'s rule 4 gives an existing
//! directory priority, so a caller that addresses a topic by name alone reaches
//! the dir-mode one and silently never reaches its fork-mode twin.
//! [`enumerate::Topic`] carries the mode for exactly this reason, and it has to be
//! passed back as `--topic-mode` when the topic is addressed.
//!
//! ## Two things that look like authorities and are not
//!
//! 1. **The registry.** Fork topics are named by `UUIDv5( canonical base, name )`,
//!    which is one-way, so [`registry`] exists to remember the names. It is an
//!    index: entries outlive the sessions they name, and a name containing a
//!    newline is never recorded at all. The session file is the authority.
//! 2. **The `-` prefix.** [`identity::topic_name_of`] accepts any `-`-prefixed
//!    directory, and this workspace marks generated directories the same way.
//!    [`enumerate::enumerate_live`] is what separates the two in practice, and it
//!    does so by looking for sessions rather than by pattern-matching names.

#![ deny( missing_docs ) ]
#![ warn( rust_2018_idioms ) ]

pub mod enumerate;
pub mod identity;
pub mod lock;
pub mod pool;
pub mod registry;
pub mod select;

pub use enumerate::{ enumerate, enumerate_live, session_count, Topic };
pub use identity::
{
  effective_topic_mode,
  fork_session_file,
  topic_base,
  topic_dir,
  topic_home,
  topic_name_of,
  TopicMode,
};
pub use lock::
{
  enabled_for_run_path,
  lock_file,
  try_lock,
  LockDenied,
  TopicLock,
  LOCK_DIR_ENV,
  LOCK_ENV,
};
pub use pool::{ missing_names, pool_index, validate_prefix, DEFAULT_PREFIX };
pub use select::{ default_seed, is_busy, select, select_with, Pick, Selection };

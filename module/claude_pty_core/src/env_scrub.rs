//! Environment scrubbing for spawned PTY children.
//!
//! Two distinct hazards, deliberately kept as two lists rather than one:
//!
//! 1. **Terminal identity** — variables naming the *host's* terminal multiplexer
//!    session. A spawned agent that shells out to `tmux send-keys` or
//!    `wezterm cli` while `TMUX`/`WEZTERM_UNIX_SOCKET` is still inherited acts on
//!    the operator's own session, not its own. This is a containment failure, not
//!    a cosmetic one.
//! 2. **Claude Code markers** — variables Claude Code sets for its own children.
//!    Inheriting `CLAUDE_CODE_CHILD_SESSION` suppresses both session-registry
//!    registration and transcript persistence, so a child spawned from inside a
//!    Claude session silently becomes unobservable.

/// Environment variables that name the host's terminal or multiplexer session.
///
/// Removed from every spawned child. A child that keeps these can drive the
/// operator's own multiplexer pane instead of its own PTY.
pub const TERMINAL_IDENTITY_VARS : &[ &str ] =
&[
  "COLORTERM",
  "ITERM_SESSION_ID",
  "TERM_PROGRAM",
  "TERM_PROGRAM_VERSION",
  "TMUX",
  "TMUX_PANE",
  "WEZTERM_PANE",
  "WEZTERM_UNIX_SOCKET",
];

/// Prefix of the Claude Code marker variables removed from every spawned child.
///
/// A prefix rather than an explicit list: Claude Code adds `CLAUDE_*` variables
/// across releases, and an allowlist would silently go stale. The caller sets
/// back whichever it deliberately wants (see `SessionConfig::env`).
pub const CLAUDE_MARKER_PREFIX : &str = "CLAUDE_";

/// The `TERM` value advertised to spawned children.
///
/// Fixed rather than inherited: the child must be told the capabilities of the
/// PTY it is actually attached to, not those of whatever terminal happens to be
/// hosting the parent.
pub const CHILD_TERM : &str = "xterm-256color";

/// Return the names to remove from an environment map before spawning a child.
///
/// Combines [`TERMINAL_IDENTITY_VARS`] with every `CLAUDE_`-prefixed name found
/// in `source`. `source` is the iterator of the environment the child would
/// otherwise inherit — typically `std::env::vars()`.
#[ inline ]
pub fn scrub_list< 'a, I >( source : I ) -> Vec< String >
where
  I : IntoIterator< Item = &'a str >,
{
  let mut names : Vec< String > = TERMINAL_IDENTITY_VARS.iter().map( | v | ( *v ).to_string() ).collect();
  for name in source
  {
    if name.starts_with( CLAUDE_MARKER_PREFIX )
    {
      names.push( name.to_string() );
    }
  }
  names.sort();
  names.dedup();
  names
}

/// Return `true` when `name` must be removed from a spawned child's environment.
#[ inline ]
#[ must_use ]
pub fn is_scrubbed( name : &str ) -> bool
{
  name.starts_with( CLAUDE_MARKER_PREFIX ) || TERMINAL_IDENTITY_VARS.contains( &name )
}

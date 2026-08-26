//! Environment-scrubbing rules applied to a spawned child.
//!
//! Pure-function tests over the scrub list. The end-to-end effect on a real
//! child is covered by `session_test.rs` (sess07, sess08, sess09).
//!
//! ## Specification References
//!
//! - `docs/feature/002_session_spawn.md` — the scrubbing contract
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | env01 | Every terminal identity var | `is_scrubbed` is true |
//! | env02 | A `CLAUDE_`-prefixed name | `is_scrubbed` is true |
//! | env03 | An ordinary name (`PATH`, `HOME`) | `is_scrubbed` is false |
//! | env04 | `TERM` itself | Not scrubbed — it is replaced, not removed |
//! | env05 | `scrub_list` on a mixed input | Terminal vars plus the `CLAUDE_` ones |
//! | env06 | `scrub_list` output is sorted and deduplicated | No repeats, ascending |
//! | env07 | `scrub_list` on empty input | Still lists the terminal vars |
//! | env08 | `CHILD_TERM` value | `xterm-256color` |

use claude_pty_core::env_scrub::{
  is_scrubbed, scrub_list, CHILD_TERM, CLAUDE_MARKER_PREFIX, TERMINAL_IDENTITY_VARS,
};

/// env01: every enumerated terminal-identity variable is scrubbed.
///
/// These describe the *parent's* terminal emulator, not the pty. A child that
/// finds `TMUX` set will address escape sequences to a multiplexer that is not
/// there — which is why the list is broader than the `CLAUDE_` prefix alone.
#[ test ]
fn env01_terminal_identity_vars_are_scrubbed()
{
  assert!( !TERMINAL_IDENTITY_VARS.is_empty(), "the terminal identity list is empty" );

  for name in TERMINAL_IDENTITY_VARS
  {
    assert!( is_scrubbed( name ), "{name} is in the list but is_scrubbed says otherwise" );
  }
}

/// env02: the prefix rule catches names not enumerated anywhere.
#[ test ]
fn env02_claude_prefixed_names_are_scrubbed()
{
  for name in [ "CLAUDE_CODE_ENTRYPOINT", "CLAUDE_SESSION_ID", "CLAUDE_ANYTHING_AT_ALL" ]
  {
    assert!( name.starts_with( CLAUDE_MARKER_PREFIX ), "test fixture {name} lacks the prefix" );
    assert!( is_scrubbed( name ), "{name} was not scrubbed despite the CLAUDE_ prefix" );
  }
}

/// env03: ordinary variables pass through.
///
/// Scrubbing too broadly is its own failure — a child without `PATH` cannot
/// resolve a program name at all.
#[ test ]
fn env03_ordinary_names_pass_through()
{
  for name in [ "PATH", "HOME", "USER", "LANG", "SHELL", "PWD" ]
  {
    assert!( !is_scrubbed( name ), "{name} was scrubbed but must reach the child" );
  }
}

/// env04: `TERM` is replaced, not removed.
///
/// Removing it would leave the child with no terminal description at all, which
/// is worse than an inaccurate one. `SessionConfig` sets it to [`CHILD_TERM`]
/// after scrubbing.
#[ test ]
fn env04_term_is_not_in_the_scrub_list()
{
  assert!(
    !is_scrubbed( "TERM" ),
    "TERM is scrubbed — it must be replaced with CHILD_TERM, not removed",
  );
}

/// env05: `scrub_list` combines the enumerated names with the prefixed ones found.
#[ test ]
fn env05_scrub_list_combines_both_rules()
{
  let source = [ "PATH", "CLAUDE_CODE_ENTRYPOINT", "HOME", "TMUX", "CLAUDE_SESSION_ID" ];
  let list = scrub_list( source );

  assert!( list.contains( &"CLAUDE_CODE_ENTRYPOINT".to_string() ), "prefixed name missing: {list:?}" );
  assert!( list.contains( &"CLAUDE_SESSION_ID".to_string() ), "prefixed name missing: {list:?}" );
  assert!( list.contains( &"TMUX".to_string() ), "terminal identity var missing: {list:?}" );
  assert!( !list.contains( &"PATH".to_string() ), "PATH must not be scrubbed: {list:?}" );
  assert!( !list.contains( &"HOME".to_string() ), "HOME must not be scrubbed: {list:?}" );
}

/// env06: the result is sorted and free of duplicates.
///
/// A name present in both rules — a terminal var that also appears in the source
/// — must be listed once. `Command::env_remove` tolerates repeats, but a list
/// that silently grows with duplicates hides how large it actually is.
#[ test ]
fn env06_scrub_list_is_sorted_and_deduplicated()
{
  // `TMUX` is in TERMINAL_IDENTITY_VARS and also present in the source.
  let source = [ "TMUX", "CLAUDE_A", "CLAUDE_A", "TMUX" ];
  let list = scrub_list( source );

  let mut sorted = list.clone();
  sorted.sort();
  assert_eq!( list, sorted, "scrub_list output is not sorted: {list:?}" );

  let mut deduped = list.clone();
  deduped.dedup();
  assert_eq!( list, deduped, "scrub_list output contains duplicates: {list:?}" );
}

/// env07: with nothing in the source, the enumerated names still apply.
#[ test ]
fn env07_scrub_list_of_empty_source_keeps_terminal_vars()
{
  let list = scrub_list( core::iter::empty::< &str >() );

  assert_eq!(
    list.len(),
    TERMINAL_IDENTITY_VARS.len(),
    "empty source should yield exactly the terminal identity vars, got {list:?}",
  );
  for name in TERMINAL_IDENTITY_VARS
  {
    assert!( list.contains( &( *name ).to_string() ), "{name} missing from {list:?}" );
  }
}

/// env08: the terminal description handed to the child.
#[ test ]
fn env08_child_term_is_xterm_256color()
{
  assert_eq!(
    CHILD_TERM, "xterm-256color",
    "CHILD_TERM changed — update docs/feature/002_session_spawn.md",
  );
}

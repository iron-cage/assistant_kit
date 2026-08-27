//! Identity tests — what a topic name resolves to, and which mechanism answers.
//!
//! Almost everything here is pure path arithmetic, so the assertions are exact
//! rather than "contains". The one function that touches the filesystem is
//! `effective_topic_mode`, and it does so for exactly one reason: rule 4 asks
//! whether a legacy directory already exists. Those cases build the directory for
//! real rather than faking the probe.
//!
//! The cases worth having are the precedence ones. `effective_topic_mode` has five
//! rules and four of them are overrides, so the way to get it wrong is not to
//! mis-implement a rule but to let a lower one win — tid13 in particular: an
//! explicit `fork` has to beat an existing directory, or a topic that exists in
//! both mechanisms is permanently unreachable in the one the user asked for.
//!
//! ## Specification References
//!
//! - `docs/feature/001_topic_identity.md` — the two mechanisms and the precedence
//! - `docs/invariant/002_mode_travels_with_name.md` — why rule 4 makes the mode
//!   part of the address rather than a default
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | tid01 | `TopicMode` through `as_str` and `FromStr` | Round-trips both ways |
//! | tid02 | An unrecognised mode string | Error naming both modes |
//! | tid03 | `topic_dir` | `<base>/-<name>` |
//! | tid04 | `topic_name_of` on `topic_dir`'s output | The original name |
//! | tid05 | A bare `-` directory | Not a topic name |
//! | tid06 | A directory with no `-` prefix | Not a topic name |
//! | tid07 | Explicit mode against every other signal | Explicit wins |
//! | tid08 | `--global` | Dir |
//! | tid09 | Non-empty `--from` | Dir |
//! | tid10 | Empty `--from` | Not treated as a signal |
//! | tid11 | An existing `<base>/-<name>` directory | Dir |
//! | tid12 | No signal at all | Fork |
//! | tid13 | Explicit `fork` over an existing directory | Fork — rule 1 beats rule 4 |
//! | tid14 | `--dir` and `--global` together | `--dir` wins |
//! | tid15 | `CLR_TOPIC_HOME` set | Used verbatim as the global base |
//! | tid16 | `fork_session_file` | Deterministic, in the base's own storage |

use std::path::Path;
use core::str::FromStr as _;

use claude_topic_core::
{
  effective_topic_mode,
  fork_session_file,
  topic_base,
  topic_dir,
  topic_home,
  topic_name_of,
  TopicMode,
};
use tempfile::TempDir;

/// Serializes the tests that mutate process-wide env vars.
///
/// `std::env::set_var` / `remove_var` are not thread-safe across concurrent tests.
/// Every test that calls either must hold this lock for its whole body.
static ENV_LOCK : std::sync::Mutex< () > = std::sync::Mutex::new( () );

/// tid01: the wire name and the parser are exact inverses.
#[ test ]
fn tid01_mode_round_trips_through_its_wire_name()
{
  for mode in [ TopicMode::Fork, TopicMode::Dir ]
  {
    assert_eq!( TopicMode::from_str( mode.as_str() ).unwrap(), mode );
    assert_eq!( mode.to_string(), mode.as_str() );
  }
}

/// tid02: an unrecognised mode says what the alternatives are.
#[ test ]
fn tid02_unknown_mode_names_both_alternatives()
{
  let err = TopicMode::from_str( "forked" ).unwrap_err();
  assert!( err.contains( "fork" ), "error must name fork: {err}" );
  assert!( err.contains( "dir" ), "error must name dir: {err}" );
}

/// tid03: a topic directory is the base plus a hyphenated name.
#[ test ]
fn tid03_topic_dir_prefixes_the_name_with_a_hyphen()
{
  assert_eq!( topic_dir( Path::new( "/base" ), "review" ), Path::new( "/base/-review" ) );
}

/// tid04: reading the name back out inverts writing it in.
#[ test ]
fn tid04_topic_name_of_inverts_topic_dir()
{
  let dir = topic_dir( Path::new( "/base" ), "review" );
  let entry = dir.file_name().unwrap().to_str().unwrap();
  assert_eq!( topic_name_of( entry ), Some( "review" ) );
}

/// tid05: a bare hyphen is a directory, not a topic with an empty name.
#[ test ]
fn tid05_bare_hyphen_is_not_a_topic_name()
{
  assert_eq!( topic_name_of( "-" ), None );
}

/// tid06: an ordinary sibling directory is not a topic.
#[ test ]
fn tid06_unprefixed_entry_is_not_a_topic_name()
{
  assert_eq!( topic_name_of( "review" ), None );
}

/// tid07: an explicit mode outranks every other signal at once.
#[ test ]
fn tid07_explicit_mode_outranks_every_other_signal()
{
  let base = TempDir::new().unwrap();
  std::fs::create_dir( topic_dir( base.path(), "review" ) ).unwrap();
  let dir = base.path().to_str().unwrap();

  let mode = effective_topic_mode
  (
    Some( TopicMode::Fork ),
    true,
    Some( "/elsewhere" ),
    Some( dir ),
    "review",
  );
  assert_eq!( mode, TopicMode::Fork );
}

/// tid08: a global topic is shared across callers' directories, so fork mode's
/// same-directory premise never holds for it.
#[ test ]
fn tid08_global_selects_dir_mode()
{
  let base = TempDir::new().unwrap();
  let dir = base.path().to_str().unwrap();
  assert_eq!( effective_topic_mode( None, true, None, Some( dir ), "review" ), TopicMode::Dir );
}

/// tid09: an explicit cross-directory source needs the transplant machinery.
#[ test ]
fn tid09_explicit_from_selects_dir_mode()
{
  let base = TempDir::new().unwrap();
  let dir = base.path().to_str().unwrap();
  let mode = effective_topic_mode( None, false, Some( "/elsewhere" ), Some( dir ), "review" );
  assert_eq!( mode, TopicMode::Dir );
}

/// tid10: an empty `--from` is an absent one — the check is emptiness, not
/// `Some`-ness, because the CLI layer supplies `Some( "" )` for an unset flag.
#[ test ]
fn tid10_empty_from_is_not_a_signal()
{
  let base = TempDir::new().unwrap();
  let dir = base.path().to_str().unwrap();
  assert_eq!( effective_topic_mode( None, false, Some( "" ), Some( dir ), "review" ), TopicMode::Fork );
}

/// tid11: a topic created by the legacy mechanism keeps it — fork mode starting a
/// parallel same-name session would orphan the accumulated history.
#[ test ]
fn tid11_existing_directory_selects_dir_mode()
{
  let base = TempDir::new().unwrap();
  std::fs::create_dir( topic_dir( base.path(), "review" ) ).unwrap();
  let dir = base.path().to_str().unwrap();
  assert_eq!( effective_topic_mode( None, false, None, Some( dir ), "review" ), TopicMode::Dir );
}

/// tid12: with nothing pointing elsewhere, a new topic is a fork.
#[ test ]
fn tid12_no_signal_selects_fork_mode()
{
  let base = TempDir::new().unwrap();
  let dir = base.path().to_str().unwrap();
  assert_eq!( effective_topic_mode( None, false, None, Some( dir ), "review" ), TopicMode::Fork );
}

/// tid13: rule 1 beats rule 4. Without this, a name held in both mechanisms is
/// permanently unreachable in fork mode.
#[ test ]
fn tid13_explicit_fork_reaches_past_an_existing_directory()
{
  let base = TempDir::new().unwrap();
  std::fs::create_dir( topic_dir( base.path(), "review" ) ).unwrap();
  let dir = base.path().to_str().unwrap();
  let mode = effective_topic_mode( Some( TopicMode::Fork ), false, None, Some( dir ), "review" );
  assert_eq!( mode, TopicMode::Fork );
}

/// tid14: an explicit path beats a named default.
#[ test ]
fn tid14_explicit_dir_outranks_global()
{
  assert_eq!( topic_base( Some( "/explicit" ), true ), Path::new( "/explicit" ) );
}

/// tid15: `CLR_TOPIC_HOME` is used verbatim — no `.clr` or `clr-topic` appended,
/// so a caller who sets it knows exactly where topics land.
#[ test ]
fn tid15_global_base_honours_the_topic_home_override()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let home = TempDir::new().unwrap();
  std::env::set_var( "CLR_TOPIC_HOME", home.path() );

  assert_eq!( topic_home(), home.path() );
  assert_eq!( topic_base( None, true ), home.path() );

  std::env::remove_var( "CLR_TOPIC_HOME" );
}

/// tid16: a fork topic's session file is deterministic and lives in the base's own
/// storage — that co-location is what keeps the prompt-cache prefix identical.
#[ test ]
fn tid16_fork_session_file_lands_in_the_bases_own_storage()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let home = TempDir::new().unwrap();
  let base = TempDir::new().unwrap();
  std::env::remove_var( "CLAUDE_HOME" );
  std::env::set_var( "HOME", home.path() );

  let first = fork_session_file( base.path(), "review" ).unwrap();
  let again = fork_session_file( base.path(), "review" ).unwrap();
  assert_eq!( first, again, "the same name must resolve to the same file every time" );

  let expected_dir = claude_storage_core::scope_for( base.path() ).claude_session_dir;
  assert_eq!( first.parent().unwrap(), expected_dir );
  assert_eq!( first.extension().unwrap(), "jsonl" );

  let other = fork_session_file( base.path(), "release" ).unwrap();
  assert_ne!( first, other, "distinct names must not collide" );
}

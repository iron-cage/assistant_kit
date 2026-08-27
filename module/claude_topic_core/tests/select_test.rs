//! Selection tests — which topic gets the prompt.
//!
//! Pure: topics, a policy, a seed, and a process list in; one topic out. The
//! process list is constructed rather than scanned, which is the whole reason
//! `select_with` exists as a separate entry point — a draw asserted against
//! whatever happens to be running on the test machine is not an assertion.
//!
//! The cases worth having are the ones that separate `Idle` from `Random`. A
//! uniform draw passes tsl02 and fails tsl03/tsl04, which is exactly the bug the
//! policy exists to prevent: handing a second prompt to a topic already mid-turn
//! reproduces one level down the problem that made the caller reach for a topic in
//! the first place.
//!
//! ## Specification References
//!
//! - `docs/feature/003_topic_selection.md` — the policy and the busy test
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | tsl01 | No topics at all | `None` |
//! | tsl02 | `Random` over three topics | Index is `seed % 3` |
//! | tsl03 | `Idle` with one fork topic mid-turn | The other one |
//! | tsl04 | `Idle` with one dir topic mid-turn | The other one |
//! | tsl05 | `Idle` with every topic mid-turn | Still returns one, `all_busy` set |
//! | tsl06 | `is_busy` with an unrelated process | Not busy |
//! | tsl07 | `Pick` through `as_str` and `FromStr` | Round-trips both ways |
//! | tsl08 | `select` with `default_seed` | One of the inputs |

use std::path::PathBuf;
use core::str::FromStr as _;

use claude_core::process::ProcessInfo;
use claude_topic_core::{ default_seed, is_busy, select, select_with, Pick, Topic, TopicMode };

/// A fork topic whose session file — and therefore whose resume id — is `id`.
fn fork( name : &str, id : &str ) -> Topic
{
  Topic
  {
    name : name.to_string(),
    mode : TopicMode::Fork,
    path : PathBuf::from( format!( "/no-such-storage/{id}.jsonl" ) ),
    sessions : 1,
  }
}

/// A dir topic working in `/no-such-base/-<name>`.
fn dir( name : &str ) -> Topic
{
  Topic
  {
    name : name.to_string(),
    mode : TopicMode::Dir,
    path : PathBuf::from( format!( "/no-such-base/-{name}" ) ),
    sessions : 1,
  }
}

/// A live `claude` running in `cwd` with `flags` after the binary name.
fn process( cwd : &str, flags : &[ &str ] ) -> ProcessInfo
{
  let mut command_line = vec![ "claude".to_string() ];
  command_line.extend( flags.iter().map( | f | ( *f ).to_string() ) );
  ProcessInfo
  {
    pid : 4242,
    cmdline : command_line.join( " " ),
    cwd : PathBuf::from( cwd ),
    args : command_line,
  }
}

/// tsl01: nothing to draw from is `None`, not a panic and not a default.
#[ test ]
fn tsl01_empty_topic_list_selects_nothing()
{
  assert!( select_with( &[], Pick::Idle, 0, &[] ).is_none() );
  assert!( select_with( &[], Pick::Random, 7, &[] ).is_none() );
}

/// tsl02: the draw is `seed % len` — deliberately the most predictable mapping
/// there is, so a pick can be reproduced and asserted on.
#[ test ]
fn tsl02_random_draw_is_seed_modulo_length()
{
  let topics = [ fork( "a", "id-a" ), fork( "b", "id-b" ), fork( "c", "id-c" ) ];

  for ( seed, expected ) in [ ( 0_u64, "a" ), ( 1, "b" ), ( 2, "c" ), ( 3, "a" ) ]
  {
    let chosen = select_with( &topics, Pick::Random, seed, &[] ).unwrap();
    assert_eq!( chosen.topic.name, expected, "seed {seed}" );
    assert!( !chosen.all_busy );
  }
}

/// tsl03: a fork topic is busy when a live `claude` carries its resume id, and
/// `Idle` must route around it even when the seed points straight at it.
#[ test ]
fn tsl03_idle_skips_a_fork_topic_that_is_mid_turn()
{
  let topics = [ fork( "busy", "id-busy" ), fork( "free", "id-free" ) ];
  let processes = [ process( "/anywhere", &[ "--resume", "id-busy" ] ) ];

  // Seed 0 would land on "busy" under Random; Idle removes it from the pool first.
  let chosen = select_with( &topics, Pick::Idle, 0, &processes ).unwrap();
  assert_eq!( chosen.topic.name, "free" );
  assert!( !chosen.all_busy );

  assert_eq!( select_with( &topics, Pick::Random, 0, &processes ).unwrap().topic.name, "busy" );
}

/// tsl04: a dir topic has no name-derived id, so its working directory is its
/// identity — a `claude` running there is that topic's turn in flight.
#[ test ]
fn tsl04_idle_skips_a_dir_topic_that_is_mid_turn()
{
  let topics = [ dir( "busy" ), dir( "free" ) ];
  let processes = [ process( "/no-such-base/-busy", &[] ) ];

  let chosen = select_with( &topics, Pick::Idle, 0, &processes ).unwrap();
  assert_eq!( chosen.topic.name, "free" );
}

/// tsl05: "all of them are working" is a reason to say so, not a reason to refuse —
/// the caller asked for a topic and still gets one.
#[ test ]
fn tsl05_idle_falls_back_when_everything_is_busy()
{
  let topics = [ fork( "a", "id-a" ), fork( "b", "id-b" ) ];
  let processes =
  [
    process( "/anywhere", &[ "--resume", "id-a" ] ),
    process( "/anywhere", &[ "--session-id", "id-b" ] ),
  ];

  let chosen = select_with( &topics, Pick::Idle, 1, &processes ).unwrap();
  assert_eq!( chosen.topic.name, "b", "the fallback pool is the full set, in order" );
  assert!( chosen.all_busy, "the caller has to be told the prompt will queue" );
}

/// tsl06: an unrelated `claude` elsewhere does not make a topic busy.
#[ test ]
fn tsl06_unrelated_process_leaves_a_topic_idle()
{
  let topic = fork( "a", "id-a" );
  let processes = [ process( "/somewhere/else", &[ "--resume", "id-of-something-else" ] ) ];

  assert!( !is_busy( &topic, &processes ) );
  assert!( !is_busy( &topic, &[] ) );
}

/// tsl07: the wire name and the parser are exact inverses.
#[ test ]
fn tsl07_pick_round_trips_through_its_wire_name()
{
  for pick in [ Pick::Idle, Pick::Random ]
  {
    assert_eq!( Pick::from_str( pick.as_str() ).unwrap(), pick );
    assert_eq!( pick.to_string(), pick.as_str() );
  }
  assert_eq!( Pick::default(), Pick::Idle, "idle is the default policy" );
  assert!( Pick::from_str( "any" ).is_err() );
}

/// tsl08: the convenience entry point draws from the list it was given, whatever
/// the machine it runs on happens to be doing.
#[ test ]
fn tsl08_default_seed_draw_stays_within_the_given_topics()
{
  let topics = [ fork( "a", "id-a" ), fork( "b", "id-b" ) ];
  let chosen = select( &topics, Pick::Idle, default_seed() ).unwrap();
  assert!( topics.iter().any( | t | t == chosen.topic ) );
}

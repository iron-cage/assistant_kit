//! Pool tests — naming topics that exist only to be somewhere to work.
//!
//! Pure: a list of existing topics, a target, and a prefix in; a list of names
//! out. No filesystem, no environment.
//!
//! The cases worth having are the ones that separate "make sure N exist" from "add
//! N more". An implementation that appends N names passes tpl01 and fails tpl02
//! and tpl08 — and it fails them the second time a script runs, which is the point
//! at which a person is least likely to be watching.
//!
//! ## Specification References
//!
//! - `docs/feature/004_topic_pool.md` — idempotence, gap filling, prefix rules
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | tpl01 | Nothing exists yet, target 3 | `t1`, `t2`, `t3` |
//! | tpl02 | Run again against what the first run made | Nothing to create |
//! | tpl03 | `t1` and `t3` exist, target 4 | Gaps first: `t2`, `t4` |
//! | tpl04 | A richly-named topic exists, target 2 | It does not count: `t1`, `t2` |
//! | tpl05 | `t01` | Not a pool name |
//! | tpl06 | `t0` | Not a pool name |
//! | tpl07 | Unusable prefixes | Each rejected with a reason |
//! | tpl08 | Target already met | Nothing to create |
//! | tpl09 | The same index in both mechanisms | Counted once |

use std::path::PathBuf;

use claude_topic_core::{ missing_names, pool_index, validate_prefix, Topic, TopicMode, DEFAULT_PREFIX };

/// A topic of `name`, in `mode`. Only the name is read by pooling; path and
/// session count are along for the ride.
fn topic( name : &str, mode : TopicMode ) -> Topic
{
  Topic
  {
    name : name.to_string(),
    mode,
    path : PathBuf::from( format!( "/no-such-base/-{name}" ) ),
    sessions : 1,
  }
}

/// tpl01: from nothing, the pool is `t1`..`tN` in order.
#[ test ]
fn tpl01_empty_base_needs_the_whole_pool()
{
  assert_eq!( missing_names( &[], 3, DEFAULT_PREFIX ), vec![ "t1", "t2", "t3" ] );
}

/// tpl02: idempotence — this is what makes the command safe to run twice.
#[ test ]
fn tpl02_second_run_creates_nothing()
{
  let existing : Vec< _ > = missing_names( &[], 3, DEFAULT_PREFIX )
    .iter()
    .map( | n | topic( n, TopicMode::Fork ) )
    .collect();

  assert!( missing_names( &existing, 3, DEFAULT_PREFIX ).is_empty() );
}

/// tpl03: a deleted topic leaves a slot, not a permanent hole — the gap is filled
/// before the range is extended.
#[ test ]
fn tpl03_gaps_are_filled_before_the_range_extends()
{
  let existing = [ topic( "t1", TopicMode::Fork ), topic( "t3", TopicMode::Fork ) ];
  assert_eq!( missing_names( &existing, 4, DEFAULT_PREFIX ), vec![ "t2", "t4" ] );
}

/// tpl04: only pool-pattern names count toward the target, so the meaning of `N`
/// does not depend on unrelated work living in the same base.
#[ test ]
fn tpl04_named_topics_do_not_count_toward_the_target()
{
  let existing = [ topic( "review", TopicMode::Fork ), topic( "release", TopicMode::Dir ) ];
  assert_eq!( missing_names( &existing, 2, DEFAULT_PREFIX ), vec![ "t1", "t2" ] );
}

/// tpl05: a leading zero does not round-trip through `format!( "{prefix}{index}" )`,
/// so it is not a pool name — the mapping stays one-to-one.
#[ test ]
fn tpl05_leading_zero_is_not_a_pool_name()
{
  assert_eq!( pool_index( "t01", "t" ), None );
  assert_eq!( pool_index( "t1", "t" ), Some( 1 ) );
  assert_eq!( pool_index( "t10", "t" ), Some( 10 ) );
}

/// tpl06: indices start at 1, so `t0` names nothing.
#[ test ]
fn tpl06_zero_index_is_not_a_pool_name()
{
  assert_eq!( pool_index( "t0", "t" ), None );
  assert_eq!( pool_index( "t", "t" ), None );
  assert_eq!( pool_index( "review", "t" ), None );
  assert_eq!( pool_index( "u1", "t" ), None );
}

/// tpl07: each rejected prefix says why, because the caller typed it.
#[ test ]
fn tpl07_unusable_prefixes_are_rejected()
{
  assert!( validate_prefix( "t" ).is_ok() );
  assert!( validate_prefix( "work" ).is_ok() );

  for bad in [ "", "a/b", "a\nb", "-t", "t1" ]
  {
    let err = validate_prefix( bad ).unwrap_err();
    assert!( !err.is_empty(), "rejecting {bad:?} must say why" );
  }
}

/// tpl08: asking for fewer than already exist creates nothing — never deletes.
#[ test ]
fn tpl08_target_already_met_creates_nothing()
{
  let existing = [ topic( "t1", TopicMode::Fork ), topic( "t2", TopicMode::Fork ) ];
  assert!( missing_names( &existing, 2, DEFAULT_PREFIX ).is_empty() );
  assert!( missing_names( &existing, 1, DEFAULT_PREFIX ).is_empty() );
}

/// tpl09: one name held by both mechanisms is one slot, not two.
#[ test ]
fn tpl09_the_same_index_in_both_modes_counts_once()
{
  let existing = [ topic( "t1", TopicMode::Fork ), topic( "t1", TopicMode::Dir ) ];
  assert_eq!( missing_names( &existing, 2, DEFAULT_PREFIX ), vec![ "t2" ] );
}

//! Naming a pool of anonymous topics.
//!
//! A pool topic is one whose name carries no meaning — it exists to be a place
//! for work to go, not to describe the work. That makes it the opposite of the
//! auto-named topic derived from a message, which is descriptive by construction
//! and unique by a collision counter.
//!
//! # Idempotence is the whole design
//!
//! "Make sure N topics exist" and "add N more topics" are different commands, and
//! only the first is usable from a script that may run twice. [`missing_names`]
//! implements the first: it reports what is absent, so running it again after a
//! successful pass reports nothing.
//!
//! Only names that match the pool pattern count toward the target. A base holding
//! ten richly-named topics has zero pool topics, and asking for four gets four —
//! anything else would make the meaning of `N` depend on unrelated work that
//! happens to live in the same directory.
//!
//! Gaps are filled before the range is extended. With `t1` and `t3` present, a
//! target of four yields `t2` and `t4`, not `t4` and `t5`: a pool is a set of
//! slots, and a deleted topic leaves a slot rather than a permanent hole.

use crate::enumerate::Topic;

/// Prefix used when a caller does not name one.
pub const DEFAULT_PREFIX : &str = "t";

/// Reject a prefix that cannot produce usable topic names.
///
/// A topic name is a single path component that must survive a command line and a
/// one-name-per-line registry file, and it must not start with the `-` that marks
/// a topic directory. `Ok` carries nothing; the prefix is returned unchanged by
/// the caller.
///
/// # Errors
///
/// A human-readable reason the prefix is unusable.
#[ inline ]
pub fn validate_prefix( prefix : &str ) -> Result< (), String >
{
  if prefix.is_empty()
  {
    return Err( "prefix must not be empty".to_owned() );
  }
  if prefix.contains( '/' )
  {
    return Err( "prefix must be a single name component (no '/' separators)".to_owned() );
  }
  if prefix.contains( '\n' )
  {
    return Err( "prefix must not contain a newline (the topics registry is one name per line)".to_owned() );
  }
  if prefix.starts_with( '-' )
  {
    return Err( "prefix must not start with '-' (that prefix marks a topic directory)".to_owned() );
  }
  if prefix.chars().last().is_some_and( | c | c.is_ascii_digit() )
  {
    // `t1` + index 2 would be `t12`, which also reads as `t1` + index 2 the other
    // way round. Refusing the ambiguity is cheaper than resolving it.
    return Err( "prefix must not end in a digit (the index is appended directly)".to_owned() );
  }
  Ok( () )
}

/// The pool index encoded in `name`, or `None` when `name` is not a pool name for
/// `prefix`.
///
/// Exact inverse of `format!( "{prefix}{index}" )`. A leading zero (`t01`) does not
/// round-trip and is therefore not a pool name, which keeps the mapping one-to-one.
#[ inline ]
#[ must_use ]
pub fn pool_index( name : &str, prefix : &str ) -> Option< u32 >
{
  let digits = name.strip_prefix( prefix )?;
  if digits.is_empty() || ( digits.len() > 1 && digits.starts_with( '0' ) )
  {
    return None;
  }
  digits.parse().ok().filter( | n | *n > 0 )
}

/// The pool names that must be created for `target` pool topics to exist under
/// `prefix`, given what `existing` already holds.
///
/// Empty when the target is already met. A name is reported once even if the same
/// index is missing in both mechanisms — the caller creates one topic per name,
/// and which mechanism it lands in is not this function's decision.
#[ inline ]
#[ must_use ]
pub fn missing_names( existing : &[ Topic ], target : usize, prefix : &str ) -> Vec< String >
{
  let mut taken : Vec< u32 > = existing
    .iter()
    .filter_map( | t | pool_index( &t.name, prefix ) )
    .collect();
  taken.sort_unstable();
  taken.dedup();

  if taken.len() >= target
  {
    return Vec::new();
  }

  let wanted = target - taken.len();
  let mut names = Vec::with_capacity( wanted );
  let mut index : u32 = 1;
  while names.len() < wanted
  {
    if taken.binary_search( &index ).is_err()
    {
      names.push( format!( "{prefix}{index}" ) );
    }
    index += 1;
  }
  names
}

//! Choosing one topic out of many.
//!
//! # Why the default is not uniform random
//!
//! The reason to hand a prompt to a topic instead of the session in front of you
//! is usually that the session in front of you is busy. A uniform draw over every
//! topic reproduces exactly that problem one level down: it will cheerfully pick a
//! topic that is mid-turn, and the second prompt queues behind the first. So
//! [`Pick::Idle`] — draw uniformly from the topics that are *not* mid-turn — is
//! the default, and [`Pick::Random`] is available for callers who want the literal
//! semantics.
//!
//! When every topic is busy, [`Pick::Idle`] falls back to the full set rather than
//! failing: the caller asked for a topic, and "all of them are working" is a
//! reason to say so, not a reason to refuse. [`Selection::all_busy`] carries that
//! fact back so it can be reported.
//!
//! # Seeds
//!
//! The draw is `seed % len` — deliberately the most predictable mapping there is.
//! A seed exists so a pick can be reproduced and asserted on, not to supply
//! entropy; making it unpredictable would defeat the only reason to expose it.
//! [`default_seed`] is where entropy quality actually matters, and it mixes rather
//! than returning a raw clock reading, because consecutive invocations of a
//! command can land in the same nanosecond bucket on a coarse clock.

use core::time::Duration;

use claude_core::process::ProcessInfo;

use crate::enumerate::Topic;
use crate::identity::TopicMode;

/// Which topic a draw should prefer.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Default ) ]
pub enum Pick
{
  /// Draw from topics with no turn in flight, falling back to all of them when
  /// every topic is busy. The default — see the module docs.
  #[ default ]
  Idle,
  /// Draw from every topic, busy or not.
  Random,
}

impl Pick
{
  /// The lowercase wire name — `"idle"` or `"random"`.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Idle   => "idle",
      Self::Random => "random",
    }
  }
}

impl core::fmt::Display for Pick
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    f.write_str( self.as_str() )
  }
}

impl core::str::FromStr for Pick
{
  type Err = String;

  #[ inline ]
  fn from_str( s : &str ) -> core::result::Result< Self, Self::Err >
  {
    match s
    {
      "idle"   => Ok( Self::Idle ),
      "random" => Ok( Self::Random ),
      _ => Err( format!( "invalid pick policy: {s}\nExpected: idle or random" ) ),
    }
  }
}

/// The outcome of a draw.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub struct Selection< 't >
{
  /// The topic drawn.
  pub topic : &'t Topic,
  /// True when [`Pick::Idle`] was asked for, every candidate was busy, and the
  /// draw therefore fell back to the full set. The caller should say so — the
  /// prompt will queue behind a turn already in flight.
  pub all_busy : bool,
}

/// `SplitMix64`'s finalizer — a full 64-bit avalanche, so low bits of a coarse
/// clock reading are as well distributed as high ones.
///
/// Not a random number generator and not used as one: a single mixing step over
/// a value that already varies per invocation.
const fn mix( z : u64 ) -> u64
{
  let z = ( z ^ ( z >> 30 ) ).wrapping_mul( 0xbf58_476d_1ce4_e5b9 );
  let z = ( z ^ ( z >> 27 ) ).wrapping_mul( 0x94d0_49bb_1331_11eb );
  z ^ ( z >> 31 )
}

/// A seed for callers who did not supply one, from the wall clock and this
/// process's id.
///
/// The pid matters: two `clr` invocations started by the same shell loop can read
/// the same nanosecond on a clock with millisecond granularity, and without the
/// pid they would then draw the same topic.
#[ inline ]
#[ must_use ]
pub fn default_seed() -> u64
{
  let since_epoch = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap_or( Duration::ZERO );
  // Reassembled from the two u64-safe halves rather than truncating `as_nanos`'s
  // u128 — the workspace denies truncating casts, and wrapping at 2^64 nanoseconds
  // costs a seed nothing.
  let nanos = since_epoch
    .as_secs()
    .wrapping_mul( 1_000_000_000 )
    .wrapping_add( u64::from( since_epoch.subsec_nanos() ) );
  mix( nanos ^ u64::from( std::process::id() ).rotate_left( 32 ) )
}

/// Whether `topic` has a turn in flight, judged against an already-collected
/// process list.
///
/// The list is passed in rather than scanned per topic so that one `/proc` sweep
/// answers the question for every candidate — a sweep per topic would also give
/// each candidate a different instant to be judged at.
///
/// - A fork topic is busy when some live `claude` carries its deterministic
///   session id in argv (`--resume <id>`, `--session-id <id>`, either spelling).
/// - A dir topic is busy when some live `claude` is running in its directory.
///   There is no name-derived id to match on, so the directory is the identity.
#[ inline ]
#[ must_use ]
pub fn is_busy( topic : &Topic, processes : &[ ProcessInfo ] ) -> bool
{
  match topic.mode
  {
    TopicMode::Fork =>
    {
      let Some( id ) = topic.session_id() else { return false };
      processes.iter().any( | p | p.args.iter().any( | a | a == &id ) )
    },
    TopicMode::Dir =>
    {
      let here = claude_storage_core::physical_abs( &topic.path );
      processes.iter().any( | p | claude_storage_core::physical_abs( &p.cwd ) == here )
    },
  }
}

/// Draw one topic from `topics` under `pick`, using `seed`.
///
/// `None` only when `topics` is empty. Scans `/proc` once for [`Pick::Idle`]; a
/// caller that already holds a process list should use [`select_with`] so the
/// whole draw is judged at one instant. See the module docs for the draw rule and
/// the [`Pick::Idle`] fallback.
#[ inline ]
#[ must_use ]
pub fn select( topics : &[ Topic ], pick : Pick, seed : u64 ) -> Option< Selection< '_ > >
{
  let processes = match pick
  {
    // Random ignores busyness entirely, so the sweep would be pure cost.
    Pick::Random => Vec::new(),
    Pick::Idle => claude_core::process::find_claude_processes(),
  };
  select_with( topics, pick, seed, &processes )
}

/// [`select`], against an already-collected process list.
///
/// This is the whole of the selection logic; [`select`] only supplies the sweep.
/// Separating them is what makes a draw assertable — the outcome is then a pure
/// function of `topics`, `pick`, `seed`, and `processes`, with nothing read from
/// the machine it runs on.
#[ inline ]
#[ must_use ]
pub fn select_with< 't >
(
  topics : &'t [ Topic ],
  pick : Pick,
  seed : u64,
  processes : &[ ProcessInfo ],
) -> Option< Selection< 't > >
{
  if topics.is_empty()
  {
    return None;
  }

  let ( candidates, all_busy ) = match pick
  {
    Pick::Random => ( topics.iter().collect::< Vec< _ > >(), false ),
    Pick::Idle =>
    {
      let idle : Vec< _ > = topics.iter().filter( | t | !is_busy( t, processes ) ).collect();
      if idle.is_empty() { ( topics.iter().collect(), true ) } else { ( idle, false ) }
    },
  };

  // `seed % len`, via try_from rather than `as` because the workspace denies
  // truncating casts. Neither fallback is reachable: a slice length always fits a
  // u64, and a value already reduced modulo that length always fits a usize.
  let len = u64::try_from( candidates.len() ).unwrap_or( u64::MAX );
  let index = usize::try_from( seed % len ).unwrap_or( 0 );
  Some( Selection { topic : candidates[ index ], all_busy } )
}

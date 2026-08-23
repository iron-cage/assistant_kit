//! Unit tests for `src/cli/projects_overview.rs` — the totals line.
//!
//! Relocated out of a `#[ cfg( test ) ]` module in the source file: every test
//! in this crate lives under `tests/`. `claude_storage::cli::projects_overview`
//! is `#[ doc( hidden ) ] pub` for exactly this purpose (see `src/cli/mod.rs`).

use claude_storage::cli::liveness::Liveness;
use claude_storage::cli::projects_overview::{ OverviewRow, summary_line };
use std::time::SystemTime;

/// A row carrying only the fields the totals line reads.
fn row( conversations : usize, agents : usize ) -> OverviewRow
{
  OverviewRow
  {
    display_path : "~/p".to_string(),
    conversations,
    agents,
    last_mtime   : SystemTime::UNIX_EPOCH,
  }
}

/// The live clause spells out both halves only when both are non-empty.
///
/// A totals line exists to be read at a glance, and `1 live (1 working, 0
/// waiting)` spends a third of its width restating the count it just gave and
/// naming a state nothing is in — the same waste the `agents` clause already
/// avoids by disappearing at zero.
#[ test ]
fn test_live_clause_collapses_a_zero_half()
{
  let rows = [ row( 1, 0 ), row( 1, 0 ), row( 1, 0 ) ];

  let mixed = [ Some( Liveness::Working ), Some( Liveness::Waiting ), Some( Liveness::Waiting ) ];
  assert!( summary_line( &rows, &mixed ).ends_with( "· 3 live (1 working, 2 waiting)" ),
    "both halves present: {}", summary_line( &rows, &mixed ) );

  let working_only = [ Some( Liveness::Working ), Some( Liveness::Working ), None ];
  assert!( summary_line( &rows, &working_only ).ends_with( "· 2 live (working)" ),
    "all working: {}", summary_line( &rows, &working_only ) );

  let waiting_only = [ Some( Liveness::Waiting ), None, None ];
  assert!( summary_line( &rows, &waiting_only ).ends_with( "· 1 live (waiting)" ),
    "all waiting: {}", summary_line( &rows, &waiting_only ) );
}

/// No attachment found means no clause at all — never `0 live`.
///
/// Detection reports only positives, so an empty result is "nothing seen",
/// which a rendered zero would misstate as "nothing running".
#[ test ]
fn test_live_clause_absent_when_nothing_is_attached()
{
  let rows = [ row( 2, 5 ) ];
  let line = summary_line( &rows, &[ None ] );

  assert!( !line.contains( "live" ), "no live clause without an attachment: {line}" );
  assert!( line.contains( "5 agents" ), "the rest of the line is unaffected: {line}" );
}

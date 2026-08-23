//! Flexible grouped/filtered/sorted token-usage rollup engine.
//!
//! Pure aggregation over already-computed [`SessionStats`] — no filesystem or
//! CLI-argument-parsing dependency, so every grouping/sort/filter path is
//! unit-testable in isolation without touching JSONL storage at all. Powers
//! the `claude_storage` CLI's `.rollup` command; see
//! `claude_storage/docs/cli/command/14_rollup.md` for the full CLI contract
//! this engine is built to serve.

use crate::{ SessionStats, StringMatcher };
use std::collections::HashMap;

/// Dimension a `.rollup` result set is grouped by.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum GroupKey
{
  /// One row per session (finest granularity) — closest to `.usage`'s shape,
  /// but still projectable/sortable/filterable unlike that fixed command.
  Session,
  /// One row per project, summing every session under it.
  Project,
  /// One row per model name (first-seen model per session; sessions with no
  /// recorded model group under `"unknown"`).
  Model,
  /// One row per calendar day (`first_timestamp`'s `YYYY-MM-DD`, UTC as
  /// recorded; sessions with no timestamp group under `"unknown"`).
  Day,
}

/// Column a result set is sorted by. Every key operates on already-aggregated
/// row totals — sorting always happens after grouping, never before.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum SortKey
{
  /// `input + output + cache_read + cache_creation` — the default.
  Total,
  /// Fresh (non-cached) input tokens.
  Input,
  /// Generated output tokens.
  Output,
  /// `cache_read + cache_creation` combined.
  Cache,
  /// Largest single call's context size (the "window size" metric).
  MaxContext,
  /// Number of assistant turns (deduplicated API calls).
  Calls,
  /// Number of distinct sessions contributing to the row.
  Sessions,
  /// Lexicographic by the row's group label.
  Group,
}

/// Sort direction.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum SortOrder
{
  /// Ascending — smallest first.
  Asc,
  /// Descending — largest first.
  Desc,
}

/// One session's contribution to the rollup, assembled before grouping.
///
/// Built by the CLI layer walking scope-resolved projects/sessions (mirrors
/// `.usage`'s own `collect_rows` glue) — this engine never touches the
/// filesystem itself, which is what keeps it unit-testable with plain
/// synthetic values.
#[ derive( Debug, Clone ) ]
pub struct RollupInput
{
  /// Session ID — used verbatim as the group key under [`GroupKey::Session`].
  pub session_id : String,
  /// Project identifier/path this session belongs to — used as the group key
  /// under [`GroupKey::Project`].
  pub project_label : String,
  /// This session's already-deduplicated stats (see `Session::stats()`,
  /// `Fix(issue-038)`).
  pub stats : SessionStats,
}

/// One aggregated, fully-computed output row.
///
/// Every field is always populated — column *projection* (choosing which
/// fields to print, and in what order) is a pure display concern left to the
/// CLI layer, matching `.usage`'s existing core/CLI split (`render_row`/
/// `format_tokens` live in `cli/usage.rs`, not here).
#[ derive( Debug, Clone, PartialEq ) ]
pub struct RollupRow
{
  /// Group label: session id / project label / model name / day
  /// (`YYYY-MM-DD`) — `"unknown"` when the grouping field was absent on
  /// every contributing session.
  pub group : String,
  /// Number of distinct sessions contributing to this row.
  pub sessions : usize,
  /// Number of distinct assistant turns (deduplicated API calls) contributing.
  pub calls : usize,
  /// Fresh (non-cached) input tokens.
  pub input : u64,
  /// Generated output tokens.
  pub output : u64,
  /// Tokens read from prompt cache.
  pub cache_read : u64,
  /// Tokens written to prompt cache.
  pub cache_creation : u64,
  /// Largest single call's context size seen across contributing sessions
  /// (see `SessionStats::max_context_tokens`) — the "window size" metric.
  pub max_context : u64,
  /// `100.0 * total() / grand_total`, computed against the full filtered
  /// result set — before `limit` truncates it (see [`build_rollup`]'s doc).
  /// `0.0` when the grand total itself is `0`.
  pub percent : f64,
  /// Earliest `first_timestamp` among contributing sessions.
  pub first : Option< String >,
  /// Latest `last_timestamp` among contributing sessions.
  pub last : Option< String >,
}

impl RollupRow
{
  /// `cache_read + cache_creation` combined.
  #[ must_use ]
  #[ inline ]
  pub fn cache( &self ) -> u64
  {
    self.cache_read + self.cache_creation
  }

  /// `input + output + cache()` — the metric [`SortKey::Total`] and
  /// `percent` are both computed against.
  #[ must_use ]
  #[ inline ]
  pub fn total( &self ) -> u64
  {
    self.input + self.output + self.cache()
  }

  fn empty( group : String ) -> Self
  {
    Self
    {
      group,
      sessions : 0,
      calls : 0,
      input : 0,
      output : 0,
      cache_read : 0,
      cache_creation : 0,
      max_context : 0,
      percent : 0.0,
      first : None,
      last : None,
    }
  }
}

/// Parameters controlling one [`build_rollup`] call.
///
/// Not `Clone`: `StringMatcher` doesn't implement it, and nothing in this
/// engine needs to duplicate a `RollupParams` — callers build one and pass it
/// by reference.
#[ derive( Debug ) ]
pub struct RollupParams
{
  /// Dimension to group rows by.
  pub group_by : GroupKey,
  /// Column to sort the grouped rows by.
  pub sort_by : SortKey,
  /// Sort direction applied to `sort_by`.
  pub order : SortOrder,
  /// Session-granularity model substring filter, applied before grouping —
  /// a session whose `stats.model` doesn't match (or is absent while a
  /// filter is set) is dropped before it can contribute to any row.
  pub model_filter : Option< StringMatcher >,
  /// `0` = unbounded. Applied after sort, as a flat cap on the row count.
  pub limit : usize,
}

/// Group, filter, aggregate, compute percentages, sort, and cap `entries`
/// per `params`. Pure function — no I/O, cannot fail.
///
/// # Percent semantics
///
/// `percent` is computed against the grand total of the *entire filtered*
/// result set (every group that survives `model_filter`, before `limit`
/// truncates rows) — not just the rows that end up visible. This keeps "this
/// row is N% of the total" meaningful regardless of how narrow `limit::` is;
/// a `limit::5` view still reports each row's true share of everything that
/// matched, not just of the other 4 rows shown alongside it.
#[ must_use ]
#[ inline ]
pub fn build_rollup( entries : &[ RollupInput ], params : &RollupParams ) -> Vec< RollupRow >
{
  let filtered = entries.iter().filter( | e | matches_model_filter( e, params.model_filter.as_ref() ) );

  let mut groups : HashMap< String, RollupRow > = HashMap::new();
  for entry in filtered
  {
    let key = group_key_for( entry, params.group_by );
    let row = groups.entry( key.clone() ).or_insert_with( || RollupRow::empty( key ) );
    accumulate( row, entry );
  }

  let grand_total : u64 = groups.values().map( RollupRow::total ).sum();
  let mut rows : Vec< RollupRow > = groups.into_values()
    .map( | mut row |
    {
      row.percent = if grand_total == 0
      {
        0.0
      }
      else
      {
        100.0 * row.total() as f64 / grand_total as f64
      };
      row
    })
    .collect();

  sort_rows( &mut rows, params.sort_by, params.order );

  if params.limit > 0
  {
    rows.truncate( params.limit );
  }

  rows
}

/// Does `entry` survive `filter`? No filter (`None`) always matches; a set
/// filter requires `entry.stats.model` to be present *and* match.
fn matches_model_filter( entry : &RollupInput, filter : Option< &StringMatcher > ) -> bool
{
  let Some( matcher ) = filter else { return true };
  entry.stats.model.as_deref().is_some_and( | model | matcher.matches( model ) )
}

fn group_key_for( entry : &RollupInput, group_by : GroupKey ) -> String
{
  match group_by
  {
    GroupKey::Session => entry.session_id.clone(),
    GroupKey::Project => entry.project_label.clone(),
    GroupKey::Model => entry.stats.model.clone().unwrap_or_else( || "unknown".to_string() ),
    GroupKey::Day => entry.stats.first_timestamp.as_deref()
      .and_then( | ts | ts.get( 0..10 ) )
      .map_or_else( || "unknown".to_string(), std::string::ToString::to_string ),
  }
}

/// Fold one `entry` into `row`: bump counts, sum tokens, track the running
/// max context, and widen the `first`/`last` timestamp span.
fn accumulate( row : &mut RollupRow, entry : &RollupInput )
{
  row.sessions += 1;
  row.calls += entry.stats.assistant_entries;
  row.input += entry.stats.total_input_tokens;
  row.output += entry.stats.total_output_tokens;
  row.cache_read += entry.stats.total_cache_read_tokens;
  row.cache_creation += entry.stats.total_cache_creation_tokens;
  if entry.stats.max_context_tokens > row.max_context
  {
    row.max_context = entry.stats.max_context_tokens;
  }

  // ISO-8601 timestamps compare correctly as plain strings (zero-padded,
  // fixed-width, same format throughout) — no date parsing needed.
  if let Some( ts ) = entry.stats.first_timestamp.as_deref()
  {
    if row.first.as_deref().map_or( true, | cur | ts < cur )
    {
      row.first = Some( ts.to_string() );
    }
  }
  if let Some( ts ) = entry.stats.last_timestamp.as_deref()
  {
    if row.last.as_deref().map_or( true, | cur | ts > cur )
    {
      row.last = Some( ts.to_string() );
    }
  }
}

// Fix(BUG-529): append a secondary, always-ascending tie-break on `group`
// after the `order`-adjusted primary comparison.
//
// Root cause: rows arrive here from `HashMap::into_values()` (see
// `build_rollup`), whose iteration order is process-randomized by `HashMap`'s
// default `RandomState` hasher. `sort_by` is a *stable* sort, but stability
// only preserves the incoming order for elements that compare `Equal` — it
// supplies no ordering of its own. Two rows tied on `sort_by`'s metric (e.g.
// two projects with identical `total()`) therefore surfaced in whatever
// arbitrary order `HashMap` handed them in, changing on every fresh process
// invocation despite unchanged underlying data.
//
// Pitfall: "stable sort" reads like a determinism guarantee; it only
// relocates non-determinism from "tie order after sorting" to "arrival order
// before sorting" — which is exactly as random as its source. Any pipeline
// that sorts a `Vec` sourced from a `HashMap` needs an explicit secondary key
// wherever a *total* order (not just a partial order by the primary metric)
// is actually part of the contract.
fn sort_rows( rows : &mut [ RollupRow ], sort_by : SortKey, order : SortOrder )
{
  rows.sort_by( | a, b |
  {
    let ord = match sort_by
    {
      SortKey::Total => a.total().cmp( &b.total() ),
      SortKey::Input => a.input.cmp( &b.input ),
      SortKey::Output => a.output.cmp( &b.output ),
      SortKey::Cache => a.cache().cmp( &b.cache() ),
      SortKey::MaxContext => a.max_context.cmp( &b.max_context ),
      SortKey::Calls => a.calls.cmp( &b.calls ),
      SortKey::Sessions => a.sessions.cmp( &b.sessions ),
      SortKey::Group => a.group.cmp( &b.group ),
    };
    let ordered = match order
    {
      SortOrder::Asc => ord,
      SortOrder::Desc => ord.reverse(),
    };
    // Tie-break is always ascending by group, regardless of `order::` — it's
    // a display-stability concern, not part of the user's requested
    // direction. Group labels are unique per row, so this always yields a
    // total order with no further ties possible.
    ordered.then_with( || a.group.cmp( &b.group ) )
  });
}

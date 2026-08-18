//! Quota measurement history ring buffer (Feature 040).

use claude_core::file_io::atomic_write;
use super::quota_cache::{ local_cache_path, read_json_value, read_volatile_candidates };

/// Timestamped quota measurement stored in the per-host cache file's `history[]`
/// (`cache/{host}_{user}/{name}.json`; legacy stores: gitignored
/// `-cache/{name}.json` or tracked `cache.history[]`).
///
/// Each successful fetch appends one entry; the array is capped at 10 entries (FIFO).
/// Used by the approximation module to fit a polynomial when the API is unavailable
/// (Feature 040).
#[ derive( Debug ) ]
pub struct HistoryEntry
{
  /// Unix timestamp (seconds) when the measurement was taken.
  pub t  : u64,
  /// 5h period: `(utilization 0–100, resets_at ISO string)`; `None` when absent.
  pub h5 : Option< ( f64, String ) >,
  /// 7d period: `(utilization 0–100, resets_at ISO string)`; `None` when absent.
  pub d7 : Option< ( f64, String ) >,
  /// 7d-sonnet period: `(utilization 0–100, resets_at ISO string)`; `None` when absent.
  pub sn : Option< ( f64, String ) >,
}

/// Parse a `[f64, string]` JSON array into a period tuple.
fn parse_history_period( val : &serde_json::Value ) -> Option< ( f64, String ) >
{
  let arr = val.as_array()?;
  if arr.len() != 2 { return None; }
  let u = arr[ 0 ].as_f64()?;
  let r = arr[ 1 ].as_str()?.to_string();
  Some( ( u, r ) )
}

/// Read measurement history (Feature 040 AC-11) — the freshest volatile
/// candidate carrying a `"history"` array wins, across every host subtree
/// `cache/*/{name}.json` and the legacy gitignored `-cache/{name}.json`
/// (TSK-502); a history-only own-host file (no `fetched_at` yet) is read
/// directly when no candidate carries a ring; a legacy tracked
/// `cache.history[]` is the pre-migration fallback (TSK-500).
///
/// Returns an empty `Vec` when no location has a `"history"` array —
/// backward compatible with old cache format from Feature 033.
#[ must_use ]
#[ inline ]
pub fn read_history(
  credential_store : &std::path::Path,
  name             : &str,
) -> Vec< HistoryEntry >
{
  let candidate = read_volatile_candidates( credential_store, name )
    .into_iter()
    .find_map( | mut c | match c.remove( "history" )
    {
      Some( serde_json::Value::Array( a ) ) => Some( a ),
      _ => None,
    } )
    .or_else( ||
      // A history-only own-host file (written by `write_history_entry` before any
      //   fetch) carries no `fetched_at`, so it is not a candidate — read it directly.
      read_json_value( &local_cache_path( credential_store, name ) )
        .and_then( | mut l | match l.as_object_mut().and_then( | o | o.remove( "history" ) )
        {
          Some( serde_json::Value::Array( a ) ) => Some( a ),
          _ => None,
        } )
    );
  let tracked;
  let arr = if let Some( a ) = candidate.as_ref()
  {
    a
  }
  else
  {
    tracked = read_json_value( &credential_store.join( format!( "{name}.json" ) ) );
    let Some( a ) = tracked.as_ref()
      .and_then( | t | t.get( "cache" ) )
      .and_then( | c | c.get( "history" ) )
      .and_then( | h | h.as_array() ) else { return vec![] };
    a
  };
  arr.iter().filter_map( |entry|
  {
    let t = entry.get( "t" )?.as_u64()?;
    Some( HistoryEntry
    {
      t,
      h5 : entry.get( "h5" ).and_then( parse_history_period ),
      d7 : entry.get( "d7" ).and_then( parse_history_period ),
      sn : entry.get( "sn" ).and_then( parse_history_period ),
    } )
  } ).collect()
}

/// Serialize an optional period to a JSON array `[utilization, resets_at]` or `null`.
fn history_period_json( period : Option< ( f64, &str ) > ) -> serde_json::Value
{
  match period
  {
    Some( ( u, r ) ) => serde_json::json!( [ u, r ] ),
    None             => serde_json::Value::Null,
  }
}

/// Append a quota measurement to `history[]` in this host's tracked cache file
/// `cache/{host}_{user}/{name}.json` (Feature 040 AC-01, AC-02, AC-13; TSK-500/502).
///
/// - Enforces a 10-entry FIFO ring buffer: oldest entry evicted when buffer is full (AC-02).
/// - Overwrites the last entry when `t` matches its timestamp to prevent fast-cycle fill (AC-13).
/// - When the own-host file has no ring yet, seeds from the freshest candidate anywhere
///   (another host's subtree or the legacy gitignored file — TSK-502 cross-host
///   continuity), then from a legacy tracked `cache.history[]` (TSK-500 upgrade);
///   the tracked `{name}.json` is never written.
/// - Write failures are silently ignored — quota display is non-critical (matches Feature 033 pattern).
#[ inline ]
pub fn write_history_entry(
  credential_store : &std::path::Path,
  name             : &str,
  t                : u64,
  h5               : Option< ( f64, &str ) >,
  d7               : Option< ( f64, &str ) >,
  sn               : Option< ( f64, &str ) >,
)
{
  let local_path = local_cache_path( credential_store, name );
  let mut snapshot = read_json_value( &local_path ).unwrap_or_else( || serde_json::json!( {} ) );
  if let Some( obj ) = snapshot.as_object_mut()
  {
    if !obj.contains_key( "history" )
    {
      // Ring seed: the freshest candidate anywhere first (another host's subtree or
      //   the legacy gitignored file — TSK-502 continuity), then a pre-migration
      //   tracked ring (read-only on the tracked file — TSK-500).
      let seed = read_volatile_candidates( credential_store, name )
        .into_iter()
        .find_map( | mut c | c.remove( "history" ) )
        .or_else( ||
          read_json_value( &credential_store.join( format!( "{name}.json" ) ) )
            .and_then( | tr | tr.get( "cache" ).and_then( | c | c.get( "history" ) ).cloned() )
        );
      if let Some( h ) = seed
      {
        obj.insert( "history".into(), h );
      }
    }
    let history = obj.entry( "history" ).or_insert_with( || serde_json::json!( [] ) );
    if let Some( arr ) = history.as_array_mut()
    {
      let entry = serde_json::json!(
      {
        "t"  : t,
        "h5" : history_period_json( h5 ),
        "d7" : history_period_json( d7 ),
        "sn" : history_period_json( sn ),
      } );
      // AC-13: duplicate-timestamp dedup — overwrite last entry when same Unix second.
      if let Some( last ) = arr.last()
      {
        if last.get( "t" ).and_then( serde_json::Value::as_u64 ) == Some( t )
        {
          let len = arr.len();
          arr[ len - 1 ] = entry;
        }
        else
        {
          arr.push( entry );
        }
      }
      else
      {
        arr.push( entry );
      }
      // AC-02: ring buffer FIFO cap — evict oldest (index 0) when over 10 entries.
      while arr.len() > 10
      {
        arr.remove( 0 );
      }
    }
  }
  if let Some( dir ) = local_path.parent()
  {
    let _ = std::fs::create_dir_all( dir );
  }
  let _ = atomic_write(
    &local_path,
    &serde_json::to_string_pretty( &snapshot ).map( |s| s + "\n" ).unwrap_or_default(),
  );
}

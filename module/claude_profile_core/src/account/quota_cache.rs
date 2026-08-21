//! Per-host quota cache tree — volatile cache writes, merged reads, legacy migration.

use claude_core::chrono_now_utc;
use claude_core::file_io::atomic_write;
use super::ownership::host_user_slug;

/// Cached quota entry — volatile fields from the freshest per-host tracked
/// cache file `cache/{host}_{user}/{name}.json` across all host subtrees
/// (TSK-502; the legacy gitignored `-cache/{name}.json` participates as a
/// migration-era candidate), low-churn metadata from top-level keys of the
/// tracked `{name}.json`; a legacy tracked `cache{}` block is honored as
/// pre-migration fallback for both groups (TSK-500).
#[ derive( Debug ) ]
pub struct QuotaCacheEntry
{
  /// UTC ISO-8601 timestamp of the last successful fetch.
  pub fetched_at        : String,
  /// 5h period: (`utilization` 0–100, `resets_at` ISO string or `None`).
  pub five_hour         : Option< ( f64, Option< String > ) >,
  /// 7d period: (`utilization`, `resets_at`).
  pub seven_day         : Option< ( f64, Option< String > ) >,
  /// 7d-sonnet period: (`utilization`, `resets_at`).
  pub seven_day_sonnet  : Option< ( f64, Option< String > ) >,
  /// Persisted model override decision.
  pub model_override    : Option< String >,
  /// Last touch timestamp (UTC ISO-8601).
  pub last_touch_at     : Option< String >,
  /// Whether the account is idle (no active 5h window).
  pub touch_idle        : Option< bool >,
  // Fix(BUG-327): QuotaCacheEntry had no field to carry org_created_at, so the 3
  //   non-live branches (G1-not-owned, cache-first, approximate_quota) could never
  //   populate renews_label()'s org_created_at_opt — ~Renews always showed "?".
  //   Root cause: org_created_at lived only on AccountQuota.account (live-fetch only);
  //   no field existed to persist/restore it across a cache round-trip.
  //   Pitfall: independent of `account: Option<OauthAccountData>` — never reconstruct
  //   a fake OauthAccountData to backfill it (risks BUG-232 regression).
  /// Org creation timestamp (UTC ISO-8601), persisted so cache-only reads can compute renewal dates.
  pub org_created_at    : Option< String >,
}

/// Root of the tracked per-host cache tree inside the credential store (TSK-502).
fn cache_tree_dir( credential_store : &std::path::Path ) -> std::path::PathBuf
{
  credential_store.join( "cache" )
}

/// Path of this host's tracked quota cache file for `name` (TSK-502).
///
/// Lives in the per-host subtree `cache/{host}_{user}/` — no component is
/// hyphen-prefixed, so the global `-*` gitignore rule cannot match it and the
/// file rides ordinary commits, restoring fleet-wide cache visibility. Only
/// this host writes its own subtree (slug shared with `active_marker_filename`),
/// so the churn is merge-trivial.
pub( super ) fn local_cache_path( credential_store : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  cache_tree_dir( credential_store ).join( host_user_slug() ).join( format!( "{name}.json" ) )
}

/// Path of the legacy gitignored host-local cache file (TSK-500's layout).
///
/// Read as a migration-era candidate; deleted by `write_quota_cache` after the
/// first successful per-host write (self-cleaning migration, TSK-502).
fn legacy_local_cache_path( credential_store : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  credential_store.join( "-cache" ).join( format!( "{name}.json" ) )
}

/// Volatile cache candidates for `name` from every host subtree under `cache/`
/// plus the legacy gitignored file, freshest `fetched_at` first (TSK-502).
///
/// A candidate participates only with a parseable `fetched_at` — an unparseable
/// timestamp is skipped entirely (feature/033's "treat as no cache"), never
/// selected and never aborting the merge.
pub( super ) fn read_volatile_candidates(
  credential_store : &std::path::Path,
  name             : &str,
) -> Vec< serde_json::Map< String, serde_json::Value > >
{
  let mut paths : Vec< std::path::PathBuf > = vec![];
  if let Ok( entries ) = std::fs::read_dir( cache_tree_dir( credential_store ) )
  {
    for entry in entries.flatten()
    {
      paths.push( entry.path().join( format!( "{name}.json" ) ) );
    }
  }
  paths.push( legacy_local_cache_path( credential_store, name ) );
  let mut candidates : Vec< ( u64, serde_json::Map< String, serde_json::Value > ) > = paths
    .iter()
    .filter_map( | p | read_json_value( p ) )
    .filter_map( | v | if let serde_json::Value::Object( o ) = v { Some( o ) } else { None } )
    .filter_map( | o |
    {
      let secs = o.get( "fetched_at" ).and_then( | f | f.as_str() ).and_then( parse_iso_utc_secs )?;
      Some( ( secs, o ) )
    } )
    .collect();
  candidates.sort_by_key( | c | core::cmp::Reverse( c.0 ) );
  candidates.into_iter().map( | ( _, o ) | o ).collect()
}

/// Parse a JSON file into a `Value`; `None` when absent, unreadable, or malformed.
pub( super ) fn read_json_value( path : &std::path::Path ) -> Option< serde_json::Value >
{
  let text = std::fs::read_to_string( path ).ok()?;
  serde_json::from_str( &text ).ok()
}

/// Dissolve a legacy tracked `cache{}` block — TSK-500's one-time migration write.
///
/// Low-churn metadata keys relocate to top level (an existing top-level value
/// wins — it is newer), volatile quota fields are dropped with the block (the
/// caller re-persists fresh values to the local file and adopts the returned
/// legacy history), and the `cache` key is removed entirely. Returns the removed
/// legacy object so the caller can honor it; `None` — with zero tracked writes —
/// when the store is already migrated.
fn migrate_legacy_cache(
  credential_store : &std::path::Path,
  name             : &str,
) -> Option< serde_json::Map< String, serde_json::Value > >
{
  let meta_path = credential_store.join( format!( "{name}.json" ) );
  let mut snapshot = read_json_value( &meta_path )?;
  let obj = snapshot.as_object_mut()?;
  if !obj.get( "cache" ).is_some_and( serde_json::Value::is_object )
  {
    return None;
  }
  let Some( serde_json::Value::Object( legacy ) ) = obj.remove( "cache" ) else { return None };
  for key in [ "model_override", "last_touch_at", "touch_idle", "org_created_at" ]
  {
    if let Some( v ) = legacy.get( key )
    {
      if !obj.contains_key( key )
      {
        obj.insert( key.to_string(), v.clone() );
      }
    }
  }
  let _ = atomic_write( &meta_path, &serde_json::to_string_pretty( &snapshot ).map( | s | s + "\n" ).unwrap_or_default() );
  Some( legacy )
}

/// Write the volatile quota cache to this host's tracked file
/// `cache/{host}_{user}/{name}.json` (TSK-502).
///
/// Persists the last successful fetch result so it can be used as fallback when
/// the usage API is unavailable. The tracked `{name}.json` is never rewritten on
/// the steady-state path — a successful fetch performs zero writes to tracked
/// credential files (TSK-500's defining property; the per-host cache file is
/// tracked but dedicated, single-writer, and never carries credentials). The one
/// exception is the first call against a store still carrying a legacy `cache{}`
/// block, which triggers `migrate_legacy_cache`'s single transition write. The
/// history ring is seeded from the freshest candidate anywhere — another host's
/// subtree or the legacy gitignored file — so ring continuity survives host
/// handoffs. After a successful write, the legacy gitignored `-cache/{name}.json`
/// is deleted (self-cleaning migration). Failures are silently ignored.
#[ inline ]
pub fn write_quota_cache(
  credential_store  : &std::path::Path,
  name              : &str,
  five_hour         : Option< ( f64, Option< &str > ) >,
  seven_day         : Option< ( f64, Option< &str > ) >,
  seven_day_sonnet  : Option< ( f64, Option< &str > ) >,
)
{
  let legacy = migrate_legacy_cache( credential_store, name );
  let local_path = local_cache_path( credential_store, name );
  let candidates = read_volatile_candidates( credential_store, name );
  let mut cache = serde_json::json!( { "fetched_at": chrono_now_utc(), "status": "ok" } );
  if let Some( co ) = cache.as_object_mut()
  {
    if let Some( ( u, r ) ) = five_hour
    {
      co.insert( "five_hour".into(), period_json( u, r ) );
    }
    if let Some( ( u, r ) ) = seven_day
    {
      co.insert( "seven_day".into(), period_json( u, r ) );
    }
    if let Some( ( u, r ) ) = seven_day_sonnet
    {
      co.insert( "seven_day_sonnet".into(), period_json( u, r ) );
    }
    // Feature 040: "history" must survive write_quota_cache or every successful fetch
    //   would clobber the stored ring buffer (verification finding F4-3). The freshest
    //   candidate carrying a ring wins — own subtree, another host's, or the legacy
    //   gitignored file (TSK-502 cross-host continuity); a just-dissolved legacy
    //   `cache{}` block seeds it on the first post-upgrade write (TSK-500).
    let history = candidates.iter().find_map( | c | c.get( "history" ) ).cloned()
      .or_else( ||
        // History-only own-host file (ring written before any fetch): no `fetched_at`
        //   means it is not a candidate — preserve its ring by reading it directly.
        read_json_value( &local_path )
          .and_then( | mut l | l.as_object_mut().and_then( | o | o.remove( "history" ) ) )
      )
      .or_else( || legacy.as_ref().and_then( | l | l.get( "history" ) ).cloned() );
    if let Some( h ) = history
    {
      co.insert( "history".into(), h );
    }
  }
  if let Some( dir ) = local_path.parent()
  {
    let _ = std::fs::create_dir_all( dir );
  }
  if atomic_write( &local_path, &serde_json::to_string_pretty( &cache ).map( | s | s + "\n" ).unwrap_or_default() ).is_ok()
  {
    // Self-cleaning migration (TSK-502): the legacy gitignored file's role is fully
    //   absorbed by the per-host tracked file — deleted only after the new-path
    //   write succeeded, so a failed write never orphans the only copy.
    let _ = std::fs::remove_file( legacy_local_cache_path( credential_store, name ) );
  }
}

/// Read cached quota, merging per-host volatile files with tracked metadata.
///
/// Volatile fields (`fetched_at`, `status`, periods) come from the freshest
/// candidate across every host subtree `cache/*/{name}.json` and the legacy
/// gitignored `-cache/{name}.json` (freshest-`fetched_at`-wins, TSK-502) — this
/// is what makes another host's fetch visible here; low-churn metadata
/// (`model_override`, `last_touch_at`, `touch_idle`, `org_created_at`) from
/// top-level keys of the tracked `{name}.json`. A legacy tracked `cache{}`
/// block (pre-TSK-500 store) is honored as fallback for both groups. Returns
/// `None` when no volatile cache exists in any location — the pre-split
/// "no cache" contract.
#[ inline ]
pub fn read_quota_cache( credential_store : &std::path::Path, name : &str ) -> Option< QuotaCacheEntry >
{
  let tracked = read_json_value( &credential_store.join( format!( "{name}.json" ) ) );
  let legacy = tracked.as_ref().and_then( | t | t.get( "cache" ) ).and_then( | c | c.as_object() );
  let freshest = read_volatile_candidates( credential_store, name ).into_iter().next();
  let volatile = match freshest.as_ref()
  {
    Some( o ) => o,
    None => legacy?,
  };
  let fetched_at = volatile.get( "fetched_at" )?.as_str()?.to_string();
  // Low-churn lookup: tracked top-level first (newer), legacy cache{} fallback.
  let low = | key : &str |
  {
    tracked.as_ref().and_then( | t | t.get( key ) )
      .or_else( || legacy.and_then( | l | l.get( key ) ) )
  };
  Some( QuotaCacheEntry
  {
    fetched_at,
    five_hour        : read_period( volatile, "five_hour" ),
    seven_day        : read_period( volatile, "seven_day" ),
    seven_day_sonnet : read_period( volatile, "seven_day_sonnet" ),
    model_override   : low( "model_override" ).and_then( | v | v.as_str() ).map( str::to_string ),
    last_touch_at    : low( "last_touch_at" ).and_then( | v | v.as_str() ).map( str::to_string ),
    touch_idle       : low( "touch_idle" ).and_then( serde_json::Value::as_bool ),
    // Fix(BUG-327): org_created_at written by write_cache_string(); defaults to
    //   None gracefully for stores that predate this field (T06).
    org_created_at   : low( "org_created_at" ).and_then( | v | v.as_str() ).map( str::to_string ),
  } )
}

/// Write a single low-churn metadata field as a top-level key of `{name}.json`
/// (read-merge-write).
///
/// Used by model override, touch persistence, and org-creation stamping — the
/// tracked metadata that must survive across hosts via git, unlike the volatile
/// quota cache which lives in the untracked local file (TSK-500).
#[ inline ]
pub fn write_cache_field(
  credential_store : &std::path::Path,
  name             : &str,
  key              : &str,
  value            : serde_json::Value,
)
{
  let meta_path = credential_store.join( format!( "{name}.json" ) );
  let mut snapshot = read_json_value( &meta_path ).unwrap_or_else( || serde_json::json!( {} ) );
  if let Some( obj ) = snapshot.as_object_mut()
  {
    obj.insert( key.to_string(), value );
  }
  let _ = atomic_write( &meta_path, &serde_json::to_string_pretty( &snapshot ).map( | s | s + "\n" ).unwrap_or_default() );
}

/// Write a string value into the cache object (typed convenience wrapper).
#[ inline ]
pub fn write_cache_string(
  credential_store : &std::path::Path,
  name             : &str,
  key              : &str,
  value            : &str,
)
{
  write_cache_field( credential_store, name, key, serde_json::Value::String( value.to_string() ) );
}

/// Write a bool value into the cache object (typed convenience wrapper).
#[ inline ]
pub fn write_cache_bool(
  credential_store : &std::path::Path,
  name             : &str,
  key              : &str,
  value            : bool,
)
{
  write_cache_field( credential_store, name, key, serde_json::Value::Bool( value ) );
}

/// Write a tracked metadata string only when its current value differs (TSK-500).
///
/// Keeps the steady-state fetch path at zero tracked writes: `org_created_at`
/// is stamped on every successful fetch, but its value practically never
/// changes — an unconditional write would re-dirty the tracked file every
/// sweep. Reads the top-level key first, a legacy `cache{}` block as
/// pre-migration fallback.
#[ inline ]
pub fn write_cache_string_if_changed(
  credential_store : &std::path::Path,
  name             : &str,
  key              : &str,
  value            : &str,
)
{
  let current = read_json_value( &credential_store.join( format!( "{name}.json" ) ) )
    .and_then( | t |
    {
      t.get( key )
        .or_else( || t.get( "cache" ).and_then( | c | c.get( key ) ) )
        .and_then( | v | v.as_str() )
        .map( str::to_string )
    } );
  if current.as_deref() == Some( value )
  {
    return;
  }
  write_cache_string( credential_store, name, key, value );
}

/// Build a period cache JSON value from utilization + optional `resets_at`.
// Fix(BUG-540, task/claude_profile_core registry):
// Root cause: the utilization value was serialized under the key `left_pct` —
//   the stored name asserted percent-REMAINING while the value is percent-CONSUMED,
//   inverting the meaning for every raw-JSON consumer (a 100%-burned quota read as
//   "100 left"). The symmetric reader made the inversion invisible from inside clp.
// Pitfall: never invert the stored VALUE to match the old name — history rings and
//   cross-host caches already hold utilization; the key rename preserves them.
fn period_json( utilization : f64, resets_at : Option< &str > ) -> serde_json::Value
{
  let mut m = serde_json::Map::new();
  m.insert( "utilization".into(), serde_json::Value::from( utilization ) );
  if let Some( r ) = resets_at
  {
    m.insert( "resets_at".into(), serde_json::Value::String( r.to_string() ) );
  }
  serde_json::Value::Object( m )
}

/// Extract a period tuple from a cache object.
///
/// Reads `utilization` first; falls back to the pre-BUG-540 `left_pct` key so
/// legacy cache files (own host or another host's subtree) stay readable —
/// both names always held the same utilization value, only the old name lied.
fn read_period( cache : &serde_json::Map< String, serde_json::Value >, key : &str ) -> Option< ( f64, Option< String > ) >
{
  let p = cache.get( key )?.as_object()?;
  let utilization = p.get( "utilization" ).or_else( || p.get( "left_pct" ) )?.as_f64()?;
  let resets_at = p.get( "resets_at" ).and_then( |v| v.as_str() ).map( str::to_string );
  Some( ( utilization, resets_at ) )
}

/// Parse an ISO-8601 UTC timestamp to seconds since epoch.
///
/// Accepts the format `YYYY-MM-DDTHH:MM:SSZ` as produced by `chrono_now_utc`.
/// Returns `None` on any parse failure.
#[ must_use ]
#[ inline ]
pub fn parse_iso_utc_secs( s : &str ) -> Option< u64 >
{
  if s.len() < 20 || !s.ends_with( 'Z' ) { return None; }
  let y : i64 = s[ 0..4 ].parse().ok()?;
  let m : i64 = s[ 5..7 ].parse().ok()?;
  let d : i64 = s[ 8..10 ].parse().ok()?;
  let hh : u64 = s[ 11..13 ].parse().ok()?;
  let mm : u64 = s[ 14..16 ].parse().ok()?;
  let ss : u64 = s[ 17..19 ].parse().ok()?;
  // Inverse of Hinnant: Y/M/D → days since epoch.
  let y2 = if m <= 2 { y - 1 } else { y };
  let era = if y2 >= 0 { y2 } else { y2 - 399 } / 400;
  let yoe = y2 - era * 400;
  let m2  = if m > 2 { m - 3 } else { m + 9 };
  let doy = ( 153 * m2 + 2 ) / 5 + d - 1;
  let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
  #[ allow( clippy::cast_sign_loss ) ]
  let days = ( era * 146_097 + doe - 719_468 ) as u64;
  Some( days * 86400 + hh * 3600 + mm * 60 + ss )
}

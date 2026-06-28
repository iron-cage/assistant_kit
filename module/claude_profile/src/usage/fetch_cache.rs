//! Centralized quota cache read with Feature 040 polynomial approximation.
//!
//! All paths that need cached utilization values must call `read_cached_quota`.
//! Direct `read_quota_cache()` calls are reserved for metadata-only callers
//! (e.g., touch-idle age hints) that don't need approximated utilization.

// ── Cache age ────────────────────────────────────────────────────────────────

/// Compute seconds elapsed since a `fetched_at` ISO-8601 UTC timestamp.
fn cache_age_from_fetched_at( fetched_at : &str ) -> u64
{
  let now  = std::time::SystemTime::now().duration_since( std::time::UNIX_EPOCH ).unwrap_or_default().as_secs();
  let then = claude_profile_core::account::parse_iso_utc_secs( fetched_at ).unwrap_or( now );
  now.saturating_sub( then )
}

// ── Centralized cache-read + approximation ───────────────────────────────────

/// Read quota cache and apply Feature 040 polynomial approximation when available.
///
/// Returns `None` when no cache entry exists for `name`. When `cache.history[]` has
/// ≥2 entries in the current window, applies `approximate_utilization()` for each
/// period independently (AC-04, AC-05 from `docs/feature/040_quota_measurement_history.md`).
///
/// Called by all three utilization cache-read paths: G1 (non-owned accounts), HTTP
/// error fallback, and `approximate_quota()`. Eliminates BUG-304 — each previous
/// caller duplicated the 40–55 line approximation block independently.
///
/// Fix(BUG-304): centralize cache-read + approximation in one function; all utilization
///   read paths call this function so approximation is never silently absent.
/// Root cause: three independent callers each reconstructed `OauthUsageData` from the
///   cache; G1 applied no approximation, HTTP fallback and `approximate_quota()` each
///   duplicated the ~50-line approximation block, creating divergence risk.
/// Pitfall: `read_quota_cache()` remains available for metadata-only reads (`touch_idle`,
///   age hints); this function is only for paths that need utilization values.
pub( crate ) fn read_cached_quota(
  credential_store : &std::path::Path,
  name             : &str,
  now_secs         : u64,
) -> Option< ( claude_quota::OauthUsageData, u64 /* cache_age_secs */ ) >
{
  let entry = claude_profile_core::account::read_quota_cache( credential_store, name )?;
  let age   = cache_age_from_fetched_at( &entry.fetched_at );
  let mut data = claude_quota::OauthUsageData
  {
    five_hour        : entry.five_hour.map( |( u, r )| claude_quota::PeriodUsage { utilization : u, resets_at : r } ),
    seven_day        : entry.seven_day.map( |( u, r )| claude_quota::PeriodUsage { utilization : u, resets_at : r } ),
    seven_day_sonnet : entry.seven_day_sonnet.map( |( u, r )| claude_quota::PeriodUsage { utilization : u, resets_at : r } ),
  };
  let history = claude_profile_core::account::read_history( credential_store, name );
  if history.len() >= 2
  {
    if let Some( ref mut fh ) = data.five_hour
    {
      let pts : Vec< ( u64, f64 ) > = history.iter()
        .filter_map( |m| m.h5.as_ref().map( |( u, _ )| ( m.t, *u ) ) )
        .collect();
      let resets = history.iter().rev()
        .find_map( |m| m.h5.as_ref().map( |( _, r )| r.as_str() ) )
        .and_then( claude_quota::iso_to_unix_secs );
      if let Some( v ) = super::approx::approximate_utilization( &pts, resets, 18_000, now_secs )
      {
        fh.utilization = v;
      }
    }
    if let Some( ref mut d7 ) = data.seven_day
    {
      let pts : Vec< ( u64, f64 ) > = history.iter()
        .filter_map( |m| m.d7.as_ref().map( |( u, _ )| ( m.t, *u ) ) )
        .collect();
      let resets = history.iter().rev()
        .find_map( |m| m.d7.as_ref().map( |( _, r )| r.as_str() ) )
        .and_then( claude_quota::iso_to_unix_secs );
      if let Some( v ) = super::approx::approximate_utilization( &pts, resets, 604_800, now_secs )
      {
        d7.utilization = v;
      }
    }
    if let Some( ref mut sn ) = data.seven_day_sonnet
    {
      let pts : Vec< ( u64, f64 ) > = history.iter()
        .filter_map( |m| m.sn.as_ref().map( |( u, _ )| ( m.t, *u ) ) )
        .collect();
      let resets = history.iter().rev()
        .find_map( |m| m.sn.as_ref().map( |( _, r )| r.as_str() ) )
        .and_then( claude_quota::iso_to_unix_secs );
      if let Some( v ) = super::approx::approximate_utilization( &pts, resets, 604_800, now_secs )
      {
        sn.utilization = v;
      }
    }
  }
  Some( ( data, age ) )
}

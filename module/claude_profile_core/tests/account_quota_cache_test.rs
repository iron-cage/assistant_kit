//! Quota-cache tests: per-host cache tree writes and merged reads (TSK-500/TSK-502)
//! and the Feature 040 measurement-history ring.
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `history_append_stores_correct_fields` | FT-01: write_history_entry stores t/h5/d7/sn in the history ring (local cache file since TSK-500) |
//! | `history_ring_buffer_evicts_oldest` | FT-02: 11th append evicts entry 0; length stays 10 |
//! | `history_read_absent_key_returns_empty` | FT-11: absent history key → empty vec (AC-11 backward compat) |
//! | `history_duplicate_timestamp_overwrites` | FT-13: same-second append overwrites last entry, not appends |
//! | `t500_01_fetch_write_sequence_leaves_migrated_tracked_byte_identical` | TSK-500/T01: write_quota_cache + write_history_entry leave a migrated tracked file byte-identical (AF1 hash compare) |
//! | `t502_01_volatile_cache_lands_in_per_host_tracked_file` | TSK-502/T01: volatile cache written to `cache/{host}_{user}/{name}.json` — no hyphen-prefixed component, so tracked; no account file created; merged read round-trips |
//! | `t500_02b_history_entry_lands_local_only` | TSK-500/T02: write_history_entry targets the per-host cache file only (TSK-502 location) |
//! | `t500_05_low_churn_writes_land_top_level_tracked` | TSK-500/T05: write_cache_string/bool land top-level tracked; no cache{} block recreated; merged read surfaces both |
//! | `t500_06_legacy_cache_migrates_prunes_and_preserves` | TSK-500/T06: legacy cache{} readable pre-migration, dissolved in one write (volatile→per-host file since TSK-502, low-churn→top-level, AF2 cache-key absent) |
//! | `t500_01b_if_changed_write_skips_identical_value` | TSK-500/T01: write_cache_string_if_changed skips the tracked write when the value is unchanged (steady-state org_created_at stamp) |
//! | `t502_02_read_merges_freshest_across_host_subtrees` | TSK-502/T02+T03: freshest fetched_at wins across cache/*/ subtrees in both directions; an unparseable fetched_at candidate is skipped |
//! | `t502_03_legacy_gitignored_cache_read_then_self_cleaned` | TSK-502/T04: legacy `-cache/{name}.json` readable as fallback; the next write relocates values + history to the per-host file and deletes it |
//! | `t502_04_no_cache_anywhere_returns_none` | TSK-502/T05 (regression guard): empty store → read_quota_cache None — the no-cache contract unchanged |
//! | `t502_05_history_ring_continues_across_hosts` | TSK-502/T06: another host's ring is carried into the own-host file by write_quota_cache and continued by write_history_entry |
//! | `bug_540_period_key_is_utilization_not_left_pct` | BUG-540: serialized period key is `utilization` (what the value is), never the inverted `left_pct` |
//! | `bug_540_legacy_left_pct_cache_file_still_reads` | BUG-540: legacy cache files carrying `left_pct` stay readable via the dual-key reader |

use tempfile::TempDir;
use claude_profile_core::account;

// ── Quota cache (Feature 033) ────────────────────────────────────────────────

/// AC-01: `write_quota_cache` targets this host's per-host cache file
/// (TSK-502 location) and leaves the metadata `{name}.json` byte-identical.
///
/// Given: `alice@acme.com.json` containing `{"host":"wbox","role":"dev"}` (no legacy `cache{}`)
/// When: `write_quota_cache` called with `five_hour` utilization 14.0
/// Then: metadata file unchanged; `cache/{host}_{user}/alice@acme.com.json` holds `fetched_at` + periods
#[ test ]
fn cache_write_preserves_existing_fields()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "alice@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write( &meta, r#"{"host":"wbox","role":"dev"}"# ).unwrap();
  let before = std::fs::read( &meta ).unwrap();

  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 14.0, Some( "2026-06-07T12:00:00Z" ) ) ),
    Some( ( 25.0, None ) ),
    None,
  );

  assert_eq!(
    std::fs::read( &meta ).unwrap(), before,
    "tracked file must stay byte-identical on a migrated store",
  );
  let local   = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  let content = std::fs::read_to_string( &local ).expect( "local cache file must exist" );
  assert!( content.contains( r#""fetched_at""# ), "fetched_at present: {content}" );
  assert!( content.contains( r#""utilization""# ), "utilization present: {content}" );
  assert!( content.contains( r#""five_hour""# ), "five_hour present: {content}" );
  assert!( content.contains( r#""seven_day""# ), "seven_day present: {content}" );
}

/// AC-02: `read_quota_cache` returns `None` when no cache exists.
///
/// Given: `{name}.json` with `{"host":"wbox"}` but no `"cache"` key
/// When: `read_quota_cache` called
/// Then: returns `None`
#[ test ]
fn cache_read_returns_none_when_absent()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "bob@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write( &meta, r#"{"host":"wbox"}"# ).unwrap();

  let result = claude_profile_core::account::read_quota_cache( store.path(), name );
  assert!( result.is_none(), "no cache key must return None" );
}

/// AC-02: `read_quota_cache` returns cached data when valid cache exists.
///
/// Given: `{name}.json` with a fully populated `"cache"` object
/// When: `read_quota_cache` called
/// Then: returns `Some(QuotaCacheEntry)` with all fields matching
#[ test ]
fn cache_read_returns_entry_when_present()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "carol@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write( &meta, r#"{"cache":{"fetched_at":"2026-06-07T10:00:00Z","status":"ok","five_hour":{"left_pct":86.0,"resets_at":"2026-06-07T15:00:00Z"},"seven_day":{"left_pct":42.5},"model_override":"opus","last_touch_at":"2026-06-07T09:55:00Z","touch_idle":false}}"# ).unwrap();

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "valid cache must return Some" );
  assert_eq!( entry.fetched_at, "2026-06-07T10:00:00Z" );
  let ( h5_util, h5_reset ) = entry.five_hour.expect( "five_hour must be Some" );
  assert!( ( h5_util - 86.0 ).abs() < f64::EPSILON, "five_hour utilization: {h5_util}" );
  assert_eq!( h5_reset.as_deref(), Some( "2026-06-07T15:00:00Z" ) );
  let ( d7_util, d7_reset ) = entry.seven_day.expect( "seven_day must be Some" );
  assert!( ( d7_util - 42.5 ).abs() < f64::EPSILON, "seven_day utilization: {d7_util}" );
  assert!( d7_reset.is_none(), "seven_day resets_at must be None" );
  assert!( entry.seven_day_sonnet.is_none(), "seven_day_sonnet must be None" );
  assert_eq!( entry.model_override.as_deref(), Some( "opus" ) );
  assert_eq!( entry.last_touch_at.as_deref(), Some( "2026-06-07T09:55:00Z" ) );
  assert_eq!( entry.touch_idle, Some( false ) );
}

/// AC-07: `write_quota_cache` round-trips through `read_quota_cache`.
///
/// Given: empty credential store
/// When: write then read
/// Then: read returns the same values written
#[ test ]
fn cache_write_read_roundtrip()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "rt@test.com";

  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 50.0, Some( "2026-06-07T18:00:00Z" ) ) ),
    None,
    Some( ( 90.0, Some( "2026-06-14T00:00:00Z" ) ) ),
  );

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "written cache must be readable" );
  let ( h5, h5r ) = entry.five_hour.expect( "five_hour present" );
  assert!( ( h5 - 50.0 ).abs() < f64::EPSILON );
  assert_eq!( h5r.as_deref(), Some( "2026-06-07T18:00:00Z" ) );
  assert!( entry.seven_day.is_none() );
  let ( sn, snr ) = entry.seven_day_sonnet.expect( "sonnet present" );
  assert!( ( sn - 90.0 ).abs() < f64::EPSILON );
  assert_eq!( snr.as_deref(), Some( "2026-06-14T00:00:00Z" ) );
}

/// `parse_iso_utc_secs` correctly converts known timestamps.
#[ test ]
fn parse_iso_utc_secs_known_values()
{
  // 2026-06-07T12:00:00Z = a known date, verify deterministic output.
  let secs = claude_profile_core::account::parse_iso_utc_secs( "2026-06-07T12:00:00Z" );
  assert!( secs.is_some(), "valid ISO must parse" );
  let s = secs.unwrap();
  // Cross-check: 2026-06-07 is day index from epoch; rough range 1780000000..1790000000
  assert!( s > 1_780_000_000, "must be in 2026 range: {s}" );
  assert!( s < 1_790_000_000, "must be in 2026 range: {s}" );

  // Invalid inputs return None.
  assert!( claude_profile_core::account::parse_iso_utc_secs( "short" ).is_none() );
  assert!( claude_profile_core::account::parse_iso_utc_secs( "not-a-date-at-all!" ).is_none() );
}

/// AC-05: `write_cache_string` persists a field in the cache sub-object.
#[ test ]
fn cache_field_string_persisted()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "field@test.com";
  // Pre-populate with cache.
  claude_profile_core::account::write_quota_cache(
    store.path(), name, Some( ( 10.0, None ) ), None, None,
  );
  // Write model_override field.
  claude_profile_core::account::write_cache_string( store.path(), name, "model_override", "opus" );

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable" );
  assert_eq!( entry.model_override.as_deref(), Some( "opus" ) );
  // Quota data must survive the field write.
  assert!( entry.five_hour.is_some(), "five_hour must survive write_cache_string" );
}

/// AC-06: `write_cache_bool` persists a boolean in the cache sub-object.
#[ test ]
fn cache_field_bool_persisted()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "bool@test.com";
  claude_profile_core::account::write_quota_cache(
    store.path(), name, Some( ( 20.0, None ) ), None, None,
  );
  claude_profile_core::account::write_cache_bool( store.path(), name, "touch_idle", false );

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable" );
  assert_eq!( entry.touch_idle, Some( false ) );
  assert!( entry.five_hour.is_some(), "five_hour must survive write_cache_bool" );
}

// ── Cache corner cases ────────────────────────────────────────────────────────

/// `read_quota_cache` returns `None` when `{name}.json` does not exist.
#[ test ]
fn cache_read_none_when_file_absent()
{
  let store = tempfile::tempdir().unwrap();
  let result = claude_profile_core::account::read_quota_cache( store.path(), "ghost@test.com" );
  assert!( result.is_none(), "absent file must return None" );
}

/// `read_quota_cache` returns `None` when `{name}.json` contains malformed JSON.
#[ test ]
fn cache_read_none_when_json_malformed()
{
  let store = tempfile::tempdir().unwrap();
  let meta  = store.path().join( "bad@test.com.json" );
  std::fs::write( &meta, "{not valid json!!!}" ).unwrap();
  let result = claude_profile_core::account::read_quota_cache( store.path(), "bad@test.com" );
  assert!( result.is_none(), "malformed JSON must return None" );
}

/// `read_quota_cache` returns `None` when cache object has no `fetched_at` key.
#[ test ]
fn cache_read_none_when_fetched_at_missing()
{
  let store = tempfile::tempdir().unwrap();
  let meta  = store.path().join( "notime@test.com.json" );
  std::fs::write( &meta, r#"{"cache":{"status":"ok","five_hour":{"left_pct":50.0}}}"# ).unwrap();
  let result = claude_profile_core::account::read_quota_cache( store.path(), "notime@test.com" );
  assert!( result.is_none(), "cache without fetched_at must return None" );
}

/// `write_quota_cache` preserves `model_override` written by a prior `write_cache_string`.
///
/// The quota write copies side-effect fields (`model_override`, `last_touch_at`, `touch_idle`)
/// from the previous cache object into the new one (account.rs:1207-1212 at fix time; now in `quota_cache.rs`).
#[ test ]
fn cache_write_preserves_prior_side_effects()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "preserve@test.com";
  // Step 1: write side-effect fields via cache field API.
  claude_profile_core::account::write_cache_string( store.path(), name, "model_override", "opus" );
  claude_profile_core::account::write_cache_string( store.path(), name, "last_touch_at", "2026-06-07T09:00:00Z" );
  claude_profile_core::account::write_cache_bool( store.path(), name, "touch_idle", true );
  // Step 2: write quota cache — must preserve all three side-effect fields.
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 30.0, Some( "2026-06-07T20:00:00Z" ) ) ),
    None,
    None,
  );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable" );
  assert_eq!( entry.model_override.as_deref(), Some( "opus" ), "model_override must survive" );
  assert_eq!( entry.last_touch_at.as_deref(), Some( "2026-06-07T09:00:00Z" ), "last_touch_at must survive" );
  assert_eq!( entry.touch_idle, Some( true ), "touch_idle must survive" );
  assert!( entry.five_hour.is_some(), "quota data must be present" );
}

/// `write_cache_field` creates `{name}.json` from scratch when file is absent.
#[ test ]
fn cache_field_creates_file_from_scratch()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "scratch@test.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  assert!( !meta.exists(), "pre-condition: file must not exist" );
  claude_profile_core::account::write_cache_string( store.path(), name, "model_override", "sonnet" );
  assert!( meta.exists(), "write_cache_string must create file" );
  let content = std::fs::read_to_string( &meta ).unwrap();
  assert!( content.contains( r#""model_override": "sonnet""# ), "field must be in file: {content}" );
  // read_quota_cache returns None because no fetched_at.
  assert!(
    claude_profile_core::account::read_quota_cache( store.path(), name ).is_none(),
    "cache without fetched_at must return None even after write_cache_field",
  );
}

// ── TSK-500: quota cache externalized to local untracked file ─────────────────

/// T01 (TSK-500): the fetch-success write sequence leaves a migrated tracked file
/// byte-identical (AF1: hash compare, not `git status` silence).
///
/// Sequence mirrors `fetch.rs`'s success path: `write_quota_cache` then
/// `write_history_entry`. Store is post-migration: low-churn fields top-level,
/// no legacy `cache{}` block. Zero tracked writes is the defining property.
#[ test ]
fn t500_01_fetch_write_sequence_leaves_migrated_tracked_byte_identical()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "owner@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write(
    &meta,
    r#"{"host":"wbox","org_created_at":"2025-11-30T00:00:00Z","model_override":"opus"}"#,
  ).unwrap();
  let before = std::fs::read( &meta ).unwrap();

  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 42.0, Some( "2026-08-16T18:00:00Z" ) ) ),
    Some( ( 30.0, None ) ),
    None,
  );
  claude_profile_core::account::write_history_entry(
    store.path(), name, 1_755_360_000,
    Some( ( 42.0, "2026-08-16T18:00:00Z" ) ),
    None,
    None,
  );

  assert_eq!(
    std::fs::read( &meta ).unwrap(), before,
    "T01: tracked credential file must be byte-identical after the fetch write sequence",
  );
}

/// T01 (TSK-500): `write_cache_string_if_changed` skips the tracked write when the
/// value is unchanged — the steady-state `org_created_at` stamp on every successful
/// fetch must not re-dirty the tracked file — and still writes on a real change.
#[ test ]
fn t500_01b_if_changed_write_skips_identical_value()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "stamp@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write( &meta, r#"{"org_created_at":"2025-11-30T00:00:00Z"}"# ).unwrap();
  let before = std::fs::read( &meta ).unwrap();

  claude_profile_core::account::write_cache_string_if_changed(
    store.path(), name, "org_created_at", "2025-11-30T00:00:00Z",
  );
  assert_eq!(
    std::fs::read( &meta ).unwrap(), before,
    "identical value must be a zero-write no-op (byte-identical tracked file)",
  );

  claude_profile_core::account::write_cache_string_if_changed(
    store.path(), name, "org_created_at", "2026-01-01T00:00:00Z",
  );
  let json : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &meta ).unwrap() ).unwrap();
  assert_eq!(
    json[ "org_created_at" ].as_str(), Some( "2026-01-01T00:00:00Z" ),
    "changed value must be written",
  );
}

/// T01 (TSK-502, supersedes TSK-500/T02's location assertion): volatile cache
/// lands in the tracked per-host file `cache/{host}_{user}/{name}.json` — no
/// path component is hyphen-prefixed, so the global `-*` gitignore rule cannot
/// match it — and no tracked account file is created.
#[ test ]
fn t502_01_volatile_cache_lands_in_per_host_tracked_file()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "fresh@acme.com";

  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 55.5, Some( "2026-08-16T20:00:00Z" ) ) ),
    None,
    None,
  );

  let local = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  assert!( local.is_file(), "T01: cache/{}/{name}.json must exist", host_slug() );
  for component in local.strip_prefix( store.path() ).unwrap().components()
  {
    let c = component.as_os_str().to_string_lossy();
    assert!(
      !c.starts_with( '-' ),
      "T01: no cache path component may be hyphen-prefixed (would be gitignored by the global -* rule): {c}",
    );
  }
  let json : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &local ).unwrap() ).unwrap();
  assert!( json[ "fetched_at" ].is_string(), "T01: per-host cache carries fetched_at" );
  let u = json[ "five_hour" ][ "utilization" ].as_f64().expect( "utilization" );
  assert!( ( u - 55.5 ).abs() < 1e-9, "T01: utilization stored per-host, got {u}" );
  assert!(
    !store.path().join( format!( "{name}.json" ) ).exists(),
    "T01: write_quota_cache must not create a tracked account file",
  );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T01: merged read must see the per-host cache" );
  let ( util, reset ) = entry.five_hour.expect( "five_hour present" );
  assert!( ( util - 55.5 ).abs() < f64::EPSILON );
  assert_eq!( reset.as_deref(), Some( "2026-08-16T20:00:00Z" ) );
}

/// T02 (TSK-500): `write_history_entry` targets the per-host cache file
/// (TSK-502 location) — the ring buffer never touches (or creates) the tracked
/// `{name}.json`.
#[ test ]
fn t500_02b_history_entry_lands_local_only()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "hist@acme.com";

  claude_profile_core::account::write_history_entry(
    store.path(), name, 2_000,
    Some( ( 40.0, "2026-08-16T20:00:00Z" ) ),
    None,
    None,
  );

  let local = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  let json : serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string( &local ).expect( "local cache file must exist" )
  ).unwrap();
  assert_eq!(
    json[ "history" ].as_array().map( Vec::len ), Some( 1 ),
    "T02: history entry stored in the local file",
  );
  assert!(
    !store.path().join( format!( "{name}.json" ) ).exists(),
    "T02: write_history_entry must not create a tracked file",
  );
  let entries = claude_profile_core::account::read_history( store.path(), name );
  assert_eq!( entries.len(), 1, "read_history must read the local ring" );
  assert_eq!( entries[ 0 ].t, 2_000 );
}

/// T05 (TSK-500): low-churn writes (`write_cache_string`/`write_cache_bool`) land
/// as top-level keys of the tracked file — never under a `cache{}` block — and the
/// merged reader surfaces them alongside local volatile data.
#[ test ]
fn t500_05_low_churn_writes_land_top_level_tracked()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "meta@acme.com";
  std::fs::write( store.path().join( format!( "{name}.json" ) ), r#"{"host":"wbox"}"# ).unwrap();

  claude_profile_core::account::write_quota_cache(
    store.path(), name, Some( ( 10.0, None ) ), None, None,
  );
  claude_profile_core::account::write_cache_string( store.path(), name, "model_override", "opus" );
  claude_profile_core::account::write_cache_bool( store.path(), name, "touch_idle", false );

  let json : serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string( store.path().join( format!( "{name}.json" ) ) ).unwrap()
  ).unwrap();
  assert_eq!( json[ "model_override" ].as_str(), Some( "opus" ), "T05: model_override top-level tracked" );
  assert_eq!( json[ "touch_idle" ].as_bool(), Some( false ), "T05: touch_idle top-level tracked" );
  assert!( json.get( "cache" ).is_none(), "T05: no cache{{}} block may be (re)created: {json}" );
  assert_eq!( json[ "host" ].as_str(), Some( "wbox" ), "T05: unrelated fields preserved" );

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "merged read must succeed" );
  assert_eq!( entry.model_override.as_deref(), Some( "opus" ) );
  assert_eq!( entry.touch_idle, Some( false ) );
  assert!( entry.five_hour.is_some(), "volatile (local) + low-churn (tracked) must merge" );
}

/// T06 (TSK-500): first post-upgrade write migrates a legacy `cache{}` block —
/// volatile fields move to the local file, low-churn fields relocate to top level,
/// and the `cache` key is pruned from the tracked JSON in one write.
///
/// AF2 — all three legs asserted: legacy readable pre-migration AND volatile
/// present locally post-migration AND `cache` key absent from tracked JSON.
#[ test ]
fn t500_06_legacy_cache_migrates_prunes_and_preserves()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "legacy@acme.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write(
    &meta,
    r#"{"host":"wbox","cache":{"fetched_at":"2026-08-01T00:00:00Z","status":"ok","five_hour":{"left_pct":80.0},"history":[{"t":1000,"h5":[80.0,"2026-08-01T05:00:00Z"],"d7":null,"sn":null}],"model_override":"opus","org_created_at":"2025-11-30T00:00:00Z"}}"#,
  ).unwrap();

  // AF2 leg 1: legacy values readable BEFORE migration (fallback window).
  let pre = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T06: legacy cache{} must be readable pre-migration" );
  assert_eq!( pre.fetched_at, "2026-08-01T00:00:00Z", "T06: legacy fetched_at honored" );
  assert_eq!( pre.org_created_at.as_deref(), Some( "2025-11-30T00:00:00Z" ) );
  assert_eq!(
    claude_profile_core::account::read_history( store.path(), name ).len(), 1,
    "T06: legacy history readable pre-migration",
  );

  // First post-upgrade quota write triggers the migration.
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 60.0, Some( "2026-08-16T21:00:00Z" ) ) ),
    None,
    None,
  );

  // AF2 leg 3: `cache` key absent from tracked; low-churn relocated top-level.
  let tracked : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &meta ).unwrap() ).unwrap();
  assert!( tracked.get( "cache" ).is_none(), "T06/AF2: cache key pruned from tracked JSON: {tracked}" );
  assert_eq!(
    tracked[ "org_created_at" ].as_str(), Some( "2025-11-30T00:00:00Z" ),
    "T06: org_created_at retained tracked (TSK-368 not regressed)",
  );
  assert_eq!( tracked[ "model_override" ].as_str(), Some( "opus" ), "T06: model_override relocated top-level" );
  assert_eq!( tracked[ "host" ].as_str(), Some( "wbox" ), "T06: unrelated fields preserved" );

  // AF2 leg 2: volatile + history present in the per-host file post-migration.
  let local = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  let ljson : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &local ).unwrap() ).unwrap();
  let u = ljson[ "five_hour" ][ "utilization" ].as_f64().expect( "utilization" );
  assert!( ( u - 60.0 ).abs() < 1e-9, "T06: new fetch values in local file, got {u}" );
  assert_eq!(
    ljson[ "history" ].as_array().map( Vec::len ), Some( 1 ),
    "T06: legacy history carried into the local file",
  );

  // Merged read: new volatile + preserved low-churn.
  let post = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "post-migration read must succeed" );
  let ( util, _ ) = post.five_hour.expect( "five_hour" );
  assert!( ( util - 60.0 ).abs() < f64::EPSILON );
  assert_eq!( post.org_created_at.as_deref(), Some( "2025-11-30T00:00:00Z" ) );
  assert_eq!( post.model_override.as_deref(), Some( "opus" ) );

  // Steady state: a second write leaves tracked byte-identical (T01 property).
  let before = std::fs::read( &meta ).unwrap();
  claude_profile_core::account::write_quota_cache( store.path(), name, Some( ( 61.0, None ) ), None, None );
  assert_eq!(
    std::fs::read( &meta ).unwrap(), before,
    "T06/T01: second write must leave tracked byte-identical",
  );
}

// ── TSK-502: per-host tracked quota cache tree ────────────────────────────────

/// Expected per-host cache subdirectory name — derived from the shipped
/// active-marker filename so the test can never drift from the slug
/// sanitization actually used by the write path.
fn host_slug() -> String
{
  claude_profile_core::account::active_marker_filename()
    .strip_prefix( "_active_" )
    .expect( "marker filename always starts with _active_" )
    .to_string()
}

/// Seed a raw volatile cache object into a named host subtree.
fn seed_host_cache(
  store      : &std::path::Path,
  host_dir   : &str,
  name       : &str,
  fetched_at : &str,
  left_pct   : f64,
)
{
  let dir = store.join( "cache" ).join( host_dir );
  std::fs::create_dir_all( &dir ).unwrap();
  std::fs::write(
    dir.join( format!( "{name}.json" ) ),
    format!( r#"{{"fetched_at":"{fetched_at}","status":"ok","five_hour":{{"left_pct":{left_pct}}}}}"# ),
  ).unwrap();
}

/// T02+T03 (TSK-502): the merged read returns the freshest `fetched_at` across
/// host subtrees — in both directions — and a candidate whose `fetched_at`
/// does not parse is skipped instead of winning or aborting the merge.
#[ test ]
fn t502_02_read_merges_freshest_across_host_subtrees()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "fleet@acme.com";
  seed_host_cache( store.path(), "w001_user1", name, "2026-01-01T00:00:00Z", 10.0 );
  seed_host_cache( store.path(), "w002_user1", name, "2026-01-02T00:00:00Z", 20.0 );
  // T03: lexicographically huge but unparseable timestamp must never win.
  seed_host_cache( store.path(), "w009_user1", name, "not-a-timestamp", 99.0 );

  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T02: merged read must see the host subtrees" );
  let ( u, _ ) = entry.five_hour.expect( "five_hour present" );
  assert!( ( u - 20.0 ).abs() < f64::EPSILON, "T02: fresher w002 entry must win, got {u}" );
  assert_eq!( entry.fetched_at, "2026-01-02T00:00:00Z" );

  // Direction flip: w001 becomes the freshest.
  seed_host_cache( store.path(), "w001_user1", name, "2026-01-03T00:00:00Z", 30.0 );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T02: merged read after flip" );
  let ( u, _ ) = entry.five_hour.expect( "five_hour present" );
  assert!( ( u - 30.0 ).abs() < f64::EPSILON, "T02: freshest must flip to w001, got {u}" );
}

/// T04 (TSK-502): a legacy gitignored `-cache/{name}.json` (pre-502 layout) is
/// readable as fallback, and the next `write_quota_cache` relocates its role to
/// the per-host tracked file — carrying the history ring — and deletes it.
#[ test ]
fn t502_03_legacy_gitignored_cache_read_then_self_cleaned()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "legacylocal@acme.com";
  let legacy_dir = store.path().join( "-cache" );
  std::fs::create_dir_all( &legacy_dir ).unwrap();
  let legacy = legacy_dir.join( format!( "{name}.json" ) );
  std::fs::write(
    &legacy,
    r#"{"fetched_at":"2026-01-01T00:00:00Z","status":"ok","five_hour":{"left_pct":70.0},"history":[{"t":1000,"h5":[70.0,"2026-01-01T05:00:00Z"],"d7":null,"sn":null}]}"#,
  ).unwrap();

  let pre = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T04: legacy gitignored cache must be readable as fallback" );
  let ( u, _ ) = pre.five_hour.expect( "five_hour present" );
  assert!( ( u - 70.0 ).abs() < f64::EPSILON, "T04: legacy value honored, got {u}" );

  claude_profile_core::account::write_quota_cache(
    store.path(), name, Some( ( 71.0, None ) ), None, None,
  );

  assert!( !legacy.exists(), "T04: legacy -cache file must be deleted after a successful per-host write" );
  let per_host = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  let json : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &per_host ).unwrap() ).unwrap();
  assert_eq!(
    json[ "history" ].as_array().map( Vec::len ), Some( 1 ),
    "T04/AF2: legacy history must be carried into the per-host file",
  );
  let post = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "T04: post-migration read" );
  let ( u, _ ) = post.five_hour.expect( "five_hour present" );
  assert!( ( u - 71.0 ).abs() < f64::EPSILON, "T04: per-host file serves reads, got {u}" );
}

/// T05 (TSK-502, regression guard — passed before the change too): a store with
/// no cache in any location returns `None`, the unchanged no-cache contract.
#[ test ]
fn t502_04_no_cache_anywhere_returns_none()
{
  let store = tempfile::tempdir().unwrap();
  assert!(
    claude_profile_core::account::read_quota_cache( store.path(), "nobody@acme.com" ).is_none(),
    "T05: empty store must read as no cache",
  );
}

/// T06 (TSK-502): a history ring living in another host's subtree is carried
/// into the own-host file by `write_quota_cache` and continued — not restarted —
/// by `write_history_entry`.
#[ test ]
fn t502_05_history_ring_continues_across_hosts()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "ring@acme.com";
  let other = store.path().join( "cache" ).join( "otherbox_op" );
  std::fs::create_dir_all( &other ).unwrap();
  std::fs::write(
    other.join( format!( "{name}.json" ) ),
    r#"{"fetched_at":"2026-01-01T00:00:00Z","status":"ok","history":[{"t":1000,"h5":[80.0,"2026-01-01T05:00:00Z"],"d7":null,"sn":null},{"t":2000,"h5":[75.0,"2026-01-01T05:00:00Z"],"d7":null,"sn":null}]}"#,
  ).unwrap();

  claude_profile_core::account::write_quota_cache(
    store.path(), name, Some( ( 60.0, None ) ), None, None,
  );
  let own = store.path().join( "cache" ).join( host_slug() ).join( format!( "{name}.json" ) );
  let json : serde_json::Value = serde_json::from_str( &std::fs::read_to_string( &own ).unwrap() ).unwrap();
  assert_eq!(
    json[ "history" ].as_array().map( Vec::len ), Some( 2 ),
    "T06: other host's ring must be carried into the own-host file",
  );

  claude_profile_core::account::write_history_entry(
    store.path(), name, 3_000, Some( ( 60.0, "2026-01-02T05:00:00Z" ) ), None, None,
  );
  let entries = claude_profile_core::account::read_history( store.path(), name );
  assert_eq!( entries.len(), 3, "T06: ring continued (2 carried + 1 appended)" );
  assert_eq!( entries[ 0 ].t, 1_000 );
  assert_eq!( entries[ 2 ].t, 3_000 );
}

/// Second `write_quota_cache` replaces first period data.
#[ test ]
fn cache_write_second_replaces_first()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "overwrite@test.com";
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 10.0, Some( "2026-06-07T12:00:00Z" ) ) ),
    None,
    None,
  );
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 90.0, Some( "2026-06-07T18:00:00Z" ) ) ),
    Some( ( 50.0, None ) ),
    None,
  );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable" );
  let ( h5, h5r ) = entry.five_hour.expect( "five_hour must be present" );
  assert!( ( h5 - 90.0 ).abs() < f64::EPSILON, "five_hour must be from second write: {h5}" );
  assert_eq!( h5r.as_deref(), Some( "2026-06-07T18:00:00Z" ) );
  assert!( entry.seven_day.is_some(), "seven_day from second write must be present" );
}

/// All three periods written and read back simultaneously.
#[ test ]
fn cache_write_read_all_three_periods()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "all3@test.com";
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 14.0, Some( "2026-06-07T12:00:00Z" ) ) ),
    Some( ( 25.0, Some( "2026-06-14T00:00:00Z" ) ) ),
    Some( ( 100.0, None ) ),
  );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable" );
  let ( h5, _ ) = entry.five_hour.expect( "five_hour present" );
  assert!( ( h5 - 14.0 ).abs() < f64::EPSILON );
  let ( d7, _ ) = entry.seven_day.expect( "seven_day present" );
  assert!( ( d7 - 25.0 ).abs() < f64::EPSILON );
  let ( sn, sn_r ) = entry.seven_day_sonnet.expect( "sonnet present" );
  assert!( ( sn - 100.0 ).abs() < f64::EPSILON, "100.0 utilization boundary" );
  assert!( sn_r.is_none(), "sonnet resets_at must be None" );
}

/// `chrono_now_utc` output is parseable by `parse_iso_utc_secs` (round-trip).
#[ test ]
fn chrono_now_utc_parse_roundtrip()
{
  let ts   = claude_profile_core::account::chrono_now_utc();
  let secs = claude_profile_core::account::parse_iso_utc_secs( &ts )
    .expect( "chrono_now_utc output must be parseable" );
  let now  = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap_or_default()
    .as_secs();
  assert!(
    now.abs_diff( secs ) <= 2,
    "round-trip must be within 2 seconds of wall clock: now={now}, parsed={secs}",
  );
}

/// `trace_ts()` returns a UTC-marked timestamp prefix (BUG-338).
///
/// ## Fix Documentation — BUG-338
///
/// - **Root Cause:** `trace_ts()` sliced `chrono_now_utc()`'s ISO-8601 output into
///   `"YYYY-MM-DD · HH:MM:SS · "`, dropping the trailing `Z` (UTC marker) entirely.
///   The rendered prefix was visually indistinguishable from a differently-clocked
///   (e.g. local-time) timestamp source sharing the same shape in a combined transcript.
/// - **Why Not Caught:** No test asserted on `trace_ts()`'s own return value — only
///   `chrono_now_utc()` (the function it wraps) had a round-trip test. The slicing
///   step that drops the `Z` had no dedicated coverage.
/// - **Fix Applied:** `format!` literal changed from `"{} · {} · "` to `"{} · {} UTC · "`,
///   restoring an explicit timezone marker in place of the dropped `Z`.
/// - **Prevention:** This test asserts both the substring and the full structural shape,
///   so a future slicing change that drops the marker again fails immediately.
/// - **Pitfall:** A bare `.contains("UTC")` check would pass even if `UTC` appeared in
///   the wrong position (e.g. before the date) — the structural check below pins the
///   exact position, ensuring the marker sits between time and trailer.
#[ test ]
fn trace_ts_returns_utc_marked_timestamp()
{
  let ts = claude_profile_core::account::trace_ts();

  assert!( ts.contains( " UTC · " ), "must contain UTC marker substring: {ts}" );

  // Structural check (AF1): validate the full shape, not just substring presence.
  let mut parts = ts.splitn( 2, " · " );
  let date_part = parts.next().expect( "date segment present" );
  let rest       = parts.next().expect( "time+marker segment present" );

  assert_eq!( date_part.len(), 10, "date segment must be YYYY-MM-DD: {date_part}" );
  assert!( date_part.chars().enumerate().all( |( i, c )| if i == 4 || i == 7 { c == '-' } else { c.is_ascii_digit() } ), "date segment must be YYYY-MM-DD: {date_part}" );

  assert_eq!( rest, format!( "{} UTC · ", &rest[ ..8 ] ), "time+marker segment must be HH:MM:SS UTC · : {rest}" );
  let time_part = &rest[ ..8 ];
  assert!( time_part.chars().enumerate().all( |( i, c )| if i == 2 || i == 5 { c == ':' } else { c.is_ascii_digit() } ), "time segment must be HH:MM:SS: {time_part}" );
}

/// `write_quota_cache` gracefully handles malformed existing `{name}.json`.
///
/// When the file contains invalid JSON, `serde_json::from_str` returns Err
/// and the code falls back to an empty object. The cache is written to a fresh
/// JSON — non-cache fields (host, role) in the malformed file are lost.
#[ test ]
fn cache_write_recovers_from_malformed_json()
{
  let store = tempfile::tempdir().unwrap();
  let name  = "recover@test.com";
  let meta  = store.path().join( format!( "{name}.json" ) );
  std::fs::write( &meta, "NOT VALID JSON AT ALL" ).unwrap();
  claude_profile_core::account::write_quota_cache(
    store.path(), name,
    Some( ( 45.0, None ) ),
    None,
    None,
  );
  let entry = claude_profile_core::account::read_quota_cache( store.path(), name )
    .expect( "cache must be readable after recovery" );
  let ( h5, _ ) = entry.five_hour.expect( "five_hour must be present" );
  assert!( ( h5 - 45.0 ).abs() < f64::EPSILON, "five_hour utilization: {h5}" );
}


// ── Feature 040: Measurement history storage ──────────────────────────────────

/// FT-01 (AC-01): `write_history_entry()` stores correct `t`, `h5`, `d7`, `sn` fields.
///
/// # Given
/// Account `alice` has `alice.json` with an empty `cache.history[]` array.
/// A successful quota fetch returned utilization values for all three periods.
/// # When
/// `write_history_entry()` is called with the current timestamp and period data.
/// # Then
/// `cache.history[0]` contains `t` (Unix seconds), `h5: [42.0, "..."]`, `d7: [35.0, "..."]`, `sn: [20.0, "..."]`.
#[ test ]
fn history_append_stores_correct_fields()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Write alice.json with empty cache.history[].
  std::fs::write(
    store.join( "alice.json" ),
    r#"{"cache":{"fetched_at":"2026-06-21T12:00:00Z","status":"ok","history":[]}}"#,
  ).unwrap();

  let t = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .map_or( 0, |d| d.as_secs() );

  account::write_history_entry(
    store,
    "alice",
    t,
    Some( ( 42.0, "2026-06-21T14:00:00+00:00" ) ),
    Some( ( 35.0, "2026-06-25T00:00:00+00:00" ) ),
    Some( ( 20.0, "2026-06-25T00:00:00+00:00" ) ),
  );

  let entries = account::read_history( store, "alice" );
  assert_eq!( entries.len(), 1, "FT-01: exactly 1 history entry after first append" );
  let e = &entries[ 0 ];
  assert!(
    t.abs_diff( e.t ) <= 2,
    "FT-01: stored t={} must be within 2s of now t={}", e.t, t,
  );
  let h5 = e.h5.as_ref().expect( "FT-01: h5 must be Some" );
  assert!( ( h5.0 - 42.0 ).abs() < 1e-9, "FT-01: h5 utilization got {}", h5.0 );
  assert_eq!( h5.1, "2026-06-21T14:00:00+00:00", "FT-01: h5 resets_at" );
  let d7 = e.d7.as_ref().expect( "FT-01: d7 must be Some" );
  assert!( ( d7.0 - 35.0 ).abs() < 1e-9, "FT-01: d7 utilization got {}", d7.0 );
  let sn = e.sn.as_ref().expect( "FT-01: sn must be Some" );
  assert!( ( sn.0 - 20.0 ).abs() < 1e-9, "FT-01: sn utilization got {}", sn.0 );
}

/// FT-02 (AC-02): Ring buffer evicts oldest entry when 11th measurement appended.
///
/// # Given
/// `alice.json` `cache.history[]` already has 10 entries with `t` values 1000..1009.
/// # When
/// An 11th measurement is appended with `t = 1010`.
/// # Then
/// `cache.history[]` has exactly 10 entries; `history[0].t == 1001` (oldest evicted);
/// `history[9].t == 1010` (newest appended).
#[ test ]
fn history_ring_buffer_evicts_oldest()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Append 10 entries with t = 1000..1009.
  for i in 0_u64..10
  {
    account::write_history_entry( store, "alice", 1000 + i, None, None, None );
  }

  let entries = account::read_history( store, "alice" );
  assert_eq!( entries.len(), 10, "FT-02: exactly 10 entries after 10 appends" );

  // Append 11th entry.
  account::write_history_entry( store, "alice", 1010, None, None, None );

  let entries = account::read_history( store, "alice" );
  assert_eq!( entries.len(), 10, "FT-02: still 10 entries after 11th append (ring buffer cap)" );
  assert_eq!(
    entries[ 0 ].t, 1001,
    "FT-02: oldest entry (t=1000) evicted; t=1001 is now first",
  );
  assert_eq!(
    entries[ 9 ].t, 1010,
    "FT-02: newest entry (t=1010) is last",
  );
}

/// FT-11 (AC-11): Backward compatibility — absent `"history"` key returns empty vec.
///
/// # Given
/// `alice.json` has a `cache` object with quota fields but no `"history"` key (old format).
/// # When
/// `read_history()` is called for `alice`.
/// # Then
/// Returns empty `Vec` — existing single-point fallback behavior preserved.
#[ test ]
fn history_read_absent_key_returns_empty()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Old cache format without "history" key.
  std::fs::write(
    store.join( "alice.json" ),
    r#"{"cache":{"fetched_at":"2026-06-21T12:00:00Z","status":"ok","five_hour":{"left_pct":42.0}}}"#,
  ).unwrap();

  let entries = account::read_history( store, "alice" );
  assert!(
    entries.is_empty(),
    "FT-11: absent history key must return empty vec (AC-11 backward compat); got: {}", entries.len(),
  );
}

/// FT-13 (AC-13): Duplicate timestamp overwrites last entry instead of appending.
///
/// # Given
/// `alice.json` `cache.history[]` has 3 entries. The last entry has `t = 1002`.
/// # When
/// A new measurement is appended with `t = 1002` (same Unix second, updated values).
/// # Then
/// `cache.history[]` still has 3 entries (not 4). The last entry's `h5` is updated.
#[ test ]
fn history_duplicate_timestamp_overwrites()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  // Append 3 entries.
  account::write_history_entry( store, "alice", 1000, Some( ( 10.0, "2026-06-21T14:00:00+00:00" ) ), None, None );
  account::write_history_entry( store, "alice", 1001, Some( ( 20.0, "2026-06-21T14:00:00+00:00" ) ), None, None );
  account::write_history_entry( store, "alice", 1002, Some( ( 30.0, "2026-06-21T14:00:00+00:00" ) ), None, None );

  let entries = account::read_history( store, "alice" );
  assert_eq!( entries.len(), 3, "FT-13: 3 entries before duplicate-timestamp test" );

  // Append with same t as last entry (duplicate timestamp).
  account::write_history_entry( store, "alice", 1002, Some( ( 99.0, "2026-06-21T14:00:00+00:00" ) ), None, None );

  let entries = account::read_history( store, "alice" );
  assert_eq!( entries.len(), 3, "FT-13: still 3 entries after duplicate-timestamp append" );
  let last = &entries[ 2 ];
  let h5   = last.h5.as_ref().expect( "FT-13: last entry h5 must be Some" );
  assert!(
    ( h5.0 - 99.0 ).abs() < 1e-9,
    "FT-13: last entry updated to new value (99.0), not 30.0; got {}", h5.0,
  );
}

// ── BUG-540: cache period key must be `utilization`, not `left_pct` ──────────

/// bug_reproducer(BUG-540): the serialized period key must be named for what
/// the value IS — utilization (percent consumed) — not `left_pct` (percent
/// remaining), which inverts the meaning for every raw-JSON consumer.
///
/// # Root Cause
///
/// `period_json()` stored the utilization value under the key `left_pct`:
/// a 100%-consumed quota serialized as `"left_pct": 100.0` while the CLI
/// displayed "0% left" — the stored name asserted the exact opposite of the
/// stored value. `read_period()` read it back symmetrically, so clp itself
/// rendered correctly and the inversion was invisible from inside the crate.
///
/// # Why Not Caught
///
/// All existing tests exercised the write→read round-trip through
/// `write_quota_cache`/`read_quota_cache`, where the symmetric misnaming
/// cancels out. No test asserted the raw serialized key name against the
/// value's documented meaning.
///
/// # Fix Applied
///
/// `period_json()` now writes the key `utilization`; `read_period()` reads
/// `utilization` first and falls back to `left_pct` for legacy cache files.
///
/// # Prevention
///
/// When a field's name encodes a direction (left/used, remaining/consumed),
/// assert the raw serialized name in a test — round-trip tests alone cancel
/// out symmetric naming errors.
///
/// # Pitfall
///
/// Do NOT "fix" this by inverting the stored value to match the old name —
/// every history ring and cross-host cache already holds utilization values;
/// renaming the key preserves them, inverting the value would corrupt them.
#[ test ]
fn bug_540_period_key_is_utilization_not_left_pct()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  account::write_quota_cache( store, "alice", Some( ( 83.0, Some( "2026-08-20T12:00:00Z" ) ) ), None, None );

  let raw : serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string( store.join( "cache" ).join( host_slug() ).join( "alice.json" ) ).expect( "cache file readable" )
  ).expect( "cache file is JSON" );
  let period = raw.get( "five_hour" ).expect( "five_hour period present" );

  assert!(
    period.get( "left_pct" ).is_none(),
    "BUG-540: the misleading `left_pct` key must not be written; got {period}"
  );
  let utilization = period.get( "utilization" )
    .and_then( serde_json::Value::as_f64 )
    .expect( "BUG-540: period must carry a numeric `utilization` key" );
  assert!(
    ( utilization - 83.0 ).abs() < 1e-9,
    "BUG-540: stored utilization must equal the written value; got {utilization}"
  );
}

/// bug_reproducer(BUG-540): a legacy cache file that still carries the old
/// `left_pct` key must remain readable — the dual-key reader surfaces its
/// value unchanged (it always held utilization, only the name lied).
#[ test ]
fn bug_540_legacy_left_pct_cache_file_still_reads()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  let legacy_dir = store.join( "cache" ).join( "legacyhost_legacyuser" );
  std::fs::create_dir_all( &legacy_dir ).unwrap();
  std::fs::write(
    legacy_dir.join( "alice.json" ),
    r#"{"fetched_at":"2026-08-19T10:00:00Z","status":"ok","five_hour":{"left_pct":42.5,"resets_at":"2026-08-19T15:00:00Z"}}"#,
  ).unwrap();

  let entry = account::read_quota_cache( store, "alice" ).expect( "legacy cache must be readable" );
  let ( utilization, resets_at ) = entry.five_hour.expect( "five_hour present" );
  assert!(
    ( utilization - 42.5 ).abs() < 1e-9,
    "BUG-540: legacy left_pct value must surface as utilization; got {utilization}"
  );
  assert_eq!( resets_at.as_deref(), Some( "2026-08-19T15:00:00Z" ) );
}


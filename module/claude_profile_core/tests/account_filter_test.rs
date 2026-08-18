//! Identity tag-filter tests: `_filter_*` filename derivation, read/write
//! semantics, contradiction rejection, and the eligibility predicate
//! (Feature 076, task 527 T11–T16).
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `filter_t11_read_filter_absent_and_malformed` | absent file → permit-all `Ok`; malformed JSON → loud `Err` |
//! | `filter_t12_write_filter_replaces_only_given_side` | include-only then exclude-only writes; other side survives; sorted/deduped |
//! | `filter_t13_write_filter_rejects_overlap` | `include ∩ exclude ≠ ∅` rejected naming the overlap; file unchanged |
//! | `filter_t14_filter_filename_matches_marker_slug` | `_filter_` slug byte-identical to the `_active_` marker's |
//! | `filter_t15_eligible_predicate_tagged` | superset passes; missing include fails; exclude hit fails |
//! | `filter_t16_eligible_predicate_untagged` | untagged fails non-empty include; passes exclude-only |

use tempfile::TempDir;
use claude_profile_core::account;

mod account_fixture;
use account_fixture::strings;

/// T11 — `read_filter()` treats an absent file as permit-all and malformed
/// JSON as a loud error, never a silent permit-all.
///
/// ## Setup
/// Empty store dir; then a `_filter_*` file with non-JSON content; then one
/// with valid JSON that is not an object.
///
/// ## Assert
/// Absent → `Ok` with both sets empty; both malformed variants → `Err`.
///
/// Spec: task 527 T11; `docs/feature/076_identity_tag_filter.md` AC-01/AC-16
#[ test ]
fn filter_t11_read_filter_absent_and_malformed()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  let filter = account::read_filter( store ).unwrap();
  assert!(
    filter.include.is_empty() && filter.exclude.is_empty(),
    "T11: absent filter file must read as permit-all (both sets empty); got: {filter:?}",
  );

  let path = store.join( account::filter_filename() );
  std::fs::write( &path, "{not json" ).unwrap();
  assert!(
    account::read_filter( store ).is_err(),
    "T11: malformed JSON must be a loud error, never silent permit-all",
  );

  std::fs::write( &path, "[1,2]" ).unwrap();
  assert!(
    account::read_filter( store ).is_err(),
    "T11: valid JSON that is not an object must still be a loud error",
  );
}

/// T12 — each `write_filter()` call fully replaces only its given side; the
/// omitted side survives; both sides are stored sorted and deduplicated.
///
/// ## Setup
/// Include-only write with unsorted duplicates, then an exclude-only write.
///
/// ## Assert
/// After call 1: include `[ci,work]`, exclude `[]`. After call 2: include
/// survives, exclude `[x]`. Re-read from disk matches the returned filter.
///
/// Spec: task 527 T12; `docs/feature/076_identity_tag_filter.md` AC-03/AC-04
#[ test ]
fn filter_t12_write_filter_replaces_only_given_side()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  let written = account::write_filter( store, Some( &strings( &[ "work", "ci", "work" ] ) ), None )
    .unwrap();
  assert_eq!(
    written.include, strings( &[ "ci", "work" ] ),
    "T12: include side must be stored sorted + deduped",
  );
  assert!(
    written.exclude.is_empty(),
    "T12: omitted exclude side must stay empty on a fresh file",
  );

  let written = account::write_filter( store, None, Some( &strings( &[ "x" ] ) ) ).unwrap();
  assert_eq!(
    written.include, strings( &[ "ci", "work" ] ),
    "T12: include side must survive an exclude-only write",
  );
  assert_eq!(
    written.exclude, strings( &[ "x" ] ),
    "T12: exclude side must be fully replaced by the given set",
  );

  let read_back = account::read_filter( store ).unwrap();
  assert_eq!( read_back, written, "T12: on-disk filter must match the returned one" );
}

/// T13 — a write with non-empty `include ∩ exclude` is rejected with a
/// descriptive error naming the overlap, before any file write.
///
/// ## Setup
/// A valid existing filter file; then a contradictory write attempt.
///
/// ## Assert
/// `Err` names the overlapping tag; the file bytes are unchanged.
///
/// Spec: task 527 T13; `docs/feature/076_identity_tag_filter.md` AC-05
#[ test ]
fn filter_t13_write_filter_rejects_overlap()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  account::write_filter( store, Some( &strings( &[ "work" ] ) ), None ).unwrap();
  let path   = store.join( account::filter_filename() );
  let before = std::fs::read( &path ).unwrap();

  let err = account::write_filter(
    store,
    Some( &strings( &[ "ci" ] ) ),
    Some( &strings( &[ "ci", "x" ] ) ),
  ).unwrap_err();
  assert!(
    err.to_string().contains( "ci" ),
    "T13: the rejection must name the overlapping tag; got: {err}",
  );
  assert_eq!(
    std::fs::read( &path ).unwrap(), before,
    "T13: a rejected contradictory write must leave the filter file unchanged",
  );
}

/// T14 — the filter filename's identity slug is byte-identical to the one the
/// `_active_*` marker uses (same `host_user_slug()` sanitization).
///
/// ## Assert
/// `filter_filename()` == `"_filter_"` + (`active_marker_filename()` minus its
/// `"_active_"` prefix).
///
/// Spec: task 527 T14; `docs/schema/009_identity_filter_json.md` (AC-08)
#[ test ]
fn filter_t14_filter_filename_matches_marker_slug()
{
  let marker = account::active_marker_filename();
  let slug = marker.strip_prefix( "_active_" )
    .expect( "T14: active_marker_filename() must start with '_active_'" );
  assert_eq!(
    account::filter_filename(), format!( "_filter_{slug}" ),
    "T14: filter filename slug must be byte-identical to the active marker's",
  );
}

/// T15 — `eligible()` on a tagged account: `tags ⊇ include ∧ tags ∩ exclude = ∅`.
///
/// ## Assert
/// `{ci,work}` vs include `{ci}` → pass; vs include `{ci,kimi}` → fail
/// (missing `kimi`); vs exclude `{work}` → fail (exclude hit).
///
/// Spec: task 527 T15; `docs/type/004_tag_filter.md § Definition` (AC-09)
#[ test ]
fn filter_t15_eligible_predicate_tagged()
{
  let tags = strings( &[ "ci", "work" ] );

  let pass = account::TagFilter { include : strings( &[ "ci" ] ), exclude : Vec::new() };
  assert!(
    account::eligible( &tags, &pass ),
    "T15: a tag superset of include with no exclude hit must pass",
  );

  let missing = account::TagFilter { include : strings( &[ "ci", "kimi" ] ), exclude : Vec::new() };
  assert!(
    !account::eligible( &tags, &missing ),
    "T15: a missing include tag must fail eligibility",
  );

  let excluded = account::TagFilter { include : Vec::new(), exclude : strings( &[ "work" ] ) };
  assert!(
    !account::eligible( &tags, &excluded ),
    "T15: carrying an excluded tag must fail eligibility",
  );
}

/// T16 — `eligible()` on an untagged account: fails any non-empty include
/// (empty set ⊉ non-empty include), trivially passes exclude-only filters.
///
/// ## Assert
/// `{}` vs include `{ci}` → fail; `{}` vs include `{}` exclude `{x}` → pass.
///
/// Spec: task 527 T16; `docs/type/004_tag_filter.md § Validation` (AC-09)
#[ test ]
fn filter_t16_eligible_predicate_untagged()
{
  let untagged : Vec< String > = Vec::new();

  let include_only = account::TagFilter { include : strings( &[ "ci" ] ), exclude : Vec::new() };
  assert!(
    !account::eligible( &untagged, &include_only ),
    "T16: an untagged account must fail a non-empty include",
  );

  let exclude_only = account::TagFilter { include : Vec::new(), exclude : strings( &[ "x" ] ) };
  assert!(
    account::eligible( &untagged, &exclude_only ),
    "T16: an untagged account must trivially pass an exclude-only filter",
  );
}

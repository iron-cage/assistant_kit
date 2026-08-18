//! Account tag tests: normalization, set ops on `{name}.json`, the lazy `role`
//! migration, and save/list round-trip (Feature 075, task 527 T01–T10).
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `tags_t01_normalize_tag_lowercases_before_validation` | `CI-East_1` → `ci-east_1`; set normalization dedups + sorts |
//! | `tags_t02_normalize_tag_rejects_invalid_loudly` | space / empty / 65-char rejected naming the tag; store byte-identical on `write_tags` rejection |
//! | `tags_t03_add_op_dedups_and_sorts` | add `[work,ci,work]` to `[ci]` → stored `[ci,work]` |
//! | `tags_t04_remove_op_set_semantics` | remove present tag; remove absent tag is a no-op success; preview does not write |
//! | `tags_t05_replace_op_overwrites` | replace `[ci,work]` with `[kimi_pool]` → exactly `[kimi_pool]` |
//! | `tags_t06_migration_on_first_add` | `"role":"Work"` → `work` joins the set; `role` key deleted from real bytes |
//! | `tags_t07_migration_on_first_remove` | first write is a REMOVE: `role` migrates first, then removal applies |
//! | `tags_t08_no_role_resurrection` | second write after migration: no `role` key reappears |
//! | `tags_t09_empty_role_migrates_nothing` | `"role":""` adds no entry but the key is still removed; absent `role` fine |
//! | `tags_t10_list_and_save_round_trip` | `list()` absent `tags` key → empty vec; `save()` with `None` preserves stored set |

use tempfile::TempDir;
use claude_profile_core::account;
use claude_core::ClaudePaths;

mod account_fixture;
use account_fixture::strings;

/// Read `{name}.json` back as a JSON object for byte-level post-condition asserts.
fn meta_object( store : &std::path::Path, name : &str ) -> serde_json::Map< String, serde_json::Value >
{
  let text = std::fs::read_to_string( store.join( format!( "{name}.json" ) ) ).unwrap();
  serde_json::from_str::< serde_json::Value >( &text ).unwrap().as_object().unwrap().clone()
}

/// Extract the `tags` array from `{name}.json` as strings.
fn stored_tags( store : &std::path::Path, name : &str ) -> Vec< String >
{
  meta_object( store, name )
    .get( "tags" )
    .and_then( | v | v.as_array() )
    .map( | a | a.iter().map( | v | v.as_str().unwrap().to_string() ).collect() )
    .unwrap_or_default()
}

/// T01 — `normalize_tag()` lowercases BEFORE validating, so mixed-case input
/// with a valid post-lowercase charset is accepted, never rejected for case.
///
/// ## Assert
/// `CI-East_1` → `ci-east_1`; `normalize_tag_set()` also dedups + sorts.
///
/// Spec: task 527 T01; `docs/type/003_tag.md § Validation` (AC-03)
#[ test ]
fn tags_t01_normalize_tag_lowercases_before_validation()
{
  assert_eq!(
    account::normalize_tag( "CI-East_1" ).unwrap(), "ci-east_1",
    "T01: mixed-case tag must lowercase to a valid tag, not be rejected",
  );
  let set = account::normalize_tag_set( &strings( &[ "Work", "CI", "work" ] ) ).unwrap();
  assert_eq!(
    set, strings( &[ "ci", "work" ] ),
    "T01: set normalization must lowercase, deduplicate, and sort",
  );
}

/// T02 — invalid tags (charset, empty, >64 chars) are rejected loudly with the
/// violating tag named, BEFORE any file write (store byte-identical).
///
/// ## Setup
/// A `{name}.json` with existing metadata; a `write_tags()` attempt carrying an
/// invalid tag.
///
/// ## Assert
/// Each `normalize_tag()` error names/describes the violation; the rejected
/// `write_tags()` leaves `{name}.json` byte-identical (C15/AC-02).
///
/// Spec: task 527 T02; `docs/feature/075_account_tags.md` AC-02
#[ test ]
fn tags_t02_normalize_tag_rejects_invalid_loudly()
{
  let err_space = account::normalize_tag( "has space" ).unwrap_err();
  assert!(
    err_space.to_string().contains( "has space" ),
    "T02: charset error must name the violating tag; got: {err_space}",
  );

  let err_empty = account::normalize_tag( "" ).unwrap_err();
  assert!(
    err_empty.to_string().contains( "empty" ),
    "T02: empty-tag error must describe the violation; got: {err_empty}",
  );

  let long = "x".repeat( 65 );
  let err_long = account::normalize_tag( &long ).unwrap_err();
  assert!(
    err_long.to_string().contains( "64" ),
    "T02: length error must state the 64-char limit; got: {err_long}",
  );

  // Rejection happens before any write: the store stays byte-identical.
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "alice@test.com";
  let meta  = store.join( format!( "{name}.json" ) );
  std::fs::write( &meta, r#"{"host":"laptop","tags":["ci"]}"# ).unwrap();
  let before = std::fs::read( &meta ).unwrap();

  let err = account::write_tags( name, store, &account::TagOp::Add( strings( &[ "has space" ] ) ) )
    .unwrap_err();
  assert!(
    err.to_string().contains( "has space" ),
    "T02: write_tags rejection must name the violating tag; got: {err}",
  );
  assert_eq!(
    std::fs::read( &meta ).unwrap(), before,
    "T02: a rejected tag write must leave {{name}}.json byte-identical",
  );
}

/// T03 — the add op unions into the stored set, deduplicated and sorted.
///
/// ## Setup
/// `{name}.json` with `"tags":["ci"]`; add `[work, ci, work]` (dups on both sides).
///
/// ## Assert
/// Stored set is exactly `["ci","work"]`; `write_tags` returns the same set.
///
/// Spec: task 527 T03; `docs/feature/075_account_tags.md` AC-04
#[ test ]
fn tags_t03_add_op_dedups_and_sorts()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "alice@test.com";
  std::fs::write( store.join( format!( "{name}.json" ) ), r#"{"tags":["ci"]}"# ).unwrap();

  let result = account::write_tags( name, store, &account::TagOp::Add( strings( &[ "work", "ci", "work" ] ) ) )
    .unwrap();
  assert_eq!( result, strings( &[ "ci", "work" ] ), "T03: returned set must be deduped + sorted" );
  assert_eq!(
    stored_tags( store, name ), strings( &[ "ci", "work" ] ),
    "T03: stored set must be deduped + sorted",
  );
}

/// T04 — the remove op has set semantics: removing a present tag drops it;
/// removing an absent tag is a no-op success; the file stays valid JSON.
/// `preview_tags()` computes the same result without writing.
///
/// ## Setup
/// `{name}.json` with `"tags":["ci","work"]`.
///
/// ## Assert
/// Preview of the removal returns `["work"]` with the file untouched; the real
/// removal stores `["work"]`; removing `absent` succeeds and changes nothing.
///
/// Spec: task 527 T04; `docs/feature/075_account_tags.md` AC-04
#[ test ]
fn tags_t04_remove_op_set_semantics()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "alice@test.com";
  let meta  = store.join( format!( "{name}.json" ) );
  std::fs::write( &meta, r#"{"tags":["ci","work"]}"# ).unwrap();
  let before = std::fs::read( &meta ).unwrap();

  // Preview: same computation, no write.
  let preview = account::preview_tags( name, store, &account::TagOp::Remove( strings( &[ "ci" ] ) ) )
    .unwrap();
  assert_eq!( preview, strings( &[ "work" ] ), "T04: preview must compute the post-removal set" );
  assert_eq!(
    std::fs::read( &meta ).unwrap(), before,
    "T04: preview_tags must not write the file",
  );

  let result = account::write_tags( name, store, &account::TagOp::Remove( strings( &[ "ci" ] ) ) )
    .unwrap();
  assert_eq!( result, strings( &[ "work" ] ), "T04: removing a present tag must drop it" );

  // Removing an absent tag: no-op success, file still valid JSON.
  let result = account::write_tags( name, store, &account::TagOp::Remove( strings( &[ "absent" ] ) ) )
    .unwrap();
  assert_eq!( result, strings( &[ "work" ] ), "T04: removing an absent tag must be a no-op success" );
  assert_eq!(
    stored_tags( store, name ), strings( &[ "work" ] ),
    "T04: stored set must survive the no-op removal, file valid JSON",
  );
}

/// T05 — the replace op overwrites the stored set exactly.
///
/// ## Setup
/// `{name}.json` with `"tags":["ci","work"]` (already migrated — no `role`).
///
/// ## Assert
/// After replace with `[kimi_pool]`, the stored set is exactly `["kimi_pool"]`.
///
/// Spec: task 527 T05; `docs/feature/075_account_tags.md` AC-04
#[ test ]
fn tags_t05_replace_op_overwrites()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "alice@test.com";
  std::fs::write( store.join( format!( "{name}.json" ) ), r#"{"tags":["ci","work"]}"# ).unwrap();

  let result = account::write_tags( name, store, &account::TagOp::Replace( strings( &[ "kimi_pool" ] ) ) )
    .unwrap();
  assert_eq!( result, strings( &[ "kimi_pool" ] ), "T05: replace must overwrite the whole set" );
  assert_eq!(
    stored_tags( store, name ), strings( &[ "kimi_pool" ] ),
    "T05: stored set must be exactly the replacement",
  );
}

/// T06 — lazy migration on first add: a non-empty legacy `role` is normalized
/// into the tag set and the `role` key is deleted in the same write.
///
/// ## Setup
/// `{name}.json` with `"role":"Work"` and no `tags` key; first write adds `[ci]`.
///
/// ## Assert
/// BOTH post-conditions on the real `{name}.json` bytes (AF3): `tags` is
/// `["ci","work"]` AND the `role` key is absent. Sibling `host` field survives
/// the read-merge.
///
/// Spec: task 527 T06; `docs/feature/075_account_tags.md` AC-09
#[ test ]
fn tags_t06_migration_on_first_add()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "alice@test.com";
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"host":"laptop","role":"Work"}"#,
  ).unwrap();

  let result = account::write_tags( name, store, &account::TagOp::Add( strings( &[ "ci" ] ) ) )
    .unwrap();
  assert_eq!( result, strings( &[ "ci", "work" ] ), "T06: migrated role must join the added tags" );

  let obj = meta_object( store, name );
  assert!(
    !obj.contains_key( "role" ),
    "T06: the role key must be deleted from {{name}}.json in the same write; got: {obj:?}",
  );
  assert_eq!(
    stored_tags( store, name ), strings( &[ "ci", "work" ] ),
    "T06: stored bytes must carry the migrated tag alongside the added one",
  );
  assert_eq!(
    obj.get( "host" ).and_then( | v | v.as_str() ), Some( "laptop" ),
    "T06: read-merge must preserve sibling fields",
  );
}

/// T07 — lazy migration fires even when the FIRST tag write is a remove: the
/// `role` migrates into the set first, then the removal applies.
///
/// ## Setup
/// `{name}.json` with `"role":"work"` and `"tags":["ci"]`; first write removes `ci`.
///
/// ## Assert
/// BOTH post-conditions on real bytes (AF3): stored set is `["work"]` (migrated
/// entry survives, removed tag gone) AND the `role` key is absent. A removal
/// targeting the migrated tag itself also applies after migration.
///
/// Spec: task 527 T07; `docs/feature/075_account_tags.md` AC-09
#[ test ]
fn tags_t07_migration_on_first_remove()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "alice@test.com";
  std::fs::write(
    store.join( format!( "{name}.json" ) ),
    r#"{"role":"work","tags":["ci"]}"#,
  ).unwrap();

  let result = account::write_tags( name, store, &account::TagOp::Remove( strings( &[ "ci" ] ) ) )
    .unwrap();
  assert_eq!(
    result, strings( &[ "work" ] ),
    "T07: role must migrate into the set before the removal applies",
  );
  let obj = meta_object( store, name );
  assert!(
    !obj.contains_key( "role" ),
    "T07: the role key must be deleted even on a pure remove; got: {obj:?}",
  );
  assert_eq!(
    stored_tags( store, name ), strings( &[ "work" ] ),
    "T07: stored bytes must show migration-then-removal ordering",
  );

  // Removing the migrated tag itself on first write: migration first, removal after.
  let name2 = "bob@test.com";
  std::fs::write(
    store.join( format!( "{name2}.json" ) ),
    r#"{"role":"work","tags":["ci"]}"#,
  ).unwrap();
  let result = account::write_tags( name2, store, &account::TagOp::Remove( strings( &[ "work" ] ) ) )
    .unwrap();
  assert_eq!(
    result, strings( &[ "ci" ] ),
    "T07: a removal naming the migrated tag must drop it (removal applies AFTER migration)",
  );
  assert!(
    !meta_object( store, name2 ).contains_key( "role" ),
    "T07: role key must be deleted regardless of the removal target",
  );
}

/// T08 — a second tag write on an already-migrated account never resurrects
/// the `role` key; the set changes only per the requested op.
///
/// ## Setup
/// T06's scenario completed (role migrated); then a second add.
///
/// ## Assert
/// `role` key still absent; set is the prior set plus the newly added tag only.
///
/// Spec: task 527 T08; `docs/feature/075_account_tags.md` AC-09
#[ test ]
fn tags_t08_no_role_resurrection()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();
  let name  = "alice@test.com";
  std::fs::write( store.join( format!( "{name}.json" ) ), r#"{"role":"Work"}"# ).unwrap();

  account::write_tags( name, store, &account::TagOp::Add( strings( &[ "ci" ] ) ) ).unwrap();
  let result = account::write_tags( name, store, &account::TagOp::Add( strings( &[ "kimi_pool" ] ) ) )
    .unwrap();

  assert_eq!(
    result, strings( &[ "ci", "kimi_pool", "work" ] ),
    "T08: second write must change the set only per the requested op",
  );
  assert!(
    !meta_object( store, name ).contains_key( "role" ),
    "T08: no role key may reappear after migration",
  );
}

/// T09 — an empty `role` migrates nothing but the key is still removed; an
/// absent `role` key migrates nothing and the op succeeds.
///
/// ## Setup
/// One account with `"role":""`, one with no `role` key at all.
///
/// ## Assert
/// Both first writes succeed with exactly the added tag; the empty `role` key
/// is gone from the stored bytes.
///
/// Spec: task 527 T09; `docs/feature/075_account_tags.md` AC-09
#[ test ]
fn tags_t09_empty_role_migrates_nothing()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path();

  let name = "alice@test.com";
  std::fs::write( store.join( format!( "{name}.json" ) ), r#"{"role":""}"# ).unwrap();
  let result = account::write_tags( name, store, &account::TagOp::Add( strings( &[ "ci" ] ) ) )
    .unwrap();
  assert_eq!( result, strings( &[ "ci" ] ), "T09: empty role must not add a migration entry" );
  assert!(
    !meta_object( store, name ).contains_key( "role" ),
    "T09: the empty role key must still be removed",
  );

  let name2 = "bob@test.com";
  std::fs::write( store.join( format!( "{name2}.json" ) ), r"{}" ).unwrap();
  let result = account::write_tags( name2, store, &account::TagOp::Add( strings( &[ "ci" ] ) ) )
    .unwrap();
  assert_eq!( result, strings( &[ "ci" ] ), "T09: absent role key must migrate nothing, op succeeds" );
}

/// T10 — `list()` yields an empty tag vec for account files predating the
/// field, and `save()` with tags `None` preserves an existing stored set.
///
/// ## Setup
/// Two accounts with credential files (so `list()` sees them): one whose
/// `{name}.json` has no `tags` key, one with tags written via `write_tags()`.
/// A background-style `save()` (tags `None`) runs on the tagged account.
///
/// ## Assert
/// Untagged account loads with `tags == []`; after `save( .., None )` the
/// stored `tags` array is unchanged and `list()` returns it sorted/deduped.
///
/// Spec: task 527 T10; `docs/feature/075_account_tags.md` AC-01
#[ test ]
fn tags_t10_list_and_save_round_trip()
{
  let tmp   = TempDir::new().unwrap();
  let store = tmp.path().join( "store" );
  std::fs::create_dir_all( &store ).unwrap();
  let dot_claude = tmp.path().join( ".claude" );
  std::fs::create_dir_all( &dot_claude ).unwrap();
  std::fs::write( dot_claude.join( ".credentials.json" ), r#"{"accessToken":"tok"}"# ).unwrap();
  let paths = ClaudePaths::with_home( tmp.path() );

  let legacy = "legacy@test.com";
  account_fixture::write_credentials_file( &store, legacy );
  std::fs::write( store.join( format!( "{legacy}.json" ) ), r#"{"host":"laptop"}"# ).unwrap();

  let tagged = "tagged@test.com";
  account_fixture::write_credentials_file( &store, tagged );
  account::write_tags( tagged, &store, &account::TagOp::Replace( strings( &[ "work", "ci" ] ) ) )
    .unwrap();
  let before = stored_tags( &store, tagged );
  assert_eq!( before, strings( &[ "ci", "work" ] ), "T10: precondition — tags stored sorted" );

  // Background-style save with tags None must not clobber the stored set.
  account::save(
    tagged, &store, &paths, false, Some( br#"{"accessToken":"tok"}"# ),
    None, None, None, account::AccountBackend::Anthropic, None, None, None, None,
  ).unwrap();
  assert_eq!(
    stored_tags( &store, tagged ), before,
    "T10: save() with tags None must preserve the stored tag set",
  );

  let accounts = account::list( &store ).unwrap();
  let legacy_acct = accounts.iter().find( | a | a.name == legacy ).unwrap();
  assert!(
    legacy_acct.tags.is_empty(),
    "T10: absent tags key must load as an empty vec; got: {:?}", legacy_acct.tags,
  );
  let tagged_acct = accounts.iter().find( | a | a.name == tagged ).unwrap();
  assert_eq!(
    tagged_acct.tags, strings( &[ "ci", "work" ] ),
    "T10: list() must parse the stored tag set",
  );
}

//! Integration tests: Feature 075 — account tags (`.account.save tags::`,
//! `.account.tag`, `.tags`, `.accounts` tag surfaces, `role::` removal).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`
//! against a temporary isolated credential store — no network, no real HOME.
//!
//! ## Test Matrix
//!
//! | ID    | Test Function                                    | AC    |
//! |-------|--------------------------------------------------|-------|
//! | FT-01 | `account_tag_t01_save_tags_writes_sorted_dedup`  | AC-01 |
//! | FT-02 | `account_tag_t02_save_without_tags_omits_field`  | AC-02 |
//! | FT-03 | `account_tag_t03_invalid_tag_exits_1_no_write`   | AC-03 |
//! | FT-04 | `account_tag_t04_role_param_exits_1_migration`   | AC-04 |
//! | FT-05 | `account_tag_t05_tag_add_unions_set`             | AC-05 |
//! | FT-06 | `account_tag_t06_tag_remove_idempotent`          | AC-06 |
//! | FT-07 | `account_tag_t07_tag_replace_and_mutual_exclusion` | AC-07 |
//! | FT-08 | `account_tag_t08_tag_no_operation_exits_1`       | AC-08 |
//! | FT-09 | `account_tag_t09_first_tag_write_migrates_role`  | AC-09 |
//! | FT-10 | `account_tag_t10_tag_ungated_batch_dry`          | AC-10 |
//! | FT-11 | `account_tag_t11_tags_lists_union_sorted`        | AC-11 |
//! | FT-12 | `account_tag_t12_tags_json_shape`                | AC-12 |
//! | FT-13 | `account_tag_t13_accounts_tags_subset_filter`    | AC-13 |
//! | FT-14 | `account_tag_t14_accounts_tags_line_and_json`    | AC-14 |
//! | FT-15 | `account_tag_t15_cols_plus_tags_column`          | AC-15 |
//! | FT-16 | `account_tag_t16_untagged_store_byte_identical`  | AC-16 |
//! | IT-05 | `account_tag_t17_tags_bad_format_exits_1`        | —     |
//! | IT-07 | `account_tag_t19_account_tag_unknown_account_exits_2` | — |
//! | IT-08 | `account_tag_t20_account_tag_missing_name_exits_1` | —   |
//!
//! Spec: `tests/docs/feature/075_account_tags.md`,
//! `tests/docs/cli/command/22_tags.md`, `tests/docs/cli/command/25_account_tag.md`.

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, write_account_owner,
  write_account_profile_json, write_account_quota_cache,
  write_account_tags, write_filter_file,
  read_account_meta, account_exists,
  credential_store_dir, normalized_lines,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

/// Extract the `tags` array of `{name}.json` as plain strings; `None` when the
/// key is absent (distinct from present-but-empty).
fn meta_tags( home : &std::path::Path, name : &str ) -> Option< Vec< String > >
{
  read_account_meta( home, name ).get( "tags" )?.as_array().map( | a |
    a.iter().filter_map( | v | v.as_str().map( str::to_string ) ).collect() )
}

/// Read every regular file in the credential store into a sorted (name, bytes)
/// snapshot — for byte-identity assertions around read-only commands.
fn store_snapshot( home : &std::path::Path ) -> Vec< ( String, Vec< u8 > ) >
{
  let mut out : Vec< ( String, Vec< u8 > ) > = std::fs::read_dir( credential_store_dir( home ) )
    .map( | rd | rd
      .filter_map( Result::ok )
      .filter( | e | e.path().is_file() )
      .map( | e | ( e.file_name().to_string_lossy().to_string(), std::fs::read( e.path() ).unwrap() ) )
      .collect() )
    .unwrap_or_default();
  out.sort();
  out
}

// ── FT-01: `.account.save tags::` writes sorted deduplicated set (AC-01) ──────

/// FT-01 — `.account.save tags::kimi_pool,ci,kimi_pool` stores
/// `"tags": ["ci", "kimi_pool"]` — lowercased input normalized, deduplicated, sorted.
///
/// ## Setup
/// Live credentials present; account not yet saved.
///
/// ## Assert
/// Exit 0; meta `tags` array is exactly `["ci", "kimi_pool"]`.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-01 (AC-01)
#[ test ]
fn account_tag_t01_save_tags_writes_sorted_dedup()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_credentials( home, "max", "default_claude_max_20x", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@test.com", "tags::kimi_pool,ci,kimi_pool" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 0 );
  assert_eq!(
    meta_tags( home, "alice@test.com" ), Some( vec![ "ci".to_string(), "kimi_pool".to_string() ] ),
    "FT-01: tags:: must store a sorted deduplicated array; meta: {}",
    read_account_meta( home, "alice@test.com" ),
  );
}

// ── FT-02: omitted `tags::` leaves the field absent (AC-02) ───────────────────

/// FT-02 — `.account.save` without `tags::` writes no `tags` key at all;
/// listing commands treat the account as untagged without erroring.
///
/// ## Assert
/// Exit 0; meta object has no `tags` key; `.accounts` succeeds.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-02 (AC-02)
#[ test ]
fn account_tag_t02_save_without_tags_omits_field()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_credentials( home, "max", "default_claude_max_20x", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".account.save", "name::alice@test.com" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  assert_eq!(
    meta_tags( home, "alice@test.com" ), None,
    "FT-02: omitted tags:: must leave the field absent; meta: {}",
    read_account_meta( home, "alice@test.com" ),
  );

  let list = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &list, 0 );
}

// ── FT-03: invalid tag exits 1 without writing (AC-03) ────────────────────────

/// FT-03 — an invalid tag (charset violation, >64 chars, empty comma item)
/// exits 1 naming the offending tag in its post-lowercased form; nothing written.
///
/// ## Assert
/// All three variants exit 1; the charset error names `bad!tag`; no account
/// file is created by any of them.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-03 (AC-03)
#[ test ]
fn account_tag_t03_invalid_tag_exits_1_no_write()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_credentials( home, "max", "default_claude_max_20x", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@test.com", "tags::Bad!Tag" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "bad!tag" ),
    "FT-03: rejection must name the post-lowercased tag; stderr: {}", stderr( &out ),
  );

  let long_tag = format!( "tags::{}", "a".repeat( 65 ) );
  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@test.com", &long_tag ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 1 );

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@test.com", "tags::a,,b" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 1 );

  assert!(
    !account_exists( home, "alice@test.com" ),
    "FT-03: a rejected save must not create the account",
  );
}

// ── FT-04: `role::` exits 1 with migration message (AC-04) ────────────────────

/// FT-04 — `.account.save role::work` is REMOVED: exits 1, stderr names
/// `tags::` as the replacement, nothing written.
///
/// ## Assert
/// Exit 1; stderr contains `tags::`; account not created.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-04 (AC-04)
#[ test ]
fn account_tag_t04_role_param_exits_1_migration()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_credentials( home, "max", "default_claude_max_20x", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@test.com", "role::work" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "tags::" ),
    "FT-04: the migration message must name tags:: as the replacement; stderr: {}", stderr( &out ),
  );
  assert!(
    !account_exists( home, "alice@test.com" ),
    "FT-04: role:: rejection must happen before any write",
  );
}

// ── FT-05: `add::` unions into the existing set (AC-05) ───────────────────────

/// FT-05 — `.account.tag add::kimi_pool,ci` on a `["ci"]` account unions to
/// `["ci", "kimi_pool"]` (dedup against stored, sorted).
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-05 (AC-05)
#[ test ]
fn account_tag_t05_tag_add_unions_set()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "alice@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_tags( home, "alice@test.com", &[ "ci" ] );

  let out = run_cs_with_env(
    &[ ".account.tag", "name::alice@test.com", "add::kimi_pool,ci" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 0 );
  assert_eq!(
    meta_tags( home, "alice@test.com" ), Some( vec![ "ci".to_string(), "kimi_pool".to_string() ] ),
    "FT-05: add:: must union into the stored set",
  );
}

// ── FT-06: `remove::` is idempotent (AC-06) ───────────────────────────────────

/// FT-06 — `remove::ci` drops the tag; removing an absent tag is a no-op
/// success leaving the set unchanged.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-06 (AC-06)
#[ test ]
fn account_tag_t06_tag_remove_idempotent()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "alice@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_tags( home, "alice@test.com", &[ "ci", "kimi_pool" ] );

  let out = run_cs_with_env(
    &[ ".account.tag", "name::alice@test.com", "remove::ci" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 0 );
  assert_eq!(
    meta_tags( home, "alice@test.com" ), Some( vec![ "kimi_pool".to_string() ] ),
    "FT-06: remove:: must drop the named tag",
  );

  let out = run_cs_with_env(
    &[ ".account.tag", "name::alice@test.com", "remove::nonexistent" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 0 );
  assert_eq!(
    meta_tags( home, "alice@test.com" ), Some( vec![ "kimi_pool".to_string() ] ),
    "FT-06: removing an absent tag must be a no-op success",
  );
}

// ── FT-07: `tags::` replaces; combined operations exit 1 (AC-07) ──────────────

/// FT-07 — `tags::personal` overwrites the whole set; combining two operations
/// in one invocation (`tags::`+`add::`, `add::`+`remove::`) exits 1 unchanged.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-07 (AC-07)
#[ test ]
fn account_tag_t07_tag_replace_and_mutual_exclusion()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "alice@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_tags( home, "alice@test.com", &[ "ci", "kimi_pool" ] );

  let out = run_cs_with_env(
    &[ ".account.tag", "name::alice@test.com", "tags::personal" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 0 );
  assert_eq!(
    meta_tags( home, "alice@test.com" ), Some( vec![ "personal".to_string() ] ),
    "FT-07: tags:: must replace the whole stored set",
  );

  let out = run_cs_with_env(
    &[ ".account.tag", "name::alice@test.com", "tags::a", "add::b" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 1 );
  let out = run_cs_with_env(
    &[ ".account.tag", "name::alice@test.com", "add::a", "remove::b" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 1 );
  assert_eq!(
    meta_tags( home, "alice@test.com" ), Some( vec![ "personal".to_string() ] ),
    "FT-07: a rejected combined operation must leave the set unchanged",
  );
}

// ── FT-08: no operation given exits 1 (AC-08) ─────────────────────────────────

/// FT-08 — `.account.tag name::X` with no operation exits 1, stderr naming the
/// three operation params.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-08 (AC-08)
#[ test ]
fn account_tag_t08_tag_no_operation_exits_1()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "alice@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.tag", "name::alice@test.com" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "add::" ) && err.contains( "remove::" ) && err.contains( "tags::" ),
    "FT-08: the no-operation error must name add::/remove::/tags::; stderr: {err}",
  );
}

// ── FT-09: first tag write migrates legacy `role` (AC-09) ─────────────────────

/// FT-09 — the first tag write of ANY variant (`.account.tag add::`/`remove::`,
/// `.account.save tags::`) converts a non-empty legacy `role` into a sanitized
/// tag and deletes the `role` key in the same write.
///
/// ## Setup
/// Three fresh accounts, each with `"role": "Work"` and no `tags` key.
///
/// ## Assert
/// (a) `add::ci` → tags `["ci", "work"]`, no `role`; (b) `.account.save tags::ci`
/// → same; (c) `remove::x` → tags `["work"]`, no `role`.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-09 (AC-09)
#[ test ]
fn account_tag_t09_first_tag_write_migrates_role()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_credentials( home, "max", "default_claude_max_20x", FAR_FUTURE_MS );
  for name in [ "a@test.com", "b@test.com", "c@test.com" ]
  {
    write_account( home, name, "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
    write_account_profile_json( home, name, None, Some( "Work" ) );
  }

  let out = run_cs_with_env(
    &[ ".account.tag", "name::a@test.com", "add::ci" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 0 );
  assert_eq!(
    meta_tags( home, "a@test.com" ), Some( vec![ "ci".to_string(), "work".to_string() ] ),
    "FT-09a: add:: must merge the migrated role into the set",
  );
  assert!(
    read_account_meta( home, "a@test.com" ).get( "role" ).is_none(),
    "FT-09a: the role key must be deleted by the migration",
  );

  let out = run_cs_with_env(
    &[ ".account.save", "name::b@test.com", "tags::ci" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 0 );
  assert_eq!(
    meta_tags( home, "b@test.com" ), Some( vec![ "ci".to_string(), "work".to_string() ] ),
    "FT-09b: a save tags:: write must also fire the migration",
  );
  assert!(
    read_account_meta( home, "b@test.com" ).get( "role" ).is_none(),
    "FT-09b: the role key must be deleted by the save-path migration",
  );

  let out = run_cs_with_env(
    &[ ".account.tag", "name::c@test.com", "remove::x" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 0 );
  assert_eq!(
    meta_tags( home, "c@test.com" ), Some( vec![ "work".to_string() ] ),
    "FT-09c: even a pure remove:: fires the migration (removed tag absent, role kept as tag)",
  );
  assert!(
    read_account_meta( home, "c@test.com" ).get( "role" ).is_none(),
    "FT-09c: the role key must be deleted by the remove-path migration",
  );
}

// ── FT-10: ungated writes, comma-list batch, `dry::1` (AC-10) ─────────────────

/// FT-10 — `.account.tag` is ungated (no G5/G9): works on an account owned by
/// a different Identity; `name::X,Y` batches; `dry::1` previews without writing.
///
/// ## Setup
/// `alice` owned by a foreign Identity; `bob` unowned.
///
/// ## Assert
/// Dry run exits 0 leaving both metas byte-identical; real run applies `ci`
/// to both with no ownership error.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-10 (AC-10)
#[ test ]
fn account_tag_t10_tag_ungated_batch_dry()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "alice@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account( home, "bob@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_owner( home, "alice@test.com", "someone_else@otherhost" );

  let before = store_snapshot( home );
  let out = run_cs_with_env(
    &[ ".account.tag", "name::alice@test.com,bob@test.com", "add::ci", "dry::1" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 0 );
  assert_eq!(
    store_snapshot( home ), before,
    "FT-10: dry::1 must leave every store file byte-identical",
  );

  let out = run_cs_with_env(
    &[ ".account.tag", "name::alice@test.com,bob@test.com", "add::ci" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 0 );
  assert_eq!(
    meta_tags( home, "alice@test.com" ), Some( vec![ "ci".to_string() ] ),
    "FT-10: the batch write must apply to a foreign-owned account (ungated)",
  );
  assert_eq!(
    meta_tags( home, "bob@test.com" ), Some( vec![ "ci".to_string() ] ),
    "FT-10: the batch write must apply to every listed account",
  );
}

// ── FT-11: `.tags` lists the union, sorted, with counts (AC-11) ───────────────

/// FT-11 — `.tags` unions account tags and filter-file tags into sorted rows
/// with account/filter counts; the filter-only row is the typo-hazard surface;
/// an untagged store prints `(no tags)`; the command is read-only.
///
/// ## Setup
/// `a`,`b` carry `ci`; `c` carries `kimi_pool`; one `_filter_*` file with
/// include `[kimi_pool]`, exclude `[typo_tag]`.
///
/// ## Assert
/// Normalized rows `ci 2 0`, `kimi_pool 1 1`, `typo_tag 0 1` in sorted order;
/// store byte-identical after; untagged store → `(no tags)`, exit 0.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-11 (AC-11);
/// `tests/docs/cli/command/22_tags.md` IT-01/IT-02/IT-03
#[ test ]
fn account_tag_t11_tags_lists_union_sorted()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  for name in [ "a@test.com", "b@test.com", "c@test.com" ]
  {
    write_account( home, name, "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  }
  write_account_tags( home, "a@test.com", &[ "ci" ] );
  write_account_tags( home, "b@test.com", &[ "ci" ] );
  write_account_tags( home, "c@test.com", &[ "kimi_pool" ] );
  write_filter_file( home, "somehost_someuser", &[ "kimi_pool" ], &[ "typo_tag" ] );

  let before = store_snapshot( home );
  let out = run_cs_with_env( &[ ".tags" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  let lines = normalized_lines( &stdout( &out ) );
  let ci   = lines.iter().position( | l | l == "ci 2 0" );
  let kimi = lines.iter().position( | l | l == "kimi_pool 1 1" );
  let typo = lines.iter().position( | l | l == "typo_tag 0 1" );
  assert!(
    ci.is_some() && kimi.is_some() && typo.is_some(),
    "FT-11: rows 'ci 2 0', 'kimi_pool 1 1', 'typo_tag 0 1' must all be present; stdout: {}",
    stdout( &out ),
  );
  assert!(
    ci < kimi && kimi < typo,
    "FT-11: rows must be sorted by tag; stdout: {}", stdout( &out ),
  );
  assert_eq!(
    store_snapshot( home ), before,
    "FT-11: .tags is read-only — no store file may change",
  );

  let tmp2    = TempDir::new().unwrap();
  let home2   = tmp2.path();
  let home2_s = home2.to_str().unwrap();
  write_account( home2, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  let out = run_cs_with_env( &[ ".tags" ], &[ ( "HOME", home2_s ) ] );
  assert_exit( &out, 0 );
  assert_eq!(
    stdout( &out ).trim(), "(no tags)",
    "FT-11: an untagged store must print exactly '(no tags)'",
  );
}

// ── FT-12: `.tags format::json` shape (AC-12) ─────────────────────────────────

/// FT-12 — `format::json` emits an array of `{"tag","accounts","filters"}`
/// objects sorted by tag.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-12 (AC-12);
/// `tests/docs/cli/command/22_tags.md` IT-04
#[ test ]
fn account_tag_t12_tags_json_shape()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  for name in [ "a@test.com", "b@test.com", "c@test.com" ]
  {
    write_account( home, name, "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  }
  write_account_tags( home, "a@test.com", &[ "ci" ] );
  write_account_tags( home, "b@test.com", &[ "ci" ] );
  write_account_tags( home, "c@test.com", &[ "kimi_pool" ] );
  write_filter_file( home, "somehost_someuser", &[ "kimi_pool" ], &[ "typo_tag" ] );

  let out = run_cs_with_env( &[ ".tags", "format::json" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  let parsed : serde_json::Value = serde_json::from_str( &stdout( &out ) )
    .expect( "FT-12: .tags format::json must emit valid JSON" );
  let rows = parsed.as_array().expect( "FT-12: JSON output must be an array" );
  assert_eq!( rows.len(), 3, "FT-12: one row per distinct tag; got: {parsed}" );
  let tags : Vec< &str > = rows.iter().map( | r | r[ "tag" ].as_str().unwrap() ).collect();
  assert_eq!( tags, [ "ci", "kimi_pool", "typo_tag" ], "FT-12: rows must be sorted by tag" );
  assert_eq!( rows[ 0 ][ "accounts" ], 2, "FT-12: ci account count" );
  assert_eq!( rows[ 0 ][ "filters" ], 0, "FT-12: ci filter count" );
  assert_eq!( rows[ 1 ][ "accounts" ], 1, "FT-12: kimi_pool account count" );
  assert_eq!( rows[ 1 ][ "filters" ], 1, "FT-12: kimi_pool filter count" );
  assert_eq!( rows[ 2 ][ "accounts" ], 0, "FT-12: typo_tag account count (typo-hazard row)" );
  assert_eq!( rows[ 2 ][ "filters" ], 1, "FT-12: typo_tag filter count" );
}

// ── FT-13: `.accounts tags::` subset filter (AC-13) ───────────────────────────

/// FT-13 — `.accounts tags::a,b` lists only accounts carrying ALL listed tags.
///
/// ## Setup
/// `alice` `[ci, kimi_pool]`; `bob` `[ci]`; `carol` untagged.
///
/// ## Assert
/// `tags::ci,kimi_pool` → alice only; `tags::ci` → alice and bob, never carol.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-13 (AC-13)
#[ test ]
fn account_tag_t13_accounts_tags_subset_filter()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  for name in [ "alice@test.com", "bob@test.com", "carol@test.com" ]
  {
    write_account( home, name, "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  }
  write_account_tags( home, "alice@test.com", &[ "ci", "kimi_pool" ] );
  write_account_tags( home, "bob@test.com", &[ "ci" ] );

  let out = run_cs_with_env( &[ ".accounts", "tags::ci,kimi_pool" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "alice@test.com" ) && !text.contains( "bob@test.com" ) && !text.contains( "carol@test.com" ),
    "FT-13: tags::ci,kimi_pool must list only accounts carrying ALL listed tags; stdout: {text}",
  );

  let out = run_cs_with_env( &[ ".accounts", "tags::ci" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "alice@test.com" ) && text.contains( "bob@test.com" ) && !text.contains( "carol@test.com" ),
    "FT-13: tags::ci must list every account carrying ci; stdout: {text}",
  );
}

// ── FT-14: `Tags:` line rules and JSON array (AC-14) ──────────────────────────

/// FT-14 — text mode shows a `Tags:` line only for accounts with ≥1 tag;
/// `format::json` ALWAYS includes the `tags` array (even empty).
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-14 (AC-14)
#[ test ]
fn account_tag_t14_accounts_tags_line_and_json()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "alice@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account( home, "carol@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_tags( home, "alice@test.com", &[ "ci" ] );

  let out = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert_eq!(
    text.matches( "Tags:" ).count(), 1,
    "FT-14: exactly one Tags: line (alice tagged, carol untagged); stdout: {text}",
  );
  assert!(
    text.contains( "ci" ),
    "FT-14: the Tags: line must carry the tag value; stdout: {text}",
  );

  let out = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  let parsed : serde_json::Value = serde_json::from_str( &stdout( &out ) )
    .expect( "FT-14: .accounts format::json must emit valid JSON" );
  let rows = parsed.as_array().expect( "FT-14: JSON output must be an array" );
  for row in rows
  {
    let tags = row.get( "tags" ).and_then( | v | v.as_array() )
      .unwrap_or_else( || panic!( "FT-14: every JSON row must carry a tags array; row: {row}" ) );
    match row[ "name" ].as_str()
    {
      Some( "alice@test.com" ) => assert_eq!( tags.len(), 1, "FT-14: alice carries [ci]" ),
      Some( "carol@test.com" ) => assert!( tags.is_empty(), "FT-14: carol's array is empty, never absent" ),
      other => panic!( "FT-14: unexpected account row {other:?}" ),
    }
  }
}

// ── FT-15: `cols::+tags` opt-in column (AC-15) ────────────────────────────────

/// FT-15 — `cols::+tags` adds a Tags column to `.accounts`/`.usage` table
/// output; the column is in neither command's default set.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-15 (AC-15)
#[ test ]
fn account_tag_t15_cols_plus_tags_column()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_credentials( home, "max", "default_claude_max_20x", FAR_FUTURE_MS );
  write_account( home, "alice@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, true );
  write_account_tags( home, "alice@test.com", &[ "ci" ] );
  write_account_quota_cache( home, "alice@test.com", 20.0, 30.0, None );

  let out = run_cs_with_env( &[ ".accounts", "cols::+tags", "format::table" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "Tags" ),
    "FT-15: .accounts cols::+tags must add the Tags column; stdout: {}", stdout( &out ),
  );

  let out = run_cs_with_env( &[ ".accounts", "format::table" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  assert!(
    !stdout( &out ).contains( "Tags" ),
    "FT-15: the Tags column must not be in .accounts' default set; stdout: {}", stdout( &out ),
  );

  let out = run_cs_with_env( &[ ".usage", "cols::+tags" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "Tags" ),
    "FT-15: .usage cols::+tags must add the Tags column; stdout: {}", stdout( &out ),
  );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  assert!(
    !stdout( &out ).contains( "Tags" ),
    "FT-15: the Tags column must not be in .usage's default set; stdout: {}", stdout( &out ),
  );
}

// ── FT-16: zero-migration adoption (AC-16) ────────────────────────────────────

/// FT-16 — on a store never tag-written, pre-existing text surfaces are
/// byte-identical to pre-feature behavior and store files stay untouched.
///
/// ## Setup
/// Accounts carrying legacy `role` fields; no `tags` key anywhere; no filter file.
///
/// ## Assert
/// `.accounts` text has no `Tags:` line; `.usage` has no exclusion note; every
/// store file is byte-identical after both commands; `role` fields survive.
/// The one deliberate carve-out (AC-14): `format::json` always carries a
/// `tags` array — an ADDITIVE key on a machine surface, asserted here as the
/// documented exception rather than a byte-identity violation.
///
/// Spec: `tests/docs/feature/075_account_tags.md` FT-16 (AC-16)
#[ test ]
fn account_tag_t16_untagged_store_byte_identical()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_credentials( home, "max", "default_claude_max_20x", FAR_FUTURE_MS );
  write_account( home, "alice@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, true );
  write_account( home, "bob@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_profile_json( home, "alice@test.com", None, Some( "Work" ) );
  write_account_quota_cache( home, "bob@test.com", 20.0, 30.0, None );

  let before = store_snapshot( home );

  let out = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  assert!(
    !stdout( &out ).contains( "Tags:" ),
    "FT-16: an untagged store must render no Tags: line; stdout: {}", stdout( &out ),
  );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  assert!(
    !stdout( &out ).contains( "excluded by tag filter" ),
    "FT-16: no filter file → no exclusion note; stdout: {}", stdout( &out ),
  );

  assert_eq!(
    store_snapshot( home ), before,
    "FT-16: read paths must leave every store file byte-identical",
  );
  assert_eq!(
    read_account_meta( home, "alice@test.com" )[ "role" ], "Work",
    "FT-16: legacy role fields must survive untouched until a tag write",
  );

  let out = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 0 );
  let parsed : serde_json::Value = serde_json::from_str( &stdout( &out ) ).unwrap();
  for row in parsed.as_array().unwrap()
  {
    assert!(
      row.get( "tags" ).is_some_and( | t | t.as_array().is_some_and( Vec::is_empty ) ),
      "FT-16/AC-14 carve-out: JSON rows carry an (empty) tags array even on an untagged store; row: {row}",
    );
  }
}

// ── IT-05: `.tags` unsupported format exits 1 ─────────────────────────────────

/// IT-05 — `.tags format::table` exits 1; stderr states `format::` must be
/// `text` or `json`.
///
/// Spec: `tests/docs/cli/command/22_tags.md` IT-05
#[ test ]
fn account_tag_t17_tags_bad_format_exits_1()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".tags", "format::table" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "format::" ) && err.contains( "text" ) && err.contains( "json" ),
    "IT-05: rejection must state format:: must be text or json; stderr: {err}",
  );
}

// ── IT-07: `.account.tag` unknown account exits 2 ─────────────────────────────

/// IT-07 — an unknown account exits 2; in a batch, ALL names resolve before
/// ANY mutation, so the known account stays untouched.
///
/// Spec: `tests/docs/cli/command/25_account_tag.md` IT-07
#[ test ]
fn account_tag_t19_account_tag_unknown_account_exits_2()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "alice@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_tags( home, "alice@test.com", &[ "ci" ] );

  let out = run_cs_with_env(
    &[ ".account.tag", "name::ghost@test.com", "add::ci" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 2 );

  let out = run_cs_with_env(
    &[ ".account.tag", "name::alice@test.com,ghost@test.com", "add::kimi_pool" ],
    &[ ( "HOME", home_s ) ],
  );
  assert_exit( &out, 2 );
  assert_eq!(
    meta_tags( home, "alice@test.com" ), Some( vec![ "ci".to_string() ] ),
    "IT-07: batch must resolve every name before mutating any account",
  );
}

// ── IT-08: `.account.tag` missing name exits 1 ────────────────────────────────

/// IT-08 — `.account.tag add::ci` without `name::` exits 1 (usage error).
///
/// Spec: `tests/docs/cli/command/25_account_tag.md` IT-08
#[ test ]
fn account_tag_t20_account_tag_missing_name_exits_1()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "alice@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".account.tag", "add::ci" ], &[ ( "HOME", home_s ) ] );
  assert_exit( &out, 1 );
}

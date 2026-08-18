//! Integration tests: Feature 076 — Identity tag filter (`.identity.filter`
//! get/set/clear, `.identities` listing, Gate 11 CLI binding, `.account.use`
//! bypass, loud exclusion reporting).
//!
//! Tests invoke the compiled `clp` binary as a subprocess with controlled
//! `HOSTNAME`/`USER` env — the filter filename derives from the child's
//! Identity — against a temporary isolated credential store.
//!
//! ## Test Matrix
//!
//! | ID    | Test Function                                          | AC    |
//! |-------|--------------------------------------------------------|-------|
//! | FT-01 | `identity_filter_t01_filter_get_permit_all`            | AC-01 |
//! | FT-02 | `identity_filter_t02_filter_include_write`             | AC-02 |
//! | FT-03 | `identity_filter_t03_filter_both_sides_replace`        | AC-03 |
//! | FT-04 | `identity_filter_t04_filter_overlap_exits_1`           | AC-04 |
//! | FT-05 | `identity_filter_t05_filter_invalid_tag_exits_1`       | AC-05 |
//! | FT-06 | `identity_filter_t06_filter_clear_idempotent_and_exclusive` | AC-06 |
//! | FT-07 | `identity_filter_t07_filter_identity_targeting`        | AC-07 |
//! | FT-08 | `identity_filter_t08_filter_typo_guard_warns`          | AC-08 |
//! | FT-10 | `identity_filter_t10_account_use_ignores_filter`       | AC-10 |
//! | FT-13 | `identity_filter_t13_usage_reports_excluded_count`     | AC-13 |
//! | FT-14 | `identity_filter_t14_identities_lists_union`           | AC-14 |
//! | FT-15 | `identity_filter_t15_identity_commands_json`           | AC-15 |
//! | FT-16 | `identity_filter_t16_filter_filename_derivation`       | AC-16 |
//! | IT-05 | `identity_filter_t18_identities_filename_derived_row`  | AC-14 |
//!
//! Spec: `tests/docs/feature/076_identity_tag_filter.md`,
//! `tests/docs/cli/command/23_identities.md`, `tests/docs/cli/command/24_identity_filter.md`.

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, write_account_owner,
  write_account_quota_cache, write_account_tags,
  write_filter_file, write_active_marker,
  credential_store_dir, normalized_lines,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

/// The controlled child Identity every non-derivation test runs under.
const TEST_HOST : &str = "testhost";
/// See [`TEST_HOST`].
const TEST_USER : &str = "testuser";
/// `_filter_*`/`_active_*` slug for [`TEST_HOST`]/[`TEST_USER`].
const TEST_SLUG : &str = "testhost_testuser";

/// Child env: isolated HOME plus the controlled test Identity.
fn id_env( home_s : &str ) -> [ ( &str, &str ) ; 3 ]
{
  [ ( "HOME", home_s ), ( "HOSTNAME", TEST_HOST ), ( "USER", TEST_USER ) ]
}

/// Path of the `_filter_{slug}` file under a test HOME.
fn filter_path( home : &std::path::Path, slug : &str ) -> std::path::PathBuf
{
  credential_store_dir( home ).join( format!( "_filter_{slug}" ) )
}

/// Parse a `_filter_*` file's `include`/`exclude` arrays as plain strings.
fn read_filter( home : &std::path::Path, slug : &str ) -> ( Vec< String >, Vec< String > )
{
  let text = std::fs::read_to_string( filter_path( home, slug ) ).unwrap();
  let v : serde_json::Value = serde_json::from_str( &text ).unwrap();
  let side = | key : &str | v[ key ].as_array().unwrap().iter()
    .filter_map( | t | t.as_str().map( str::to_string ) ).collect::< Vec< _ > >();
  ( side( "include" ), side( "exclude" ) )
}

// ── FT-01: get with no filter file is permit-all (AC-01) ──────────────────────

/// FT-01 — `.identity.filter` with no filter file prints the permit-all state
/// and creates nothing.
///
/// ## Assert
/// Stdout (trimmed) is exactly `include=[] exclude=[] (permit-all)`; exit 0;
/// no `_filter_*` file appears.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-01 (AC-01)
#[ test ]
fn identity_filter_t01_filter_get_permit_all()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".identity.filter" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  assert_eq!(
    stdout( &out ).trim(), "include=[] exclude=[] (permit-all)",
    "FT-01: absent filter must read as permit-all",
  );
  assert!(
    !filter_path( home, TEST_SLUG ).exists(),
    "FT-01: a get must never create the filter file",
  );
}

// ── FT-02: `include::` writes sorted deduplicated set (AC-02) ─────────────────

/// FT-02 — `include::kimi_pool,ci,kimi_pool` stores
/// `{"include": ["ci", "kimi_pool"], "exclude": []}` under `_filter_{slug}`.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-02 (AC-02)
#[ test ]
fn identity_filter_t02_filter_include_write()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_tags( home, "a@test.com", &[ "ci", "kimi_pool" ] );

  let out = run_cs_with_env( &[ ".identity.filter", "include::kimi_pool,ci,kimi_pool" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  let ( include, exclude ) = read_filter( home, TEST_SLUG );
  assert_eq!(
    include, [ "ci", "kimi_pool" ],
    "FT-02: include side must be stored sorted + deduplicated",
  );
  assert!( exclude.is_empty(), "FT-02: unset exclude side must be stored empty" );
}

// ── FT-03: each given side fully replaces (AC-03) ─────────────────────────────

/// FT-03 — a given side fully replaces that side; the omitted side survives;
/// both sides replace when both are given in one invocation.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-03 (AC-03)
#[ test ]
fn identity_filter_t03_filter_both_sides_replace()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_tags( home, "a@test.com", &[ "a", "b", "ci", "kimi_pool", "personal" ] );
  write_filter_file( home, TEST_SLUG, &[ "ci" ], &[ "personal" ] );

  let out = run_cs_with_env( &[ ".identity.filter", "include::kimi_pool" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  assert_eq!(
    read_filter( home, TEST_SLUG ), ( vec![ "kimi_pool".to_string() ], vec![ "personal".to_string() ] ),
    "FT-03: include replaced, omitted exclude side preserved",
  );

  let out = run_cs_with_env( &[ ".identity.filter", "include::a", "exclude::b" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  assert_eq!(
    read_filter( home, TEST_SLUG ), ( vec![ "a".to_string() ], vec![ "b".to_string() ] ),
    "FT-03: both sides replaced when both given in one invocation",
  );
}

// ── FT-04: include/exclude overlap exits 1 (AC-04) ────────────────────────────

/// FT-04 — a write producing non-empty `include ∩ exclude` exits 1 naming the
/// overlap; the file is unchanged (or still absent).
///
/// ## Assert
/// `include::a exclude::a` on a fresh store → exit 1, no file; `include::x`
/// against an existing `{"exclude": ["x"]}` → exit 1, file byte-identical.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-04 (AC-04)
#[ test ]
fn identity_filter_t04_filter_overlap_exits_1()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".identity.filter", "include::a", "exclude::a" ], &id_env( home_s ) );
  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "overlap" ),
    "FT-04: rejection must name the overlap; stderr: {}", stderr( &out ),
  );
  assert!(
    !filter_path( home, TEST_SLUG ).exists(),
    "FT-04: a rejected contradictory write must create nothing",
  );

  write_filter_file( home, TEST_SLUG, &[], &[ "x" ] );
  let before = std::fs::read( filter_path( home, TEST_SLUG ) ).unwrap();
  let out = run_cs_with_env( &[ ".identity.filter", "include::x" ], &id_env( home_s ) );
  assert_exit( &out, 1 );
  assert_eq!(
    std::fs::read( filter_path( home, TEST_SLUG ) ).unwrap(), before,
    "FT-04: an overlap against the preserved side must leave the file unchanged",
  );
}

// ── FT-05: invalid tag in either set exits 1 (AC-05) ──────────────────────────

/// FT-05 — an invalid tag in either side exits 1 naming its post-lowercased
/// form; nothing written.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-05 (AC-05)
#[ test ]
fn identity_filter_t05_filter_invalid_tag_exits_1()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );

  for args in [ [ ".identity.filter", "include::Bad!Tag" ], [ ".identity.filter", "exclude::Bad!Tag" ] ]
  {
    let out = run_cs_with_env( &args, &id_env( home_s ) );
    assert_exit( &out, 1 );
    assert!(
      stderr( &out ).contains( "bad!tag" ),
      "FT-05: rejection must name the post-lowercased tag; stderr: {}", stderr( &out ),
    );
  }
  assert!(
    !filter_path( home, TEST_SLUG ).exists(),
    "FT-05: a rejected write must create nothing",
  );
}

// ── FT-06: `clear::1` deletes, idempotent, excludes set params (AC-06) ────────

/// FT-06 — `clear::1` deletes the filter file, succeeds again when already
/// absent, and combined with `include::`/`exclude::` exits 1.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-06 (AC-06)
#[ test ]
fn identity_filter_t06_filter_clear_idempotent_and_exclusive()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_filter_file( home, TEST_SLUG, &[ "ci" ], &[] );

  let out = run_cs_with_env( &[ ".identity.filter", "clear::1" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  assert!(
    !filter_path( home, TEST_SLUG ).exists(),
    "FT-06: clear::1 must delete the filter file",
  );

  let out = run_cs_with_env( &[ ".identity.filter", "clear::1" ], &id_env( home_s ) );
  assert_exit( &out, 0 );

  let out = run_cs_with_env( &[ ".identity.filter", "clear::1", "include::ci" ], &id_env( home_s ) );
  assert_exit( &out, 1 );
}

// ── FT-07: `identity::` targets another seat (AC-07) ──────────────────────────

/// FT-07 — `identity::bob@laptop` routes get/set/clear to `_filter_laptop_bob`
/// and never touches the current Identity's file; a malformed `identity::`
/// (not exactly one `@`, both halves non-empty) exits 1.
///
/// ## Setup
/// Current Identity `alice@desk` (env `HOSTNAME=desk`, `USER=alice`).
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-07 (AC-07)
#[ test ]
fn identity_filter_t07_filter_identity_targeting()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_tags( home, "a@test.com", &[ "ci" ] );
  let env = [ ( "HOME", home_s ), ( "HOSTNAME", "desk" ), ( "USER", "alice" ) ];

  let out = run_cs_with_env( &[ ".identity.filter", "identity::bob@laptop", "include::ci" ], &env );
  assert_exit( &out, 0 );
  assert_eq!(
    read_filter( home, "laptop_bob" ), ( vec![ "ci".to_string() ], Vec::new() ),
    "FT-07: identity:: must write the targeted seat's file",
  );
  assert!(
    !filter_path( home, "desk_alice" ).exists(),
    "FT-07: the current Identity's file must stay untouched",
  );

  let out = run_cs_with_env( &[ ".identity.filter", "identity::bob@laptop" ], &env );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "include=[ci]" ),
    "FT-07: identity:: get must read the targeted seat's filter; stdout: {}", stdout( &out ),
  );

  let out = run_cs_with_env( &[ ".identity.filter", "identity::bob@laptop", "clear::1" ], &env );
  assert_exit( &out, 0 );
  assert!(
    !filter_path( home, "laptop_bob" ).exists(),
    "FT-07: identity:: clear must delete the targeted seat's file",
  );

  for bad in [ "identity::bob", "identity::b@b@b", "identity::@laptop", "identity::bob@" ]
  {
    let out = run_cs_with_env( &[ ".identity.filter", bad, "include::ci" ], &env );
    assert_exit( &out, 1 );
  }
}

// ── FT-08: typo guard warns on zero-match include (AC-08) ─────────────────────

/// FT-08 — an `include::` naming a tag carried by no account still writes
/// (exit 0) but warns on stderr naming the unmatched tag.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-08 (AC-08)
#[ test ]
fn identity_filter_t08_filter_typo_guard_warns()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_tags( home, "a@test.com", &[ "ci" ] );

  let out = run_cs_with_env( &[ ".identity.filter", "include::typo_tag" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  assert_eq!(
    read_filter( home, TEST_SLUG ).0, [ "typo_tag" ],
    "FT-08: the guard warns — it must not block the write",
  );
  assert!(
    stderr( &out ).contains( "typo_tag" ),
    "FT-08: the warning must name the tag carried by no account; stderr: {}", stderr( &out ),
  );
}

// ── FT-10: `.account.use name::X` is never filtered (AC-10) ───────────────────

/// FT-10 — explicit selection bypasses Gate 11 entirely: `.account.use` on an
/// account the current filter excludes still switches.
///
/// ## Setup
/// Filter `include=[kimi_pool]`; target `x@test.com` untagged (fails include);
/// marker initially names another account.
///
/// ## Assert
/// Exit 0; the active marker CHANGED to `x@test.com`; live credentials replaced.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-10 (AC-10)
#[ test ]
fn identity_filter_t10_account_use_ignores_filter()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_credentials( home, "max", "default_claude_max_20x", FAR_FUTURE_MS );
  write_account( home, "x@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_active_marker( home, TEST_SLUG, "other@test.com" );
  write_filter_file( home, TEST_SLUG, &[ "kimi_pool" ], &[] );

  let out = run_cs_with_env( &[ ".account.use", "name::x@test.com", "touch::0" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  let marker = std::fs::read_to_string(
    credential_store_dir( home ).join( format!( "_active_{TEST_SLUG}" ) ) ).unwrap();
  assert_eq!(
    marker.trim(), "x@test.com",
    "FT-10: explicit .account.use must switch despite the filter (marker must CHANGE)",
  );
  let live = std::fs::read_to_string( home.join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  let stored = std::fs::read_to_string(
    credential_store_dir( home ).join( "x@test.com.credentials.json" ) ).unwrap();
  assert_eq!(
    live, stored,
    "FT-10: live credentials must be replaced by the target account's",
  );
}

// ── FT-13: loud exclusion reporting on `.usage` (AC-13) ───────────────────────

/// FT-13 — when Gate 11 excludes ≥1 account, `.usage` text output carries
/// `N excluded by tag filter include=[…] exclude=[…]`; rotation picks only a
/// filter-passing winner; without a filter file no such line appears.
///
/// ## Setup
/// `current` (active) and `winner` tagged `kimi_pool`; `loser` tagged `ci`,
/// otherwise fully eligible (fresh cache, unowned, unlocked); filter
/// `include=[kimi_pool]`.
///
/// ## Assert
/// `.usage` prints `1 excluded by tag filter` naming the include set;
/// `.usage rotate::1` switches to `winner@test.com` (never `loser`); a
/// filterless twin store prints no exclusion line.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-13 (AC-13/AC-09 CLI binding)
#[ test ]
fn identity_filter_t13_usage_reports_excluded_count()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_credentials( home, "max", "default_claude_max_20x", FAR_FUTURE_MS );
  write_account( home, "current@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account( home, "winner@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account( home, "loser@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_active_marker( home, TEST_SLUG, "current@test.com" );
  write_account_quota_cache( home, "winner@test.com", 20.0, 30.0, None );
  write_account_quota_cache( home, "loser@test.com", 10.0, 20.0, None );
  write_account_tags( home, "current@test.com", &[ "kimi_pool" ] );
  write_account_tags( home, "winner@test.com", &[ "kimi_pool" ] );
  write_account_tags( home, "loser@test.com", &[ "ci" ] );
  write_filter_file( home, TEST_SLUG, &[ "kimi_pool" ], &[] );

  let out = run_cs_with_env( &[ ".usage" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "1 excluded by tag filter" ) && text.contains( "include=[kimi_pool]" ),
    "FT-13: the exclusion note must carry count and sets; stdout: {text}",
  );

  let out = run_cs_with_env( &[ ".usage", "rotate::1" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched to 'winner@test.com'" ),
    "FT-13/AC-09: rotation must pick the filter-passing account only; stdout: {}", stdout( &out ),
  );

  let bare_tmp    = TempDir::new().unwrap();
  let bare_home   = bare_tmp.path();
  let bare_home_s = bare_home.to_str().unwrap();
  write_credentials( bare_home, "max", "default_claude_max_20x", FAR_FUTURE_MS );
  write_account( bare_home, "current@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account( bare_home, "loser@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_active_marker( bare_home, TEST_SLUG, "current@test.com" );
  write_account_quota_cache( bare_home, "loser@test.com", 10.0, 20.0, None );
  write_account_tags( bare_home, "loser@test.com", &[ "ci" ] );

  let out = run_cs_with_env( &[ ".usage" ], &id_env( bare_home_s ) );
  assert_exit( &out, 0 );
  assert!(
    !stdout( &out ).contains( "excluded by tag filter" ),
    "FT-13: no filter file → no exclusion note; stdout: {}", stdout( &out ),
  );
}

// ── FT-14: `.identities` unions all three sources (AC-14) ─────────────────────

/// FT-14 — `.identities` unions active markers, filter files, and `owner`
/// fields into one sorted row per Identity; an empty union prints
/// `(no identities)`; the command is read-only.
///
/// ## Setup
/// Marker `_active_desk_alice` → `alice@acme.com`; filter `_filter_laptop_bob`
/// include `[ci]`; account owned by `carol@ws1`.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-14 (AC-14)
#[ test ]
fn identity_filter_t14_identities_lists_union()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_active_marker( home, "desk_alice", "alice@acme.com" );
  write_filter_file( home, "laptop_bob", &[ "ci" ], &[] );
  write_account_owner( home, "a@test.com", "carol@ws1" );

  let out = run_cs_with_env( &[ ".identities" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  let lines = normalized_lines( &stdout( &out ) );
  let alice = lines.iter().position( | l | l == "alice@desk alice@acme.com 0 — —" );
  let bob   = lines.iter().position( | l | l == "bob@laptop — 0 ci —" );
  let carol = lines.iter().position( | l | l == "carol@ws1 — 1 — —" );
  assert!(
    alice.is_some() && bob.is_some() && carol.is_some(),
    "FT-14: one row per Identity from each source; stdout: {}", stdout( &out ),
  );
  assert!(
    alice < bob && bob < carol,
    "FT-14: rows must be sorted by Identity; stdout: {}", stdout( &out ),
  );

  let bare_tmp    = TempDir::new().unwrap();
  let bare_home   = bare_tmp.path();
  let bare_home_s = bare_home.to_str().unwrap();
  write_account( bare_home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  let out = run_cs_with_env( &[ ".identities" ], &id_env( bare_home_s ) );
  assert_exit( &out, 0 );
  assert_eq!(
    stdout( &out ).trim(), "(no identities)",
    "FT-14: an empty union must print exactly '(no identities)'",
  );
}

// ── FT-15: `format::json` on both commands; other formats exit 1 (AC-15) ──────

/// FT-15 — `.identities format::json` emits an array of
/// `{"identity","active","owned","include","exclude"}`; `.identity.filter
/// format::json` emits `{"identity","include","exclude"}`; `format::table`
/// exits 1 on both.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-15 (AC-15)
#[ test ]
fn identity_filter_t15_identity_commands_json()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_tags( home, "a@test.com", &[ "ci" ] );
  write_active_marker( home, "desk_alice", "alice@acme.com" );
  write_filter_file( home, TEST_SLUG, &[ "ci" ], &[] );

  let out = run_cs_with_env( &[ ".identities", "format::json" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  let parsed : serde_json::Value = serde_json::from_str( &stdout( &out ) )
    .expect( "FT-15: .identities format::json must emit valid JSON" );
  let rows = parsed.as_array().expect( "FT-15: JSON output must be an array" );
  assert!( !rows.is_empty(), "FT-15: fixture identities must produce rows" );
  for row in rows
  {
    for key in [ "identity", "active", "owned", "include", "exclude" ]
    {
      assert!(
        row.get( key ).is_some(),
        "FT-15: every row must carry '{key}'; row: {row}",
      );
    }
  }

  let out = run_cs_with_env( &[ ".identity.filter", "format::json" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  let parsed : serde_json::Value = serde_json::from_str( &stdout( &out ) )
    .expect( "FT-15: .identity.filter format::json must emit valid JSON" );
  assert_eq!(
    parsed[ "identity" ].as_str(), Some( "testuser@testhost" ),
    "FT-15: the filter JSON must name its Identity; got: {parsed}",
  );
  assert_eq!(
    parsed[ "include" ][ 0 ].as_str(), Some( "ci" ),
    "FT-15: the filter JSON must carry the include set; got: {parsed}",
  );

  for cmd in [ ".identities", ".identity.filter" ]
  {
    let out = run_cs_with_env( &[ cmd, "format::table" ], &id_env( home_s ) );
    assert_exit( &out, 1 );
    let err = stderr( &out );
    assert!(
      err.contains( "format::" ) && err.contains( "text" ) && err.contains( "json" ),
      "FT-15: {cmd} must reject format::table naming text/json; stderr: {err}",
    );
  }
}

// ── FT-16: filename derivation and sync intent (AC-16) ────────────────────────

/// FT-16 — the filter filename is `_filter_{machine}_{user}` with the exact
/// `_active_*` marker sanitization (keep `[a-zA-Z0-9.-]`, replace others with
/// `_`), and never matches the `_active_*` pattern (store-sync intent).
///
/// ## Setup
/// Child Identity `john doe`@`ws 1` — both halves need sanitization.
///
/// Spec: `tests/docs/feature/076_identity_tag_filter.md` FT-16 (AC-16/AC-08 parity)
#[ test ]
fn identity_filter_t16_filter_filename_derivation()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_account_tags( home, "a@test.com", &[ "ci" ] );
  let env = [ ( "HOME", home_s ), ( "HOSTNAME", "ws 1" ), ( "USER", "john doe" ) ];

  let out = run_cs_with_env( &[ ".identity.filter", "include::ci" ], &env );
  assert_exit( &out, 0 );
  let expected = filter_path( home, "ws_1_john_doe" );
  assert!(
    expected.exists(),
    "FT-16: the filename must sanitize spaces to '_' exactly like the active marker",
  );
  let names : Vec< String > = std::fs::read_dir( credential_store_dir( home ) ).unwrap()
    .filter_map( Result::ok )
    .map( | e | e.file_name().to_string_lossy().to_string() )
    .collect();
  assert!(
    !names.iter().any( | n | n.starts_with( "_active_" ) ),
    "FT-16: a filter write must never produce an _active_* name (sync-intent split); store: {names:?}",
  );
}

// ── IT-05 (23_identities.md): filename-derived row for unmatched marker ───────

/// IT-05 — a marker whose Identity appears in no `owner` field still yields a
/// row, derived by last-`_` split of the filename suffix (sanitized display).
///
/// ## Setup
/// Marker `_active_devbox_dave` → `dave@x.com`; no owners, no filters.
///
/// Spec: `tests/docs/cli/command/23_identities.md` IT-05
#[ test ]
fn identity_filter_t18_identities_filename_derived_row()
{
  let tmp    = TempDir::new().unwrap();
  let home   = tmp.path();
  let home_s = home.to_str().unwrap();
  write_account( home, "a@test.com", "max", "default_claude_max_20x", FAR_FUTURE_MS, false );
  write_active_marker( home, "devbox_dave", "dave@x.com" );

  let out = run_cs_with_env( &[ ".identities" ], &id_env( home_s ) );
  assert_exit( &out, 0 );
  let lines = normalized_lines( &stdout( &out ) );
  assert!(
    lines.iter().any( | l | l == "dave@devbox dave@x.com 0 — —" ),
    "IT-05: the marker's Identity must be derived by last-'_' split; stdout: {}", stdout( &out ),
  );
}

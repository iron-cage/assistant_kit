//! Integration tests: ACC `.accounts` list — Part B (acc26–acc52+).
//!
//! Continuation of `accounts_list_test.rs`.

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token, write_credentials, write_claude_json_full,
  write_settings_json,
  write_live_credentials_with_token, write_account_claude_json_extended,
  write_account_roles_json, write_account_profile_json,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

/// acc26 (T09 — save with claude.json + settings.json): `account::save` writes credential
/// and `.json` snapshot files when all sources are present (BUG-222 fix).
///
/// Root Cause (before fix): `save()` only called `std::fs::copy(paths.credentials_file(), dest)`.
///   The `oauthAccount` data from `~/.claude.json` was never persisted to the credential store.
/// Why Not Caught: No save test verified the presence of snapshot files after save.
/// Fix Applied: `save()` surgically extracts the `oauthAccount` subtree from `~/.claude.json`
///   and writes it to `{name}.json`. BUG-222 additionally captures `model` from
///   `~/.claude/settings.json` into `{name}.json`, enabling model preference restore
///   on `switch_account()`. When settings.json source is absent, no snapshot is written (acc28).
/// Prevention: After any `save()` implementation change, verify ALL expected output files exist.
///   `settings.json` snapshot must be created when source model is present; absent when source
///   is absent (see acc28 for the no-source case).
/// Pitfall: The `oauthAccount` extraction silently skips if the key is absent — this is
///   intentional best-effort, but means a wrong source path would silently produce no output.
///   This test catches that by asserting `{name}.json` EXISTS after save.
#[ test ]
fn acc26_save_creates_snapshot_files()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Source files that save() will copy.
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_full( dir.path(), "alice@acme.com", "Alice K", "admin", "stripe" );
  write_settings_json( dir.path(), "claude-sonnet" );

  let out  = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!(
    store.join( "alice@acme.com.credentials.json" ).exists(),
    "save must create credentials snapshot, store: {}", store.display(),
  );
  assert!(
    store.join( "alice@acme.com.json" ).exists(),
    "save must create .json snapshot, store: {}", store.display(),
  );
  // Fix(BUG-222): save() captures model from ~/.claude/settings.json into {name}.json.
  //   write_settings_json above created the source with "claude-sonnet" → snapshot must exist.
  let settings_snap = store.join( "alice@acme.com.json" );
  assert!(
    settings_snap.exists(),
    "save must create settings.json snapshot when source model present (BUG-222), store: {}", store.display(),
  );
  let settings_content = std::fs::read_to_string( &settings_snap ).unwrap();
  assert!(
    settings_content.contains( "claude-sonnet" ),
    "settings.json snapshot must contain the source model value, got: {settings_content}",
  );
}

/// acc27 (T09 — save without `~/.claude.json`): save succeeds even when source is absent.
///
/// Root Cause (before fix): `save()` only copied credentials; no `.json` snapshot
///   was ever created. After BUG-174, oauthAccount extraction was added but must silently
///   skip if `~/.claude.json` is absent or unparseable.
/// Why Not Caught: All prior save tests relied on a credentials file being present;
///   no test verified the best-effort behaviour for the optional `.json` source.
/// Fix Applied: `save()` wraps the oauthAccount extraction in `if let Ok(text) = read_to_string(...)` —
///   absent or malformed `~/.claude.json` silently skips; credential copy still uses `?` (required).
/// Prevention: For every best-effort file operation, add a test where the source is absent
///   to confirm the operation succeeds and no partial output is written.
/// Pitfall: Silently-discarded read errors mean a wrong path never fails — always add an
///   absent-source test to confirm no snapshot is created when source is missing.
#[ test ]
fn acc27_save_succeeds_without_claude_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Only credentials file — no ~/.claude.json, no settings.json.
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out  = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!(
    store.join( "alice@acme.com.credentials.json" ).exists(),
    "save must still create credential file when snapshots absent",
  );
  // Unified {{name}}.json is always created but must not contain oauthAccount (no source).
  let meta = std::fs::read_to_string( store.join( "alice@acme.com.json" ) )
    .unwrap_or_default();
  assert!(
    !meta.contains( "oauthAccount" ),
    "no ~/.claude.json → {{name}}.json must not contain oauthAccount, got: {meta}",
  );
}

/// acc28 (T09 — save with `.claude.json` but without `settings.json`): confirms oauthAccount
/// extraction succeeds when `settings.json` source is absent; `{name}.json` is created
/// with oauthAccount but no `model` field when `settings.json` has no model.
///
/// Root Cause (before fix): After the initial snapshot feature was added, `save()` tried
///   to read `settings.json`; a missing file could interfere. BUG-222 made model capture
///   best-effort: `save()` reads `model` from `~/.claude/settings.json` when present.
/// Why Not Caught: No test verified that `settings.json` absence did not affect the
///   `{name}.json` snapshot creation.
/// Fix Applied (BUG-222): `save()` merges `model` from `~/.claude/settings.json` into
///   `{name}.json` when present. When source is absent, `{name}.json` is still written
///   with whatever other data is available (e.g. oauthAccount).
/// Prevention: Verify `{name}.json` contains oauthAccount but no model when source absent.
/// Pitfall: `save()` only captures the `model` field, not the entire `settings.json`;
///   machine-global keys (commands.*, mcpServers) are never stored per-account.
#[ test ]
fn acc28_save_succeeds_without_settings_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Credentials + .claude.json present; settings.json absent.
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_full( dir.path(), "alice@acme.com", "Alice K", "admin", "stripe" );

  let out  = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!(
    store.join( "alice@acme.com.credentials.json" ).exists(),
    "save must create credential snapshot",
  );
  let meta = std::fs::read_to_string( store.join( "alice@acme.com.json" ) )
    .expect( "save must create {{name}}.json" );
  assert!(
    meta.contains( "oauthAccount" ),
    "{{name}}.json must contain oauthAccount from ~/.claude.json, got: {meta}",
  );
  assert!(
    !meta.contains( "\"model\"" ),
    "no settings.json source → {{name}}.json must not contain model, got: {meta}",
  );
}

// ── acc29 ─────────────────────────────────────────────────────────────────────

#[ test ]
fn acc29_accounts_positional_bare_arg()
{
  // AC-03: positional form `clp .accounts alice@acme.com` is equivalent to
  // `clp .accounts name::alice@acme.com` — shows only that account block.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".accounts", "alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com" ), "must show alice@acme.com, got:\n{text}" );
  assert!( !text.contains( "work@acme.com" ), "must not show work@acme.com, got:\n{text}" );
}

// ── acc30 ─────────────────────────────────────────────────────────────────────

#[ test ]
fn acc30_accounts_prefix_resolves()
{
  // AC-05 (accounts): prefix `alice` resolves to `alice@acme.com` — shows only that block.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, false );
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, true  );

  let out = run_cs_with_env( &[ ".accounts", "alice" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com" ), "prefix alice must resolve to alice@acme.com, got:\n{text}" );
  assert!( !text.contains( "work@acme.com" ), "must not show work@acme.com, got:\n{text}" );
}

// ── acc31 ─────────────────────────────────────────────────────────────────────

/// acc31 (IT-26): live creds `accessToken` matches `work@acme.com` → `Current: yes` on that
/// account and `Current: no` on `alice@acme.com`.
///
/// Both saved accounts have `accessToken` fields (via `write_account_with_token`) so
/// `detect_current_account()` can compare them against the live creds.
#[ test ]
fn acc31_accounts_shows_current_yes_no()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "work@acme.com",  "tok-work",  false );
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", false );
  write_live_credentials_with_token( dir.path(), "tok-work" );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // Each account block: find work@ and alice@ sections and verify Current: line.
  let lines : Vec< &str > = text.lines().collect();
  let work_idx  = lines.iter().position( |l| l.contains( "work@acme.com"  ) );
  let alice_idx = lines.iter().position( |l| l.contains( "alice@acme.com" ) );

  let work_idx  = work_idx.expect( "work@acme.com not found in output" );
  let alice_idx = alice_idx.expect( "alice@acme.com not found in output" );

  // Find Current: line near each account header (within the next 10 lines).
  let work_block  = &lines[ work_idx  ..( work_idx  + 10 ).min( lines.len() ) ];
  let alice_block = &lines[ alice_idx ..( alice_idx + 10 ).min( lines.len() ) ];

  assert!(
    work_block.iter().any( |l| l.contains( "Current:" ) && l.contains( "yes" ) ),
    "work@acme.com block must have 'Current: yes', got block:\n{}", work_block.join( "\n" ),
  );
  assert!(
    alice_block.iter().any( |l| l.contains( "Current:" ) && l.contains( "no" ) ),
    "alice@acme.com block must have 'Current: no', got block:\n{}", alice_block.join( "\n" ),
  );
}

// ── acc32 ─────────────────────────────────────────────────────────────────────

/// acc32 (IT-27): no live credentials file → `Current:` line is suppressed entirely.
///
/// When `~/.claude/.credentials.json` is absent, the detection algorithm cannot match
/// any account and the `Current:` line must not appear at all (not even `Current: no`).
#[ test ]
fn acc32_accounts_suppresses_current_when_creds_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, false );
  // Deliberately do NOT write live credentials file.

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    !text.contains( "Current:" ),
    "Current: line must be absent when creds file is missing, got:\n{text}",
  );
}

// ── acc33 ─────────────────────────────────────────────────────────────────────

/// acc33 (IT-28): two sub-tests for the `current::` parameter and JSON `is_current` field.
///
/// (a) `current::0` suppresses the `Current:` line even when live creds are present.
/// (b) `format::json` output includes an `is_current` boolean field per account object.
#[ test ]
fn acc33_accounts_current_param_and_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", "tok-alice", false );
  write_live_credentials_with_token( dir.path(), "tok-alice" );

  // (a) cols::-current must suppress the Current: line.
  let out_off = run_cs_with_env( &[ ".accounts", "cols::-current" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_off, 0 );
  let text_off = stdout( &out_off );
  assert!(
    !text_off.contains( "Current:" ),
    "cols::-current must suppress Current: line, got:\n{text_off}",
  );

  // (b) format::json must include is_current boolean field.
  let out_json = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_json, 0 );
  let json = stdout( &out_json );
  assert!(
    json.contains( "\"is_current\"" ),
    "JSON output must include is_current field, got:\n{json}",
  );
}

// ── acc34 ─────────────────────────────────────────────────────────────────────

/// acc34 (IT-34): `format::table` renders a `data_fmt` ASCII table with column headers.
///
/// # Root Cause
/// Task 131 adds `OutputFormat::Table` to `.accounts`. Before implementation this
/// exits 1 with `"unknown format 'table'"`.
///
/// # Why Not Caught
/// New feature; no prior test existed.
///
/// # Fix Applied
/// Added `OutputFormat::Table` variant to `output.rs`, `"table"` parse arm in
/// `from_cmd()`, and `render_accounts_table()` in `commands.rs`.
///
/// # Prevention
/// Covered by this test: two accounts saved; `format::table` asserted to exit 0
/// and contain `Account` header (column header from `data_fmt` table).
///
/// # Pitfall
/// `format::table` for non-`.accounts` commands must exit 1. Only `.accounts`
/// accepts this format; all other routines reject with `ArgumentTypeMismatch`.
#[ test ]
fn acc34_accounts_table_format()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "work@acme.com",  "max", "default", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".accounts", "format::table" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Account" ),
    "format::table must include 'Account' column header, got:\n{text}",
  );
  assert!(
    text.contains( "alice@acme.com" ),
    "format::table must include alice@acme.com in output, got:\n{text}",
  );
  assert!(
    text.contains( "work@acme.com" ),
    "format::table must include work@acme.com in output, got:\n{text}",
  );
}

// ── acc35–acc41: uuid:: and capabilities:: on .accounts (FR-21) ──────────────

/// acc35: `uuid::1` shows `ID:` line from saved snapshot.
#[ test ]
fn acc35_uuid_shows_id_from_snapshot()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "user_abc123", "some-uuid", &[ "claude_code" ] );

  let out  = run_cs_with_env( &[ ".accounts", "cols::+uuid" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "ID:" ),         "cols::+uuid must emit ID: line, got:\n{text}" );
  assert!( text.contains( "user_abc123" ), "ID: must show taggedId from snapshot, got:\n{text}" );
}

/// acc36: Default — `ID:` absent when `uuid::` not specified.
#[ test ]
fn acc36_uuid_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "user_abc123", "some-uuid", &[] );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "ID:" ), "ID: must be absent by default, got:\n{text}" );
}

/// acc37: `format::json` always includes `tagged_id` key regardless of `uuid::`.
#[ test ]
fn acc37_json_includes_tagged_id()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "user_abc123", "some-uuid", &[] );

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"tagged_id\"" ), "format::json must include tagged_id key, got:\n{text}" );
  assert!( text.contains( "user_abc123" ),   "tagged_id must contain the snapshot value, got:\n{text}" );
}

/// acc38: `capabilities::1` shows `Capabilities:` line from saved snapshot.
#[ test ]
fn acc38_capabilities_shows_list_from_snapshot()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "", "", &[ "claude_max", "chat" ] );

  let out  = run_cs_with_env( &[ ".accounts", "cols::+capabilities" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Capabilities:" ), "cols::+capabilities must emit Capabilities: line, got:\n{text}" );
  assert!( text.contains( "claude_max" ),    "Capabilities: must list claude_max, got:\n{text}" );
  assert!( text.contains( "chat" ),          "Capabilities: must list chat, got:\n{text}" );
}

/// acc39: Default — `Capabilities:` absent when `capabilities::` not specified.
#[ test ]
fn acc39_capabilities_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "", "", &[ "claude_max" ] );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Capabilities:" ), "Capabilities: must be absent by default, got:\n{text}" );
}

/// acc40: `format::json` always includes `capabilities` key.
#[ test ]
fn acc40_json_includes_capabilities()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json_extended( dir.path(), "alice@acme.com", "", "", &[ "claude_max" ] );

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"capabilities\"" ), "format::json must include capabilities key, got:\n{text}" );
  assert!( text.contains( "claude_max" ),       "capabilities must contain the snapshot value, got:\n{text}" );
}

/// acc41: No snapshot → `ID: N/A` and `Capabilities: N/A` when opted in.
#[ test ]
fn acc41_no_snapshot_uuid_capabilities_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  // No snapshot files written.

  let out  = run_cs_with_env( &[ ".accounts", "cols::+uuid,+capabilities" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "ID:" ),           "ID: line must appear with cols::+uuid, got:\n{text}" );
  assert!( text.contains( "Capabilities:" ), "Capabilities: line must appear, got:\n{text}" );
  assert!( text.contains( "N/A" ),           "absent snapshot must show N/A for new fields, got:\n{text}" );
}

// ── acc42 ─────────────────────────────────────────────────────────────────────

/// acc42 (EC-1): `org_uuid::1` shows `Org ID:` line with value from `{name}.json`.
#[ test ]
fn acc42_org_uuid_shows_from_roles_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "alice@acme.com", "org-xyz-789", "Acme Corp", "admin" );

  let out  = run_cs_with_env( &[ ".accounts", "cols::+org_uuid" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org ID:" ),    "cols::+org_uuid must emit Org ID: line, got:\n{text}" );
  assert!( text.contains( "org-xyz-789" ), "Org ID: must show organization_uuid from roles.json, got:\n{text}" );
}

// ── acc43 ─────────────────────────────────────────────────────────────────────

/// acc43 (EC-4): Default — `Org ID:` absent when `org_uuid::` not specified.
#[ test ]
fn acc43_org_uuid_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "alice@acme.com", "org-xyz-789", "Acme Corp", "admin" );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Org ID:" ), "Org ID: must be absent by default, got:\n{text}" );
}

// ── acc44 ─────────────────────────────────────────────────────────────────────

/// acc44 (EC-7): Missing roles.json → `Org ID:  N/A` when `org_uuid::1`.
#[ test ]
fn acc44_org_uuid_missing_roles_json_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  // No roles.json written.

  let out  = run_cs_with_env( &[ ".accounts", "cols::+org_uuid" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org ID:" ), "Org ID: line must appear with cols::+org_uuid, got:\n{text}" );
  assert!( text.contains( "N/A" ),     "absent roles.json must show N/A, got:\n{text}" );
}

// ── acc45 ─────────────────────────────────────────────────────────────────────

/// acc45 (EC-6): `format::json` always includes `organization_uuid` key.
#[ test ]
fn acc45_json_includes_org_uuid()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "alice@acme.com", "org-xyz-789", "Acme Corp", "admin" );

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"organization_uuid\"" ), "JSON must include organization_uuid key, got:\n{text}" );
  assert!( text.contains( "org-xyz-789" ),           "organization_uuid must contain the snapshot value, got:\n{text}" );
}

// ── acc46 ─────────────────────────────────────────────────────────────────────

/// acc46 (EC-1): `org_name::1` shows `Org:` line with value from `{name}.json`.
#[ test ]
fn acc46_org_name_shows_from_roles_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "alice@acme.com", "org-xyz-789", "Acme Corp", "admin" );

  let out  = run_cs_with_env( &[ ".accounts", "cols::+org_name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org:" ),      "cols::+org_name must emit Org: line, got:\n{text}" );
  assert!( text.contains( "Acme Corp" ), "Org: must show organization_name from roles.json, got:\n{text}" );
}

// ── acc47 ─────────────────────────────────────────────────────────────────────

/// acc47 (EC-4): Default — `Org:` absent when `org_name::` not specified.
#[ test ]
fn acc47_org_name_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_roles_json( dir.path(), "alice@acme.com", "org-xyz-789", "Acme Corp", "admin" );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Org:" ), "Org: must be absent by default, got:\n{text}" );
}

// ── acc48 ─────────────────────────────────────────────────────────────────────

/// acc48 (EC-7): Missing roles.json → `Org:     N/A` when `org_name::1`.
#[ test ]
fn acc48_org_name_missing_roles_json_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  // No roles.json written.

  let out  = run_cs_with_env( &[ ".accounts", "cols::+org_name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Org:" ), "Org: line must appear with cols::+org_name, got:\n{text}" );
  assert!( text.contains( "N/A" ),  "absent roles.json must show N/A, got:\n{text}" );
}

// ── it_trace_accounts_accepted ─────────────────────────────────────────────────

/// EC-9 (023): `trace::1` accepted by `.accounts` on empty store — no "Unknown parameter" error.
/// TSK-210 RED gate: fails before `trace::` is registered (exit 1 + Unknown parameter).
#[ test ]
fn it_trace_accounts_accepted()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".accounts", "trace::1" ], &[ ( "HOME", home ) ] );
  let err = stderr( &out );
  assert!(
    !err.contains( "Unknown parameter" ),
    "trace::1 must be accepted by .accounts, got stderr:\n{err}",
  );
  assert_exit( &out, 0 );
  assert!(
    err.contains( " · " ),
    "trace::1 must emit trace lines to stderr for .accounts, got:\n{err}",
  );
}

// ── acc49: host::1 role::1 shows profile metadata ────────────────────────────

/// acc49 — `.accounts host::1 role::1` shows Host and Role from profile.json.
///
/// Spec: [`tests/docs/feature/029_account_host_metadata.md` FT-08]
#[ test ]
fn acc49_accounts_host_role_shows_profile_metadata()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  write_account_profile_json( dir.path(), "test@example.com", Some( "mybox" ), Some( "work" ) );

  let out  = run_cs_with_env(
    &[ ".accounts", "cols::+host,+role" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Host:    mybox" ),
    "cols::+host must show Host: from profile.json, got:\n{text}",
  );
  assert!(
    text.contains( "Role:    work" ),
    "cols::+role must show Role: from profile.json, got:\n{text}",
  );
}

// ── acc50: absent profile.json — host::1 exits 0, shows N/A ──────────────────

/// acc50 — absent `profile.json` must not cause any command to exit non-zero.
///
/// When `host::1` is given but no `{name}.json` exists, the Host field
/// shows `N/A` (empty → fallback) and exit is 0. Resilience spec.
///
/// Spec: [`tests/docs/feature/029_account_host_metadata.md` FT-09]
#[ test ]
fn acc50_accounts_host_no_profile_json_exits_0()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "test@example.com", "max", "standard", FAR_FUTURE_MS, false );
  // No profile.json written — must be treated as optional metadata.

  let out  = run_cs_with_env(
    &[ ".accounts", "cols::+host" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Host:    N/A" ),
    "absent profile.json must show Host: N/A (not error), got:\n{text}",
  );
}

// ── acc51 ─────────────────────────────────────────────────────────────────────

#[ test ]
fn acc51_accounts_positional_after_key_value()
{
  // BUG-294: reversed arg order `clp .accounts format::json alice@acme.com` — key::value
  // before bare name — must rewrite positional arg regardless of argv position.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".accounts", "format::json", "alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@acme.com" ), "reversed-order positional must show alice@acme.com, got:\n{text}" );
  assert!( !text.contains( "work@acme.com" ), "must not show work@acme.com, got:\n{text}" );
}


//! Integration tests: AR (Account Relogin) + AW trace/feature027 tests.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | ar01 | `relogin_mre_no_name_uses_active` | no `name::` + active → uses active (dry-run) | P |
//! | ar02 | `relogin_mre_no_name_no_active_exits2` | no `name::` + no `_active` → exit 2 | N |
//! | ar03 | `ar03_relogin_empty_name_exits_1` | empty `name::` → exit 1 | N |
//! | ar04 | `ar04_relogin_not_found_exits_2` | unknown account → exit 2 | N |
//! | ar05 | `ar05_relogin_dry_explicit_name` | `dry::1` with explicit name prints message | P |
//! | ar07 | `ar07_relogin_positional_bare_arg` | positional `work@acme.com dry::1` → resolves | P |
//! | ar08 | `ar08_relogin_prefix_resolves` | prefix `work dry::1` → `work@acme.com` | P |
//! | ar09 | `ar09_relogin_invalid_chars_exits_1` | `name::bad/name` → exit 1 | N |
//! | aw22 | `aw22_touch_disabled_switch_succeeds` | `touch::0` → switch exits 0, no quota fetch | P |
//! | mre_bug586 | `mre_bug_586_untraced_switch_leaves_stderr_empty` | no `trace::` → stderr empty | P |
//! | — | `bug_586_traced_switch_prefixes_every_stderr_line` | `trace::1` → every stderr line timestamped | P |
//! | aw23 | `aw23_touch_skipped_no_access_token` | `touch::1` + no `accessToken` → exit 0 | P |
//! | aw24 | `aw24_imodel_bad_value_exits_1` | `imodel::bad` → exit 1 | N |
//! | aw25 | `aw25_effort_bad_value_exits_1` | `effort::bad` → exit 1 | N |
//! | aw26 | `aw26_help_shows_touch_imodel_effort` | `.account.use.help` lists params | P |
//! | aw27 | `aw27_lim_it_touch_with_live_token` | live token + `touch::1` → exit 0 | P |
//! | aw28 | `aw28_lim_it_trace_idle_account_all_lines` | `trace::1` + live idle → all 6 trace lines | P |
//! | aw29 | `aw29_lim_it_trace_active_account_subprocess_skipped` | `trace::1` + live active | P |
//! | aw30 | `aw30_trace_fetch_failure_skips_idle_model_lines` | `trace::1` + invalid token | N |
//! | aw31 | `aw31_trace_touch_disabled_no_trace_lines` | `touch::0 trace::1` → no trace lines | P |
//! | aw32 | `aw32_trace_bad_value_exits_1` | `trace::bad` → exit 1 | N |
//! | aw35 | `aw35_help_shows_positional_example` | `.account.use.help` → positional example | P |
//! | mre_bug209 | `mre_bug_209_account_save_uses_active_marker_not_stale_email` | save reads _active not stale email | P |
//! | — | `switch_restores_claude_json` | `~/.claude.json` restored (BUG-277) | P |
//! | — | `mre_bug_217_switch_account_enforces_emailaddress` | switch enforces emailAddress | P |

use crate::cli_runner::{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, write_account_roles_json,
  write_account_with_token, live_active_token,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── relogin: optional-name default-to-active tests ────────────────────────────

/// IT-1 / AC-02: `.account.relogin` with no `name::` uses the active account.
///
/// Verifies that when `name::` is omitted and the `_active` marker names
/// `work@acme.com`, the dry-run output names that account — confirming the
/// active-account fallback per `invariant/006_param_defaults.md`.
#[ test ]
fn relogin_mre_no_name_uses_active()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.relogin", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run] would re-authenticate 'work@acme.com' via browser login" ),
    "dry-run must print full re-auth message naming active account, got:\n{text}",
  );
}

/// IT-2 / AC-03: `.account.relogin` with no `name::` and no `_active` marker exits 2.
///
/// Verifies that omitting `name::` when no active account is set produces
/// exit 2 with an actionable message — not exit 1 (usage error).
#[ test ]
fn relogin_mre_no_name_no_active_exits2()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Account file exists but no _active marker written (make_active = false).
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.relogin" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    err.contains( "no active account" ) || err.contains( "name::" ),
    "error must mention missing active account, got:\n{err}",
  );
}

// ── IT-3 through IT-9 ─────────────────────────────────────────────────────────

#[ test ]
fn ar03_relogin_empty_name_exits_1()
{
  // IT-3: empty `name::` value → exit 1 (ArgumentMissing).
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.relogin", "name::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn ar04_relogin_not_found_exits_2()
{
  // IT-4: named account does not exist in the store → check_switch_preconditions fails → exit 2.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.relogin", "name::ghost@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
}

#[ test ]
fn ar05_relogin_dry_explicit_name()
{
  // IT-5: dry::1 with an existing name prints the re-auth message without spawning claude.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.relogin", "name::work@acme.com", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run] would re-authenticate 'work@acme.com' via browser login" ),
    "dry-run must print full re-auth message, got:\n{text}",
  );
}

#[ test ]
fn ar07_relogin_positional_bare_arg()
{
  // IT-7: positional form `clp .account.relogin work@acme.com dry::1` resolves the account.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.relogin", "work@acme.com", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "work@acme.com" ),
    "positional arg must resolve account name, got:\n{text}",
  );
}

#[ test ]
fn ar08_relogin_prefix_resolves()
{
  // IT-8: prefix `work` uniquely resolves to `work@acme.com` and uses it.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.relogin", "work", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "work@acme.com" ),
    "prefix 'work' must resolve to work@acme.com, got:\n{text}",
  );
}

#[ test ]
fn ar09_relogin_invalid_chars_exits_1()
{
  // IT-9: `name::bad/name` — no `@`, path-unsafe `/` → ArgumentTypeMismatch → exit 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.relogin", "name::bad/name" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── ar10 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn ar10_relogin_positional_after_key_value()
{
  // BUG-294: reversed arg order `clp .account.relogin dry::1 work@acme.com` — key::value
  // before bare name — must rewrite positional arg regardless of argv position.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "pro", "standard", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.relogin", "dry::1", "work@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "work@acme.com" ),
    "reversed-order positional must resolve account name for relogin, got:\n{text}",
  );
}

// ── ad15 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn ad15_delete_removes_roles_json()
{
  // AC-04: delete removes {name}.json alongside credentials.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",   "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "old@archive.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account_roles_json( dir.path(), "old@archive.com", "org-del-123", "Delete Corp", "admin" );

  let out = run_cs_with_env( &[ ".account.delete", "name::old@archive.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!( !store.join( "old@archive.com.credentials.json" ).exists(), "credentials must be removed" );
  assert!( !store.join( "old@archive.com.json" ).exists(),       "{{name}}.json snapshot must be removed after delete" );
}

// ── ad16 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn ad16_delete_positional_after_key_value()
{
  // BUG-294: reversed arg order `clp .account.delete dry::1 alice@home.com` — key::value
  // before bare name — must rewrite positional arg regardless of argv position.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",  "pro", "standard", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alice@home.com", "max", "tier4",    FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.delete", "dry::1", "alice@home.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "[dry-run] would delete account 'alice@home.com'" ),
    "reversed-order positional must resolve account for dry-run delete, got:\n{text}",
  );
}

// ── as19 ──────────────────────────────────────────────────────────────────────

#[ test ]
fn as19_save_best_effort_no_roles_json()
{
  // AC-02: save with no valid accessToken in credentials → exit 0; roles data absent from
  // {{name}}.json.  The unified file must not contain org identity fields.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // credentials JSON has no accessToken field, so fetch_claude_cli_roles is never called.

  let out = run_cs_with_env( &[ ".account.save", "name::user@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let meta = std::fs::read_to_string( store.join( "user@example.com.json" ) )
    .unwrap_or_default();
  assert!(
    !meta.contains( "organization_uuid" ),
    "{{name}}.json must not contain org identity when no accessToken, got: {meta}",
  );
}

// ── as20 (lim_it) ─────────────────────────────────────────────────────────────

#[ test ]
fn as20_lim_it_save_writes_roles_json()
{
  // AC-01 (FT-01): .account.save with a valid accessToken calls fetch_claude_cli_roles and
  // writes {name}.json to the credential store. Requires live Anthropic credentials.
  let token = live_active_token().expect( "as20: live API token required — no ~/.claude/.credentials.json" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "user@example.com", &token, false );
  // Copy credentials.json into ~/.claude/.credentials.json so the binary can read it.
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let cred_src = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "user@example.com.credentials.json" );
  std::fs::copy( &cred_src, claude_dir.join( ".credentials.json" ) ).unwrap();

  let out = run_cs_with_env( &[ ".account.save", "name::user@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let roles_path = store.join( "user@example.com.json" );
  assert!( roles_path.exists(), "{{name}}.json must be created after save with valid token" );
  let content = std::fs::read_to_string( &roles_path ).unwrap();
  assert!( content.contains( "\"organization_uuid\"" ), "{{name}}.json must contain organization_uuid, got:\n{content}" );
  assert!( content.contains( "\"organization_name\"" ), "{{name}}.json must contain organization_name, got:\n{content}" );
}

// ── as21 (lim_it) ─────────────────────────────────────────────────────────────

#[ test ]
fn as21_lim_it_resave_overwrites_roles_json()
{
  // AC-03 (FT-03): Second .account.save overwrites existing {name}.json with fresh data.
  // Idempotency: stale snapshot is replaced by new API response. Requires live credentials.
  let token = live_active_token().expect( "as21: live API token required — no ~/.claude/.credentials.json" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "user@example.com", &token, false );
  let claude_dir = dir.path().join( ".claude" );
  std::fs::create_dir_all( &claude_dir ).unwrap();
  let cred_src = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "user@example.com.credentials.json" );
  std::fs::copy( &cred_src, claude_dir.join( ".credentials.json" ) ).unwrap();
  // Pre-seed stale {name}.json with a sentinel value.
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::write(
    store.join( "user@example.com.json" ),
    r#"{"organization_uuid":"stale-sentinel","organization_name":"Stale","organization_role":"none","workspace_uuid":null,"workspace_name":null}"#,
  ).unwrap();

  // Second save must overwrite.
  let out = run_cs_with_env( &[ ".account.save", "name::user@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let roles_path = store.join( "user@example.com.json" );
  assert!( roles_path.exists(), "{{name}}.json must still exist after re-save" );
  let content = std::fs::read_to_string( &roles_path ).unwrap();
  assert!(
    !content.contains( "stale-sentinel" ),
    "re-save must overwrite stale {{name}}.json; sentinel must be gone, got:\n{content}",
  );
}

// ── AW: Feature 027 — post-switch touch control ────────────────────────────────

// BUG-586 — aw22 never reads stderr; the two BUG-586 tests below check it at trace::0 and trace::1
/// aw22: `touch::0` disables post-switch subprocess; switch still succeeds (IT-18).
///
/// Verifies that explicitly disabling touch does not interfere with the switch itself.
/// No accessToken is present — if touch were attempted, the quota fetch would fail;
/// exit 0 with "switched" proves touch was skipped before any quota API call.
#[ test ]
fn aw22_touch_disabled_switch_succeeds()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "target@example.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "touch::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "touch::0 must not block switch, got:\n{}", stdout( &out ),
  );
}

/// bug_reproducer(BUG-586): a switch onto an Anthropic account with no `trace::` writes nothing
/// to stderr.
///
/// # Root Cause
/// `claude_core::settings_io`'s four mutators each opened with an unconditional `eprintln!` of
/// their parameters, and an Anthropic switch clears its env vars through `remove_env_var`, so
/// a `trace::0` switch printed one line per key.
///
/// # Why Not Caught
/// No switch test asserted an empty stderr (aw22 never reads it), the `trace::0` tests it049
/// and it052 run a `.usage` that never reaches `settings_io`, and `claude_core`'s own tests
/// required the print.
///
/// # Fix Applied
/// The mutators take `trace : bool` and print only when it's set; `switch.rs` passes `false`.
///
/// # Prevention
/// The whole stderr is asserted empty, with `CLR_PROC_DIR` on an empty dir so the
/// running-session advisory can't add a line of its own.
///
/// # Pitfall
/// A check for "no ` · `" passes an unprefixed library line; assert the exact stderr.
#[ doc = "bug_reproducer(BUG-586)" ]
#[ test ]
fn mre_bug_586_untraced_switch_leaves_stderr_empty()
{
  let dir      = TempDir::new().unwrap();
  let home     = dir.path().to_str().unwrap();
  let proc_dir = TempDir::new().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "target@example.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "touch::0" ],
    &[ ( "HOME", home ), ( "CLR_PROC_DIR", proc_dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "the switch must succeed, got:\n{}", stdout( &out ),
  );
  assert_eq!(
    stderr( &out ), "",
    "BUG-586: a trace::0 switch must write nothing to stderr",
  );
}

/// `YYYY-MM-DD · HH:MM:SS UTC · `, the prefix every clp trace line starts with.
fn has_trace_prefix( line : &str ) -> bool
{
  let mut parts = line.splitn( 3, " · " );
  let ( Some( date ), Some( time ), Some( _ ) ) = ( parts.next(), parts.next(), parts.next() )
  else { return false };
  date.len() == 10
    && date.chars().all( | c | c.is_ascii_digit() || c == '-' )
    && time.len() == 12
    && time.ends_with( " UTC" )
    && time.get( ..8 ).is_some_and( | hms | hms.chars().all( | c | c.is_ascii_digit() || c == ':' ) )
}

/// bug_reproducer(BUG-586): under `trace::1`, every stderr line of a switch carries the
/// `YYYY-MM-DD · HH:MM:SS UTC · ` prefix.
///
/// # Root Cause
/// See `mre_bug_586_untraced_switch_leaves_stderr_empty`. The mutators' lines carry no
/// timestamp, so they also broke the prefix contract when clp's own trace was on.
///
/// # Why Not Caught
/// The traced switch tests look for clp's own lines by substring and never check that
/// every line has the prefix.
///
/// # Fix Applied
/// `switch.rs` passes `false`, so a switch's `settings_io` calls print nothing at either level.
///
/// # Prevention
/// `touch::1` with no `accessToken` makes clp trace its expiry check, so the prefix check
/// runs over at least one line.
///
/// # Pitfall
/// "Every line has the prefix" holds for an empty stderr too; require a traced line first.
#[ doc = "bug_reproducer(BUG-586)" ]
#[ test ]
fn bug_586_traced_switch_prefixes_every_stderr_line()
{
  let dir      = TempDir::new().unwrap();
  let home     = dir.path().to_str().unwrap();
  let proc_dir = TempDir::new().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "target@example.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com", "trace::1" ],
    &[ ( "HOME", home ), ( "CLR_PROC_DIR", proc_dir.path().to_str().unwrap() ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "expiry check: valid" ),
    "trace::1 must trace the expiry check, got:\n{err}",
  );
  let unprefixed : Vec< &str > = err.lines().filter( | line | !has_trace_prefix( line ) ).collect();
  assert!(
    unprefixed.is_empty(),
    "BUG-586: every trace::1 stderr line must carry the timestamp prefix, these don't: {unprefixed:?}",
  );
}

/// aw23: `touch::1` (default) with no `accessToken` → exit 0, touch silently skipped (IT-20).
///
/// `write_account` produces credentials without `accessToken`; `pre_switch_touch_ctx`
/// returns `None` (token read fails) so no subprocess is spawned. The switch still succeeds.
#[ test ]
fn aw23_touch_skipped_no_access_token()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  // write_account produces credentials without accessToken — quota fetch path returns None.
  write_account( dir.path(), "target@example.com", "max", "tier4", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".account.use", "name::target@example.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!(
    stdout( &out ).contains( "switched" ),
    "touch skipped (no token) must not block switch, got:\n{}", stdout( &out ),
  );
}

/// aw24: `imodel::bad` → exit 1; stderr names all valid values (IT-21).
///
/// Validation fires before any filesystem I/O — no accounts needed in the temp dir.
#[ test ]
fn aw24_imodel_bad_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::any@example.com", "imodel::bad" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "auto" ) && err.contains( "sonnet" ) && err.contains( "opus" ) && err.contains( "keep" ),
    "stderr must name all valid imodel:: values; got:\n{err}",
  );
}

/// aw25: `effort::bad` → exit 1; stderr names all valid values (IT-22).
///
/// Validation fires before any filesystem I/O — no accounts needed in the temp dir.
#[ test ]
fn aw25_effort_bad_value_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env(
    &[ ".account.use", "name::any@example.com", "effort::bad" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "auto" ) && err.contains( "high" ) && err.contains( "max" ),
    "stderr must name all valid effort:: values; got:\n{err}",
  );
}


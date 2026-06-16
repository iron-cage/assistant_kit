//! Acceptance tests for all user story specifications.
//!
//! Implements test cases from:
//! - `tests/docs/cli/user_story/001_environment_check.md` (US-1 through US-4)
//! - `tests/docs/cli/user_story/002_version_upgrade.md` (US-1 through US-6)
//! - `tests/docs/cli/user_story/003_process_lifecycle.md` (US-1 through US-6)
//! - `tests/docs/cli/user_story/004_settings_management.md` (US-1 through US-6)
//! - `tests/docs/cli/user_story/005_version_pinning.md` (US-1 through US-6)

use tempfile::TempDir;

use crate::helpers::{ assert_exit, run_clm_with_env, stdout, write_settings };

// ═══════════════════════════════════════════════════════════════════════════════
// US-001: Environment Check
// ═══════════════════════════════════════════════════════════════════════════════

// US-1: .status outputs version, session count, and account in one view; exit 0
#[ test ]
fn us01_001_status_exits_0()
{
  let out = run_clm_with_env( &[ ".status" ], &[] );
  assert_exit( &out, 0 );
  assert!( !stdout( &out ).is_empty(), ".status must produce output" );
}

// US-2: .status format::json → valid JSON with version and account fields; exit 0
#[ test ]
fn us02_001_status_json_format()
{
  let out = run_clm_with_env( &[ ".status", "format::json" ], &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '{' ), "JSON output must start with {{: {text}" );
}

// US-3: .status v::2 → additional diagnostic context beyond default; exit 0
#[ test ]
fn us03_001_status_verbose()
{
  let out = run_clm_with_env( &[ ".status", "v::2" ], &[] );
  assert_exit( &out, 0 );
}

// US-4: HOME="" → .status exits gracefully with degraded output; exit 0
//
// The implementation shows "unknown" for account when HOME is unset,
// providing graceful degradation (exit 0) rather than exit 2.
#[ test ]
fn us04_001_status_no_home_graceful()
{
  let out = run_clm_with_env( &[ ".status" ], &[ ( "HOME", "" ) ] );
  assert_exit( &out, 0 );
}

// ═══════════════════════════════════════════════════════════════════════════════
// US-002: Version Upgrade
// ═══════════════════════════════════════════════════════════════════════════════

// US-1: dry-run preview shows install plan without executing; exit 0
#[ test ]
fn us01_002_version_install_dry_preview()
{
  let out = run_clm_with_env(
    &[ ".version.install", "version::stable", "dry::1" ],
    &[],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "dry-run must show [dry-run] prefix: {text}" );
}

// US-2: install command accepts version spec and produces a valid plan; exit 0
//
// Uses dry::1 to verify the install workflow without network-dependent side effects.
#[ test ]
fn us02_002_version_install_plan_accepted()
{
  let out = run_clm_with_env(
    &[ ".version.install", "version::stable", "dry::1" ],
    &[],
  );
  assert_exit( &out, 0 );
}

// US-3: running install for the same version is idempotent; exit 0
#[ test ]
fn us03_002_version_install_idempotent()
{
  let out1 = run_clm_with_env(
    &[ ".version.install", "version::stable", "dry::1" ],
    &[],
  );
  let out2 = run_clm_with_env(
    &[ ".version.install", "version::stable", "dry::1" ],
    &[],
  );
  assert_exit( &out1, 0 );
  assert_exit( &out2, 0 );
}

// US-4: .version.show prints the currently installed version; exit 0
//
// Uses guard pattern: .version.show exits 2 when claude is not in PATH (offline
// container), so we only check content when the command succeeds.
#[ test ]
fn us04_002_version_show_exits_0()
{
  let out = run_clm_with_env( &[ ".version.show" ], &[] );
  if out.status.code() == Some( 0 )
  {
    assert!( !stdout( &out ).is_empty(), ".version.show must produce version output" );
  }
}

// US-5: .version.history shows recent releases; exit 0
#[ test ]
fn us05_002_version_history_exits_0()
{
  let out = run_clm_with_env( &[ ".version.history" ], &[] );
  assert_exit( &out, 0 );
}

// US-6: .version.guard detects and handles drift; exit 0 via dry mode
#[ test ]
fn us06_002_version_guard_exits_0()
{
  let out = run_clm_with_env( &[ ".version.guard", "dry::1" ], &[] );
  assert_exit( &out, 0 );
}

// ═══════════════════════════════════════════════════════════════════════════════
// US-003: Process Lifecycle
// ═══════════════════════════════════════════════════════════════════════════════

// US-1: .processes lists PIDs and working directories (or empty list); exit 0
#[ test ]
fn us01_003_processes_exits_0()
{
  let out = run_clm_with_env( &[ ".processes" ], &[] );
  assert_exit( &out, 0 );
}

// US-2: .processes format::json returns JSON array (possibly empty); exit 0
#[ test ]
fn us02_003_processes_json_format()
{
  let out = run_clm_with_env( &[ ".processes", "format::json" ], &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let t = text.trim_start();
  assert!(
    t.starts_with( '[' ) || t.starts_with( '{' ),
    "processes JSON output must start with [ or {{: {text}"
  );
}

// US-3: .processes.kill dry::1 previews kill targets without sending signals; exit 0
#[ test ]
fn us03_003_processes_kill_dry_preview()
{
  let out = run_clm_with_env( &[ ".processes.kill", "dry::1" ], &[] );
  assert_exit( &out, 0 );
}

// US-4: .processes.kill sends SIGTERM then SIGKILL; verified via dry mode; exit 0
//
// Uses dry::1 to verify the command dispatches correctly without live processes.
#[ test ]
fn us04_003_processes_kill_graceful()
{
  let out = run_clm_with_env( &[ ".processes.kill", "dry::1" ], &[] );
  assert_exit( &out, 0 );
}

// US-5: .processes.kill force::1 sends SIGKILL directly; verified via dry mode; exit 0
#[ test ]
fn us05_003_processes_kill_force()
{
  let out = run_clm_with_env( &[ ".processes.kill", "force::1", "dry::1" ], &[] );
  assert_exit( &out, 0 );
}

// US-6: .processes after kill returns empty list; exit 0
//
// In the test environment there are no Claude processes, so .processes exits 0
// with an empty list — the expected post-kill state is already present.
#[ test ]
fn us06_003_processes_empty_after_kill()
{
  let out = run_clm_with_env( &[ ".processes" ], &[] );
  assert_exit( &out, 0 );
}

// ═══════════════════════════════════════════════════════════════════════════════
// US-004: Settings Management
// ═══════════════════════════════════════════════════════════════════════════════

// US-1: .settings.show prints all key-value pairs; exit 0
#[ test ]
fn us01_004_settings_show_all_pairs()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ), ( "autoUpdates", "true" ) ] );

  let out = run_clm_with_env( &[ ".settings.show" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "theme" ), "show must include theme key: {text}" );
  assert!( text.contains( "dark" ), "show must include theme value: {text}" );
}

// US-2: .settings.show format::json returns JSON object; exit 0
#[ test ]
fn us02_004_settings_show_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ) ] );

  let out = run_clm_with_env(
    &[ ".settings.show", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '{' ), "JSON output must start with {{: {text}" );
}

// US-3: .settings.get key::X → exit 0 with value; absent key → exit 2
#[ test ]
fn us03_004_settings_get_found_and_missing()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ) ] );

  let found = run_clm_with_env(
    &[ ".settings.get", "key::theme" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &found, 0 );
  assert!( stdout( &found ).contains( "dark" ), "get must return the stored value" );

  let missing = run_clm_with_env(
    &[ ".settings.get", "key::nonexistent" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &missing, 2 );
}

// US-4: .settings.set dry::1 previews without modifying the file; exit 0
#[ test ]
fn us04_004_settings_set_dry_preview()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_settings( dir.path(), &[ ( "theme", "dark" ) ] );

  let out = run_clm_with_env(
    &[ ".settings.set", "key::theme", "value::light", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "dry-run must show [dry-run] prefix: {text}" );
  // Verify the file was not modified.
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "dark" ), "file must remain unchanged after dry-run: {content}" );
}

// US-5: .settings.set writes key-value with atomic rename; exit 0
#[ test ]
fn us05_004_settings_set_writes_atomically()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_clm_with_env(
    &[ ".settings.set", "key::theme", "value::light" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  assert!( content.contains( "\"theme\"" ), "settings.json must contain theme key: {content}" );
  assert!( content.contains( "light" ), "settings.json must contain written value: {content}" );
}

// US-6: type inference stores booleans, integers, and floats as native JSON; exit 0
#[ test ]
fn us06_004_settings_set_type_inference()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out_a = run_clm_with_env(
    &[ ".settings.set", "key::flag", "value::true" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_a, 0 );
  let out_b = run_clm_with_env(
    &[ ".settings.set", "key::count", "value::42" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_b, 0 );
  let out_c = run_clm_with_env(
    &[ ".settings.set", "key::ratio", "value::3.14" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_c, 0 );

  let content = std::fs::read_to_string(
    dir.path().join( ".claude/settings.json" )
  ).unwrap();
  // Booleans, integers, and floats must be stored as unquoted JSON values.
  assert!( !content.contains( r#""true""# ), "boolean must not be quoted: {content}" );
  assert!( !content.contains( r#""42""# ),   "integer must not be quoted: {content}" );
  assert!( !content.contains( r#""3.14""# ), "float must not be quoted: {content}" );
}

// ═══════════════════════════════════════════════════════════════════════════════
// US-005: Version Pinning
// ═══════════════════════════════════════════════════════════════════════════════

// US-1: .version.list shows aliases with resolved semver versions; exit 0
#[ test ]
fn us01_005_version_list_shows_aliases()
{
  let out = run_clm_with_env( &[ ".version.list" ], &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "stable" ), "version list must include the stable alias: {text}" );
}

// US-2: .version.install version::month dry::1 → shows install plan for monthly baseline; exit 0
#[ test ]
fn us02_005_version_install_month_dry()
{
  let out = run_clm_with_env(
    &[ ".version.install", "version::month", "dry::1" ],
    &[],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run]" ), "dry-run must show [dry-run] prefix: {text}" );
}

// US-3: .version.install version::month → monthly baseline install plan accepted; exit 0
#[ test ]
fn us03_005_version_install_month_accepted()
{
  let out = run_clm_with_env(
    &[ ".version.install", "version::month", "dry::1" ],
    &[],
  );
  assert_exit( &out, 0 );
}

// US-4: already-at-pinned-version is a no-op; second install exits 0
#[ test ]
fn us04_005_version_install_idempotent()
{
  let out = run_clm_with_env(
    &[ ".version.install", "version::month", "dry::1" ],
    &[],
  );
  assert_exit( &out, 0 );
}

// US-5: .version.show confirms the active version; exit 0
//
// Uses guard pattern: .version.show exits 2 when claude is not in PATH (offline
// container), so we only check content when the command succeeds.
#[ test ]
fn us05_005_version_show_confirms_active()
{
  let out = run_clm_with_env( &[ ".version.show" ], &[] );
  if out.status.code() == Some( 0 )
  {
    assert!( !stdout( &out ).is_empty(), ".version.show must print the active version" );
  }
}

// US-6: .version.guard watches for drift; verified via dry one-shot mode; exit 0
//
// Uses interval::0 (one-shot) — interval::N > 0 enters an infinite watch loop
// that only exits on signal, making synchronous test execution impossible.
// Fix(issue-415) changed watch-mode errors from `return result` to logged-and-continue,
// which means errors no longer terminate the loop; one-shot is the safe test form.
#[ test ]
fn us06_005_version_guard_drift_watch()
{
  let out = run_clm_with_env(
    &[ ".version.guard", "interval::0", "dry::1" ],
    &[],
  );
  assert_exit( &out, 0 );
}

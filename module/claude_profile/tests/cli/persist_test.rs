//! Integration tests: P (`PersistPaths` — FR-15).
//!
//! ## Root Cause of Env-Var Isolation
//!
//! `PersistPaths::new()` reads `$PRO` and `$HOME` from the live process environment. Tests that
//! set or remove these vars must run serially to avoid cross-test contamination.
//!
//! ## Why Not Mock
//!
//! Per project rules: no mocking. Tests use real tmpdirs via `tempfile::TempDir`.
//! The static `ENV_MUTEX` serializes env-var access instead.
//!
//! ## Test Matrix
//!
//! ### Category A — Environment Variable Resolution
//!
//! | id  | condition                                     | expected                                        |
//! |-----|-----------------------------------------------|-------------------------------------------------|
//! | p01 | `$PRO` set (real dir), `$HOME` also set       | `base()` under `$PRO`                           |
//! | p02 | `$PRO` unset, `$HOME` set                     | `base()` under `$HOME`                          |
//! | p03 | `$PRO`, `$HOME`, `$USERPROFILE` all unset     | `Err` (`NotFound` kind)                         |
//! | p04 | `$PRO` set                                    | path ends with `.persistent/claude_profile`     |
//! | p06 | `$PRO` set to non-existent path               | falls back to `$HOME`                           |
//! | p07 | `$USERPROFILE` set, `$PRO`+`$HOME` unset      | `base()` under `$USERPROFILE`                   |
//! | p08 | `$HOME` and `$USERPROFILE` both set, no `$PRO`| `$HOME` takes priority over `$USERPROFILE`      |
//! | p09 | `$PRO` unset, `$HOME` set                     | path ends with `.persistent/claude_profile`     |
//! | p13 | `$PRO` set to `""` (empty string)             | falls back to `$HOME` (empty path is not a dir) |
//! | p14 | `$PRO` set to existing FILE (not directory)   | falls back to `$HOME` (file is not a directory) |
//!
//! ### Category B — `ensure_exists()` Robustness
//!
//! | id  | condition                                     | expected                                        |
//! |-----|-----------------------------------------------|-------------------------------------------------|
//! | p05 | `ensure_exists()` on fresh path               | directory created on disk                       |
//! | p10 | `ensure_exists()` called twice (idempotency)  | both calls succeed — no error                   |
//! | p11 | `ensure_exists()` when directory already exists | `Ok(())`, no error                            |
//!
//! ### Category C — API and Error Quality
//!
//! | id  | condition                                     | expected                                        |
//! |-----|-----------------------------------------------|-------------------------------------------------|
//! | p12 | both `$PRO` and `$HOME` unset → error message | message text is actionable (mentions `$HOME`)   |
//! | p15 | `Debug` formatting                            | `{:?}` produces non-empty output                |
//!
//! ### Category D — `credential_store()` path
//!
//! | id  | condition                                     | expected                                        |
//! |-----|-----------------------------------------------|-------------------------------------------------|
//! | p16 | `$PRO` set → `credential_store()` under `$PRO` | path starts with `$PRO`                        |
//! | p17 | `$PRO` set → path shape                       | ends with `.persistent/claude/credential`       |
//! | p18 | `$PRO` unset, `$HOME` set → path shape        | ends with `.persistent/claude/credential`       |

use claude_profile::PersistPaths;
use std::env;
use tempfile::TempDir;

/// Serialize all env-mutating tests: `$PRO` / `$HOME` are process-global state.
static ENV_MUTEX : std::sync::Mutex< () > = std::sync::Mutex::new( () );

fn lock() -> std::sync::MutexGuard< 'static, () >
{
  ENV_MUTEX.lock().unwrap_or_else( std::sync::PoisonError::into_inner )
}

fn restore( key : &str, original : Option< String > )
{
  match original
  {
  Some( val ) => env::set_var( key, val ),
  None        => env::remove_var( key ),
  }
}

/// p01 — `$PRO` set → `base()` resolves under `$PRO`
#[ test ]
fn p01_pro_set_base_under_pro()
{
  let _lock   = lock();
  let pro_dir = TempDir::new().unwrap();
  let home_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO" ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::set_var( "PRO",  pro_dir.path() );
  env::set_var( "HOME", home_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p01: PersistPaths::new() must succeed when $PRO is set" );
  assert!(
  paths.base().starts_with( pro_dir.path() ),
  "p01: base() must start with $PRO; got: {}",
  paths.base().display()
  );
}

/// p02 — `$PRO` unset, `$HOME` set → `base()` resolves under `$HOME`
#[ test ]
fn p02_pro_unset_home_set_base_under_home()
{
  let _lock    = lock();
  let home_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO" ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::remove_var( "PRO" );
  env::set_var( "HOME", home_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p02: PersistPaths::new() must succeed when $HOME is set" );
  assert!(
  paths.base().starts_with( home_dir.path() ),
  "p02: base() must start with $HOME; got: {}",
  paths.base().display()
  );
}

/// p03 — both `$PRO` and `$HOME` unset → `Err`
#[ test ]
fn p03_both_unset_returns_err()
{
  let _lock = lock();
  let orig_pro         = env::var( "PRO" ).ok();
  let orig_home        = env::var( "HOME" ).ok();
  let orig_userprofile = env::var( "USERPROFILE" ).ok();

  env::remove_var( "PRO" );
  env::remove_var( "HOME" );
  env::remove_var( "USERPROFILE" );

  let result = PersistPaths::new();

  restore( "PRO",         orig_pro         );
  restore( "HOME",        orig_home        );
  restore( "USERPROFILE", orig_userprofile );

  assert!(
  result.is_err(),
  "p03: PersistPaths::new() must return Err when both $PRO and $HOME are unset"
  );
  assert_eq!(
  result.unwrap_err().kind(),
  std::io::ErrorKind::NotFound,
  "p03: error kind must be NotFound"
  );
}

/// p04 — path shape: ends with `persistent/claude_profile`
#[ test ]
fn p04_base_path_shape_ends_with_persistent_claude_profile()
{
  let _lock   = lock();
  let pro_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO" ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::set_var( "PRO", pro_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p04: PersistPaths::new() must succeed" );
  assert!(
  paths.base().ends_with( ".persistent/claude_profile" ),
  "p04: base() must end with .persistent/claude_profile; got: {}",
  paths.base().display()
  );
}

/// p06 — `$PRO` set to non-existent path → falls back to `$HOME`
///
/// FR-15 step 1: "$PRO (if set **and path exists on disk**)". A set-but-missing
/// path must be transparent — callers must still get a valid base under `$HOME`.
#[ test ]
fn p06_pro_set_to_nonexistent_falls_back_to_home()
{
  let _lock    = lock();
  let home_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO" ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::set_var( "PRO",  "/this/path/does/not/exist/at/all" );
  env::set_var( "HOME", home_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p06: PersistPaths::new() must succeed when $HOME is set" );
  assert!(
  paths.base().starts_with( home_dir.path() ),
  "p06: base() must start with $HOME when $PRO points to non-existent path; got: {}",
  paths.base().display()
  );
}

/// p05 — `ensure_exists()` creates the directory on disk
#[ test ]
fn p05_ensure_exists_creates_directory()
{
  let _lock   = lock();
  let pro_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO" ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::set_var( "PRO", pro_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p05: PersistPaths::new() must succeed" );

  assert!( !paths.base().exists(), "p05: directory must not exist before ensure_exists()" );
  paths.ensure_exists().expect( "p05: ensure_exists() must succeed" );
  assert!( paths.base().exists(), "p05: directory must exist after ensure_exists()" );
}

/// p07 — `$USERPROFILE` set, `$PRO` and `$HOME` both unset → `base()` under `$USERPROFILE`
///
/// FR-15 fallback chain: `$PRO` → `$HOME` → `$USERPROFILE`. On systems where `$HOME`
/// is absent (Windows-style environments), `$USERPROFILE` must be the last resort.
#[ test ]
fn p07_userprofile_fallback_when_pro_and_home_unset()
{
  let _lock        = lock();
  let profile_dir  = TempDir::new().unwrap();
  let orig_pro     = env::var( "PRO"         ).ok();
  let orig_home    = env::var( "HOME"        ).ok();
  let orig_profile = env::var( "USERPROFILE" ).ok();

  env::remove_var( "PRO"  );
  env::remove_var( "HOME" );
  env::set_var( "USERPROFILE", profile_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",         orig_pro     );
  restore( "HOME",        orig_home    );
  restore( "USERPROFILE", orig_profile );

  let paths = result.expect( "p07: PersistPaths::new() must succeed when $USERPROFILE is set" );
  assert!(
  paths.base().starts_with( profile_dir.path() ),
  "p07: base() must start with $USERPROFILE when both $PRO and $HOME are unset; got: {}",
  paths.base().display()
  );
}

/// p08 — `$HOME` takes priority over `$USERPROFILE` when both are set
///
/// The fallback order is `$HOME` first, then `$USERPROFILE`. When both are present,
/// `$HOME` must win regardless of `$USERPROFILE` value.
#[ test ]
fn p08_home_priority_over_userprofile_when_both_set()
{
  let _lock        = lock();
  let home_dir     = TempDir::new().unwrap();
  let profile_dir  = TempDir::new().unwrap();
  let orig_pro     = env::var( "PRO"         ).ok();
  let orig_home    = env::var( "HOME"        ).ok();
  let orig_profile = env::var( "USERPROFILE" ).ok();

  env::remove_var( "PRO" );
  env::set_var( "HOME",        home_dir.path()    );
  env::set_var( "USERPROFILE", profile_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",         orig_pro     );
  restore( "HOME",        orig_home    );
  restore( "USERPROFILE", orig_profile );

  let paths = result.expect( "p08: PersistPaths::new() must succeed" );
  assert!(
  paths.base().starts_with( home_dir.path() ),
  "p08: $HOME must take priority over $USERPROFILE; got: {}",
  paths.base().display()
  );
  assert!(
  !paths.base().starts_with( profile_dir.path() ),
  "p08: base() must NOT start with $USERPROFILE when $HOME is also set; got: {}",
  paths.base().display()
  );
}

/// p09 — `$PRO` unset, `$HOME` set → path ends with `persistent/claude_profile`
///
/// Path shape validation for the `$HOME` code path. p04 verifies path shape only when
/// `$PRO` is the root; this test verifies the identical shape when `$HOME` is the root.
#[ test ]
fn p09_path_shape_ends_with_persistent_claude_profile_under_home()
{
  let _lock    = lock();
  let home_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO"  ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::remove_var( "PRO" );
  env::set_var( "HOME", home_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p09: PersistPaths::new() must succeed when $HOME is set" );
  assert!(
  paths.base().ends_with( ".persistent/claude_profile" ),
  "p09: base() must end with .persistent/claude_profile when $HOME is root; got: {}",
  paths.base().display()
  );
}

/// p10 — `ensure_exists()` is idempotent: calling twice must not return an error
///
/// `create_dir_all` guarantees idempotency. Two successive calls on the same `PersistPaths`
/// must both return `Ok(())` — the second call must not fail with "already exists".
#[ test ]
fn p10_ensure_exists_is_idempotent()
{
  let _lock   = lock();
  let pro_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO"  ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::set_var( "PRO", pro_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p10: PersistPaths::new() must succeed" );
  paths.ensure_exists().expect( "p10: first ensure_exists() must succeed" );
  paths.ensure_exists().expect( "p10: second ensure_exists() must also succeed (idempotent)" );
  assert!( paths.base().exists(), "p10: directory must exist after two ensure_exists() calls" );
}

/// p11 — `ensure_exists()` succeeds when the directory was already created externally
///
/// If the base directory exists before `ensure_exists()` is called (e.g., created by another
/// process or a prior `PersistPaths` instance), the call must still return `Ok(())`.
#[ test ]
fn p11_ensure_exists_succeeds_when_dir_already_exists()
{
  let _lock   = lock();
  let pro_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO"  ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::set_var( "PRO", pro_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p11: PersistPaths::new() must succeed" );

  // pre-create the directory externally, simulating another process
  std::fs::create_dir_all( paths.base() )
  .expect( "p11: pre-creating directory must succeed" );

  assert!( paths.base().exists(), "p11: directory must exist before ensure_exists() call" );
  paths.ensure_exists()
  .expect( "p11: ensure_exists() must succeed even when directory already exists" );
}

/// p12 — error message is actionable when both `$PRO` and `$HOME` are unset
///
/// NFR-4 requires "actionable messages". The error returned when no root can be
/// resolved must mention `$HOME` so the user knows what to set.
#[ test ]
fn p12_error_message_is_actionable_when_both_unset()
{
  let _lock            = lock();
  let orig_pro         = env::var( "PRO"         ).ok();
  let orig_home        = env::var( "HOME"        ).ok();
  let orig_userprofile = env::var( "USERPROFILE" ).ok();

  env::remove_var( "PRO"         );
  env::remove_var( "HOME"        );
  env::remove_var( "USERPROFILE" );

  let result = PersistPaths::new();

  restore( "PRO",         orig_pro         );
  restore( "HOME",        orig_home        );
  restore( "USERPROFILE", orig_userprofile );

  let err = result.expect_err( "p12: must return Err when no root is resolvable" );
  let msg = err.to_string();
  assert!(
  msg.contains( "HOME" ),
  "p12: error message must mention $HOME to be actionable; got: {msg:?}"
  );
}

/// p13 — `$PRO` set to empty string falls back to `$HOME`
///
/// An empty-string `$PRO` produces `PathBuf::from("")`. On all platforms, this path
/// does not exist on disk, so the code must fall through to `$HOME`.
#[ test ]
fn p13_pro_empty_string_falls_back_to_home()
{
  let _lock    = lock();
  let home_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO"  ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::set_var( "PRO",  "" );
  env::set_var( "HOME", home_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p13: PersistPaths::new() must succeed when $HOME is set" );
  assert!(
  paths.base().starts_with( home_dir.path() ),
  "p13: base() must start with $HOME when $PRO is empty; got: {}",
  paths.base().display()
  );
}

/// p14 — `$PRO` set to existing FILE (not a directory) falls back to `$HOME`
///
/// `$PRO` is a storage root — it must be a directory. If `$PRO` resolves to an
/// existing file, it is unusable as a root and must be treated as unset (fall
/// through to `$HOME`).
// test_kind: bug_reproducer(issue-001)
//
// Root Cause: The original guard used `path.exists()` which returns `true` for
//   both files and directories. A file path passed the guard and was used as root,
//   producing `<file>/persistent/claude_profile` — a path `ensure_exists()` would
//   fail with `ENOTDIR` at call time, not at validation time.
// Why Not Caught: p06 tests a non-existent `$PRO` path (falls back correctly), but
//   no test covered `$PRO` pointing to an existing file — a distinct case because
//   `exists()` returns true for files while `is_dir()` does not.
// Fix Applied: Replaced `path.exists()` with `path.is_dir()` in `resolve_root()`.
//   Only a confirmed directory is accepted as a valid `$PRO` root.
// Prevention: Use `is_dir()` for any environment variable that must resolve to a
//   directory root. Reserve `exists()` for cases where file vs. directory is irrelevant.
// Pitfall: `is_dir()` returns `false` for non-existent paths (same as `exists()`),
//   so the fallback behaviour for missing paths is preserved — no separate check needed.
#[ test ]
fn p14_pro_set_to_existing_file_falls_back_to_home()
{
  let _lock    = lock();
  let home_dir = TempDir::new().unwrap();
  // create a plain file (not directory) to assign to $PRO
  let file_path = home_dir.path().join( "not_a_dir.txt" );
  std::fs::write( &file_path, b"just a file" ).expect( "p14: creating test file must succeed" );

  let orig_pro  = env::var( "PRO"  ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::set_var( "PRO",  &file_path );
  env::set_var( "HOME", home_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p14: PersistPaths::new() must succeed when $HOME is set" );
  assert!(
  paths.base().starts_with( home_dir.path() ),
  "p14: base() must start with $HOME when $PRO is a file (not a directory); got: {}",
  paths.base().display()
  );
  assert!(
  !paths.base().starts_with( &file_path ),
  "p14: base() must NOT be under the file path; got: {}",
  paths.base().display()
  );
}

/// p15 — `Debug` formatting produces non-empty output
///
/// `PersistPaths` derives `Debug`. The formatted output must be a non-empty string
/// that includes the resolved base path, useful for diagnostics and logging.
#[ test ]
fn p15_debug_formatting_is_non_empty()
{
  let _lock   = lock();
  let pro_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO"  ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::set_var( "PRO", pro_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p15: PersistPaths::new() must succeed" );
  let debug_str = format!( "{paths:?}" );
  assert!(
  !debug_str.is_empty(),
  "p15: Debug formatting must produce non-empty output"
  );
  assert!(
  debug_str.contains( "persistent" ),
  "p15: Debug output must include the base path; got: {debug_str:?}"
  );
}

/// p16 — `credential_store()` resolves under `$PRO` when `$PRO` is set
#[ test ]
fn p16_credential_store_under_pro()
{
  let _lock   = lock();
  let pro_dir = TempDir::new().unwrap();
  let home_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO"  ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::set_var( "PRO",  pro_dir.path() );
  env::set_var( "HOME", home_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p16: PersistPaths::new() must succeed" );
  assert!(
  paths.credential_store().starts_with( pro_dir.path() ),
  "p16: credential_store() must start with $PRO; got: {}",
  paths.credential_store().display()
  );
}

/// p17 — `credential_store()` path shape ends with `.persistent/claude/credential`
#[ test ]
fn p17_credential_store_path_shape_under_pro()
{
  let _lock   = lock();
  let pro_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO"  ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::set_var( "PRO", pro_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p17: PersistPaths::new() must succeed" );
  assert!(
  paths.credential_store().ends_with( ".persistent/claude/credential" ),
  "p17: credential_store() must end with .persistent/claude/credential; got: {}",
  paths.credential_store().display()
  );
}

/// p18 — `credential_store()` path shape ends with `.persistent/claude/credential` under `$HOME`
#[ test ]
fn p18_credential_store_path_shape_under_home()
{
  let _lock    = lock();
  let home_dir = TempDir::new().unwrap();
  let orig_pro  = env::var( "PRO"  ).ok();
  let orig_home = env::var( "HOME" ).ok();

  env::remove_var( "PRO" );
  env::set_var( "HOME", home_dir.path() );

  let result = PersistPaths::new();

  restore( "PRO",  orig_pro  );
  restore( "HOME", orig_home );

  let paths = result.expect( "p18: PersistPaths::new() must succeed" );
  assert!(
  paths.credential_store().ends_with( ".persistent/claude/credential" ),
  "p18: credential_store() must end with .persistent/claude/credential under $HOME; got: {}",
  paths.credential_store().display()
  );
}

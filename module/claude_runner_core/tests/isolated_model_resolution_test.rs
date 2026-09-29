//! `resolve_isolated_default_model()` tiered resolution tests (tasks 407, 410; BUG-007).
//!
//! Covers the 2-tier chain `IsolatedModel::Default` resolves through:
//! project `.clr.toml` → user `config.toml` → `None` (caller falls back
//! to `DEFAULT_MODEL`, unchanged, covered by `isolated_test.rs` T10).
//! The user tier is wherever `claude_runner_core::user_config_path()` puts it:
//! `$CLR_CONFIG_DIR/config.toml` when that override is set and non-empty,
//! otherwise `$HOME/.clr/config.toml` (the path's own decision table lives in
//! `config_path_test.rs`).
//! Task 410 removed the prior `~/.clr/prefs.json` fallback tier entirely —
//! `read_subprocess_model_pref()` no longer exists. `claude_core::settings_io`
//! itself is untouched — it remains live, shared code used by
//! `claude_version`/`claude_version_core` for unrelated `~/.claude/settings.json`
//! management.
//!
//! Each test runs in its own process under cargo-nextest, so mutating `HOME`,
//! `CLR_CONFIG_DIR` and the process CWD is safe — matches the established
//! `HOME`-mutation pattern used throughout this workspace's test suites. T4-T7
//! remove `CLR_CONFIG_DIR` so an inherited value can't redirect their user tier.
//!
//! ## Test Matrix
//!
//! | ID | Scenario | Expected |
//! |----|----------|----------|
//! | T4 | `~/.clr/config.toml` has `model = "claude-opus-4-8"`, no project file | `Some("claude-opus-4-8")` |
//! | T5 | project `.clr.toml` and user `config.toml` both set `model` to different values | `Some(project's value)` |
//! | T6 | config.toml unset, a `~/.clr/prefs.json` file with a value is present (dead file) | `None` — regression guard proving the fallback tier is gone |
//! | T7 | neither config.toml nor prefs.json set | `None` |
//! | T8 | `$HOME/.clr/config.toml` and `$CLR_CONFIG_DIR/config.toml` set different models | `Some(override's value)` (BUG-007) |
//! | T9 | project `.clr.toml` and `$CLR_CONFIG_DIR/config.toml` set different models | `Some(project's value)` |
//! | T10 | `HOME` unset, `$CLR_CONFIG_DIR/config.toml` sets `model` | `Some(override's value)` (BUG-007) |

use claude_runner_core::resolve_isolated_default_model;

fn write_file( dir : &std::path::Path, name : &str, content : &str ) -> std::path::PathBuf
{
  let path = dir.join( name );
  std::fs::write( &path, content ).expect( "write file" );
  path
}

// ── T4 ───────────────────────────────────────────────────────────────────────

/// T4: `~/.clr/config.toml`'s `model` key is honored when no project `.clr.toml` exists.
#[ test ]
fn t4_config_toml_model_set_is_honored()
{
  let home_dir = tempfile::TempDir::new().expect( "temp HOME dir" );
  let clr_dir  = home_dir.path().join( ".clr" );
  std::fs::create_dir_all( &clr_dir ).expect( "create .clr dir" );
  write_file( &clr_dir, "config.toml", "model = \"claude-opus-4-8\"\n" );

  let project_dir = tempfile::TempDir::new().expect( "temp project dir (no .clr.toml inside)" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into empty project dir" );
  std::env::set_var( "HOME", home_dir.path() );
  std::env::remove_var( "CLR_CONFIG_DIR" );

  assert_eq!( resolve_isolated_default_model(), Some( "claude-opus-4-8".to_string() ) );
}

// ── T5 ───────────────────────────────────────────────────────────────────────

/// T5: project `.clr.toml`'s `model` overrides user `~/.clr/config.toml`'s `model`.
#[ test ]
fn t5_project_tier_overrides_user_tier()
{
  let home_dir = tempfile::TempDir::new().expect( "temp HOME dir" );
  let clr_dir  = home_dir.path().join( ".clr" );
  std::fs::create_dir_all( &clr_dir ).expect( "create .clr dir" );
  write_file( &clr_dir, "config.toml", "model = \"user-value\"\n" );

  let project_dir = tempfile::TempDir::new().expect( "temp project dir" );
  write_file( project_dir.path(), ".clr.toml", "model = \"project-value\"\n" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into project dir" );
  std::env::set_var( "HOME", home_dir.path() );
  std::env::remove_var( "CLR_CONFIG_DIR" );

  assert_eq!( resolve_isolated_default_model(), Some( "project-value".to_string() ) );
}

// ── T6 ───────────────────────────────────────────────────────────────────────

/// T6: regression guard — task 410 removed the `~/.clr/prefs.json` fallback
/// tier entirely. A `prefs.json` with a value present must NOT be consulted;
/// `config.toml` unset means the result is `None`.
#[ test ]
fn t6_prefs_json_no_longer_consulted_when_config_toml_unset()
{
  let home_dir = tempfile::TempDir::new().expect( "temp HOME dir" );
  let clr_dir  = home_dir.path().join( ".clr" );
  std::fs::create_dir_all( &clr_dir ).expect( "create .clr dir" );
  write_file( &clr_dir, "prefs.json", r#"{"subprocess_model":"claude-sonnet-5"}"# );

  let project_dir = tempfile::TempDir::new().expect( "temp project dir (no .clr.toml inside)" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into empty project dir" );
  std::env::set_var( "HOME", home_dir.path() );
  std::env::remove_var( "CLR_CONFIG_DIR" );

  assert_eq!( resolve_isolated_default_model(), None );
}

// ── T7 ───────────────────────────────────────────────────────────────────────

/// T7: neither `config.toml` nor `prefs.json` set anywhere → `None` (caller's
/// existing `model.model_id()` fallback then supplies `DEFAULT_MODEL`).
#[ test ]
fn t7_neither_set_returns_none()
{
  let home_dir = tempfile::TempDir::new().expect( "temp HOME dir" );

  let project_dir = tempfile::TempDir::new().expect( "temp project dir (no .clr.toml inside)" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into empty project dir" );
  std::env::set_var( "HOME", home_dir.path() );
  std::env::remove_var( "CLR_CONFIG_DIR" );

  assert_eq!( resolve_isolated_default_model(), None );
}

// ── T8 ───────────────────────────────────────────────────────────────────────

/// T8: `$CLR_CONFIG_DIR/config.toml` is the user tier when the override is set, even with a
/// different model in `$HOME/.clr/config.toml`.
///
/// # Root Cause
///
/// `resolve_isolated_default_model()` built the user tier as `$HOME/.clr/config.toml` itself
/// (`isolated.rs:226-227`) and never read `CLR_CONFIG_DIR`. The only override-aware resolver
/// was `user_config_dir()`, private to the `clr` binary, so `clr run` read the redirected file
/// while `clr isolated` read the `HOME` one.
///
/// # Why Not Caught
///
/// T4-T7 isolate through `HOME` alone, where both locations coincide. The runner's isolated
/// tests remove `CLR_CONFIG_DIR` instead of setting it, and `config_file_test.rs` T10 covers
/// the override for `clr run` only.
///
/// # Fix Applied
///
/// `claude_runner_core::user_config_path()` (`src/config_path.rs`) is now the one resolver for
/// the user `config.toml`: a non-empty override wins, `HOME` is the fallback, and neither gives
/// `None`. `resolve_isolated_default_model()` and the runner's config tier both call it.
/// Tracked as BUG-007.
///
/// # Prevention
///
/// T8 and T10 point the override at a different directory than `HOME`, so a resolver that
/// reads `HOME` alone fails here instead of passing by coincidence.
///
/// # Pitfall
///
/// A temp `HOME` only isolates code that reads `HOME`. When an override can relocate a file,
/// a test must point the override and `HOME` at different places, or both paths agree and
/// every resolver passes.
// test_kind: bug_reproducer(BUG-007)
#[ test ]
fn t8_clr_config_dir_overrides_home_user_tier()
{
  let home_dir = tempfile::TempDir::new().expect( "temp HOME dir" );
  let clr_dir  = home_dir.path().join( ".clr" );
  std::fs::create_dir_all( &clr_dir ).expect( "create .clr dir" );
  write_file( &clr_dir, "config.toml", "model = \"home-value\"\n" );

  let override_dir = tempfile::TempDir::new().expect( "temp CLR_CONFIG_DIR" );
  write_file( override_dir.path(), "config.toml", "model = \"override-value\"\n" );

  let project_dir = tempfile::TempDir::new().expect( "temp project dir (no .clr.toml inside)" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into empty project dir" );
  std::env::set_var( "HOME", home_dir.path() );
  std::env::set_var( "CLR_CONFIG_DIR", override_dir.path() );

  assert_eq!( resolve_isolated_default_model(), Some( "override-value".to_string() ) );
}

// ── T9 ───────────────────────────────────────────────────────────────────────

/// T9: project `.clr.toml` still overrides the user tier when that tier comes from
/// `CLR_CONFIG_DIR`. The override relocates the user tier only — decision D-07 in
/// `task/claude_runner/decisions.md` leaves project `.clr.toml` discovery unconditional.
#[ test ]
fn t9_project_tier_overrides_clr_config_dir_user_tier()
{
  let home_dir = tempfile::TempDir::new().expect( "temp HOME dir" );

  let override_dir = tempfile::TempDir::new().expect( "temp CLR_CONFIG_DIR" );
  write_file( override_dir.path(), "config.toml", "model = \"override-value\"\n" );

  let project_dir = tempfile::TempDir::new().expect( "temp project dir" );
  write_file( project_dir.path(), ".clr.toml", "model = \"project-value\"\n" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into project dir" );
  std::env::set_var( "HOME", home_dir.path() );
  std::env::set_var( "CLR_CONFIG_DIR", override_dir.path() );

  assert_eq!( resolve_isolated_default_model(), Some( "project-value".to_string() ) );
}

// ── T10 ──────────────────────────────────────────────────────────────────────

/// T10: with `HOME` unset, a set `CLR_CONFIG_DIR` still locates the user tier. See T8 for BUG-007.
// test_kind: bug_reproducer(BUG-007)
#[ test ]
fn t10_clr_config_dir_honored_without_home()
{
  let override_dir = tempfile::TempDir::new().expect( "temp CLR_CONFIG_DIR" );
  write_file( override_dir.path(), "config.toml", "model = \"override-value\"\n" );

  let project_dir = tempfile::TempDir::new().expect( "temp project dir (no .clr.toml inside)" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into empty project dir" );
  std::env::remove_var( "HOME" );
  std::env::set_var( "CLR_CONFIG_DIR", override_dir.path() );

  assert_eq!( resolve_isolated_default_model(), Some( "override-value".to_string() ) );
}

//! `resolve_isolated_default_model()` tiered resolution tests (tasks 407, 410).
//!
//! Covers the 2-tier chain `IsolatedModel::Default` resolves through:
//! project `.clr.toml` → user `~/.clr/config.toml` → `None` (caller falls back
//! to `ISOLATED_DEFAULT_MODEL`, unchanged, covered by `isolated_test.rs` T10).
//! Task 410 removed the prior `~/.clr/prefs.json` fallback tier entirely —
//! `read_subprocess_model_pref()` no longer exists. `claude_core::settings_io`
//! itself is untouched — it remains live, shared code used by
//! `claude_version`/`claude_version_core` for unrelated `~/.claude/settings.json`
//! management.
//!
//! Each test runs in its own process under cargo-nextest, so mutating `HOME`
//! and the process CWD is safe — matches the established `HOME`-mutation
//! pattern used throughout this workspace's test suites.
//!
//! ## Test Matrix
//!
//! | ID | Scenario | Expected |
//! |----|----------|----------|
//! | T4 | `~/.clr/config.toml` has `model = "claude-opus-4-8"`, no project file | `Some("claude-opus-4-8")` |
//! | T5 | project `.clr.toml` and user `config.toml` both set `model` to different values | `Some(project's value)` |
//! | T6 | config.toml unset, a `~/.clr/prefs.json` file with a value is present (dead file) | `None` — regression guard proving the fallback tier is gone |
//! | T7 | neither config.toml nor prefs.json set | `None` |

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

  assert_eq!( resolve_isolated_default_model(), None );
}

// ── T7 ───────────────────────────────────────────────────────────────────────

/// T7: neither `config.toml` nor `prefs.json` set anywhere → `None` (caller's
/// existing `model.model_id()` fallback then supplies `ISOLATED_DEFAULT_MODEL`).
#[ test ]
fn t7_neither_set_returns_none()
{
  let home_dir = tempfile::TempDir::new().expect( "temp HOME dir" );

  let project_dir = tempfile::TempDir::new().expect( "temp project dir (no .clr.toml inside)" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into empty project dir" );
  std::env::set_var( "HOME", home_dir.path() );

  assert_eq!( resolve_isolated_default_model(), None );
}

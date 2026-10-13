//! `resolve_isolated_default_model()` tiered resolution tests (tasks 407, 410; BUG-007).
//!
//! Covers the 2-tier chain `IsolatedModel::Default` resolves through:
//! project `.clr.toml` → user `config.toml` → `None` (caller falls back
//! to `DEFAULT_MODEL`, unchanged, covered by `isolated_test.rs` T10).
//! The user tier is wherever `claude_runner_core::user_config_path()` puts it:
//! `$CLR_CONFIG_DIR/config.toml` when that override is set and non-empty,
//! otherwise `$HOME/.clr/config.toml` (the path's own decision table lives in
//! `config_path_test.rs`). With no user tier at all, the project file isn't read either (T11).
//! Task 410 removed the prior `~/.clr/prefs.json` fallback tier entirely —
//! `read_subprocess_model_pref()` no longer exists. `claude_core::settings_io`
//! itself is untouched — it remains live, shared code used by
//! `claude_version`/`claude_version_core` for unrelated `~/.claude/settings.json`
//! management.
//!
//! Each test runs in its own process under cargo-nextest, so mutating `HOME`,
//! `CLR_CONFIG_DIR` and the process CWD is safe — matches the established
//! `HOME`-mutation pattern used throughout this workspace's test suites. T4-T7 and T11
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
//! | T11 | `CLR_CONFIG_DIR` unset, `HOME` unset or empty, project `.clr.toml` sets `model` | `None` — no user tier, project not read |
//! | T12 | `CLR_CONFIG_DIR` has no `config.toml` (its directory present or missing), `$HOME/.clr/config.toml` sets `model` | `None` — `HOME`'s file not read |
//! | T13 | `CLR_CONFIG_DIR` empty, `$HOME/.clr/config.toml` sets `model` | `Some(HOME's value)` |
//! | T14 | `$CLR_CONFIG_DIR/config.toml` sets only `effort`, is empty, or holds only a comment; `$HOME/.clr/config.toml` sets `model` | `None` — `HOME`'s file not read |

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
/// (`isolated.rs:226-227` before `cc988323`) and never read `CLR_CONFIG_DIR`. The only
/// override-aware resolver was `user_config_dir()`, private to the `clr` binary, so `clr run`
/// read the redirected file while `clr isolated` read the `HOME` one.
///
/// # Why Not Caught
///
/// T4-T7 isolate through `HOME` alone, where both locations coincide. The runner's isolated
/// tests remove `CLR_CONFIG_DIR` (ISD-14) or use the project tier (BUG-485's), so none set
/// the override, and `config_file_test.rs` T10 covers it for `clr run` only.
///
/// # Fix Applied
///
/// `claude_runner_core::user_config_path()` (`src/config_path.rs:42-49`, a wrapper over
/// `user_config_path_from()` at `:26-35`) is now the one resolver for the user `config.toml`: a
/// non-empty override wins, a non-empty `HOME` is the fallback, and neither gives `None`.
/// `resolve_isolated_default_model()` (`src/isolated.rs:239`) and the runner's
/// `discover_config_paths()` (`claude_runner/src/cli/config.rs:117`) both call it, and the
/// runner's private `user_config_dir()` is gone. Landed in `cc988323`; tracked as BUG-007.
///
/// # Prevention
///
/// T8 points the override at a different directory than `HOME` and T10 unsets `HOME`, so a
/// resolver that reads `HOME` alone fails here instead of passing by coincidence. Every reader
/// or writer of the user `config.toml` calls `user_config_path()`, whose decision table is
/// `config_path_test.rs`; in review, a hand-built `HOME` + `.clr/config.toml` join anywhere
/// else is this bug again. A new reader gets a test that sets the override and `HOME` apart,
/// as `claude_runner`'s `bug007_isolated_dry_run_reads_user_tier_through_clr_config_dir` does.
///
/// # Pitfall
///
/// A temp `HOME` only isolates code that reads `HOME`. When an override can relocate a file,
/// a test must point the override and `HOME` at different places, or both paths agree and
/// every resolver passes. An override implemented inside one consumer redirects only that
/// consumer: the file's other readers keep agreeing with each other, so the split shows only
/// while the override is set. clp's three readers and writers of this file had the same
/// shape (BUG-560, `task/claude_profile/bug/`). Its fix routes them through this resolver:
/// `.model` and `.provider.select` at `claude_profile/src/commands/cmd_context.rs:39`,
/// `.usage` at `claude_profile/src/usage/render.rs:41`.
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
/// `CLR_CONFIG_DIR`. The override moves the user tier and leaves project discovery alone
/// (decision D-07 in `task/claude_runner/decisions.md`). For `isolated`, though, the project
/// file is read only when a user tier location resolves; T11 pins the case where none does.
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

/// T10: with `HOME` unset, a set `CLR_CONFIG_DIR` still locates the user tier. T8 is the
/// tagged reproducer for BUG-007; this test covers the `HOME`-unset variant and carries no
/// separate `bug_reproducer` tag of its own (TG012 — one reproducer function per bug).
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

// ── T11 ──────────────────────────────────────────────────────────────────────

/// T11: with no user tier (`CLR_CONFIG_DIR` unset, `HOME` unset or empty), a project
/// `.clr.toml` isn't read either, so the caller falls back to `DEFAULT_MODEL`. `clr run`
/// still reads the project file here. This pins current behavior, which open decision D-14
/// (`task/claude_runner/decisions.md`: should `isolated` get the full config tier) may change.
/// With `HOME` empty the skip is new with BUG-007's fix: the old lookup built a cwd-relative
/// user path and still read the project file.
#[ test ]
fn t11_no_user_tier_skips_project_tier()
{
  let project_dir = tempfile::TempDir::new().expect( "temp project dir" );
  write_file( project_dir.path(), ".clr.toml", "model = \"project-value\"\n" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into project dir" );
  std::env::remove_var( "CLR_CONFIG_DIR" );

  std::env::remove_var( "HOME" );
  assert_eq!( resolve_isolated_default_model(), None, "HOME unset" );

  std::env::set_var( "HOME", "" );
  assert_eq!( resolve_isolated_default_model(), None, "HOME empty" );
}

// ── T12 ──────────────────────────────────────────────────────────────────────

/// T12: a set override with no `config.toml`, in a directory that exists or one that doesn't,
/// is still the user tier: `$HOME/.clr/config.toml` isn't read and the result is `None`, as for
/// `clr run`. A lookup falling back to `HOME` here passes T8-T10 and splits the two commands.
#[ test ]
fn t12_clr_config_dir_without_file_does_not_fall_back_to_home()
{
  let home_dir = tempfile::TempDir::new().expect( "temp HOME dir" );
  let clr_dir  = home_dir.path().join( ".clr" );
  std::fs::create_dir_all( &clr_dir ).expect( "create .clr dir" );
  write_file( &clr_dir, "config.toml", "model = \"home-value\"\n" );

  let override_dir = tempfile::TempDir::new().expect( "temp CLR_CONFIG_DIR, no config.toml" );

  let project_dir = tempfile::TempDir::new().expect( "temp project dir (no .clr.toml inside)" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into empty project dir" );
  std::env::set_var( "HOME", home_dir.path() );
  std::env::set_var( "CLR_CONFIG_DIR", override_dir.path() );

  assert_eq!( resolve_isolated_default_model(), None, "directory without config.toml" );

  std::env::set_var( "CLR_CONFIG_DIR", override_dir.path().join( "missing" ) );
  assert_eq!( resolve_isolated_default_model(), None, "directory that doesn't exist" );
}

// ── T13 ──────────────────────────────────────────────────────────────────────

/// T13: an empty `CLR_CONFIG_DIR` counts as unset, so the user tier is
/// `$HOME/.clr/config.toml`, as it is for `clr run`. A lookup that treats the empty value as
/// set looks for a cwd-relative `config.toml` instead, and finds nothing here.
#[ test ]
fn t13_empty_clr_config_dir_falls_back_to_home()
{
  let home_dir = tempfile::TempDir::new().expect( "temp HOME dir" );
  let clr_dir  = home_dir.path().join( ".clr" );
  std::fs::create_dir_all( &clr_dir ).expect( "create .clr dir" );
  write_file( &clr_dir, "config.toml", "model = \"home-value\"\n" );

  let project_dir = tempfile::TempDir::new().expect( "temp project dir (no .clr.toml inside)" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into empty project dir" );
  std::env::set_var( "HOME", home_dir.path() );
  std::env::set_var( "CLR_CONFIG_DIR", "" );

  assert_eq!( resolve_isolated_default_model(), Some( "home-value".to_string() ) );
}

// ── T14 ──────────────────────────────────────────────────────────────────────

/// T14: a set override whose `config.toml` gives no `model` (it sets only `effort`, is empty, or
/// holds only a comment) is still the whole user tier: `$HOME/.clr/config.toml` isn't read and
/// the result is `None`, as for `clr run` (`claude_runner`'s `config_file_test.rs` T10). A lookup
/// that falls back to `HOME` whenever the override's file has no `model` passes T8-T13 and splits
/// the two commands.
#[ test ]
fn t14_clr_config_dir_file_without_model_does_not_fall_back_to_home()
{
  let home_dir = tempfile::TempDir::new().expect( "temp HOME dir" );
  let clr_dir  = home_dir.path().join( ".clr" );
  std::fs::create_dir_all( &clr_dir ).expect( "create .clr dir" );
  write_file( &clr_dir, "config.toml", "model = \"home-value\"\n" );

  let project_dir = tempfile::TempDir::new().expect( "temp project dir (no .clr.toml inside)" );
  std::env::set_current_dir( project_dir.path() ).expect( "chdir into empty project dir" );
  std::env::set_var( "HOME", home_dir.path() );

  for ( case, content ) in
  [
    ( "override config.toml with only `effort`", "effort = \"high\"\n" ),
    ( "empty override config.toml", "" ),
    ( "override config.toml with only a comment", "# no keys yet\n" ),
  ]
  {
    let override_dir = tempfile::TempDir::new().expect( "temp CLR_CONFIG_DIR" );
    write_file( override_dir.path(), "config.toml", content );
    std::env::set_var( "CLR_CONFIG_DIR", override_dir.path() );
    assert_eq!( resolve_isolated_default_model(), None, "{case}" );
  }
}

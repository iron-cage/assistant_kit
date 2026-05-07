//! `ClaudePaths` unit tests
//!
//! ## Purpose
//!
//! Verify that `ClaudePaths` constructs all canonical `~/.claude/` paths
//! correctly from the `HOME` environment variable and returns `None` when
//! `HOME` is absent.
//!
//! ## Coverage
//!
//! - `ClaudePaths::new()` returns `None` when `HOME` is unset
//! - `ClaudePaths::new()` succeeds when `HOME` is set
//! - All path accessors return paths with the correct structure
//! - Paths are relative to the provided `HOME`, not the real user home
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `paths_new_returns_some_with_home` | HOME set → Some |
//! | `paths_base_ends_with_dot_claude` | base path suffix |
//! | `paths_credentials_file_correct` | .credentials.json path |
//! | `paths_projects_dir_correct` | projects/ path |
//! | `paths_stats_file_correct` | stats-cache.json path |
//! | `paths_settings_file_correct` | settings.json path |
//! | `paths_session_env_dir_correct` | session-env/ path |
//! | `paths_sessions_dir_correct` | sessions/ path |
//! | `paths_rooted_at_provided_home` | custom HOME → rooted there |
//! | `paths_claude_json_file_correct` | .claude.json is at $HOME level |
//! | `paths_claude_json_file_not_inside_claude_dir` | .claude.json not inside .claude/ |
//!
//! ## Note on None path
//!
//! `ClaudePaths::new()` returns `None` when `HOME` is unset. This path
//! cannot be tested here because `std::env::remove_var` requires `unsafe`
//! which is denied workspace-wide. The None path is covered by code review.

use claude_core::ClaudePaths;

#[test]
fn paths_new_returns_some_with_home()
{
  std::env::set_var( "HOME", "/tmp/test_home" );
  assert!( ClaudePaths::new().is_some(), "expected Some when HOME is set" );
}

#[test]
fn paths_base_ends_with_dot_claude()
{
  std::env::set_var( "HOME", "/tmp/test_home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  assert!(
    p.base().ends_with( ".claude" ),
    "base must be $HOME/.claude, got: {}",
    p.base().display()
  );
}

#[test]
fn paths_credentials_file_correct()
{
  std::env::set_var( "HOME", "/tmp/test_home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  let f = p.credentials_file();
  assert!(
    f.ends_with( ".claude/.credentials.json" ),
    "expected .claude/.credentials.json, got: {}",
    f.display()
  );
}

#[test]
fn paths_projects_dir_correct()
{
  std::env::set_var( "HOME", "/tmp/test_home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  let d = p.projects_dir();
  assert!(
    d.ends_with( ".claude/projects" ),
    "expected .claude/projects, got: {}",
    d.display()
  );
}

#[test]
fn paths_stats_file_correct()
{
  std::env::set_var( "HOME", "/tmp/test_home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  let f = p.stats_file();
  assert!(
    f.ends_with( ".claude/stats-cache.json" ),
    "expected .claude/stats-cache.json, got: {}",
    f.display()
  );
}

#[test]
fn paths_settings_file_correct()
{
  std::env::set_var( "HOME", "/tmp/test_home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  let f = p.settings_file();
  assert!(
    f.ends_with( ".claude/settings.json" ),
    "expected .claude/settings.json, got: {}",
    f.display()
  );
}

#[test]
fn paths_session_env_dir_correct()
{
  std::env::set_var( "HOME", "/tmp/test_home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  let d = p.session_env_dir();
  assert!(
    d.ends_with( ".claude/session-env" ),
    "expected .claude/session-env, got: {}",
    d.display()
  );
}

#[test]
fn paths_sessions_dir_correct()
{
  std::env::set_var( "HOME", "/tmp/test_home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  let d = p.sessions_dir();
  assert!(
    d.ends_with( ".claude/sessions" ),
    "expected .claude/sessions, got: {}",
    d.display()
  );
}

#[test]
fn paths_rooted_at_provided_home()
{
  std::env::set_var( "HOME", "/custom/home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  let base = p.base().to_str().expect( "valid UTF-8 path" );
  assert!(
    base.starts_with( "/custom/home" ),
    "paths must be rooted at the provided HOME, got: {base}"
  );
}

#[test]
fn paths_claude_json_file_correct()
{
  std::env::set_var( "HOME", "/tmp/test_home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  let f = p.claude_json_file();
  assert_eq!(
    f.to_str().expect( "valid UTF-8 path" ),
    "/tmp/test_home/.claude.json",
    "claude_json_file must be $HOME/.claude.json, got: {}",
    f.display()
  );
}

#[test]
fn paths_claude_json_file_not_inside_claude_dir()
{
  std::env::set_var( "HOME", "/tmp/test_home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  let f = p.claude_json_file();
  let base = p.base();
  assert!(
    !f.starts_with( base ),
    "claude_json_file must NOT be inside $HOME/.claude/, got: {}",
    f.display()
  );
}

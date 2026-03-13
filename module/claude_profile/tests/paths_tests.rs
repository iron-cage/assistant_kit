//! Integration tests for `ClaudePaths` canonical path resolution.
//!
//! Each test sets `HOME` to an isolated fake path.
//! Safe because nextest runs every test in its own process.

use claude_profile::ClaudePaths;
use std::path::PathBuf;

#[ test ]
fn new_returns_none_when_home_not_set()
{
  // Safe: nextest runs each test in its own process,
  // so removing HOME here does not affect other tests.
  std::env::remove_var( "HOME" );
  assert!( ClaudePaths::new().is_none() );
}

#[ test ]
fn base_is_dot_claude_under_home()
{
  std::env::set_var( "HOME", "/test/home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  assert_eq!( p.base(), PathBuf::from( "/test/home/.claude" ) );
}

#[ test ]
fn credentials_file_returns_dot_credentials_json()
{
  std::env::set_var( "HOME", "/test/home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  assert_eq!(
    p.credentials_file(),
    PathBuf::from( "/test/home/.claude/.credentials.json" ),
  );
}

#[ test ]
fn accounts_dir_returns_accounts_under_base()
{
  std::env::set_var( "HOME", "/test/home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  assert_eq!( p.accounts_dir(), PathBuf::from( "/test/home/.claude/accounts" ) );
}

#[ test ]
fn projects_dir_returns_projects_under_base()
{
  std::env::set_var( "HOME", "/test/home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  assert_eq!( p.projects_dir(), PathBuf::from( "/test/home/.claude/projects" ) );
}

#[ test ]
fn stats_file_returns_stats_cache_json()
{
  std::env::set_var( "HOME", "/test/home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  assert_eq!( p.stats_file(), PathBuf::from( "/test/home/.claude/stats-cache.json" ) );
}

#[ test ]
fn settings_file_returns_settings_json()
{
  std::env::set_var( "HOME", "/test/home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  assert_eq!( p.settings_file(), PathBuf::from( "/test/home/.claude/settings.json" ) );
}

#[ test ]
fn session_env_dir_returns_session_env_under_base()
{
  std::env::set_var( "HOME", "/test/home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  assert_eq!(
    p.session_env_dir(),
    PathBuf::from( "/test/home/.claude/session-env" ),
  );
}

#[ test ]
fn sessions_dir_returns_sessions_under_base()
{
  std::env::set_var( "HOME", "/test/home" );
  let p = ClaudePaths::new().expect( "HOME is set" );
  assert_eq!( p.sessions_dir(), PathBuf::from( "/test/home/.claude/sessions" ) );
}

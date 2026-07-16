//! `toml_io` tiered get/set unit tests
//!
//! ## Purpose
//!
//! Verify `get_tiered` merges project and user TOML tiers (project wins),
//! treats missing files as absent (never panics/errors), and only accepts
//! plain double-quoted string values. Verify `set_user_tier` writes the
//! user-tier file and never clobbers sibling keys of other types. Verify
//! `remove_user_tier` deletes a key without disturbing siblings and is a
//! true no-op (no file created) when the key or file is absent.
//!
//! ## Coverage
//!
//! - User tier only, key present → `Some`
//! - Project tier overrides user tier when both set
//! - Both files absent → `None`, no panic
//! - Non-string value (bare/number) → `None`
//! - `set_user_tier` creates the file and the key is readable back
//! - `set_user_tier` preserves sibling keys of other types untouched
//! - `set_user_tier` overwrites an existing key's value
//! - `remove_user_tier` removes the key; `get_tiered` then returns `None`
//! - `remove_user_tier` preserves sibling keys
//! - `remove_user_tier` on an absent file is a no-op (no file created)
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `get_tiered_returns_user_tier_value` | T1: user file has `model = "x"`, no project path → `Some("x")` |
//! | `get_tiered_project_overrides_user` | T2: both set to different values → project's value wins |
//! | `get_tiered_both_absent_returns_none` | T3: both files missing → `None`, no panic |
//! | `get_tiered_rejects_non_string_value` | `model = 42` (bare number) → `None` |
//! | `set_user_tier_creates_file_and_is_readable` | absent file → key created, `get_tiered` reads it back |
//! | `set_user_tier_preserves_sibling_keys` | pre-existing string/number/bool keys untouched after setting `model` |
//! | `set_user_tier_overwrites_existing_key` | pre-existing `model` value replaced by new value |
//! | `remove_user_tier_removes_key` | pre-existing `model` key → removed, `get_tiered` returns `None` |
//! | `remove_user_tier_preserves_sibling_keys` | pre-existing sibling keys untouched after removing `model` |
//! | `remove_user_tier_missing_file_is_noop` | absent file → `Ok(())`, no file created |

use claude_core::toml_io::{ get_tiered, set_user_tier, remove_user_tier };

fn write_toml( dir : &std::path::Path, name : &str, raw : &str ) -> std::path::PathBuf
{
  let path = dir.join( name );
  std::fs::write( &path, raw ).expect( "write toml file" );
  path
}

#[test]
fn get_tiered_returns_user_tier_value()
{
  let dir       = tempfile::TempDir::new().expect( "temp dir" );
  let user_path = write_toml( dir.path(), "config.toml", "model = \"x\"\n" );
  assert_eq!( get_tiered( None, &user_path, "model" ), Some( "x".to_string() ) );
}

#[test]
fn get_tiered_project_overrides_user()
{
  let dir          = tempfile::TempDir::new().expect( "temp dir" );
  let user_path    = write_toml( dir.path(), "config.toml", "model = \"user-value\"\n" );
  let project_path = write_toml( dir.path(), ".clr.toml", "model = \"project-value\"\n" );
  assert_eq!(
    get_tiered( Some( &project_path ), &user_path, "model" ),
    Some( "project-value".to_string() )
  );
}

#[test]
fn get_tiered_both_absent_returns_none()
{
  let dir             = tempfile::TempDir::new().expect( "temp dir" );
  let missing_user    = dir.path().join( "config.toml" );
  let missing_project = dir.path().join( ".clr.toml" );
  assert_eq!( get_tiered( Some( &missing_project ), &missing_user, "model" ), None );
}

#[test]
fn get_tiered_rejects_non_string_value()
{
  let dir       = tempfile::TempDir::new().expect( "temp dir" );
  let user_path = write_toml( dir.path(), "config.toml", "model = 42\n" );
  assert_eq!( get_tiered( None, &user_path, "model" ), None );
}

#[test]
fn set_user_tier_creates_file_and_is_readable()
{
  let dir       = tempfile::TempDir::new().expect( "temp dir" );
  let user_path = dir.path().join( "config.toml" );
  set_user_tier( &user_path, "model", "claude-sonnet-5" ).expect( "set_user_tier must succeed" );
  assert_eq!( get_tiered( None, &user_path, "model" ), Some( "claude-sonnet-5".to_string() ) );
}

#[test]
fn set_user_tier_preserves_sibling_keys()
{
  let dir       = tempfile::TempDir::new().expect( "temp dir" );
  let user_path = write_toml(
    dir.path(),
    "config.toml",
    "max_tokens = 200000\nno_effort_max = true\nname = \"unchanged\"\n",
  );
  set_user_tier( &user_path, "model", "claude-opus-4-8" ).expect( "set_user_tier must succeed" );

  let raw = std::fs::read_to_string( &user_path ).expect( "read back config.toml" );
  assert!( raw.contains( "max_tokens = 200000" ), "sibling number key must survive untouched. Got:\n{raw}" );
  assert!( raw.contains( "no_effort_max = true" ), "sibling bool key must survive untouched. Got:\n{raw}" );
  assert_eq!( get_tiered( None, &user_path, "name" ), Some( "unchanged".to_string() ) );
  assert_eq!( get_tiered( None, &user_path, "model" ), Some( "claude-opus-4-8".to_string() ) );
}

#[test]
fn set_user_tier_overwrites_existing_key()
{
  let dir       = tempfile::TempDir::new().expect( "temp dir" );
  let user_path = write_toml( dir.path(), "config.toml", "model = \"old-value\"\n" );
  set_user_tier( &user_path, "model", "new-value" ).expect( "set_user_tier must succeed" );
  assert_eq!( get_tiered( None, &user_path, "model" ), Some( "new-value".to_string() ) );
}

#[test]
fn remove_user_tier_removes_key()
{
  let dir       = tempfile::TempDir::new().expect( "temp dir" );
  let user_path = write_toml( dir.path(), "config.toml", "model = \"claude-opus-4-8\"\n" );
  remove_user_tier( &user_path, "model" ).expect( "remove_user_tier must succeed" );
  assert_eq!( get_tiered( None, &user_path, "model" ), None );
}

#[test]
fn remove_user_tier_preserves_sibling_keys()
{
  let dir       = tempfile::TempDir::new().expect( "temp dir" );
  let user_path = write_toml(
    dir.path(),
    "config.toml",
    "model = \"claude-opus-4-8\"\nmax_tokens = 200000\nname = \"unchanged\"\n",
  );
  remove_user_tier( &user_path, "model" ).expect( "remove_user_tier must succeed" );

  let raw = std::fs::read_to_string( &user_path ).expect( "read back config.toml" );
  assert!( raw.contains( "max_tokens = 200000" ), "sibling number key must survive untouched. Got:\n{raw}" );
  assert_eq!( get_tiered( None, &user_path, "name" ), Some( "unchanged".to_string() ) );
  assert_eq!( get_tiered( None, &user_path, "model" ), None );
}

#[test]
fn remove_user_tier_missing_file_is_noop()
{
  let dir       = tempfile::TempDir::new().expect( "temp dir" );
  let user_path = dir.path().join( "config.toml" );
  remove_user_tier( &user_path, "model" ).expect( "remove_user_tier must succeed on absent file" );
  assert!( !user_path.exists(), "remove_user_tier must not create the file when it was absent" );
}

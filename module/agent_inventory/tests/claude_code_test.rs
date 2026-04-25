//! Integration tests for the Claude Code adapter.
//!
//! ## Root Cause
//! New crate — no prior bugs; these tests establish the behavioral contract for
//! `SyncStatus` computation from source/target filesystem state.
//!
//! ## Why Not Caught
//! N/A — greenfield implementation.
//!
//! ## Fix Applied
//! N/A — TDD baseline tests.
//!
//! ## Prevention
//! Run `cargo nextest run -p agent_inventory --all-features` in CI.
//!
//! ## Pitfall
//! Directory-layout artifacts (skills, plugins) use `is_dir()` matching; dangling symlinks
//! are included via `symlink_metadata()` in `claude_assets_core::registry`. Tests must
//! create real symlinks — not just files — to simulate the installed state.

#![ cfg( feature = "claude_code" ) ]

use agent_inventory::adapter::AgentAdapter;
use agent_inventory::claude_code::ClaudeCodeAdapter;
use agent_inventory::entry::{ AssetKind, SyncStatus };
use agent_inventory::inventory::Inventory;
use claude_assets_core::paths::AssetPaths;
use std::fs;
use tempfile::TempDir;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_paths( source : &TempDir, target : &TempDir ) -> AssetPaths
{
  AssetPaths::new( source.path().to_path_buf(), target.path().to_path_buf() )
}

// ── Adapter identity ──────────────────────────────────────────────────────────

#[ test ]
fn adapter_name_is_claude_code()
{
  let source = tempfile::tempdir().unwrap();
  let target = tempfile::tempdir().unwrap();
  let adapter = ClaudeCodeAdapter::with_paths( make_paths( &source, &target ) );
  assert_eq!( adapter.name(), "claude_code" );
}

// ── Empty source ──────────────────────────────────────────────────────────────

#[ test ]
fn empty_source_returns_empty_entries()
{
  let source = tempfile::tempdir().unwrap();
  let target = tempfile::tempdir().unwrap();
  let adapter = ClaudeCodeAdapter::with_paths( make_paths( &source, &target ) );
  let entries = adapter.list_all().expect( "empty source must not error" );
  assert!( entries.is_empty(), "expected empty; got {} entries", entries.len() );
}

// ── NotInstalled ──────────────────────────────────────────────────────────────

#[ test ]
fn source_skill_not_installed_gives_not_installed_status()
{
  let source = tempfile::tempdir().unwrap();
  let target = tempfile::tempdir().unwrap();
  // Create a skill directory in source (directory-layout artifact)
  fs::create_dir_all( source.path().join( "skills" ).join( "commit" ) ).unwrap();
  let adapter = ClaudeCodeAdapter::with_paths( make_paths( &source, &target ) );
  let entries = adapter.list_by_kind( AssetKind::Skill ).expect( "list_by_kind must not error" );
  assert_eq!( entries.len(), 1 );
  assert_eq!( entries[ 0 ].name,   "commit" );
  assert_eq!( entries[ 0 ].kind,   AssetKind::Skill );
  assert_eq!( entries[ 0 ].agent,  "claude_code" );
  assert_eq!( entries[ 0 ].status, SyncStatus::NotInstalled );
}

#[ test ]
fn source_rule_not_installed_gives_not_installed_status()
{
  let source = tempfile::tempdir().unwrap();
  let target = tempfile::tempdir().unwrap();
  // Create a rule file in source (file-layout artifact)
  let rules_dir = source.path().join( "rules" );
  fs::create_dir_all( &rules_dir ).unwrap();
  fs::write( rules_dir.join( "rust.md" ), "# rust rule" ).unwrap();
  let adapter = ClaudeCodeAdapter::with_paths( make_paths( &source, &target ) );
  let entries = adapter.list_by_kind( AssetKind::Rule ).expect( "list_by_kind must not error" );
  assert_eq!( entries.len(), 1 );
  assert_eq!( entries[ 0 ].name,   "rust" );
  assert_eq!( entries[ 0 ].status, SyncStatus::NotInstalled );
}

// ── Synced ────────────────────────────────────────────────────────────────────

#[ test ]
fn installed_skill_gives_synced_status()
{
  let source = tempfile::tempdir().unwrap();
  let target = tempfile::tempdir().unwrap();
  // Create source skill
  let source_skill = source.path().join( "skills" ).join( "commit" );
  fs::create_dir_all( &source_skill ).unwrap();
  // Create symlink in target
  let target_skills = target.path().join( ".claude" ).join( "skills" );
  fs::create_dir_all( &target_skills ).unwrap();
  std::os::unix::fs::symlink( &source_skill, target_skills.join( "commit" ) ).unwrap();
  let adapter = ClaudeCodeAdapter::with_paths( make_paths( &source, &target ) );
  let entries = adapter.list_by_kind( AssetKind::Skill ).expect( "list_by_kind must not error" );
  assert_eq!( entries.len(), 1 );
  assert_eq!( entries[ 0 ].name,   "commit" );
  assert_eq!( entries[ 0 ].status, SyncStatus::Synced );
}

// ── Orphaned ──────────────────────────────────────────────────────────────────

#[ test ]
fn dangling_symlink_gives_orphaned_status()
{
  let source = tempfile::tempdir().unwrap();
  let target = tempfile::tempdir().unwrap();
  // No source skill — only a dangling symlink in target
  let target_skills = target.path().join( ".claude" ).join( "skills" );
  fs::create_dir_all( &target_skills ).unwrap();
  let ghost_source = source.path().join( "skills" ).join( "old_skill" );
  std::os::unix::fs::symlink( &ghost_source, target_skills.join( "old_skill" ) ).unwrap();
  let adapter = ClaudeCodeAdapter::with_paths( make_paths( &source, &target ) );
  let entries = adapter.list_by_kind( AssetKind::Skill ).expect( "list_by_kind must not error" );
  assert_eq!( entries.len(), 1, "expected 1 orphan; got {}", entries.len() );
  assert_eq!( entries[ 0 ].name,   "old_skill" );
  assert_eq!( entries[ 0 ].status, SyncStatus::Orphaned );
}

// ── Kind filter ───────────────────────────────────────────────────────────────

#[ test ]
fn list_by_kind_returns_only_requested_kind()
{
  let source = tempfile::tempdir().unwrap();
  let target = tempfile::tempdir().unwrap();
  // Skill
  fs::create_dir_all( source.path().join( "skills" ).join( "my_skill" ) ).unwrap();
  // Rule
  let rules_dir = source.path().join( "rules" );
  fs::create_dir_all( &rules_dir ).unwrap();
  fs::write( rules_dir.join( "my_rule.md" ), "# rule" ).unwrap();
  let adapter = ClaudeCodeAdapter::with_paths( make_paths( &source, &target ) );
  let skills = adapter.list_by_kind( AssetKind::Skill ).unwrap();
  let rules  = adapter.list_by_kind( AssetKind::Rule  ).unwrap();
  assert_eq!( skills.len(), 1 );
  assert_eq!( skills[ 0 ].kind, AssetKind::Skill );
  assert_eq!( rules.len(),  1 );
  assert_eq!( rules[ 0 ].kind, AssetKind::Rule );
}

// ── Multiple adapters merge ───────────────────────────────────────────────────

#[ test ]
fn inventory_merges_results_from_multiple_adapters()
{
  let source_a = tempfile::tempdir().unwrap();
  let target_a = tempfile::tempdir().unwrap();
  let source_b = tempfile::tempdir().unwrap();
  let target_b = tempfile::tempdir().unwrap();
  // Adapter A has one skill
  fs::create_dir_all( source_a.path().join( "skills" ).join( "skill_a" ) ).unwrap();
  // Adapter B has one skill
  fs::create_dir_all( source_b.path().join( "skills" ).join( "skill_b" ) ).unwrap();
  let mut inv = Inventory::new();
  inv.register( Box::new( ClaudeCodeAdapter::with_paths( make_paths( &source_a, &target_a ) ) ) );
  inv.register( Box::new( ClaudeCodeAdapter::with_paths( make_paths( &source_b, &target_b ) ) ) );
  let entries = inv.list_all().expect( "merged list_all must not error" );
  // Both skills from both adapters must appear
  let skill_names : Vec< &str > = entries.iter()
    .filter( | e | e.kind == AssetKind::Skill )
    .map(    | e | e.name.as_str() )
    .collect();
  assert!( skill_names.contains( &"skill_a" ), "skill_a missing: {skill_names:?}" );
  assert!( skill_names.contains( &"skill_b" ), "skill_b missing: {skill_names:?}" );
}

// ── All six kinds mapped ──────────────────────────────────────────────────────

#[ test ]
fn all_six_artifact_kinds_map_to_asset_kinds()
{
  // Verify that list_by_kind for each AssetKind works without panic
  let source = tempfile::tempdir().unwrap();
  let target = tempfile::tempdir().unwrap();
  let adapter = ClaudeCodeAdapter::with_paths( make_paths( &source, &target ) );
  for &kind in AssetKind::all()
  {
    let result = adapter.list_by_kind( kind );
    assert!( result.is_ok(), "list_by_kind({kind:?}) returned error: {result:?}" );
  }
}

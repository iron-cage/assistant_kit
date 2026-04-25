//! Integration tests for core types and `Inventory` registry.
//!
//! ## Root Cause
//! New crate — no prior bugs; these tests establish the behavioral contract.
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
//! `AssetKind::from_name` returns `None` for unknown strings — callers must handle `Option`.

use agent_inventory::entry::{ AssetKind, SyncStatus };
use agent_inventory::inventory::Inventory;

// ── AssetKind ─────────────────────────────────────────────────────────────────

#[ test ]
fn asset_kind_as_str_all_variants()
{
  assert_eq!( AssetKind::Skill.as_str(),   "skill" );
  assert_eq!( AssetKind::Command.as_str(), "command" );
  assert_eq!( AssetKind::Rule.as_str(),    "rule" );
  assert_eq!( AssetKind::Agent.as_str(),   "agent" );
  assert_eq!( AssetKind::Plugin.as_str(),  "plugin" );
  assert_eq!( AssetKind::Hook.as_str(),    "hook" );
}

#[ test ]
fn asset_kind_from_name_round_trip()
{
  for &kind in AssetKind::all()
  {
    let name = kind.as_str();
    assert_eq!(
      AssetKind::from_name( name ),
      Some( kind ),
      "round-trip failed for {name}"
    );
  }
}

#[ test ]
fn asset_kind_from_name_unknown_returns_none()
{
  assert_eq!( AssetKind::from_name( "unknown" ), None );
  assert_eq!( AssetKind::from_name( "" ),        None );
}

#[ test ]
fn asset_kind_all_has_six_variants()
{
  assert_eq!( AssetKind::all().len(), 6 );
}

// ── SyncStatus ────────────────────────────────────────────────────────────────

#[ test ]
fn sync_status_display_synced()
{
  assert_eq!( format!( "{}", SyncStatus::Synced ), "✅ synced" );
}

#[ test ]
fn sync_status_display_not_installed()
{
  assert_eq!( format!( "{}", SyncStatus::NotInstalled ), "⬇️ not installed" );
}

#[ test ]
fn sync_status_display_orphaned()
{
  assert_eq!( format!( "{}", SyncStatus::Orphaned ), "⚠️ orphaned" );
}

// ── Inventory ─────────────────────────────────────────────────────────────────

#[ test ]
fn empty_inventory_list_all_returns_empty()
{
  let inv = Inventory::new();
  let entries = inv.list_all().expect( "list_all on empty inventory must not fail" );
  assert!( entries.is_empty(), "expected empty Vec, got {}", entries.len() );
}

#[ test ]
fn empty_inventory_list_by_kind_returns_empty()
{
  let inv = Inventory::new();
  for &kind in AssetKind::all()
  {
    let entries = inv.list_by_kind( kind ).expect( "list_by_kind on empty inventory must not fail" );
    assert!( entries.is_empty(), "expected empty Vec for kind {:?}", kind );
  }
}

#[ test ]
fn inventory_default_equals_new()
{
  let a = Inventory::new();
  let b = Inventory::default();
  let a_entries = a.list_all().unwrap();
  let b_entries = b.list_all().unwrap();
  assert_eq!( a_entries.len(), b_entries.len() );
}

//! Claude Code adapter — wraps `claude_assets_core` for asset discovery.
//!
//! Gated under the `claude_code` feature to avoid the `claude_assets_core`
//! dependency for consumers that do not need Claude Code support.

#![ cfg( feature = "claude_code" ) ]

use std::collections::BTreeSet;

use claude_assets_core::artifact::ArtifactKind;
use claude_assets_core::paths::AssetPaths;
use claude_assets_core::registry;

use crate::adapter::AgentAdapter;
use crate::entry::{ AssetEntry, AssetKind, SyncStatus };
use crate::error::InventoryError;

// ── Kind mapping ──────────────────────────────────────────────────────────────

fn artifact_to_asset( kind : ArtifactKind ) -> AssetKind
{
  match kind
  {
    ArtifactKind::Skill   => AssetKind::Skill,
    ArtifactKind::Command => AssetKind::Command,
    ArtifactKind::Rule    => AssetKind::Rule,
    ArtifactKind::Agent   => AssetKind::Agent,
    ArtifactKind::Plugin  => AssetKind::Plugin,
    ArtifactKind::Hook    => AssetKind::Hook,
  }
}

fn asset_to_artifact( kind : AssetKind ) -> ArtifactKind
{
  match kind
  {
    AssetKind::Skill   => ArtifactKind::Skill,
    AssetKind::Command => ArtifactKind::Command,
    AssetKind::Rule    => ArtifactKind::Rule,
    AssetKind::Agent   => ArtifactKind::Agent,
    AssetKind::Plugin  => ArtifactKind::Plugin,
    AssetKind::Hook    => ArtifactKind::Hook,
  }
}

// ── Per-kind listing ──────────────────────────────────────────────────────────

fn list_for_kind(
  paths         : &AssetPaths,
  artifact_kind : ArtifactKind,
) -> Result< Vec< AssetEntry >, InventoryError >
{
  let asset_kind = artifact_to_asset( artifact_kind );

  let available = registry::list_available( paths, artifact_kind )
    .map_err( | e | InventoryError::Adapter
    {
      adapter : "claude_code".into(),
      message : e.to_string(),
    } )?;

  let installed = registry::list_installed( paths, artifact_kind )
    .map_err( | e | InventoryError::Adapter
    {
      adapter : "claude_code".into(),
      message : e.to_string(),
    } )?;

  let avail_set   : BTreeSet< _ > = available.iter().cloned().collect();
  let install_set : BTreeSet< _ > = installed.into_iter().collect();

  let mut entries = Vec::new();

  // Source-present entries: Synced or NotInstalled
  for name in &available
  {
    let status = if install_set.contains( name )
    {
      SyncStatus::Synced
    }
    else
    {
      SyncStatus::NotInstalled
    };
    entries.push( AssetEntry
    {
      agent  : "claude_code".into(),
      kind   : asset_kind,
      name   : name.clone(),
      status,
    } );
  }

  // Installed but source missing — Orphaned
  for name in &install_set
  {
    if !avail_set.contains( name )
    {
      entries.push( AssetEntry
      {
        agent  : "claude_code".into(),
        kind   : asset_kind,
        name   : name.clone(),
        status : SyncStatus::Orphaned,
      } );
    }
  }

  entries.sort_by( | a, b | a.name.cmp( &b.name ) );
  Ok( entries )
}

// ── Adapter ───────────────────────────────────────────────────────────────────

/// Claude Code adapter for [`Inventory`](crate::inventory::Inventory).
///
/// Reads from `$PRO_CLAUDE` (or `$PRO/genai/claude/`) as the source root and
/// uses the current working directory as the target root for `.claude/<kind>/`
/// symlink detection.
///
/// For testing, construct via [`with_paths()`](Self::with_paths) to inject
/// explicit source/target roots instead of relying on environment variables.
#[ derive( Debug ) ]
pub struct ClaudeCodeAdapter
{
  /// Explicit path override for tests; `None` resolves from the environment.
  paths_override : Option< AssetPaths >,
}

impl ClaudeCodeAdapter
{
  /// Create an adapter that resolves paths from the environment at call time.
  ///
  /// Calls `AssetPaths::from_env()` on every `list_*` invocation.
  #[ must_use ]
  #[ inline ]
  pub fn new() -> Self
  {
    Self { paths_override : None }
  }

  /// Create an adapter with explicit source/target roots (useful for tests).
  #[ must_use ]
  #[ inline ]
  pub fn with_paths( paths : AssetPaths ) -> Self
  {
    Self { paths_override : Some( paths ) }
  }

  fn resolve_paths( &self ) -> Result< AssetPaths, InventoryError >
  {
    match &self.paths_override
    {
      Some( p ) => Ok( p.clone() ),
      None      => claude_assets_core::paths::AssetPaths::from_env()
        .map_err( | e | InventoryError::EnvNotSet( e.to_string() ) ),
    }
  }
}

impl Default for ClaudeCodeAdapter
{
  #[ inline ]
  fn default() -> Self
  {
    Self::new()
  }
}

impl AgentAdapter for ClaudeCodeAdapter
{
  #[ inline ]
  fn name( &self ) -> &'static str
  {
    "claude_code"
  }

  #[ inline ]
  fn list_all( &self ) -> Result< Vec< AssetEntry >, InventoryError >
  {
    let paths = self.resolve_paths()?;
    let mut result = Vec::new();
    for &kind in ArtifactKind::all()
    {
      result.extend( list_for_kind( &paths, kind )? );
    }
    Ok( result )
  }

  #[ inline ]
  fn list_by_kind( &self, kind : AssetKind ) -> Result< Vec< AssetEntry >, InventoryError >
  {
    let paths = self.resolve_paths()?;
    list_for_kind( &paths, asset_to_artifact( kind ) )
  }
}

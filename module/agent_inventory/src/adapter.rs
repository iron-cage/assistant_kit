//! `AgentAdapter` trait for agent-agnostic asset discovery.

use crate::entry::{ AssetEntry, AssetKind };
use crate::error::InventoryError;

/// Adapter for a single AI coding agent.
///
/// Implementors expose the agent's assets as a flat `Vec<AssetEntry>`.
/// Each adapter should be feature-gated so that only the required agents
/// add compile-time and link-time cost to a consumer crate.
///
/// The trait is object-safe: `Box<dyn AgentAdapter>` is valid.
pub trait AgentAdapter : core::fmt::Debug
{
  /// Canonical agent name (e.g., `"claude_code"`).
  fn name( &self ) -> &'static str;

  /// List all assets this adapter can discover.
  ///
  /// # Errors
  ///
  /// Returns [`InventoryError::EnvNotSet`] when the environment is not configured,
  /// or [`InventoryError::Adapter`] when a directory read fails.
  fn list_all( &self ) -> Result< Vec< AssetEntry >, InventoryError >;

  /// List assets filtered to a specific kind.
  ///
  /// # Errors
  ///
  /// Same conditions as [`list_all()`](Self::list_all).
  fn list_by_kind( &self, kind : AssetKind ) -> Result< Vec< AssetEntry >, InventoryError >;
}

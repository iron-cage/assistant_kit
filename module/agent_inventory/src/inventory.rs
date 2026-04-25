//! `Inventory` registry — collects adapters and merges their asset results.

use crate::adapter::AgentAdapter;
use crate::entry::{ AssetEntry, AssetKind };
use crate::error::InventoryError;

/// Registry of `AgentAdapter` instances.
///
/// Register adapters with [`register()`](Self::register), then call
/// [`list_all()`](Self::list_all) or [`list_by_kind()`](Self::list_by_kind) to obtain
/// a merged flat table of assets from all registered agents.
#[ derive( Debug, Default ) ]
pub struct Inventory
{
  adapters : Vec< Box< dyn AgentAdapter > >,
}

impl Inventory
{
  /// Create an empty inventory with no registered adapters.
  #[ must_use ]
  #[ inline ]
  pub fn new() -> Self
  {
    Self { adapters : Vec::new() }
  }

  /// Register an adapter.
  ///
  /// Adapters are queried in registration order.
  #[ inline ]
  pub fn register( &mut self, adapter : Box< dyn AgentAdapter > )
  {
    self.adapters.push( adapter );
  }

  /// List all assets from every registered adapter merged into one flat table.
  ///
  /// Fails fast on the first adapter that returns an error.
  ///
  /// # Errors
  ///
  /// Propagates the first `InventoryError` returned by any adapter.
  #[ inline ]
  pub fn list_all( &self ) -> Result< Vec< AssetEntry >, InventoryError >
  {
    let mut result = Vec::new();
    for adapter in &self.adapters
    {
      result.extend( adapter.list_all()? );
    }
    Ok( result )
  }

  /// List assets of a specific kind from every registered adapter.
  ///
  /// Fails fast on the first adapter that returns an error.
  ///
  /// # Errors
  ///
  /// Propagates the first `InventoryError` returned by any adapter.
  #[ inline ]
  pub fn list_by_kind( &self, kind : AssetKind ) -> Result< Vec< AssetEntry >, InventoryError >
  {
    let mut result = Vec::new();
    for adapter in &self.adapters
    {
      result.extend( adapter.list_by_kind( kind )? );
    }
    Ok( result )
  }
}

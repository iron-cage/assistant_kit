//! Error type for agent inventory operations.

/// Error type for all `agent_inventory` operations.
#[ derive( Debug ) ]
pub enum InventoryError
{
  /// Required environment variable (`$PRO_CLAUDE` / `$PRO`) is not set.
  EnvNotSet( String ),
  /// An adapter failed while enumerating its assets.
  Adapter
  {
    /// Name of the adapter that failed (e.g., `"claude_code"`).
    adapter : String,
    /// Human-readable description of the failure.
    message : String,
  },
}

impl core::fmt::Display for InventoryError
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    match self
    {
      Self::EnvNotSet( msg ) =>
        write!( f, "environment not configured: {msg}" ),
      Self::Adapter { adapter, message } =>
        write!( f, "adapter '{adapter}' error: {message}" ),
    }
  }
}

impl core::error::Error for InventoryError {}

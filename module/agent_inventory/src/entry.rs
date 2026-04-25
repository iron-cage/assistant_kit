//! Asset entry: flat-table row and kind/status enumerations.

/// Kind of a named asset exposed by an AI coding agent.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum AssetKind
{
  /// Multi-step procedural skill packages.
  Skill,
  /// User-invocable slash commands.
  Command,
  /// Unconditional rules and coding guidelines.
  Rule,
  /// Specialised subagent definitions.
  Agent,
  /// Plugin directory bundles.
  Plugin,
  /// Event-triggered hook configurations.
  Hook,
}

impl AssetKind
{
  /// All supported asset kinds in display order.
  #[ must_use ]
  #[ inline ]
  pub fn all() -> &'static [ Self ]
  {
    &[ Self::Skill, Self::Command, Self::Rule, Self::Agent, Self::Plugin, Self::Hook ]
  }

  /// Canonical lowercase name for this kind (e.g., `"skill"`).
  #[ must_use ]
  #[ inline ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Skill   => "skill",
      Self::Command => "command",
      Self::Rule    => "rule",
      Self::Agent   => "agent",
      Self::Plugin  => "plugin",
      Self::Hook    => "hook",
    }
  }

  /// Parse a canonical kind name; returns `None` for unknown strings.
  ///
  /// Accepts the same names produced by [`as_str()`](Self::as_str).
  #[ must_use ]
  #[ inline ]
  pub fn from_name( s : &str ) -> Option< Self >
  {
    match s
    {
      "skill"   => Some( Self::Skill ),
      "command" => Some( Self::Command ),
      "rule"    => Some( Self::Rule ),
      "agent"   => Some( Self::Agent ),
      "plugin"  => Some( Self::Plugin ),
      "hook"    => Some( Self::Hook ),
      _         => None,
    }
  }
}

/// Symlink synchronisation state for a named asset.
///
/// Since assets are installed as live symlinks there is no version drift —
/// only presence states matter.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum SyncStatus
{
  /// Source exists and symlink is installed (always in sync — live pointer).
  Synced,
  /// Source exists but symlink is not yet installed.
  NotInstalled,
  /// Symlink is installed but the development source was deleted.
  Orphaned,
}

impl core::fmt::Display for SyncStatus
{
  #[ inline ]
  fn fmt( &self, f : &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    match self
    {
      Self::Synced       => write!( f, "✅ synced" ),
      Self::NotInstalled => write!( f, "⬇️ not installed" ),
      Self::Orphaned     => write!( f, "⚠️ orphaned" ),
    }
  }
}

/// Single flat-table row: one named asset from one agent.
#[ derive( Debug, Clone ) ]
pub struct AssetEntry
{
  /// Canonical name of the agent that owns this asset (e.g., `"claude_code"`).
  pub agent  : String,
  /// Kind of asset.
  pub kind   : AssetKind,
  /// Canonical name of the asset (e.g., `"commit"`).
  pub name   : String,
  /// Synchronisation status.
  pub status : SyncStatus,
}

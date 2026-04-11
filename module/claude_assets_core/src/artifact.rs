//! Artifact kind and layout classification.
//!
//! Defines all supported Claude Code artifact types with their canonical
//! source and target directory mappings.

/// Layout of an artifact in the filesystem.
///
/// `File` artifacts are single files with a known extension.
/// `Directory` artifacts are entire subdirectory trees.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum ArtifactLayout
{
  /// A single file (e.g., `.md`, `.yaml`).
  File,
  /// An entire directory tree.
  Directory,
}

/// A Claude Code artifact kind supported by the installer.
///
/// Each variant maps to a dedicated subdirectory in both the source
/// (`$PRO_CLAUDE/<subdir>/`) and target (`.claude/<subdir>/`) trees.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum ArtifactKind
{
  /// Unconditional rules and coding guidelines (`rules/*.md`).
  Rule,
  /// User-invocable slash commands (`commands/*.md`).
  Command,
  /// Specialised subagent definitions (`agents/*.md`).
  Agent,
  /// Multi-step procedural skill packages (`skills/<name>/`).
  Skill,
  /// Plugin directory bundles (`plugins/<name>/`).
  Plugin,
  /// Event-triggered hook configurations (`hooks/*.yaml`).
  Hook,
}

impl ArtifactKind
{
  /// All supported artifact kinds in display order.
  #[ must_use ]
  #[ inline ]
  pub fn all() -> &'static [ Self ]
  {
    &[ Self::Rule, Self::Command, Self::Agent, Self::Skill, Self::Plugin, Self::Hook ]
  }

  /// Canonical lowercase name used in CLI arguments (`kind::rule`).
  #[ must_use ]
  #[ inline ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Rule    => "rule",
      Self::Command => "command",
      Self::Agent   => "agent",
      Self::Skill   => "skill",
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
      "rule"    => Some( Self::Rule ),
      "command" => Some( Self::Command ),
      "agent"   => Some( Self::Agent ),
      "skill"   => Some( Self::Skill ),
      "plugin"  => Some( Self::Plugin ),
      "hook"    => Some( Self::Hook ),
      _         => None,
    }
  }

  /// Subdirectory name inside `$PRO_CLAUDE/` that holds this kind's sources.
  #[ must_use ]
  #[ inline ]
  pub fn source_subdir( self ) -> &'static str
  {
    match self
    {
      Self::Rule    => "rules",
      Self::Command => "commands",
      Self::Agent   => "agents",
      Self::Skill   => "skills",
      Self::Plugin  => "plugins",
      Self::Hook    => "hooks",
    }
  }

  /// Subdirectory name inside `.claude/` that receives installed symlinks.
  #[ must_use ]
  #[ inline ]
  pub fn target_subdir( self ) -> &'static str
  {
    match self
    {
      Self::Rule    => "rules",
      Self::Command => "commands",
      Self::Agent   => "agents",
      Self::Skill   => "skills",
      Self::Plugin  => "plugins",
      Self::Hook    => "hooks",
    }
  }

  /// How this artifact kind is stored in the filesystem.
  #[ must_use ]
  #[ inline ]
  pub fn layout( self ) -> ArtifactLayout
  {
    match self
    {
      Self::Rule | Self::Command | Self::Agent | Self::Hook => ArtifactLayout::File,
      Self::Skill | Self::Plugin                            => ArtifactLayout::Directory,
    }
  }

  /// File extension for `File`-layout kinds; `None` for `Directory`-layout kinds.
  #[ must_use ]
  #[ inline ]
  pub fn file_extension( self ) -> Option< &'static str >
  {
    match self
    {
      Self::Rule | Self::Command | Self::Agent => Some( "md" ),
      Self::Hook                               => Some( "yaml" ),
      Self::Skill | Self::Plugin               => None,
    }
  }
}

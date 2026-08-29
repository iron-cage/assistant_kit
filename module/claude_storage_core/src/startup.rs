//! What a session started in a directory would begin with.
//!
//! [`ContextFold`] answers "what does this session hold" by replaying a
//! transcript. That needs a session to have existed and written one. This module
//! answers the question one step earlier — *before* any session exists — by
//! reading the same directories Claude Code reads at startup.
//!
//! The two are not interchangeable and must not be confused for each other. A
//! fold reports what *was* loaded; a projection reports what *would be*. Where
//! they disagree, the fold is right: a session can defer a tool, be started with
//! `--disable-bundled-skills`, or have been running since before a skill was
//! installed.
//!
//! # What this cannot see
//!
//! Three gaps, named here because a silent one would read as an empty roster
//! rather than an unanswerable question:
//!
//! - **Bundled skills ship inside the `claude` binary.** They are not on disk in
//!   any directory, so no amount of filesystem reading enumerates them. A
//!   projection's skill list is therefore always a lower bound.
//! - **Nested `.claude/skills` directories** load when Claude Code is working on
//!   files near them, appearing under a directory-qualified `dir:name` on a
//!   clash. Finding them means scanning the whole tree, which this does not do.
//! - **Runtime state has no disk analogue.** Deferred tools, token usage,
//!   invoked skills, and background tasks are all products of a conversation.
//!   They are absent here because they cannot exist yet, not because they are
//!   empty.
//!
//! # On precedence
//!
//! This module deliberately decides none. Where a name appears in both user and
//! project scope, both are reported with their [`AssetScope`], and
//! [`StartupProjection::clashes`] names them — because which one wins is not
//! documented anywhere in the contract this crate is written against, and a
//! guess here would be indistinguishable from a fact.
//!
//! Settings precedence is a separate matter, and a settled one:
//! `claude_version_core`'s config resolution owns it. This module reports only
//! which settings files exist.
//!
//! [`ContextFold`]: crate::ContextFold

use std::path::{ Path, PathBuf };

/// Filename that marks a directory as a skill.
pub const SKILL_MARKER : &str = "SKILL.md";

/// Directory a project's Claude Code configuration lives in.
pub const PROJECT_CONFIG_DIR : &str = ".claude";

/// Whether an asset came from the user's home or from the project.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord ) ]
pub enum AssetScope
{
  /// Found under Claude Code's home directory — available in every project.
  User,
  /// Found under the project's `.claude/` — available only here.
  Project,
}

impl AssetScope
{
  /// The scope's name, as reported.
  #[ inline ]
  #[ must_use ]
  pub const fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::User => "user",
      Self::Project => "project",
    }
  }
}

/// One asset that would be available, and where it was found.
///
/// The path is kept alongside the name because a name alone cannot be checked.
/// Skill directories are frequently symlinks into a separate assets tree, so the
/// path a projection reports is the one inside `.claude/` — the link, which is
/// what Claude Code reads — not wherever it happens to resolve to.
#[ derive( Debug, Clone, PartialEq, Eq, PartialOrd, Ord ) ]
pub struct AssetOrigin
{
  /// Name Claude Code would refer to it by — the file stem, or the directory
  /// name for a skill.
  pub name : String,
  /// Where it was found.
  pub path : PathBuf,
  /// Which scope supplied it.
  pub scope : AssetScope,
}

/// What a session started in a directory would begin with.
///
/// Built by [`StartupProjection::resolve`]. Every roster is sorted by name and
/// then scope, so two projections of the same tree compare equal regardless of
/// the order the filesystem happened to hand entries back.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub struct StartupProjection
{
  /// Directory the projection was taken for.
  pub cwd : PathBuf,
  /// Claude Code's home directory, as resolved for this projection.
  pub claude_home : PathBuf,
  /// Nearest ancestor of `cwd` holding a `.claude/` directory.
  ///
  /// `None` when there is none — the projection is then user-scoped only, which
  /// is a real answer rather than a failure.
  pub project_root : Option< PathBuf >,
  /// Skills that would be on offer. A lower bound — see the module note on
  /// bundled skills.
  pub skills : Vec< AssetOrigin >,
  /// Agent types that would be available.
  pub agents : Vec< AssetOrigin >,
  /// Slash commands that would be available.
  pub commands : Vec< AssetOrigin >,
  /// Settings files that exist, project scope first.
  ///
  /// Existence only. Which one wins for a given key belongs to
  /// `claude_version_core`'s config resolution, not here.
  pub settings_files : Vec< PathBuf >,
  /// The project's `.mcp.json`, when it has one.
  pub mcp_config : Option< PathBuf >,
}

impl StartupProjection
{
  /// Project what a session started in `cwd` would begin with.
  ///
  /// Resolves Claude Code's home through [`crate::scope_for`], so `CLAUDE_HOME`
  /// is honoured with the same semantics as everywhere else in this crate.
  #[ inline ]
  #[ must_use ]
  pub fn resolve( cwd : &Path ) -> Self
  {
    let home = crate::scope_for( cwd ).claude_home;
    Self::resolve_in( &home, cwd )
  }

  /// Project against an explicit Claude Code home — the form tests use.
  ///
  /// Nothing here fails: an unreadable or absent directory contributes nothing
  /// and is not an error. A projection of a machine with no `~/.claude` at all is
  /// legitimately empty, and reporting that as an I/O failure would make the
  /// common case of a fresh install look broken.
  #[ inline ]
  #[ must_use ]
  pub fn resolve_in( claude_home : &Path, cwd : &Path ) -> Self
  {
    let project_root = project_root_for( cwd );
    let project_config = project_root.as_ref().map( | root | root.join( PROJECT_CONFIG_DIR ) );

    let mut skills = directory_assets( claude_home, project_config.as_deref(), "skills", AssetKind::SkillDir );
    let mut agents = directory_assets( claude_home, project_config.as_deref(), "agents", AssetKind::MarkdownFile );
    let mut commands = directory_assets( claude_home, project_config.as_deref(), "commands", AssetKind::MarkdownFile );

    // Sorted so a projection is a value rather than a record of directory
    // iteration order, which is not stable across filesystems.
    skills.sort();
    agents.sort();
    commands.sort();

    Self
    {
      cwd : cwd.to_path_buf(),
      claude_home : claude_home.to_path_buf(),
      settings_files : settings_files( claude_home, project_config.as_deref() ),
      mcp_config : project_root
        .as_ref()
        .map( | root | root.join( ".mcp.json" ) )
        .filter( | path | path.is_file() ),
      project_root,
      skills,
      agents,
      commands,
    }
  }

  /// Names offered by both scopes at once.
  ///
  /// Reported rather than resolved. Which scope wins a clash is not documented
  /// in the contract this crate is written against, so a caller that cares is
  /// told there is a question rather than handed an invented answer.
  #[ inline ]
  #[ must_use ]
  pub fn clashes( &self ) -> Vec< &str >
  {
    let mut names : Vec< &str > = [ &self.skills, &self.agents, &self.commands ]
      .into_iter()
      .flat_map( | roster | clashing_names( roster ) )
      .collect();

    names.sort_unstable();
    names.dedup();
    names
  }
}

/// How a roster's entries are laid out on disk.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
enum AssetKind
{
  /// A directory containing [`SKILL_MARKER`]; the directory name is the asset name.
  SkillDir,
  /// A `.md` file; its stem is the asset name.
  MarkdownFile,
}

/// The nearest ancestor of `cwd` — `cwd` included — holding a `.claude/` directory.
///
/// Walking up rather than checking `cwd` alone matches how project settings are
/// already resolved elsewhere in this workspace: a session started in a
/// subdirectory of a project is still in that project.
fn project_root_for( cwd : &Path ) -> Option< PathBuf >
{
  let mut current = cwd;
  loop
  {
    if current.join( PROJECT_CONFIG_DIR ).is_dir()
    {
      return Some( current.to_path_buf() );
    }
    current = current.parent()?;
  }
}

/// Read one named roster from both scopes.
fn directory_assets
(
  claude_home : &Path,
  project_config : Option< &Path >,
  roster : &str,
  kind : AssetKind,
)
-> Vec< AssetOrigin >
{
  let mut found = read_roster( &claude_home.join( roster ), kind, AssetScope::User );
  if let Some( config ) = project_config
  {
    found.extend( read_roster( &config.join( roster ), kind, AssetScope::Project ) );
  }
  found
}

/// Every asset in one directory, in one scope.
///
/// An absent or unreadable directory yields nothing. That is the ordinary case —
/// most projects define no agents of their own — and treating it as a failure
/// would make the empty answer unreachable.
fn read_roster( dir : &Path, kind : AssetKind, scope : AssetScope ) -> Vec< AssetOrigin >
{
  let Ok( entries ) = std::fs::read_dir( dir ) else { return Vec::new() };

  entries
    .filter_map( Result::ok )
    .filter_map( | entry | asset_from( &entry.path(), kind, scope ) )
    .collect()
}

/// Read one directory entry as an asset, or skip it.
fn asset_from( path : &Path, kind : AssetKind, scope : AssetScope ) -> Option< AssetOrigin >
{
  // `is_dir` and `is_file` follow symlinks, which is required rather than
  // incidental: skills are routinely installed as symlinks into a separate
  // assets tree, and a check that did not follow would report every one of them
  // as absent.
  let name = match kind
  {
    AssetKind::SkillDir =>
    {
      if !path.join( SKILL_MARKER ).is_file()
      {
        return None;
      }
      path.file_name()?.to_str()?.to_string()
    },
    AssetKind::MarkdownFile =>
    {
      if !path.is_file() || path.extension()? != "md"
      {
        return None;
      }
      path.file_stem()?.to_str()?.to_string()
    },
  };

  Some( AssetOrigin { name, path : path.to_path_buf(), scope } )
}

/// Settings files that exist, project scope first.
fn settings_files( claude_home : &Path, project_config : Option< &Path > ) -> Vec< PathBuf >
{
  let mut found = Vec::new();

  if let Some( config ) = project_config
  {
    found.push( config.join( "settings.local.json" ) );
    found.push( config.join( "settings.json" ) );
  }
  found.push( claude_home.join( "settings.json" ) );

  found.retain( | path | path.is_file() );
  found
}

/// Names appearing under more than one scope within one roster.
fn clashing_names( roster : &[ AssetOrigin ] ) -> Vec< &str >
{
  roster
    .iter()
    .filter( | one | roster.iter().any( | other | other.name == one.name && other.scope != one.scope ) )
    .map( | one | one.name.as_str() )
    .collect()
}

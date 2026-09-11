//! Unit tests for `StartupProjection` — what a session started in a directory
//! would begin with, read from disk.
//!
//! ## Purpose
//!
//! Verify that `StartupProjection::resolve_in()` reads skills, agents, commands,
//! settings files, and MCP config from the same directories Claude Code reads at
//! startup, correctly distinguishing user scope from project scope, following
//! symlinks, and reporting rather than resolving cross-scope name clashes.
//!
//! ## Test Strategy
//!
//! Every test builds its own `TempDir` tree for `claude_home` and, where a
//! project is involved, a second `TempDir` for `cwd`. `resolve_in()` takes both
//! explicitly, so no test needs to touch `HOME`/`CLAUDE_HOME` — the one exception
//! is `sp15_resolve_wraps_scope_for`, which exercises the public `resolve()`
//! wrapper and therefore isolates its env mutation behind `ENV_LOCK`.
//!
//! ## Related Requirements
//!
//! `docs/feature/007_startup_projection.md`.

use std::path::Path;
use tempfile::TempDir;
use claude_storage_core::{ StartupProjection, AssetScope, SKILL_MARKER, PROJECT_CONFIG_DIR };

/// Serializes the one test that mutates process-wide env vars.
static ENV_LOCK : std::sync::Mutex<()> = std::sync::Mutex::new( () );

fn mkdirp( path : &Path )
{
  std::fs::create_dir_all( path ).unwrap();
}

fn touch( path : &Path )
{
  mkdirp( path.parent().unwrap() );
  std::fs::write( path, "" ).unwrap();
}

fn skill( base : &Path, name : &str )
{
  let dir = base.join( "skills" ).join( name );
  mkdirp( &dir );
  std::fs::write( dir.join( SKILL_MARKER ), "# skill" ).unwrap();
}

// ============================================================================
// SP-1: a user-scope skill is found; no project scope is consulted
// ============================================================================

/// Test skill discovery under user scope alone.
///
/// ## Purpose
/// Verify a skill directory under `<claude_home>/skills/` is reported with
/// `AssetScope::User` when no project config is given.
///
/// ## Coverage
/// `directory_assets()` / `read_roster()` — user-scope-only call.
///
/// ## Validation Strategy
/// Build one skill under `claude_home`, resolve with `cwd` outside any project,
/// assert the roster has exactly that one entry, scoped `User`.
///
/// ## Related Requirements
/// SP-1 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp1_user_scope_skill_found()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  skill( home.path(), "reviewer" );

  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );

  assert_eq!( projection.skills.len(), 1, "expected exactly one skill" );
  assert_eq!( projection.skills[ 0 ].name, "reviewer" );
  assert_eq!( projection.skills[ 0 ].scope, AssetScope::User );
  assert_eq!( projection.skills[ 0 ].path, home.path().join( "skills" ).join( "reviewer" ) );
}

// ============================================================================
// SP-2: a project-scope skill is found; user scope contributes nothing
// ============================================================================

/// Test skill discovery under project scope alone.
///
/// ## Purpose
/// Verify a skill directory under `<project>/.claude/skills/` is reported with
/// `AssetScope::Project` when the user home has no skills of its own.
///
/// ## Coverage
/// `directory_assets()` project-scope branch.
///
/// ## Validation Strategy
/// Build one skill under `cwd/.claude/skills/`, an empty `claude_home`, assert
/// the roster has exactly that one entry, scoped `Project`.
///
/// ## Related Requirements
/// SP-2 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp2_project_scope_skill_found()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  skill( &cwd.path().join( PROJECT_CONFIG_DIR ), "packager" );

  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );

  assert_eq!( projection.skills.len(), 1, "expected exactly one skill" );
  assert_eq!( projection.skills[ 0 ].name, "packager" );
  assert_eq!( projection.skills[ 0 ].scope, AssetScope::Project );
}

// ============================================================================
// SP-3: the same skill name in both scopes is reported twice, and flagged
// ============================================================================

/// Test a skill name present in both scopes.
///
/// ## Purpose
/// Verify neither scope is dropped when a name clashes — both entries survive,
/// distinguished only by `scope` — and `clashes()` names it.
///
/// ## Coverage
/// `StartupProjection::clashes()`.
///
/// ## Validation Strategy
/// Create a skill named `shared` in both `claude_home` and the project, assert
/// the roster has two entries with that name (one per scope) and `clashes()`
/// returns exactly `[ "shared" ]`.
///
/// ## Related Requirements
/// SP-3 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp3_cross_scope_clash_is_reported_not_resolved()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  skill( home.path(), "shared" );
  skill( &cwd.path().join( PROJECT_CONFIG_DIR ), "shared" );

  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );

  let shared : Vec< _ > = projection.skills.iter().filter( | s | s.name == "shared" ).collect();
  assert_eq!( shared.len(), 2, "both scopes must survive a clash" );
  assert_eq!( projection.clashes(), vec![ "shared" ] );
}

// ============================================================================
// SP-4: a skill directory that is a symlink is followed, not skipped
// ============================================================================

/// Test symlinked skill directories are discovered.
///
/// ## Purpose
/// Verify a skill installed as a symlink — the routine installation shape — is
/// found, and the reported path is the link inside `.claude/`, not wherever it
/// resolves to.
///
/// ## Coverage
/// `asset_from()`'s `is_dir`/`is_file` symlink-following behaviour.
///
/// ## Validation Strategy
/// Build a real skill directory outside `claude_home`, symlink it in under
/// `skills/linked`, assert it is found named `linked` at the link's own path.
///
/// ## Related Requirements
/// SP-4 — `docs/feature/007_startup_projection.md`
#[ test ]
#[ cfg( unix ) ]
fn sp4_symlinked_skill_directory_is_followed()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  let real = TempDir::new().unwrap();

  let real_skill = real.path().join( "actual-skill" );
  mkdirp( &real_skill );
  std::fs::write( real_skill.join( SKILL_MARKER ), "# skill" ).unwrap();

  let skills_dir = home.path().join( "skills" );
  mkdirp( &skills_dir );
  std::os::unix::fs::symlink( &real_skill, skills_dir.join( "linked" ) ).unwrap();

  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );

  assert_eq!( projection.skills.len(), 1, "the symlinked skill must be found" );
  assert_eq!( projection.skills[ 0 ].name, "linked" );
  assert_eq!( projection.skills[ 0 ].path, skills_dir.join( "linked" ), "path must be the link, not its target" );
}

// ============================================================================
// SP-5: a directory with no SKILL.md is not a skill
// ============================================================================

/// Test a marker-less directory is excluded from the skill roster.
///
/// ## Purpose
/// Verify a bare directory under `skills/` — no `SKILL.md` — is not reported,
/// since the marker is what identifies a skill rather than mere presence.
///
/// ## Coverage
/// `asset_from()`'s `SkillDir` branch, marker-absent case.
///
/// ## Validation Strategy
/// Create `skills/not-a-skill/` with no `SKILL.md` inside; assert the roster
/// is empty.
///
/// ## Related Requirements
/// SP-5 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp5_directory_without_marker_is_not_a_skill()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  mkdirp( &home.path().join( "skills" ).join( "not-a-skill" ) );

  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );

  assert!( projection.skills.is_empty(), "a directory with no SKILL.md must not be a skill" );
}

// ============================================================================
// SP-6: agents are flat .md files, both scopes
// ============================================================================

/// Test agent discovery across both scopes.
///
/// ## Purpose
/// Verify `.md` files directly under `agents/` are reported by file stem, from
/// both user and project scope at once.
///
/// ## Coverage
/// `directory_assets()` with `AssetKind::MarkdownFile`.
///
/// ## Validation Strategy
/// Write one agent file in each scope with different names; assert both are
/// present with the right names and scopes.
///
/// ## Related Requirements
/// SP-6 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp6_agents_found_in_both_scopes()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  touch( &home.path().join( "agents" ).join( "reviewer.md" ) );
  touch( &cwd.path().join( PROJECT_CONFIG_DIR ).join( "agents" ).join( "deployer.md" ) );

  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );

  let mut names : Vec< _ > = projection.agents.iter().map( | a | a.name.as_str() ).collect();
  names.sort_unstable();
  assert_eq!( names, vec![ "deployer", "reviewer" ] );
  assert_eq!(
    projection.agents.iter().find( | a | a.name == "reviewer" ).unwrap().scope,
    AssetScope::User
  );
  assert_eq!(
    projection.agents.iter().find( | a | a.name == "deployer" ).unwrap().scope,
    AssetScope::Project
  );
}

// ============================================================================
// SP-7: commands ignore non-.md files
// ============================================================================

/// Test non-markdown files are excluded from the command roster.
///
/// ## Purpose
/// Verify a file under `commands/` without a `.md` extension — and a
/// subdirectory, which has no extension at all — are both ignored.
///
/// ## Coverage
/// `asset_from()`'s `MarkdownFile` branch, extension and `is_file` guards.
///
/// ## Validation Strategy
/// Write one real `.md` command alongside a `.txt` file and a bare
/// subdirectory; assert only the `.md` file is reported.
///
/// ## Related Requirements
/// SP-7 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp7_commands_ignore_non_markdown_entries()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  let commands = home.path().join( "commands" );
  touch( &commands.join( "deploy.md" ) );
  touch( &commands.join( "notes.txt" ) );
  mkdirp( &commands.join( "subdir" ) );

  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );

  assert_eq!( projection.commands.len(), 1, "only the .md file must be reported" );
  assert_eq!( projection.commands[ 0 ].name, "deploy" );
}

// ============================================================================
// SP-8: settings files are existence-only, project scope first
// ============================================================================

/// Test settings file listing order and existence filtering.
///
/// ## Purpose
/// Verify `settings_files` lists only files that exist, project
/// `settings.local.json` first, then project `settings.json`, then user
/// `settings.json` — without reading or interpreting any of their content.
///
/// ## Coverage
/// `settings_files()`.
///
/// ## Validation Strategy
/// Create all three files, assert the exact order; then remove the project
/// `settings.json` and assert it drops out while the other two keep their
/// relative order.
///
/// ## Related Requirements
/// SP-8 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp8_settings_files_existence_only_project_first()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  let project_config = cwd.path().join( PROJECT_CONFIG_DIR );
  touch( &project_config.join( "settings.local.json" ) );
  touch( &project_config.join( "settings.json" ) );
  touch( &home.path().join( "settings.json" ) );

  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );

  assert_eq!
  (
    projection.settings_files,
    vec!
    [
      project_config.join( "settings.local.json" ),
      project_config.join( "settings.json" ),
      home.path().join( "settings.json" ),
    ]
  );

  std::fs::remove_file( project_config.join( "settings.json" ) ).unwrap();
  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );
  assert_eq!
  (
    projection.settings_files,
    vec![ project_config.join( "settings.local.json" ), home.path().join( "settings.json" ) ],
    "an absent file must drop out without disturbing the others' order"
  );
}

// ============================================================================
// SP-9: mcp_config is project-scoped and existence-gated
// ============================================================================

/// Test `.mcp.json` detection.
///
/// ## Purpose
/// Verify `mcp_config` is `Some` only when a project root exists and its
/// `.mcp.json` is an actual file, and `None` in every other case.
///
/// ## Coverage
/// `StartupProjection::resolve_in()`'s `mcp_config` field.
///
/// ## Validation Strategy
/// Three sub-cases in one test: no project root at all; a project root with no
/// `.mcp.json`; a project root with `.mcp.json` present.
///
/// ## Related Requirements
/// SP-9 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp9_mcp_config_requires_project_root_and_file()
{
  let home = TempDir::new().unwrap();

  let no_project = TempDir::new().unwrap();
  assert_eq!( StartupProjection::resolve_in( home.path(), no_project.path() ).mcp_config, None );

  let bare_project = TempDir::new().unwrap();
  mkdirp( &bare_project.path().join( PROJECT_CONFIG_DIR ) );
  assert_eq!( StartupProjection::resolve_in( home.path(), bare_project.path() ).mcp_config, None );

  let mcp_project = TempDir::new().unwrap();
  mkdirp( &mcp_project.path().join( PROJECT_CONFIG_DIR ) );
  touch( &mcp_project.path().join( ".mcp.json" ) );
  assert_eq!
  (
    StartupProjection::resolve_in( home.path(), mcp_project.path() ).mcp_config,
    Some( mcp_project.path().join( ".mcp.json" ) )
  );
}

// ============================================================================
// SP-10: project_root walks up from a nested cwd
// ============================================================================

/// Test project root resolution from a subdirectory.
///
/// ## Purpose
/// Verify a session started several directories below the project root still
/// resolves to that root, matching how project settings are resolved
/// elsewhere in this workspace.
///
/// ## Coverage
/// `project_root_for()`, ancestor-walking branch.
///
/// ## Validation Strategy
/// Create `.claude/` at a root, a nested `src/inner` subdirectory with no
/// `.claude/` of its own, resolve from the subdirectory, assert `project_root`
/// is the outer root.
///
/// ## Related Requirements
/// SP-10 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp10_project_root_found_from_nested_cwd()
{
  let home = TempDir::new().unwrap();
  let root = TempDir::new().unwrap();
  mkdirp( &root.path().join( PROJECT_CONFIG_DIR ) );
  let nested = root.path().join( "src" ).join( "inner" );
  mkdirp( &nested );

  let projection = StartupProjection::resolve_in( home.path(), &nested );

  assert_eq!( projection.project_root, Some( root.path().to_path_buf() ) );
}

// ============================================================================
// SP-11: no ancestor has .claude/ — project_root is None, not a panic
// ============================================================================

/// Test the no-project case.
///
/// ## Purpose
/// Verify a `cwd` with no `.claude/` anywhere in its ancestry — walking all
/// the way to the filesystem root — yields `project_root: None` rather than
/// looping or panicking, and the projection is otherwise still user-scoped.
///
/// ## Coverage
/// `project_root_for()`'s termination on `Path::parent() == None`.
///
/// ## Validation Strategy
/// Resolve against a bare `TempDir` with nothing inside it; assert
/// `project_root` is `None` and every project-scoped field is empty/absent.
///
/// ## Related Requirements
/// SP-11 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp11_no_project_root_is_none_not_a_panic()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();

  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );

  assert_eq!( projection.project_root, None );
  assert_eq!( projection.mcp_config, None );
}

// ============================================================================
// SP-12: clashes() dedupes and sorts across all three rosters
// ============================================================================

/// Test `clashes()` across multiple rosters at once.
///
/// ## Purpose
/// Verify a clash in more than one roster is reported once each, sorted, and
/// a name appearing only once anywhere is never reported.
///
/// ## Coverage
/// `StartupProjection::clashes()`.
///
/// ## Validation Strategy
/// Clash `zeta` in agents, `alpha` in commands, and leave a `solo` command in
/// user scope only; assert `clashes()` is exactly `[ "alpha", "zeta" ]`.
///
/// ## Related Requirements
/// SP-12 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp12_clashes_dedupe_and_sort_across_rosters()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  let project_config = cwd.path().join( PROJECT_CONFIG_DIR );

  touch( &home.path().join( "agents" ).join( "zeta.md" ) );
  touch( &project_config.join( "agents" ).join( "zeta.md" ) );
  touch( &home.path().join( "commands" ).join( "alpha.md" ) );
  touch( &project_config.join( "commands" ).join( "alpha.md" ) );
  touch( &home.path().join( "commands" ).join( "solo.md" ) );

  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );

  assert_eq!( projection.clashes(), vec![ "alpha", "zeta" ] );
}

// ============================================================================
// SP-13: rosters are sorted regardless of directory iteration order
// ============================================================================

/// Test roster ordering is a value, not an artifact of directory iteration.
///
/// ## Purpose
/// Verify the skill roster comes back sorted by name so two projections of the
/// same tree compare equal, independent of the order the filesystem happens to
/// hand entries back.
///
/// ## Coverage
/// `StartupProjection::resolve_in()`'s post-read `sort()` calls.
///
/// ## Validation Strategy
/// Create skills in an order unlikely to match sorted order (`zebra`, `alpha`,
/// `mid`); assert the returned roster's names are alphabetical.
///
/// ## Related Requirements
/// SP-13 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp13_roster_is_sorted_by_name()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  skill( home.path(), "zebra" );
  skill( home.path(), "alpha" );
  skill( home.path(), "mid" );

  let projection = StartupProjection::resolve_in( home.path(), cwd.path() );

  let names : Vec< _ > = projection.skills.iter().map( | s | s.name.as_str() ).collect();
  assert_eq!( names, vec![ "alpha", "mid", "zebra" ] );
}

// ============================================================================
// SP-14: an absent claude_home is an empty projection, not an error
// ============================================================================

/// Test a fresh-install machine with no `claude_home` on disk at all.
///
/// ## Purpose
/// Verify pointing `resolve_in` at a `claude_home` path that does not exist
/// produces empty rosters and no settings files, never a panic or an error —
/// a fresh install is a real, ordinary answer.
///
/// ## Coverage
/// `read_roster()`'s absent-directory branch, exercised for every roster kind
/// at once via a wholly nonexistent home.
///
/// ## Validation Strategy
/// Resolve against `home.path().join("does-not-exist")`; assert every roster
/// and `settings_files` is empty.
///
/// ## Related Requirements
/// SP-14 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp14_absent_claude_home_is_empty_not_an_error()
{
  let home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  let missing_home = home.path().join( "does-not-exist" );

  let projection = StartupProjection::resolve_in( &missing_home, cwd.path() );

  assert!( projection.skills.is_empty() );
  assert!( projection.agents.is_empty() );
  assert!( projection.commands.is_empty() );
  assert!( projection.settings_files.is_empty() );
}

// ============================================================================
// SP-15: resolve() wraps scope_for() with CLAUDE_HOME semantics
// ============================================================================

/// Test the public `resolve()` entry point honours `CLAUDE_HOME`.
///
/// ## Purpose
/// Verify `StartupProjection::resolve()` derives `claude_home` through
/// `scope_for()` rather than assuming a fixed layout, so a `CLAUDE_HOME`
/// override reaches the projection the same way it reaches the rest of this
/// crate.
///
/// ## Coverage
/// `StartupProjection::resolve()`.
///
/// ## Validation Strategy
/// Set `CLAUDE_HOME` to a temp dir containing one skill, resolve with no
/// explicit home, assert that skill is found and `claude_home` matches the
/// override.
///
/// ## Related Requirements
/// SP-15 — `docs/feature/007_startup_projection.md`
#[ test ]
fn sp15_resolve_wraps_scope_for()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let claude_home = TempDir::new().unwrap();
  let cwd = TempDir::new().unwrap();
  skill( claude_home.path(), "from-env" );

  std::env::set_var( "CLAUDE_HOME", claude_home.path() );
  let projection = StartupProjection::resolve( cwd.path() );
  std::env::remove_var( "CLAUDE_HOME" );

  assert_eq!( projection.claude_home, claude_home.path() );
  assert_eq!( projection.skills.len(), 1 );
  assert_eq!( projection.skills[ 0 ].name, "from-env" );
}

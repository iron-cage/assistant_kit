//! Workspace structural invariant tests.
//!
//! ## Purpose
//!
//! Verify the structural requirements documented in the workspace-level doc
//! instances. All checks parse `Cargo.toml` files statically — no build
//! artefacts or network access required.
//!
//! ## Specification References
//!
//! - `docs/feature/001_workspace_design.md` — crate inventory (WD-1)
//! - `docs/invariant/001_privacy_invariant.md` — privacy constraints (PI-1, PI-2)
//! - `docs/invariant/002_versioning_strategy.md` — versioning policy (VS-1, VS-2)
//! - `docs/invariant/005_dependency_management.md` — dependency rules (DM-1, DM-2)
//! - `docs/pattern/001_crate_layering.md` — layer architecture (CL-1, CL-2)
//!
//! ## Test Matrix
//!
//! | Test | Spec | Scenario |
//! |------|------|----------|
//! | `wd1_workspace_members_completeness` | WD-1 | All 19 documented crates in members list |
//! | `pi1_no_private_path_deps` | PI-1 | No workspace path dep points outside the workspace |
//! | `pi2_out_of_workspace_path_deps_have_version` | PI-2 | Out-of-workspace path deps carry version field |
//! | `vs1_workspace_package_version_declared` | VS-1 | `[workspace.package]` declares a version |
//! | `vs2_crate_versions_declared` | VS-2 | Every crate Cargo.toml has a version field in `[package]` |
//! | `dm1_no_bare_version_declarations` | DM-1 | No bare `dep = "x.y.z"` in crate dep sections |
//! | `dm2_external_deps_use_workspace_true` | DM-2 | All crate dep entries include `workspace = true` |
//! | `cl1_no_same_layer_deps` | CL-1 | No crate deps on another crate in the same layer |
//! | `cl2_deps_flow_downward_only` | CL-2 | All workspace deps target a strictly lower layer |

use std::{
  collections::HashSet,
  fs,
  path::{ Path, PathBuf },
};

// ──────────────────────────────── constants ────────────────────────────────

const MANIFEST_DIR : &str = env!( "CARGO_MANIFEST_DIR" );

/// All 19 workspace member crate names.
const WORKSPACE_MEMBERS : &[ &str ] = &[
  "claude_storage_core",
  "claude_auth",
  "claude_quota",
  "claude_core",
  "claude_profile_core",
  "claude_version_core",
  "claude_runner_core",
  "claude_assets_core",
  "claude_profile",
  "claude_storage",
  "claude_runner",
  "dream",
  "claude_version",
  "claude_assets",
  "assistant",
  "assistant_kit",
  "runbox",
  "claude_journal",
  "claude_journal_viewer",
];

// ──────────────────────────────── helpers ─────────────────────────────────

/// Resolve the workspace root from the `assistant` crate's manifest directory.
///
/// `CARGO_MANIFEST_DIR` = `.../module/assistant`
/// `.parent()` = `.../module`
/// `.parent()` = `.../` (workspace root)
fn workspace_root() -> PathBuf
{
  Path::new( MANIFEST_DIR )
    .parent()
    // SAFETY: module/assistant always has a parent (module/) — path is never root.
    .unwrap()
    .parent()
    // SAFETY: module/ always has a parent (workspace root) — path is never root.
    .unwrap()
    .to_path_buf()
}

/// Read `Cargo.toml` for the named workspace member.
fn read_crate_manifest( root : &Path, crate_name : &str ) -> String
{
  let p = root.join( "module" ).join( crate_name ).join( "Cargo.toml" );
  fs::read_to_string( &p )
    .unwrap_or_else( | e | panic!( "cannot read {}: {e}", p.display() ) )
}

/// Returns `true` if `line` (trimmed) starts a TOML dependency section.
///
/// Matches `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, and
/// `[target.'cfg(...)'.dependencies]` variants.  Does NOT match workspace-level
/// `[workspace.dependencies]` sections (those use per-dep subsections).
fn is_dep_section_header( trimmed : &str ) -> bool
{
  trimmed == "[dependencies]"
    || trimmed == "[dev-dependencies]"
    || trimmed == "[build-dependencies]"
    || ( trimmed.starts_with( "[target." ) && trimmed.ends_with( ".dependencies]" ) )
}

/// Returns `true` if `line` (trimmed) is any TOML section header `[...]`.
fn is_any_section_header( trimmed : &str ) -> bool
{
  trimmed.starts_with( '[' )
}

/// Parse the workspace member names referenced in crate dep sections of `content`.
///
/// Only lines inside `[dependencies]`, `[dev-dependencies]`, or
/// `[build-dependencies]` sections are inspected.  A line matches when the
/// key (text before ` = `) is a known workspace member name.
fn workspace_deps_in( content : &str ) -> Vec< String >
{
  let member_set : HashSet< &str > = WORKSPACE_MEMBERS.iter().copied().collect();
  let mut in_dep = false;
  let mut deps = Vec::new();

  for line in content.lines()
  {
    let t = line.trim();
    if is_any_section_header( t )
    {
      in_dep = is_dep_section_header( t );
      continue;
    }
    if !in_dep || t.is_empty() || t.starts_with( '#' ) { continue; }
    // Extract identifier before " = "
    if let Some( eq ) = t.find( " = " )
    {
      let key = t[ ..eq ].trim().replace( '-', "_" );
      if member_set.contains( key.as_str() )
      {
        deps.push( key );
      }
    }
  }
  deps
}

/// Return the numeric layer (0–3) for a workspace member, or `None` for Layer * crates.
///
/// Layer * crates (`claude_storage_core`, `claude_auth`, `claude_quota`, `runbox`)
/// are excluded from cross-layer dependency checks (CL-1, CL-2).
fn layer_of( name : &str ) -> Option< u8 >
{
  match name
  {
    "claude_core" => Some( 0 ),
    "claude_assets_core"
    | "claude_profile_core"
    | "claude_version_core"
    | "claude_runner_core"
    | "claude_journal" => Some( 1 ),
    "dream"
    | "claude_assets"
    | "claude_version"
    | "claude_runner"
    | "claude_profile"
    | "claude_storage"
    | "claude_journal_viewer" => Some( 2 ),
    "assistant" | "assistant_kit" => Some( 3 ),
    // Layer * — no numeric layer; exempt from CL checks
    _ => None,
  }
}

// ──────────────────────── feature: workspace design ───────────────────────

/// WD-1: All 19 documented workspace members are present in `[workspace.members]`.
///
/// ## Root Cause (why this test exists)
/// Workspace membership controls which crates are built and tested in CI.
/// A crate added to `module/` but missing from `members` is silently excluded.
///
/// ## Prevention
/// Any new crate under `module/` must also appear in `Cargo.toml` members and
/// in the `WORKSPACE_MEMBERS` constant in this file.
#[test]
fn wd1_workspace_members_completeness()
{
  let root = workspace_root();
  let cargo = fs::read_to_string( root.join( "Cargo.toml" ) )
    .expect( "cannot read workspace Cargo.toml" );

  let mut in_members = false;
  let mut found : HashSet< String > = HashSet::new();

  for line in cargo.lines()
  {
    let t = line.trim();
    if t == "members = [" { in_members = true; continue; }
    if in_members
    {
      if t == "]" { break; }
      // Lines look like:  `"module/crate_name",`
      if let Some( inner ) = t.strip_prefix( '"' )
      {
        if let Some( name_part ) = inner.split( '"' ).next()
        {
          if let Some( crate_name ) = name_part.strip_prefix( "module/" )
          {
            found.insert( crate_name.to_owned() );
          }
        }
      }
    }
  }

  let mut missing = Vec::new();
  for &member in WORKSPACE_MEMBERS
  {
    if !found.contains( member )
    {
      missing.push( member );
    }
  }
  assert!(
    missing.is_empty(),
    "WD-1: {} documented crate(s) absent from [workspace.members]: {:?}",
    missing.len(),
    missing,
  );
  assert_eq!(
    found.len(),
    WORKSPACE_MEMBERS.len(),
    "WD-1: workspace has {} members but WORKSPACE_MEMBERS constant lists {}; \
     update WORKSPACE_MEMBERS to match",
    found.len(),
    WORKSPACE_MEMBERS.len(),
  );
}

// ─────────────────────── invariant: privacy constraints ───────────────────

/// PI-1: No workspace path dependency points outside the workspace root.
///
/// ## Root Cause (why this test exists)
/// The workspace privacy invariant forbids encoding any knowledge of consumer
/// workspace paths. A path dep escaping the workspace root (e.g. `path = "../.."`)
/// would couple this workspace to the consumer layout.
///
/// ## Prevention
/// All workspace member path deps must use relative paths that stay within the
/// workspace root (i.e., start with `module/`).
#[test]
fn pi1_no_private_path_deps()
{
  let root = workspace_root();
  let cargo = fs::read_to_string( root.join( "Cargo.toml" ) )
    .expect( "cannot read workspace Cargo.toml" );

  let mut violations = Vec::new();
  // Each workspace dep lives in its own [workspace.dependencies.<name>] section.
  // The path line within that section looks like: `path = "module/..."`.
  let mut current_dep = String::new();
  for line in cargo.lines()
  {
    let t = line.trim();
    if t.starts_with( "[workspace.dependencies." ) && t.ends_with( ']' )
    {
      // e.g. [workspace.dependencies.claude_auth]
      current_dep = t
        .trim_start_matches( "[workspace.dependencies." )
        .trim_end_matches( ']' )
        .to_owned();
      continue;
    }
    if t.starts_with( "path = \"" )
    {
      let path_val = t
        .trim_start_matches( "path = \"" )
        .trim_end_matches( '"' );
      // Paths must stay within the workspace — never traverse upward
      if path_val.contains( "../" ) || path_val.starts_with( '/' )
      {
        violations.push( format!( "{current_dep}: path = \"{path_val}\"" ) );
      }
    }
  }
  assert!(
    violations.is_empty(),
    "PI-1: {} workspace dep(s) point outside the workspace root: {:?}",
    violations.len(),
    violations,
  );
}

/// PI-2: Every out-of-workspace path dependency also declares a `version` field.
///
/// ## Root Cause (why this test exists)
/// A path-only dependency (no version) cannot be published; Cargo rejects it.
/// If a companion crate were added as an out-of-workspace path dep without a
/// version field, publishing would fail silently until the release step.
///
/// ## Prevention
/// Out-of-workspace path deps (path containing `../`) must always include
/// `version = "..."` in the same `[workspace.dependencies.*]` block.
///
/// *Currently vacuously true — no out-of-workspace path deps exist.*
#[test]
fn pi2_out_of_workspace_path_deps_have_version()
{
  let root = workspace_root();
  let cargo = fs::read_to_string( root.join( "Cargo.toml" ) )
    .expect( "cannot read workspace Cargo.toml" );

  // Collect (dep_name, path_value, has_version) for each workspace dep section.
  let mut current_dep = String::new();
  let mut current_path : Option< String > = None;
  let mut current_has_version = false;
  let mut violations = Vec::new();

  let lines : Vec< &str > = cargo.lines().collect();
  for ( idx, line ) in lines.iter().enumerate()
  {
    let t = line.trim();
    let is_new_section = t.starts_with( '[' );

    // Flush previous section before starting a new one
    if is_new_section && !current_dep.is_empty()
    {
      if let Some( ref path ) = current_path
      {
        if ( path.contains( "../" ) || path.starts_with( '/' ) )
          && !current_has_version
        {
          violations.push( format!(
            "{current_dep}: path=\"{path}\" has no version field",
          ) );
        }
      }
      current_dep.clear();
      current_path = None;
      current_has_version = false;
    }

    if t.starts_with( "[workspace.dependencies." ) && t.ends_with( ']' )
    {
      current_dep = t
        .trim_start_matches( "[workspace.dependencies." )
        .trim_end_matches( ']' )
        .to_owned();
    }
    else if !current_dep.is_empty()
    {
      if t.starts_with( "path = \"" )
      {
        current_path = Some(
          t.trim_start_matches( "path = \"" )
            .trim_end_matches( '"' )
            .to_owned(),
        );
      }
      else if t.starts_with( "version = " )
      {
        current_has_version = true;
      }
    }

    // Flush at EOF
    if idx + 1 == lines.len() && !current_dep.is_empty()
    {
      if let Some( ref path ) = current_path
      {
        if ( path.contains( "../" ) || path.starts_with( '/' ) )
          && !current_has_version
        {
          violations.push( format!(
            "{current_dep}: path=\"{path}\" has no version field",
          ) );
        }
      }
    }
  }
  assert!(
    violations.is_empty(),
    "PI-2: {} out-of-workspace path dep(s) lack a version field: {:?}",
    violations.len(),
    violations,
  );
}

// ───────────────────── invariant: versioning strategy ─────────────────────

/// VS-1: The workspace `[workspace.package]` section declares a `version` field.
///
/// ## Root Cause (why this test exists)
/// A shared workspace version enables coordinated releases. Without it, each
/// crate must declare its version independently with no common baseline.
///
/// ## Prevention
/// Never remove the `version` line from `[workspace.package]`.
#[test]
fn vs1_workspace_package_version_declared()
{
  let root = workspace_root();
  let cargo = fs::read_to_string( root.join( "Cargo.toml" ) )
    .expect( "cannot read workspace Cargo.toml" );

  let mut in_ws_pkg = false;
  let mut version_found = false;

  for line in cargo.lines()
  {
    let t = line.trim();
    if t == "[workspace.package]"
    {
      in_ws_pkg = true;
      continue;
    }
    if in_ws_pkg
    {
      if t.starts_with( '[' ) { break; } // left the section
      if t.starts_with( "version = " )
      {
        version_found = true;
        break;
      }
    }
  }
  assert!(
    version_found,
    "VS-1: [workspace.package] does not contain a `version` field",
  );
}

/// VS-2: Every crate `Cargo.toml` has a version field in `[package]`.
///
/// ## Root Cause (why this test exists)
/// An undeclared version makes a crate unpublishable and breaks release
/// tooling that reads `CARGO_PKG_VERSION` at build time.
///
/// ## Prevention
/// Every crate under `module/` must declare `version = "x.y.z"` or
/// `version.workspace = true` in its `[package]` section.
///
/// *Note: the workspace currently uses explicit per-crate versions throughout.*
#[test]
fn vs2_crate_versions_declared()
{
  let root = workspace_root();
  let mut missing = Vec::new();

  for &name in WORKSPACE_MEMBERS
  {
    let content = read_crate_manifest( &root, name );
    let mut in_pkg = false;
    let mut found = false;

    for line in content.lines()
    {
      let t = line.trim();
      if t == "[package]" { in_pkg = true; continue; }
      if in_pkg
      {
        if t.starts_with( '[' ) { break; } // left [package]
        // Accept `version = "x.y.z"` or `version.workspace = true`
        if t.starts_with( "version" )
        {
          found = true;
          break;
        }
      }
    }
    if !found { missing.push( name ); }
  }
  assert!(
    missing.is_empty(),
    "VS-2: {} crate(s) lack a version field in [package]: {:?}",
    missing.len(),
    missing,
  );
}

// ──────────────────── invariant: dependency management ────────────────────

/// DM-1: No crate-level `Cargo.toml` contains a bare `dep = "x.y.z"` declaration.
///
/// ## Root Cause (why this test exists)
/// Bare string version declarations bypass the workspace dependency table,
/// silently creating a second independently-updated version pin. When the
/// workspace version is updated, the bare pin is left stale.
///
/// ## Prevention
/// All external deps must be declared in `[workspace.dependencies]` and
/// referenced via `{ workspace = true }` in crate files.
#[test]
fn dm1_no_bare_version_declarations()
{
  let root = workspace_root();
  let mut violations = Vec::new();

  for &name in WORKSPACE_MEMBERS
  {
    let content = read_crate_manifest( &root, name );
    let mut in_dep = false;

    for line in content.lines()
    {
      let t = line.trim();
      if is_any_section_header( t )
      {
        in_dep = is_dep_section_header( t );
        continue;
      }
      if !in_dep || t.is_empty() || t.starts_with( '#' ) { continue; }

      // A bare declaration looks like:  `dep_name = "x.y.z"` or `dep_name = "^x"`
      // (string literal as the value, not an inline table `{ ... }`)
      if let Some( eq ) = t.find( " = " )
      {
        let value = t[ eq + 3.. ].trim();
        if value.starts_with( '"' )
        {
          violations.push( format!( "{name}: {t}" ) );
        }
      }
    }
  }
  assert!(
    violations.is_empty(),
    "DM-1: {} bare version declaration(s) found in crate dep sections:\n  {}",
    violations.len(),
    violations.join( "\n  " ),
  );
}

/// DM-2: Every dep entry in crate dependency sections includes `workspace = true`.
///
/// ## Root Cause (why this test exists)
/// A dep entry without `workspace = true` uses a crate-local version pin that
/// diverges from the workspace table, causing version skew and potential
/// compilation failures when the workspace pin is updated.
///
/// ## Prevention
/// All entries in `[dependencies]`, `[dev-dependencies]`, and
/// `[build-dependencies]` must use `{ workspace = true, ... }`.
#[test]
fn dm2_external_deps_use_workspace_true()
{
  let root = workspace_root();
  let mut violations = Vec::new();

  for &name in WORKSPACE_MEMBERS
  {
    let content = read_crate_manifest( &root, name );
    let mut in_dep = false;

    for line in content.lines()
    {
      let t = line.trim();
      if is_any_section_header( t )
      {
        in_dep = is_dep_section_header( t );
        continue;
      }
      if !in_dep || t.is_empty() || t.starts_with( '#' ) { continue; }

      // A dep line must contain `workspace = true`
      if t.contains( " = " ) && !t.contains( "workspace = true" )
      {
        violations.push( format!( "{name}: {t}" ) );
      }
    }
  }
  assert!(
    violations.is_empty(),
    "DM-2: {} dep entry(ies) missing `workspace = true`:\n  {}",
    violations.len(),
    violations.join( "\n  " ),
  );
}

// ──────────────────────── pattern: crate layering ─────────────────────────

/// Known same-layer dep exceptions: `(source_crate, dep_crate)`.
///
/// `claude_profile_core` (L1) has an optional dep on `claude_runner_core` (L1)
/// to spawn the Claude CLI for token refresh.  This intentional cross-L1 optional
/// coupling is documented in `docs/pattern/001_crate_layering.md`.  It does not
/// create a build cycle because it is `optional = true` and only activates when
/// the `enabled` feature is requested.
const ALLOWED_SAME_LAYER_DEPS : &[ ( &str, &str ) ] = &[
  ( "claude_profile_core", "claude_runner_core" ),
];

/// CL-1: No crate depends on another crate assigned to the same layer.
///       Layer * crates (`claude_storage_core`, `claude_auth`, `claude_quota`,
///       `runbox`) are verified to have zero workspace dependencies.
///
/// ## Root Cause (why this test exists)
/// Same-layer dependencies signal confused ownership and make the build graph
/// cyclic or fragile. Layer * crates are utility singletons; coupling them to
/// other workspace members would break the isolation guarantee.
///
/// ## Prevention
/// When adding a dep, ensure the target crate is in a strictly lower (or Layer *)
/// layer than the depending crate.  Intentional optional exceptions must be
/// added to `ALLOWED_SAME_LAYER_DEPS` with a justification comment.
#[test]
fn cl1_no_same_layer_deps()
{
  let root = workspace_root();
  let mut violations = Vec::new();

  for &name in WORKSPACE_MEMBERS
  {
    let crate_layer = layer_of( name );
    let content = read_crate_manifest( &root, name );
    let deps = workspace_deps_in( &content );

    for dep in &deps
    {
      // Skip documented intentional exceptions
      if ALLOWED_SAME_LAYER_DEPS.contains( &( name, dep.as_str() ) ) { continue; }

      let dep_layer = layer_of( dep );
      match ( crate_layer, dep_layer )
      {
        // Layer * crate must have NO workspace deps
        ( None, _ ) =>
        {
          violations.push( format!(
            "{name} (Layer *) depends on workspace member {dep}",
          ) );
        }
        // Layer N crate must not dep on same Layer N
        ( Some( c ), Some( d ) ) if c == d =>
        {
          violations.push( format!(
            "{name} (Layer {c}) depends on same-layer crate {dep} (Layer {d})",
          ) );
        }
        // Layer N → Layer * or Layer N → Layer M≠N — ok
        _ => {}
      }
    }
  }
  assert!(
    violations.is_empty(),
    "CL-1: {} same-layer or Layer-* dep violation(s):\n  {}",
    violations.len(),
    violations.join( "\n  " ),
  );
}

/// CL-2: All workspace dependencies target a strictly lower layer.
///       Layer * crates are excluded (they have no workspace deps per CL-1).
///
/// ## Root Cause (why this test exists)
/// An upward dependency (e.g. Layer 1 → Layer 2) creates a cyclic dependency
/// in the build graph and breaks the compilation order. The four-layer
/// architecture is only effective when the flow is enforced automatically.
///
/// ## Prevention
/// All deps within the workspace must target a lower (not equal, not higher)
/// numeric layer. Deps to Layer * crates are exempt from this ordering check.
/// Documented same-layer exceptions (see `ALLOWED_SAME_LAYER_DEPS`) are also
/// excluded to keep CL-1 and CL-2 consistent.
#[test]
fn cl2_deps_flow_downward_only()
{
  let root = workspace_root();
  let mut violations = Vec::new();

  for &name in WORKSPACE_MEMBERS
  {
    let Some( crate_layer ) = layer_of( name ) else { continue }; // Layer * excluded per spec
    let content = read_crate_manifest( &root, name );
    let deps = workspace_deps_in( &content );

    for dep in &deps
    {
      // Skip documented intentional exceptions (aligned with CL-1 allowlist)
      if ALLOWED_SAME_LAYER_DEPS.contains( &( name, dep.as_str() ) ) { continue; }

      match layer_of( dep )
      {
        None => {} // Layer * dep — exempt from ordering check
        Some( dep_layer ) if dep_layer < crate_layer => {} // downward — ok
        Some( dep_layer ) =>
        {
          violations.push( format!(
            "{name} (Layer {crate_layer}) → {dep} (Layer {dep_layer}): \
             not downward",
          ) );
        }
      }
    }
  }
  assert!(
    violations.is_empty(),
    "CL-2: {} upward or same-layer dep flow violation(s):\n  {}",
    violations.len(),
    violations.join( "\n  " ),
  );
}

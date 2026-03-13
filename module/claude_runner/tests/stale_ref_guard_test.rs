//! Guard tests: verify no stale references remain in permanent files.
//!
//! ## Context
//!
//! Two sets of stale references are guarded:
//!
//! **1. `claude_runner_plugin`** — removed from willbe/dev workspace on 2026-03-09.
//! YAML consumers now aggregate `claude.commands.yaml` directly via `build.rs`.
//!
//! **2. `dream_agent`** — removed as a coupling concern on 2026-03-26.
//! `claude_runner` is now a standalone CLI. Session continuation is its own responsibility.
//! Any reference framing it as a `dream_agent` subprocess target is a stale coupling.
//!
//! ## Tests
//!
//! Plugin references:
//! - `no_plugin_ref_in_docs`: all `docs/` `.md` files must have zero `claude_runner_plugin`
//!   matches (Note lines about proset preservation are exempt — intentional historical context)
//! - `no_plugin_ref_in_lib_rs`: `src/lib.rs` must have zero `claude_runner_plugin` matches
//! - `no_plugin_ref_in_readme`: `readme.md` must have zero `claude_runner_plugin` matches
//!
//! `dream_agent` references:
//! - `no_dream_agent_ref_in_docs`: all `docs/` `.md` files must have zero `dream_agent`
//!   matches, except `design_decisions.md` which documents the historical decoupling
//! - `no_dream_agent_ref_in_lib_rs`: `src/lib.rs` must have zero `dream_agent` matches
//! - `no_dream_agent_ref_in_readme`: `readme.md` must have zero `dream_agent` matches
//!
//! Structure:
//! - `src_readme_exists`: `src/readme.md` must exist (3+ files require Responsibility Table)
//!
//! ## Pitfalls
//!
//! **`spec.md` was migrated to `docs/`.** The guards that previously checked `spec.md` now
//! scan all `.md` files under `docs/`. `spec.md` no longer exists in this crate.
//!
//! **`design_decisions.md` is exempt from `dream_agent` guard.** That file documents *why*
//! the decoupling happened — mentioning `dream_agent` there is intentional historical context,
//! not a stale coupling. Exempt pattern: skip files named `design_decisions.md`.
//!
//! **Proset ≠ deleted.** `claude_runner_plugin` still exists at `willbe/proset/module/`
//! as a reference implementation. It was only *removed from willbe/dev workspace*. Guard
//! tests that check for the plugin name must exempt deliberate Note lines documenting this
//! preservation — those are accurate history, not stale refs. Exempt pattern:
//! `!line.contains("removed from willbe/dev workspace")`.
//!
//! **MSRV is 1.70.** `.is_some_and()` is available and preferred over
//! `.map_or( false, |x| ... )`. Use `is_some_and` in this crate.
//!
//! **`display()` is not `Copy`.** `Path::display()` returns a `Display` wrapper that
//! borrows the path. Because it is not `Copy`, rustfmt/clippy cannot inline it into
//! format-string variable slots. The `uninlined_format_args` lint will fire on the
//! *other* arguments in the same call. Keep the explicit `{}` placeholder and add a
//! `// display() not Copy, can't inline` comment to explain why the lint is suppressed.
//!
//! **`doc_markdown` lint.** Bare crate/binary names in doc comments (e.g. `claude_runner`)
//! must be wrapped in backticks or the `doc_markdown` clippy lint fires with `-D warnings`.

use std::fs;
use std::path::{ Path, PathBuf };

fn collect_violations( file_path : &Path, pattern : &str ) -> Vec< String >
{
  let content = fs::read_to_string( file_path )
    .unwrap_or_else( |e| panic!( "Cannot read {}: {e}", file_path.display() ) );
  content
    .lines()
    .enumerate()
    .filter( |( _, line )| line.contains( pattern ) )
    .map( |( i, line )| format!( "  {}:{}: {}", file_path.display(), i + 1, line.trim() ) ) // display() not Copy, can't inline
    .collect()
}

/// Collect all `.md` files under `dir`, optionally skipping files whose name
/// (without extension) matches any entry in `skip_names`.
fn md_files_in_dir( dir : &Path, skip_names : &[ &str ] ) -> Vec< PathBuf >
{
  let mut result = Vec::new();
  let entries = fs::read_dir( dir )
    .unwrap_or_else( |e| panic!( "Cannot read dir {}: {e}", dir.display() ) );
  for entry in entries.flatten()
  {
    let path = entry.path();
    if path.is_dir()
    {
      result.extend( md_files_in_dir( &path, skip_names ) );
    }
    else if path.extension().is_some_and( |ext| ext == "md" )
    {
      let stem = path.file_stem().and_then( |s| s.to_str() ).unwrap_or( "" );
      if !skip_names.contains( &stem )
      {
        result.push( path );
      }
    }
  }
  result
}

#[ test ]

fn no_plugin_ref_in_docs()
{
  // Note lines about proset preservation are exempt: they document historical removal, not stale refs.
  // Exempt pattern: lines containing "removed from willbe/dev workspace"
  // spec.md was migrated to docs/ — scan all .md files under docs/ instead.
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let docs_dir = manifest.join( "docs" );
  let files = md_files_in_dir( &docs_dir, &[] );
  let violations : Vec< String > = files
    .iter()
    .flat_map( |f| collect_violations( f, "claude_runner_plugin" ) )
    .filter( |line| !line.contains( "removed from willbe/dev workspace" ) )
    .collect();
  assert!(
    violations.is_empty(),
    "Stale `claude_runner_plugin` references in docs/:\n{}",
    violations.join( "\n" )
  );
}

#[ test ]

fn no_plugin_ref_in_lib_rs()
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let path = manifest.join( "src/lib.rs" );
  let violations = collect_violations( &path, "claude_runner_plugin" );
  assert!(
    violations.is_empty(),
    "Stale claude_runner_plugin references in src/lib.rs:\n{}",
    violations.join( "\n" )
  );
}

#[ test ]

fn no_plugin_ref_in_readme()
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let path = manifest.join( "readme.md" );
  let violations = collect_violations( &path, "claude_runner_plugin" );
  assert!(
    violations.is_empty(),
    "Stale claude_runner_plugin references in readme.md:\n{}",
    violations.join( "\n" )
  );
}

#[ test ]

fn src_readme_exists()
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let readme = manifest.join( "src" ).join( "readme.md" );
  assert!(
    readme.exists(),
    "src/readme.md must exist (3+ files require Responsibility Table): {}",
    readme.display()
  );
}

#[ test ]

fn no_dream_agent_ref_in_docs()
{
  // design_decisions.md is exempt: it documents *why* the decoupling happened —
  // mentioning dream_agent there is intentional historical context, not a stale coupling.
  // spec.md was migrated to docs/ — scan all .md files under docs/ instead.
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let docs_dir = manifest.join( "docs" );
  let files = md_files_in_dir( &docs_dir, &[ "design_decisions" ] );
  let violations : Vec< String > = files
    .iter()
    .flat_map( |f| collect_violations( f, "dream_agent" ) )
    .collect();
  assert!(
    violations.is_empty(),
    "Stale `dream_agent` references in docs/ (`claude_runner` is standalone):\n{}",
    violations.join( "\n" )
  );
}

#[ test ]

fn no_dream_agent_ref_in_lib_rs()
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let path = manifest.join( "src/lib.rs" );
  let violations = collect_violations( &path, "dream_agent" );
  assert!(
    violations.is_empty(),
    "Stale dream_agent references in src/lib.rs:\n{}",
    violations.join( "\n" )
  );
}

#[ test ]

fn no_dream_agent_ref_in_readme()
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let path = manifest.join( "readme.md" );
  let violations = collect_violations( &path, "dream_agent" );
  assert!(
    violations.is_empty(),
    "Stale dream_agent references in readme.md:\n{}",
    violations.join( "\n" )
  );
}

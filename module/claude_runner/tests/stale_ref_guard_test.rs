//! Guard tests: verify no stale references remain in permanent files.
//!
//! ## Context
//!
//! Two sets of stale references are guarded:
//!
//! **1. `claude_runner_plugin`** — removed from the consumer workspace on 2026-03-09.
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
//! - `no_routines_rs_in_src`: `src/routines.rs` must not exist (dep constraint IT-2)
//! - `no_build_rs_at_crate_root`: `build.rs` must not exist at crate root (dep constraint IT-3)
//! - `all_cargo_dependencies_are_optional`: all `[dependencies]` entries have `optional = true` (dep constraint IT-4)
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
//! **Proset ≠ deleted.** `claude_runner_plugin` still exists at `consumer/proset/module/`
//! as a reference implementation. It was only *removed from the consumer workspace*. Guard
//! tests that check for the plugin name must exempt deliberate Note lines documenting this
//! preservation — those are accurate history, not stale refs. Exempt pattern:
//! `!line.contains("removed from the consumer workspace")`.
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
  // Exempt pattern: lines containing "removed from the consumer workspace"
  // spec.md was migrated to docs/ — scan all .md files under docs/ instead.
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let docs_dir = manifest.join( "docs" );
  let files = md_files_in_dir( &docs_dir, &[] );
  let violations : Vec< String > = files
    .iter()
    .flat_map( |f| collect_violations( f, "claude_runner_plugin" ) )
    .filter( |line| !line.contains( "removed from the consumer workspace" ) )
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

/// `src/routines.rs` must not exist.
///
/// This crate does not define an internal `routines` module. If this file appears,
/// it signals a structural regression (dependency on an abstraction layer that was
/// never part of this crate's design).
///
/// Spec: `tests/docs/invariant/002_dep_constraints.md` IT-2
#[ test ]
fn no_routines_rs_in_src()
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let path = manifest.join( "src/routines.rs" );
  assert!(
    !path.exists(),
    "src/routines.rs must not exist (dep constraint IT-2): {}",
    path.display()
  );
}

/// `build.rs` must not exist at crate root.
///
/// `claude_runner` has no build-time code generation. A `build.rs` would introduce
/// an undocumented build-time dependency and is not permitted by the dep constraints.
///
/// Spec: `tests/docs/invariant/002_dep_constraints.md` IT-3
#[ test ]
fn no_build_rs_at_crate_root()
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let path = manifest.join( "build.rs" );
  assert!(
    !path.exists(),
    "build.rs must not exist at crate root (dep constraint IT-3): {}",
    path.display()
  );
}

/// All `[dependencies]` entries in `Cargo.toml` must declare `optional = true`.
///
/// `claude_runner` gates every dependency behind the `enabled` feature flag.
/// An unconditionally required dep would break `cargo check --no-default-features`
/// and violate the dep-constraints invariant. This test catches the regression where
/// a new dependency is added without `optional = true`.
///
/// Spec: `tests/docs/invariant/002_dep_constraints.md` IT-4
#[ test ]
fn all_cargo_dependencies_are_optional()
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let cargo_toml = fs::read_to_string( manifest.join( "Cargo.toml" ) )
    .expect( "Cannot read Cargo.toml" );
  let mut in_deps = false;
  let mut violations = Vec::new();
  for line in cargo_toml.lines()
  {
    let trimmed = line.trim();
    if trimmed.starts_with( '[' )
    {
      in_deps = trimmed == "[dependencies]";
      continue;
    }
    if !in_deps { continue; }
    if trimmed.is_empty() || trimmed.starts_with( '#' ) { continue; }
    if !trimmed.contains( "optional = true" )
    {
      violations.push( line.to_owned() );
    }
  }
  assert!(
    violations.is_empty(),
    "All [dependencies] must declare `optional = true` (dep constraint IT-4).\nViolations:\n{}",
    violations.join( "\n" )
  );
}

/// BUG-506 reproducer: `Cargo.toml` must set `default-run = "clr"` under `[package]`.
///
/// # Root Cause
/// This crate declares 4 `[[bin]]` targets (`claude_runner`, `clr`, `c`,
/// `fake_claude_control`). Cargo requires disambiguation (`--bin <name>` or a
/// `default-run` key) whenever a manifest has more than one binary target; with
/// neither present, a bare `cargo run -p claude_runner -- <args>` is rejected by
/// Cargo before any crate code runs.
///
/// # Why Not Caught
/// The project's own tooling never hits it — `verb/run.d/l1` already hardcodes
/// `--bin clr`, `verb/build`/`verb/install` never need run-target selection. Only
/// the manually-typed bare form, used throughout `tests/manual/readme.md`'s ~90
/// test-case recipes, is affected — nothing in the automated suite invokes
/// `cargo run`, so no prior test exercised this path.
///
/// # Fix Applied
/// Added `default-run = "clr"` under `[package]` in `Cargo.toml`. `clr` is the
/// canonical product name used throughout documentation, CLI help text, and error
/// messages; `claude_runner`/`clr`/`c` are behaviorally identical thin wrappers
/// (each is `fn main() { claude_runner::run_cli(); }`), so the choice is a
/// naming/UX decision only, not a behavior change.
///
/// # Prevention
/// Any crate adding a second `[[bin]]` target must set `default-run` in the same
/// change. See BUG-506's Prevention section for a read-only detection command.
///
/// # Pitfall
/// A future `[[bin]]` target added to this crate must not remove or shadow this
/// key — the plain `cargo run -p claude_runner -- <args>` form (used throughout
/// `tests/manual/readme.md`) depends on it resolving unambiguously.
#[ test ]
#[ doc = "bug_reproducer(BUG-506)" ]
fn default_run_is_set_to_clr()
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let cargo_toml = fs::read_to_string( manifest.join( "Cargo.toml" ) )
    .expect( "Cannot read Cargo.toml" );
  let has_default_run = cargo_toml
    .lines()
    .any( |line| line.trim() == r#"default-run = "clr""# );
  assert!(
    has_default_run,
    "Cargo.toml [package] must set default-run = \"clr\" (BUG-506): with 4 [[bin]] \
     targets and no default-run, `cargo run -p claude_runner -- <args>` is ambiguous \
     and rejected by Cargo before any crate code runs."
  );
}

/// BUG-507 reproducer: `tests/manual/readme.md` must not `cd` to an external directory
/// and then invoke `cargo run -p claude_runner` from inside it.
///
/// # Root Cause
/// `cargo run -p <crate>` locates the workspace `Cargo.toml` by walking up from the
/// shell's cwd, unconditionally, before it ever reaches package or binary selection.
/// TC-84/85/86/88 simulate "the user is in a different project directory" (the point of
/// testing `--to`/`--from` CWD-defaulting) by `cd`-ing to a `mktemp -d` directory outside
/// the repo tree, then invoking `cargo run -p claude_runner` from inside it — which can
/// never find this repo's `Cargo.toml` and fails before any crate code runs.
///
/// # Why Not Caught
/// No automated test executes `tests/manual/readme.md`'s shell snippets (manual testing
/// plans are run by hand per project convention). TC-84/85/86/88 were newly authored the
/// same session as BUG-506; each recipe also makes an earlier, non-`cd`'d `cargo run`
/// call that BUG-506 broke independently, and that earlier failure's cascading error
/// noise dominated the pre-fix output, masking this second, distinct defect until
/// BUG-506's own fix cleared its error first.
///
/// # Fix Applied
/// TC-84/85/86/88 now build the binary once (`cargo build -q -p claude_runner`) and
/// resolve `TARGET_DIR="${CARGO_TARGET_DIR:-$(pwd)/target}"` while still in the original
/// crate directory — before any `cd` — then invoke the resolved `"$BIN"` directly instead
/// of `cargo run -p claude_runner` after `cd`-ing away. Generalizes NC-27's already
/// documented `${CARGO_TARGET_DIR:-target}` idiom to the absolute-path case this use
/// needs (NC-27's own check never changes directory, so a relative fallback was safe
/// there; here `$(pwd)` must be captured before the subsequent `cd` changes it).
///
/// # Prevention
/// Any manual-test recipe that `cd`s outside this crate's workspace tree to simulate
/// "a different project directory" must resolve `clr`'s absolute binary path before that
/// `cd`, and invoke the resolved path directly — never `cargo run -p claude_runner` after
/// leaving the workspace tree. See BUG-507's Prevention section for a read-only detection
/// command.
///
/// # Pitfall
/// A future recipe added to `tests/manual/readme.md` that needs to simulate "the user is
/// elsewhere" must not reintroduce `cd "$VAR" && cargo run -p claude_runner` — this test
/// only catches that exact shape; a differently-worded but equivalent mistake (e.g. `cd
/// "$VAR"; cargo run -p claude_runner` on separate lines) would not trip this assertion.
#[ test ]
#[ doc = "bug_reproducer(BUG-507)" ]
fn manual_doc_has_no_cd_then_cargo_run_pattern()
{
  let manifest = Path::new( env!( "CARGO_MANIFEST_DIR" ) );
  let doc = fs::read_to_string( manifest.join( "tests/manual/readme.md" ) )
    .expect( "Cannot read tests/manual/readme.md" );
  let broken_lines : Vec< &str > = doc
    .lines()
    .filter( | line |
    {
      let trimmed = line.trim_start();
      trimmed.starts_with( "(cd " ) && trimmed.contains( "&& cargo run -p claude_runner" )
    } )
    .collect();
  assert!(
    broken_lines.is_empty(),
    "tests/manual/readme.md must not `cd` to an external directory and then invoke \
     `cargo run -p claude_runner` from inside it (BUG-507): cargo run -p <crate> cannot \
     locate the workspace Cargo.toml from outside the repo tree, so this pattern always \
     fails with 'could not find Cargo.toml'. Resolve an absolute binary path before \
     cd-ing away and invoke it directly instead.\nViolating lines:\n{}",
    broken_lines.join( "\n" )
  );
}

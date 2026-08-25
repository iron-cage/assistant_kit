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
///
/// Never descends into `target/`, `.git/`, or any hyphen-prefixed directory, and never yields a
/// hyphen-prefixed file. Crate-scoped callers see no behavior change (their `docs/` trees contain
/// none of those); the exclusions exist so a workspace-root walk stays fast and stays restricted
/// to committed documentation — a temporary file is permitted to reference other temporary files,
/// so scanning one would produce noise rather than signal.
fn md_files_in_dir( dir : &Path, skip_names : &[ &str ] ) -> Vec< PathBuf >
{
  let mut result = Vec::new();
  let entries = fs::read_dir( dir )
    .unwrap_or_else( |e| panic!( "Cannot read dir {}: {e}", dir.display() ) );
  for entry in entries.flatten()
  {
    let path = entry.path();
    let name = path.file_name().and_then( |s| s.to_str() ).unwrap_or( "" );
    if name.starts_with( '-' )
    {
      continue;
    }
    // `entry.file_type()` reports on the entry itself; `path.is_dir()` would follow a symlink.
    // A symlink pointing at any ancestor makes this recursion unbounded, and the walk now
    // spans the whole workspace rather than a single crate's `docs/`, so the opportunity for
    // one is real. Skipping symlinks outright costs nothing: a symlinked `.md` whose target
    // lives in the tree is already scanned at that target's own path.
    let Ok( file_type ) = entry.file_type() else { continue };
    if file_type.is_symlink()
    {
      continue;
    }
    if file_type.is_dir()
    {
      if name == "target" || name == ".git"
      {
        continue;
      }
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

/// Workspace root, two levels up from this crate's manifest (`<root>/module/claude_runner`).
fn workspace_root() -> PathBuf
{
  Path::new( env!( "CARGO_MANIFEST_DIR" ) )
    .parent()
    .expect( "module/ dir must have parent" )
    .parent()
    .expect( "workspace root must be 2 levels up from crate" )
    .to_path_buf()
}

/// Whether `line` *points a reader at* a hyphen-prefixed temporary file.
///
/// Two pointer forms are recognized, matching the rule's own wording ("imports, includes, links
/// in committed docs"):
///
/// 1. A markdown link whose target is hyphen-prefixed — `](-...)`. Always a pointer.
/// 2. Prose of the form ``See `-foo.md` `` — the shape both real violations took.
///
/// Deliberately NOT flagged: a bare inline-code mention of a hyphen-prefixed name that merely
/// *names* something rather than directing the reader to open it. `tests/docs/cli/command/
/// 12_topics.md` describes a fixture layout containing `-not-a-dir.txt`; that is a scenario
/// description, not a link, and forbidding it would be enforcing a rule that does not exist.
///
/// Two discriminators keep CLI flags out: a `--flag` is rejected by the second-hyphen check, and
/// a filename is required to contain a `.`. Without the first, ``see `--quiet` `` and four
/// siblings in `docs/claude_params/` would all read as violations.
fn line_points_at_temp_file( line : &str ) -> bool
{
  if line.contains( "](-" )
  {
    return true;
  }
  let lowered = line.to_lowercase();
  let Some( start ) = lowered.find( "see `-" ) else { return false };
  let after = &lowered[ start + "see `-".len().. ];
  // `--flag`, not a filename.
  if after.starts_with( '-' )
  {
    return false;
  }
  let Some( end ) = after.find( '`' ) else { return false };
  after[ ..end ].contains( '.' )
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

/// Committed documentation must never point a reader at a hyphen-prefixed temporary file.
///
/// # What This Guards
/// Project convention makes `-`-prefixed files temporary and gitignored: freely deletable,
/// never committed. A permanent document that links to one is therefore a reference that is
/// *already* dead for every reader except the author, on the day it is written. This is the
/// one direction the rule forbids — a temporary file referencing a permanent one is fine.
///
/// # Why This Is Workspace-Scoped
/// Unlike the crate-scoped guards above, this one walks the whole workspace from
/// [`workspace_root`]. The defect class is not specific to `claude_runner`: the three
/// instances that motivated this test were in `claude_runner_core/tests/manual/readme.md`,
/// `claude_storage/tests/manual/readme.md`, and `claude_storage_core/changelog.md`. A
/// crate-scoped version would have caught none of them.
///
/// # What Counts As A Pointer
/// [`line_points_at_temp_file`] recognizes exactly two forms — a markdown link `](-...)`, and
/// prose of the shape ``See `-foo.md` ``. It deliberately does not flag a bare inline-code
/// mention that merely *names* a hyphen-prefixed thing without directing the reader to it:
/// `tests/docs/cli/command/12_topics.md` describes a fixture layout containing
/// `-not-a-dir.txt`, which is a scenario description, not a link. Flagging it would enforce a
/// prohibition the convention does not actually state.
///
/// # Pitfall
/// The predicate is intentionally shape-based, not exhaustive. A pointer worded differently
/// (`` refer to `-notes.md` ``, or a bare `-plan.md` on its own line) slips through. That is the
/// accepted trade: a broader pattern collides with legitimate CLI-flag prose — `` see
/// `--quiet` `` and four siblings under `docs/claude_params/` are why the `--`-rejection and
/// the required `.` both exist. Widen the predicate only alongside a re-check of those.
#[ test ]
fn no_temp_file_pointers_in_committed_docs()
{
  let root = workspace_root();
  let docs = md_files_in_dir( &root, &[] );
  // A sweep that silently collapses to a subtree would pass vacuously. The workspace carries
  // ~2500 markdown files and the largest single crate ~412, so this floor fails loudly if
  // `workspace_root()` ever resolves to a crate directory instead of the workspace.
  assert!(
    docs.len() > 1000,
    "Expected a workspace-wide markdown sweep, but only {} files were found under {}. \
     This assertion exists because a guard that scans nothing passes for the wrong reason.",
    docs.len(),
    root.display()
  );
  let violations : Vec< String > = docs
    .iter()
    .flat_map( | path |
    {
      let content = fs::read_to_string( path )
        .unwrap_or_else( |e| panic!( "Cannot read {}: {e}", path.display() ) );
      let relative = path.strip_prefix( &root ).unwrap_or( path ).to_path_buf();
      content
        .lines()
        .enumerate()
        .filter( |( _, line )| line_points_at_temp_file( line ) )
        .map( |( i, line )| format!( "  {}:{}: {}", relative.display(), i + 1, line.trim() ) )
        .collect::< Vec< String > >()
    } )
    .collect();
  assert!(
    violations.is_empty(),
    "Committed documentation must not point readers at hyphen-prefixed temporary files: \
     those are gitignored and freely deletable, so the reference is dead for everyone but \
     its author. Inline the content the reader needs, or promote the target to a permanent \
     (non-hyphenated) file.\nViolating lines:\n{}",
    violations.join( "\n" )
  );
}

/// [`line_points_at_temp_file`] must discriminate pointers from lookalikes.
///
/// # Why This Exists Separately
/// `no_temp_file_pointers_in_committed_docs` is green on the current corpus, so it alone
/// cannot demonstrate the predicate fires at all — a predicate hardwired to `false` would
/// pass it identically. These cases pin both directions against fixed inputs, independent of
/// what the repository happens to contain today.
///
/// # Pitfall
/// The negative cases are not hypothetical. `--quiet`, `--agents`, `--tools`,
/// `--replay-user-messages`, and `--setting-sources` all appear in committed docs introduced
/// by the word "see" and wrapped in inline code — the same shape a temp-file pointer takes.
/// A predicate keying on that shape alone flags every one of them. Any widening of the
/// predicate must keep these five negatives passing.
#[ test ]
fn temp_file_pointer_predicate_discriminates()
{
  // Pointers — must be caught.
  assert!( line_points_at_temp_file( "See [the plan](-plan.md) for details." ) );
  assert!( line_points_at_temp_file( "See `-corner_cases_exhaustive.md` for analysis." ) );
  assert!( line_points_at_temp_file( "see `-v1_0_release_checklist.md` for the process." ) );

  // CLI flags — the reason the `--` rejection exists.
  assert!( !line_points_at_temp_file( "see `--quiet` to suppress output" ) );
  assert!( !line_points_at_temp_file( "see `--replay-user-messages` for details" ) );
  assert!( !line_points_at_temp_file( "see `--tools \"\"` to disable tools" ) );

  // Naming a hyphen-prefixed thing without pointing at it — deliberately allowed.
  assert!( !line_points_at_temp_file( "The fixture directory contains `-not-a-dir.txt`." ) );

  // No extension: not a file reference.
  assert!( !line_points_at_temp_file( "see `-notes` in the sibling directory" ) );

  // Ordinary prose must not trip it.
  assert!( !line_points_at_temp_file( "See the readme for installation instructions." ) );
}

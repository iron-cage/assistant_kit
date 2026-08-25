//! Docs-to-code consistency for the `clj` CLI surface — DC-1 through DC-5.
//!
//! ## Purpose
//!
//! The CLI surface is described in four places that had no way of contradicting
//! each other loudly: each command page's Parameters table, each parameter
//! page's Referenced Commands table, the `Commands` column of
//! `docs/cli/param/readme.md`, and `known_params` in `src/cli_main.rs`. Three of
//! those are prose. This file makes disagreement between any of them a failing
//! test.
//!
//! The code side is read from the binary rather than from the source: passing an
//! unknown key makes `clj` print `Accepted: …`, which is `known_params` verbatim.
//! That keeps the gate honest even though `known_params` is private to the
//! binary, and it fails for the right reason if the rejection path itself ever
//! regresses.
//!
//! ## Specification References
//!
//! - `docs/cli/param/readme.md` — the authoritative per-parameter command set
//! - `docs/cli/command/readme.md` — why a command page may enumerate fewer params than it accepts
//!
//! ## Test Matrix
//!
//! | Test | Gate | Scenario |
//! |------|------|----------|
//! | `dc1_command_page_params_have_pages` | DC-1 | Every param a command page names has a `param/NN_<name>.md` |
//! | `dc2_live_param_pages_are_reachable` | DC-2 | Every live param page is named by at least one command page |
//! | `dc3_param_readme_matches_param_pages` | DC-3 | `param/readme.md`'s Commands column equals the page's own Referenced Commands |
//! | `dc4_param_pages_match_known_params` | DC-4 | That set equals what the binary actually accepts |
//! | `dc5_param_readme_totals_are_real` | DC-5 | Both declared totals equal the live parameter count |
//!
//! A parameter page is *live* unless its `**Type:**` line says `not accepted`.
//! That is the tombstone marker — `28_include_stdout.md` describes a flag that
//! was superseded, is linked from documents that still exist, and must not be
//! held to DC-2 through DC-5. Keying the exemption off the page's own text
//! rather than a list in this file means retracting the next parameter needs no
//! edit here.
//!
//! ## Known Pitfalls
//!
//! DC-1 and DC-2 are deliberately asymmetric. A command page may enumerate
//! *fewer* parameters than the command accepts — every event-reading command
//! takes the whole filter vocabulary whether or not its page spells all of it
//! out — so DC-2 asks only for one command page per parameter, never for all of
//! them. DC-4 is where the full set is checked, against the binary rather than
//! against another document.

#![ allow( missing_docs ) ]
#![ cfg( unix ) ]

use std::collections::{ BTreeMap, BTreeSet };
use std::path::{ Path, PathBuf };
use std::process::Command;

const CLJ          : &str = env!( "CARGO_BIN_EXE_clj" );
const MANIFEST_DIR : &str = env!( "CARGO_MANIFEST_DIR" );

fn assert_container()
{
  let in_container = Path::new( "/.dockerenv" ).exists()
    || Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: ./verb/test (from workspace root)\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

// ── Markdown helpers ──────────────────────────────────────────────────────────

/// `docs/cli/` for this crate.
fn cli_docs() -> PathBuf
{
  Path::new( MANIFEST_DIR ).join( "docs" ).join( "cli" )
}

/// Every `*.md` in `dir` except `readme.md`, sorted by filename.
fn instance_pages( dir : &Path ) -> Vec< PathBuf >
{
  let entries = std::fs::read_dir( dir )
    .unwrap_or_else( | e | panic!( "cannot read {}: {e}", dir.display() ) );
  let mut out : Vec< PathBuf > = entries
    .map( | e | e.unwrap_or_else( | e | panic!( "cannot enumerate {}: {e}", dir.display() ) ).path() )
    .filter( | p | p.extension().is_some_and( | x | x == "md" ) )
    .filter( | p | p.file_name().is_some_and( | n | n != "readme.md" ) )
    .collect();
  out.sort();
  assert!( !out.is_empty(), "no instance pages under {} — the gates would be vacuous", dir.display() );
  out
}

/// Split a Markdown table row into trimmed cells.
///
/// Returns `None` for anything that is not a row, including the `|---|---|`
/// separator — so no caller has to recognise one.
fn table_cells( line : &str ) -> Option< Vec< &str > >
{
  let trimmed = line.trim();
  if !trimmed.starts_with( '|' ) { return None; }
  let cells : Vec< &str > = trimmed.trim_matches( '|' ).split( '|' ).map( str::trim ).collect();
  let separator = cells
    .iter()
    .all( | c | !c.is_empty() && c.chars().all( | ch | ch == '-' || ch == ':' ) );
  if separator { return None; }
  Some( cells )
}

/// Lines under the first heading `want` accepts, up to the next heading.
fn section_by( content : &str, want : impl Fn( &str ) -> bool ) -> Vec< &str >
{
  let mut out = Vec::new();
  let mut inside = false;
  for line in content.lines()
  {
    if line.trim_start().starts_with( '#' )
    {
      if inside { break; }
      inside = want( line.trim() );
      continue;
    }
    if inside { out.push( line ); }
  }
  out
}

/// Lines under the heading exactly equal to `heading`.
fn section< 'a >( content : &'a str, heading : &str ) -> Vec< &'a str >
{
  section_by( content, | line | line == heading )
}

/// The text between the first pair of backticks in `cell`.
fn backticked( cell : &str ) -> Option< &str >
{
  let ( _, rest ) = cell.split_once( '`' )?;
  rest.split_once( '`' ).map( | ( inner, _ ) | inner )
}

fn read( path : &Path ) -> String
{
  std::fs::read_to_string( path )
    .unwrap_or_else( | e | panic!( "cannot read {}: {e}", path.display() ) )
}

// ── Document models ───────────────────────────────────────────────────────────

/// Every command page's own name and the parameters its Parameters table names.
fn command_page_params() -> BTreeMap< String, BTreeSet< String > >
{
  let mut out = BTreeMap::new();
  for page in instance_pages( &cli_docs().join( "command" ) )
  {
    let content = read( &page );
    let title = content
      .lines()
      .next()
      .and_then( | l | l.strip_prefix( "# " ) )
      .unwrap_or_else( || panic!( "{} has no `# .command` title line", page.display() ) )
      .trim()
      .to_owned();
    let params : BTreeSet< String > = section( &content, "### Parameters" )
      .iter()
      .filter_map( | line | table_cells( line ) )
      .filter_map( | cells | backticked( cells[ 0 ] ).map( str::to_owned ) )
      .collect();
    assert!( !params.is_empty(), "{} has an empty Parameters table", page.display() );
    out.insert( title, params );
  }
  out
}

/// One parameter page, reduced to what the gates below compare.
struct ParamPage
{
  /// Parameter name, from the `# CLI Parameter: <name>` title.
  name     : String,
  /// Filename, for failure messages that have to be actionable.
  file     : String,
  /// False when the `**Type:**` line marks the page a tombstone.
  live     : bool,
  /// Commands the page's Referenced Commands table claims.
  commands : BTreeSet< String >,
}

fn param_pages() -> Vec< ParamPage >
{
  let mut out = Vec::new();
  for page in instance_pages( &cli_docs().join( "param" ) )
  {
    let content = read( &page );
    let name = content
      .lines()
      .next()
      .and_then( | l | l.strip_prefix( "# CLI Parameter: " ) )
      .unwrap_or_else( || panic!( "{} has no `# CLI Parameter: <name>` title line", page.display() ) )
      .trim()
      .to_owned();
    let type_line = content
      .lines()
      .find( | l | l.trim_start().starts_with( "- **Type:**" ) )
      .unwrap_or_else( || panic!( "{} has no `- **Type:**` line", page.display() ) );
    let commands : BTreeSet< String > = section( &content, "### Referenced Commands" )
      .iter()
      .filter_map( | line | table_cells( line ) )
      .filter( | cells | cells.len() >= 2 )
      .filter_map( | cells | backticked( cells[ 1 ] ).map( str::to_owned ) )
      .collect();
    out.push( ParamPage
    {
      name,
      file : page.file_name().unwrap_or_default().to_string_lossy().into_owned(),
      live : !type_line.contains( "not accepted" ),
      commands,
    } );
  }
  out
}

/// The `Commands` column of `param/readme.md`'s All Parameters table, per parameter.
fn param_readme_commands() -> BTreeMap< String, BTreeSet< String > >
{
  let content = read( &cli_docs().join( "param" ).join( "readme.md" ) );
  let mut out = BTreeMap::new();
  for cells in section_by( &content, | l | l.starts_with( "### All Parameters" ) )
    .iter()
    .filter_map( | line | table_cells( line ) )
  {
    assert!(
      cells.len() >= 5,
      "All Parameters row has {} cells, expected at least 5: {cells:?}",
      cells.len(),
    );
    let Some( name ) = backticked( cells[ 1 ] ) else { continue };
    let commands = cells[ 4 ]
      .split( ',' )
      .map( str::trim )
      .filter( | s | !s.is_empty() )
      .map( str::to_owned )
      .collect();
    out.insert( name.to_owned(), commands );
  }
  assert!( !out.is_empty(), "parsed no rows from param/readme.md's All Parameters table" );
  out
}

/// What `command` actually accepts, read back out of the binary's own rejection.
///
/// `reject_unknown_params` runs before any command does work, so this probe
/// neither writes a chart, binds a port, nor deletes a journal file.
fn accepted_params( command : &str ) -> BTreeSet< String >
{
  let out = Command::new( CLJ )
    .args( [ command, "zz_probe_unknown_param::1" ] )
    .output()
    .unwrap_or_else( | e | panic!( "failed to run clj {command}: {e}" ) );
  assert_eq!(
    out.status.code(), Some( 1 ),
    "clj {command} zz_probe_unknown_param::1 should exit 1; unknown-param rejection has regressed",
  );
  let stderr = String::from_utf8_lossy( &out.stderr ).into_owned();
  let line = stderr
    .lines()
    .find_map( | l | l.strip_prefix( "Accepted: " ) )
    .unwrap_or_else( || panic!(
      "clj {command} printed no `Accepted:` line — this gate reads `known_params` \
       through it, so it cannot check anything:\n{stderr}"
    ) );
  line.split( ',' ).map( | s | s.trim().to_owned() ).collect()
}

// ── DC-1 : every param a command page names has a page ───────────────────────

#[ test ]
fn dc1_command_page_params_have_pages()
{
  let pages : BTreeSet< String > = param_pages().into_iter().map( | p | p.name ).collect();
  let mut missing = Vec::new();
  for ( command, params ) in command_page_params()
  {
    for param in params
    {
      if !pages.contains( &param ) { missing.push( format!( "{command} names `{param}`" ) ); }
    }
  }
  assert!(
    missing.is_empty(),
    "command pages name parameters with no page under docs/cli/param/:\n  {}\n\
     Add the page, or drop the parameter from the command's table.",
    missing.join( "\n  " ),
  );
}

// ── DC-2 : every live param page is named by some command page ───────────────

#[ test ]
fn dc2_live_param_pages_are_reachable()
{
  let named : BTreeSet< String > = command_page_params().into_values().flatten().collect();
  let orphans : Vec< String > = param_pages()
    .into_iter()
    .filter( | p | p.live && !named.contains( &p.name ) )
    .map( | p | format!( "{} (`{}`)", p.file, p.name ) )
    .collect();
  assert!(
    orphans.is_empty(),
    "parameter pages that no command page enumerates:\n  {}\n\
     Either add the parameter to a command's Parameters table, or retract the page \
     by marking its `- **Type:**` line `not accepted`.",
    orphans.join( "\n  " ),
  );
}

// ── DC-3 : param/readme.md agrees with each param page ───────────────────────

#[ test ]
fn dc3_param_readme_matches_param_pages()
{
  let readme = param_readme_commands();
  let mut problems = Vec::new();
  for page in param_pages().into_iter().filter( | p | p.live )
  {
    let Some( listed ) = readme.get( &page.name ) else
    {
      problems.push( format!( "`{}` has a page ({}) but no All Parameters row", page.name, page.file ) );
      continue;
    };
    if *listed != page.commands
    {
      problems.push( format!(
        "`{}`: readme says {:?}, {} says {:?}",
        page.name, listed, page.file, page.commands,
      ) );
    }
  }
  for name in readme.keys()
  {
    let has_live_page = param_pages().into_iter().any( | p | p.live && p.name == *name );
    if !has_live_page
    {
      problems.push( format!( "`{name}` has an All Parameters row but no live page" ) );
    }
  }
  assert!(
    problems.is_empty(),
    "docs/cli/param/readme.md and the parameter pages disagree:\n  {}",
    problems.join( "\n  " ),
  );
}

// ── DC-4 : the pages agree with what the binary accepts ──────────────────────

#[ test ]
fn dc4_param_pages_match_known_params()
{
  assert_container();

  let commands : Vec< String > = command_page_params().into_keys().collect();
  let accepted : BTreeMap< String, BTreeSet< String > > = commands
    .iter()
    .map( | c | ( c.clone(), accepted_params( c ) ) )
    .collect();

  let mut problems = Vec::new();
  for page in param_pages().into_iter().filter( | p | p.live )
  {
    let real : BTreeSet< String > = accepted
      .iter()
      .filter( | ( _, params ) | params.contains( &page.name ) )
      .map( | ( command, _ ) | command.clone() )
      .collect();
    if real != page.commands
    {
      problems.push( format!(
        "`{}`: binary accepts it on {:?}, {} claims {:?}",
        page.name, real, page.file, page.commands,
      ) );
    }
  }

  // The other direction: a param the binary accepts on some command but that no
  // page claims at all would otherwise slip through, since the loop above only
  // walks pages that exist.
  let claimed : BTreeSet< String > = param_pages().into_iter().map( | p | p.name ).collect();
  for ( command, params ) in &accepted
  {
    for param in params
    {
      if !claimed.contains( param )
      {
        problems.push( format!( "{command} accepts `{param}`, which has no page at all" ) );
      }
    }
  }

  assert!(
    problems.is_empty(),
    "docs/cli/param/ and `known_params` disagree about which commands take what:\n  {}",
    problems.join( "\n  " ),
  );
}

// ── DC-5 : param/readme.md's declared totals are the real count ──────────────

#[ test ]
fn dc5_param_readme_totals_are_real()
{
  let content = read( &cli_docs().join( "param" ).join( "readme.md" ) );
  let live = param_pages().into_iter().filter( | p | p.live ).count();

  let heading = content
    .lines()
    .find( | l | l.trim().starts_with( "### All Parameters" ) )
    .unwrap_or_else( || panic!( "param/readme.md has no `### All Parameters` heading" ) );
  let declared_heading : usize = heading
    .split_whitespace()
    .find_map( | w | w.trim_start_matches( '(' ).parse().ok() )
    .unwrap_or_else( || panic!( "no count in heading {heading:?}" ) );

  let total = content
    .lines()
    .find( | l | l.trim().starts_with( "**Total:**" ) && l.contains( "parameters" ) )
    .unwrap_or_else( || panic!( "param/readme.md has no `**Total:** N parameters` line" ) );
  let declared_total : usize = total
    .split_whitespace()
    .find_map( | w | w.parse().ok() )
    .unwrap_or_else( || panic!( "no count in total line {total:?}" ) );

  assert_eq!(
    declared_heading, live,
    "`### All Parameters ({declared_heading} total)` but {live} live parameter pages exist",
  );
  assert_eq!(
    declared_total, live,
    "`**Total:** {declared_total} parameters` but {live} live parameter pages exist",
  );
}

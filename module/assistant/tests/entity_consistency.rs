//! Doc entity index consistency tests.
//!
//! ## Purpose
//!
//! Verify every `entity.md` module index in the workspace stays consistent
//! with the doc instance files actually on disk. The indexes silently drift
//! as documentation evolves — wrong `Instances` counts, deleted files still
//! listed, new files unregistered. All checks read Markdown and directory
//! listings statically — no build artefacts or network access required.
//!
//! ## Specification References
//!
//! - `docs/invariant/006_doc_entity_index_consistency.md` — index accuracy (DEI-1, DEI-2)
//!
//! ## Test Matrix
//!
//! | Test | Spec | Scenario |
//! |------|------|----------|
//! | `dei1_entity_index_counts_match_files` | DEI-1 | Declared `Instances` equals instance-file count per entity dir |
//! | `dei2_entity_index_files_exist` | DEI-2 | Every Master Doc Instances Table `File` link resolves to a file |
//!
//! An instance file is any `*.md` in the entity directory except `readme.md`
//! (the registry) and `procedure.md` (the ops doc). Prefix conventions vary
//! by entity family (`NNN_` per `doc_des`; `NN_`/`cmd_NNN_` for CLI families
//! per `cli_doc_des`) and are governed by those design rulesets, not counted
//! differently here.

use std::{
  fs,
  path::{ Path, PathBuf },
};

// ──────────────────────────────── constants ────────────────────────────────

const MANIFEST_DIR : &str = env!( "CARGO_MANIFEST_DIR" );

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

/// Discover every `entity.md` module index in the workspace:
/// `docs/entity.md` plus each `module/*/docs/entity.md`.
///
/// Discovery-based rather than a hardcoded path list — BUG-005 documented how
/// a hand-maintained inventory in this test suite silently drifts stale when
/// crates are added or removed.
fn entity_md_files( workspace_root : &Path ) -> Vec< PathBuf >
{
  let mut found = Vec::new();

  let ws_index = workspace_root.join( "docs" ).join( "entity.md" );
  if ws_index.is_file()
  {
    found.push( ws_index );
  }

  let module_dir = workspace_root.join( "module" );
  let entries = fs::read_dir( &module_dir )
    .unwrap_or_else( | e | panic!( "cannot read {}: {e}", module_dir.display() ) );
  let mut module_hits = Vec::new();
  for entry in entries
  {
    let entry = entry
      .unwrap_or_else( | e | panic!( "cannot enumerate {}: {e}", module_dir.display() ) );
    let candidate = entry.path().join( "docs" ).join( "entity.md" );
    if candidate.is_file()
    {
      module_hits.push( candidate );
    }
  }
  module_hits.sort();
  found.extend( module_hits );

  assert!(
    found.len() >= 2,
    "entity.md discovery returned {} files — expected the workspace index plus \
     at least one module index; the discovery pattern is likely broken and the \
     consistency checks would be vacuous",
    found.len(),
  );
  found
}

/// Extract `(entity_name, entity_dir_relative, declared_instance_count)`
/// triples from the Master Doc Entities Table.
///
/// Entity rows are identified structurally, not by section tracking: the first
/// cell is a backtick-quoted directory name with a trailing slash (e.g.
/// `` `cli/param/` ``) — a shape no header, separator, or instance row shares.
/// The last cell is the `Instances` count.
///
/// The entity directory is NOT derived from the name: entities outside `docs/`
/// (e.g. `` `tests/docs/api/` ``) are named by crate-relative convention while
/// their `Master File` link carries the real docs-relative location
/// (`../tests/docs/api/readme.md`). The link target's parent is the
/// authoritative directory, so that cell — the first containing a markdown
/// link — is what gets resolved.
fn parse_entity_rows( content : &str ) -> Vec< ( String, String, usize ) >
{
  let mut rows = Vec::new();
  for line in content.lines()
  {
    let trimmed = line.trim();
    if !trimmed.starts_with( '|' )
    {
      continue;
    }
    let cells : Vec< &str > = trimmed.trim_matches( '|' ).split( '|' ).map( str::trim ).collect();
    if cells.len() < 2
    {
      continue;
    }
    let Some( quoted ) = cells[ 0 ].strip_prefix( '`' ).and_then( | s | s.strip_suffix( '`' ) ) else
    {
      continue;
    };
    let Some( name ) = quoted.strip_suffix( '/' ) else
    {
      continue;
    };
    let last = cells[ cells.len() - 1 ];
    let count = last.parse::< usize >().unwrap_or_else( | _ | panic!(
      "entity row for `{name}/` has non-numeric Instances cell {last:?} — \
       table format drifted from what this parser recognizes"
    ) );
    let master_file = cells[ 1.. cells.len() - 1 ]
      .iter()
      .find_map( | cell | extract_link_target( cell ) )
      .unwrap_or_else( || panic!(
        "entity row for `{name}/` has no Master File markdown link — \
         table format drifted from what this parser recognizes"
      ) );
    let dir = master_file
      .rsplit_once( '/' )
      .map_or_else( || name.to_string(), | ( parent, _ ) | parent.to_string() );
    rows.push( ( name.to_string(), dir, count ) );
  }
  rows
}

/// Extract the target of the first markdown link (`[text](target)`) in `cell`,
/// if any.
fn extract_link_target( cell : &str ) -> Option< String >
{
  let open = cell.find( "](" )?;
  let close = cell[ open.. ].find( ')' )? + open;
  if close <= open + 2
  {
    return None;
  }
  Some( cell[ open + 2..close ].to_string() )
}

/// Extract every `File`-column link target from the Master Doc Instances
/// Table.
///
/// Instance rows are identified structurally: the last cell is a markdown
/// link (`[text](target)`). Entity rows end in an integer count, headers and
/// separators in plain text — neither matches. Entity rows' `Master File`
/// column also holds a link, but never in last position.
fn parse_instance_rows( content : &str ) -> Vec< String >
{
  let mut files = Vec::new();
  for line in content.lines()
  {
    let trimmed = line.trim();
    if !trimmed.starts_with( '|' )
    {
      continue;
    }
    let cells : Vec< &str > = trimmed.trim_matches( '|' ).split( '|' ).map( str::trim ).collect();
    let Some( last ) = cells.last() else
    {
      continue;
    };
    if !last.starts_with( '[' )
    {
      continue;
    }
    if let Some( target ) = extract_link_target( last )
    {
      files.push( target );
    }
  }
  files
}

/// Returns `true` when `file_name` is a doc instance file: any `*.md` except
/// `readme.md` (the entity registry) and `procedure.md` (the ops doc).
///
/// Prefix shape (`NNN_`, `NN_`, `cmd_NNN_`) varies by entity family and is
/// governed by each family's design ruleset — DEI-1 counts instances, it does
/// not police naming.
fn is_instance_file( file_name : &str ) -> bool
{
  file_name != "readme.md"
    && file_name != "procedure.md"
    && Path::new( file_name ).extension().is_some_and( | ext | ext.eq_ignore_ascii_case( "md" ) )
}

/// Count doc instance files (non-recursive, regular files only) in `dir`.
///
/// A missing or unreadable directory counts 0 — the divergence from the
/// declared `Instances` value is then reported by DEI-1 itself rather than
/// panicking here.
fn count_instance_files( dir : &Path ) -> usize
{
  let Ok( entries ) = fs::read_dir( dir ) else
  {
    return 0;
  };
  entries
    .filter_map( Result::ok )
    .filter( | e | e.file_type().is_ok_and( | t | t.is_file() ) )
    .filter( | e | is_instance_file( &e.file_name().to_string_lossy() ) )
    .count()
}

// ---------- Invariant: Doc Entity Index Consistency (DEI-*) ----------

/// DEI-1: Every entity row's `Instances` count in every `entity.md` equals
/// the number of instance files actually present in that entity directory
/// (resolved via the row's own Master File link).
#[test]
fn dei1_entity_index_counts_match_files()
{
  let root = workspace_root();
  let mut violations = Vec::new();

  for entity_md in entity_md_files( &root )
  {
    let content = fs::read_to_string( &entity_md )
      .unwrap_or_else( | e | panic!( "cannot read {}: {e}", entity_md.display() ) );
    // SAFETY: entity.md always lives inside a docs/ directory — never at root.
    let docs_dir = entity_md.parent().unwrap();

    let rows = parse_entity_rows( &content );
    assert!(
      !rows.is_empty(),
      "DEI-1: no entity rows parsed from {} — the Master Doc Entities Table \
       format drifted from what parse_entity_rows() recognizes; the count \
       check would be vacuous",
      entity_md.display(),
    );

    for ( entity, dir, declared ) in rows
    {
      let actual = count_instance_files( &docs_dir.join( &dir ) );
      if actual != declared
      {
        violations.push( format!(
          "{}/{entity}: expected {declared} got {actual}",
          entity_md.display(),
        ) );
      }
    }
  }

  assert!(
    violations.is_empty(),
    "DEI-1: entity.md Instances counts diverge from instance files on disk \
     ({} violations):\n{}",
    violations.len(),
    violations.join( "\n" ),
  );
}

/// DEI-2: Every file listed in every `entity.md` Master Doc Instances Table
/// `File` column exists on disk, resolved relative to the `entity.md`
/// parent directory.
#[test]
fn dei2_entity_index_files_exist()
{
  let root = workspace_root();
  let mut violations = Vec::new();

  for entity_md in entity_md_files( &root )
  {
    let content = fs::read_to_string( &entity_md )
      .unwrap_or_else( | e | panic!( "cannot read {}: {e}", entity_md.display() ) );
    // SAFETY: entity.md always lives inside a docs/ directory — never at root.
    let docs_dir = entity_md.parent().unwrap();

    let files = parse_instance_rows( &content );
    assert!(
      !files.is_empty(),
      "DEI-2: no instance rows parsed from {} — the Master Doc Instances Table \
       format drifted from what parse_instance_rows() recognizes; the existence \
       check would be vacuous",
      entity_md.display(),
    );

    for file in files
    {
      if !docs_dir.join( &file ).is_file()
      {
        violations.push( format!( "{} → {file}: not found", entity_md.display() ) );
      }
    }
  }

  assert!(
    violations.is_empty(),
    "DEI-2: entity.md lists files missing on disk ({} violations):\n{}",
    violations.len(),
    violations.join( "\n" ),
  );
}

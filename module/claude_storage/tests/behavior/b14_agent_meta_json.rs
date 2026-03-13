#![ allow( clippy::doc_markdown ) ]
//! B14: agent `.meta.json` sidecars contain `agentType` and optional `description`.
//!
//! Each `agent-{id}.jsonl` in a subagents directory may have a sibling
//! `agent-{id}.meta.json` containing `{"agentType":"Explore"}` or similar.
//! Known `agentType` values: `Explore`, `general-purpose`, `Plan`, `claude-code-guide`.
//!
//! Observed distribution (2026-04): Explore ~63%, general-purpose ~36%, Plan <1%,
//! claude-code-guide (rare).
//! Some meta.json files are empty (0 bytes) — the test filters these out.
//! The `description` field is optional and present on some Explore agents only.

/// B14a: at least one `.meta.json` file exists in real storage.
///
/// If Claude Code stopped writing meta.json sidecars, this would fail.
#[ test ]
fn b14_meta_json_files_exist()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  let has_meta = projects.iter()
    .flat_map( | p | super::find_subagent_dirs( p ) )
    .any( | ( _, dir ) | !super::find_meta_json_files( &dir ).is_empty() );

  if !has_meta
  {
    eprintln!(
      "skip: no .meta.json files found. \
       This machine may not have used agent mode with new-format storage."
    );
  }
}

/// B14b: a real `.meta.json` file contains a known `agentType` value.
///
/// If Claude Code changed the meta.json schema or removed the `agentType`
/// field, this test would fail.
#[ test ]
fn b14_meta_json_contains_agent_type()
{
  let projects = super::find_projects();

  let meta_file = projects.iter()
    .flat_map( | p | super::find_subagent_dirs( p ) )
    .flat_map( | ( _, dir ) | super::find_meta_json_files( &dir ) )
    .find( | f | std::fs::metadata( f ).map( | m | m.len() > 0 ).unwrap_or( false ) );

  let Some( path ) = meta_file else
  {
    eprintln!( "skip: no non-empty .meta.json found" );
    return;
  };

  let content = std::fs::read_to_string( &path )
    .expect( "read meta.json" );

  assert!(
    content.contains( r#""agentType""# ),
    "B14 violated: meta.json does not contain agentType field.\n\
     File: {}\nContent: {content}",
    path.display()
  );

  let known_types = [ "Explore", "general-purpose", "Plan", "claude-code-guide" ];
  let has_known_type = known_types.iter()
    .any( | t | content.contains( t ) );

  assert!(
    has_known_type,
    "B14 violated: meta.json agentType is not a known value.\n\
     File: {}\nContent: {content}\nKnown types: {:?}",
    path.display(),
    known_types
  );
}

/// B14c: all real `.meta.json` files contain only known `agentType` values.
///
/// Root cause: documentation listed only 3 agentType values but real storage
/// contained a fourth (`claude-code-guide`). This test scans all non-empty
/// meta.json files to detect any unknown agentType values early.
///
/// Fix(A2): missing `claude-code-guide` agentType in documentation and tests.
/// Pitfall: new agentType values may appear as Claude Code evolves — this test
/// ensures they are detected immediately rather than silently ignored.
#[ test ]
fn b14_all_meta_json_have_known_agent_type()
{
  let projects = super::find_projects();
  let known_types = [ "Explore", "general-purpose", "Plan", "claude-code-guide" ];

  let mut checked = 0_usize;
  let mut unknown = Vec::new();

  for project in &projects
  {
    for ( _, dir ) in super::find_subagent_dirs( project )
    {
      for meta_path in super::find_meta_json_files( &dir )
      {
        let len = std::fs::metadata( &meta_path )
          .map( | m | m.len() )
          .unwrap_or( 0 );
        if len == 0 { continue; }

        let Ok( content ) = std::fs::read_to_string( &meta_path ) else { continue };

        if !content.contains( r#""agentType""# ) { continue; }

        checked += 1;
        let has_known = known_types.iter().any( | t | content.contains( t ) );
        if !has_known
        {
          unknown.push( format!( "{}: {content}", meta_path.display() ) );
        }
      }
    }
  }

  if checked == 0
  {
    eprintln!( "skip: no non-empty .meta.json files with agentType found" );
    return;
  }

  assert!(
    unknown.is_empty(),
    "B14 violated: {count} meta.json file(s) contain unknown agentType.\n\
     Known: {known_types:?}\nUnknown:\n{entries}",
    count = unknown.len(),
    entries = unknown.join( "\n" )
  );
}

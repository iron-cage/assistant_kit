#![ allow( clippy::doc_markdown ) ]
//! B12: agent JSONL entries carry `sessionId` equal to the parent session UUID.
//!
//! The `sessionId` field in agent entries references the parent (root) session,
//! not the agent's own ID. This is the primary programmatic link between a
//! sub-agent and the conversation that spawned it.

/// B12a: at least one subagent directory exists in real storage.
///
/// Prerequisite for the main B12 test. If Claude Code stopped creating
/// `{uuid}/subagents/` directories, this would fail.
#[ test ]
fn b12_subagent_dirs_exist()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  let has_subagents = projects.iter()
    .any( | p | !super::find_subagent_dirs( p ).is_empty() );

  if !has_subagents
  {
    eprintln!(
      "skip: no {{uuid}}/subagents/ directories found across {} projects. \
       This machine may not have used agent mode with new-format storage.",
      projects.len()
    );
  }
}

/// B12b: agent entry `sessionId` matches the parent directory UUID.
///
/// Reads the first line of a real subagent JSONL file and verifies that
/// the `sessionId` field equals the UUID from the parent directory name.
/// If Claude Code changed the semantics of `sessionId` in agent entries,
/// this test would fail.
#[ test ]
fn b12_agent_session_id_matches_parent_dir()
{
  let projects = super::find_projects();

  let sample = projects.iter()
    .flat_map( | p | super::find_subagent_dirs( p ) )
    .flat_map( | ( parent_uuid, subagents_dir ) |
    {
      super::find_subagent_sessions( &subagents_dir )
        .into_iter()
        .map( move | f | ( parent_uuid.clone(), f ) )
    })
    .next();

  let Some( ( parent_uuid, agent_path ) ) = sample else
  {
    eprintln!( "skip: no non-empty subagent JSONL found" );
    return;
  };

  let first_line = std::fs::read_to_string( &agent_path )
    .expect( "read agent session" )
    .lines()
    .next()
    .unwrap_or( "" )
    .to_owned();

  let expected = format!( r#""sessionId":"{parent_uuid}""# );
  let expected_spaced = format!( r#""sessionId": "{parent_uuid}""# );

  assert!(
    first_line.contains( &expected ) || first_line.contains( &expected_spaced ),
    "B12 violated: agent sessionId does not match parent directory UUID.\n\
     Parent UUID: {parent_uuid}\nAgent file: {}\nFirst line: {first_line}",
    agent_path.display()
  );
}

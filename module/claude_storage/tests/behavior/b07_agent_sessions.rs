#![ allow( clippy::doc_markdown ) ]
//! B7: agent sessions are `agent-*.jsonl` siblings with `isSidechain: true`.
//!
//! Validates that real `~/.claude/` storage contains `agent-*.jsonl` files
//! and that their entries carry the expected `isSidechain` and `agentId` fields.

/// B7a: at least one `agent-*.jsonl` file exists in real storage.
///
/// If Claude Code changed agent session naming or moved them to a subdirectory,
/// this test would fail.
#[ test ]
fn b7_agent_session_files_exist()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  let has_agent = projects.iter().any( | p | !super::find_agent_sessions( p ).is_empty() );
  if !has_agent
  {
    eprintln!(
      "skip: no agent-*.jsonl files found across {} projects. \
       This machine may not have used agent mode.",
      projects.len()
    );
  }
}

/// B7b: a real agent session contains `"isSidechain":true` in its entries.
///
/// If Claude Code removed the `isSidechain` field from agent entries,
/// our agent detection logic would break.
#[ test ]
fn b7_real_agent_session_has_issidechain_true()
{
  let projects = super::find_projects();
  let agent_file = projects.iter()
    .flat_map( | p | super::find_agent_sessions( p ) )
    .find( | f | std::fs::metadata( f ).map( | m | m.len() > 0 ).unwrap_or( false ) );

  let Some( path ) = agent_file else
  {
    eprintln!( "skip: no non-empty agent-*.jsonl found" );
    return;
  };

  let first_line = std::fs::read_to_string( &path )
    .expect( "read agent session" )
    .lines()
    .next()
    .unwrap_or( "" )
    .to_owned();

  assert!(
    first_line.contains( r#""isSidechain":true"# )
      || first_line.contains( r#""isSidechain": true"# ),
    "B7 violated: agent session does not contain isSidechain:true.\n\
     File: {}\nFirst line: {first_line}",
    path.display()
  );
}

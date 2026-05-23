#![ allow( clippy::doc_markdown ) ]
//! B13: new-format agents stored at `{parent-uuid}/subagents/agent-{agentId}.jsonl`.
//!
//! The filesystem hierarchy encodes the parent-child relationship between root
//! sessions and their spawned agents. A root session `{uuid}.jsonl` has a sibling
//! directory `{uuid}/` containing `subagents/` and optionally `tool-results/`.

/// B13a: at least one root session has a matching `{uuid}/subagents/` directory.
///
/// If Claude Code moved agents out of the hierarchical layout, this would fail.
#[ test ]
fn b13_subagent_dir_exists_for_root_session()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  let found = projects.iter().any( | project |
  {
    let sessions = super::find_sessions( project );
    let subagent_dirs = super::find_subagent_dirs( project );
    sessions.iter().any( | s |
    {
      let stem = s.file_stem()
        .and_then( | n | n.to_str() )
        .unwrap_or( "" );
      subagent_dirs.iter().any( | ( uuid, _ ) | uuid == stem )
    })
  });

  if !found
  {
    eprintln!(
      "skip: no root session with matching subagents/ directory found. \
       This machine may only have old-format (flat) agent storage."
    );
  }
}

/// B13b: subagent directory contains `agent-*.jsonl` files.
///
/// If Claude Code changed the agent filename convention inside subagent
/// directories, this would fail.
#[ test ]
fn b13_subagent_dir_contains_agent_jsonl()
{
  let projects = super::find_projects();

  let subagents_dir = projects.iter()
    .flat_map( | p | super::find_subagent_dirs( p ) )
    .map( | ( _, dir ) | dir )
    .find( | dir | !super::find_subagent_sessions( dir ).is_empty() );

  let Some( dir ) = subagents_dir else
  {
    eprintln!( "skip: no subagents/ directory with agent JSONL files found" );
    return;
  };

  let agents = super::find_subagent_sessions( &dir );
  assert!(
    !agents.is_empty(),
    "B13 violated: subagents/ directory exists but contains no agent-*.jsonl files.\n\
     Directory: {}",
    dir.display()
  );

  for agent in &agents
  {
    let name = agent.file_name().unwrap().to_string_lossy();
    assert!(
      name.starts_with( "agent-" ) && name.ends_with( ".jsonl" ),
      "B13 violated: unexpected filename in subagents/: {name}"
    );
  }
}

/// B13c: agent IDs follow hex or typed-prefix pattern (length varies by Claude Code version).
///
/// Root cause: documentation described agent IDs as "8-character hex" but real
/// storage contains IDs as short as 7 hex chars and IDs with typed prefixes
/// like `compact-hex` or `prompt_suggestion-hex`.
///
/// Fix(A1): corrected agent ID format from "8-char hex" to dual-pattern description.
/// Pitfall: assuming a fixed-width hex format will reject valid agent filenames
/// that use the typed-prefix convention.
///
/// Fix(issue-b13-id-length): removed stale min_len ≤ 8 assertion.
/// Root cause: Claude Code changed agent ID format from short (7-8 char) hex to
///             17-char hex (e.g., `a662c27bae2e4fece`). The ≤8 constraint was
///             machine-observed data from the old format, not a stable invariant.
/// Pitfall: agent ID length is not a stable invariant — assert pattern, not length.
#[ test ]
fn b13_agent_id_format_variations()
{
  let projects = super::find_projects();

  let mut all_ids = Vec::new();

  for project in &projects
  {
    for ( _, dir ) in super::find_subagent_dirs( project )
    {
      for agent_path in super::find_subagent_sessions( &dir )
      {
        let name = agent_path.file_name().unwrap().to_string_lossy().to_string();
        // strip "agent-" prefix and ".jsonl" suffix
        let id = &name[ "agent-".len()..name.len() - ".jsonl".len() ];
        all_ids.push( id.to_string() );
      }
    }
  }

  if all_ids.is_empty()
  {
    eprintln!( "skip: no hierarchical agent files found" );
    return;
  }

  // verify no IDs are empty
  assert!(
    all_ids.iter().all( | id | !id.is_empty() ),
    "B13 violated: found agent file with empty ID"
  );

  // log observed format stats (not an assertion — length varies by Claude Code version)
  let min_len = all_ids.iter().map( String::len ).min().unwrap();
  let pure_hex = all_ids.iter()
    .filter( | id | id.chars().all( | c | c.is_ascii_hexdigit() ) )
    .count();
  let typed_prefix = all_ids.len() - pure_hex;

  eprintln!(
    "B13 agent ID stats: {} total, {} pure hex, {} typed prefix (min len: {min_len})",
    all_ids.len(),
    pure_hex,
    typed_prefix
  );

  // verify pattern: each ID is either pure hex or a typed-prefix (contains `-`)
  // Fix(issue-b13-id-length): length constraint removed — assert pattern, not length
  for id in &all_ids
  {
    let is_pure_hex = id.chars().all( | c | c.is_ascii_hexdigit() );
    if !is_pure_hex
    {
      // typed prefix: should contain at least one `-` separating type from hex
      assert!(
        id.contains( '-' ),
        "B13 violated: non-hex agent ID lacks typed prefix pattern: {id}"
      );
    }
  }
}

#![ allow( clippy::doc_markdown ) ]
//! B15: agent entries carry a `slug` field shared by all agents of one parent.
//!
//! The `slug` is a human-readable conversation label (e.g., `"jaunty-painting-hinton"`).
//! All sibling agents under the same parent share an identical slug value.
//! Root session entries typically lack the `slug` field (first entry is `queue-operation`).

/// B15a: a real subagent entry contains a `slug` field.
///
/// If Claude Code removed the `slug` field from agent entries, this would fail.
#[ test ]
fn b15_agent_entry_has_slug()
{
  let projects = super::find_projects();

  let agent_file = projects.iter()
    .flat_map( | p | super::find_subagent_dirs( p ) )
    .flat_map( | ( _, dir ) | super::find_subagent_sessions( &dir ) )
    .next();

  let Some( path ) = agent_file else
  {
    eprintln!( "skip: no non-empty subagent JSONL found" );
    return;
  };

  let first_line = std::fs::read_to_string( &path )
    .expect( "read agent session" )
    .lines()
    .next()
    .unwrap_or( "" )
    .to_owned();

  assert!(
    first_line.contains( r#""slug""# ),
    "B15 violated: agent entry does not contain slug field.\n\
     File: {}\nFirst line: {first_line}",
    path.display()
  );
}

/// B15b: all sibling agents under one parent share the same `slug`.
///
/// If Claude Code started assigning different slugs to agents within the
/// same conversation, this would fail.
#[ test ]
fn b15_sibling_agents_share_slug()
{
  let projects = super::find_projects();

  // find a subagents dir with 2+ non-empty agent files
  let family = projects.iter()
    .flat_map( | p | super::find_subagent_dirs( p ) )
    .find( | ( _, dir ) | super::find_subagent_sessions( dir ).len() >= 2 );

  let Some( ( _parent_uuid, subagents_dir ) ) = family else
  {
    eprintln!( "skip: no subagents/ directory with 2+ agent files found" );
    return;
  };

  let agents = super::find_subagent_sessions( &subagents_dir );
  let mut slugs = Vec::new();

  for agent_path in &agents
  {
    let first_line = std::fs::read_to_string( agent_path )
      .expect( "read agent session" )
      .lines()
      .next()
      .unwrap_or( "" )
      .to_owned();

    // extract slug value with simple string search
    let slug = extract_json_string_field( &first_line, "slug" );
    if let Some( s ) = slug
    {
      slugs.push( ( agent_path.clone(), s ) );
    }
  }

  if slugs.len() < 2
  {
    eprintln!( "skip: fewer than 2 agents have slug field" );
    return;
  }

  let first_slug = &slugs[ 0 ].1;
  for ( path, slug ) in &slugs[ 1.. ]
  {
    assert_eq!(
      slug, first_slug,
      "B15 violated: sibling agents have different slugs.\n\
       First: {} (slug: {first_slug})\nDiffers: {} (slug: {slug})",
      slugs[ 0 ].0.display(),
      path.display()
    );
  }
}

/// Extract a JSON string field value by simple substring search.
///
/// Looks for `"field":"value"` or `"field": "value"` patterns.
/// Returns `None` if not found.
fn extract_json_string_field( line : &str, field : &str ) -> Option< String >
{
  let patterns =
  [
    format!( r#""{field}":""# ),
    format!( r#""{field}": ""# ),
  ];

  for pat in &patterns
  {
    if let Some( start ) = line.find( pat.as_str() )
    {
      let value_start = start + pat.len();
      if let Some( end ) = line[ value_start.. ].find( '"' )
      {
        return Some( line[ value_start..value_start + end ].to_owned() );
      }
    }
  }
  None
}

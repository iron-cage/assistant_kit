#![ allow( clippy::doc_markdown ) ]
//! B37: subagent API requests write the 5-minute prompt-cache tier; the main
//! conversation writes the 1-hour tier on subscription.
//!
//! Every `cache_creation` object in session JSONL splits cache writes into
//! `ephemeral_5m_input_tokens` and `ephemeral_1h_input_tokens`. Agent tool
//! subagents (plain-hex agent IDs, not forks) must only ever populate the
//! 5-minute field. Two documented exceptions inherit the parent conversation's
//! tier instead of starting their own: fork agents (`"isFork":true` in the
//! `.meta.json` sidecar) and typed-prefix system sidechains (e.g.
//! `agent-acompact-…`), which operate on the parent conversation's cache and
//! may therefore carry 1-hour writes.

/// B37a: plain-hex, non-fork agent transcripts never write the 1-hour cache tier.
///
/// Scans every agent transcript in real storage (hierarchical `{uuid}/subagents/`
/// plus old-format flat `agent-*.jsonl`), excluding the two documented
/// parent-tier-inheriting exceptions: fork agents and typed-prefix system
/// sidechains. If Claude Code granted regular subagents the 1-hour TTL, or the
/// tier accounting moved, violations would accumulate in storage and this would
/// go RED.
#[ test ]
fn b37_plain_agent_transcripts_never_write_1h_tier()
{
  let projects = super::find_projects();
  if projects.is_empty()
  {
    eprintln!( "skip: no ~/.claude/projects/ found" );
    return;
  }

  let mut files_scanned = 0_u64;
  let mut entries = 0_u64;
  let mut five_m_writes = 0_u64;
  let mut excluded_forks = 0_u64;
  let mut excluded_typed = 0_u64;
  let mut violations : Vec< ( std::path::PathBuf, u64 ) > = Vec::new();

  for project in &projects
  {
    let mut agent_files = super::find_agent_sessions( project );
    for ( _, dir ) in super::find_subagent_dirs( project )
    {
      agent_files.extend( super::find_subagent_sessions( &dir ) );
    }

    for agent in agent_files
    {
      let Some( id ) = agent_id( &agent ) else { continue };
      if !id.chars().all( | c | c.is_ascii_hexdigit() )
      {
        excluded_typed += 1;
        continue;
      }
      if is_fork( &agent )
      {
        excluded_forks += 1;
        continue;
      }
      files_scanned += 1;
      for ( five_m, one_h ) in cache_creation_pairs( &agent )
      {
        entries += 1;
        if five_m > 0 { five_m_writes += 1; }
        if one_h > 0 { violations.push( ( agent.clone(), one_h ) ); }
      }
    }
  }

  if five_m_writes == 0
  {
    eprintln!(
      "skip: no 5-minute cache writes found in any agent transcript \
       (cache_creation fields absent — storage may predate v2.0.25)"
    );
    return;
  }

  eprintln!(
    "B37 scan: {files_scanned} plain agent files, {entries} cache_creation entries, \
     {five_m_writes} 5m writes; excluded {excluded_forks} forks + {excluded_typed} typed-prefix sidechains"
  );

  assert!(
    violations.is_empty(),
    "B37 violated: {} plain (non-fork) agent transcript entries carry ephemeral_1h_input_tokens > 0.\n\
     Subagents are documented to use the 5-minute TTL tier exclusively; the 1-hour tier is\n\
     reserved for the main conversation (and inherited by forks / system sidechains).\n\
     First violations:\n{}",
    violations.len(),
    violations.iter().take( 5 )
      .map( | ( p, v ) | format!( "  {} (ephemeral_1h={v})", p.display() ) )
      .collect::< Vec< _ > >()
      .join( "\n" )
  );
}

/// B37b: main sessions write the 1-hour tier on subscription (observation).
///
/// The counterpart signature of the tier split: on a subscription machine the
/// main conversation writes `ephemeral_1h_input_tokens > 0`. When usage draws on
/// extra usage credits instead, the main conversation legitimately drops to the
/// 5-minute tier — that documented mode is indistinguishable from a regression
/// here, so absence is reported as a skip, never a failure (observation only).
#[ test ]
fn b37_main_sessions_write_1h_tier_on_subscription()
{
  let projects = super::find_projects();

  let mut any_cache_data = false;
  let mut found_1h : Option< std::path::PathBuf > = None;

  'outer : for project in &projects
  {
    for session in super::find_sessions( project )
    {
      for ( _, one_h ) in cache_creation_pairs( &session )
      {
        any_cache_data = true;
        if one_h > 0
        {
          found_1h = Some( session.clone() );
          break 'outer;
        }
      }
    }
  }

  if !any_cache_data
  {
    eprintln!( "skip: no cache_creation data found in any main session" );
    return;
  }

  match found_1h
  {
    Some( session ) => eprintln!(
      "B37 asymmetry confirmed: main session {} carries ephemeral_1h_input_tokens > 0",
      session.display()
    ),
    None => eprintln!(
      "skip: no 1-hour main-session writes found — machine may be running on \
       usage credits (documented 5-minute fallback for the main conversation)"
    ),
  }
}

// ---------------------------------------------------------------------------
// Local helpers
// ---------------------------------------------------------------------------

/// Extract `( ephemeral_5m, ephemeral_1h )` from every `cache_creation` object
/// in a JSONL file. Entries from versions predating the field yield nothing.
fn cache_creation_pairs( path : &std::path::Path ) -> Vec< ( u64, u64 ) >
{
  let Ok( text ) = std::fs::read_to_string( path ) else { return vec![] };
  let marker = "\"cache_creation\":{";
  let mut pairs = Vec::new();
  let mut rest = text.as_str();
  while let Some( start ) = rest.find( marker )
  {
    let tail = &rest[ start + marker.len().. ];
    let Some( end ) = tail.find( '}' ) else { break };
    let object = &tail[ ..end ];
    let five_m = field_u64( object, "\"ephemeral_5m_input_tokens\":" ).unwrap_or( 0 );
    let one_h = field_u64( object, "\"ephemeral_1h_input_tokens\":" ).unwrap_or( 0 );
    pairs.push( ( five_m, one_h ) );
    rest = &tail[ end.. ];
  }
  pairs
}

/// Parse the unsigned integer immediately following `label` inside `object`.
fn field_u64( object : &str, label : &str ) -> Option< u64 >
{
  let pos = object.find( label )? + label.len();
  let digits : String = object[ pos.. ]
    .chars()
    .take_while( char::is_ascii_digit )
    .collect();
  digits.parse().ok()
}

/// Agent ID from an `agent-{id}.jsonl` filename; `None` when the name doesn't match.
fn agent_id( path : &std::path::Path ) -> Option< String >
{
  let name = path.file_name()?.to_str()?;
  let id = name.strip_prefix( "agent-" )?.strip_suffix( ".jsonl" )?;
  Some( id.to_owned() )
}

/// `true` when the sibling `.meta.json` marks this agent as a fork.
///
/// Forks inherit the parent conversation — and with it the parent's cache tier —
/// so a fork of a main conversation legitimately carries 1-hour writes, while a
/// fork spawned by a subagent stays on the 5-minute tier.
fn is_fork( agent_path : &std::path::Path ) -> bool
{
  let Some( name ) = agent_path.file_name().and_then( | n | n.to_str() ) else { return false };
  let meta_name = name.replace( ".jsonl", ".meta.json" );
  let meta_path = agent_path.with_file_name( meta_name );
  let Ok( meta ) = std::fs::read_to_string( meta_path ) else { return false };
  meta.contains( "\"isFork\":true" ) || meta.contains( "\"agentType\":\"fork\"" )
}

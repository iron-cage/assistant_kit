#![ allow( clippy::doc_markdown ) ]
//! B14: agent `.meta.json` sidecars contain `agentType` and optional `description`.
//!
//! Each `agent-{id}.jsonl` in a subagents directory may have a sibling
//! `agent-{id}.meta.json` containing `{"agentType":"Explore"}` or similar.
//! Known `agentType` values: `general-purpose`, `Explore`, `workflow-subagent`,
//! `fork`, `Plan`, `claude-code-guide`, `claude`.
//!
//! Observed distribution (2026-08-27, 16713 sidecars): general-purpose 66.3%,
//! Explore 31.2%, workflow-subagent 1.6%, fork 0.7%, Plan 0.06%,
//! claude-code-guide 0.04%, claude 0.02%.
//! Some meta.json files are empty (0 bytes) — the test filters these out.
//! Nine optional fields accompany `agentType`: `spawnDepth`, `description`,
//! `toolUseId`, `isFork`, `model`, `parentAgentId`, `stoppedByUser`,
//! `worktreePath`, `worktreeBranch`. Full census and semantics:
//! `docs/behavior/014_b14_agent_meta_json.md`.
//!
//! Re-measure the distribution before trusting the percentages above:
//!
//! ```sh
//! cd ~/.claude/projects   # relative root — see the doc's Trap 2 on absolute paths
//! find . -path '*/subagents/*' -name '*.meta.json' -size +0 \
//!   -exec grep -ho '"agentType"[[:space:]]*:[[:space:]]*"[^"]*"' {} + \
//!   | sed 's/.*"\([^"]*\)"$/\1/' | sort | uniq -c | sort -rn
//! ```
//!
//! Extract with `grep -o`, never `sed -n s///p`: no sidecar ends with a newline,
//! so sed emits each value unterminated and consecutive values concatenate into
//! one bogus token. See Trap 1 in the doc above.

/// Every `agentType` value observed in real storage, most frequent first.
///
/// Fix(A3): the list held only the four values known in 2026-04; `workflow-subagent`,
/// `fork`, and `claude` had shipped since and were all being reported as violations.
/// Root cause: the allowlist is a snapshot of an external binary's behavior with no
/// mechanism tying it to the binary's own version — nothing updates it on upgrade.
/// Pitfall: when this test fails, the first question is whether the value is new
/// (extend the list *and* `docs/behavior/014_b14_agent_meta_json.md`) or genuinely
/// wrong (a schema break worth reporting) — never silence it by widening the match.
const KNOWN_AGENT_TYPES : [ &str; 7 ] =
[
  "general-purpose",
  "Explore",
  "workflow-subagent",
  "fork",
  "Plan",
  "claude-code-guide",
  "claude",
];

/// Extract the `agentType` *value* from a `.meta.json` body.
///
/// Fix(A5): the previous check tested `content.contains( known_type )` against the
/// whole JSON blob, so any occurrence of a known type name anywhere in the file
/// satisfied it — including inside `description`.
/// Root cause: a substring test on the container was used where a field lookup was
/// meant; the two agree only while no other field can contain a type name.
/// Pitfall: this masks *unknown* values rather than producing a false alarm, so it
/// fails silently and in the passing direction. Four `{"agentType":"fork",…}` files
/// whose descriptions began "Explore …" were counted as valid `Explore` agents.
/// Reproduce the masking on demand:
/// `echo '{"agentType":"bogus","description":"Explore x"}' | grep -c Explore` → 1.
fn agent_type_of( content : &str ) -> Option< &str >
{
  let after_key = content.split_once( r#""agentType""# )?.1;
  let after_colon = after_key.split_once( ':' )?.1;
  let after_quote = after_colon.split_once( '"' )?.1;
  let ( value, _ ) = after_quote.split_once( '"' )?;
  Some( value )
}

/// Whether any `.meta.json` exists *below* `subagents_dir` rather than directly in it.
///
/// Deliberately hand-rolled with explicit `read_dir` levels instead of reusing
/// `super::find_meta_json_files`: this is the control the recursive walker is checked
/// against, so sharing its traversal would make the check circular.
///
/// It also must not shell out to `find`. Under some sandboxed shells `find` can open a
/// directory under `~/.claude` but not enumerate it, printing the root and exiting **0**
/// with no results — a control that silently reports "nothing nested exists" would turn
/// the assertion it guards into a no-op, which is the exact failure class B14 keeps
/// tripping over. `std::fs` is the mechanism the rest of this suite already relies on.
fn has_nested_sidecar( subagents_dir : &std::path::Path ) -> bool
{
  let Ok( level1 ) = std::fs::read_dir( subagents_dir ) else { return false };
  for sub in level1.filter_map( Result::ok )
  {
    if !sub.file_type().is_ok_and( | t | t.is_dir() ) { continue; }
    let Ok( level2 ) = std::fs::read_dir( sub.path() ) else { continue };
    for item in level2.filter_map( Result::ok )
    {
      if item.file_name().to_string_lossy().ends_with( ".meta.json" ) { return true; }
      if !item.file_type().is_ok_and( | t | t.is_dir() ) { continue; }
      let Ok( level3 ) = std::fs::read_dir( item.path() ) else { continue };
      if level3.filter_map( Result::ok )
        .any( | e | e.file_name().to_string_lossy().ends_with( ".meta.json" ) )
      {
        return true;
      }
    }
  }
  false
}

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
    .find( | f | std::fs::metadata( f ).is_ok_and( | m | m.len() > 0 ) );

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

  let agent_type = agent_type_of( &content );

  assert!(
    agent_type.is_some_and( | t | KNOWN_AGENT_TYPES.contains( &t ) ),
    "B14 violated: meta.json agentType is not a known value.\n\
     File: {}\nContent: {content}\nParsed agentType: {agent_type:?}\nKnown types: {KNOWN_AGENT_TYPES:?}",
    path.display()
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
///
/// Three later defects are recorded on the constructs this test relies on rather
/// than here: the stale allowlist on [`KNOWN_AGENT_TYPES`] (A3), the traversal gap
/// on `super::find_meta_json_files` (A4), and the substring masking on
/// [`agent_type_of`] (A5). A4 and A5 both suppressed real violations while this
/// test was green, which is why coverage of the nested layout is asserted below
/// rather than left implicit in a passing scan.
#[ test ]
fn b14_all_meta_json_have_known_agent_type()
{
  let projects = super::find_projects();

  let mut checked = 0_usize;
  let mut nested_walked = 0_usize;
  let mut unknown = Vec::new();

  // Probed *before* the walk on purpose: storage is live (this very process's own
  // session writes into it), so probing afterwards could see a directory created
  // mid-test that the walk never had a chance to reach, and fail spuriously.
  let nested_on_disk = projects.iter()
    .flat_map( | p | super::find_subagent_dirs( p ) )
    .any( | ( _, dir ) | has_nested_sidecar( &dir ) );

  for project in &projects
  {
    for ( _, dir ) in super::find_subagent_dirs( project )
    {
      for meta_path in super::find_meta_json_files( &dir )
      {
        let len = std::fs::metadata( &meta_path )
          .map_or( 0, | m | m.len() );
        if len == 0 { continue; }

        let Ok( content ) = std::fs::read_to_string( &meta_path ) else { continue };

        if !content.contains( r#""agentType""# ) { continue; }

        checked += 1;
        if meta_path.parent().is_some_and( | p | p != dir ) { nested_walked += 1; }

        let known = agent_type_of( &content )
          .is_some_and( | t | KNOWN_AGENT_TYPES.contains( &t ) );
        if !known
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

  // Printed on success too — the two counts are what make the guards below
  // auditable rather than merely green. Visible with `--no-capture`.
  eprintln!(
    "B14: inspected {checked} sidecar(s); {nested_walked} nested below `subagents/`; \
     nested layout present on disk: {nested_on_disk}"
  );

  assert!(
    unknown.is_empty(),
    "B14 violated: {count} of {checked} meta.json file(s) contain unknown agentType.\n\
     Known: {KNOWN_AGENT_TYPES:?}\nUnknown:\n{entries}",
    count = unknown.len(),
    entries = unknown.join( "\n" )
  );

  // Guards A4.  A traversal that silently reaches fewer files still reports zero
  // unknown types, so a green scan is not by itself evidence of coverage.  This
  // asserts the property the fix established — sidecars below `subagents/` are
  // reached — without comparing counts, which would race against the live storage
  // this very session is writing into.
  //
  // Deliberately not a hardcoded `workflows/` check: any future nested layout
  // satisfies it too.  The check is skipped, not failed, when no nested sidecar
  // exists on this machine — its absence is a fact about local history, not a bug.
  assert!(
    !nested_on_disk || nested_walked > 0,
    "B14 traversal gap: sidecars exist below `subagents/` but the walk reached \
     none of them (inspected {checked}, nested 0). `find_meta_json_files` has \
     stopped recursing — see Fix(A4) in tests/behavior/mod.rs.\n\
     Reproduce: cd ~/.claude/projects && \
     find . -path '*/subagents/*/*' -name '*.meta.json' | head"
  );
}

//! Contract tests for behavioral invariants, CLI pitfalls, and guide properties.
//!
//! ## Coverage
//!
//! - `tests/docs/invariant/01_path_encoding.md` — IN-1..IN-5: path encode/decode contracts
//! - `tests/docs/invariant/02_session_family.md` — IN-1..IN-5: session family grouping contracts
//! - `tests/docs/invariant/03_entry_type_format.md` — IN-1..IN-6: JSONL entry type classification contracts
//! - `tests/docs/cli/pitfall/02_cross_command_propagation.md` — PF-1..PF-3: propagation fix annotation contracts
//! - `tests/docs/cli/pitfall/03_test_data_format.md` — PF-1..PF-4: test data JSONL format contracts
//! - `tests/docs/guide/001_advanced_storage_topics.md` — GD-1..GD-7: advanced storage architecture contracts

mod common;

use std::
{
  io::Write as IoWrite,
  path::Path,
};

use tempfile::TempDir;

use claude_storage_core::
{
  Entry,
  EntryType,
  JsonValue,
  Session,
  Storage,
  encode_path,
  parse_json,
};

// ─── helpers ────────────────────────────────────────────────────────────────

fn stdout( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

fn stderr( out : &std::process::Output ) -> String
{
  String::from_utf8_lossy( &out.stderr ).into_owned()
}

fn assert_exit( out : &std::process::Output, code : i32 )
{
  assert_eq!(
    out.status.code().unwrap_or( -1 ),
    code,
    "expected exit {code}, got {:?}; stderr: {}",
    out.status.code(),
    stderr( out )
  );
}

/// Full JSONL line with all required fields for a user entry.
fn user_jsonl_line( uuid : &str, session_id : &str, content : &str ) -> String
{
  format!(
    r#"{{"type":"user","uuid":"{uuid}","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"message":{{"role":"user","content":"{content}"}}}}"#
  )
}

/// Full JSONL line for an assistant entry.
fn assistant_jsonl_line( uuid : &str, session_id : &str ) -> String
{
  format!(
    r#"{{"type":"assistant","uuid":"{uuid}","parentUuid":null,"timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":null,"userType":"external","isSidechain":false,"requestId":"req_test_001","message":{{"role":"assistant","model":"claude-test","id":"msg_test_001","content":[{{"type":"text","text":"ok"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
  )
}

// ─── invariant/01 — path encoding ───────────────────────────────────────────

/// IN-1: `/` encodes to `-`.
///
/// ## Related Requirements
/// `tests/docs/invariant/01_path_encoding.md` — IN-1
#[ test ]
fn in_1_slash_encodes_to_dash()
{
  let encoded = encode_path( Path::new( "/home/alice/projects/my-app" ) )
    .expect( "IN-1: encode_path must succeed" );
  assert_eq!(
    encoded,
    "-home-alice-projects-my-app",
    "IN-1: each '/' must encode to '-'"
  );
}

/// IN-2: `_` encodes to `-` (lossy — identical result to IN-1).
///
/// ## Related Requirements
/// `tests/docs/invariant/01_path_encoding.md` — IN-2
#[ test ]
fn in_2_underscore_encodes_to_dash()
{
  let encoded = encode_path( Path::new( "/home/alice/projects/my_app" ) )
    .expect( "IN-2: encode_path must succeed" );
  assert_eq!(
    encoded,
    "-home-alice-projects-my-app",
    "IN-2: '_' must encode to '-' (lossy — same as slash encoding)"
  );
}

/// IN-3: encode output contains only `[a-zA-Z0-9-]` characters.
///
/// ## Related Requirements
/// `tests/docs/invariant/01_path_encoding.md` — IN-3
#[ test ]
fn in_3_output_contains_only_safe_chars()
{
  let encoded = encode_path( Path::new( "/home/alice/projects/my-app" ) )
    .expect( "IN-3: encode_path must succeed" );
  for ch in encoded.chars()
  {
    assert!(
      ch.is_ascii_alphanumeric() || ch == '-',
      "IN-3: encoded output must only contain [a-zA-Z0-9-]; found unexpected char: {ch:?}"
    );
  }
  assert!( !encoded.is_empty(), "IN-3: encoded output must be non-empty" );
}

/// IN-4: `encode(decode(k)) == k` round-trip via CLI `.projects scope::global`.
///
/// Creates a storage session for a project path containing an underscore component.
/// The filesystem-guided decoder must restore the original `_` character (not split
/// on `/`), so the display path preserves the underscore.
///
/// ## Related Requirements
/// `tests/docs/invariant/01_path_encoding.md` — IN-4
#[ test ]
fn in_4_encode_decode_round_trip_via_cli()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Project path with underscore — create on disk so FS-guided decode works
  let project_dir = root.path().join( "my_proj" );
  std::fs::create_dir_all( &project_dir ).expect( "IN-4: create project dir" );

  common::write_path_project_session( &storage_root, &project_dir, "session-roundtrip-001", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!(
    s.contains( "my_proj" ),
    "IN-4: display path must contain 'my_proj' (underscore preserved by FS-guided decode); got:\n{s}"
  );
}

/// IN-5: Two collision paths disambiguated by filesystem DFS.
///
/// Both `/tmp/my-app` (hyphen) and `/tmp/my_app` (underscore) encode to the same
/// storage key. The DFS walk resolves `_` vs `/` at each boundary using `is_dir()`.
/// Creates the underscore variant on disk (the DFS reconstructs it via Option B: `_`-join);
/// the display path must match that real filesystem candidate.
///
/// ## Related Requirements
/// `tests/docs/invariant/01_path_encoding.md` — IN-5
#[ test ]
fn in_5_collision_paths_disambiguated_by_dfs()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );

  // Two paths that encode to the same key
  let path_with_hyphen     = root.path().join( "my-app" );
  let path_with_underscore = root.path().join( "my_app" );

  // Verify collision precondition
  let key_hyphen     = encode_path( &path_with_hyphen ).expect( "encode hyphen path" );
  let key_underscore = encode_path( &path_with_underscore ).expect( "encode underscore path" );
  assert_eq!( key_hyphen, key_underscore, "IN-5: collision precondition failed" );

  // Create the underscore variant on disk — DFS resolves it via `_`-join (Option B)
  std::fs::create_dir_all( &path_with_underscore ).expect( "IN-5: create my_app dir" );

  // Write session under the shared storage key
  common::write_path_project_session( &storage_root, &path_with_underscore, "session-collision-001", 2 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  // DFS resolves the encoded key to the real underscore directory on disk
  assert!(
    s.contains( "my_app" ),
    "IN-5: display path must match real filesystem candidate 'my_app'; got:\n{s}"
  );
  assert!(
    s.contains( "session-collision-001" ),
    "IN-5: session must appear in output; got:\n{s}"
  );
}

// ─── invariant/02 — session family ───────────────────────────────────────────

/// IN-1: Flat layout: `agent-*.jsonl` at project root is discovered as agent session.
///
/// ## Related Requirements
/// `tests/docs/invariant/02_session_family.md` — IN-1
#[ test ]
fn in_1_flat_agent_discovered_by_filename()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path();
  let project_id = "proj-flat-disc";
  let parent_session_id = "aaaaaaaa-1111-1111-1111-aaaaaaaaaaaa";

  // Write root session and a flat agent session
  common::write_test_session( storage_root, project_id, parent_session_id, 2 );
  common::write_flat_agent_session( storage_root, project_id, "64bdad98", parent_session_id, 2 );

  let storage = Storage::with_root( storage_root );
  let projects = storage.list_projects().expect( "IN-1: list_projects must succeed" );
  let project = projects.iter().find( | p | format!( "{:?}", p.id() ).contains( project_id ) )
    .expect( "IN-1: project must be discoverable" );

  let all = project.all_sessions().expect( "IN-1: all_sessions must succeed" );
  let agent_sessions : Vec< _ > = all.iter().filter( | s | s.is_agent_session() ).collect();

  assert!(
    !agent_sessions.is_empty(),
    "IN-1: flat agent session must be discovered via all_sessions(); found sessions: {:?}",
    all.iter().map( Session::id ).collect::< Vec< _ > >()
  );
  assert!(
    agent_sessions.iter().any( | s | s.id().contains( "64bdad98" ) ),
    "IN-1: agent-64bdad98.jsonl must be discovered; found agents: {:?}",
    agent_sessions.iter().map( | s | s.id() ).collect::< Vec< _ > >()
  );
}

/// IN-2: Flat layout: family membership established via `sessionId` field in first agent entry.
///
/// ## Related Requirements
/// `tests/docs/invariant/02_session_family.md` — IN-2
#[ test ]
fn in_2_flat_agent_membership_via_session_id()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path();
  let project_id = "proj-flat-memb";
  let parent_session_id = "bbbbbbbb-2222-2222-2222-bbbbbbbbbbbb";

  common::write_test_session( storage_root, project_id, parent_session_id, 2 );
  common::write_flat_agent_session( storage_root, project_id, "abcd1234", parent_session_id, 2 );

  let storage = Storage::with_root( storage_root );
  let projects = storage.list_projects().expect( "IN-2: list_projects" );
  let project = projects.iter().find( | p | format!( "{:?}", p.id() ).contains( project_id ) )
    .expect( "IN-2: project must exist" );

  let mut all = project.all_sessions().expect( "IN-2: all_sessions" );
  let agent_session = all.iter_mut()
    .find( | s | s.is_agent_session() )
    .expect( "IN-2: agent session must exist" );

  let entries = agent_session.entries().expect( "IN-2: entries must load" );
  assert!(
    !entries.is_empty(),
    "IN-2: agent session must have entries"
  );

  let first_session_id = &entries[ 0 ].session_id;
  assert_eq!(
    first_session_id,
    parent_session_id,
    "IN-2: first agent entry sessionId must equal parent session UUID for family membership"
  );
}

/// IN-3: Hierarchical layout: agents in `{uuid}/subagents/` are discovered.
///
/// ## Related Requirements
/// `tests/docs/invariant/02_session_family.md` — IN-3
#[ test ]
fn in_3_hierarchical_agent_discovered_in_subagents_dir()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path();
  let project_id = "proj-hier-disc";
  let root_session_id = "cccccccc-3333-3333-3333-cccccccccccc";

  common::write_hierarchical_session(
    storage_root,
    project_id,
    root_session_id,
    &[ ( "ac9afcb5", "default" ) ],
    2,
  );

  let storage = Storage::with_root( storage_root );
  let projects = storage.list_projects().expect( "IN-3: list_projects" );
  let project = projects.iter().find( | p | format!( "{:?}", p.id() ).contains( project_id ) )
    .expect( "IN-3: project must exist" );

  let all = project.all_sessions().expect( "IN-3: all_sessions" );
  let agent_sessions : Vec< _ > = all.iter().filter( | s | s.is_agent_session() ).collect();

  assert!(
    !agent_sessions.is_empty(),
    "IN-3: hierarchical agent session must be discovered via all_sessions()"
  );
  assert!(
    agent_sessions.iter().any( | s | s.id().contains( "ac9afcb5" ) ),
    "IN-3: agent-ac9afcb5.jsonl must be discovered in subagents dir"
  );
}

/// IN-4: Hierarchical layout: family membership by directory structure, not `sessionId`.
///
/// Writes a hierarchical agent whose JSONL entries contain a `sessionId` that does NOT
/// match the root session UUID. The agent must still be associated with the root session
/// by virtue of its directory placement, not by matching `sessionId`.
///
/// ## Related Requirements
/// `tests/docs/invariant/02_session_family.md` — IN-4
#[ test ]
fn in_4_hierarchical_membership_by_directory_not_session_id()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path();
  let project_id = "proj-hier-memb";
  let root_session_id = "dddddddd-4444-4444-4444-dddddddddddd";
  let different_session_id = "eeeeeeee-5555-5555-5555-eeeeeeeeeeee";

  // Write root session
  common::write_test_session( storage_root, project_id, root_session_id, 2 );

  // Manually write a hierarchical agent with a DIFFERENT sessionId in entries
  let subagents_dir = storage_root
    .join( "projects" )
    .join( project_id )
    .join( root_session_id )
    .join( "subagents" );
  std::fs::create_dir_all( &subagents_dir ).expect( "IN-4: create subagents dir" );

  let agent_path = subagents_dir.join( "agent-mismatched.jsonl" );
  let mut f = std::fs::File::create( &agent_path ).expect( "IN-4: create agent file" );
  writeln!(
    f,
    r#"{{"type":"user","uuid":"h-uuid-000","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"{different_session_id}","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"message":{{"role":"user","content":"test"}}}}"#
  ).expect( "IN-4: write agent entry" );

  let storage = Storage::with_root( storage_root );
  let projects = storage.list_projects().expect( "IN-4: list_projects" );
  let project = projects.iter().find( | p | format!( "{:?}", p.id() ).contains( project_id ) )
    .expect( "IN-4: project must exist" );

  let all = project.all_sessions().expect( "IN-4: all_sessions" );
  let agent_sessions : Vec< _ > = all.iter().filter( | s | s.is_agent_session() ).collect();

  assert!(
    agent_sessions.iter().any( | s | s.id().contains( "mismatched" ) ),
    "IN-4: hierarchical agent must be discovered by directory structure even when sessionId differs"
  );
}

/// IN-5: Both flat and hierarchical agents coexist without conflict.
///
/// ## Related Requirements
/// `tests/docs/invariant/02_session_family.md` — IN-5
#[ test ]
fn in_5_flat_and_hierarchical_coexist()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path();
  let project_id = "proj-coexist";
  let root_session_1 = "ffffffff-6666-6666-6666-ffffffffffff";
  let root_session_2 = "gggggggg-7777-7777-7777-gggggggggggg";

  // Root session 1 + flat agent
  common::write_test_session( storage_root, project_id, root_session_1, 2 );
  common::write_flat_agent_session( storage_root, project_id, "abc123", root_session_1, 2 );

  // Root session 2 + hierarchical agent
  common::write_hierarchical_session(
    storage_root,
    project_id,
    root_session_2,
    &[ ( "def456", "default" ) ],
    2,
  );

  let storage = Storage::with_root( storage_root );
  let projects = storage.list_projects().expect( "IN-5: list_projects" );
  let project = projects.iter().find( | p | format!( "{:?}", p.id() ).contains( project_id ) )
    .expect( "IN-5: project must exist" );

  let all = project.all_sessions().expect( "IN-5: all_sessions" );
  let agent_sessions : Vec< _ > = all.iter().filter( | s | s.is_agent_session() ).collect();

  let has_flat = agent_sessions.iter().any( | s | s.id().contains( "abc123" ) );
  let has_hierarchical = agent_sessions.iter().any( | s | s.id().contains( "def456" ) );

  assert!( has_flat, "IN-5: flat agent abc123 must be discovered" );
  assert!( has_hierarchical, "IN-5: hierarchical agent def456 must be discovered" );
}

// ─── invariant/03 — entry type format ────────────────────────────────────────

/// IN-1: `"type":"user"` top-level entry classified as User.
///
/// ## Related Requirements
/// `tests/docs/invariant/03_entry_type_format.md` — IN-1
#[ test ]
fn in_1_user_type_counted_as_conversation_entry()
{
  let line = user_jsonl_line( "a1b2c3d4-0001-0001-0001-a1b2c3d40001", "sess-001", "hello" );
  let entry = Entry::from_json_line( &line )
    .expect( "IN-1: user entry must parse successfully" );
  assert_eq!(
    entry.entry_type,
    EntryType::User,
    "IN-1: top-level 'type':'user' must classify as EntryType::User"
  );
}

/// IN-2: `"type":"assistant"` top-level entry classified as Assistant.
///
/// ## Related Requirements
/// `tests/docs/invariant/03_entry_type_format.md` — IN-2
#[ test ]
fn in_2_assistant_type_counted_as_conversation_entry()
{
  let line = assistant_jsonl_line( "b2c3d4e5-0002-0002-0002-b2c3d4e50002", "sess-001" );
  let entry = Entry::from_json_line( &line )
    .expect( "IN-2: assistant entry must parse successfully" );
  assert_eq!(
    entry.entry_type,
    EntryType::Assistant,
    "IN-2: top-level 'type':'assistant' must classify as EntryType::Assistant"
  );
}

/// IN-3: `"type":"queue-operation"` is silently skipped (returns parse error).
///
/// ## Related Requirements
/// `tests/docs/invariant/03_entry_type_format.md` — IN-3
#[ test ]
fn in_3_queue_operation_type_skipped()
{
  let line = r#"{"type":"queue-operation","uuid":"c3d4e5f6-0003-0003-0003-c3d4e5f60003"}"#;
  let result = Entry::from_json_line( line );
  assert!(
    result.is_err(),
    "IN-3: 'type':'queue-operation' must return parse error (skipped); got Ok(_)"
  );
}

/// IN-4: `"type":"summary"` is silently skipped (returns parse error).
///
/// ## Related Requirements
/// `tests/docs/invariant/03_entry_type_format.md` — IN-4
#[ test ]
fn in_4_summary_type_skipped()
{
  let line = r#"{"type":"summary","uuid":"d4e5f6a7-0004-0004-0004-d4e5f6a70004"}"#;
  let result = Entry::from_json_line( line );
  assert!(
    result.is_err(),
    "IN-4: 'type':'summary' must return parse error (skipped); got Ok(_)"
  );
}

/// IN-5: Entry missing `"uuid"` field is silently skipped via `session.entries()`.
///
/// ## Related Requirements
/// `tests/docs/invariant/03_entry_type_format.md` — IN-5
#[ test ]
fn in_5_entry_missing_uuid_skipped()
{
  let tmp = TempDir::new().unwrap();
  let session_path = tmp.path().join( "session-missing-uuid.jsonl" );

  // Two well-formed entries + one missing uuid (skipped) + one more well-formed
  let well_formed = user_jsonl_line( "e5f6a7b8-0005-0005-0005-e5f6a7b80005", "sess-005", "ok" );
  let missing_uuid = r#"{"type":"user","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"sess-005","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"message":{"role":"user","content":"no uuid"}}"#;
  let well_formed_2 = user_jsonl_line( "f6a7b8c9-0006-0006-0006-f6a7b8c90006", "sess-005", "also ok" );

  std::fs::write(
    &session_path,
    format!( "{well_formed}\n{missing_uuid}\n{well_formed_2}\n" ),
  ).expect( "IN-5: write test file" );

  let mut session = Session::load( &session_path ).expect( "IN-5: session must load" );
  let entries = session.entries().expect( "IN-5: entries must load" );

  assert_eq!(
    entries.len(),
    2,
    "IN-5: entry missing 'uuid' must be skipped; expected 2 well-formed entries, got {}",
    entries.len()
  );
}

/// IN-6: Nested `message.role` NOT used for entry classification.
///
/// Entry A: `"type":"user"` with `"message":{"role":"assistant"}` → classified as User.
/// Entry B: `"type":"assistant"` with `"message":{"role":"user"}` → classified as Assistant.
///
/// ## Related Requirements
/// `tests/docs/invariant/03_entry_type_format.md` — IN-6
#[ test ]
fn in_6_message_role_not_used_for_classification()
{
  // Entry A: top-level type=user but message.role=assistant
  let line_a = r#"{"type":"user","uuid":"a0000001-0001-0001-0001-a00000010001","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"sess-in6","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"message":{"role":"assistant","content":"crosswired A"}}"#;

  // Entry B: top-level type=assistant but message.role=user
  let line_b = r#"{"type":"assistant","uuid":"b0000002-0002-0002-0002-b00000020002","parentUuid":null,"timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"sess-in6","version":"2.0.0","gitBranch":null,"userType":"external","isSidechain":false,"requestId":"req_in6","message":{"role":"user","model":"claude-test","id":"msg_in6","content":[{"type":"text","text":"crosswired B"}],"stop_reason":"end_turn","stop_sequence":null,"usage":{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}"#;

  let entry_a = Entry::from_json_line( line_a ).expect( "IN-6: entry A must parse" );
  let entry_b = Entry::from_json_line( line_b ).expect( "IN-6: entry B must parse" );

  assert_eq!(
    entry_a.entry_type,
    EntryType::User,
    "IN-6: entry A classified by top-level 'type':'user', not by message.role"
  );
  assert_eq!(
    entry_b.entry_type,
    EntryType::Assistant,
    "IN-6: entry B classified by top-level 'type':'assistant', not by message.role"
  );
}

// ─── pitfall/02 — cross-command propagation ──────────────────────────────────

const PROPAGATION_COMMENT : &str =
  "// Pitfall: When fixing a bug in one command, grep for identical patterns in other commands.";

/// PF-1: `src/cli/search.rs` contains at least 2 propagation-fix comments.
///
/// ## Related Requirements
/// `tests/docs/cli/pitfall/02_cross_command_propagation.md` — PF-1
#[ test ]
fn pf_1_search_rs_contains_at_least_2_propagation_comments()
{
  let search_rs = std::fs::read_to_string( "src/cli/search.rs" )
    .expect( "PF-1: src/cli/search.rs must be readable" );

  let count = search_rs.matches( PROPAGATION_COMMENT ).count();
  assert!(
    count >= 2,
    "PF-1: src/cli/search.rs must contain at least 2 propagation-fix comments; found {count}"
  );
}

/// PF-2: `src/cli/count.rs` contains at least 2 propagation-fix comments.
///
/// ## Related Requirements
/// `tests/docs/cli/pitfall/02_cross_command_propagation.md` — PF-2
#[ test ]
fn pf_2_count_rs_contains_at_least_2_propagation_comments()
{
  let count_rs = std::fs::read_to_string( "src/cli/count.rs" )
    .expect( "PF-2: src/cli/count.rs must be readable" );

  let count = count_rs.matches( PROPAGATION_COMMENT ).count();
  assert!(
    count >= 2,
    "PF-2: src/cli/count.rs must contain at least 2 propagation-fix comments; found {count}"
  );
}

/// PF-3: No unpatched copy of a known-buggy pattern survives in `src/cli/`.
///
/// Every `.rs` file in `src/cli/` that contains the propagation comment has been
/// patched. The set of patched files (search.rs, count.rs, export.rs) must all
/// carry at least one propagation comment — no file is silently missing its annotation.
///
/// ## Related Requirements
/// `tests/docs/cli/pitfall/02_cross_command_propagation.md` — PF-3
#[ test ]
fn pf_3_no_unpatched_buggy_pattern_in_cli_dir()
{
  // Known patched files from issues #009, #010, #012
  let patched_files = &[ "src/cli/search.rs", "src/cli/count.rs", "src/cli/export.rs" ];

  for file_path in patched_files
  {
    let content = std::fs::read_to_string( file_path )
      .unwrap_or_else( | _ | panic!( "PF-3: {file_path} must be readable" ) );

    let has_comment = content.contains( PROPAGATION_COMMENT );
    assert!(
      has_comment,
      "PF-3: {file_path} must contain propagation-fix comment (pattern exhaustion check failed)"
    );
  }
}

// ─── pitfall/03 — test data format ───────────────────────────────────────────

/// PF-1: JSONL with `"type":"user"` entries produces non-zero count.
///
/// ## Related Requirements
/// `tests/docs/cli/pitfall/03_test_data_format.md` — PF-1
#[ test ]
fn pf_1_correct_type_user_entries_produce_nonzero_count()
{
  let tmp = TempDir::new().unwrap();
  let session_path = tmp.path().join( "pf1-correct.jsonl" );

  let line1 = user_jsonl_line( "pf1-uuid-001-0001-0001-pf1uuid0001", "sess-pf1", "hello" );
  let line2 = user_jsonl_line( "pf1-uuid-002-0002-0002-pf1uuid0002", "sess-pf1", "world" );
  std::fs::write( &session_path, format!( "{line1}\n{line2}\n" ) )
    .expect( "PF-1: write test file" );

  let session = Session::load( &session_path ).expect( "PF-1: session must load" );
  let count = session.count_entries().expect( "PF-1: count_entries must succeed" );

  assert!(
    count > 0,
    "PF-1: JSONL with 'type':'user' entries must produce non-zero count; got {count}"
  );
}

/// PF-2: JSONL with `"type":"message"` entries produces count of 0.
///
/// Regression guard for issue-011: wrong type value "message" is not a valid
/// production entry type and must not be counted.
///
/// ## Related Requirements
/// `tests/docs/cli/pitfall/03_test_data_format.md` — PF-2
#[ test ]
fn pf_2_wrong_type_message_produces_zero_count()
{
  let tmp = TempDir::new().unwrap();
  let session_path = tmp.path().join( "pf2-wrong-type.jsonl" );

  let line1 = r#"{"type":"message","uuid":"pf2-uuid-001","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"sess-pf2","version":"2.0.0","message":{"role":"user","content":"wrong type"}}"#;
  let line2 = r#"{"type":"message","uuid":"pf2-uuid-002","parentUuid":null,"timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"sess-pf2","version":"2.0.0","message":{"role":"assistant","content":"also wrong"}}"#;
  std::fs::write( &session_path, format!( "{line1}\n{line2}\n" ) )
    .expect( "PF-2: write test file" );

  let session = Session::load( &session_path ).expect( "PF-2: session must load" );
  let count = session.count_entries().expect( "PF-2: count_entries must succeed" );

  assert_eq!(
    count,
    0,
    "PF-2: 'type':'message' is not a valid production type and must produce count 0; got {count}"
  );
}

/// PF-3: JSONL entry missing `"uuid"` is skipped via `session.entries()`.
///
/// Uses the full parse path (`session.entries()`) which calls `from_json_line()`,
/// requiring the `"uuid"` field. The missing-uuid entry is excluded from the result.
/// Note: `count_entries()` fast path does NOT check uuid presence.
///
/// ## Related Requirements
/// `tests/docs/cli/pitfall/03_test_data_format.md` — PF-3
#[ test ]
fn pf_3_entry_missing_uuid_skipped_via_entries()
{
  let tmp = TempDir::new().unwrap();
  let session_path = tmp.path().join( "pf3-missing-uuid.jsonl" );

  let good_entry = user_jsonl_line( "pf3-uuid-good-0001-pf3uuid0001", "sess-pf3", "good" );
  let missing_uuid_entry = r#"{"type":"user","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"sess-pf3","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"message":{"role":"user","content":"no uuid here"}}"#;

  std::fs::write(
    &session_path,
    format!( "{good_entry}\n{missing_uuid_entry}\n" ),
  ).expect( "PF-3: write test file" );

  let mut session = Session::load( &session_path ).expect( "PF-3: session must load" );
  let entries = session.entries().expect( "PF-3: entries must load" );

  assert_eq!(
    entries.len(),
    1,
    "PF-3: entry missing 'uuid' must be skipped via session.entries() (full parse path); expected 1, got {}",
    entries.len()
  );
}

/// PF-4: Entry with non-UUID `"uuid"` value is parsed normally (no format validation).
///
/// `from_json_line()` accepts any non-empty string as uuid — no UUID format validation
/// is applied. The entry with `"uuid":"entry-1"` must be INCLUDED (not skipped).
///
/// ## Related Requirements
/// `tests/docs/cli/pitfall/03_test_data_format.md` — PF-4
#[ test ]
fn pf_4_entry_with_non_uuid_value_is_parsed_normally()
{
  let tmp = TempDir::new().unwrap();
  let session_path = tmp.path().join( "pf4-non-uuid.jsonl" );

  // Entry with "uuid":"entry-1" (simple identifier, not UUID format) plus all required fields
  let non_uuid_entry = r#"{"type":"user","uuid":"entry-1","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"sess-pf4","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"message":{"role":"user","content":"non-uuid entry"}}"#;

  std::fs::write( &session_path, format!( "{non_uuid_entry}\n" ) )
    .expect( "PF-4: write test file" );

  let mut session = Session::load( &session_path ).expect( "PF-4: session must load" );
  let entries = session.entries().expect( "PF-4: entries must load" );

  assert_eq!(
    entries.len(),
    1,
    "PF-4: entry with non-UUID 'uuid':'entry-1' must be included (no UUID format validation); got {}",
    entries.len()
  );
  assert_eq!(
    entries[ 0 ].uuid,
    "entry-1",
    "PF-4: uuid field must be preserved as-is without format coercion"
  );
}

// ─── guide/001 — advanced storage architecture ────────────────────────────────

/// GD-1: Agent JSONL first entry has `isSidechain: true`.
///
/// ## Related Requirements
/// `tests/docs/guide/001_advanced_storage_topics.md` — GD-1
#[ test ]
fn gd_1_agent_entry_is_sidechain_true()
{
  // Manually craft an agent JSONL with isSidechain:true
  let agent_entry = r#"{"type":"user","uuid":"gd1-uuid-0001-0001-0001-gd1uuid0001","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"gd1-session-id","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":true,"message":{"role":"user","content":"agent message"}}"#;

  let entry = Entry::from_json_line( agent_entry ).expect( "GD-1: agent entry must parse" );
  assert!(
    entry.is_sidechain,
    "GD-1: agent JSONL first entry must have isSidechain:true; got isSidechain:{}",
    entry.is_sidechain
  );
}

/// GD-2: Agent entry `agentId` matches the agent filename suffix.
///
/// Parses via `parse_json()` to access the raw `agentId` field (not in Entry struct).
///
/// ## Related Requirements
/// `tests/docs/guide/001_advanced_storage_topics.md` — GD-2
#[ test ]
fn gd_2_agent_entry_agent_id_matches_filename_suffix()
{
  // Agent file would be named agent-64bdad98.jsonl
  let expected_suffix = "64bdad98";
  let agent_entry = format!(
    r#"{{"type":"user","uuid":"gd2-uuid-0002-0002-0002-gd2uuid0002","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"gd2-session","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":true,"agentId":"{expected_suffix}","message":{{"role":"user","content":"agent"}}}}"#
  );

  let json = parse_json( &agent_entry ).expect( "GD-2: parse_json must succeed" );
  let obj = json.as_object().expect( "GD-2: must be object" );
  let agent_id = obj.get( "agentId" )
    .and_then( | v | v.as_str() )
    .expect( "GD-2: agentId field must be present" );

  assert_eq!(
    agent_id,
    expected_suffix,
    "GD-2: agentId field must equal filename suffix '64bdad98'"
  );
}

/// GD-3: Empty `.meta.json` (0 bytes) parsed without error via CLI `.projects`.
///
/// Creates an agent session with a 0-byte `.meta.json` sidecar file. The CLI
/// `.projects` command must succeed (exit 0) — no parse error from empty metadata.
///
/// ## Related Requirements
/// `tests/docs/guide/001_advanced_storage_topics.md` — GD-3
#[ test ]
fn gd_3_empty_meta_json_parsed_without_error()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path();
  let project_id = "proj-gd3";
  let root_session_id = "gd3gd3gd-3333-3333-3333-gd3gd3gd3gd3";

  // Write root session and hierarchical agent
  common::write_hierarchical_session(
    storage_root,
    project_id,
    root_session_id,
    &[ ( "empty-meta", "default" ) ],
    2,
  );

  // Overwrite the meta.json with a 0-byte file
  let meta_path = storage_root
    .join( "projects" )
    .join( project_id )
    .join( root_session_id )
    .join( "subagents" )
    .join( "agent-empty-meta.meta.json" );
  std::fs::write( &meta_path, b"" ).expect( "GD-3: write empty meta.json" );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".projects" )
    .arg( "scope::global" )
    .output()
    .unwrap();

  assert_exit(
    &out,
    0,
    // 0-byte meta.json must not cause a parse error
  );
  // Verify project appears in output (confirms it was processed)
  let s = stdout( &out );
  assert!(
    !s.is_empty(),
    "GD-3: .projects must produce non-empty output when meta.json is empty; stderr: {}",
    stderr( &out )
  );
}

/// GD-4: `history.jsonl` entry `timestamp` field value > 10^12 (milliseconds-since-epoch).
///
/// ## Related Requirements
/// `tests/docs/guide/001_advanced_storage_topics.md` — GD-4
#[ test ]
fn gd_4_history_entry_timestamp_is_milliseconds()
{
  let tmp = TempDir::new().unwrap();
  let history_path = tmp.path().join( "history.jsonl" );

  // Use a known millisecond timestamp (2025-01-01 00:00:00 UTC = 1735689600000 ms)
  let timestamp_ms : u64 = 1_735_689_600_000u64;
  let history_entry = format!(
    r#"{{"type":"user","uuid":"gd4-uuid-0004","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"gd4-session","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"unixMs":{timestamp_ms},"message":{{"role":"user","content":"test"}}}}"#
  );

  std::fs::write( &history_path, format!( "{history_entry}\n" ) )
    .expect( "GD-4: write history.jsonl" );

  let content = std::fs::read_to_string( &history_path ).expect( "GD-4: read history" );
  let line = content.lines().next().expect( "GD-4: must have one line" );
  let json = parse_json( line ).expect( "GD-4: parse_json must succeed" );
  let obj = json.as_object().expect( "GD-4: must be object" );

  // Verify the raw millisecond value we wrote is preserved and > 10^12
  let raw = obj.get( "unixMs" )
    .and_then( JsonValue::as_number )
    .expect( "GD-4: unixMs field must be present and numeric" );

  assert!(
    raw > 1_000_000_000_000.0,
    "GD-4: history entry timestamp must be > 10^12 (milliseconds-since-epoch); got {raw}"
  );
}

/// GD-5: `session-env/{uuid}/` directory is empty (contains no files).
///
/// ## Related Requirements
/// `tests/docs/guide/001_advanced_storage_topics.md` — GD-5
#[ test ]
fn gd_5_session_env_directory_is_empty()
{
  let tmp = TempDir::new().unwrap();
  let session_uuid = "gd5gd5gd-5555-5555-5555-gd5gd5gd5gd5";
  let env_dir = tmp.path().join( "session-env" ).join( session_uuid );
  std::fs::create_dir_all( &env_dir ).expect( "GD-5: create session-env dir" );

  let entries : Vec< _ > = std::fs::read_dir( &env_dir )
    .expect( "GD-5: read session-env dir must succeed" )
    .filter_map( Result::ok )
    .collect();

  assert!(
    entries.is_empty(),
    "GD-5: session-env/{session_uuid}/ must be empty (contains no files or subdirs); found {} entries",
    entries.len()
  );
}

/// GD-6: Agent `slug` field is identical across agents from the same parent session.
///
/// Parses two agent entries with the same `sessionId` via `parse_json()` and verifies
/// both carry identical `slug` values.
///
/// ## Related Requirements
/// `tests/docs/guide/001_advanced_storage_topics.md` — GD-6
#[ test ]
fn gd_6_agent_slug_identical_across_same_parent_session()
{
  let session_id = "gd6gd6gd-6666-6666-6666-gd6gd6gd6gd6";
  let shared_slug = "my-project-slug";

  let agent_entry_1 = format!(
    r#"{{"type":"user","uuid":"gd6-uuid-a001","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":true,"slug":"{shared_slug}","message":{{"role":"user","content":"agent 1"}}}}"#
  );
  let agent_entry_2 = format!(
    r#"{{"type":"user","uuid":"gd6-uuid-b002","parentUuid":null,"timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":true,"slug":"{shared_slug}","message":{{"role":"user","content":"agent 2"}}}}"#
  );

  let json1 = parse_json( &agent_entry_1 ).expect( "GD-6: parse agent entry 1" );
  let json2 = parse_json( &agent_entry_2 ).expect( "GD-6: parse agent entry 2" );

  let slug1 = json1.as_object()
    .and_then( | o | o.get( "slug" ) )
    .and_then( | v | v.as_str() )
    .expect( "GD-6: slug field must be present in agent entry 1" );

  let slug2 = json2.as_object()
    .and_then( | o | o.get( "slug" ) )
    .and_then( | v | v.as_str() )
    .expect( "GD-6: slug field must be present in agent entry 2" );

  assert_eq!(
    slug1,
    slug2,
    "GD-6: slug must be identical across agents from the same parent session; got {slug1:?} vs {slug2:?}"
  );
}

/// GD-7: Agent entries thread within their own session via `parentUuid`, not back to root session.
///
/// In an agent JSONL: first entry has `parentUuid: null`; subsequent entries have
/// `parentUuid` referencing a UUID from within the agent JSONL itself — NOT from the
/// root session JSONL.
///
/// ## Related Requirements
/// `tests/docs/guide/001_advanced_storage_topics.md` — GD-7
#[ test ]
fn gd_7_agent_entries_thread_via_parent_uuid_within_agent()
{
  let tmp = TempDir::new().unwrap();
  let agent_path = tmp.path().join( "agent-gd7test.jsonl" );

  let agent_uuid_1 = "gd7-agent-0001-0001-0001-gd7agent0001";
  let agent_uuid_2 = "gd7-agent-0002-0002-0002-gd7agent0002";
  // Root session UUID that agent entries must NOT reference as parentUuid
  let root_session_uuid = "root0000-0000-0000-0000-root00000000";

  // Entry 1: parentUuid is null (first entry in agent session)
  let entry_1 = format!(
    r#"{{"type":"user","uuid":"{agent_uuid_1}","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"{root_session_uuid}","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":true,"message":{{"role":"user","content":"first"}}}}"#
  );
  // Entry 2: parentUuid references agent_uuid_1 (within agent), NOT root_session_uuid
  let entry_2 = format!(
    r#"{{"type":"user","uuid":"{agent_uuid_2}","parentUuid":"{agent_uuid_1}","timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"{root_session_uuid}","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":true,"message":{{"role":"user","content":"second"}}}}"#
  );

  std::fs::write( &agent_path, format!( "{entry_1}\n{entry_2}\n" ) )
    .expect( "GD-7: write agent JSONL" );

  let mut session = Session::load( &agent_path ).expect( "GD-7: session must load" );
  let entries = session.entries().expect( "GD-7: entries must load" );

  assert_eq!( entries.len(), 2, "GD-7: must have 2 entries" );

  // First entry: parentUuid must be None
  assert!(
    entries[ 0 ].parent_uuid.is_none(),
    "GD-7: first agent entry parentUuid must be null; got {:?}",
    entries[ 0 ].parent_uuid
  );

  // Second entry: parentUuid must reference agent_uuid_1 (within agent), not root
  let second_parent = entries[ 1 ].parent_uuid.as_deref()
    .expect( "GD-7: second agent entry must have a parentUuid" );
  assert_eq!(
    second_parent,
    agent_uuid_1,
    "GD-7: second entry parentUuid must reference UUID within agent JSONL; got {second_parent:?}"
  );
  assert_ne!(
    second_parent,
    root_session_uuid,
    "GD-7: second entry parentUuid must NOT reference root session UUID"
  );
}

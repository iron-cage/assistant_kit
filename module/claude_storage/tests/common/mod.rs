//! Shared test utilities for `claude_storage` integration tests.
//!
//! Provides pre-compiled binary resolution to avoid cargo compilation
//! during test execution. Each `cargo run` inside a test triggers a full
//! compilation cycle (300s+), causing timeouts under workspace-wide runs.
//!
//! Fix(issue-claude-storage-timeout)
//! Root cause: 85 `Command::new("cargo").args(["run", ...])` calls across
//! 13 test files each triggered cargo compilation during test execution.
//! Under workspace-wide nextest runs, the 300s timeout was exceeded.
//! Pitfall: Never use `cargo run` or `cargo build` inside tests — use
//! `cargo_bin!()` to get the pre-compiled binary path from nextest.
//!
//! # Binary Name Coupling
//!
//! The helper below and the `[[bin]]` entries in `Cargo.toml` are tightly
//! coupled:
//!
//! | Binary           | `[[bin]] name`   | helper fn  | location        |
//! |------------------|-----------------|------------|-----------------|
//! | `claude_storage` | `claude_storage` | `clg_cmd`  | `common/mod.rs` |
//! | `clg`            | `clg`            | `clg_cmd`  | `common/mod.rs` |
//!
//! Both binaries have separate source files (`src/main.rs` and `src/bin/clg.rs`)
//! but both delegate to `cli_main::run()`. Tests use `clg_cmd()`
//! which resolves the `clg` alias — exercising the alias path through Cargo.
//!
//! If the binary is renamed, three things must change atomically:
//! 1. `Cargo.toml` — `[[bin]] name`
//! 2. The `cargo_bin!("<name>")` call in the helper
//! 3. The helper function name itself
//!
//! A partial rename compiles locally (cached artefact) but breaks on a
//! clean build because `cargo_bin!` panics when the binary is not found.

/// Return a Command pointing to the pre-compiled `clg` binary.
///
/// Uses `cargo_bin!()` macro which resolves to the binary built by
/// nextest/cargo-test BEFORE test execution. No compilation at test time.
pub fn clg_cmd() -> std::process::Command
{
  std::process::Command::new( assert_cmd::cargo::cargo_bin!( "clg" ) )
}


/// Write N synthetic conversation JSONL entries into `<root>/<project_id>/<session_id>.jsonl`.
///
/// Entries alternate user/assistant. Both types use fully-valid formats that
/// `Entry::from_json_line()` can parse:
/// - User: `"message":{"role":"user","content":"entry N"}`
/// - Assistant: `"message":{"role":"assistant","model":"...","id":"...","content":[...]}`
///   with top-level `"requestId"` required by the assistant parser.
///
/// Fix(issue-023)
/// Root cause: The previous format used `"content":"entry N"` for assistant entries
/// (a string), but `parse_assistant_message` requires content to be an array of blocks,
/// and requires top-level `requestId`, `message.model`, and `message.id`. Without
/// these fields, assistant entries silently fail to parse and are dropped by
/// `load_entries()`, making it impossible to test assistant-filtering behavior.
/// Pitfall: Synthetic test sessions must match the production JSON schema exactly.
/// When adding required fields to a parser, update all test helpers simultaneously.
///
/// # Panics
///
/// Panics if directory creation or file write fails.
#[allow(dead_code)]
pub fn write_test_session(
  root : &std::path::Path,
  project_id : &str,
  session_id : &str,
  entry_count : usize,
)
{
  use std::io::Write as _;

  let dir = root.join( "projects" ).join( project_id );
  std::fs::create_dir_all( &dir ).expect( "create project dir" );
  let path = dir.join( format!( "{session_id}.jsonl" ) );
  let mut file = std::fs::File::create( &path ).expect( "create session file" );

  for i in 0..entry_count
  {
    if i % 2 == 0
    {
      // User entry: message.content is a plain string
      writeln!(
        file,
        r#"{{"type":"user","uuid":"test-uuid-{i:03}","parentUuid":null,"timestamp":"2025-01-01T00:00:0{i}Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"entry {i}"}}}}"#
      )
      .expect( "write user entry" );
    }
    else
    {
      // Assistant entry: message.content is an array of blocks; requires requestId + model + id
      writeln!(
        file,
        r#"{{"type":"assistant","uuid":"test-uuid-{i:03}","parentUuid":"test-uuid-{prev:03}","timestamp":"2025-01-01T00:00:0{i}Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req_test_{i:03}","message":{{"role":"assistant","model":"claude-test","id":"msg_test_{i:03}","content":[{{"type":"text","text":"entry {i}"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#,
        prev = i.saturating_sub( 1 )
      )
      .expect( "write assistant entry" );
    }
  }
}

/// Write N synthetic entries for a path-based project (auto-encodes the path).
///
/// Returns the path-encoded project ID so callers can use it in assertions.
///
/// # Panics
///
/// Panics if path encoding or file write fails.
#[allow(dead_code)]
pub fn write_path_project_session(
  root : &std::path::Path,
  project_path : &std::path::Path,
  session_id : &str,
  entry_count : usize,
) -> String
{
  let encoded = claude_storage_core::encode_path( project_path )
    .expect( "encode project path" );
  write_test_session( root, &encoded, session_id, entry_count );
  encoded
}

/// Write a synthetic session with a specific last message text.
///
/// Writes `n_before` standard entries (alternating user/assistant) followed by
/// one user entry whose `content` field is `last_msg`. Total entry count is
/// `n_before + 1`. Useful for testing summary-mode output where the last text
/// entry must be controlled precisely.
///
/// Restriction: `last_msg` must contain only characters safe in a JSON string
/// without escaping (no `"`, `\`, or control characters).
///
/// # Panics
///
/// Panics if directory creation or file write fails.
#[allow(dead_code)]
pub fn write_test_session_with_last_message(
  root       : &std::path::Path,
  project_id : &str,
  session_id : &str,
  n_before   : usize,
  last_msg   : &str,
)
{
  use std::io::Write as _;

  let dir = root.join( "projects" ).join( project_id );
  std::fs::create_dir_all( &dir ).expect( "create project dir" );
  let path = dir.join( format!( "{session_id}.jsonl" ) );
  let mut file = std::fs::File::create( &path ).expect( "create session file" );

  for i in 0..n_before
  {
    if i % 2 == 0
    {
      writeln!(
        file,
        r#"{{"type":"user","uuid":"test-uuid-{i:03}","parentUuid":null,"timestamp":"2025-01-01T00:00:{i:02}Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"entry {i}"}}}}"#
      ).expect( "write user entry" );
    }
    else
    {
      let prev = i.saturating_sub( 1 );
      writeln!(
        file,
        r#"{{"type":"assistant","uuid":"test-uuid-{i:03}","parentUuid":"test-uuid-{prev:03}","timestamp":"2025-01-01T00:00:{i:02}Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req_test_{i:03}","message":{{"role":"assistant","model":"claude-test","id":"msg_test_{i:03}","content":[{{"type":"text","text":"entry {i}"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
      ).expect( "write assistant entry" );
    }
  }

  let idx = n_before;
  writeln!(
    file,
    r#"{{"type":"user","uuid":"test-uuid-{idx:03}","parentUuid":null,"timestamp":"2025-01-01T01:00:00Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"{last_msg}"}}}}"#
  ).expect( "write last message entry" );
}

/// Write a `meta.json` sidecar for an agent session.
///
/// Creates `{dir}/agent-{agent_id}.meta.json` with the given `agent_type`.
/// An empty `agent_type` produces `{}` (malformed — no `agentType` key),
/// useful for testing the fallback-to-"unknown" path.
///
/// # Panics
///
/// Panics if file write fails.
#[allow(dead_code)]
pub fn write_agent_meta_json(
  dir        : &std::path::Path,
  agent_id   : &str,
  agent_type : &str,
)
{
  let path = dir.join( format!( "agent-{agent_id}.meta.json" ) );
  if agent_type.is_empty()
  {
    std::fs::write( &path, b"{}" ).expect( "write empty meta.json" );
  }
  else
  {
    let json = format!( r#"{{"agentType":"{agent_type}"}}"# );
    std::fs::write( &path, json.as_bytes() ).expect( "write meta.json" );
  }
}

/// Write a hierarchical session family: root `.jsonl` + `{uuid}/subagents/` agents.
///
/// Creates:
/// - `{root}/projects/{project_id}/{root_session_id}.jsonl` with `entries_each` entries
/// - `{root}/projects/{project_id}/{root_session_id}/subagents/agent-{id}.jsonl` per agent
/// - `{root}/projects/{project_id}/{root_session_id}/subagents/agent-{id}.meta.json` per agent
///
/// `agents` is `&[(&str, &str)]` where each tuple is `(agent_id, agent_type)`.
///
/// # Panics
///
/// Panics if directory creation or file write fails.
#[allow(dead_code)]
pub fn write_hierarchical_session(
  root             : &std::path::Path,
  project_id       : &str,
  root_session_id  : &str,
  agents           : &[ ( &str, &str ) ],
  entries_each     : usize,
)
{
  use std::io::Write as _;

  // Root session
  write_test_session( root, project_id, root_session_id, entries_each );

  // Subagents directory
  let subagents_dir = root
    .join( "projects" )
    .join( project_id )
    .join( root_session_id )
    .join( "subagents" );
  std::fs::create_dir_all( &subagents_dir ).expect( "create subagents dir" );

  for ( agent_id, agent_type ) in agents
  {
    let agent_file_name = format!( "agent-{agent_id}.jsonl" );
    let agent_path = subagents_dir.join( &agent_file_name );
    let mut file = std::fs::File::create( &agent_path ).expect( "create agent session file" );

    for i in 0..entries_each
    {
      if i % 2 == 0
      {
        writeln!(
          file,
          r#"{{"type":"user","uuid":"agent-uuid-{i:03}","parentUuid":null,"timestamp":"2025-01-01T00:00:0{i}Z","cwd":"/tmp","sessionId":"{root_session_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"agent entry {i}"}}}}"#
        ).expect( "write agent user entry" );
      }
      else
      {
        let prev = i.saturating_sub( 1 );
        writeln!(
          file,
          r#"{{"type":"assistant","uuid":"agent-uuid-{i:03}","parentUuid":"agent-uuid-{prev:03}","timestamp":"2025-01-01T00:00:0{i}Z","cwd":"/tmp","sessionId":"{root_session_id}","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req_agent_{i:03}","message":{{"role":"assistant","model":"claude-test","id":"msg_agent_{i:03}","content":[{{"type":"text","text":"agent entry {i}"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
        ).expect( "write agent assistant entry" );
      }
    }

    write_agent_meta_json( &subagents_dir, agent_id, agent_type );
  }
}

/// Write a hierarchical session family for a path-based project.
///
/// Auto-encodes the project path. Returns the encoded project ID.
///
/// # Panics
///
/// Panics if path encoding or file write fails.
#[allow(dead_code)]
pub fn write_hierarchical_path_session(
  root             : &std::path::Path,
  project_path     : &std::path::Path,
  root_session_id  : &str,
  agents           : &[ ( &str, &str ) ],
  entries_each     : usize,
) -> String
{
  let encoded = claude_storage_core::encode_path( project_path )
    .expect( "encode project path" );
  write_hierarchical_session( root, &encoded, root_session_id, agents, entries_each );
  encoded
}

/// Write a flat-format agent session at the project root.
///
/// Creates `{root}/projects/{project_id}/agent-{agent_id}.jsonl` with
/// the first JSONL entry containing `"sessionId":"{parent_session_id}"`.
///
/// # Panics
///
/// Panics if directory creation or file write fails.
#[allow(dead_code)]
pub fn write_flat_agent_session(
  root               : &std::path::Path,
  project_id         : &str,
  agent_id           : &str,
  parent_session_id  : &str,
  n_entries          : usize,
)
{
  use std::io::Write as _;

  let dir = root.join( "projects" ).join( project_id );
  std::fs::create_dir_all( &dir ).expect( "create project dir" );
  let path = dir.join( format!( "agent-{agent_id}.jsonl" ) );
  let mut file = std::fs::File::create( &path ).expect( "create flat agent file" );

  for i in 0..n_entries
  {
    if i % 2 == 0
    {
      writeln!(
        file,
        r#"{{"type":"user","uuid":"flat-uuid-{i:03}","parentUuid":null,"timestamp":"2025-01-01T00:00:0{i}Z","cwd":"/tmp","sessionId":"{parent_session_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"flat agent entry {i}"}}}}"#
      ).expect( "write flat agent user entry" );
    }
    else
    {
      let prev = i.saturating_sub( 1 );
      writeln!(
        file,
        r#"{{"type":"assistant","uuid":"flat-uuid-{i:03}","parentUuid":"flat-uuid-{prev:03}","timestamp":"2025-01-01T00:00:0{i}Z","cwd":"/tmp","sessionId":"{parent_session_id}","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req_flat_{i:03}","message":{{"role":"assistant","model":"claude-test","id":"msg_flat_{i:03}","content":[{{"type":"text","text":"flat agent entry {i}"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
      ).expect( "write flat agent assistant entry" );
    }
  }
}

/// Write a session with a specific last message for a path-based project.
///
/// Auto-encodes the project path and delegates to
/// `write_test_session_with_last_message`. Returns the encoded project ID.
///
/// # Panics
///
/// Panics if path encoding or file write fails.
#[allow(dead_code)]
pub fn write_path_project_session_with_last_message(
  root         : &std::path::Path,
  project_path : &std::path::Path,
  session_id   : &str,
  n_before     : usize,
  last_msg     : &str,
) -> String
{
  let encoded = claude_storage_core::encode_path( project_path )
    .expect( "encode project path" );
  write_test_session_with_last_message( root, &encoded, session_id, n_before, last_msg );
  encoded
}

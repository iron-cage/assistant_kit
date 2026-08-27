//! Bug Reproducer (BUG-491): `Session::is_agent_session()`'s doc comment falsely claimed an
//! `isSidechain`-based entry check that was never implemented
//!
//! ## Root Cause
//!
//! `is_agent_session()`'s own doc comment claimed two detection signals — a filename prefix
//! ("agent-{id}.jsonl") OR `isSidechain: true` in the session's entries — but the implementation
//! only ever checked the filename prefix. The doc comment (present since the initial commit) never
//! matched the canonical algorithm (`docs/algorithm/003_agent_session_tracking.md`), which
//! deliberately keeps filename-based session classification (`is_agent_session`) separate from
//! entry-level sidechain tagging (`is_agent_entry` — itself never implemented, since it depends on
//! an `Entry::agent_id` field that was deliberately never added to the `Entry` struct).
//!
//! ## Why Not Caught
//!
//! Every existing test that exercises `is_agent_session()` (`invariant_contracts_test.rs`'s
//! IN-1 through IN-5, `projects_output_format_test.rs`) constructs "agent" sessions via the
//! filename prefix only; none ever constructed a non-`agent-`-prefixed session with
//! `isSidechain:true` entries to check whether the OR branch the doc comment promised was real.
//!
//! ## Fix Applied
//!
//! Corrected the doc comment on `Session::is_agent_session()` (`session.rs`) to describe only the
//! actual filename-based check, and to explain why entry-level `isSidechain` tagging is
//! deliberately not combined into it. No code logic changed — the implementation was already
//! correct and already matched the canonical algorithm doc.
//!
//! ## Prevention
//!
//! This test locks in the filename-only contract as a permanent regression guard: it proves
//! `isSidechain:true` entries do NOT make a non-`agent-`-prefixed session register as an agent
//! session, and that `isSidechain:false`/absent entries do NOT prevent an `agent-`-prefixed
//! session from registering as one. A future contributor tempted to "complete" the old doc
//! comment's promise by adding entry-loading I/O here would break this test.
//!
//! ## Pitfall
//!
//! A function's own doc comment can silently drift from the authoritative algorithm/design doc,
//! since nothing forces them to agree. When a doc comment and a canonical design doc disagree,
//! check which one matches actual behavior and existing test expectations before assuming the
//! code is what's wrong — the code is not always the side at fault.

use std::fs;
use tempfile::TempDir;

/// Helper: create a project directory in `projects_dir`
fn create_project( projects_dir : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  let p = projects_dir.join( name );
  fs::create_dir_all( &p ).expect( "create project dir" );
  p
}

/// Test `is_agent_session()` ignores `isSidechain: true` entries on a non-`agent-`-prefixed file.
///
/// ## Coverage
///
/// A session file named with a plain UUID (no "agent-" prefix) whose entries all carry
/// `isSidechain:true`. `is_agent_session()` must still return `false` — proving the check is
/// filename-only and entry content never widens it, contrary to the old (now-corrected) doc
/// comment's claim.
#[test]
fn is_agent_session_ignores_is_sidechain_on_non_agent_filename()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-doc-mismatch-project" );

  let content = concat!(
    r#"{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00Z","cwd":"/tmp","sessionId":"11112222-3333-4444-5555-666677778888","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":true,"message":{"role":"user","content":"sidechain entry"}}"#, "\n",
    r#"{"type":"assistant","uuid":"u2","parentUuid":"u1","timestamp":"2026-01-01T00:00:01Z","cwd":"/tmp","sessionId":"11112222-3333-4444-5555-666677778888","version":"2.0.0","gitBranch":null,"userType":"external","isSidechain":true,"message":{"role":"assistant","content":[{"type":"text","text":"reply"}]}}"#, "\n",
  );

  let session_path = p_dir.join( "11112222-3333-4444-5555-666677778888.jsonl" );
  fs::write( &session_path, content ).expect( "write session file" );

  let session = claude_storage_core::Session::load( &session_path ).expect( "load session" );

  assert!(
    !session.is_agent_session(),
    "BUG-491: a non-agent--prefixed session must not register as an agent session, even when \
     every entry has isSidechain:true; is_agent_session() is filename-only by canonical design"
  );
}

/// Test `is_agent_session()` returns `true` for an `agent-`-prefixed file regardless of entry
/// content.
///
/// ## Coverage
///
/// A session file named with the `agent-` prefix whose entries carry `isSidechain:false`
/// (the opposite of what a naive isSidechain-based check would expect). `is_agent_session()`
/// must still return `true` — the filename alone is sufficient and entry content is irrelevant
/// in either direction, confirming the check never needs to load entries.
#[test]
fn is_agent_session_true_for_agent_prefixed_filename_regardless_of_entries()
{
  let temp = TempDir::new().expect( "temp dir" );
  let projects_dir = temp.path().join( "projects" );
  let p_dir = create_project( &projects_dir, "-doc-mismatch-project-2" );

  let content = concat!(
    r#"{"type":"user","uuid":"u1","parentUuid":null,"timestamp":"2026-01-01T00:00:00Z","cwd":"/tmp","sessionId":"parent-session","version":"2.0.0","gitBranch":null,"userType":"human","isSidechain":false,"message":{"role":"user","content":"not tagged sidechain"}}"#, "\n",
  );

  let session_path = p_dir.join( "agent-doccheck01.jsonl" );
  fs::write( &session_path, content ).expect( "write session file" );

  let session = claude_storage_core::Session::load( &session_path ).expect( "load session" );

  assert!(
    session.is_agent_session(),
    "BUG-491: an agent--prefixed session must register as an agent session from its filename \
     alone, even when its entries carry isSidechain:false"
  );
}

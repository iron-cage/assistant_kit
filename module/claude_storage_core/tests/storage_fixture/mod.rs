//! Deterministic Claude Code storage fixtures shared by the `export.rs`,
//! `search.rs`, and `filtering.rs` integration binaries.
//!
//! Every helper builds a self-contained `TempDir` storage tree — a `projects/`
//! directory holding project directories holding `.jsonl` session files — so the
//! tests that consume it assert against known content instead of against whatever
//! happens to sit in a developer's real `~/.claude/` directory.
//!
//! ## Entry shape
//!
//! `user_line`, `assistant_line`, and `assistant_thinking_line` emit the full
//! Claude Code v2.x entry schema, including every field `Entry::from_json_line`
//! requires: `uuid`, `timestamp`, `type`, `cwd`, `sessionId`, `version`,
//! `userType`, plus `requestId` / `message.model` / `message.id` and an array-shaped
//! `message.content` for assistant entries. A line missing any of them fails to
//! parse and is silently dropped by `Session::load_entries()`, which would make
//! every downstream assertion vacuous.
//!
//! `metadata_line` deliberately emits the opposite: a non-conversation entry
//! (`queue-operation`, `summary`, `file-history-snapshot`) that the parser rejects
//! and `stats()` skips, so tests can prove such entries never reach an export or a
//! search result.
//!
//! ## Project and session naming
//!
//! - A project directory whose name starts with `-` is a path project
//!   (`-home-user-alpha` decodes to `/home/user/alpha`); any other name is a UUID
//!   project.
//! - A session file named `agent-*.jsonl` is an agent session; every other name is
//!   a main session.
//!
//! ## Text arguments
//!
//! `text` and `thinking` arguments are interpolated straight into a JSON string
//! value, so they must already be JSON-safe: no bare `"` or `\`, unless the caller
//! deliberately writes an escape sequence (e.g. Rust `"a\\nb"` to embed a newline
//! inside one entry's content).

// Each of the three test binaries compiles this module independently and none of
// them uses every helper, so dead_code must be allowed — RUSTFLAGS="-D warnings"
// would otherwise fail the unused copies.
#![ allow( dead_code ) ]

use std::fs;
use std::path::{ Path, PathBuf };
use claude_storage_core::{ Session, Storage };
use tempfile::TempDir;

/// Create an empty storage root containing just its `projects/` directory.
///
/// The returned `TempDir` owns the tree — keep it alive for the whole test.
pub fn storage_root() -> TempDir
{
  let temp = TempDir::new().expect( "create temp storage root" );
  fs::create_dir_all( temp.path().join( "projects" ) ).expect( "create projects dir" );
  temp
}

/// Create `<root>/projects/<dir_name>/` and return its path.
pub fn project_dir( root : &Path, dir_name : &str ) -> PathBuf
{
  let dir = root.join( "projects" ).join( dir_name );
  fs::create_dir_all( &dir ).expect( "create project dir" );
  dir
}

/// Write `lines` as `<dir>/<session_name>.jsonl` and return the file path.
///
/// A trailing newline is appended, matching how Claude Code leaves a session file
/// on disk. An empty `lines` produces a session with zero entries.
pub fn write_session( dir : &Path, session_name : &str, lines : &[ String ] ) -> PathBuf
{
  let path = dir.join( format!( "{session_name}.jsonl" ) );
  let mut content = lines.join( "\n" );
  content.push( '\n' );
  fs::write( &path, content ).expect( "write session file" );
  path
}

/// Write an `entries`-entry alternating conversation as `<dir>/<session_name>.jsonl`.
///
/// Entry `0` is a user entry, entry `1` an assistant entry, and so on, so
/// `count_entries()` on the result equals `entries` exactly.
pub fn write_conversation_session( dir : &Path, session_name : &str, entries : usize ) -> PathBuf
{
  let mut lines = Vec::with_capacity( entries );

  for seq in 0..entries
  {
    if seq % 2 == 0
    {
      lines.push( user_line( session_name, seq, &format!( "user turn {seq}" ) ) );
    }
    else
    {
      lines.push( assistant_line( session_name, seq, &format!( "assistant turn {seq}" ) ) );
    }
  }

  write_session( dir, session_name, &lines )
}

/// Open `root` as storage and take the one session of the one project inside it.
///
/// Asserts the fixture really holds exactly one project and exactly one session, so
/// a mis-built tree fails loudly here instead of silently emptying a test.
pub fn single_session( root : &Path ) -> Session
{
  let storage = Storage::with_root( root );

  let projects = storage.list_projects().expect( "list projects" );
  assert_eq!( projects.len(), 1, "fixture must hold exactly one project" );

  let mut sessions = projects[ 0 ].sessions().expect( "list sessions" );
  assert_eq!( sessions.len(), 1, "fixture must hold exactly one session" );

  sessions.remove( 0 )
}

/// A user entry whose `message.content` is a plain string — the shape Claude Code
/// writes for a typed prompt.
///
/// `seq` drives the entry's uuid and its `00:00:SS` timestamp, so callers control
/// both ordering and the exact timestamps an export renders.
pub fn user_line( session_id : &str, seq : usize, text : &str ) -> String
{
  format!(
    r#"{{"type":"user","uuid":"{session_id}-u{seq:03}","parentUuid":null,"timestamp":"2026-01-01T00:00:{seq:02}Z","cwd":"/home/user/project","sessionId":"{session_id}","version":"2.0.31","gitBranch":"master","userType":"external","isSidechain":false,"message":{{"role":"user","content":"{text}"}}}}"#
  )
}

/// An assistant entry carrying a single text content block.
pub fn assistant_line( session_id : &str, seq : usize, text : &str ) -> String
{
  format!(
    r#"{{"type":"assistant","uuid":"{session_id}-a{seq:03}","parentUuid":null,"timestamp":"2026-01-01T00:00:{seq:02}Z","cwd":"/home/user/project","sessionId":"{session_id}","version":"2.0.31","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req-{seq:03}","message":{{"role":"assistant","model":"claude-sonnet-5","id":"msg-{session_id}-{seq:03}","content":[{{"type":"text","text":"{text}"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
  )
}

/// An assistant entry whose content is a thinking block followed by a text block —
/// the shape markdown export renders as a collapsible `<details>` section.
pub fn assistant_thinking_line( session_id : &str, seq : usize, thinking : &str, text : &str ) -> String
{
  format!(
    r#"{{"type":"assistant","uuid":"{session_id}-a{seq:03}","parentUuid":null,"timestamp":"2026-01-01T00:00:{seq:02}Z","cwd":"/home/user/project","sessionId":"{session_id}","version":"2.0.31","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req-{seq:03}","message":{{"role":"assistant","model":"claude-sonnet-5","id":"msg-{session_id}-{seq:03}","content":[{{"type":"thinking","thinking":"{thinking}","signature":"sig-{seq:03}"}},{{"type":"text","text":"{text}"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
  )
}

/// A non-conversation metadata entry — `kind` is a `type` value such as
/// `queue-operation`, `summary`, or `file-history-snapshot`.
///
/// `Entry::from_json_line` rejects it (unrecognised entry type) and `stats()` skips
/// it, so it must never appear in an export or a search result.
pub fn metadata_line( kind : &str, seq : usize ) -> String
{
  format!(
    r#"{{"type":"{kind}","timestamp":"2026-01-01T00:00:{seq:02}Z","payload":{{"note":"non-conversation metadata"}}}}"#
  )
}

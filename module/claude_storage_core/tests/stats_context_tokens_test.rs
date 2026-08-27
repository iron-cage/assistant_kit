//! `SessionStats.last_context_tokens` populated by `Session::stats()`
//!
//! ## Source
//!
//! - `docs/data_structure/004_session_context_state.md` — the token-accounting split
//! - `src/stats.rs` — `last_context_tokens` alongside `max_context_tokens`
//!
//! ## Why This Field Exists
//!
//! Every turn re-sends the whole conversation, so the `total_*` sums grow with
//! the number of turns and exceed the context window many times over in a long
//! session. They answer "what did this session cost", never "how full is it
//! now". Only the newest call describes the present, and that is what this field
//! holds.
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | T01 | Several turns of growing size | The LAST turn's size, not the sum |
//! | T02 | A turn smaller than an earlier one | Follows down; peak stays up |
//! | T03 | One API call split across content-block lines | Counted once, not multiplied |
//! | T04 | No assistant messages | `0`, and distinguishable from a real `0` turn |
//! | T05 | Cache-heavy turn | Cached tokens counted; `input_tokens` alone would undercount |

use std::fs;
use tempfile::TempDir;

/// Helper: create a project directory in `projects_dir`
fn create_project( projects_dir : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  let p = projects_dir.join( name );
  fs::create_dir_all( &p ).expect( "create project dir" );
  p
}

/// Helper: write `content` as a session file and load it.
fn load_session( p_dir : &std::path::Path, file_name : &str, content : &str )
  -> claude_storage_core::Session
{
  let session_path = p_dir.join( file_name );
  fs::write( &session_path, content ).expect( "write session file" );
  claude_storage_core::Session::load( &session_path ).expect( "load session" )
}

/// Helper: one assistant line whose usage adds up to a known context size.
///
/// Split across the three input-side fields deliberately — a reader that looks
/// only at `input_tokens` sees a different number than one that sums all three.
fn assistant( id : &str, input : u64, cache_read : u64, cache_creation : u64 ) -> String
{
  format!
  (
    r#"{{"type":"assistant","cwd":"/home/alice/proj","timestamp":"2026-01-01T00:00:00Z","message":{{"id":"{id}","role":"assistant","model":"claude-sonnet-5","usage":{{"input_tokens":{input},"output_tokens":10,"cache_read_input_tokens":{cache_read},"cache_creation_input_tokens":{cache_creation}}},"content":[]}}}}"#
  )
}

/// T01: the latest turn's size is reported, not the running sum.
///
/// Three turns of 100, 200, and 300 leave a sum of 600 and a present size of
/// 300. Reporting 600 would tell a caller the conversation occupies twice the
/// window it actually does, and the error grows with every turn.
#[ test ]
fn stats_last_context_is_the_newest_turn_not_the_sum()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-ctx-newest" );

  let content = format!
  (
    "{}\n{}\n{}\n",
    assistant( "msg_01", 100, 0, 0 ),
    assistant( "msg_02", 200, 0, 0 ),
    assistant( "msg_03", 300, 0, 0 ),
  );
  let mut session = load_session( &p_dir, "aaaa0001-0000-4000-8000-000000000001.jsonl", &content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!( stats.last_context_tokens, 300, "must be the newest turn's size" );
  assert_eq!( stats.total_input_tokens, 600, "the sum is still reported, separately" );
  assert_ne!
  (
    stats.last_context_tokens, stats.total_input_tokens,
    "the two must not be the same number, or this test proves nothing",
  );
}

/// T02: a shrinking conversation is followed down; the peak is not.
///
/// Compaction is the real case — it replaces the conversation with a summary, so
/// the next call is much smaller than the one before it. A field that only ever
/// rose would keep reporting the pre-compaction size forever, which is precisely
/// when a caller most needs to know that room was freed.
#[ test ]
fn stats_last_context_follows_a_compaction_down()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-ctx-shrink" );

  let content = format!
  (
    "{}\n{}\n",
    assistant( "msg_01", 5_000, 0, 0 ),
    assistant( "msg_02", 800, 0, 0 ),
  );
  let mut session = load_session( &p_dir, "aaaa0002-0000-4000-8000-000000000002.jsonl", &content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!( stats.last_context_tokens, 800, "must follow the conversation down" );
  assert_eq!( stats.max_context_tokens, 5_000, "the high-water mark must stay up" );
}

/// T03: one API call split across content-block lines counts once.
///
/// A single response spans one JSONL line per content block, each repeating the
/// same `message.id` and the same `usage` object (`Fix(issue-038)`). Assigning
/// per line rather than per deduplicated call would still land on the right
/// number here by luck — the lines are identical — so this asserts the entry
/// count too, which is what actually moves if the dedup is bypassed.
#[ test ]
fn stats_last_context_counts_one_call_once()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-ctx-dedup" );

  // Three content blocks of one response, then a second, smaller response.
  let content = format!
  (
    "{}\n{}\n{}\n{}\n",
    assistant( "msg_01", 400, 0, 0 ),
    assistant( "msg_01", 400, 0, 0 ),
    assistant( "msg_01", 400, 0, 0 ),
    assistant( "msg_02", 150, 0, 0 ),
  );
  let mut session = load_session( &p_dir, "aaaa0003-0000-4000-8000-000000000003.jsonl", &content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!( stats.last_context_tokens, 150, "the newest call, not the newest line" );
  assert_eq!( stats.assistant_entries, 2, "three blocks are one call" );
  assert_eq!( stats.total_input_tokens, 550, "the repeated blocks must not be summed" );
}

/// T04: a session with no assistant messages reports zero.
///
/// A session that has not had a turn yet has no measured context size. Zero is
/// the honest answer, and it is distinguishable from a real turn because
/// `assistant_entries` is zero alongside it.
#[ test ]
fn stats_last_context_zero_without_assistant_messages()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-ctx-none" );

  let content = concat!
  (
    r#"{"type":"user","cwd":"/home/alice/proj","message":{"role":"user","content":"hello"},"timestamp":"2026-01-01T00:00:00Z"}"#, "\n",
  );
  let mut session = load_session( &p_dir, "aaaa0004-0000-4000-8000-000000000004.jsonl", content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!( stats.last_context_tokens, 0 );
  assert_eq!( stats.assistant_entries, 0, "what tells a caller the zero is 'no data'" );
}

/// T05: cached tokens are part of the context size.
///
/// With prompt caching, most of a turn's prompt arrives as `cache_read` rather
/// than `input_tokens` — the static system prompt and tool definitions
/// especially, since they are identical every turn. A reader that took
/// `input_tokens` alone would report 120 for a conversation actually occupying
/// `18_120`, and would swing wildly between a cold and a warm cache.
#[ test ]
fn stats_last_context_includes_cached_tokens()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-ctx-cached" );

  let content = format!( "{}\n", assistant( "msg_01", 120, 17_000, 1_000 ) );
  let mut session = load_session( &p_dir, "aaaa0005-0000-4000-8000-000000000005.jsonl", &content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!
  (
    stats.last_context_tokens, 18_120,
    "input + cache_read + cache_creation, not input alone",
  );
}

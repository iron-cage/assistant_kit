//! Bug Reproducer for `Session::stats()` token/entry double-counting
//!
//! ## issue-038: `stats()` summed usage and counted turns per JSONL line, not per API call
//!
//! ## Root Cause
//!
//! One Claude API response can span multiple `assistant` JSONL lines — one per content
//! block (text, thinking, `tool_use`, ...) — and every such line repeats the identical
//! `message.id` and `message.usage` object. `Session::stats()` summed `usage` and
//! incremented `assistant_entries`/`total_entries` per LINE with no `message.id` dedup,
//! so both entry counts and every token total were inflated by exactly the response's
//! content-block multiplicity. Confirmed on real production storage: a manual audit of
//! live session data found 2505 raw assistant lines collapsing to 1201 unique message
//! ids — a ~2.1x over-count that was silently baked into every consumer of `stats()`
//! (`.usage`, `.status` verbosity 2+, and the new `.rollup` command's token totals).
//!
//! ## Why Not Caught
//!
//! Every existing test fixture (including `cli_cmd_usage_test.rs`'s own
//! `write_usage_session()` helper) assigns a distinct `message.id` per turn by
//! construction — none ever wrote two JSONL lines sharing one `message.id`, so no test
//! exercised the actual multi-content-block shape real Claude Code transcripts produce.
//!
//! ## Fix Applied
//!
//! `Session::stats()` now tracks a `HashSet<String>` of seen `message.id` values and
//! gates both the entry-count increment and the usage sum on "is this id new" — a line
//! whose `message.id` was already seen contributes to `first_timestamp`/`last_timestamp`/
//! `cwd` extraction (as any line does) but never re-counts as a turn and never re-sums
//! its tokens. A line with no `message.id` at all is treated as always-new (never
//! skipped), so malformed/legacy lines are never silently dropped from the totals. The
//! same pass also added `SessionStats::max_context_tokens` (largest single deduplicated
//! call's `input + cache_read + cache_creation`) and `SessionStats::model` (first-seen
//! model, first-entry-wins like `cwd`) — both needed by the new `.rollup` command and
//! naturally computed in the same single-pass scan already doing the dedup.
//!
//! ## Prevention
//!
//! Any future per-line JSONL scanner that aggregates "per assistant turn" data (counts,
//! sums, or a running max) must dedup by `message.id` first. Line count is never turn
//! count in Claude Code's v2.0+ transcript format — this holds for any new aggregation,
//! not just token totals.
//!
//! ## Pitfall
//!
//! A synthetic test fixture that assigns one unique id per turn (the obvious way to
//! write a session-file builder) cannot catch this class of bug at all — it must
//! deliberately reuse one `message.id` across multiple lines to reproduce the real
//! transcript shape.

use claude_storage_core::Session;
use std::fs;
use tempfile::TempDir;

/// Helper: create a project directory in `projects_dir`.
fn create_project( projects_dir : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  let p = projects_dir.join( name );
  fs::create_dir_all( &p ).expect( "create project dir" );
  p
}

/// Helper: write `content` as a session file and load it.
fn load_session( p_dir : &std::path::Path, file_name : &str, content : &str ) -> Session
{
  let session_path = p_dir.join( file_name );
  fs::write( &session_path, content ).expect( "write session file" );
  Session::load( &session_path ).expect( "load session" )
}

/// One assistant JSONL line sharing `msg_id` with any sibling content-block lines.
fn assistant_line( msg_id : &str, model : &str, input : u64, output : u64, ts : &str ) -> String
{
  format!(
    r#"{{"type":"assistant","cwd":"/proj","timestamp":"{ts}","message":{{"role":"assistant","model":"{model}","id":"{msg_id}","content":[{{"type":"text","text":"ok"}}],"usage":{{"input_tokens":{input},"output_tokens":{output},"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
  )
}

/// Reproduce issue-038: 3 content-block lines sharing one `message.id` must
/// count and sum as ONE turn, not three.
///
/// ## Purpose
/// Locks in the primary fix: multi-content-block responses collapse to one
/// deduplicated turn.
///
/// ## Coverage
/// 3 assistant lines, identical `message.id` and `usage`, distinct content —
/// `assistant_entries == 1`, token totals equal exactly ONE copy of `usage`.
///
/// ## Validation Strategy
/// Build a 3-line same-id fixture; call `stats()`; assert counts/sums are not
/// 3x inflated.
///
/// ## Related Requirements
/// issue-038
// test_kind: bug_reproducer(issue-038)
#[ test ]
fn stats_dedups_same_message_id_content_blocks()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-dedup-content-blocks" );

  let content = format!(
    "{}\n{}\n{}\n",
    assistant_line( "msg_shared", "claude-opus-5", 1000, 50, "2026-08-20T10:00:00Z" ),
    assistant_line( "msg_shared", "claude-opus-5", 1000, 50, "2026-08-20T10:00:00Z" ),
    assistant_line( "msg_shared", "claude-opus-5", 1000, 50, "2026-08-20T10:00:00Z" ),
  );
  let mut session = load_session( &p_dir, "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee.jsonl", &content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!( stats.assistant_entries, 1, "3 content-block lines sharing one id must count as 1 turn" );
  assert_eq!( stats.total_entries, 1, "total_entries must match the deduplicated turn count" );
  assert_eq!( stats.total_input_tokens, 1000, "input tokens must not be summed 3x" );
  assert_eq!( stats.total_output_tokens, 50, "output tokens must not be summed 3x" );
}

/// Test genuinely distinct `message.id` values are still all counted — a
/// regression guard against over-correcting to "only the first assistant
/// line ever counts."
///
/// ## Purpose
/// Validates the dedup fix narrows to duplicate ids specifically, and does
/// not collapse distinct turns.
///
/// ## Coverage
/// 2 assistant lines with distinct `message.id` values — both counted and
/// both summed.
///
/// ## Validation Strategy
/// Two distinct-id lines; assert `assistant_entries == 2` and summed tokens.
///
/// ## Related Requirements
/// issue-038 (regression guard)
#[ test ]
fn stats_counts_distinct_message_ids_separately()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-dedup-distinct-ids" );

  let content = format!(
    "{}\n{}\n",
    assistant_line( "msg_1", "claude-opus-5", 100, 10, "2026-08-20T10:00:00Z" ),
    assistant_line( "msg_2", "claude-opus-5", 200, 20, "2026-08-20T10:01:00Z" ),
  );
  let mut session = load_session( &p_dir, "bbbbbbbb-cccc-dddd-eeee-ffffffffffff.jsonl", &content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!( stats.assistant_entries, 2, "2 distinct ids must count as 2 turns" );
  assert_eq!( stats.total_input_tokens, 300, "distinct-id tokens must both be summed" );
  assert_eq!( stats.total_output_tokens, 30 );
}

/// Test a `message` with no `id` field at all is always counted, never
/// silently dropped by the dedup gate.
///
/// ## Purpose
/// Validates the documented "missing id = always new" fallback — the dedup
/// fix must never under-count malformed/legacy lines that lack an id.
///
/// ## Coverage
/// 2 assistant lines with no `"id"` key in `message` — both counted.
///
/// ## Validation Strategy
/// Hand-write 2 id-less assistant lines; assert both contribute.
///
/// ## Related Requirements
/// issue-038 (missing-id fallback)
#[ test ]
fn stats_missing_message_id_never_dropped()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-dedup-missing-id" );

  let line = | input : u64 | format!(
    r#"{{"type":"assistant","cwd":"/proj","timestamp":"2026-08-20T10:00:00Z","message":{{"role":"assistant","model":"claude-opus-5","content":[{{"type":"text","text":"ok"}}],"usage":{{"input_tokens":{input},"output_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
  );
  let content = format!( "{}\n{}\n", line( 10 ), line( 20 ) );
  let mut session = load_session( &p_dir, "cccccccc-dddd-eeee-ffff-000000000000.jsonl", &content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!( stats.assistant_entries, 2, "id-less lines must never be dropped by the dedup gate" );
  assert_eq!( stats.total_input_tokens, 30, "both id-less lines' tokens must be summed" );
}

/// Test `max_context_tokens` tracks the largest single deduplicated call's
/// context, not a sum across calls.
///
/// ## Purpose
/// Validates the new `max_context_tokens` field added alongside the dedup
/// fix, including that duplicate content-block lines don't inflate it.
///
/// ## Coverage
/// Call 1 (2 duplicate content-block lines, context 500) and call 2 (context
/// 9000) — `max_context_tokens` must be exactly 9000.
///
/// ## Validation Strategy
/// 3 lines total (2 sharing an id at context 500, 1 at context 9000); assert
/// the max, not `500 + 500 + 9000`.
///
/// ## Related Requirements
/// issue-038 (`max_context_tokens`)
#[ test ]
fn stats_max_context_tracks_largest_deduplicated_call()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-dedup-max-context" );

  let content = format!(
    "{}\n{}\n{}\n",
    assistant_line( "msg_small", "claude-opus-5", 500, 0, "2026-08-20T10:00:00Z" ),
    assistant_line( "msg_small", "claude-opus-5", 500, 0, "2026-08-20T10:00:00Z" ),
    assistant_line( "msg_big", "claude-opus-5", 9000, 0, "2026-08-20T10:01:00Z" ),
  );
  let mut session = load_session( &p_dir, "dddddddd-eeee-ffff-0000-111111111111.jsonl", &content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!( stats.max_context_tokens, 9000, "must be the max deduplicated call, never a sum" );
}

/// Test `model` is populated first-entry-wins, mirroring `cwd`'s semantics.
///
/// ## Purpose
/// Validates the new `model` field added alongside the dedup fix.
///
/// ## Coverage
/// First call uses `claude-opus-5`, second uses `claude-sonnet-5` —
/// `model` must be `"claude-opus-5"`.
///
/// ## Validation Strategy
/// Two distinct-id, distinct-model lines; assert the FIRST line's model wins.
///
/// ## Related Requirements
/// issue-038 (model field)
#[ test ]
fn stats_model_is_first_entry_wins()
{
  let temp = TempDir::new().expect( "temp dir" );
  let p_dir = create_project( &temp.path().join( "projects" ), "-dedup-model" );

  let content = format!(
    "{}\n{}\n",
    assistant_line( "msg_1", "claude-opus-5", 10, 0, "2026-08-20T10:00:00Z" ),
    assistant_line( "msg_2", "claude-sonnet-5", 10, 0, "2026-08-20T10:01:00Z" ),
  );
  let mut session = load_session( &p_dir, "eeeeeeee-ffff-0000-1111-222222222222.jsonl", &content );

  let stats = session.stats().expect( "stats() must succeed" );

  assert_eq!( stats.model.as_deref(), Some( "claude-opus-5" ), "model must be the FIRST call's model" );
}

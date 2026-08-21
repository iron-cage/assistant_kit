//! Unit tests for `cost::cost_report()` and `cost::aggregate_reports()` —
//! per-model usage attribution, cache-TTL split, compaction counting,
//! `<synthetic>` skipping, `message.id` dedup, and family aggregation.

use claude_storage_core::{ Session, cost_report, aggregate_reports };
use std::path::{ Path, PathBuf };

/// Write `lines` as `<dir>/<name>` and return a loaded [`Session`] for it.
fn session_with_lines( dir : &Path, name : &str, lines : &[ String ] ) -> Session
{
  let path : PathBuf = dir.join( name );
  std::fs::write( &path, lines.join( "\n" ) ).expect( "write session file" );
  Session::load( &path ).expect( "load session" )
}

/// Assistant entry with full usage. `ttl` adds the `cache_creation` TTL
/// breakdown object as `(ephemeral_5m, ephemeral_1h)`.
fn assistant_line(
  msg_id : &str,
  model : &str,
  input : u64,
  output : u64,
  cache_read : u64,
  cache_write : u64,
  ttl : Option< ( u64, u64 ) >,
) -> String
{
  let breakdown = match ttl
  {
    Some( ( c5m, c1h ) ) => format!(
      r#","cache_creation":{{"ephemeral_5m_input_tokens":{c5m},"ephemeral_1h_input_tokens":{c1h}}}"#
    ),
    None => String::new(),
  };
  format!(
    r#"{{"type":"assistant","timestamp":"2026-01-01T00:00:00Z","requestId":"req_{msg_id}","message":{{"role":"assistant","model":"{model}","id":"{msg_id}","content":[{{"type":"text","text":"x"}}],"usage":{{"input_tokens":{input},"output_tokens":{output},"cache_read_input_tokens":{cache_read},"cache_creation_input_tokens":{cache_write}{breakdown}}}}}}}"#
  )
}

/// Minimal user entry.
fn user_line( content : &str ) -> String
{
  format!( r#"{{"type":"user","timestamp":"2026-01-01T00:00:00Z","message":{{"role":"user","content":"{content}"}}}}"# )
}

/// Compaction marker entry, as Claude Code writes it.
fn compact_boundary_line() -> String
{
  r#"{"type":"system","subtype":"compact_boundary","compactMetadata":{"trigger":"auto","preTokens":150000},"timestamp":"2026-01-01T00:00:00Z"}"#.to_string()
}

/// Test `cost_report` basic single-model accounting
///
/// ## Purpose
/// Validates token sums, call count, and max-context tracking for the
/// simplest real shape: one model, several calls.
///
/// ## Coverage
/// Two assistant calls plus interleaved user entries; every `ModelUsage`
/// field plus `max_context_tokens` and `total_calls()`.
///
/// ## Validation Strategy
/// Exact equality on all sums; max context = largest input-side total.
///
/// ## Related Requirements
/// `docs/cli/command/15_cost.md` — token accounting
#[ test ]
fn cost_report_basic_single_model()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let session = session_with_lines( tmp.path(), "s1.jsonl", &[
    user_line( "q1" ),
    assistant_line( "msg_a", "claude-sonnet-5", 100, 50, 1000, 200, Some( ( 200, 0 ) ) ),
    user_line( "q2" ),
    assistant_line( "msg_b", "claude-sonnet-5", 10, 25, 2000, 300, Some( ( 100, 200 ) ) ),
  ] );

  let report = cost_report( &session ).expect( "report" );
  assert_eq!( report.session_id, "s1" );
  assert!( !report.is_agent_session );
  assert_eq!( report.compactions, 0 );
  assert_eq!( report.total_calls(), 2 );
  assert_eq!( report.models.len(), 1 );
  let m = &report.models[ 0 ];
  assert_eq!( m.model, "claude-sonnet-5" );
  assert_eq!( m.calls, 2 );
  assert_eq!( m.input_tokens, 110 );
  assert_eq!( m.output_tokens, 75 );
  assert_eq!( m.cache_read_tokens, 3000 );
  assert_eq!( m.cache_5m_write_tokens, 300 );
  assert_eq!( m.cache_1h_write_tokens, 200 );
  assert_eq!( m.cache_unknown_ttl_write_tokens, 0 );
  assert_eq!( m.cache_write_tokens(), 500 );
  assert_eq!( m.total_tokens(), 110 + 75 + 3000 + 500 );
  // Call b's input side: 10 + 2000 + 300 = 2310 > call a's 1300.
  assert_eq!( report.max_context_tokens, 2310 );
}

/// Test `cost_report` dedups by message.id
///
/// ## Purpose
/// Validates the `Fix(issue-038)` convention: one API response spanning
/// multiple JSONL lines (one per content block, same `message.id`) counts
/// once — calls and every token sum.
///
/// ## Coverage
/// Two lines sharing `message.id`/usage, then a distinct third line.
///
/// ## Validation Strategy
/// Asserts calls = 2 (not 3) and sums count the duplicate once.
///
/// ## Related Requirements
/// `Fix(issue-038)` — message.id dedup convention
#[ test ]
fn cost_report_dedup_by_message_id()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let dup = assistant_line( "msg_dup", "claude-sonnet-5", 100, 40, 500, 60, Some( ( 60, 0 ) ) );
  let session = session_with_lines( tmp.path(), "s2.jsonl", &[
    dup.clone(),
    dup,
    assistant_line( "msg_new", "claude-sonnet-5", 1, 2, 3, 4, Some( ( 4, 0 ) ) ),
  ] );

  let report = cost_report( &session ).expect( "report" );
  let m = &report.models[ 0 ];
  assert_eq!( m.calls, 2 );
  assert_eq!( m.input_tokens, 101 );
  assert_eq!( m.output_tokens, 42 );
  assert_eq!( m.cache_read_tokens, 503 );
  assert_eq!( m.cache_write_tokens(), 64 );
}

/// Test `cost_report` attributes usage per model
///
/// ## Purpose
/// Validates that a session switching models mid-way yields one bucket per
/// model — the reason `cost_report` exists instead of reusing
/// `Session::stats()`'s single first-seen `model` field.
///
/// ## Coverage
/// Calls on model A, then B, then A again; bucket order and per-bucket
/// sums.
///
/// ## Validation Strategy
/// Asserts two buckets ordered by first appearance with exact per-model
/// call/token sums.
///
/// ## Related Requirements
/// `docs/cli/command/15_cost.md` — per-model cost attribution
#[ test ]
fn cost_report_multi_model_attribution()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let session = session_with_lines( tmp.path(), "s3.jsonl", &[
    assistant_line( "msg_1", "claude-opus-4-6", 10, 1, 0, 0, None ),
    assistant_line( "msg_2", "claude-haiku-4-5-20251001", 20, 2, 0, 0, None ),
    assistant_line( "msg_3", "claude-opus-4-6", 30, 3, 0, 0, None ),
  ] );

  let report = cost_report( &session ).expect( "report" );
  assert_eq!( report.models.len(), 2 );
  assert_eq!( report.models[ 0 ].model, "claude-opus-4-6" );
  assert_eq!( report.models[ 0 ].calls, 2 );
  assert_eq!( report.models[ 0 ].input_tokens, 40 );
  assert_eq!( report.models[ 1 ].model, "claude-haiku-4-5-20251001" );
  assert_eq!( report.models[ 1 ].calls, 1 );
  assert_eq!( report.models[ 1 ].input_tokens, 20 );
}

/// Test `cost_report` skips synthetic placeholder entries
///
/// ## Purpose
/// Validates that `"model":"<synthetic>"` assistant entries (locally
/// generated placeholders, not API calls) contribute nothing — no bucket,
/// no call, no tokens.
///
/// ## Coverage
/// One synthetic entry between two real calls.
///
/// ## Validation Strategy
/// Asserts one bucket, 2 calls, and no `<synthetic>` model name anywhere.
///
/// ## Related Requirements
/// `docs/cli/command/15_cost.md` — synthetic entry exclusion
#[ test ]
fn cost_report_synthetic_skipped()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let session = session_with_lines( tmp.path(), "s4.jsonl", &[
    assistant_line( "msg_r1", "claude-sonnet-5", 10, 1, 0, 0, None ),
    assistant_line( "e0e0e0e0-1111-2222-3333-444444444444", "<synthetic>", 0, 0, 0, 0, None ),
    assistant_line( "msg_r2", "claude-sonnet-5", 20, 2, 0, 0, None ),
  ] );

  let report = cost_report( &session ).expect( "report" );
  assert_eq!( report.models.len(), 1 );
  assert_eq!( report.total_calls(), 2 );
  assert!( report.models.iter().all( | m | m.model != "<synthetic>" ) );
}

/// Test `cost_report` splits cache writes by TTL
///
/// ## Purpose
/// Validates the 5m/1h TTL split — the two buckets bill at different
/// multipliers (1.25x vs 2x input rate), so they must never be merged.
///
/// ## Coverage
/// A call with both TTL buckets populated and consistent with the total.
///
/// ## Validation Strategy
/// Exact bucket equality plus the `cache_write_tokens() == total` invariant.
///
/// ## Related Requirements
/// `docs/cli/command/15_cost.md` — cache TTL pricing
#[ test ]
fn cost_report_cache_ttl_split()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let session = session_with_lines( tmp.path(), "s5.jsonl", &[
    assistant_line( "msg_t", "claude-fable-5", 5, 5, 0, 900, Some( ( 300, 600 ) ) ),
  ] );

  let report = cost_report( &session ).expect( "report" );
  let m = &report.models[ 0 ];
  assert_eq!( m.cache_5m_write_tokens, 300 );
  assert_eq!( m.cache_1h_write_tokens, 600 );
  assert_eq!( m.cache_unknown_ttl_write_tokens, 0 );
  assert_eq!( m.cache_write_tokens(), 900 );
}

/// Test `cost_report` routes unaccounted cache writes to the unknown bucket
///
/// ## Purpose
/// Validates that cache writes without a TTL breakdown are never guessed
/// into a TTL bucket — absent `cache_creation` object (older transcript
/// format) and under-reporting buckets both land in unknown.
///
/// ## Coverage
/// One call with no breakdown object; one whose buckets sum below the
/// total.
///
/// ## Validation Strategy
/// Asserts unknown-bucket size per call shape and the write-total
/// invariant.
///
/// ## Related Requirements
/// `docs/cli/command/15_cost.md` — unknown-TTL handling
#[ test ]
fn cost_report_cache_ttl_missing_breakdown_goes_unknown()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let session = session_with_lines( tmp.path(), "s6.jsonl", &[
    assistant_line( "msg_old", "claude-3-5-sonnet-20241022", 5, 5, 0, 400, None ),
    assistant_line( "msg_gap", "claude-3-5-sonnet-20241022", 5, 5, 0, 100, Some( ( 30, 20 ) ) ),
  ] );

  let report = cost_report( &session ).expect( "report" );
  let m = &report.models[ 0 ];
  assert_eq!( m.cache_5m_write_tokens, 30 );
  assert_eq!( m.cache_1h_write_tokens, 20 );
  assert_eq!( m.cache_unknown_ttl_write_tokens, 400 + 50 );
  assert_eq!( m.cache_write_tokens(), 500 );
}

/// Test `cost_report` counts compaction boundaries
///
/// ## Purpose
/// Validates compaction counting via the real marker Claude Code writes:
/// `"type":"system","subtype":"compact_boundary"`.
///
/// ## Coverage
/// Two boundary entries interleaved with conversation entries.
///
/// ## Validation Strategy
/// Asserts `compactions == 2`.
///
/// ## Related Requirements
/// `docs/cli/command/15_cost.md` — Compact column
#[ test ]
fn cost_report_compaction_boundary_counted()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let session = session_with_lines( tmp.path(), "s7.jsonl", &[
    assistant_line( "msg_1", "claude-sonnet-5", 1, 1, 0, 0, None ),
    compact_boundary_line(),
    assistant_line( "msg_2", "claude-sonnet-5", 1, 1, 0, 0, None ),
    compact_boundary_line(),
  ] );

  let report = cost_report( &session ).expect( "report" );
  assert_eq!( report.compactions, 2 );
}

/// Test `cost_report` ignores compaction markers quoted inside content
///
/// ## Purpose
/// Validates the parse-based check is immune to false positives: a message
/// merely *mentioning* the marker text (as an escaped string inside
/// content) must not count — a raw substring scan would miscount here.
///
/// ## Coverage
/// A user entry whose content embeds the escaped marker text; zero real
/// boundaries.
///
/// ## Validation Strategy
/// Asserts `compactions == 0`.
///
/// ## Related Requirements
/// `docs/cli/command/15_cost.md` — Compact column accuracy
#[ test ]
fn cost_report_compaction_mention_in_content_not_counted()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  // Inside the JSON string value the quotes are escaped — exactly how a
  // transcript quoting the marker text actually looks on disk.
  let mention = r#"{"type":"user","message":{"role":"user","content":"look: \"type\":\"system\",\"subtype\":\"compact_boundary\" is the marker"}}"#.to_string();
  let session = session_with_lines( tmp.path(), "s8.jsonl", &[
    mention,
    assistant_line( "msg_1", "claude-sonnet-5", 1, 1, 0, 0, None ),
  ] );

  let report = cost_report( &session ).expect( "report" );
  assert_eq!( report.compactions, 0 );
}

/// Test `cost_report` skips malformed lines gracefully
///
/// ## Purpose
/// Validates per-line graceful degradation (the `Fix(BUG-489)` convention):
/// garbage lines never fail the scan or poison the sums.
///
/// ## Coverage
/// A non-JSON line, an empty line, and a JSON line without `type`, around
/// one valid call; assistant without `usage`; assistant without `model`.
///
/// ## Validation Strategy
/// Asserts the report succeeds, the no-usage call still counts with zero
/// tokens, and the no-model call lands in the `unknown` bucket.
///
/// ## Related Requirements
/// Graceful degradation convention (`Fix(BUG-489)`/`Fix(BUG-508)`)
#[ test ]
fn cost_report_malformed_lines_skipped()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let session = session_with_lines( tmp.path(), "s9.jsonl", &[
    "this is not json {{{".to_string(),
    String::new(),
    r#"{"no_type_field":true}"#.to_string(),
    assistant_line( "msg_ok", "claude-sonnet-5", 7, 3, 0, 0, None ),
    r#"{"type":"assistant","message":{"role":"assistant","model":"claude-sonnet-5","id":"msg_nousage","content":[]}}"#.to_string(),
    r#"{"type":"assistant","message":{"role":"assistant","id":"msg_nomodel","content":[],"usage":{"input_tokens":9,"output_tokens":1,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}"#.to_string(),
  ] );

  let report = cost_report( &session ).expect( "report" );
  let sonnet = report.models.iter().find( | m | m.model == "claude-sonnet-5" ).expect( "sonnet bucket" );
  assert_eq!( sonnet.calls, 2, "no-usage call still counts as a call" );
  assert_eq!( sonnet.input_tokens, 7 );
  let unknown = report.models.iter().find( | m | m.model == "unknown" ).expect( "unknown bucket" );
  assert_eq!( unknown.calls, 1 );
  assert_eq!( unknown.input_tokens, 9 );
}

/// Test `cost_report` max context uses the input side only
///
/// ## Purpose
/// Validates the "window size" definition: `input + cache_read +
/// cache_write` per call — a huge output must not inflate it.
///
/// ## Coverage
/// A small-context call with large output vs a larger-context call with
/// tiny output.
///
/// ## Validation Strategy
/// Asserts the larger input side wins and output is excluded.
///
/// ## Related Requirements
/// `docs/cli/command/15_cost.md` — MaxCtx column
#[ test ]
fn cost_report_max_context_input_side_only()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let session = session_with_lines( tmp.path(), "s10.jsonl", &[
    assistant_line( "msg_bigout", "claude-sonnet-5", 100, 50_000, 0, 0, None ),
    assistant_line( "msg_bigctx", "claude-sonnet-5", 200, 1, 700, 100, Some( ( 100, 0 ) ) ),
  ] );

  let report = cost_report( &session ).expect( "report" );
  assert_eq!( report.max_context_tokens, 1000 );
}

/// Test `aggregate_reports` folds a family into one conversation
///
/// ## Purpose
/// Validates family aggregation: same-model buckets merge, distinct models
/// append in first-appearance order, compactions sum, max context takes the
/// largest single value, and agent files are counted.
///
/// ## Coverage
/// A root report plus two agent reports (one sharing the root's model, one
/// on a different model); every `ConversationUsage` total method.
///
/// ## Validation Strategy
/// Exact equality on merged buckets, counts, and totals.
///
/// ## Related Requirements
/// `docs/cli/command/15_cost.md` — agent fold-in
#[ test ]
fn aggregate_reports_folds_family()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let root = cost_report( &session_with_lines( tmp.path(), "root.jsonl", &[
    assistant_line( "msg_r", "claude-fable-5", 100, 10, 1000, 200, Some( ( 0, 200 ) ) ),
    compact_boundary_line(),
  ] ) ).expect( "root report" );
  let agent_a = cost_report( &session_with_lines( tmp.path(), "agent-a.jsonl", &[
    assistant_line( "msg_a", "claude-fable-5", 50, 5, 4000, 100, Some( ( 100, 0 ) ) ),
  ] ) ).expect( "agent a report" );
  let agent_b = cost_report( &session_with_lines( tmp.path(), "agent-b.jsonl", &[
    assistant_line( "msg_b", "claude-haiku-4-5-20251001", 30, 3, 0, 0, None ),
    compact_boundary_line(),
  ] ) ).expect( "agent b report" );

  let usage = aggregate_reports( "root", &[ root, agent_a, agent_b ] );
  assert_eq!( usage.root_id, "root" );
  assert_eq!( usage.agent_count, 2 );
  assert_eq!( usage.compactions, 2 );
  assert_eq!( usage.max_context_tokens, 50 + 4000 + 100 );
  assert_eq!( usage.models.len(), 2 );
  assert_eq!( usage.models[ 0 ].model, "claude-fable-5" );
  assert_eq!( usage.models[ 0 ].calls, 2 );
  assert_eq!( usage.models[ 0 ].input_tokens, 150 );
  assert_eq!( usage.models[ 0 ].cache_5m_write_tokens, 100 );
  assert_eq!( usage.models[ 0 ].cache_1h_write_tokens, 200 );
  assert_eq!( usage.models[ 1 ].model, "claude-haiku-4-5-20251001" );
  assert_eq!( usage.total_calls(), 3 );
  assert_eq!( usage.total_input_tokens(), 180 );
  assert_eq!( usage.total_output_tokens(), 18 );
  assert_eq!( usage.total_cache_read_tokens(), 5000 );
  assert_eq!( usage.total_cache_write_tokens(), 300 );
  assert_eq!( usage.total_tokens(), 180 + 18 + 5000 + 300 );
}

/// Test `aggregate_reports` with the root alone
///
/// ## Purpose
/// Validates the degenerate (agents::0 / no-agents) case: aggregation of a
/// single non-agent report is the identity, with `agent_count == 0`.
///
/// ## Coverage
/// One root report, no agents.
///
/// ## Validation Strategy
/// Asserts `agent_count == 0` and totals equal the root's own.
///
/// ## Related Requirements
/// `docs/cli/command/15_cost.md` — agents::0
#[ test ]
fn aggregate_reports_root_only()
{
  let tmp = tempfile::tempdir().expect( "tempdir" );
  let root = cost_report( &session_with_lines( tmp.path(), "solo.jsonl", &[
    assistant_line( "msg_s", "claude-sonnet-5", 11, 22, 33, 44, Some( ( 44, 0 ) ) ),
  ] ) ).expect( "root report" );

  let usage = aggregate_reports( "solo", &[ root ] );
  assert_eq!( usage.agent_count, 0 );
  assert_eq!( usage.total_calls(), 1 );
  assert_eq!( usage.total_tokens(), 11 + 22 + 33 + 44 );
}

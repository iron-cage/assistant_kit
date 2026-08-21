//! Integration tests for the `.cost` command.
//!
//! ## Source
//!
//! - Command spec: `tests/docs/cli/command/15_cost.md`
//! - Param specs: `tests/docs/cli/param/39_session_ids.md`,
//!   `tests/docs/cli/param/40_agents.md`
//!
//! ## Coverage
//!
//! INT-1 through INT-14 per `tests/docs/cli/command/15_cost.md` — default
//! current-conversation resolution (single row, no TOTAL), multi-conversation
//! selection with the TOTAL row, exact/prefix session ID resolution across
//! projects (ambiguous and unknown IDs rejected), agent fold-in across BOTH
//! agent layouts (hierarchical `subagents/` and flat `agent-*.jsonl`) with
//! `agents::0` opt-out, argument validation (`agents::`, empty
//! `session_ids::`), the golden pricing example (cache-TTL split,
//! unknown-TTL fallback, multi-model summation, `<synthetic>` skip,
//! compaction count, max context), the unpriced-model footnote, `path::`
//! anchoring, duplicate-request collapse, and the cross-project
//! duplicate-ID richest-copy tie-break (`Fix(BUG-528)` convention).
//!
//! Core aggregation arithmetic (dedup by `message.id`, TTL clamping, model
//! bucket ordering) is covered line-by-line in
//! `claude_storage_core/tests/cost_report_test.rs`; these tests exercise the
//! CLI contract end-to-end through the binary.
//!
//! ## Test Case Index
//!
//! | ID | Test Name | Category |
//! |----|-----------|----------|
//! | INT-1 | `cost_int_1_default_current_single_row_no_total` | Default Resolution |
//! | INT-2 | `cost_int_2_multi_row_total_row` | Multi-Conversation |
//! | INT-3 | `cost_int_3_unique_prefix_resolves` | ID Resolution |
//! | INT-4 | `cost_int_4_ambiguous_prefix_rejected` | ID Resolution |
//! | INT-5 | `cost_int_5_unknown_id_rejected` | ID Resolution |
//! | INT-6 | `cost_int_6_agents_folded_by_default` | Agent Fold-In |
//! | INT-7 | `cost_int_7_agents_zero_root_only` | Agent Fold-In |
//! | INT-8 | `cost_int_8_agents_invalid_rejected` | Input Validation |
//! | INT-9 | `cost_int_9_empty_session_ids_rejected` | Input Validation |
//! | INT-10 | `cost_int_10_golden_pricing_ttl_synthetic_compaction` | Pricing |
//! | INT-11 | `cost_int_11_no_project_exits_2` | Exit Codes |
//! | INT-12 | `cost_int_12_path_parameter_selects_project` | Default Resolution |
//! | INT-13 | `cost_int_13_duplicate_requests_collapse` | ID Resolution |
//! | INT-14 | `cost_int_14_cross_project_duplicate_picks_richest` | ID Resolution |

mod common;

use tempfile::TempDir;

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

/// Expected `.cost` table line, using the command's exact column widths
/// (12/6/7/14×5/11/7/10, two-space separators). Building expectations
/// through the same format machinery keeps row assertions byte-exact
/// without hand-counting padding spaces.
#[ allow( clippy::too_many_arguments ) ]
fn row
(
  label : &str,
  agents : &str,
  req : &str,
  input : &str,
  output : &str,
  cache_r : &str,
  cache_w : &str,
  total : &str,
  max_ctx : &str,
  compact : &str,
  cost : &str,
) -> String
{
  format!(
    "{label:<12}  {agents:>6}  {req:>7}  {input:>14}  {output:>14}  {cache_r:>14}  {cache_w:>14}  {total:>14}  {max_ctx:>11}  {compact:>7}  {cost:>10}"
  )
}

/// Count table body lines: everything after the header, up to the first
/// footnote (`note:`) or price-date (`Cost:`) line. A TOTAL row counts as a
/// body line.
fn data_rows( s : &str ) -> usize
{
  s.lines()
    .skip( 1 )
    .take_while( | line | !line.starts_with( "note:" ) && !line.starts_with( "Cost:" ) )
    .count()
}

/// Write a session whose cost-relevant content is fully line-controlled,
/// auto-encoding the project path. Returns the encoded project ID.
///
/// Deliberately local rather than extending `common::write_test_session`:
/// `.cost` fixtures need per-line control over usage fields no other
/// command reads (cache-TTL breakdown, `<synthetic>` model, compaction
/// markers), same local-fixture precedent `cli_cmd_rollup_test.rs`'s
/// `write_rollup_session` establishes.
fn write_cost_session
(
  storage_root : &std::path::Path,
  project_path : &std::path::Path,
  session_id : &str,
  lines : &[ String ],
) -> String
{
  use std::io::Write as _;

  let encoded = claude_storage_core::encode_path( project_path )
    .expect( "encode project path" );
  let dir = storage_root.join( "projects" ).join( &encoded );
  std::fs::create_dir_all( &dir ).expect( "create project dir" );
  let path = dir.join( format!( "{session_id}.jsonl" ) );
  let mut file = std::fs::File::create( &path ).expect( "create session file" );
  for line in lines
  {
    writeln!( file, "{line}" ).expect( "write session line" );
  }
  encoded
}

/// Minimal leading user entry (fully valid for `Entry::from_json_line`).
fn user_line( session_id : &str ) -> String
{
  format!(
    r#"{{"type":"user","uuid":"u-000","parentUuid":null,"timestamp":"2025-01-01T00:00:00Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"human","isSidechain":false,"message":{{"role":"user","content":"hello"}}}}"#
  )
}

/// Assistant entry with fully-controlled usage. `ttl` supplies the
/// `usage.cache_creation` 5m/1h breakdown; `None` omits the object entirely
/// (older transcript format — the unknown-TTL path).
#[ allow( clippy::too_many_arguments ) ]
fn assistant_line
(
  session_id : &str,
  msg_id : &str,
  model : &str,
  input : u64,
  output : u64,
  cache_read : u64,
  cache_write : u64,
  ttl : Option< ( u64, u64 ) >,
) -> String
{
  let ttl_json = match ttl
  {
    Some( ( c5m, c1h ) ) =>
      format!( r#","cache_creation":{{"ephemeral_5m_input_tokens":{c5m},"ephemeral_1h_input_tokens":{c1h}}}"# ),
    None => String::new(),
  };
  format!(
    r#"{{"type":"assistant","uuid":"a-{msg_id}","parentUuid":"u-000","timestamp":"2025-01-01T00:00:01Z","cwd":"/tmp","sessionId":"{session_id}","version":"2.0.0","gitBranch":"master","userType":"external","isSidechain":false,"requestId":"req_{msg_id}","message":{{"role":"assistant","model":"{model}","id":"{msg_id}","content":[{{"type":"text","text":"x"}}],"stop_reason":"end_turn","stop_sequence":null,"usage":{{"input_tokens":{input},"output_tokens":{output},"cache_read_input_tokens":{cache_read},"cache_creation_input_tokens":{cache_write}{ttl_json}}}}}}}"#
  )
}

/// Compaction marker: `system`/`compact_boundary` entry as Claude Code
/// writes it.
fn compact_boundary_line( session_id : &str ) -> String
{
  format!(
    r#"{{"type":"system","subtype":"compact_boundary","uuid":"cb-000","timestamp":"2025-01-01T00:00:02Z","sessionId":"{session_id}","compactMetadata":{{"trigger":"auto","preTokens":150000}}}}"#
  )
}

/// INT-1: Bare `.cost` reports the current directory's most recent
/// conversation as a single row without a TOTAL row.
///
/// ## Purpose
/// Validates the default selection path (`session_ids::` omitted → cwd's
/// project → most recent session) and the single-conversation table shape:
/// exact header, one body row, no TOTAL, unpriced-model footnote, price
/// note.
///
/// ## Coverage
/// Exit 0; byte-exact header and row (`claude-test` fixture: 2 calls,
/// 20/10/0/0 tokens, max ctx 10, $0.00); `TOTAL` absent; footnote names
/// `claude-test`; trailing price-date note present.
///
/// ## Validation Strategy
/// One 4-entry session via `common::write_path_project_session`; run bare
/// `.cost` from the project directory; compare against rows built with the
/// same width spec.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-1
#[ test ]
fn cost_int_1_default_current_single_row_no_total()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "work" );
  std::fs::create_dir_all( &project ).unwrap();

  common::write_path_project_session(
    &storage_root, &project, "sessaaa1-1111-4abc-9def-000000000001", 4 );

  let out = common::clg_cmd()
    .current_dir( &project )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let header = row( "Conversation", "Agents", "Req", "Input", "Output", "CacheR", "CacheW",
    "Total", "MaxCtx", "Compact", "Cost" );
  assert!( s.starts_with( &header ), "INT-1: exact header expected; got:\n{s}" );
  let expected = row( "sessaaa1", "0", "2", "20", "10", "0", "0", "30", "10", "0", "$0.00" );
  assert!( s.contains( &expected ), "INT-1: exact conversation row expected:\n{expected}\ngot:\n{s}" );
  assert_eq!( data_rows( &s ), 1, "INT-1: exactly one body row expected; got:\n{s}" );
  assert!( !s.contains( "TOTAL" ), "INT-1: no TOTAL row for a single conversation; got:\n{s}" );
  assert!(
    s.contains( "note: no pricing for model 'claude-test' — its tokens are excluded from Cost" ),
    "INT-1: unpriced-model footnote expected; got:\n{s}"
  );
  assert!(
    s.contains( "Cost: estimated at API list prices (2026-08-21); tokens are exact." ),
    "INT-1: price-date note expected; got:\n{s}"
  );
}

/// INT-2: Multiple `session_ids::` produce one row each, in request order,
/// plus a TOTAL row.
///
/// ## Purpose
/// Validates multi-conversation selection: request order (not mtime or
/// alphabetical) drives row order, and the TOTAL row sums every additive
/// column while showing `—` for the non-additive `MaxCtx`.
///
/// ## Coverage
/// Two conversations in different projects requested B-then-A; rows appear
/// B, A, TOTAL; TOTAL is byte-exact (Req 3, Input 300, Output 120, Total
/// 420, `MaxCtx` `—`); body-row count 3.
///
/// ## Validation Strategy
/// Fully line-controlled fixtures with distinct token totals; assert both
/// rows, their relative order, and the TOTAL row.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-2
#[ test ]
fn cost_int_2_multi_row_total_row()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project_a = root.path().join( "work_a" );
  let project_b = root.path().join( "work_b" );
  std::fs::create_dir_all( &project_a ).unwrap();
  std::fs::create_dir_all( &project_b ).unwrap();

  let id_a = "aaaa1111-2222-4333-8444-000000000001";
  let id_b = "bbbb2222-3333-4444-8555-000000000002";
  write_cost_session( &storage_root, &project_a, id_a, &[
    user_line( id_a ),
    assistant_line( id_a, "msg_a1", "claude-test", 100, 50, 0, 0, None ),
  ] );
  write_cost_session( &storage_root, &project_b, id_b, &[
    user_line( id_b ),
    assistant_line( id_b, "msg_b1", "claude-test", 150, 40, 0, 0, None ),
    assistant_line( id_b, "msg_b2", "claude-test", 50, 30, 0, 0, None ),
  ] );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .arg( format!( "session_ids::{id_b},{id_a}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let row_b = row( "bbbb2222", "0", "2", "200", "70", "0", "0", "270", "150", "0", "$0.00" );
  let row_a = row( "aaaa1111", "0", "1", "100", "50", "0", "0", "150", "100", "0", "$0.00" );
  let row_total = row( "TOTAL", "0", "3", "300", "120", "0", "0", "420", "—", "0", "$0.00" );
  let pos_b = s.find( &row_b ).unwrap_or_else( || panic!( "INT-2: row B expected:\n{row_b}\ngot:\n{s}" ) );
  let pos_a = s.find( &row_a ).unwrap_or_else( || panic!( "INT-2: row A expected:\n{row_a}\ngot:\n{s}" ) );
  let pos_t = s.find( &row_total ).unwrap_or_else( || panic!( "INT-2: TOTAL row expected:\n{row_total}\ngot:\n{s}" ) );
  assert!( pos_b < pos_a && pos_a < pos_t, "INT-2: rows must follow request order then TOTAL; got:\n{s}" );
  assert_eq!( data_rows( &s ), 3, "INT-2: two conversation rows + TOTAL expected; got:\n{s}" );
}

/// INT-3: A unique session ID prefix resolves to its conversation.
///
/// ## Purpose
/// Validates prefix matching: a short unique prefix selects the full
/// session without requiring the complete UUID.
///
/// ## Coverage
/// `session_ids::aaaa1111` (8 chars of a 36-char ID) exits 0 and reports
/// exactly that conversation.
///
/// ## Validation Strategy
/// Single session; request by prefix; assert exit 0, one row, short label.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-3
#[ test ]
fn cost_int_3_unique_prefix_resolves()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "work" );
  std::fs::create_dir_all( &project ).unwrap();

  let id = "aaaa1111-2222-4333-8444-000000000001";
  write_cost_session( &storage_root, &project, id, &[
    user_line( id ),
    assistant_line( id, "msg_p1", "claude-test", 10, 5, 0, 0, None ),
  ] );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .arg( "session_ids::aaaa1111" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "aaaa1111" ), "INT-3: resolved conversation row expected; got:\n{s}" );
  assert_eq!( data_rows( &s ), 1, "INT-3: exactly one body row expected; got:\n{s}" );
}

/// INT-4: An ambiguous session ID prefix is rejected naming every match.
///
/// ## Purpose
/// Validates the uniqueness requirement on prefix resolution: a prefix
/// matching several sessions is an error that lists the candidates rather
/// than silently picking one.
///
/// ## Coverage
/// Exit 1; stderr carries the exact sorted match list; no table on stdout.
///
/// ## Validation Strategy
/// Two sessions sharing the `aaaa` prefix in one project; request `aaaa`;
/// assert the full error message.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-4
#[ test ]
fn cost_int_4_ambiguous_prefix_rejected()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "work" );
  std::fs::create_dir_all( &project ).unwrap();

  let id_1 = "aaaa1111-2222-4333-8444-000000000001";
  let id_2 = "aaaa2222-2222-4333-8444-000000000002";
  for id in [ id_1, id_2 ]
  {
    write_cost_session( &storage_root, &project, id, &[
      user_line( id ),
      assistant_line( id, "msg_x1", "claude-test", 10, 5, 0, 0, None ),
    ] );
  }

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .arg( "session_ids::aaaa" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( &format!( "ambiguous session ID prefix 'aaaa': matches {id_1}, {id_2}" ) ),
    "INT-4: stderr must list every match in sorted order; got: {}",
    stderr( &out )
  );
  assert!( stdout( &out ).is_empty(), "INT-4: no table output expected; got:\n{}", stdout( &out ) );
}

/// INT-5: A session ID matching nothing is rejected.
///
/// ## Purpose
/// Validates the not-found path: an ID (or prefix) with zero matches names
/// the failing request rather than producing an empty table.
///
/// ## Coverage
/// Exit 1; stderr carries `Session not found: zzzz`; no table on stdout.
///
/// ## Validation Strategy
/// Valid empty storage; request `zzzz`; assert exit, stderr, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-5
#[ test ]
fn cost_int_5_unknown_id_rejected()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  std::fs::create_dir_all( storage_root.join( "projects" ) ).unwrap();

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .arg( "session_ids::zzzz" )
    .output()
    .unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "Session not found: zzzz" ),
    "INT-5: stderr must name the unmatched request; got: {}",
    stderr( &out )
  );
  assert!( stdout( &out ).is_empty(), "INT-5: no table output expected; got:\n{}", stdout( &out ) );
}

/// INT-6: Agent sessions from BOTH layouts fold into the conversation row
/// by default.
///
/// ## Purpose
/// Validates the fold-in default (`agents::1`): hierarchical
/// (`{uuid}/subagents/*.jsonl`) and flat (`agent-*.jsonl` associated via
/// first-entry `sessionId`) agent sessions all contribute to the root's
/// row, counted in the Agents column.
///
/// ## Coverage
/// Root (2 calls) + two hierarchical agents (2 calls each) + one flat agent
/// (2 calls): byte-exact row with Agents 3, Req 8, Input 80, Output 40.
///
/// ## Validation Strategy
/// `common::write_hierarchical_path_session` + `common::write_flat_agent_session`
/// fixtures (all `claude-test`, 10/5 per call); assert the exact folded row.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-6
#[ test ]
fn cost_int_6_agents_folded_by_default()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "fam" );
  std::fs::create_dir_all( &project ).unwrap();

  let root_id = "cccc3333-4444-4555-8666-000000000003";
  let encoded = common::write_hierarchical_path_session(
    &storage_root, &project, root_id, &[ ( "h1", "explore" ), ( "h2", "general-purpose" ) ], 4 );
  common::write_flat_agent_session( &storage_root, &encoded, "f1", root_id, 4 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .arg( format!( "session_ids::{root_id}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let expected = row( "cccc3333", "3", "8", "80", "40", "0", "0", "120", "10", "0", "$0.00" );
  assert!( s.contains( &expected ), "INT-6: exact folded row expected:\n{expected}\ngot:\n{s}" );
}

/// INT-7: `agents::0` reports the root session alone.
///
/// ## Purpose
/// Validates the fold-in opt-out: with `agents::0` the same family fixture
/// contributes only the root's usage and Agents shows 0.
///
/// ## Coverage
/// Same family as INT-6; byte-exact row with Agents 0, Req 2, Input 20,
/// Output 10.
///
/// ## Validation Strategy
/// Identical fixture, `agents::0`; assert the exact root-only row.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-7
#[ test ]
fn cost_int_7_agents_zero_root_only()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "fam" );
  std::fs::create_dir_all( &project ).unwrap();

  let root_id = "cccc3333-4444-4555-8666-000000000003";
  let encoded = common::write_hierarchical_path_session(
    &storage_root, &project, root_id, &[ ( "h1", "explore" ), ( "h2", "general-purpose" ) ], 4 );
  common::write_flat_agent_session( &storage_root, &encoded, "f1", root_id, 4 );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .arg( format!( "session_ids::{root_id}" ) )
    .arg( "agents::0" )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let expected = row( "cccc3333", "0", "2", "20", "10", "0", "0", "30", "10", "0", "$0.00" );
  assert!( s.contains( &expected ), "INT-7: exact root-only row expected:\n{expected}\ngot:\n{s}" );
}

/// INT-8: `agents::` outside `0`/`1` is rejected.
///
/// ## Purpose
/// Validates boolean-parameter validation (Finding #010 convention):
/// defaults do not exempt a parameter from explicit range checking.
///
/// ## Coverage
/// Exit 1; stderr carries `agents must be 0 or 1`; no table on stdout.
///
/// ## Validation Strategy
/// Bare `.cost agents::2` (validation precedes storage access — no fixture
/// needed); assert exit, stderr, empty stdout.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-8
#[ test ]
fn cost_int_8_agents_invalid_rejected()
{
  let out = common::clg_cmd().arg( ".cost" ).arg( "agents::2" ).output().unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "agents must be 0 or 1" ),
    "INT-8: stderr must name the constraint; got: {}",
    stderr( &out )
  );
  assert!( stdout( &out ).is_empty(), "INT-8: no table output expected; got:\n{}", stdout( &out ) );
}

/// INT-9: `session_ids::` with no non-empty ID is rejected.
///
/// ## Purpose
/// Validates emptiness checking on the ID list: a value that trims/splits
/// to nothing (e.g. a lone comma) is an argument error, raised before any
/// storage access.
///
/// ## Coverage
/// Exit 1; stderr carries `session_ids must contain at least one session
/// ID`; no table on stdout.
///
/// ## Validation Strategy
/// Bare `.cost session_ids::,` with no storage env at all — passing proves
/// the check precedes storage access.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-9
#[ test ]
fn cost_int_9_empty_session_ids_rejected()
{
  let out = common::clg_cmd().arg( ".cost" ).arg( "session_ids::," ).output().unwrap();

  assert_exit( &out, 1 );
  assert!(
    stderr( &out ).contains( "session_ids must contain at least one session ID" ),
    "INT-9: stderr must name the emptiness constraint; got: {}",
    stderr( &out )
  );
  assert!( stdout( &out ).is_empty(), "INT-9: no table output expected; got:\n{}", stdout( &out ) );
}

/// INT-10: Golden pricing example — cache-TTL split, unknown-TTL fallback,
/// multi-model summation, `<synthetic>` skip, compaction, max context.
///
/// ## Purpose
/// Validates the full token→USD path against hand-computed arithmetic at
/// published list prices, plus every scanner special case in one
/// conversation: 5m/1h TTL buckets billed at their own multipliers, a
/// TTL-less write billed at the 5m (API default TTL) rate, a second model
/// summed into the same row, a `<synthetic>` entry contributing nothing,
/// a compaction marker counted, and `MaxCtx` taking the largest single call.
///
/// ## Coverage
/// haiku-4-5: 1M in ($1.00) + 200k out ($1.00) + 3M read ($0.30) + 400k 5m
/// write ($0.50) + 100k 1h write ($0.20) + 200k unknown-TTL write at 5m
/// rate ($0.25) = $3.25; sonnet-5: 500k in ($1.00) + 100k out ($1.00) =
/// $2.00; row total $5.25. Req 3 (synthetic excluded), Input 1,500,000,
/// `CacheW` 700,000, Total 5,500,000, `MaxCtx` 4,500,000, Compact 1. No
/// footnote (every priced).
///
/// ## Validation Strategy
/// Line-controlled fixture; byte-exact row comparison; assert `note:`
/// absent and the price-date line present.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-10
#[ test ]
fn cost_int_10_golden_pricing_ttl_synthetic_compaction()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "golden" );
  std::fs::create_dir_all( &project ).unwrap();

  let id = "dddd4444-5555-4666-8777-000000000004";
  write_cost_session( &storage_root, &project, id, &[
    user_line( id ),
    assistant_line( id, "msg_g1", "claude-haiku-4-5-20251001", 1_000_000, 200_000, 3_000_000, 500_000,
      Some( ( 400_000, 100_000 ) ) ),
    assistant_line( id, "msg_g2", "claude-haiku-4-5-20251001", 0, 0, 0, 200_000, None ),
    assistant_line( id, "msg_g3", "claude-sonnet-5-20250929", 500_000, 100_000, 0, 0, None ),
    assistant_line( id, "11111111-2222-4333-8444-555555555555", "<synthetic>", 999_999, 999, 0, 0, None ),
    compact_boundary_line( id ),
  ] );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .arg( format!( "session_ids::{id}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let expected = row( "dddd4444", "0", "3", "1,500,000", "300,000", "3,000,000", "700,000",
    "5,500,000", "4,500,000", "1", "$5.25" );
  assert!( s.contains( &expected ), "INT-10: exact golden row expected:\n{expected}\ngot:\n{s}" );
  assert!( !s.contains( "note:" ), "INT-10: no unpriced-model footnote expected; got:\n{s}" );
  assert!(
    s.contains( "Cost: estimated at API list prices (2026-08-21); tokens are exact." ),
    "INT-10: price-date note expected; got:\n{s}"
  );
}

/// INT-11: Bare `.cost` in a directory with no project exits 2.
///
/// ## Purpose
/// Validates the "not found = usage error" convention shared with
/// `.usage`/`.rollup`: default resolution failing to find a project for the
/// cwd is exit 2 with a stderr message, not a hard error or empty table.
///
/// ## Coverage
/// Exit 2; stderr carries `No project found for current directory`.
///
/// ## Validation Strategy
/// Valid empty storage, cwd with no project, bare `.cost`; assert exit 2
/// and the stderr message.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-11
#[ test ]
fn cost_int_11_no_project_exits_2()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  std::fs::create_dir_all( storage_root.join( "projects" ) ).unwrap();

  let out = common::clg_cmd()
    .current_dir( root.path() )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .output()
    .unwrap();

  assert_exit( &out, 2 );
  assert!(
    stderr( &out ).contains( "No project found for current directory" ),
    "INT-11: stderr must name the missing cwd project; got: {}",
    stderr( &out )
  );
}

/// INT-12: `path::` anchors default resolution to another project.
///
/// ## Purpose
/// Validates the `path::` parameter: with `session_ids::` omitted, the
/// reported conversation is the most recent session of the project owning
/// `path::`, independent of the process's own cwd.
///
/// ## Coverage
/// Run from an unrelated cwd; exit 0; the `path::` project's session is the
/// single reported row.
///
/// ## Validation Strategy
/// One project with one session; run `.cost path::<project>` from the temp
/// root; assert the row label.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-12
#[ test ]
fn cost_int_12_path_parameter_selects_project()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "elsewhere" );
  std::fs::create_dir_all( &project ).unwrap();

  let id = "eeee5555-6666-4777-8888-000000000005";
  write_cost_session( &storage_root, &project, id, &[
    user_line( id ),
    assistant_line( id, "msg_e1", "claude-test", 10, 5, 0, 0, None ),
  ] );

  let out = common::clg_cmd()
    .current_dir( root.path() )
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .arg( format!( "path::{}", project.to_str().unwrap() ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert!( s.contains( "eeee5555" ), "INT-12: path-selected conversation row expected; got:\n{s}" );
  assert_eq!( data_rows( &s ), 1, "INT-12: exactly one body row expected; got:\n{s}" );
}

/// INT-13: Duplicate requests for one conversation collapse to one row.
///
/// ## Purpose
/// Validates request deduplication: naming the same conversation twice
/// (exactly or via prefix) yields one row and therefore no TOTAL row —
/// never double-counted usage.
///
/// ## Coverage
/// `session_ids::<id>,<id>` produces exactly one body row and no TOTAL.
///
/// ## Validation Strategy
/// Single session requested twice by full ID; assert row count and TOTAL
/// absence.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-13
#[ test ]
fn cost_int_13_duplicate_requests_collapse()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project = root.path().join( "work" );
  std::fs::create_dir_all( &project ).unwrap();

  let id = "aaaa1111-2222-4333-8444-000000000001";
  write_cost_session( &storage_root, &project, id, &[
    user_line( id ),
    assistant_line( id, "msg_d1", "claude-test", 10, 5, 0, 0, None ),
  ] );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .arg( format!( "session_ids::{id},{id}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  assert_eq!( data_rows( &s ), 1, "INT-13: duplicate requests must collapse to one row; got:\n{s}" );
  assert!( !s.contains( "TOTAL" ), "INT-13: no TOTAL row for a single collapsed conversation; got:\n{s}" );
}

/// INT-14: A session ID duplicated across projects resolves to the richest
/// copy.
///
/// ## Purpose
/// Validates the `Fix(BUG-528)` tie-break: when one session ID exists in
/// several project directories (git-worktree-style forked history), the
/// copy with the greatest entry count is reported — never both, never the
/// poorer one.
///
/// ## Coverage
/// Same ID in two projects (1-call vs 3-call copies); byte-exact row shows
/// the 3-call copy's numbers (Req 3, Input 30); exactly one body row.
///
/// ## Validation Strategy
/// Two fixtures sharing an ID with distinct call counts; request the ID;
/// assert the richer copy's exact row.
///
/// ## Related Requirements
/// `tests/docs/cli/command/15_cost.md` — INT-14
#[ test ]
fn cost_int_14_cross_project_duplicate_picks_richest()
{
  let root = TempDir::new().unwrap();
  let storage_root = root.path().join( ".claude" );
  let project_poor = root.path().join( "poor" );
  let project_rich = root.path().join( "rich" );
  std::fs::create_dir_all( &project_poor ).unwrap();
  std::fs::create_dir_all( &project_rich ).unwrap();

  let id = "ffff6666-7777-4888-8999-000000000006";
  write_cost_session( &storage_root, &project_poor, id, &[
    user_line( id ),
    assistant_line( id, "msg_p1", "claude-test", 10, 5, 0, 0, None ),
  ] );
  write_cost_session( &storage_root, &project_rich, id, &[
    user_line( id ),
    assistant_line( id, "msg_r1", "claude-test", 10, 5, 0, 0, None ),
    assistant_line( id, "msg_r2", "claude-test", 10, 5, 0, 0, None ),
    assistant_line( id, "msg_r3", "claude-test", 10, 5, 0, 0, None ),
  ] );

  let out = common::clg_cmd()
    .env( "HOME", root.path().to_str().unwrap() )
    .env( "CLAUDE_STORAGE_ROOT", storage_root.to_str().unwrap() )
    .arg( ".cost" )
    .arg( format!( "session_ids::{id}" ) )
    .output()
    .unwrap();

  assert_exit( &out, 0 );
  let s = stdout( &out );
  let expected = row( "ffff6666", "0", "3", "30", "15", "0", "0", "45", "10", "0", "$0.00" );
  assert!( s.contains( &expected ), "INT-14: richest copy's exact row expected:\n{expected}\ngot:\n{s}" );
  assert_eq!( data_rows( &s ), 1, "INT-14: exactly one body row expected; got:\n{s}" );
}

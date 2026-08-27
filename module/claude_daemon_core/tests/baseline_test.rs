//! Tests for the static-baseline probe: its flags, its reading, and its cache.
//!
//! The probe spends a real API call, so `measure` itself is deliberately thin —
//! it builds an argument list, runs it, and hands the output to `parse_probe`.
//! Both of those halves are tested here directly, against the response schema
//! documented in `contract/claude_code/docs/format/007_json_response.md`. What is
//! left untested is the `Command::output()` call between them.
//!
//! ## Specification References
//!
//! - `docs/api/001_daemon_surface.md` — the `baseline` module
//! - `contract/claude_code/docs/format/007_json_response.md` — the response schema
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | bl01 | The probe's flags | Print, JSON, and no persisted session |
//! | bl02 | An explicit model | Passed through; absent otherwise |
//! | bl03 | A cache-heavy response | Cached tokens counted, not just uncached |
//! | bl04 | The documented minimal response | Read without cache fields present |
//! | bl05 | A response that is not JSON | `Probe`, not a floor of zero |
//! | bl06 | A response with no `usage` | `Probe`, not a floor of zero |
//! | bl07 | Store then load | Round-trips, keyed by version and model |
//! | bl08 | Two versions, then a re-measure | Kept apart; same key replaced |
//! | bl09 | A corrupt cache file | Empty, not an error |
//! | bl10 | Splitting a context figure | Conversation is what is left over |

use claude_daemon_core::{ Error, baseline };
use tempfile::TempDir;

/// A response carrying the usage fields a real cached call reports.
fn cached_response() -> String
{
  r#"{
    "id" : "msg_01AEieWYMdbGML9PEKCmB36v",
    "type" : "message",
    "role" : "assistant",
    "content" : [ { "type" : "text", "text" : "Hello" } ],
    "model" : "claude-sonnet-5",
    "stop_reason" : "end_turn",
    "stop_sequence" : null,
    "usage" :
    {
      "input_tokens" : 4,
      "output_tokens" : 8,
      "service_tier" : "standard",
      "cache_creation_input_tokens" : 1200,
      "cache_read_input_tokens" : 15800
    }
  }"#.to_string()
}

/// bl01: the probe asks for a printed JSON answer and leaves no session behind.
///
/// Each flag is load-bearing, and a silently dropped one fails in a way the
/// measurement itself would not show: without `--output-format json` there is no
/// usage object to read, and without `--no-session-persistence` every probe
/// leaves a session in the registry that no client asked for and that
/// `list_sessions` would then report.
#[ test ]
fn bl01_probe_prints_json_and_persists_nothing()
{
  let args = baseline::probe_args( None );

  assert!( args.contains( &"--print".to_string() ), "probe must not open a session: {args:?}" );
  assert!
  (
    args.windows( 2 ).any( | pair | pair == [ "--output-format", "json" ] ),
    "probe must ask for JSON, or there is no usage to read: {args:?}",
  );
  assert!
  (
    args.contains( &"--no-session-persistence".to_string() ),
    "probe must not leave a session behind: {args:?}",
  );
  assert_eq!( args.last().map( String::as_str ), Some( baseline::PROBE_PROMPT ) );
}

/// bl02: a model is passed through when named, and left to the default otherwise.
///
/// The floor differs between models, so a measurement is keyed to one. Asking
/// for a specific model is how a caller measures a model it is not currently
/// running.
#[ test ]
fn bl02_model_is_passed_through_only_when_named()
{
  let with_model = baseline::probe_args( Some( "claude-haiku-4-5-20251001" ) );
  assert!
  (
    with_model.windows( 2 ).any( | pair | pair == [ "--model", "claude-haiku-4-5-20251001" ] ),
    "named model must reach the command line: {with_model:?}",
  );

  let without = baseline::probe_args( None );
  assert!( !without.contains( &"--model".to_string() ), "unnamed model must not be guessed" );
}

/// bl03: the floor counts cached tokens, which is nearly all of it.
///
/// The static prompt is identical every call and so is exactly what caching
/// captures. Reading `input_tokens` alone would put this floor at 4 tokens
/// rather than `17_004` — a four-thousand-fold error, and one that would look
/// plausible rather than obviously broken.
#[ test ]
fn bl03_floor_counts_cached_tokens()
{
  let baseline = baseline::parse_probe( "2.1.220", &cached_response() )
    .expect( "a well-formed response should parse" );

  assert_eq!( baseline.prompt_tokens, 17_004, "input + cache_read + cache_creation" );
  assert_ne!
  (
    baseline.prompt_tokens, baseline.input_tokens,
    "if these were equal the sum would not be under test",
  );
  assert_eq!( baseline.version, "2.1.220" );
  assert_eq!( baseline.model, "claude-sonnet-5", "keyed to the model that answered" );
}

/// bl04: the documented minimal response reads without cache fields present.
///
/// `cache_read_input_tokens` and `cache_creation_input_tokens` are marked "not
/// always present" in the response schema — omitted entirely when caching did
/// not apply. That is a real measurement of an uncached call, not a missing one,
/// so it must read as zero rather than fail.
#[ test ]
fn bl04_minimal_response_reads_without_cache_fields()
{
  let minimal = r#"{
    "id" : "msg_01AEieWYMdbGML9PEKCmB36v",
    "type" : "message",
    "role" : "assistant",
    "content" : [ { "type" : "text", "text" : "Hello world" } ],
    "model" : "claude-haiku-4-5-20251001",
    "stop_reason" : "end_turn",
    "stop_sequence" : null,
    "usage" : { "input_tokens" : 10, "output_tokens" : 5 }
  }"#;

  let baseline = baseline::parse_probe( "2.1.220", minimal ).expect( "minimal response should parse" );

  assert_eq!( baseline.prompt_tokens, 10 );
  assert_eq!( baseline.cache_read_tokens, 0 );
  assert_eq!( baseline.cache_creation_tokens, 0 );
}

/// bl05: output that is not a response is a failed measurement, not a zero one.
///
/// `claude` prints diagnostics and login prompts on stdout too. Reading one as a
/// floor of zero would report a session's entire context as conversation, and
/// the number would look ordinary.
#[ test ]
fn bl05_unparseable_output_is_a_failure()
{
  match baseline::parse_probe( "2.1.220", "Please run `claude login` first." )
  {
    Err( Error::Probe { reason } ) => assert!
    (
      reason.contains( "not JSON" ),
      "the reason should say what was wrong: {reason}",
    ),
    other => panic!( "expected Probe, got {other:?}" ),
  }
}

/// bl06: a response with no usage object is a failed measurement.
///
/// Well-formed JSON that happens to carry no usage is the same hazard as bl05
/// wearing a better disguise — it parses, so only an explicit check catches it.
#[ test ]
fn bl06_response_without_usage_is_a_failure()
{
  let no_usage = r#"{ "type" : "message", "model" : "claude-sonnet-5", "content" : [] }"#;

  match baseline::parse_probe( "2.1.220", no_usage )
  {
    Err( Error::Probe { reason } ) => assert!
    (
      reason.contains( "usage" ),
      "the reason should name what was missing: {reason}",
    ),
    other => panic!( "expected Probe, got {other:?}" ),
  }
}

/// bl07: a stored measurement comes back.
#[ test ]
fn bl07_measurement_round_trips_through_the_cache()
{
  let dir = TempDir::new().expect( "temp dir" );
  let runtime = dir.path().join( "-daemon" );

  let measured = baseline::parse_probe( "2.1.220", &cached_response() ).expect( "parse" );
  baseline::store( &runtime, &measured ).expect( "store should create the runtime dir" );

  let loaded = baseline::load( &runtime, "2.1.220", "claude-sonnet-5" );
  assert_eq!( loaded.as_ref(), Some( &measured ) );
}

/// bl08: measurements are kept apart by version, and replaced within one.
///
/// The floor moves when Claude Code changes what it puts in the system prompt,
/// so a measurement of one version must never answer for another. Re-measuring
/// the same version is the opposite case: that is a correction, and keeping both
/// would leave the stale one to be found first.
#[ test ]
fn bl08_versions_kept_apart_and_remeasures_replace()
{
  let dir = TempDir::new().expect( "temp dir" );
  let runtime = dir.path().join( "-daemon" );

  let old = baseline::parse_probe( "2.1.219", &cached_response() ).expect( "parse" );
  let new = baseline::parse_probe( "2.1.220", &cached_response() ).expect( "parse" );
  baseline::store( &runtime, &old ).expect( "store old" );
  baseline::store( &runtime, &new ).expect( "store new" );

  assert_eq!( baseline::load_all( &runtime ).len(), 2, "two versions must both survive" );

  // Re-measuring 2.1.220 corrects it rather than accumulating beside it.
  let corrected = baseline::parse_probe
  (
    "2.1.220",
    r#"{ "model" : "claude-sonnet-5", "usage" : { "input_tokens" : 99 } }"#,
  ).expect( "parse" );
  baseline::store( &runtime, &corrected ).expect( "store corrected" );

  assert_eq!( baseline::load_all( &runtime ).len(), 2, "a re-measure must replace, not accumulate" );
  assert_eq!
  (
    baseline::load( &runtime, "2.1.220", "claude-sonnet-5" ).map( | one | one.prompt_tokens ),
    Some( 99 ),
    "the newer measurement must be the one found",
  );
  assert_eq!
  (
    baseline::load( &runtime, "2.1.219", "claude-sonnet-5" ).map( | one | one.prompt_tokens ),
    Some( 17_004 ),
    "the other version must be untouched",
  );
}

/// bl09: an unreadable cache costs a re-measurement, never a failed request.
///
/// The cache is a memo. A truncated or hand-edited file should send the next
/// caller back to measuring, not turn every context summary into an error.
#[ test ]
fn bl09_corrupt_cache_reads_as_empty()
{
  let dir = TempDir::new().expect( "temp dir" );
  let runtime = dir.path().join( "-daemon" );
  std::fs::create_dir_all( &runtime ).expect( "create runtime dir" );
  std::fs::write( baseline::cache_path( &runtime ), "{ this is not json" ).expect( "write" );

  assert!( baseline::load_all( &runtime ).is_empty() );
  assert!( baseline::load( &runtime, "2.1.220", "claude-sonnet-5" ).is_none() );

  // And it is recoverable in place — a corrupt file must not wedge the cache.
  let measured = baseline::parse_probe( "2.1.220", &cached_response() ).expect( "parse" );
  baseline::store( &runtime, &measured ).expect( "store over a corrupt cache" );
  assert_eq!( baseline::load( &runtime, "2.1.220", "claude-sonnet-5" ).as_ref(), Some( &measured ) );
}

/// bl10: splitting a context figure leaves the conversation.
///
/// This is what the whole measurement is for: `17_004` of a `20_000`-token
/// context was spent before the first word, so the conversation is `2_996` — a
/// very different picture from "`20_000` used".
#[ test ]
fn bl10_context_splits_into_floor_and_conversation()
{
  let measured = baseline::parse_probe( "2.1.220", &cached_response() ).expect( "parse" );

  assert_eq!( measured.conversation_tokens( 20_000 ), 2_996 );

  // A context below the floor means the floor moved, not that the conversation
  // is negative. Saturating keeps a stale measurement merely useless.
  assert_eq!( measured.conversation_tokens( 500 ), 0 );
}

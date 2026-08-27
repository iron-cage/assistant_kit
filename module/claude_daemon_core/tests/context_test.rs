//! Tests for the context summary a session reports over the wire.
//!
//! These build a transcript on disk and read it back through
//! `context::summary`, because that is what the daemon does — it holds none of
//! this state itself.
//!
//! ## Specification References
//!
//! - `docs/api/001_daemon_surface.md` — the `context_summary` method
//! - `claude_storage_core` `docs/data_structure/004_session_context_state.md` — the fold
//!
//! ## Coverage
//!
//! | TC | Scenario | Expected |
//! |----|----------|----------|
//! | ctx01 | A transcript with rosters and a budget | Every section reported |
//! | ctx02 | No transcript written yet | `NoTranscript`, not an empty summary |
//! | ctx03 | Token halves | `remaining` reported, usage summed, never conflated |
//! | ctx04 | A line kind this build cannot model | Surfaced, not silently dropped |
//! | ctx05 | Summarizing twice | Transcript unchanged — the read is pure |
//! | ctx06 | A `cwd` that will not encode | Reported, not panicked on |
//! | ctx07 | Budget and a billed turn | The window is derived from their sum |
//! | ctx08 | A budget before any turn | No window invented from one half |
//! | ctx09 | A measured baseline | Context splits into overhead and conversation |
//! | ctx10 | No measurement on file | The split is null, never a guessed zero |
//! | ctx11 | A baseline from another version | Not applied to this session |

use std::path::{ Path, PathBuf };

use claude_daemon_core::{ Error, context };
use claude_storage_core::encode_path;
use tempfile::TempDir;

/// Serializes all tests that mutate process-wide env vars.
///
/// `std::env::set_var` is not thread-safe across concurrent tests. Every test
/// here must hold this lock for its whole body.
static ENV_LOCK : std::sync::Mutex< () > = std::sync::Mutex::new( () );

/// Placeholder conversation id shared by the fixtures.
const SESSION : &str = "aaaa0001-0000-4000-8000-000000000001";

/// A temp `CLAUDE_HOME` with a project directory for `cwd` inside it.
struct Fixture
{
  _home : TempDir,
  cwd : PathBuf,
  transcript : PathBuf,
  /// Where a cached baseline would be found, once `measured` has written one.
  baselines : PathBuf,
  /// Whether to hand that directory to `summary`, as a configured daemon would.
  use_baselines : bool,
}

impl Fixture
{
  /// Point `CLAUDE_HOME` at a temp tree and name a transcript inside it.
  ///
  /// The transcript is not created — `written` does that, so a test can also
  /// exercise the not-yet-written case.
  fn new() -> Self
  {
    let home = TempDir::new().expect( "temp home" );
    std::env::set_var( "CLAUDE_HOME", home.path() );

    let cwd = PathBuf::from( "/home/alice/project" );
    let encoded = encode_path( &cwd ).expect( "cwd should encode" );
    let project_dir = home.path().join( "projects" ).join( encoded );
    std::fs::create_dir_all( &project_dir ).expect( "create project dir" );

    let transcript = project_dir.join( format!( "{SESSION}.jsonl" ) );
    let baselines = home.path().join( "-daemon" );

    Self { _home : home, cwd, transcript, baselines, use_baselines : false }
  }

  /// Cache a baseline of `prompt_tokens` for the version and model the fixtures use.
  ///
  /// Keyed off `line`'s `2.1.220` and `assistant`'s `claude-sonnet-5`, because
  /// that is what a session running them would look itself up by.
  fn measured( mut self, prompt_tokens : u64 ) -> Self
  {
    let response = format!
    (
      r#"{{ "model" : "claude-sonnet-5", "usage" : {{ "input_tokens" : {prompt_tokens} }} }}"#
    );
    let baseline = claude_daemon_core::baseline::parse_probe( "2.1.220", &response )
      .expect( "fixture response should parse" );
    claude_daemon_core::baseline::store( &self.baselines, &baseline ).expect( "store baseline" );

    self.use_baselines = true;
    self
  }

  /// Hand `summary` the baseline directory without putting a measurement in it.
  fn looking_for_baselines( mut self ) -> Self
  {
    self.use_baselines = true;
    self
  }

  /// Write `lines` as the session's transcript.
  fn written( self, lines : &[ String ] ) -> Self
  {
    let mut body = String::new();
    for one in lines
    {
      body.push_str( one );
      body.push( '\n' );
    }
    std::fs::write( &self.transcript, body ).expect( "write transcript" );

    self
  }

  /// Summarize this session's context.
  ///
  /// The baseline directory is withheld unless a test asked for it, so every
  /// assertion about the fold stays about the fold — and the two tests that do
  /// ask supply their own measurement rather than depending on whatever the
  /// machine running them happens to have on disk.
  fn summary( &self ) -> claude_daemon_core::Result< serde_json::Value >
  {
    let baselines = self.use_baselines.then_some( self.baselines.as_path() );
    context::summary( &self.cwd, SESSION, baselines )
  }
}

/// Wrap `payload` in the envelope every fixture line shares.
fn line( payload : &str ) -> String
{
  format!( r#"{{"sessionId":"{SESSION}","version":"2.1.220","cwd":"/home/alice/project",{payload}}}"# )
}

/// An `attachment` line carrying `payload`.
fn attach( payload : &str ) -> String
{
  line( &format!( r#""type":"attachment","attachment":{{{payload}}}"# ) )
}

/// An `assistant` line reporting `usage`, keyed by `id` for deduplication.
fn assistant( id : &str, input : u64, output : u64 ) -> String
{
  line
  ( &format!
    (
      r#""type":"assistant","message":{{"id":"{id}","role":"assistant","model":"claude-sonnet-5","usage":{{"input_tokens":{input},"output_tokens":{output},"cache_read_input_tokens":2000,"cache_creation_input_tokens":300}},"content":[]}}"#
    )
  )
}

/// A transcript exercising every section of the summary.
fn full_transcript() -> Vec< String >
{
  vec!
  [
    attach( r#""type":"deferred_tools_delta","addedNames":["WebFetch","Monitor"]"# ),
    attach( r#""type":"agent_listing_delta","addedTypes":["Explore"],"isInitial":true"# ),
    attach( r#""type":"mcp_instructions_delta","addedNames":["chrome"]"# ),
    attach( r#""type":"skill_listing","names":["dev","ops"],"skillCount":2,"isInitial":true"# ),
    attach( r#""type":"invoked_skills","skills":[{"name":"dev","path":"userSettings:dev"}]"# ),
    attach( r#""type":"task_status","taskId":"task_aaa","taskType":"background","status":"running""# ),
    attach( r#""type":"task_reminder","itemCount":3"# ),
    attach( r#""type":"total_tokens_reminder","text":"<total_tokens>750 tokens left</total_tokens>""# ),
    line( r#""type":"mode","mode":"plan""# ),
    assistant( "msg_01", 100, 50 ),
  ]
}

/// ctx01: a full transcript reports every section of the summary.
///
/// The sections are what a caller asked for — rosters, skills, tasks, mode.
/// Asserting them together catches a projection that drops one silently, which
/// would read to a client as "that roster is empty".
#[ test ]
fn ctx01_full_transcript_reports_every_section()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let fixture = Fixture::new().written( &full_transcript() );

  let summary = fixture.summary().expect( "summary should succeed" );

  assert_eq!( summary[ "session_id" ], SESSION );
  assert_eq!( summary[ "version" ], "2.1.220" );
  assert_eq!( summary[ "mode" ], "plan" );
  assert_eq!( summary[ "deferred_tools" ], serde_json::json!( [ "Monitor", "WebFetch" ] ) );
  assert_eq!( summary[ "agent_types" ], serde_json::json!( [ "Explore" ] ) );
  assert_eq!( summary[ "mcp_servers" ], serde_json::json!( [ "chrome" ] ) );
  assert_eq!( summary[ "skills" ][ "available" ], serde_json::json!( [ "dev", "ops" ] ) );
  assert_eq!( summary[ "skills" ][ "reported_count" ], 2 );
  assert_eq!( summary[ "skills" ][ "truncated" ], false );
  assert_eq!( summary[ "skills" ][ "invoked" ][ 0 ][ "name" ], "dev" );
  assert_eq!( summary[ "tasks" ][ "task_aaa" ][ "status" ], "running" );
  assert_eq!( summary[ "task_reminder_items" ], 3 );
  assert_eq!( summary[ "counters" ][ "assistant_messages" ], 1 );
  assert_eq!( summary[ "counters" ][ "has_unmodelled" ], false );
  assert_eq!( summary[ "transcript" ], fixture.transcript.to_string_lossy().as_ref() );
}

/// ctx02: a session with no transcript is reported as such.
///
/// Claude Code writes a transcript on the first turn, so a session spawned
/// moments ago legitimately has none. Answering with an empty summary would
/// read as "this session's context is empty" when the truth is "not known yet"
/// — the two call for opposite reactions from a client.
#[ test ]
fn ctx02_missing_transcript_is_reported_not_faked()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let fixture = Fixture::new();

  match fixture.summary()
  {
    Err( Error::NoTranscript { session_id } ) => assert_eq!( session_id, SESSION ),
    Err( other ) => panic!( "expected NoTranscript, got {other}" ),
    Ok( _ ) => panic!( "expected NoTranscript, got a summary" ),
  }
}

/// ctx03: the two token halves come from different places and stay distinct.
///
/// `remaining` is the harness's own number, parsed from a reminder — it cannot
/// be derived from usage, because it accounts for the static system prompt that
/// never appears in the transcript. Usage is summed separately, deduplicated by
/// `message.id`. Conflating them would report a budget that silently ignores
/// the largest single consumer of it.
#[ test ]
fn ctx03_token_halves_reported_separately()
{
  let _guard = ENV_LOCK.lock().unwrap();

  let mut lines = full_transcript();
  // Same message id as msg_01 — a retry, not a second call. It must not
  // double-count, which is what makes this a genuine test of the stats path
  // rather than a restatement of the fixture.
  lines.push( assistant( "msg_01", 100, 50 ) );
  lines.push( assistant( "msg_02", 7, 3 ) );

  let fixture = Fixture::new().written( &lines );
  let summary = fixture.summary().expect( "summary should succeed" );

  let tokens = &summary[ "tokens" ];
  assert_eq!( tokens[ "remaining" ], 750 );
  assert_eq!( tokens[ "input" ], 107, "msg_01 counted once despite appearing twice" );
  assert_eq!( tokens[ "output" ], 53 );
  assert_eq!( tokens[ "cache_read" ], 4000 );
  assert_eq!( tokens[ "cache_creation" ], 600 );

  // The newest call is msg_02 — 7 fresh + 2000 cached + 300 created. The sums
  // above are nearly twenty times larger, which is exactly why a client cannot
  // use them to judge how full the conversation is.
  assert_eq!( tokens[ "context" ], 2307, "the newest call's whole prompt" );
  assert_eq!( tokens[ "peak_context" ], 2400, "msg_01's larger prompt, held as the peak" );
}

/// ctx07: the window is derived from the two halves that bracket it.
///
/// The window is the one figure in the summary that appears nowhere in the
/// transcript — it belongs to the model, not the conversation. But it is pinned
/// between the budget the harness reports and the prompt the API billed, and
/// their sum is the whole of it.
#[ test ]
fn ctx07_window_is_derived_from_budget_plus_context()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let fixture = Fixture::new().written( &full_transcript() );

  let summary = fixture.summary().expect( "summary should succeed" );

  // 750 reported remaining, over a single msg_01 call of 100 + 2000 + 300.
  assert_eq!( summary[ "tokens" ][ "context" ], 2400 );
  assert_eq!( summary[ "tokens" ][ "window" ], 3150, "remaining + context" );
}

/// ctx08: a session with no billed turn reports no window rather than a wrong one.
///
/// A budget reminder can land before the first assistant message. Adding it to a
/// context of zero would claim the window equals the budget — true only of a
/// conversation that costs nothing to send, which no real one does. A client
/// cannot tell a fabricated window from a measured one, so none is reported.
#[ test ]
fn ctx08_window_absent_until_a_turn_has_been_billed()
{
  let _guard = ENV_LOCK.lock().unwrap();

  let lines = vec!
  [
    attach( r#""type":"total_tokens_reminder","text":"<total_tokens>900 tokens left</total_tokens>""# ),
  ];
  let fixture = Fixture::new().written( &lines );

  let summary = fixture.summary().expect( "summary should succeed" );

  assert_eq!( summary[ "tokens" ][ "remaining" ], 900, "the budget is still reported" );
  assert_eq!( summary[ "tokens" ][ "context" ], 0 );
  assert!
  (
    summary[ "tokens" ][ "window" ].is_null(),
    "a window must not be invented from a budget alone: {}",
    summary[ "tokens" ],
  );
}

/// ctx09: a measured baseline divides the context into overhead and conversation.
///
/// This is the whole point of the measurement. A context of 2400 tokens says
/// nothing on its own about how much room the conversation has actually used —
/// with 2000 of it spent before the first word, the conversation is 400, which
/// is a sixth of what the raw figure suggests.
#[ test ]
fn ctx09_measured_baseline_splits_the_context()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let fixture = Fixture::new().written( &full_transcript() ).measured( 2000 );

  let tokens = fixture.summary().expect( "summary should succeed" )[ "tokens" ].clone();

  assert_eq!( tokens[ "context" ], 2400, "unchanged by the measurement" );
  assert_eq!( tokens[ "static_overhead" ], 2000 );
  assert_eq!( tokens[ "conversation" ], 400, "context minus the floor" );
}

/// ctx10: with no measurement on file, the split is null rather than guessed.
///
/// The directory is supplied and simply holds nothing for this version and
/// model. Reporting `0` overhead would be a claim — that the prompt costs
/// nothing before the conversation starts — and a client could not tell it from
/// a real measurement of a very small floor.
#[ test ]
fn ctx10_unmeasured_split_is_null_not_zero()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let fixture = Fixture::new().written( &full_transcript() ).looking_for_baselines();

  let tokens = fixture.summary().expect( "summary should succeed" )[ "tokens" ].clone();

  assert_eq!( tokens[ "context" ], 2400, "the figure that needs no measurement" );
  assert!( tokens[ "static_overhead" ].is_null(), "got {tokens}" );
  assert!( tokens[ "conversation" ].is_null(), "got {tokens}" );
}

/// ctx11: a baseline for another version does not answer for this one.
///
/// The floor moves when Claude Code changes what it puts in the system prompt,
/// so a measurement of 2.1.219 describes a prompt this session never sent.
/// Subtracting it would silently misreport the conversation by whatever the two
/// versions differ by — an error with no symptom.
#[ test ]
fn ctx11_baseline_from_another_version_is_not_applied()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let fixture = Fixture::new().written( &full_transcript() ).looking_for_baselines();

  let other = claude_daemon_core::baseline::parse_probe
  (
    "2.1.219",
    r#"{ "model" : "claude-sonnet-5", "usage" : { "input_tokens" : 2000 } }"#,
  ).expect( "fixture response should parse" );
  claude_daemon_core::baseline::store( &fixture.baselines, &other ).expect( "store" );

  let tokens = fixture.summary().expect( "summary should succeed" )[ "tokens" ].clone();

  assert!
  (
    tokens[ "static_overhead" ].is_null(),
    "a 2.1.219 measurement must not answer for a 2.1.220 session: {tokens}",
  );
}

/// ctx04: a line kind this build cannot model is surfaced, not dropped.
///
/// Claude Code's format grows between releases. A client that never learns its
/// daemon could not parse part of a transcript cannot tell a genuinely empty
/// roster from one this build failed to read.
#[ test ]
fn ctx04_unmodelled_kinds_surface_in_the_summary()
{
  let _guard = ENV_LOCK.lock().unwrap();

  let mut lines = full_transcript();
  lines.push( line( r#""type":"future-envelope""# ) );
  lines.push( attach( r#""type":"future_attachment""# ) );

  let fixture = Fixture::new().written( &lines );
  let summary = fixture.summary().expect( "summary should succeed" );

  let counters = &summary[ "counters" ];
  assert_eq!( counters[ "has_unmodelled" ], true );
  assert_eq!( counters[ "unmodelled_kinds" ][ "future-envelope" ], 1 );
  assert_eq!( counters[ "unmodelled_attachments" ][ "future_attachment" ], 1 );
}

/// ctx05: summarizing does not touch the transcript.
///
/// The request is issued against a live session, possibly mid-turn. If it wrote
/// to, truncated, or re-created the file Claude Code is appending to, it would
/// corrupt the very session it was asked about.
#[ test ]
fn ctx05_summary_leaves_the_transcript_untouched()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let fixture = Fixture::new().written( &full_transcript() );

  let before = std::fs::read( &fixture.transcript ).expect( "read before" );

  fixture.summary().expect( "first summary" );
  fixture.summary().expect( "second summary" );

  let after = std::fs::read( &fixture.transcript ).expect( "read after" );
  assert_eq!( before, after, "summarizing must not modify the transcript" );
}

/// ctx06: a working directory that will not encode is reported, not panicked on.
///
/// `cwd` comes from the daemon's own session record, but a session adopted from
/// a re-host can carry a path that no longer encodes cleanly. That is a report,
/// not a crash.
#[ test ]
fn ctx06_unencodable_cwd_reports_no_transcript()
{
  let _guard = ENV_LOCK.lock().unwrap();
  let home = TempDir::new().expect( "temp home" );
  std::env::set_var( "CLAUDE_HOME", home.path() );

  // A relative path has no storage encoding — every project directory is named
  // from an absolute one.
  match context::summary( Path::new( "" ), SESSION, None )
  {
    Err( Error::NoTranscript { .. } ) => (),
    Err( other ) => panic!( "expected NoTranscript, got {other}" ),
    Ok( _ ) => panic!( "expected NoTranscript, got a summary" ),
  }
}

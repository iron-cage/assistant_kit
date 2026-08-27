//! Unit tests for `ContextFold` — accumulating context state from an event stream.
//!
//! Fixtures are hand-written JSONL lines using placeholder ids and paths. The
//! field names match what Claude Code writes; no line is copied from a real
//! session.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

use claude_storage_core::ContextFold;
use tempfile::TempDir;

/// Placeholder conversation id shared by the fixtures.
const SESSION : &str = "aaaa0001-0000-4000-8000-000000000001";

/// Wrap `payload` in the envelope every fixture line shares.
fn line( payload : &str ) -> String
{
  format!( r#"{{"sessionId":"{SESSION}","version":"2.1.220",{payload}}}"# )
}

/// An `attachment` line carrying `payload`.
fn attach( payload : &str ) -> String
{
  line( &format!( r#""type":"attachment","attachment":{{{payload}}}"# ) )
}

/// A temp directory plus the session file path inside it.
struct Fixture
{
  _dir : TempDir,
  path : PathBuf,
}

impl Fixture
{
  /// An empty session file.
  fn new() -> Self
  {
    let dir = TempDir::new().expect( "temp dir" );
    let path = dir.path().join( "session.jsonl" );
    std::fs::write( &path, "" ).expect( "create session file" );

    Self { _dir : dir, path }
  }

  /// Append `text` verbatim — no newline is added.
  fn append_raw( &self, text : &str )
  {
    let mut file = OpenOptions::new()
      .append( true )
      .open( &self.path )
      .expect( "open for append" );

    file.write_all( text.as_bytes() ).expect( "append" );
  }

  /// Append each line, newline-terminated.
  fn append_lines( &self, lines : &[ String ] )
  {
    for one in lines
    {
      self.append_raw( &format!( "{one}\n" ) );
    }
  }

  /// Replace the file's whole contents.
  fn overwrite( &self, text : &str )
  {
    std::fs::write( &self.path, text ).expect( "overwrite" );
  }
}

/// Test deferred-tool deltas accumulate into the current set
///
/// ## Purpose
/// The file never states which tools are deferred *now* — only what changed.
/// Reconstructing the live set is the fold's core job.
///
/// ## Coverage
/// Three deltas: an initial add, a removal, and a re-add of a removed tool.
///
/// ## Validation Strategy
/// Asserts the final set holds exactly the tools that survived, including the
/// one restored via `readdedNames` rather than `addedNames`.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — delta accumulation
#[ test ]
fn context_deferred_tools_accumulate_across_deltas()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[
    attach( r#""type":"deferred_tools_delta","addedNames":["WebFetch","Monitor","CronList"]"# ),
    attach( r#""type":"deferred_tools_delta","removedNames":["Monitor","CronList"]"# ),
    attach( r#""type":"deferred_tools_delta","readdedNames":["CronList"]"# ),
  ]);

  let mut fold = ContextFold::new();
  assert_eq!( fold.read_file( &fixture.path ).expect( "read" ), 3 );

  let tools : Vec< &str > = fold.state().deferred_tools.iter().map( String::as_str ).collect();
  assert_eq!( tools, vec![ "CronList", "WebFetch" ] );
}

/// Test a removal and an addition in one delta resolve to the addition
///
/// ## Purpose
/// Within a single delta the two lists are applied in a defined order. Leaving
/// it undefined would make the resulting set depend on iteration order.
///
/// ## Coverage
/// One delta naming the same tool in both `removedNames` and `addedNames`.
///
/// ## Validation Strategy
/// Asserts the tool is present — the addition is the newer fact.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — delta accumulation
#[ test ]
fn context_deferred_tools_addition_wins_within_one_delta()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[
    attach( r#""type":"deferred_tools_delta","addedNames":["Monitor"],"removedNames":["Monitor"]"# ),
  ]);

  let mut fold = ContextFold::new();
  fold.read_file( &fixture.path ).expect( "read" );

  assert!( fold.state().deferred_tools.contains( "Monitor" ) );
}

/// Test an initial agent listing replaces rather than merges
///
/// ## Purpose
/// `isInitial` marks a full snapshot, not a delta. Merging it into an existing
/// roster would leave agents from a prior listing that are no longer offered.
///
/// ## Coverage
/// A delta adding two agents, then an initial listing naming a different one.
///
/// ## Validation Strategy
/// Asserts only the agent from the initial listing survives.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — initial vs delta listings
#[ test ]
fn context_initial_agent_listing_replaces_roster()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[
    attach( r#""type":"agent_listing_delta","addedTypes":["Explore","Plan"],"isInitial":false"# ),
    attach( r#""type":"agent_listing_delta","addedTypes":["claude"],"isInitial":true"# ),
  ]);

  let mut fold = ContextFold::new();
  fold.read_file( &fixture.path ).expect( "read" );

  let agents : Vec< &str > = fold.state().agent_types.iter().map( String::as_str ).collect();
  assert_eq!( agents, vec![ "claude" ] );
}

/// Test the token budget keeps the last parseable number
///
/// ## Purpose
/// The budget is prose, and its wording is not a stable contract. A reminder
/// whose phrasing changed must not erase a number that was correctly read.
///
/// ## Coverage
/// Two parseable reminders followed by an unparseable one.
///
/// ## Validation Strategy
/// Asserts the retained value is the second reminder's, not the first's and
/// not `None`.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — token accounting
#[ test ]
fn context_token_budget_keeps_last_parseable()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[
    attach( r#""type":"total_tokens_reminder","text":"<total_tokens>900 tokens left</total_tokens>""# ),
    attach( r#""type":"total_tokens_reminder","text":"<total_tokens>750 tokens left</total_tokens>""# ),
    attach( r#""type":"total_tokens_reminder","text":"budget unavailable""# ),
  ]);

  let mut fold = ContextFold::new();
  fold.read_file( &fixture.path ).expect( "read" );

  assert_eq!( fold.state().tokens_remaining, Some( 750 ) );
}

/// Test sidechain lines are counted but never folded
///
/// ## Purpose
/// A subagent's context is its own. Letting its roster deltas through would
/// corrupt the main conversation's view of what it has loaded.
///
/// ## Coverage
/// A main-conversation delta followed by a sidechain delta adding a different
/// tool.
///
/// ## Validation Strategy
/// Asserts the sidechain tool is absent from the set and that the line was
/// counted rather than silently dropped.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — sidechain exclusion
#[ test ]
fn context_sidechain_lines_excluded_from_fold()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[
    attach( r#""type":"deferred_tools_delta","addedNames":["WebFetch"]"# ),
    format!
    (
      r#"{{"sessionId":"{SESSION}","isSidechain":true,"type":"attachment","attachment":{{"type":"deferred_tools_delta","addedNames":["SubagentOnly"]}}}}"#
    ),
  ]);

  let mut fold = ContextFold::new();
  fold.read_file( &fixture.path ).expect( "read" );

  assert!( fold.state().deferred_tools.contains( "WebFetch" ) );
  assert!( !fold.state().deferred_tools.contains( "SubagentOnly" ) );
  assert_eq!( fold.state().counters.sidechain_events, 1 );
}

/// Test a trailing line without a newline is left unconsumed
///
/// ## Purpose
/// This is what makes tailing a live session safe. A line with no terminator
/// is a write still in progress — parsing it would fold a half-written event,
/// and consuming it would mean never seeing the whole one.
///
/// ## Coverage
/// A complete line followed by a partial one, then the partial line's
/// completion appended.
///
/// ## Validation Strategy
/// Asserts the first read applies only the complete line and leaves the offset
/// short of the file's length, then that the second read applies the completed
/// line exactly once.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — incremental reads
#[ test ]
fn context_partial_trailing_line_not_consumed()
{
  let fixture = Fixture::new();
  fixture.append_raw( &format!( "{}\n", attach( r#""type":"deferred_tools_delta","addedNames":["WebFetch"]"# ) ) );

  let complete = attach( r#""type":"deferred_tools_delta","addedNames":["Monitor"]"# );
  let ( head, tail ) = complete.split_at( 20 );
  fixture.append_raw( head );

  let mut fold = ContextFold::new();
  assert_eq!( fold.read_file( &fixture.path ).expect( "first read" ), 1 );
  assert!( fold.state().deferred_tools.contains( "WebFetch" ) );
  assert!( !fold.state().deferred_tools.contains( "Monitor" ) );

  let consumed = fold.offset();

  fixture.append_raw( &format!( "{tail}\n" ) );

  assert_eq!( fold.read_file( &fixture.path ).expect( "second read" ), 1 );
  assert!( fold.state().deferred_tools.contains( "Monitor" ) );
  assert!( fold.offset() > consumed );

  // A third read with nothing appended applies nothing and re-reads nothing.
  assert_eq!( fold.read_file( &fixture.path ).expect( "third read" ), 0 );
  assert_eq!( fold.state().counters.user_messages, 0 );
}

/// Test a shortened file restarts the fold
///
/// ## Purpose
/// A file shorter than the offset was replaced, not appended to. Continuing
/// from the old offset would skip the new file's opening lines and mix state
/// from two different sessions.
///
/// ## Coverage
/// A fold advanced over several lines, then the file replaced with a shorter,
/// different one.
///
/// ## Validation Strategy
/// Asserts state from the original file is gone and only the replacement's is
/// present.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — incremental reads
#[ test ]
fn context_shortened_file_restarts_fold()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[
    attach( r#""type":"deferred_tools_delta","addedNames":["WebFetch","Monitor","CronList"]"# ),
    attach( r#""type":"total_tokens_reminder","text":"<total_tokens>900 tokens left</total_tokens>""# ),
  ]);

  let mut fold = ContextFold::new();
  fold.read_file( &fixture.path ).expect( "first read" );
  assert_eq!( fold.state().deferred_tools.len(), 3 );

  fixture.overwrite( &format!( "{}\n", attach( r#""type":"deferred_tools_delta","addedNames":["OnlyOne"]"# ) ) );

  fold.read_file( &fixture.path ).expect( "second read" );

  let tools : Vec< &str > = fold.state().deferred_tools.iter().map( String::as_str ).collect();
  assert_eq!( tools, vec![ "OnlyOne" ] );
  assert_eq!( fold.state().tokens_remaining, None );
}

/// Test malformed and empty lines are skipped and counted
///
/// ## Purpose
/// One bad line must not discard the rest of a session, matching the per-line
/// skip policy the statistics and search readers already use. Counting them
/// keeps the skip visible rather than silent.
///
/// ## Coverage
/// A valid line, invalid JSON, a JSON object with no `type`, a blank line, and
/// another valid line.
///
/// ## Validation Strategy
/// Asserts both valid lines were folded and that the skipped count matches the
/// two malformed lines — the blank line is consumed without counting as a skip.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — degradation policy
#[ test ]
fn context_malformed_lines_skipped_and_counted()
{
  let fixture = Fixture::new();
  fixture.append_raw
  (
    &format!
    (
      "{}\n{{not json\n{{\"noType\":1}}\n\n{}\n",
      attach( r#""type":"deferred_tools_delta","addedNames":["WebFetch"]"# ),
      attach( r#""type":"deferred_tools_delta","addedNames":["Monitor"]"# ),
    )
  );

  let mut fold = ContextFold::new();
  assert_eq!( fold.read_file( &fixture.path ).expect( "read" ), 2 );

  assert_eq!( fold.state().deferred_tools.len(), 2 );
  assert_eq!( fold.state().counters.lines_read, 5 );
  assert_eq!( fold.state().counters.lines_skipped, 2 );
}

/// Test a skill listing replaces and reports its own truncation
///
/// ## Purpose
/// A listing is a snapshot, and its self-reported count is the only way to know
/// it was truncated — the names alone cannot say what is missing.
///
/// ## Coverage
/// A full listing followed by a truncated one declaring a larger count.
///
/// ## Validation Strategy
/// Asserts the names were replaced rather than merged, and that
/// `skills_truncated()` flips only for the disagreeing listing.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — skill listings
#[ test ]
fn context_skill_listing_replaces_and_reports_truncation()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[ attach( r#""type":"skill_listing","names":["dev","ops"],"skillCount":2,"isInitial":true"# ) ] );

  let mut fold = ContextFold::new();
  fold.read_file( &fixture.path ).expect( "first read" );
  assert_eq!( fold.state().skills_available, vec![ "dev".to_string(), "ops".to_string() ] );
  assert!( !fold.state().skills_truncated() );

  fixture.append_lines
  ( &[ attach( r#""type":"skill_listing","names":["dev"],"skillCount":134"# ) ] );

  fold.read_file( &fixture.path ).expect( "second read" );
  assert_eq!( fold.state().skills_available, vec![ "dev".to_string() ] );
  assert!( fold.state().skills_truncated() );
}

/// Test a skill invoked twice is recorded once
///
/// ## Purpose
/// The harness injects a skill's text on first invocation. A repeat adds
/// nothing to what is in context, so recording it twice would overstate the
/// context's contents.
///
/// ## Coverage
/// The same skill invoked in two separate attachments, plus a second skill.
///
/// ## Validation Strategy
/// Asserts each skill appears exactly once, in first-invocation order.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — invoked skills
#[ test ]
fn context_repeated_skill_invocation_recorded_once()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[
    attach( r#""type":"invoked_skills","skills":[{"name":"dev","path":"userSettings:dev"}]"# ),
    attach( r#""type":"invoked_skills","skills":[{"name":"ops"}]"# ),
    attach( r#""type":"invoked_skills","skills":[{"name":"dev","path":"userSettings:dev"}]"# ),
  ]);

  let mut fold = ContextFold::new();
  fold.read_file( &fixture.path ).expect( "read" );

  let names : Vec< &str > = fold.state().skills_invoked.iter().map( | s | s.name.as_str() ).collect();
  assert_eq!( names, vec![ "dev", "ops" ] );
}

/// Test task status lines upsert by task id
///
/// ## Purpose
/// A task reports its state repeatedly as it progresses. Only the most recent
/// report describes it now.
///
/// ## Coverage
/// One task reported twice with different statuses, plus a second task.
///
/// ## Validation Strategy
/// Asserts two tasks are tracked and the repeated one holds its later status.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — background tasks
#[ test ]
fn context_task_status_upserts_by_id()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[
    attach( r#""type":"task_status","taskId":"task_aaa","taskType":"background","status":"running","description":"Build docs""# ),
    attach( r#""type":"task_status","taskId":"task_bbb","taskType":"background","status":"running""# ),
    attach( r#""type":"task_status","taskId":"task_aaa","taskType":"background","status":"completed","outputFilePath":"/tmp/out.txt""# ),
  ]);

  let mut fold = ContextFold::new();
  fold.read_file( &fixture.path ).expect( "read" );

  assert_eq!( fold.state().tasks.len(), 2 );

  let first = fold.state().tasks.get( "task_aaa" ).expect( "task_aaa tracked" );
  assert_eq!( first.status, "completed" );
  assert_eq!( first.output_file_path.as_deref(), Some( "/tmp/out.txt" ) );
}

/// Test unmodelled line and attachment kinds are counted by name
///
/// ## Purpose
/// A newer Claude Code's added kind must be visible as "this build's schema is
/// behind", not silently absent. Silence would read as "nothing was there".
///
/// ## Coverage
/// An unknown envelope type and an unknown attachment type, the latter twice.
///
/// ## Validation Strategy
/// Asserts each is counted under its own name and that `has_unmodelled()`
/// reports the condition.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — forward compatibility
#[ test ]
fn context_unmodelled_kinds_counted_by_name()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[
    line( r#""type":"future-envelope""# ),
    attach( r#""type":"future_attachment""# ),
    attach( r#""type":"future_attachment""# ),
  ]);

  let mut fold = ContextFold::new();
  fold.read_file( &fixture.path ).expect( "read" );

  let state = fold.state();
  assert!( state.has_unmodelled() );
  assert_eq!( state.counters.unmodelled_kinds.get( "future-envelope" ), Some( &1 ) );
  assert_eq!( state.counters.unmodelled_attachments.get( "future_attachment" ), Some( &2 ) );
}

/// Test envelope metadata resolves first-wins or last-wins per field
///
/// ## Purpose
/// `cwd` identifies where the session started and must not drift; `version`
/// identifies what wrote the newest line and must, so a session resumed after
/// an upgrade reports the version now in use.
///
/// ## Coverage
/// Two lines carrying different `cwd` and `version` values.
///
/// ## Validation Strategy
/// Asserts `cwd` holds the first value and `version` the last.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — envelope resolution
#[ test ]
fn context_envelope_first_wins_cwd_last_wins_version()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[
    format!( r#"{{"sessionId":"{SESSION}","type":"user","cwd":"/home/alice/first","version":"2.1.220"}}"# ),
    format!( r#"{{"sessionId":"{SESSION}","type":"user","cwd":"/home/alice/second","version":"2.2.0"}}"# ),
  ]);

  let mut fold = ContextFold::new();
  fold.read_file( &fixture.path ).expect( "read" );

  let state = fold.state();
  assert_eq!( state.session_id, SESSION );
  assert_eq!( state.cwd.as_deref(), Some( std::path::Path::new( "/home/alice/first" ) ) );
  assert_eq!( state.version.as_deref(), Some( "2.2.0" ) );
  assert_eq!( state.counters.user_messages, 2 );
}

/// Test mode, permission mode, title, and date hold their latest values
///
/// ## Purpose
/// These four are session-level scalars that change over a conversation; only
/// the current value is meaningful.
///
/// ## Coverage
/// Each kind appearing twice with different values, plus a compact boundary.
///
/// ## Validation Strategy
/// Asserts each field holds the later value and that compactions were tallied.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — session scalars
#[ test ]
fn context_session_scalars_hold_latest_values()
{
  let fixture = Fixture::new();
  fixture.append_lines
  ( &[
    line( r#""type":"mode","mode":"normal""# ),
    line( r#""type":"mode","mode":"plan""# ),
    line( r#""type":"permission-mode","permissionMode":"default""# ),
    line( r#""type":"permission-mode","permissionMode":"acceptEdits""# ),
    line( r#""type":"ai-title","aiTitle":"First title""# ),
    line( r#""type":"ai-title","aiTitle":"Second title""# ),
    attach( r#""type":"date_change","newDate":"2026-01-16""# ),
    line( r#""type":"system","subtype":"compact_boundary""# ),
  ]);

  let mut fold = ContextFold::new();
  fold.read_file( &fixture.path ).expect( "read" );

  let state = fold.state();
  assert_eq!( state.mode.as_deref(), Some( "plan" ) );
  assert_eq!( state.permission_mode.as_deref(), Some( "acceptEdits" ) );
  assert_eq!( state.title.as_deref(), Some( "Second title" ) );
  assert_eq!( state.date.as_deref(), Some( "2026-01-16" ) );
  assert_eq!( state.counters.compactions, 1 );
  assert_eq!( state.counters.system_subtypes.get( "compact_boundary" ), Some( &1 ) );
}

/// Test an empty file folds to empty state without error
///
/// ## Purpose
/// A session file that exists but has no lines yet is the normal state of a
/// session just created. It is not an error.
///
/// ## Coverage
/// A zero-byte session file.
///
/// ## Validation Strategy
/// Asserts the read succeeds, applies nothing, and leaves the offset at zero.
///
/// ## Related Requirements
/// `docs/data_structure/004_session_context_state.md` — incremental reads
#[ test ]
fn context_empty_file_folds_to_empty_state()
{
  let fixture = Fixture::new();

  let mut fold = ContextFold::new();
  assert_eq!( fold.read_file( &fixture.path ).expect( "read" ), 0 );

  assert_eq!( fold.offset(), 0 );
  assert!( fold.state().deferred_tools.is_empty() );
  assert_eq!( fold.state().tokens_remaining, None );
  assert!( !fold.state().has_unmodelled() );
}

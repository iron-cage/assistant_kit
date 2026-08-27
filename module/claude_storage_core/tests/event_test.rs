//! Unit tests for `SessionEvent` — the wider-schema parser over session JSONL.
//!
//! Every fixture here is a hand-written line using placeholder ids and paths.
//! The field names and value shapes match what Claude Code writes, but no line
//! is copied from a real session.

use claude_storage_core::{ Attachment, EventKind, SessionEvent };

/// Placeholder conversation id shared by the fixtures.
const SESSION : &str = "aaaa0001-0000-4000-8000-000000000001";

/// Build an envelope line of `kind` carrying `extra` as additional fields.
fn envelope( kind : &str, extra : &str ) -> String
{
  format!
  (
    r#"{{"type":"{kind}","sessionId":"{SESSION}","uuid":"bbbb0001-0000-4000-8000-000000000002","parentUuid":null,"timestamp":"2026-01-15T10:00:00.000Z","cwd":"/home/alice/project","version":"2.1.220","isSidechain":false{extra}}}"#
  )
}

/// Build an `attachment` line whose payload is `payload`.
fn attachment_line( payload : &str ) -> String
{
  envelope( "attachment", &format!( r#","attachment":{payload}"# ) )
}

/// Parse a line, failing the test loudly if it does not parse.
fn parse( line : &str ) -> SessionEvent
{
  SessionEvent::from_json_line( line )
    .unwrap_or_else( | e | panic!( "expected line to parse, got {e}: {line}" ) )
}

/// Extract the attachment payload, failing the test if the line was not one.
fn attachment_of( line : &str ) -> Attachment
{
  parse( line ).attachment().cloned()
    .unwrap_or_else( || panic!( "expected an attachment line: {line}" ) )
}

/// Test envelope metadata is read from a conversation line
///
/// ## Purpose
/// The envelope fields are what let a fold attribute an event to a session,
/// order it, and know which Claude Code version produced it.
///
/// ## Coverage
/// All envelope fields populated on a `user` line.
///
/// ## Validation Strategy
/// Asserts each field equals the exact fixture value.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — envelope shape
#[ test ]
fn event_envelope_fields_populated()
{
  let event = parse( &envelope( "user", r#","message":{"role":"user","content":"hi"}"# ) );

  assert_eq!( event.session_id, SESSION );
  assert_eq!( event.uuid.as_deref(), Some( "bbbb0001-0000-4000-8000-000000000002" ) );
  assert_eq!( event.parent_uuid, None );
  assert_eq!( event.timestamp.as_deref(), Some( "2026-01-15T10:00:00.000Z" ) );
  assert_eq!( event.cwd.as_deref(), Some( std::path::Path::new( "/home/alice/project" ) ) );
  assert_eq!( event.version.as_deref(), Some( "2.1.220" ) );
  assert!( !event.is_sidechain );
}

/// Test conversation lines are recognized without being re-parsed
///
/// ## Purpose
/// `Entry` is the single parser for conversation content. This module must
/// classify those lines without duplicating that schema, or the two will drift.
///
/// ## Coverage
/// `user` and `assistant` lines carry payload-free variants and report
/// `is_conversation()`.
///
/// ## Validation Strategy
/// Asserts the variants match exactly and that `is_conversation()` is true for
/// both and false for a non-conversation line.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — conversation delegation rule
#[ test ]
fn event_conversation_lines_carry_no_payload()
{
  let user = parse( &envelope( "user", r#","message":{"role":"user","content":"hi"}"# ) );
  let assistant = parse( &envelope( "assistant", r#","message":{"role":"assistant","content":[]}"# ) );
  let mode = parse( &envelope( "mode", r#","mode":"normal""# ) );

  assert_eq!( user.kind, EventKind::User );
  assert_eq!( assistant.kind, EventKind::Assistant );
  assert!( user.is_conversation() );
  assert!( assistant.is_conversation() );
  assert!( !mode.is_conversation() );
}

/// Test each short envelope kind parses into its own variant
///
/// ## Purpose
/// `mode`, `permission-mode`, `last-prompt`, `ai-title`, and `queue-operation`
/// carry session state that no conversation line repeats.
///
/// ## Coverage
/// One line per short envelope kind.
///
/// ## Validation Strategy
/// Asserts the exact variant and payload value for each.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — envelope kind table
#[ test ]
fn event_short_envelope_kinds_parse()
{
  let cases : Vec< ( String, EventKind ) > = vec!
  [
    (
      envelope( "mode", r#","mode":"plan""# ),
      EventKind::Mode { mode : "plan".to_string() },
    ),
    (
      envelope( "permission-mode", r#","permissionMode":"acceptEdits""# ),
      EventKind::PermissionMode { permission_mode : "acceptEdits".to_string() },
    ),
    (
      envelope( "last-prompt", r#","leafUuid":"cccc0001-0000-4000-8000-000000000003""# ),
      EventKind::LastPrompt { leaf_uuid : "cccc0001-0000-4000-8000-000000000003".to_string() },
    ),
    (
      envelope( "ai-title", r#","aiTitle":"Refactor the parser""# ),
      EventKind::AiTitle { title : "Refactor the parser".to_string() },
    ),
    (
      envelope( "queue-operation", r#","operation":"enqueue""# ),
      EventKind::QueueOperation { operation : "enqueue".to_string() },
    ),
  ];

  for ( line, expected ) in cases
  {
    assert_eq!( parse( &line ).kind, expected, "line: {line}" );
  }
}

/// Test system telemetry lines carry their subtype and counters
///
/// ## Purpose
/// `turn_duration` and `compact_boundary` mark turn and context boundaries a
/// fold reports on.
///
/// ## Coverage
/// A `turn_duration` line with both counters, and a `compact_boundary` line
/// with neither.
///
/// ## Validation Strategy
/// Asserts the subtype string and that absent counters are `None` rather than
/// zero, so "not reported" stays distinguishable from "reported as zero".
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — system subtypes
#[ test ]
fn event_system_subtype_and_counters()
{
  let turn = parse( &envelope( "system", r#","subtype":"turn_duration","durationMs":418205,"messageCount":81"# ) );

  assert_eq!
  (
    turn.kind,
    EventKind::System
    {
      subtype : "turn_duration".to_string(),
      duration_ms : Some( 418_205 ),
      message_count : Some( 81 ),
    }
  );

  let compact = parse( &envelope( "system", r#","subtype":"compact_boundary""# ) );

  assert_eq!
  (
    compact.kind,
    EventKind::System
    {
      subtype : "compact_boundary".to_string(),
      duration_ms : None,
      message_count : None,
    }
  );
}

/// Test the token reminder's number is parsed out of its prose
///
/// ## Purpose
/// The remaining token budget is reported only as prose inside a reminder, and
/// it is not derivable from summing usage — it is the harness's own number.
///
/// ## Coverage
/// A well-formed reminder, and one whose wording carries no number.
///
/// ## Validation Strategy
/// Asserts the parsed value, and that unparseable wording yields `None` rather
/// than a wrong number.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — `total_tokens_reminder`
#[ test ]
fn event_total_tokens_reminder_parsed()
{
  let ok = attachment_line( r#"{"type":"total_tokens_reminder","text":"<total_tokens>14869351 tokens left</total_tokens>"}"# );
  assert_eq!( attachment_of( &ok ), Attachment::TotalTokensReminder { remaining : Some( 14_869_351 ) } );

  let prose = attachment_line( r#"{"type":"total_tokens_reminder","text":"budget unavailable"}"# );
  assert_eq!( attachment_of( &prose ), Attachment::TotalTokensReminder { remaining : None } );
}

/// Test the deferred-tool delta retains all four name lists
///
/// ## Purpose
/// A fold reconstructs the live deferred-tool set from these deltas; dropping
/// `readdedNames` would leave a re-deferred tool permanently missing.
///
/// ## Coverage
/// A delta carrying added, removed, readded, and pending-server names.
///
/// ## Validation Strategy
/// Asserts all four vectors match the fixture exactly.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — `deferred_tools_delta`
#[ test ]
fn event_deferred_tools_delta_retains_all_lists()
{
  let line = attachment_line
  (
    r#"{"type":"deferred_tools_delta","addedNames":["WebFetch","Monitor"],"removedNames":["CronList"],"readdedNames":["TaskGet"],"pendingMcpServers":["chrome"]}"#
  );

  assert_eq!
  (
    attachment_of( &line ),
    Attachment::DeferredToolsDelta
    {
      added : vec![ "WebFetch".to_string(), "Monitor".to_string() ],
      removed : vec![ "CronList".to_string() ],
      readded : vec![ "TaskGet".to_string() ],
      pending_mcp_servers : vec![ "chrome".to_string() ],
    }
  );
}

/// Test the roster attachments parse into their variants
///
/// ## Purpose
/// Agent types, MCP servers, and skills each have their own roster attachment
/// that a fold accumulates independently.
///
/// ## Coverage
/// One line each for `agent_listing_delta`, `mcp_instructions_delta`, and
/// `skill_listing`.
///
/// ## Validation Strategy
/// Asserts each variant's fields, including the `isInitial` flag that
/// distinguishes a first listing from a delta.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — roster attachments
#[ test ]
fn event_roster_attachments_parse()
{
  let agents = attachment_line
  (
    r#"{"type":"agent_listing_delta","addedTypes":["Explore","Plan"],"removedTypes":[],"isInitial":true}"#
  );
  assert_eq!
  (
    attachment_of( &agents ),
    Attachment::AgentListingDelta
    {
      added : vec![ "Explore".to_string(), "Plan".to_string() ],
      removed : Vec::new(),
      is_initial : true,
    }
  );

  let mcp = attachment_line
  (
    r#"{"type":"mcp_instructions_delta","addedNames":["chrome"],"removedNames":[]}"#
  );
  assert_eq!
  (
    attachment_of( &mcp ),
    Attachment::McpInstructionsDelta
    {
      added : vec![ "chrome".to_string() ],
      removed : Vec::new(),
    }
  );

  let skills = attachment_line
  (
    r#"{"type":"skill_listing","names":["dev","tst_fix"],"skillCount":2,"isInitial":true,"content":"- dev: ...\n- tst_fix: ..."}"#
  );
  assert_eq!
  (
    attachment_of( &skills ),
    Attachment::SkillListing
    {
      names : vec![ "dev".to_string(), "tst_fix".to_string() ],
      skill_count : 2,
      is_initial : true,
    }
  );
}

/// Test the reported skill count is kept rather than derived from the names
///
/// ## Purpose
/// A disagreement between `skillCount` and the length of `names` means the
/// listing was truncated. Deriving the count from `names` would erase that
/// signal.
///
/// ## Coverage
/// A listing whose declared count exceeds the names it carries.
///
/// ## Validation Strategy
/// Asserts the retained count is the declared one, not the vector length.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — `skill_listing`
#[ test ]
fn event_skill_listing_keeps_reported_count()
{
  let line = attachment_line
  (
    r#"{"type":"skill_listing","names":["dev"],"skillCount":134,"isInitial":true}"#
  );

  let Attachment::SkillListing { names, skill_count, .. } = attachment_of( &line )
  else
  {
    panic!( "expected a skill listing" )
  };

  assert_eq!( names.len(), 1 );
  assert_eq!( skill_count, 134 );
}

/// Test invoked skills keep name and path but drop the injected content
///
/// ## Purpose
/// A fold needs to know a skill ran and where it resolved from. The `content`
/// field repeats the skill's whole text, which is already on disk at `path`.
///
/// ## Coverage
/// An `invoked_skills` line carrying one skill with all three fields.
///
/// ## Validation Strategy
/// Asserts name and path survive; the absence of a content field on
/// `InvokedSkill` is enforced by the type.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — `invoked_skills`
#[ test ]
fn event_invoked_skills_drop_content()
{
  let line = attachment_line
  (
    r#"{"type":"invoked_skills","skills":[{"name":"dev","path":"userSettings:dev","content":"the skill's whole body text"}]}"#
  );

  let Attachment::InvokedSkills { skills } = attachment_of( &line )
  else
  {
    panic!( "expected invoked skills" )
  };

  assert_eq!( skills.len(), 1 );
  assert_eq!( skills[ 0 ].name, "dev" );
  assert_eq!( skills[ 0 ].path.as_deref(), Some( "userSettings:dev" ) );
}

/// Test task attachments parse into their variants
///
/// ## Purpose
/// `task_reminder` and `task_status` are the only record of background task
/// state in the transcript.
///
/// ## Coverage
/// A reminder with a count, and a status line with optional fields both
/// present and absent.
///
/// ## Validation Strategy
/// Asserts every field, including that absent optionals are `None`.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — task attachments
#[ test ]
fn event_task_attachments_parse()
{
  let reminder = attachment_line( r#"{"type":"task_reminder","content":[],"itemCount":3}"# );
  assert_eq!( attachment_of( &reminder ), Attachment::TaskReminder { item_count : 3 } );

  let full = attachment_line
  (
    r#"{"type":"task_status","taskId":"task_abc12345","taskType":"background","status":"running","description":"Build docs","outputFilePath":"/tmp/out.txt"}"#
  );
  assert_eq!
  (
    attachment_of( &full ),
    Attachment::TaskStatus
    {
      task_id : "task_abc12345".to_string(),
      task_type : "background".to_string(),
      status : "running".to_string(),
      description : Some( "Build docs".to_string() ),
      output_file_path : Some( "/tmp/out.txt".to_string() ),
    }
  );

  let sparse = attachment_line
  (
    r#"{"type":"task_status","taskId":"task_def67890","taskType":"background","status":"completed"}"#
  );
  assert_eq!
  (
    attachment_of( &sparse ),
    Attachment::TaskStatus
    {
      task_id : "task_def67890".to_string(),
      task_type : "background".to_string(),
      status : "completed".to_string(),
      description : None,
      output_file_path : None,
    }
  );
}

/// Test file and date attachments parse into their variants
///
/// ## Purpose
/// File attachments and date changes both alter what the model sees without
/// appearing as conversation content.
///
/// ## Coverage
/// `file`, `compact_file_reference`, `edited_text_file`, `command_permissions`,
/// `queued_command`, and `date_change`.
///
/// ## Validation Strategy
/// Asserts each variant's fields against the fixture.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — file and environment attachments
#[ test ]
fn event_file_and_environment_attachments_parse()
{
  let file = attachment_line
  (
    r#"{"type":"file","filename":"/home/alice/project/readme.md","displayPath":"readme.md","content":"the file's whole text"}"#
  );
  assert_eq!
  (
    attachment_of( &file ),
    Attachment::File
    {
      filename : "/home/alice/project/readme.md".to_string(),
      display_path : Some( "readme.md".to_string() ),
    }
  );

  let compact = attachment_line
  (
    r#"{"type":"compact_file_reference","filename":"/home/alice/project/src/lib.rs","displayPath":"src/lib.rs"}"#
  );
  assert_eq!
  (
    attachment_of( &compact ),
    Attachment::CompactFileReference
    {
      filename : "/home/alice/project/src/lib.rs".to_string(),
      display_path : Some( "src/lib.rs".to_string() ),
    }
  );

  let edited = attachment_line
  (
    r#"{"type":"edited_text_file","filename":"/home/alice/project/src/main.rs","snippet":"fn main() {}"}"#
  );
  assert_eq!
  (
    attachment_of( &edited ),
    Attachment::EditedTextFile { filename : "/home/alice/project/src/main.rs".to_string() }
  );

  let perms = attachment_line( r#"{"type":"command_permissions","allowedTools":["Bash(git *)","Edit"]}"# );
  assert_eq!
  (
    attachment_of( &perms ),
    Attachment::CommandPermissions
    {
      allowed_tools : vec![ "Bash(git *)".to_string(), "Edit".to_string() ],
    }
  );

  let queued = attachment_line
  (
    r#"{"type":"queued_command","prompt":"run the tests","commandMode":"prompt","timestamp":"2026-01-15T10:00:00.000Z"}"#
  );
  assert_eq!
  (
    attachment_of( &queued ),
    Attachment::QueuedCommand
    {
      prompt : Some( "run the tests".to_string() ),
      command_mode : Some( "prompt".to_string() ),
    }
  );

  let date = attachment_line( r#"{"type":"date_change","newDate":"2026-01-16"}"# );
  assert_eq!( attachment_of( &date ), Attachment::DateChange { new_date : "2026-01-16".to_string() } );
}

/// Test an unmodelled line kind is retained rather than rejected
///
/// ## Purpose
/// Claude Code's format grows between releases. A newer line kind must not
/// fail the fold that reads it, or a version bump silently breaks every reader.
///
/// ## Coverage
/// An unknown envelope `type` and an unknown `attachment.type`.
///
/// ## Validation Strategy
/// Asserts both parse successfully into their `Other` variants carrying the
/// unrecognized kind string.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — forward compatibility
#[ test ]
fn event_unknown_kinds_become_other()
{
  let envelope_event = parse( &envelope( "future-kind", r#","payload":1"# ) );
  assert_eq!( envelope_event.kind, EventKind::Other { kind : "future-kind".to_string() } );

  let attach = attachment_line( r#"{"type":"future_attachment","whatever":true}"# );
  assert_eq!( attachment_of( &attach ), Attachment::Other { kind : "future_attachment".to_string() } );
}

/// Test structurally invalid lines are rejected
///
/// ## Purpose
/// Forward compatibility covers unknown kinds, not malformed data. A line that
/// is not a typed JSON object is not a session line, and saying so is what
/// keeps the `Other` variant meaningful.
///
/// ## Coverage
/// Invalid JSON, a non-object JSON value, and an object with no `type`.
///
/// ## Validation Strategy
/// Asserts each returns `Err`.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — error contract
#[ test ]
fn event_malformed_lines_rejected()
{
  assert!( SessionEvent::from_json_line( "{not json" ).is_err() );
  assert!( SessionEvent::from_json_line( "[1,2,3]" ).is_err() );
  assert!( SessionEvent::from_json_line( r#"{"sessionId":"x"}"# ).is_err() );
}

/// Test a sidechain line is flagged as one
///
/// ## Purpose
/// Subagent conversations interleave into the same store; a fold reporting on
/// the main session must be able to tell them apart.
///
/// ## Coverage
/// A line with `isSidechain: true`.
///
/// ## Validation Strategy
/// Asserts the flag survives parsing.
///
/// ## Related Requirements
/// `docs/data_structure/003_session_event.md` — envelope shape
#[ test ]
fn event_sidechain_flag_retained()
{
  let line = format!
  (
    r#"{{"type":"user","sessionId":"{SESSION}","isSidechain":true,"message":{{"role":"user","content":"hi"}}}}"#
  );

  assert!( parse( &line ).is_sidechain );
}

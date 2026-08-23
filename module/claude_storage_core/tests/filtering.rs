//! Filtering integration tests
//!
//! ## Test Coverage
//!
//! 1. Session filtering (`agent_only`, `min_entries`, `session_id_substring`)
//! 2. Project filtering (`path_substring`, `min_sessions`)
//! 3. Filter composition (AND logic)
//! 4. Default filter (no filtering)
//! 5. Empty results
//!
//! ## Design
//!
//! Every test builds the same deterministic storage tree in a temp directory — see
//! `filter_storage` below — and asserts on exact result counts and exact project
//! and session identities. No test reads the developer's real `~/.claude/`
//! directory, so results never depend on the machine the suite runs on and no test
//! can be skipped by an empty-storage guard.
//!
//! Because each test owns its own temp root, the whole file is safe under
//! workspace-wide parallel execution: there is no shared filesystem state to
//! contend over.
#![ cfg( unix ) ]

mod storage_fixture;

use claude_storage_core::{ Storage, SessionFilter, ProjectFilter, ProjectId, StringMatcher, Session, Project };
use std::path::PathBuf;
use tempfile::TempDir;

/// Storage directory name of the fixture's first path project.
const ALPHA_DIR : &str = "-home-user-alpha";

/// Storage directory name of the fixture's second path project.
const BETA_DIR : &str = "-home-user-beta";

/// Storage directory name of the fixture's UUID project.
const UUID_DIR : &str = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";

/// Build the storage tree every test in this file asserts against.
///
/// ```text
/// projects/
///   -home-user-alpha/                            -> path project /home/user/alpha
///     11111111-1111-...-111111111111.jsonl   main,  12 entries
///     22222222-2222-...-222222222222.jsonl   main,   2 entries
///     33333333-3333-...-333333333333.jsonl   main,   0 entries
///     44444444-4444-...-444444444444.jsonl   main,   2 entries
///     55555555-5555-...-555555555555.jsonl   main,   2 entries
///     agent-aaaa1111.jsonl                   agent, 12 entries
///     agent-bbbb2222.jsonl                   agent,  2 entries
///   -home-user-beta/                             -> path project /home/user/beta
///     66666666-6666-...-666666666666.jsonl   main,   2 entries
///     77777777-7777-...-777777777777.jsonl   main,   2 entries
///   a1b2c3d4-e5f6-7890-abcd-ef1234567890/        -> uuid project
///     88888888-8888-...-888888888888.jsonl   main,   2 entries
/// ```
///
/// The returned temp root must stay alive for the whole test.
fn filter_storage() -> TempDir
{
  let temp = storage_fixture::storage_root();

  let alpha = storage_fixture::project_dir( temp.path(), ALPHA_DIR );
  storage_fixture::write_conversation_session( &alpha, "11111111-1111-1111-1111-111111111111", 12 );
  storage_fixture::write_conversation_session( &alpha, "22222222-2222-2222-2222-222222222222", 2 );
  storage_fixture::write_conversation_session( &alpha, "33333333-3333-3333-3333-333333333333", 0 );
  storage_fixture::write_conversation_session( &alpha, "44444444-4444-4444-4444-444444444444", 2 );
  storage_fixture::write_conversation_session( &alpha, "55555555-5555-5555-5555-555555555555", 2 );
  storage_fixture::write_conversation_session( &alpha, "agent-aaaa1111", 12 );
  storage_fixture::write_conversation_session( &alpha, "agent-bbbb2222", 2 );

  let beta = storage_fixture::project_dir( temp.path(), BETA_DIR );
  storage_fixture::write_conversation_session( &beta, "66666666-6666-6666-6666-666666666666", 2 );
  storage_fixture::write_conversation_session( &beta, "77777777-7777-7777-7777-777777777777", 2 );

  let uuid = storage_fixture::project_dir( temp.path(), UUID_DIR );
  storage_fixture::write_conversation_session( &uuid, "88888888-8888-8888-8888-888888888888", 2 );

  temp
}

/// Load the fixture's alpha project — the one every session-level test filters.
///
/// Goes through the path encoder rather than picking an arbitrary entry out of
/// `list_projects()`, whose order is unspecified.
fn alpha_project( temp : &TempDir ) -> Project
{
  Storage::with_root( temp.path() )
    .load_project_for_path( "/home/user/alpha" )
    .expect( "the alpha project must load" )
}

/// Session ids of `sessions`, sorted — directory iteration order is unspecified.
fn session_ids( sessions : &[ Session ] ) -> Vec< String >
{
  let mut ids : Vec< String > = sessions.iter().map( | s | s.id().to_string() ).collect();
  ids.sort();
  ids
}

/// Storage directory names of `projects`, sorted — directory iteration order is
/// unspecified.
fn project_dir_names( projects : &[ Project ] ) -> Vec< String >
{
  let mut names : Vec< String > = projects
    .iter()
    .map( | p | p.storage_dir().file_name().expect( "project dir has a name" ).to_string_lossy().into_owned() )
    .collect();
  names.sort();
  names
}

/// Test `SessionFilter` with `agent_only`
///
/// ## Purpose
///
/// Verifies `agent_only` selects sessions by the `agent-` filename prefix, keeping
/// exactly the agent sessions and excluding every main session.
///
/// ## Coverage
///
/// A project holding five main sessions and two agent sessions, filtered twice:
/// `agent_only : Some( true )` and `agent_only : Some( false )`.
///
/// ## Validation Strategy
///
/// Asserts the returned session ids equal exactly the two agent session names, then
/// re-asserts each result via `is_agent_session()`. The inverse filter is run as a
/// control and asserted to return exactly the five main sessions, which proves the
/// first result excluded genuine candidates rather than finding an empty project.
///
/// ## Related Requirements
///
/// `docs/data_structure/002_filter_types.md` § Structure — `SessionFilter` fields
#[ test ]
fn session_filter_agent_only()
{
  let temp = filter_storage();
  let mut project = alpha_project( &temp );

  let filter = SessionFilter
  {
    agent_only : Some( true ),
    min_entries : None,
    session_id_substring : None,
  };

  let filtered = project.sessions_filtered( &filter ).expect( "filtering must succeed" );

  assert_eq!
  (
    session_ids( &filtered ),
    [ "agent-aaaa1111", "agent-bbbb2222" ],
    "only the two agent sessions may match"
  );

  for session in &filtered
  {
    assert!( session.is_agent_session(), "every result must be an agent session" );
  }

  let main_only = SessionFilter
  {
    agent_only : Some( false ),
    min_entries : None,
    session_id_substring : None,
  };

  let mains = project.sessions_filtered( &main_only ).expect( "filtering must succeed" );
  assert_eq!( mains.len(), 5, "the inverse filter must return the five main sessions" );

  for session in &mains
  {
    assert!( !session.is_agent_session(), "the inverse filter must exclude agent sessions" );
  }
}

/// Test `SessionFilter` with `min_entries`
///
/// ## Purpose
///
/// Verifies `min_entries` keeps exactly the sessions whose entry count reaches the
/// threshold, counting both main and agent sessions.
///
/// ## Coverage
///
/// A project whose seven sessions hold 12, 2, 0, 2, 2, 12 and 2 entries, filtered at
/// a threshold of ten and again at a threshold of zero.
///
/// ## Validation Strategy
///
/// Asserts the ten-entry threshold returns exactly the two twelve-entry sessions by
/// id, and re-checks each result's real `count_entries()`. The zero threshold is run
/// as a control and asserted to return all seven, which proves the first result
/// excluded genuine candidates.
///
/// ## Related Requirements
///
/// `docs/data_structure/002_filter_types.md` § Structure — `SessionFilter` fields
#[ test ]
fn session_filter_min_entries()
{
  let temp = filter_storage();
  let mut project = alpha_project( &temp );

  let filter = SessionFilter
  {
    agent_only : None,
    min_entries : Some( 10 ),
    session_id_substring : None,
  };

  let filtered = project.sessions_filtered( &filter ).expect( "filtering must succeed" );

  assert_eq!
  (
    session_ids( &filtered ),
    [ "11111111-1111-1111-1111-111111111111", "agent-aaaa1111" ],
    "only the two twelve-entry sessions may match"
  );

  for session in &filtered
  {
    let count = session.count_entries().expect( "counting entries must succeed" );
    let sid = session.id();
    assert!( count >= 10, "session {sid} has {count} entries, expected >= 10" );
  }

  let no_threshold = SessionFilter
  {
    agent_only : None,
    min_entries : Some( 0 ),
    session_id_substring : None,
  };

  let all = project.sessions_filtered( &no_threshold ).expect( "filtering must succeed" );
  assert_eq!( all.len(), 7, "a zero threshold must admit every session, including the empty one" );
}

/// Test `SessionFilter` with `session_id_substring`
///
/// ## Purpose
///
/// Verifies `session_id_substring` keeps exactly the sessions whose id contains the
/// substring, across both main and agent sessions.
///
/// ## Coverage
///
/// A project where the substring `2222` appears in one main session id and one agent
/// session id, and in no other.
///
/// ## Validation Strategy
///
/// Asserts the returned ids equal exactly those two, then re-checks each through
/// `StringMatcher` — the same matcher the filter uses. Because five sibling sessions
/// lack the substring, an implementation that ignored the field entirely would
/// return seven and fail.
///
/// ## Related Requirements
///
/// `docs/data_structure/002_filter_types.md` § Structure — `SessionFilter` fields
#[ test ]
fn session_filter_id_substring()
{
  let temp = filter_storage();
  let mut project = alpha_project( &temp );

  let substring = "2222";
  let filter = SessionFilter
  {
    agent_only : None,
    min_entries : None,
    session_id_substring : Some( substring.to_string() ),
  };

  let filtered = project.sessions_filtered( &filter ).expect( "filtering must succeed" );

  assert_eq!
  (
    session_ids( &filtered ),
    [ "22222222-2222-2222-2222-222222222222", "agent-bbbb2222" ],
    "only the two sessions carrying the substring may match"
  );

  let matcher = StringMatcher::new( substring );
  for session in &filtered
  {
    let sid = session.id();
    assert!( matcher.matches( sid ), "session ID {sid} should contain {substring}" );
  }
}

/// Test `SessionFilter` with AND composition
///
/// ## Purpose
///
/// Verifies multiple `SessionFilter` fields compose with AND — a session must
/// satisfy every set condition, not merely one of them.
///
/// ## Coverage
///
/// A project where `agent_only` alone matches two sessions and `min_entries : 10`
/// alone matches two sessions, but only one session satisfies both.
///
/// ## Validation Strategy
///
/// Asserts the composed filter returns exactly that one session by id, then runs
/// each condition separately and asserts each returns two. Two-and-two-yields-one is
/// the signature of AND composition; OR composition would yield three, and a single
/// dominant condition would yield two.
///
/// ## Related Requirements
///
/// `docs/data_structure/002_filter_types.md` § Structure — AND composition
#[ test ]
fn session_filter_and_composition()
{
  let temp = filter_storage();
  let mut project = alpha_project( &temp );

  let composed = SessionFilter
  {
    agent_only : Some( true ),
    min_entries : Some( 10 ),
    session_id_substring : None,
  };

  let filtered = project.sessions_filtered( &composed ).expect( "filtering must succeed" );

  assert_eq!
  (
    session_ids( &filtered ),
    [ "agent-aaaa1111" ],
    "only the agent session that also clears the entry threshold may match"
  );

  for session in &filtered
  {
    assert!( session.is_agent_session(), "the result must satisfy the agent condition" );
    let count = session.count_entries().expect( "counting entries must succeed" );
    assert!( count >= 10, "the result must satisfy the entry-count condition, got {count}" );
  }

  let agent_only = SessionFilter
  {
    agent_only : Some( true ),
    min_entries : None,
    session_id_substring : None,
  };
  let min_entries = SessionFilter
  {
    agent_only : None,
    min_entries : Some( 10 ),
    session_id_substring : None,
  };

  assert_eq!
  (
    project.sessions_filtered( &agent_only ).expect( "filtering must succeed" ).len(),
    2,
    "the agent condition alone matches two sessions"
  );
  assert_eq!
  (
    project.sessions_filtered( &min_entries ).expect( "filtering must succeed" ).len(),
    2,
    "the entry-count condition alone matches two sessions"
  );
}

/// Test `ProjectFilter` with `path_substring`
///
/// ## Purpose
///
/// Verifies `path_substring` matches against a project's decoded path,
/// case-insensitively, and excludes both the sibling path project and the UUID
/// project.
///
/// ## Coverage
///
/// A storage root holding two path projects and one UUID project, filtered by a
/// substring unique to one path project — once lowercase, once uppercase.
///
/// ## Validation Strategy
///
/// Asserts exactly one project matches and that its id is the decoded
/// `ProjectId::Path`, proving the filter ran against the decoded path rather than
/// the encoded directory name. The uppercase run asserts the same single result,
/// pinning down case-insensitivity.
///
/// ## Related Requirements
///
/// `docs/data_structure/002_filter_types.md` § Structure — `ProjectFilter` fields
#[ test ]
fn project_filter_path_substring()
{
  let temp = filter_storage();
  let storage = Storage::with_root( temp.path() );

  let filter = ProjectFilter
  {
    path_substring : Some( "alpha".to_string() ),
    min_entries : None,
    min_sessions : None,
  };

  let filtered = storage.list_projects_filtered( &filter ).expect( "filtering must succeed" );

  assert_eq!( filtered.len(), 1, "only the alpha project's decoded path contains 'alpha'" );
  assert_eq!
  (
    filtered[ 0 ].id(),
    &ProjectId::Path( PathBuf::from( "/home/user/alpha" ) ),
    "the filter must match against the decoded path"
  );

  let upper = ProjectFilter
  {
    path_substring : Some( "ALPHA".to_string() ),
    min_entries : None,
    min_sessions : None,
  };

  assert_eq!
  (
    project_dir_names( &storage.list_projects_filtered( &upper ).expect( "filtering must succeed" ) ),
    [ ALPHA_DIR ],
    "path matching must be case-insensitive"
  );
}

/// Test `ProjectFilter` with `min_sessions`
///
/// ## Purpose
///
/// Verifies `min_sessions` keeps exactly the projects whose main-session count
/// reaches the threshold.
///
/// ## Coverage
///
/// A storage root holding projects with five, two and one main sessions, filtered at
/// a threshold of five and again at a threshold of two.
///
/// ## Validation Strategy
///
/// Asserts the five-session threshold returns only the alpha project by directory
/// name, and re-checks its real `count_sessions()`. The lower threshold is run as a
/// control and asserted to return alpha and beta but still exclude the
/// single-session UUID project, which pins the comparison to the threshold rather
/// than to a constant.
///
/// ## Related Requirements
///
/// `docs/data_structure/002_filter_types.md` § Structure — `ProjectFilter` fields
#[ test ]
fn project_filter_min_sessions()
{
  let temp = filter_storage();
  let storage = Storage::with_root( temp.path() );

  let filter = ProjectFilter
  {
    path_substring : None,
    min_entries : None,
    min_sessions : Some( 5 ),
  };

  let filtered = storage.list_projects_filtered( &filter ).expect( "filtering must succeed" );

  assert_eq!( project_dir_names( &filtered ), [ ALPHA_DIR ], "only alpha holds five main sessions" );

  for project in &filtered
  {
    let count = project.count_sessions().expect( "counting sessions must succeed" );
    assert!( count >= 5, "project has {count} sessions, expected >= 5" );
  }

  let lower = ProjectFilter
  {
    path_substring : None,
    min_entries : None,
    min_sessions : Some( 2 ),
  };

  assert_eq!
  (
    project_dir_names( &storage.list_projects_filtered( &lower ).expect( "filtering must succeed" ) ),
    [ ALPHA_DIR, BETA_DIR ],
    "a threshold of two admits alpha and beta but still excludes the single-session project"
  );
}

/// Test default filter (no filtering)
///
/// ## Purpose
///
/// Verifies a freshly constructed `ProjectFilter` reports itself as default and
/// returns every project, matching an unfiltered listing exactly.
///
/// ## Coverage
///
/// A storage root holding three projects — two path projects and one UUID project.
///
/// ## Validation Strategy
///
/// Asserts `is_default()`, then asserts the filtered listing equals the unfiltered
/// listing both in length and in the exact set of project directory names. Naming
/// all three is what makes the comparison meaningful — two empty listings would
/// otherwise compare equal.
///
/// ## Related Requirements
///
/// `docs/data_structure/002_filter_types.md` § Structure — default filter semantics
#[ test ]
fn default_filter_matches_all()
{
  let temp = filter_storage();
  let storage = Storage::with_root( temp.path() );

  let all_projects = storage.list_projects().expect( "listing must succeed" );
  assert_eq!
  (
    project_dir_names( &all_projects ),
    [ ALPHA_DIR, BETA_DIR, UUID_DIR ],
    "the fixture holds exactly three projects"
  );

  let filter = ProjectFilter::new();
  assert!( filter.is_default(), "a freshly constructed ProjectFilter must be the default" );

  let filtered = storage.list_projects_filtered( &filter ).expect( "filtering must succeed" );

  assert_eq!( filtered.len(), all_projects.len(), "the default filter must not drop a project" );
  assert_eq!
  (
    project_dir_names( &filtered ),
    [ ALPHA_DIR, BETA_DIR, UUID_DIR ],
    "the default filter must return every project"
  );
}

/// Test empty results
///
/// ## Purpose
///
/// Verifies a filter no project satisfies returns an empty result rather than an
/// error or an unfiltered listing.
///
/// ## Coverage
///
/// A non-empty storage root filtered twice: once by a substring no project path
/// contains, once by a substring both path projects share.
///
/// ## Validation Strategy
///
/// Asserts the impossible substring returns zero projects, then asserts a control
/// substring against the same storage returns exactly the two path projects. The
/// control is what proves the empty result came from the filter rather than from an
/// empty or unreadable storage root.
///
/// ## Related Requirements
///
/// `docs/data_structure/002_filter_types.md` § Structure — `ProjectFilter` fields
#[ test ]
fn filter_with_no_matches()
{
  let temp = filter_storage();
  let storage = Storage::with_root( temp.path() );

  let impossible = ProjectFilter
  {
    path_substring : Some( "definitely_does_not_exist_xyzabc123".to_string() ),
    min_entries : None,
    min_sessions : None,
  };

  let filtered = storage.list_projects_filtered( &impossible ).expect( "filtering must succeed" );

  assert!( filtered.is_empty(), "a substring no project contains must return no results" );

  let control = ProjectFilter
  {
    path_substring : Some( "/home/user/".to_string() ),
    min_entries : None,
    min_sessions : None,
  };

  assert_eq!
  (
    project_dir_names( &storage.list_projects_filtered( &control ).expect( "filtering must succeed" ) ),
    [ ALPHA_DIR, BETA_DIR ],
    "the storage is not empty — a shared path prefix still matches both path projects"
  );
}

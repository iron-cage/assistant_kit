//! Search functionality tests
//!
//! Tests for content search across sessions.
//!
//! Every test builds its own deterministic storage tree in a temp directory via
//! `storage_fixture` and asserts on exact match counts, entry indices, entry types,
//! line numbers, and excerpts. No test reads the developer's real `~/.claude/`
//! directory, so none of them can be skipped by an empty-storage guard.
#![ cfg( unix ) ]

mod storage_fixture;

use claude_storage_core::{ Storage, SearchFilter, EntryType };
use tempfile::TempDir;

/// Session id every single-session fixture in this file writes.
const SESSION : &str = "aaaaaaaa-1111-2222-3333-444444444444";

/// Build a one-project, one-session storage tree holding `lines`.
///
/// The returned temp root must stay alive for the whole test.
fn fixture( lines : &[ String ] ) -> TempDir
{
  let temp = storage_fixture::storage_root();
  let project = storage_fixture::project_dir( temp.path(), "-home-user-alpha" );
  storage_fixture::write_session( &project, SESSION, lines );
  temp
}

/// Test case-insensitive search (the default)
///
/// ## Purpose
///
/// Verifies a lowercase query matches content regardless of the stored casing, and
/// that non-matching entries are excluded rather than merely unchecked.
///
/// ## Coverage
///
/// A three-entry session: one entry containing the query capitalised, one
/// containing it lowercase, one containing it in no form at all.
///
/// ## Validation Strategy
///
/// Asserts an exact match count of two, then asserts each match's entry index,
/// entry type, and full line individually. The third entry's absence from the
/// results is what proves the search discriminates instead of matching everything.
///
/// ## Related Requirements
///
/// `docs/feature/002_content_search.md` § Design — case-insensitive default
#[ test ]
fn search_basic_case_insensitive()
{
  let lines =
  [
    storage_fixture::user_line( SESSION, 1, "The quick brown fox" ),
    storage_fixture::assistant_line( SESSION, 2, "the lazy dog sleeps" ),
    storage_fixture::user_line( SESSION, 3, "no needle in this one" ),
  ];
  let temp = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let matches = session.search( &SearchFilter::new( "the" ) ).expect( "search must succeed" );

  assert_eq!( matches.len(), 2, "a lowercase query must match both the capitalised and the lowercase entry" );

  assert_eq!( matches[ 0 ].entry_index(), 0 );
  assert_eq!( matches[ 0 ].entry_type(), EntryType::User );
  assert_eq!( matches[ 0 ].full_line(), "The quick brown fox" );

  assert_eq!( matches[ 1 ].entry_index(), 1 );
  assert_eq!( matches[ 1 ].entry_type(), EntryType::Assistant );
  assert_eq!( matches[ 1 ].full_line(), "the lazy dog sleeps" );
}

/// Test case-sensitive search
///
/// ## Purpose
///
/// Verifies `case_sensitive( true )` narrows the result set to exactly the entries
/// whose casing matches the query.
///
/// ## Coverage
///
/// The same three-entry session, searched twice: once case-sensitively and once
/// with the default case-insensitive behaviour.
///
/// ## Validation Strategy
///
/// Asserts the case-sensitive query returns exactly the one capitalised entry, then
/// runs the identical query without the flag as a control and asserts it returns
/// two. The contrast proves the flag — not the fixture — is what narrows the
/// result.
///
/// ## Related Requirements
///
/// `docs/feature/002_content_search.md` § Design — case sensitivity flag
#[ test ]
fn search_case_sensitive()
{
  let lines =
  [
    storage_fixture::user_line( SESSION, 1, "The quick brown fox" ),
    storage_fixture::assistant_line( SESSION, 2, "the lazy dog sleeps" ),
    storage_fixture::user_line( SESSION, 3, "no needle in this one" ),
  ];
  let temp = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let sensitive = session
    .search( &SearchFilter::new( "The" ).case_sensitive( true ) )
    .expect( "search must succeed" );

  assert_eq!( sensitive.len(), 1, "a case-sensitive query must match only the capitalised entry" );
  assert_eq!( sensitive[ 0 ].entry_index(), 0 );
  assert_eq!( sensitive[ 0 ].entry_type(), EntryType::User );
  assert_eq!( sensitive[ 0 ].full_line(), "The quick brown fox" );

  let insensitive = session
    .search( &SearchFilter::new( "The" ) )
    .expect( "search must succeed" );

  assert_eq!( insensitive.len(), 2, "the same query without the flag must match both casings" );
}

/// Test filtering search results by entry type
///
/// ## Purpose
///
/// Verifies `match_entry_type` restricts results to one role, and that the excluded
/// role's matching content really would have matched without the filter.
///
/// ## Coverage
///
/// A three-entry session where both a user entry and an assistant entry contain the
/// query.
///
/// ## Validation Strategy
///
/// Runs the same query three ways — user-only, assistant-only, and unfiltered — and
/// asserts exact counts of one, one, and two respectively, each with its entry index
/// and full line. The unfiltered run is what proves the two filtered runs each
/// excluded a genuine candidate rather than finding nothing.
///
/// ## Related Requirements
///
/// `docs/feature/002_content_search.md` § Design — entry type filtering
#[ test ]
fn search_filter_by_entry_type()
{
  let lines =
  [
    storage_fixture::user_line( SESSION, 1, "please help me" ),
    storage_fixture::assistant_line( SESSION, 2, "sure, i can help" ),
    storage_fixture::user_line( SESSION, 3, "unrelated content" ),
  ];
  let temp = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let user_only = session
    .search( &SearchFilter::new( "help" ).match_entry_type( EntryType::User ) )
    .expect( "search must succeed" );

  assert_eq!( user_only.len(), 1, "only the user entry may match a user-filtered search" );
  assert_eq!( user_only[ 0 ].entry_index(), 0 );
  assert_eq!( user_only[ 0 ].entry_type(), EntryType::User );
  assert_eq!( user_only[ 0 ].full_line(), "please help me" );

  let assistant_only = session
    .search( &SearchFilter::new( "help" ).match_entry_type( EntryType::Assistant ) )
    .expect( "search must succeed" );

  assert_eq!( assistant_only.len(), 1, "only the assistant entry may match an assistant-filtered search" );
  assert_eq!( assistant_only[ 0 ].entry_index(), 1 );
  assert_eq!( assistant_only[ 0 ].entry_type(), EntryType::Assistant );
  assert_eq!( assistant_only[ 0 ].full_line(), "sure, i can help" );

  let unfiltered = session
    .search( &SearchFilter::new( "help" ) )
    .expect( "search must succeed" );

  assert_eq!( unfiltered.len(), 2, "without a type filter both entries must match" );
}

/// Test a query that matches nothing
///
/// ## Purpose
///
/// Verifies a query absent from the session returns an empty result set rather than
/// an error or a spurious match.
///
/// ## Coverage
///
/// A two-entry session searched twice: once for a string that appears nowhere in it,
/// once for a string that appears in exactly one entry.
///
/// ## Validation Strategy
///
/// Asserts the impossible query returns zero matches, then asserts a control query
/// against the same session returns exactly one. The control is what proves the
/// empty result came from the query rather than from an unreadable or empty session.
///
/// ## Related Requirements
///
/// `docs/feature/002_content_search.md` § Design — no-match behaviour
#[ test ]
fn search_no_matches()
{
  let lines =
  [
    storage_fixture::user_line( SESSION, 1, "The quick brown fox" ),
    storage_fixture::assistant_line( SESSION, 2, "the lazy dog sleeps" ),
  ];
  let temp = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let none = session
    .search( &SearchFilter::new( "xyzabc123definitely_not_in_conversation" ) )
    .expect( "search must succeed" );

  assert!( none.is_empty(), "an absent query must return no matches" );

  let control = session
    .search( &SearchFilter::new( "brown" ) )
    .expect( "search must succeed" );

  assert_eq!( control.len(), 1, "the session is searchable — a present query still matches" );
  assert_eq!( control[ 0 ].full_line(), "The quick brown fox" );
}

/// Test the metadata carried on each search match
///
/// ## Purpose
///
/// Verifies every `SearchMatch` reports the right entry index, entry type, content
/// line number, full line, and excerpt — including the excerpt truncation rule for
/// lines longer than 150 characters.
///
/// ## Coverage
///
/// Three entries chosen to exercise each metadata field: a short single-line entry,
/// a 214-character entry that forces excerpt truncation, and a two-line entry whose
/// match falls on the second content line.
///
/// ## Validation Strategy
///
/// Asserts each field of each match against its exact expected value. The long
/// entry's expected excerpt is computed from the documented rule — the middle 100
/// characters wrapped in ellipsis markers — so a change to the truncation window
/// fails the test rather than being absorbed by a range check.
///
/// ## Related Requirements
///
/// `docs/feature/002_content_search.md` § Design — match metadata and excerpts
#[ test ]
fn search_match_metadata()
{
  let long_text = format!( "the long line {}", "x".repeat( 200 ) );
  let lines =
  [
    storage_fixture::user_line( SESSION, 1, "the short line" ),
    storage_fixture::assistant_line( SESSION, 2, &long_text ),
    storage_fixture::user_line( SESSION, 3, "alpha\\nthe second line" ),
  ];
  let temp = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let matches = session.search( &SearchFilter::new( "the" ) ).expect( "search must succeed" );

  assert_eq!( matches.len(), 3, "one match per entry" );

  // Short line: excerpt is the full line, untruncated.
  assert_eq!( matches[ 0 ].entry_index(), 0 );
  assert_eq!( matches[ 0 ].entry_type(), EntryType::User );
  assert_eq!( matches[ 0 ].line_number(), 0 );
  assert_eq!( matches[ 0 ].full_line(), "the short line" );
  assert_eq!( matches[ 0 ].excerpt(), "the short line", "a line of 150 chars or fewer is not truncated" );

  // Long line: full line preserved, excerpt is the middle 100 chars in ellipses.
  let expected_excerpt = format!( "...{}...", "x".repeat( 100 ) );
  assert_eq!( matches[ 1 ].entry_index(), 1 );
  assert_eq!( matches[ 1 ].entry_type(), EntryType::Assistant );
  assert_eq!( matches[ 1 ].line_number(), 0 );
  assert_eq!( matches[ 1 ].full_line(), long_text.as_str(), "the full line is never truncated" );
  assert_eq!( matches[ 1 ].excerpt(), expected_excerpt.as_str(), "a line over 150 chars keeps its middle 100" );
  assert_eq!( matches[ 1 ].excerpt().chars().count(), 106, "100 content chars plus two ellipsis markers" );

  // Two-line content: the match is reported against the second content line.
  assert_eq!( matches[ 2 ].entry_index(), 2 );
  assert_eq!( matches[ 2 ].entry_type(), EntryType::User );
  assert_eq!( matches[ 2 ].line_number(), 1, "line numbers are relative to the entry's own content" );
  assert_eq!( matches[ 2 ].full_line(), "the second line" );
  assert_eq!( matches[ 2 ].excerpt(), "the second line" );
}

/// Test that an empty query matches all content
///
/// ## Purpose
///
/// Verifies an empty query behaves like `StringMatcher`'s empty pattern — matching
/// every content line rather than none.
///
/// ## Coverage
///
/// A two-entry session searched with an empty query.
///
/// ## Validation Strategy
///
/// Asserts an exact match count of two — one per content line across both entries —
/// and asserts each match's full line, so a result set that is merely non-empty
/// cannot pass.
///
/// ## Related Requirements
///
/// `docs/feature/002_content_search.md` § Design — empty query semantics
#[ test ]
fn search_empty_query()
{
  let lines =
  [
    storage_fixture::user_line( SESSION, 1, "alpha" ),
    storage_fixture::assistant_line( SESSION, 2, "beta" ),
  ];
  let temp = fixture( &lines );
  let mut session = storage_fixture::single_session( temp.path() );

  let matches = session.search( &SearchFilter::new( "" ) ).expect( "search must succeed" );

  assert_eq!( matches.len(), 2, "an empty query must match every content line" );
  assert_eq!( matches[ 0 ].entry_index(), 0 );
  assert_eq!( matches[ 0 ].full_line(), "alpha" );
  assert_eq!( matches[ 1 ].entry_index(), 1 );
  assert_eq!( matches[ 1 ].full_line(), "beta" );
}

/// Test searching across multiple sessions of one project
///
/// ## Purpose
///
/// Verifies each session is searched independently — matches are counted per
/// session and never pooled or carried across session boundaries.
///
/// ## Coverage
///
/// One project holding three sessions with one, two, and zero matching entries
/// respectively.
///
/// ## Validation Strategy
///
/// Sorts the sessions by id — directory iteration order is unspecified — then
/// asserts the per-session match counts equal exactly `[ 1, 2, 0 ]` and that the
/// two-match session's results are its own entries in order. A pooled or
/// cross-contaminated implementation cannot produce that vector.
///
/// ## Related Requirements
///
/// `docs/feature/002_content_search.md` § Design — per-session search scope
#[ test ]
fn search_multiple_sessions()
{
  let temp = storage_fixture::storage_root();
  let project = storage_fixture::project_dir( temp.path(), "-home-user-alpha" );

  let one = "11111111-1111-1111-1111-111111111111";
  let two = "22222222-2222-2222-2222-222222222222";
  let three = "33333333-3333-3333-3333-333333333333";

  let one_lines = [ storage_fixture::user_line( one, 1, "the alpha entry" ) ];
  let two_lines =
  [
    storage_fixture::user_line( two, 1, "the beta entry" ),
    storage_fixture::assistant_line( two, 2, "the gamma reply" ),
  ];
  let three_lines = [ storage_fixture::user_line( three, 1, "no match at all" ) ];

  storage_fixture::write_session( &project, one, &one_lines );
  storage_fixture::write_session( &project, two, &two_lines );
  storage_fixture::write_session( &project, three, &three_lines );

  let storage = Storage::with_root( temp.path() );
  let projects = storage.list_projects().expect( "list projects" );
  assert_eq!( projects.len(), 1, "fixture holds exactly one project" );

  let mut sessions = projects[ 0 ].sessions().expect( "list sessions" );
  assert_eq!( sessions.len(), 3, "fixture holds exactly three sessions" );
  // Directory iteration order is unspecified — sort so per-session expectations hold.
  sessions.sort_by( | a, b | a.id().cmp( b.id() ) );

  let filter = SearchFilter::new( "the" );
  let counts : Vec< usize > = sessions
    .iter_mut()
    .map( | session | session.search( &filter ).expect( "search must succeed" ).len() )
    .collect();

  assert_eq!( counts, [ 1, 2, 0 ], "matches must be counted per session, never pooled" );

  let second = sessions[ 1 ].search( &filter ).expect( "search must succeed" );
  assert_eq!( second[ 0 ].entry_type(), EntryType::User );
  assert_eq!( second[ 0 ].full_line(), "the beta entry" );
  assert_eq!( second[ 1 ].entry_type(), EntryType::Assistant );
  assert_eq!( second[ 1 ].full_line(), "the gamma reply" );
}

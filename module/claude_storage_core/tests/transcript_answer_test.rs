//! One turn's answer, read out of a transcript rather than off a terminal.
//!
//! # Why this exists
//!
//! A caller hosting a live interactive session cannot read the answer from what
//! the session prints. What physically arrives is Claude Code's interface,
//! redrawn frame by frame, with the words somewhere inside it. Printing that was
//! the observed defect: a smoke run answered `pineapple` correctly and displayed
//! box rules, a `❯` prompt, a status bar, and spinner frames around it.
//!
//! The answer is instead read from the session's own transcript, which Claude
//! Code writes as it goes and keys by the same conversation id the caller already
//! holds. These tests pin the reading, because a wrong read is not a crash — it is
//! a plausible-looking answer with the wrong words in it.
//!
//! # What is not tested here
//!
//! A real turn. That needs a real model call, and it belongs to whichever crate
//! owns the session — here, the first consumer is `claude_runner`'s `clr chat`
//! (`module/claude_runner/tests/manual/readme.md`). The fixtures below are
//! hand-built to the shape a real transcript actually has — the required-field
//! list comes from this crate's own parser, and the block mix (thinking, tool use,
//! tool result, text) from a recorded session.
//!
//! # Coverage
//!
//! | ID | Aspect | Expectation |
//! |----|--------|-------------|
//! | CA-1 | An assistant text block past the mark | Is the answer; the user's own prompt is not |
//! | CA-2 | Thinking and tool blocks in the same turn | Excluded — only text survives |
//! | CA-3 | A second turn appended after a first | Only the second turn's answer comes back |
//! | CA-4 | A transcript that does not exist | `mark` is 0 and the answer is `None`, so the caller falls back |
//! | CA-5 | Two text blocks in one turn | Joined in order, not silently reduced to one |
//! | CA-6 | A transcript written after the wait began | Found within the grace period |
//! | CA-7 | `transcript_path` | Names `<encoded cwd>/<session id>.jsonl` |
//! | CA-8 | Non-conversation lines around the answer | Neither shift the mark nor leak into the answer |

use core::time::Duration;
use std::path::{ Path, PathBuf };

use claude_storage_core::{ transcript_answer_since, transcript_mark, transcript_path };

/// A conversation id shaped like a real one — the filename is derived from it.
const SESSION : &str = "e63e8705-bb43-4bea-afad-5faf95411e33";

/// Long enough to cover a filesystem hiccup, short enough that a test asserting
/// absence does not stall the suite. Tests asserting *presence* pass immediately.
const SHORT_GRACE : Duration = Duration::from_millis( 200 );

/// A scratch transcript, deleted with the test.
struct Transcript
{
  dir : tempfile::TempDir,
}

impl Transcript
{
  fn new() -> Self
  {
    Self { dir : tempfile::tempdir().expect( "no tempdir" ) }
  }

  fn path( &self ) -> PathBuf
  {
    self.dir.path().join( format!( "{SESSION}.jsonl" ) )
  }

  /// Append lines, exactly as Claude Code appends them.
  fn append( &self, lines : &[ String ] )
  {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
      .create( true )
      .append( true )
      .open( self.path() )
      .expect( "cannot open transcript" );
    for line in lines
    {
      writeln!( file, "{line}" ).expect( "cannot write transcript" );
    }
  }
}

/// The fields every conversation entry must carry for the parser to accept it.
///
/// Not decoration: this crate rejects a line missing any of them, and a rejected
/// line is skipped rather than reported. A fixture that drops one would silently
/// test an empty file. `cwd` here is the session's own directory recorded inside
/// each entry — nothing under test reads it.
fn common( kind : &str, uuid : &str ) -> String
{
  format!(
    r#""type":"{kind}","uuid":"{uuid}","timestamp":"2026-08-26T22:00:00.000Z","cwd":"/tmp/work","sessionId":"{SESSION}","version":"2.1.220","userType":"external""#
  )
}

/// One typed prompt.
fn user_line( uuid : &str, text : &str ) -> String
{
  format!( r#"{{{},"message":{{"role":"user","content":"{text}"}}}}"#, common( "user", uuid ) )
}

/// One assistant turn, whose `blocks` are already-rendered JSON content blocks.
fn assistant_line( uuid : &str, blocks : &[ &str ] ) -> String
{
  format!(
    r#"{{{},"requestId":"req_{uuid}","message":{{"role":"assistant","id":"msg_{uuid}","model":"claude-opus-5","content":[{}]}}}}"#,
    common( "assistant", uuid ),
    blocks.join( "," ),
  )
}

fn text_block( text : &str ) -> String
{
  format!( r#"{{"type":"text","text":"{text}"}}"# )
}

fn thinking_block( text : &str ) -> String
{
  format!( r#"{{"type":"thinking","thinking":"{text}","signature":"sig"}}"# )
}

fn tool_use_block( name : &str ) -> String
{
  format!( r#"{{"type":"tool_use","id":"toolu_1","name":"{name}","input":{{"command":"ls"}}}}"# )
}

fn tool_result_block( content : &str ) -> String
{
  format!( r#"{{"type":"tool_result","tool_use_id":"toolu_1","content":"{content}","is_error":false}}"# )
}

#[ test ]
fn ca1_an_assistant_text_block_is_the_answer()
{
  let transcript = Transcript::new();
  transcript.append(
  &[
    user_line( "u1", "Reply with exactly one word: pineapple" ),
    assistant_line( "a1", &[ &text_block( "pineapple" ) ] ),
  ] );

  let answer = transcript_answer_since( &transcript.path(), 0, SHORT_GRACE );

  assert_eq!( answer.as_deref(), Some( "pineapple" ), "the assistant's text is the answer" );
}

#[ test ]
fn ca2_thinking_and_tool_blocks_are_not_the_answer()
{
  let transcript = Transcript::new();
  transcript.append(
  &[
    user_line( "u1", "What is in this directory?" ),
    assistant_line( "a1",
    &[
      &thinking_block( "The user wants a listing. I should run ls." ),
      &tool_use_block( "Bash" ),
    ] ),
    // A tool result comes back as a *user* entry, which is why filtering on the
    // entry type alone would not have been enough.
    format!( r#"{{{},"message":{{"role":"user","content":[{}]}}}}"#,
      common( "user", "u2" ), tool_result_block( "readme.md" ) ),
    assistant_line( "a2", &[ &text_block( "One file: readme.md" ) ] ),
  ] );

  let answer = transcript_answer_since( &transcript.path(), 0, SHORT_GRACE ).expect( "no answer" );

  assert_eq!( answer, "One file: readme.md", "only text blocks are the answer" );
  assert!( !answer.contains( "should run ls" ), "a thinking block reached the user:\n{answer}" );
  assert!( !answer.contains( "Bash" ), "a tool call reached the user:\n{answer}" );
  assert!( !answer.contains( "toolu_1" ), "a tool result reached the user:\n{answer}" );
}

#[ test ]
fn ca3_the_mark_excludes_the_previous_turn()
{
  let transcript = Transcript::new();
  transcript.append(
  &[
    user_line( "u1", "Reply with exactly one word: pineapple" ),
    assistant_line( "a1", &[ &text_block( "pineapple" ) ] ),
  ] );

  // Exactly what a caller does between turns: mark, then send.
  let mark = transcript_mark( &transcript.path() );
  assert_eq!( mark, 2, "two conversation entries so far" );

  transcript.append(
  &[
    user_line( "u2", "What word was that?" ),
    assistant_line( "a2", &[ &text_block( "pineapple, as you asked" ) ] ),
  ] );

  let answer = transcript_answer_since( &transcript.path(), mark, SHORT_GRACE ).expect( "no answer" );

  assert_eq!( answer, "pineapple, as you asked", "the second turn is the whole answer" );
}

#[ test ]
fn ca4_a_missing_transcript_falls_back_rather_than_failing()
{
  let transcript = Transcript::new();
  let path = transcript.path();
  assert!( !path.exists(), "the fixture wrote nothing" );

  assert_eq!( transcript_mark( &path ), 0, "a file that is not there is zero entries, not an error" );
  assert!
  (
    transcript_answer_since( &path, 0, SHORT_GRACE ).is_none(),
    "a missing transcript must report nothing, so the caller can fall back"
  );
}

#[ test ]
fn ca5_several_text_blocks_are_joined_in_order()
{
  let transcript = Transcript::new();
  transcript.append(
  &[
    user_line( "u1", "Say two things." ),
    assistant_line( "a1", &[ &text_block( "First." ), &text_block( "Second." ) ] ),
  ] );

  let answer = transcript_answer_since( &transcript.path(), 0, SHORT_GRACE ).expect( "no answer" );

  assert_eq!( answer, "First.\n\nSecond.", "both blocks, in the order written" );
}

#[ test ]
fn ca6_the_grace_period_waits_for_a_transcript_that_is_still_being_written()
{
  let transcript = Transcript::new();
  let path = transcript.path();

  let writer_path = path.clone();
  let writer = std::thread::spawn( move ||
  {
    std::thread::sleep( Duration::from_millis( 150 ) );
    use std::io::Write;
    let mut file = std::fs::File::create( &writer_path ).expect( "cannot create transcript" );
    writeln!( file, "{}", user_line( "u1", "Anything." ) ).expect( "write failed" );
    writeln!( file, "{}", assistant_line( "a1", &[ &text_block( "late but present" ) ] ) )
      .expect( "write failed" );
  } );

  // Two seconds against a 150ms delay: the assertion is that waiting happens at
  // all, not that it happens to a schedule.
  let answer = transcript_answer_since( &path, 0, Duration::from_secs( 2 ) );
  writer.join().expect( "writer panicked" );

  assert_eq!
  (
    answer.as_deref(),
    Some( "late but present" ),
    "the turn ends before the transcript is flushed — giving up at that instant leaves the caller nothing"
  );
}

#[ test ]
fn ca7_the_transcript_is_named_after_the_session()
{
  // No env is set or cleared here, deliberately: `set_var` is process-global, and
  // this file must stay safe to run concurrently with every other test in the
  // crate. Whatever HOME the suite runs under is enough — the claim under test is
  // the shape of the path, not where its root is.
  let Some( path ) = transcript_path( Path::new( "/tmp/work" ), SESSION ) else
  {
    // Only reachable with neither CLAUDE_HOME nor HOME set, which is exactly the
    // case a caller treats as "there is no transcript to read".
    return;
  };

  assert_eq!
  (
    path.file_name().and_then( std::ffi::OsStr::to_str ),
    Some( "e63e8705-bb43-4bea-afad-5faf95411e33.jsonl" ),
    "the transcript is the conversation id plus .jsonl — the whole reason a session id is enough to find it"
  );
  assert_eq!
  (
    path.parent().and_then( Path::file_name ).and_then( std::ffi::OsStr::to_str ),
    Some( "-tmp-work" ),
    "the directory is the lossy encoding of the session's cwd"
  );
}

#[ test ]
fn ca8_non_conversation_lines_are_neither_counted_nor_printed()
{
  let transcript = Transcript::new();
  transcript.append(
  &[
    // Every one of these appears in a real transcript, and none of them is a
    // conversation entry. This crate skips them; the mark must agree, or the
    // slice would start in the wrong place on the next turn.
    r#"{"type":"mode","mode":"default","timestamp":"2026-08-26T22:00:00.000Z"}"#.to_string(),
    r#"{"type":"summary","summary":"A chat about fruit","leafUuid":"a1"}"#.to_string(),
    user_line( "u1", "Reply with exactly one word: pineapple" ),
    r#"{"type":"attachment","attachment":{"type":"file","path":"/tmp/work/readme.md"}}"#.to_string(),
    assistant_line( "a1", &[ &text_block( "pineapple" ) ] ),
    r#"{"type":"system","subtype":"local_command","content":"done"}"#.to_string(),
  ] );

  assert_eq!
  (
    transcript_mark( &transcript.path() ),
    2,
    "six lines, two of them conversation — a mark that counted lines would desynchronise every later turn"
  );

  let answer = transcript_answer_since( &transcript.path(), 0, SHORT_GRACE ).expect( "no answer" );
  assert_eq!( answer, "pineapple", "only the assistant's own words:\n{answer}" );
}

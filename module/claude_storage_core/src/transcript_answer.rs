//! Reading one turn's assistant answer out of a session transcript.
//!
//! # Terms
//!
//! A **transcript** is the on-disk JSONL file at `<claude home>/projects/<encoded
//! cwd>/<session id>.jsonl`. [`Session`] is the parsed view of one. The two words
//! are not synonyms here: the transcript is the file being appended to *while*
//! this module reads it, which is the whole reason [`transcript_answer_since`]
//! waits.
//!
//! # Why read the file at all
//!
//! Because for a caller hosting a live interactive session, the transcript is the
//! only place the answer exists as *data*. What arrives over a terminal is a
//! picture of an interface — an input box redrawn every frame, a status bar,
//! spinner glyphs, box rules, with the words threaded through them. Rendering
//! those bytes faithfully (`claude_terminal_core::to_plain_text`) reproduces
//! exactly that picture. Correct, and unusable as an answer.
//!
//! Filtering the chrome out does not fix it either, because the chrome belongs to
//! Claude Code and changes whenever its interface does — a consumer doing that
//! would be pinned to a TUI layout it does not own.
//!
//! Claude Code writes the transcript as it goes, keyed by the conversation id a
//! caller already holds. The id names the file; the file holds the words.
//!
//! # What counts as the answer
//!
//! Text blocks from assistant entries, and nothing else. Not thinking blocks, not
//! tool-call parameters, not tool results — those are how the answer was reached.
//! [`Entry::content_text`] flattens all of them together, which is right for
//! searching and wrong for this, so the blocks are walked directly here.
//!
//! # Why the mark is an entry count
//!
//! Not a byte offset and not a line count. A transcript carries non-conversation
//! lines — `summary`, `mode`, `attachment`, `system` — which [`Session`] skips.
//! A mark that counted lines would desynchronise from the entry slice on the very
//! next turn.

use core::time::Duration;
use std::path::{ Path, PathBuf };
use std::time::Instant;

use crate::{ ContentBlock, Entry, EntryType, Session };

/// Gap between attempts to see a finished turn in the transcript.
const POLL : Duration = Duration::from_millis( 100 );

/// Where the transcript for `session_id` in `cwd` lives, if it can be named.
///
/// `None` when the Claude home cannot be resolved or `cwd` will not encode —
/// both of which mean the same thing to a caller: there is no file to read.
///
/// The path is not checked for existence. A session spawned moments ago has no
/// transcript until its first turn produces one, so absence is the ordinary case
/// rather than an error.
#[ inline ]
#[ must_use ]
pub fn transcript_path( cwd : &Path, session_id : &str ) -> Option< PathBuf >
{
  crate::to_storage_path_for( cwd )
    .map( | dir | dir.join( format!( "{session_id}.jsonl" ) ) )
}

/// How many conversation entries the transcript holds right now.
///
/// Taken before a prompt is sent, so that everything after it is that turn.
/// A file that does not exist yet is zero entries, not an error.
#[ inline ]
#[ must_use ]
pub fn transcript_mark( path : &Path ) -> usize
{
  Session::load( path )
    .and_then( | mut session | session.entries().map( Vec::len ) )
    .unwrap_or( 0 )
}

/// The assistant's text written after `mark`, once there is any.
///
/// Returns `None` if nothing new arrives within `grace`, which a caller should
/// read as "there is nothing to show from here" rather than "the session said
/// nothing".
///
/// The grace period exists because the two ends of a turn are not the same
/// instant. The turn is over when the session goes idle and its output stops; the
/// transcript is complete when Claude Code has finished flushing it. The second
/// follows the first closely but not atomically, so waiting a moment for a file
/// that is about to be written beats reporting an empty answer.
///
/// Blocking, by `POLL`-length sleeps, for at most `grace`.
#[ inline ]
#[ must_use ]
pub fn transcript_answer_since( path : &Path, mark : usize, grace : Duration ) -> Option< String >
{
  let deadline = Instant::now() + grace;

  loop
  {
    if let Some( answer ) = read_answer( path, mark )
    {
      return Some( answer );
    }
    if Instant::now() >= deadline
    {
      return None;
    }
    std::thread::sleep( POLL );
  }
}

/// One attempt: the assistant text past `mark`, or `None` if there is none yet.
///
/// Re-reads the whole file every attempt rather than tailing it. A transcript is
/// a handful of kilobytes next to the model call that just produced it, and a
/// tail would have to re-implement the graceful skipping of non-conversation
/// lines that [`Session`] already does.
fn read_answer( path : &Path, mark : usize ) -> Option< String >
{
  let mut session = Session::load( path ).ok()?;
  let entries = session.entries().ok()?;

  // Fewer entries than the mark means the file was replaced under us — a
  // `--continue` elsewhere, a rewrite. Nothing sensible can be sliced from it.
  let fresh = entries.get( mark.. )?;

  let answer = fresh
    .iter()
    .filter( | entry | entry.entry_type() == EntryType::Assistant )
    .flat_map( Entry::content_blocks )
    .filter_map( | block | match block
    {
      ContentBlock::Text { text } => Some( text.trim_end() ),
      _ => None,
    } )
    .filter( | text | !text.is_empty() )
    .collect::< Vec< _ > >()
    .join( "\n\n" );

  ( !answer.is_empty() ).then_some( answer )
}

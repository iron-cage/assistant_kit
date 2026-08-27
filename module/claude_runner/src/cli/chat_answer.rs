//! Reading a turn's answer out of the session transcript.
//!
//! # Why not out of the terminal
//!
//! Because the terminal does not contain an answer. It contains a picture of one.
//!
//! A hosted session is Claude Code's full interactive interface, and what arrives
//! over the pty is what a person would *see*: an input box redrawn on every
//! frame, a status bar, spinner glyphs, a tip banner, box rules, and the answer
//! threaded somewhere through all of it. Rendering those bytes faithfully — which
//! is what [`claude_daemon_core::to_plain_text`] does — produces exactly that
//! picture, spinner frames and all. Correct, and unusable as the output of a
//! command whose whole promise is "prompt in, answer out".
//!
//! No amount of filtering fixes that, because the chrome being filtered is
//! Claude Code's, and it changes whenever its interface does. A `clr` release
//! would be pinned to a TUI layout it does not own.
//!
//! # The answer is already data
//!
//! Claude Code writes every conversation to `<claude home>/projects/<encoded
//! cwd>/<session id>.jsonl` as it goes — structured entries, one JSON object per
//! line, with the assistant's text in text blocks. That file is keyed by the same
//! conversation id the daemon already holds, which is what makes this possible at
//! all: the daemon knows the id, the id names the file, the file holds the words.
//!
//! So the pty keeps doing the thing only a pty can do — carry a real interactive
//! session, statefully, across turns — and the transcript answers the question
//! the pty is bad at: what did it actually say.
//!
//! # What counts as the answer
//!
//! Text blocks from assistant entries, and nothing else. Not thinking blocks, not
//! tool-call parameters, not tool results — those are how the answer was reached,
//! and print mode does not print them either. [`claude_storage_core`]'s own
//! `content_text` flattens all of them together, which is right for searching and
//! wrong for this, so the blocks are walked directly.

use core::time::Duration;
use std::path::{ Path, PathBuf };
use std::time::Instant;

use claude_storage_core::{ ContentBlock, EntryType, Session };

/// Gap between attempts to see a finished turn in the transcript.
const POLL : Duration = Duration::from_millis( 100 );

/// Where the transcript for `session_id` in `cwd` lives, if it can be named.
///
/// `None` when the Claude home cannot be resolved or `cwd` will not encode —
/// both of which mean the same thing to a caller: fall back to the terminal.
#[ inline ]
#[ must_use ]
pub fn transcript_path( cwd : &Path, session_id : &str ) -> Option< PathBuf >
{
  claude_storage_core::to_storage_path_for( cwd )
    .map( | dir | dir.join( format!( "{session_id}.jsonl" ) ) )
}

/// How many conversation entries the transcript holds right now.
///
/// Taken before the prompt is sent, so that everything after it is this turn.
/// A file that does not exist yet is zero entries, not an error — a session
/// spawned moments ago has no transcript until its first turn produces one, and
/// that is the ordinary case for the first `clr chat` in a directory.
#[ inline ]
#[ must_use ]
pub fn mark( path : &Path ) -> usize
{
  Session::load( path )
    .and_then( | mut session | session.entries().map( Vec::len ) )
    .unwrap_or( 0 )
}

/// The assistant's text written after `mark`, once there is any.
///
/// Returns `None` if nothing new arrives within `grace`, which the caller should
/// read as "use the terminal instead" rather than "the session said nothing".
///
/// The grace period exists because the two ends of a turn are not the same
/// instant. The turn is over when the session goes idle and its output stops;
/// the transcript is complete when Claude Code has finished flushing it. The
/// second follows the first closely but not atomically, so waiting a moment for
/// a file that is about to be written beats reporting an empty answer.
#[ inline ]
#[ must_use ]
pub fn answer_since( path : &Path, mark : usize, grace : Duration ) -> Option< String >
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
    .flat_map( claude_storage_core::Entry::content_blocks )
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

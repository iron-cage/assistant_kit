//! Rendering a session's context summary for the wire.
//!
//! [`Request::ContextSummary`] answers "what is in this session's context right
//! now" — which tools are deferred, which agents and skills are on offer, how
//! much of the token budget is left, what background tasks are running.
//!
//! None of that is held by the daemon. The daemon knows a session's id and cwd;
//! everything else lives in the session's own transcript, which Claude Code
//! writes and `claude_storage_core` reads. So this module locates the transcript
//! and asks that crate for two halves of the answer:
//!
//! - [`ContextFold`] replays the transcript's context deltas into current state.
//! - [`Session::stats`] sums token *usage*, deduplicating by `message.id`.
//!
//! The split is deliberate and documented in `claude_storage_core`'s
//! `data_structure/004_session_context_state.md`: the remaining budget is the
//! harness's own number, while usage is already computed once and must not be
//! computed twice.
//!
//! # The four token figures, and why none of them is the others
//!
//! - `remaining` — what the harness says is left. Reported, never computed.
//! - `context` — what the newest call's prompt cost, `input + cache_read +
//!   cache_creation`. This is how full the conversation is *now*.
//! - `peak_context` — the same measure at its high-water mark. It never falls,
//!   so after a compaction it describes a conversation that no longer exists.
//! - `input`/`output`/`cache_*` — sums across every call. Every turn re-sends
//!   the whole conversation, so in a long session these run to many times the
//!   window. They answer "what did this cost", never "how full is it".
//!
//! The static system prompt is absent from the transcript as *text*, but its
//! cost is inside `context` — the figure is what the API billed for the entire
//! prompt, tools and system prompt included. So the budget is derivable here
//! after all, and `window` ([`window`]) is that derivation.
//!
//! # The one figure that needs a measurement
//!
//! What `context` will not say is how much of itself was spent before the
//! conversation started. That split needs [`crate::baseline`], and a baseline
//! needs an API call, so it is never taken here — this module only *reads* a
//! measurement someone else took, keyed by the session's own version and model.
//! With none on file, `static_overhead` and `conversation` are `null`, which is
//! the truthful answer rather than a guessed one.
//!
//! # Why the JSON is built here
//!
//! `claude_storage_core` has zero runtime dependencies — no serde — so its types
//! do not serialize themselves. Rendering them is this crate's job, and keeping
//! it that way is what preserves that crate's dependency guarantee. The cost is
//! this module: a field-by-field projection, which also means the wire shape is
//! owned by the protocol rather than falling out of a struct layout.
//!
//! [`Request::ContextSummary`]: crate::Request::ContextSummary
//! [`ContextFold`]: claude_storage_core::ContextFold
//! [`Session::stats`]: claude_storage_core::Session

use std::path::Path;

use claude_storage_core::{ ContextFold, Session, SessionContextState, transcript_path };
use serde_json::{ Value, json };

use crate::baseline::StaticBaseline;
use crate::{ Error, Result };

/// Build the context summary for `session_id` running in `cwd`.
///
/// `baselines` names the directory a [`crate::baseline`] measurement may have
/// been cached in. Passing `None` — or naming a directory with no measurement
/// matching this session's version and model — reports the overhead split as
/// `null` and changes nothing else.
///
/// # Errors
///
/// Returns [`Error::NoTranscript`] when the working directory will not encode to
/// a storage path, or when the session has not written a transcript yet — a
/// session spawned moments ago legitimately has none, and saying so beats an
/// empty summary that would read as "this session's context is empty".
///
/// Returns [`Error::Storage`] when the transcript exists but cannot be read or
/// folded.
#[ inline ]
pub fn summary( cwd : &Path, session_id : &str, baselines : Option< &Path > ) -> Result< Value >
{
  let path = transcript_path( cwd, session_id )
    .ok_or_else( || Error::NoTranscript { session_id : session_id.to_string() } )?;

  if !path.exists()
  {
    return Err( Error::NoTranscript { session_id : session_id.to_string() } );
  }

  let mut fold = ContextFold::new();
  fold.read_file( &path )?;

  let mut session = Session::load( &path )?;
  let usage = session.stats()?;

  let state = fold.state();
  let floor = baselines.and_then( | dir | measured_floor( dir, state, &usage ) );

  Ok( json!
  ( {
    "session_id" : session_id,
    "transcript" : path,
    "version" : state.version,
    "cwd" : state.cwd,
    "mode" : state.mode,
    "permission_mode" : state.permission_mode,
    "title" : state.title,
    "date" : state.date,
    "tokens" :
    {
      // Reported, not derived — see the module note on the split.
      "remaining" : state.tokens_remaining,
      // How full the conversation is now, and the most it has ever been. Both
      // are one call's whole prompt; neither is a sum over calls.
      "context" : usage.last_context_tokens,
      "peak_context" : usage.max_context_tokens,
      // The model's usable window, when this session says enough to place it.
      "window" : window( state.tokens_remaining, usage.last_context_tokens ),
      // How `context` divides into what was there before the first word and what
      // the conversation added. Both null until a baseline has been measured for
      // this session's version and model — see the module note.
      "static_overhead" : floor.as_ref().map( | one | one.prompt_tokens ),
      "conversation" : floor
        .as_ref()
        .map( | one | one.conversation_tokens( usage.last_context_tokens ) ),
      "input" : usage.total_input_tokens,
      "output" : usage.total_output_tokens,
      "cache_read" : usage.total_cache_read_tokens,
      "cache_creation" : usage.total_cache_creation_tokens,
    },
    "deferred_tools" : state.deferred_tools,
    "pending_mcp_servers" : state.pending_mcp_servers,
    "agent_types" : state.agent_types,
    "mcp_servers" : state.mcp_servers,
    "skills" :
    {
      "available" : state.skills_available,
      "reported_count" : state.skills_reported_count,
      "truncated" : state.skills_truncated(),
      "invoked" : invoked_skills( state ),
    },
    "allowed_tools" : state.allowed_tools,
    "attached_files" : state.attached_files,
    "tasks" : tasks( state ),
    "task_reminder_items" : state.task_reminder_items,
    "counters" : counters( state ),
  } ))
}

/// The model's usable context window, when this session says enough to place it.
///
/// Neither half is the window on its own. `remaining` is what the harness says
/// is left; `context` is what the newest call actually sent. Their sum is the
/// whole of it, and it is the only figure here that does not appear in the
/// transcript in any form — the window is a property of the model and the
/// deployment, never of the conversation.
///
/// `None` until a turn has both reported a budget and been billed for a prompt.
/// A fresh session has neither, and guessing a window there would be a fabricated
/// number a client could not tell from a measured one.
fn window( remaining : Option< u64 >, context : u64 ) -> Option< u64 >
{
  // A zero context means no assistant turn has happened yet, so there is nothing
  // to add the remainder to — `Some( remaining )` here would claim the window
  // equals the budget, which is only true of a conversation that costs nothing.
  if context == 0
  {
    return None;
  }

  remaining.map( | left | left.saturating_add( context ) )
}

/// The cached baseline that actually applies to this session, if one does.
///
/// Both halves of the key come from the session's own transcript rather than
/// from the caller: `version` is what wrote its newest line, `model` is what
/// answered its first assistant turn. A measurement taken against anything else
/// describes a different floor, and applying it would subtract a number that was
/// never part of this session's prompt.
///
/// A session with neither yet — no assistant turn, so no model — has no
/// applicable measurement, which is the same `None` as having taken none.
fn measured_floor
(
  baselines : &Path,
  state : &SessionContextState,
  usage : &claude_storage_core::SessionStats,
)
-> Option< StaticBaseline >
{
  let version = state.version.as_deref()?;
  let model = usage.model.as_deref()?;
  crate::baseline::load( baselines, version, model )
}

/// Project the invoked-skill records.
fn invoked_skills( state : &SessionContextState ) -> Vec< Value >
{
  state.skills_invoked
    .iter()
    .map( | skill | json!( { "name" : skill.name, "path" : skill.path } ) )
    .collect()
}

/// Project the background-task table, keyed by task id.
fn tasks( state : &SessionContextState ) -> Value
{
  let entries : serde_json::Map< String, Value > = state.tasks
    .iter()
    .map( | ( id, task ) | ( id.clone(), json!
    ( {
      "task_type" : task.task_type,
      "status" : task.status,
      "description" : task.description,
      "output_file_path" : task.output_file_path,
    } ) ) )
    .collect();

  Value::Object( entries )
}

/// Project the stream tallies.
///
/// `unmodelled_kinds` and `unmodelled_attachments` are included rather than
/// dropped: a non-empty one means this build's schema is behind the Claude Code
/// version that wrote the session, so the rest of the summary may under-report.
/// A client that never sees them cannot tell a truly empty roster from one this
/// build could not parse.
fn counters( state : &SessionContextState ) -> Value
{
  let counters = &state.counters;

  json!
  ( {
    "lines_read" : counters.lines_read,
    "lines_skipped" : counters.lines_skipped,
    "sidechain_events" : counters.sidechain_events,
    "user_messages" : counters.user_messages,
    "assistant_messages" : counters.assistant_messages,
    "compactions" : counters.compactions,
    "queued_commands" : counters.queued_commands,
    "system_subtypes" : counters.system_subtypes,
    "unmodelled_kinds" : counters.unmodelled_kinds,
    "unmodelled_attachments" : counters.unmodelled_attachments,
    "has_unmodelled" : state.has_unmodelled(),
  } )
}

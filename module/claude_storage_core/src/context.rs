//! Context state folded from a session's event stream.
//!
//! [`SessionEvent`] models one line. This module folds a whole stream of them
//! into the state they describe: which tools are currently deferred, which
//! agents and skills are on offer, how much of the token budget the harness last
//! reported, which background tasks are running.
//!
//! Most of that arrives as *deltas* rather than snapshots — `deferred_tools_delta`
//! says what changed, never what the set now holds — so the current state exists
//! nowhere in the file and has to be accumulated. That is what [`ContextFold`] is.
//!
//! # Token accounting
//!
//! This module reports [`SessionContextState::tokens_remaining`] and nothing
//! else about tokens. That number is the harness's own, injected as prose in a
//! reminder, and it is *not* derivable by summing usage — it accounts for the
//! static system prompt, which never appears in the JSONL at all.
//!
//! Tokens *used* are already computed by [`Session::stats`], which dedups by
//! `message.id` and splits cache reads from cache writes. Recomputing them here
//! would be a second, divergent implementation of the same sum. A caller wanting
//! both asks each for its half.
//!
//! # Tailing a live session
//!
//! [`ContextFold::read_file`] resumes from a byte offset, so a caller can poll a
//! session that is still being written. A trailing line with no newline is a
//! write in progress: it is left unconsumed rather than parsed half-formed, and
//! picked up whole on the next call.
//!
//! [`Session::stats`]: crate::Session::stats

use std::collections::{ BTreeMap, BTreeSet };
use std::fs::File;
use std::io::{ BufRead, BufReader, Seek, SeekFrom };
use std::path::{ Path, PathBuf };

use crate::{ Attachment, Error, EventKind, InvokedSkill, Result, SessionEvent };

/// The context a session has accumulated, as of the last event folded in.
///
/// Every field is the *current* value: sets reflect all deltas applied so far,
/// scalars hold the most recent value seen. A field is empty or `None` when the
/// session never reported it, which is not the same as reporting it empty — a
/// session with no `skill_listing` line has no skills field to report, and one
/// with an empty listing has an empty set.
#[ derive( Debug, Clone, Default, PartialEq, Eq ) ]
#[ non_exhaustive ]
pub struct SessionContextState
{
  /// Conversation id, from the first line that carried one.
  pub session_id : String,

  /// Claude Code version that wrote the most recent line.
  ///
  /// Last-wins rather than first-wins: a session resumed after an upgrade
  /// carries both, and the current one is the one that matters.
  pub version : Option< String >,

  /// Working directory, from the first line that carried one.
  pub cwd : Option< PathBuf >,

  /// Current operating mode, e.g. `normal` or `plan`.
  pub mode : Option< String >,

  /// Current permission mode, e.g. `acceptEdits`.
  pub permission_mode : Option< String >,

  /// Generated display title, if one was assigned.
  pub title : Option< String >,

  /// Uuid of the line holding the most recent prompt.
  pub last_prompt_uuid : Option< String >,

  /// Wall-clock date as of the last `date_change`, `YYYY-MM-DD`.
  pub date : Option< String >,

  /// Tokens the harness last reported remaining.
  ///
  /// Retains the last *parseable* number rather than being cleared by a later
  /// reminder whose wording changed — a stale-but-real budget beats none. See
  /// the module's "Token accounting" note for why used tokens are not here.
  pub tokens_remaining : Option< u64 >,

  /// Tools currently deferred — declared by name, schema not yet loaded.
  pub deferred_tools : BTreeSet< String >,

  /// MCP servers whose tools have been declared but not yet resolved.
  pub pending_mcp_servers : BTreeSet< String >,

  /// Subagent types currently on offer.
  pub agent_types : BTreeSet< String >,

  /// MCP servers whose instruction blocks are currently in context.
  pub mcp_servers : BTreeSet< String >,

  /// Skills currently on offer, in the order the listing gave them.
  pub skills_available : Vec< String >,

  /// Skill count the most recent listing reported for itself.
  ///
  /// Compare against `skills_available.len()`: a disagreement means the listing
  /// was truncated, which `skills_truncated` reports directly.
  pub skills_reported_count : usize,

  /// Skills invoked so far, first invocation of each, in order.
  pub skills_invoked : Vec< InvokedSkill >,

  /// Tools permitted for command execution, as last declared.
  pub allowed_tools : Vec< String >,

  /// Files whose contents were attached to the conversation.
  pub attached_files : BTreeSet< String >,

  /// Background tasks by id, each holding its most recent reported state.
  pub tasks : BTreeMap< String, TaskState >,

  /// Items in the pending-task list, as last restated.
  pub task_reminder_items : usize,

  /// Tallies over the stream, including what this version could not model.
  pub counters : EventCounters,
}

/// A background task's most recently reported state.
#[ derive( Debug, Clone, Default, PartialEq, Eq ) ]
#[ non_exhaustive ]
pub struct TaskState
{
  /// What kind of task it is.
  pub task_type : String,
  /// Its most recent status.
  pub status : String,
  /// Human-readable description, when reported.
  pub description : Option< String >,
  /// Where its output was written, when reported.
  pub output_file_path : Option< String >,
}

/// Tallies over a folded stream.
///
/// The `unmodelled_*` maps are the point of this struct: a newer Claude Code's
/// added line kind is counted under its own name rather than vanishing, so a
/// reader can see that its schema is behind instead of silently under-reporting.
#[ derive( Debug, Clone, Default, PartialEq, Eq ) ]
#[ non_exhaustive ]
pub struct EventCounters
{
  /// Lines read, including ones skipped.
  pub lines_read : u64,
  /// Lines that were not valid session lines and were skipped.
  pub lines_skipped : u64,
  /// Lines belonging to a subagent conversation, excluded from the fold.
  pub sidechain_events : u64,
  /// `user` conversation lines.
  pub user_messages : u64,
  /// `assistant` conversation lines.
  pub assistant_messages : u64,
  /// `compact_boundary` system lines — how often context was compacted.
  pub compactions : u64,
  /// Commands queued while a turn was in flight.
  pub queued_commands : u64,
  /// `system` lines by subtype.
  pub system_subtypes : BTreeMap< String, u64 >,
  /// Envelope `type` values this version does not model, by count.
  pub unmodelled_kinds : BTreeMap< String, u64 >,
  /// Attachment `type` values this version does not model, by count.
  pub unmodelled_attachments : BTreeMap< String, u64 >,
}

/// Accumulates [`SessionContextState`] from a stream of events.
///
/// Fold events one at a time with [`ContextFold::apply`], or read them from a
/// session file with [`ContextFold::read_file`] — which can be called
/// repeatedly against a growing file to follow a live session.
#[ derive( Debug, Clone, Default ) ]
pub struct ContextFold
{
  /// State accumulated so far.
  state : SessionContextState,
  /// Byte offset into the file up to which lines have been consumed.
  offset : u64,
}

impl ContextFold
{
  /// An empty fold.
  #[ must_use ]
  #[ inline ]
  pub fn new() -> Self
  {
    Self::default()
  }

  /// The state accumulated so far.
  #[ must_use ]
  #[ inline ]
  pub const fn state( &self ) -> &SessionContextState
  {
    &self.state
  }

  /// Take the accumulated state, consuming the fold.
  #[ must_use ]
  #[ inline ]
  pub fn into_state( self ) -> SessionContextState
  {
    self.state
  }

  /// Byte offset up to which this fold has consumed its file.
  ///
  /// Only whole lines are counted, so this never points into the middle of one.
  /// Persist it to resume a fold across process restarts.
  #[ must_use ]
  #[ inline ]
  pub const fn offset( &self ) -> u64
  {
    self.offset
  }

  /// Fold one event into the state.
  ///
  /// Sidechain lines are counted but not folded: a subagent's context is its
  /// own, and letting its roster deltas through would corrupt the main
  /// conversation's view of what it has loaded.
  #[ inline ]
  pub fn apply( &mut self, event : &SessionEvent )
  {
    if event.is_sidechain
    {
      self.state.counters.sidechain_events += 1;
      return;
    }

    self.apply_envelope( event );

    match &event.kind
    {
      EventKind::User => self.state.counters.user_messages += 1,
      EventKind::Assistant => self.state.counters.assistant_messages += 1,
      EventKind::System { subtype, .. } =>
      {
        if subtype == "compact_boundary"
        {
          self.state.counters.compactions += 1;
        }
        bump( &mut self.state.counters.system_subtypes, subtype );
      },
      EventKind::Attachment( attachment ) => self.apply_attachment( attachment ),
      EventKind::Mode { mode } => self.state.mode = Some( mode.clone() ),
      EventKind::PermissionMode { permission_mode } =>
      {
        self.state.permission_mode = Some( permission_mode.clone() );
      },
      EventKind::LastPrompt { leaf_uuid } => self.state.last_prompt_uuid = Some( leaf_uuid.clone() ),
      EventKind::AiTitle { title } => self.state.title = Some( title.clone() ),
      EventKind::QueueOperation { .. } => self.state.counters.queued_commands += 1,
      EventKind::Other { kind } => bump( &mut self.state.counters.unmodelled_kinds, kind ),
      _ => bump( &mut self.state.counters.unmodelled_kinds, "" ),
    }
  }

  /// Absorb the envelope metadata that is not specific to a line kind.
  fn apply_envelope( &mut self, event : &SessionEvent )
  {
    if self.state.session_id.is_empty() && !event.session_id.is_empty()
    {
      self.state.session_id.clone_from( &event.session_id );
    }

    if let Some( version ) = &event.version
    {
      self.state.version = Some( version.clone() );
    }

    if self.state.cwd.is_none()
    {
      self.state.cwd.clone_from( &event.cwd );
    }
  }

  /// Absorb one attachment's payload.
  fn apply_attachment( &mut self, attachment : &Attachment )
  {
    let state = &mut self.state;

    match attachment
    {
      Attachment::TotalTokensReminder { remaining } =>
      {
        // Only overwrite on a parseable number — see the field's own note.
        if remaining.is_some()
        {
          state.tokens_remaining = *remaining;
        }
      },
      Attachment::DeferredToolsDelta { added, removed, readded, pending_mcp_servers } =>
      {
        // Removals first: an addition in the same delta is the newer fact.
        for name in removed
        {
          state.deferred_tools.remove( name );
        }
        state.deferred_tools.extend( added.iter().cloned() );
        state.deferred_tools.extend( readded.iter().cloned() );
        state.pending_mcp_servers = pending_mcp_servers.iter().cloned().collect();
      },
      Attachment::AgentListingDelta { added, removed, is_initial } =>
      {
        if *is_initial
        {
          state.agent_types.clear();
        }
        for name in removed
        {
          state.agent_types.remove( name );
        }
        state.agent_types.extend( added.iter().cloned() );
      },
      Attachment::McpInstructionsDelta { added, removed } =>
      {
        for name in removed
        {
          state.mcp_servers.remove( name );
        }
        state.mcp_servers.extend( added.iter().cloned() );
      },
      Attachment::SkillListing { names, skill_count, .. } =>
      {
        // A listing replaces rather than merges: it is a full snapshot.
        state.skills_available.clone_from( names );
        state.skills_reported_count = *skill_count;
      },
      Attachment::InvokedSkills { skills } =>
      {
        for skill in skills
        {
          // First invocation wins — the harness injects a skill's text once,
          // so a repeat invocation adds nothing to what is in context.
          if !state.skills_invoked.iter().any( | seen | seen.name == skill.name )
          {
            state.skills_invoked.push( skill.clone() );
          }
        }
      },
      Attachment::TaskReminder { item_count } => state.task_reminder_items = *item_count,
      Attachment::TaskStatus { task_id, task_type, status, description, output_file_path } =>
      {
        state.tasks.insert
        (
          task_id.clone(),
          TaskState
          {
            task_type : task_type.clone(),
            status : status.clone(),
            description : description.clone(),
            output_file_path : output_file_path.clone(),
          },
        );
      },
      Attachment::CommandPermissions { allowed_tools } =>
      {
        state.allowed_tools.clone_from( allowed_tools );
      },
      Attachment::QueuedCommand { .. } => state.counters.queued_commands += 1,
      Attachment::EditedTextFile { filename }
      | Attachment::CompactFileReference { filename, .. }
      | Attachment::File { filename, .. } =>
      {
        state.attached_files.insert( filename.clone() );
      },
      Attachment::DateChange { new_date } => state.date = Some( new_date.clone() ),
      Attachment::Other { kind } => bump( &mut state.counters.unmodelled_attachments, kind ),
      _ => bump( &mut state.counters.unmodelled_attachments, "" ),
    }
  }

  /// Read every whole line added since the last call and fold it in.
  ///
  /// Returns how many events were applied. Call it again on the same fold to
  /// pick up whatever has been appended since — the offset advances only over
  /// lines actually consumed.
  ///
  /// Three cases are handled rather than reported as failures, matching the
  /// per-line skip policy [`Session::stats`] already uses:
  ///
  /// - **A trailing line with no newline** is a write in progress. It is left
  ///   unconsumed and re-read whole next time.
  /// - **A line that is not valid UTF-8, or not a session line**, is skipped
  ///   and counted in [`EventCounters::lines_skipped`]. One bad line must not
  ///   discard the rest of the file.
  /// - **A file shorter than the offset** was replaced rather than appended to,
  ///   so the fold restarts from the beginning with fresh state.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Io`] if the file cannot be opened, measured, or sought.
  ///
  /// [`Session::stats`]: crate::Session::stats
  #[ inline ]
  pub fn read_file( &mut self, path : &Path ) -> Result< usize >
  {
    let file = File::open( path ).map_err( Error::Io )?;
    let length = file.metadata().map_err( Error::Io )?.len();

    if length < self.offset
    {
      // The file was truncated or replaced; anything folded from it is stale.
      *self = Self::new();
    }

    let mut reader = BufReader::new( file );
    reader.seek( SeekFrom::Start( self.offset ) ).map_err( Error::Io )?;

    let mut applied = 0;
    let mut buffer = Vec::new();

    loop
    {
      buffer.clear();

      let read = reader.read_until( b'\n', &mut buffer ).map_err( Error::Io )?;

      if read == 0 || !buffer.ends_with( b"\n" )
      {
        // EOF, or a partial line still being written — stop without consuming.
        break;
      }

      self.offset += read as u64;
      self.state.counters.lines_read += 1;

      let Ok( line ) = core::str::from_utf8( &buffer ) else
      {
        self.state.counters.lines_skipped += 1;
        continue;
      };

      let trimmed = line.trim();

      if trimmed.is_empty()
      {
        continue;
      }

      match SessionEvent::from_json_line( trimmed )
      {
        Ok( event ) =>
        {
          self.apply( &event );
          applied += 1;
        },
        Err( _ ) => self.state.counters.lines_skipped += 1,
      }
    }

    Ok( applied )
  }
}

/// Increment `map`'s entry for `key`, inserting it at 1 if absent.
fn bump( map : &mut BTreeMap< String, u64 >, key : &str )
{
  if let Some( count ) = map.get_mut( key )
  {
    *count += 1;
  }
  else
  {
    map.insert( key.to_string(), 1 );
  }
}

impl SessionContextState
{
  /// Whether the most recent skill listing reported more skills than it named.
  ///
  /// True means the listing was truncated, so `skills_available` is a subset of
  /// what the session actually has on offer.
  #[ must_use ]
  #[ inline ]
  pub fn skills_truncated( &self ) -> bool
  {
    self.skills_reported_count > self.skills_available.len()
  }

  /// Whether the fold encountered any line or attachment kind it could not model.
  ///
  /// True means this build's schema is behind the Claude Code version that
  /// wrote the session, and the state may under-report.
  #[ must_use ]
  #[ inline ]
  pub fn has_unmodelled( &self ) -> bool
  {
    !self.counters.unmodelled_kinds.is_empty() || !self.counters.unmodelled_attachments.is_empty()
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn bump_counts_repeats()
  {
    let mut map = BTreeMap::new();
    bump( &mut map, "a" );
    bump( &mut map, "a" );
    bump( &mut map, "b" );

    assert_eq!( map.get( "a" ), Some( &2 ) );
    assert_eq!( map.get( "b" ), Some( &1 ) );
  }

  #[ test ]
  fn skills_truncated_only_when_count_exceeds_names()
  {
    let mut state = SessionContextState::default();
    state.skills_available = vec![ "a".to_string() ];
    state.skills_reported_count = 1;
    assert!( !state.skills_truncated() );

    state.skills_reported_count = 9;
    assert!( state.skills_truncated() );
  }
}

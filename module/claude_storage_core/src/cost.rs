//! Per-conversation cost accounting engine — per-model token usage with
//! cache-TTL split, compaction count, and max context, plus family
//! aggregation.
//!
//! Pure scan/aggregation layer: [`cost_report`] reads one session file into
//! a [`SessionCostReport`]; [`aggregate_reports`] folds a root session's
//! report together with its agent children's into one
//! [`ConversationUsage`]. No pricing lives here — token→USD conversion is a
//! CLI display concern (prices change; token counts don't), see
//! `claude_storage/src/cli/cost.rs`. Powers the `claude_storage` CLI's
//! `.cost` command; see `claude_storage/docs/cli/command/15_cost.md` for the
//! full CLI contract this engine is built to serve.
//!
//! Why not reuse `Session::stats()`: that scan has no per-model attribution
//! (a session that switches models mid-way — common with `/model` — must
//! price each call at its own model's rate, not the first-seen model's), no
//! cache-TTL split (5-minute and 1-hour cache writes bill at different
//! multipliers), and no compaction count. Extending `SessionStats` with all
//! of that would bloat every existing `.usage`/`.rollup`/`.status` caller
//! for fields only `.cost` needs.

use std::collections::HashSet;
use std::io::{ BufRead, BufReader };
use std::fs;

use crate::{ Error, Result, Session, json::parse_json };

/// Placeholder model name Claude Code writes on synthetic assistant entries
/// (e.g. locally-generated error notices). Such entries are not API calls:
/// their usage is all-zero and their `message.id` is uuid-form rather than
/// `msg_*`. They are skipped entirely so `"<synthetic>"` never surfaces as a
/// model bucket.
const SYNTHETIC_MODEL : &str = "<synthetic>";

/// Token usage attributed to one model within one session or conversation.
///
/// Cache writes are split by TTL because they bill at different multipliers
/// of the input rate (5m = 1.25x, 1h = 2x). Tokens whose TTL the transcript
/// does not break down land in `cache_unknown_ttl_write_tokens` instead of
/// being guessed into a TTL bucket.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub struct ModelUsage
{
  /// Model identifier as recorded on the assistant entry
  /// (e.g. `claude-sonnet-5`); `"unknown"` when the entry carries none.
  pub model : String,
  /// Deduplicated API calls (distinct `message.id` values) for this model.
  pub calls : usize,
  /// Fresh (non-cached) input tokens.
  pub input_tokens : u64,
  /// Generated output tokens.
  pub output_tokens : u64,
  /// Tokens read from prompt cache.
  pub cache_read_tokens : u64,
  /// Cache-write tokens with a 5-minute TTL
  /// (`usage.cache_creation.ephemeral_5m_input_tokens`).
  pub cache_5m_write_tokens : u64,
  /// Cache-write tokens with a 1-hour TTL
  /// (`usage.cache_creation.ephemeral_1h_input_tokens`).
  pub cache_1h_write_tokens : u64,
  /// Cache-write tokens `cache_creation_input_tokens` reports but the
  /// `cache_creation` TTL breakdown does not account for — the whole write
  /// total when the breakdown object is absent (older transcript format),
  /// or the remainder when the buckets sum below the total.
  pub cache_unknown_ttl_write_tokens : u64,
}

impl ModelUsage
{
  fn new( model : String ) -> Self
  {
    Self
    {
      model,
      calls : 0,
      input_tokens : 0,
      output_tokens : 0,
      cache_read_tokens : 0,
      cache_5m_write_tokens : 0,
      cache_1h_write_tokens : 0,
      cache_unknown_ttl_write_tokens : 0,
    }
  }

  /// All cache-write tokens across the three TTL buckets — the counterpart
  /// of `cache_read_tokens`, and the number matching the transcript's own
  /// `cache_creation_input_tokens` total.
  #[ must_use ]
  #[ inline ]
  pub fn cache_write_tokens( &self ) -> u64
  {
    self.cache_5m_write_tokens + self.cache_1h_write_tokens + self.cache_unknown_ttl_write_tokens
  }

  /// `input + output + cache_read + cache_write` — every token the model
  /// touched.
  #[ must_use ]
  #[ inline ]
  pub fn total_tokens( &self ) -> u64
  {
    self.input_tokens + self.output_tokens + self.cache_read_tokens + self.cache_write_tokens()
  }

  /// Fold `other`'s counts into `self` (same-model merge during
  /// [`aggregate_reports`]).
  fn absorb( &mut self, other : &ModelUsage )
  {
    self.calls += other.calls;
    self.input_tokens += other.input_tokens;
    self.output_tokens += other.output_tokens;
    self.cache_read_tokens += other.cache_read_tokens;
    self.cache_5m_write_tokens += other.cache_5m_write_tokens;
    self.cache_1h_write_tokens += other.cache_1h_write_tokens;
    self.cache_unknown_ttl_write_tokens += other.cache_unknown_ttl_write_tokens;
  }
}

/// Cost-relevant usage of one session file, attributed per model.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub struct SessionCostReport
{
  /// Session ID (filename stem).
  pub session_id : String,
  /// Whether the scanned file is an agent (`agent-*`) session.
  pub is_agent_session : bool,
  /// Per-model usage, ordered by each model's first appearance in the file.
  pub models : Vec< ModelUsage >,
  /// Number of context compactions
  /// (`"type":"system","subtype":"compact_boundary"` entries).
  pub compactions : usize,
  /// Largest single API call's context size — `input + cache_read +
  /// cache_creation` for one deduplicated assistant message; `0` when the
  /// session has no assistant messages.
  pub max_context_tokens : u64,
}

impl SessionCostReport
{
  /// Deduplicated API calls across all models.
  #[ must_use ]
  #[ inline ]
  pub fn total_calls( &self ) -> usize
  {
    self.models.iter().map( | m | m.calls ).sum()
  }
}

/// One conversation's aggregated usage: a root session plus every agent
/// session folded in (see [`aggregate_reports`]).
#[ derive( Debug, Clone, PartialEq, Eq ) ]
pub struct ConversationUsage
{
  /// Root session ID the conversation is keyed by.
  pub root_id : String,
  /// Number of agent session files folded into these totals.
  pub agent_count : usize,
  /// Per-model usage merged across all contributing reports, ordered by
  /// first appearance (report order, then in-file order).
  pub models : Vec< ModelUsage >,
  /// Compactions summed across all contributing reports.
  pub compactions : usize,
  /// Largest single API call's context size across all contributing
  /// reports.
  pub max_context_tokens : u64,
}

impl ConversationUsage
{
  /// Deduplicated API calls across all models and contributing sessions.
  #[ must_use ]
  #[ inline ]
  pub fn total_calls( &self ) -> usize
  {
    self.models.iter().map( | m | m.calls ).sum()
  }

  /// Fresh input tokens across all models.
  #[ must_use ]
  #[ inline ]
  pub fn total_input_tokens( &self ) -> u64
  {
    self.models.iter().map( | m | m.input_tokens ).sum()
  }

  /// Output tokens across all models.
  #[ must_use ]
  #[ inline ]
  pub fn total_output_tokens( &self ) -> u64
  {
    self.models.iter().map( | m | m.output_tokens ).sum()
  }

  /// Cache-read tokens across all models.
  #[ must_use ]
  #[ inline ]
  pub fn total_cache_read_tokens( &self ) -> u64
  {
    self.models.iter().map( | m | m.cache_read_tokens ).sum()
  }

  /// Cache-write tokens (all TTL buckets) across all models.
  #[ must_use ]
  #[ inline ]
  pub fn total_cache_write_tokens( &self ) -> u64
  {
    self.models.iter().map( ModelUsage::cache_write_tokens ).sum()
  }

  /// Every token across all models — `input + output + cache_read +
  /// cache_write`.
  #[ must_use ]
  #[ inline ]
  pub fn total_tokens( &self ) -> u64
  {
    self.models.iter().map( ModelUsage::total_tokens ).sum()
  }
}

/// Scan one session file into a [`SessionCostReport`].
///
/// Single-pass line scan mirroring `Session::stats()`'s conventions exactly
/// where they overlap — per-line graceful degradation (unreadable, empty,
/// and unparseable lines are skipped: `Fix(BUG-489)`/`Fix(BUG-508)`),
/// `message.id` dedup so one multi-content-block API response counts once
/// (`Fix(issue-038)`), top-level `"type"` dispatch — and adding what
/// `stats()` lacks: per-model attribution, cache-TTL split, compaction
/// count, `<synthetic>` skip.
///
/// # Errors
///
/// Returns `Error::Io` when the session file cannot be opened.
#[ inline ]
pub fn cost_report( session : &Session ) -> Result< SessionCostReport >
{
  let mut report = SessionCostReport
  {
    session_id : session.id().to_string(),
    is_agent_session : session.is_agent_session(),
    models : Vec::new(),
    compactions : 0,
    max_context_tokens : 0,
  };

  let file = fs::File::open( session.storage_path() )
    .map_err( | e | Error::io
    (
      e,
      format!( "reading session file: {}", session.storage_path().display() )
    ))?;
  let reader = BufReader::new( file );

  let mut seen_message_ids : HashSet< String > = HashSet::new();

  for line in reader.lines()
  {
    let Ok( line ) = line else { continue; };
    if line.trim().is_empty() { continue; }
    let Ok( json ) = parse_json( &line ) else { continue; };
    let Some( entry_type ) = json.get_str( "type" ) else { continue; };

    // Compaction marker: a `system` entry with subtype `compact_boundary`.
    // Parse-based check on the top-level fields — a transcript line merely
    // *mentioning* the marker inside message content never matches, because
    // there the text is an escaped string value, not top-level fields.
    if entry_type == "system"
    {
      if json.get_str( "subtype" ) == Some( "compact_boundary" )
      {
        report.compactions += 1;
      }
      continue;
    }

    if entry_type != "assistant" { continue; }
    let Some( message ) = json.get( "message" ) else { continue; };

    // Fix(issue-038) convention: dedup by message.id — one API response
    // spans one JSONL line per content block, each repeating the same
    // `message.id`/`message.usage`. A line with no id is always new.
    let msg_id = message.get_str( "id" );
    let is_new_message = msg_id.map_or( true, | id | seen_message_ids.insert( id.to_string() ) );
    if !is_new_message { continue; }

    let model = message.get_str( "model" ).unwrap_or( "unknown" );
    if model == SYNTHETIC_MODEL { continue; }

    let usage = bucket_for( &mut report.models, model );
    usage.calls += 1;

    let Some( usage_json ) = message.get( "usage" ) else { continue; };

    // Token counts from JSON are always non-negative integers stored as
    // f64. The cast to u64 is safe: values are positive and well within
    // u64 range. (Same rationale as `Session::stats()`.)
    #[ allow( clippy::cast_possible_truncation, clippy::cast_sign_loss ) ]
    {
      let input = usage_json.get_number( "input_tokens" ).unwrap_or( 0.0 ) as u64;
      let output = usage_json.get_number( "output_tokens" ).unwrap_or( 0.0 ) as u64;
      let cache_read = usage_json.get_number( "cache_read_input_tokens" ).unwrap_or( 0.0 ) as u64;
      let cache_write_total = usage_json.get_number( "cache_creation_input_tokens" ).unwrap_or( 0.0 ) as u64;

      // TTL breakdown: `usage.cache_creation.{ephemeral_5m,ephemeral_1h}_input_tokens`.
      // Anything the breakdown doesn't account for (absent object — older
      // transcript format — or buckets summing below the total) goes to the
      // unknown-TTL bucket rather than being guessed into 5m or 1h.
      let ( c5m, c1h ) = match usage_json.get( "cache_creation" )
      {
        Some( breakdown ) =>
        (
          breakdown.get_number( "ephemeral_5m_input_tokens" ).unwrap_or( 0.0 ) as u64,
          breakdown.get_number( "ephemeral_1h_input_tokens" ).unwrap_or( 0.0 ) as u64,
        ),
        None => ( 0, 0 ),
      };
      let accounted = c5m.saturating_add( c1h ).min( cache_write_total );
      let unknown = cache_write_total - accounted;

      usage.input_tokens += input;
      usage.output_tokens += output;
      usage.cache_read_tokens += cache_read;
      usage.cache_5m_write_tokens += c5m.min( cache_write_total );
      usage.cache_1h_write_tokens += accounted.saturating_sub( c5m.min( cache_write_total ) );
      usage.cache_unknown_ttl_write_tokens += unknown;

      // Context size for one call is the input side only — fresh input plus
      // everything read from or written to cache — never the output side.
      let call_context = input + cache_read + cache_write_total;
      if call_context > report.max_context_tokens
      {
        report.max_context_tokens = call_context;
      }
    }
  }

  Ok( report )
}

/// Fold `reports` — a root session's report plus its agent children's —
/// into one [`ConversationUsage`] keyed by `root_id`.
///
/// Model buckets merge by name, ordered by first appearance across
/// `reports` in the order given; `compactions` sums; `max_context_tokens`
/// takes the largest single value (context windows are per-call, never
/// additive across sessions). `agent_count` counts the reports flagged
/// `is_agent_session`, not `reports.len() - 1` — callers may pass the root
/// alone.
#[ must_use ]
#[ inline ]
pub fn aggregate_reports( root_id : &str, reports : &[ SessionCostReport ] ) -> ConversationUsage
{
  let mut usage = ConversationUsage
  {
    root_id : root_id.to_string(),
    agent_count : reports.iter().filter( | r | r.is_agent_session ).count(),
    models : Vec::new(),
    compactions : 0,
    max_context_tokens : 0,
  };

  for report in reports
  {
    usage.compactions += report.compactions;
    if report.max_context_tokens > usage.max_context_tokens
    {
      usage.max_context_tokens = report.max_context_tokens;
    }
    for model in &report.models
    {
      bucket_for( &mut usage.models, &model.model ).absorb( model );
    }
  }

  usage
}

/// The `ModelUsage` bucket for `model` inside `models`, appending a fresh
/// one on first appearance (which is what keeps the Vec ordered by first
/// appearance). Linear scan — a session touches a handful of models at
/// most.
fn bucket_for< 'a >( models : &'a mut Vec< ModelUsage >, model : &str ) -> &'a mut ModelUsage
{
  if let Some( idx ) = models.iter().position( | m | m.model == model )
  {
    return &mut models[ idx ];
  }
  models.push( ModelUsage::new( model.to_string() ) );
  models.last_mut().expect( "just pushed" )
}

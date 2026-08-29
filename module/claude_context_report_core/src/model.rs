//! The report model — typed rows and closed-set vocabularies.
//!
//! Every cell is a typed value or an enum; no field holds a pre-formatted
//! string. That is what makes "print exact tables" enforceable: two renderers
//! consuming this model cannot disagree about content, only about styling.
//!
//! The vocabularies here are the ones fixed in
//! `docs/format/001_context_report_tables.md`. Each enum's `as_str` is the
//! rendered token, so a renderer never invents its own spelling.

use std::collections::BTreeMap;

/// Origin channel a context block arrived on.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash ) ]
#[ non_exhaustive ]
pub enum Src
{
  /// Harness-authored system prompt.
  Sys,
  /// Injected configuration — instruction files, rulebooks.
  Cfg,
  /// A user turn.
  Usr,
  /// An assistant turn.
  Ast,
  /// A tool-result or tool-catalog injection.
  Fn,
  /// A system reminder.
  Rem,
}

impl Src
{
  /// The rendered token for this channel.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Sys => "sys",
      Self::Cfg => "cfg",
      Self::Usr => "usr",
      Self::Ast => "ast",
      Self::Fn  => "fn",
      Self::Rem => "rem",
    }
  }

  /// Which layer this channel rolls up into.
  ///
  /// The mapping is total, which is what guarantees the layer table partitions
  /// the block table exactly — every block has a `Src`, every `Src` has a layer.
  #[ inline ]
  #[ must_use ]
  pub fn layer( self ) -> Layer
  {
    match self
    {
      Self::Sys => Layer::Harness,
      Self::Cfg => Layer::Config,
      Self::Usr | Self::Ast => Layer::Conversation,
      Self::Fn | Self::Rem => Layer::Injected,
    }
  }
}

/// How a context block constrains behaviour.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash ) ]
#[ non_exhaustive ]
pub enum Force
{
  /// Constrains behaviour; must be obeyed.
  Rule,
  /// Factual state; no behavioural demand.
  Data,
  /// Advisory; may be disregarded.
  Info,
  /// An inventory of available capabilities.
  Catalog,
  /// A rule that activates only in a named situation.
  Conditional,
  /// Retrieved material supporting a claim.
  Evidence,
  /// A prior turn, retained for continuity.
  History,
  /// Loaded but explicitly inert this turn.
  Dormant,
  /// An invocable interface.
  Callable,
  /// The turn currently being answered.
  Live,
}

impl Force
{
  /// The rendered token for this force.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Rule        => "rule",
      Self::Data        => "data",
      Self::Info        => "info",
      Self::Catalog     => "catalog",
      Self::Conditional => "conditional",
      Self::Evidence    => "evidence",
      Self::History     => "history",
      Self::Dormant     => "dormant",
      Self::Callable    => "callable",
      Self::Live        => "live",
    }
  }
}

/// Aggregate grouping for the layer table.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash ) ]
#[ non_exhaustive ]
pub enum Layer
{
  /// Harness-authored blocks.
  Harness,
  /// Injected configuration.
  Config,
  /// Conversation turns.
  Conversation,
  /// Harness-injected context and reminders.
  Injected,
}

impl Layer
{
  /// Every layer, in render order.
  ///
  /// Fixed rather than derived so the layer table's row order does not depend on
  /// which layers a particular session happens to contain.
  pub const ALL : [ Self; 4 ] = [ Self::Harness, Self::Config, Self::Conversation, Self::Injected ];

  /// The rendered name for this layer.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Harness      => "Harness",
      Self::Config       => "Config",
      Self::Conversation => "Conversation",
      Self::Injected     => "Injected",
    }
  }

  /// Whether blocks in this layer can still change within the session.
  #[ inline ]
  #[ must_use ]
  pub fn mutable( self ) -> bool
  {
    match self
    {
      Self::Harness | Self::Config => false,
      Self::Conversation | Self::Injected => true,
    }
  }

  /// Why this layer is or is not mutable.
  #[ inline ]
  #[ must_use ]
  pub fn mutable_reason( self ) -> &'static str
  {
    match self
    {
      Self::Harness      => "fixed at session start",
      Self::Config       => "loaded once per session",
      Self::Conversation => "grows every turn",
      Self::Injected     => "re-injected as state changes",
    }
  }
}

/// Whether a path names a file or a directory.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash ) ]
#[ non_exhaustive ]
pub enum PathKind
{
  /// A regular file.
  File,
  /// A directory.
  Dir,
}

impl PathKind
{
  /// The rendered token for this kind.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::File => "file",
      Self::Dir  => "dir",
    }
  }
}

/// Relationship between a path and the context that names it.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash ) ]
#[ non_exhaustive ]
pub enum PathState
{
  /// The file's content is present in context.
  Loaded,
  /// Content was loaded earlier this session and has since been dropped.
  Evicted,
  /// The path appears in context; content was never loaded.
  Named,
  /// The path appears in context but does not exist on disk.
  Absent,
}

impl PathState
{
  /// The rendered glyph for this state.
  #[ inline ]
  #[ must_use ]
  pub fn glyph( self ) -> &'static str
  {
    match self
    {
      Self::Loaded  => "🟢",
      Self::Evicted => "🟡",
      Self::Named   => "⚪",
      Self::Absent  => "❌",
    }
  }

  /// The rendered token for this state.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Loaded  => "loaded",
      Self::Evicted => "evicted",
      Self::Named   => "named",
      Self::Absent  => "absent",
    }
  }
}

/// Relative size on the five-position weight scale.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash ) ]
pub struct Weight( u8 );

impl Weight
{
  /// Build a weight from a 1..=5 position, clamping out-of-range input.
  #[ inline ]
  #[ must_use ]
  pub fn new( position : u8 ) -> Self
  {
    Self( position.clamp( 1, 5 ) )
  }

  /// The position, 1..=5.
  #[ inline ]
  #[ must_use ]
  pub fn position( self ) -> u8
  {
    self.0
  }

  /// The filled-then-empty glyph run, e.g. `●●●○○`.
  #[ inline ]
  #[ must_use ]
  pub fn glyphs( self ) -> String
  {
    let filled = usize::from( self.0 );
    let mut out = String::with_capacity( 5 * 3 );
    for _ in 0 .. filled { out.push( '●' ); }
    for _ in filled .. 5 { out.push( '○' ); }
    out
  }
}

/// Byte thresholds separating the five weight positions.
///
/// Held explicitly and echoed into the legend so two reports rendered with
/// different bands are never silently compared.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub struct Bands
{
  /// Upper bound of position 1, then 2, 3, 4. Anything above the last is 5.
  thresholds : [ u64; 4 ],
}

impl Bands
{
  /// Build bands from four ascending byte thresholds.
  ///
  /// A non-ascending input is sorted rather than rejected: bands are a display
  /// parameter, and a mis-ordered set should degrade to a usable scale instead
  /// of failing a report that is otherwise correct.
  #[ inline ]
  #[ must_use ]
  pub fn new( mut thresholds : [ u64; 4 ] ) -> Self
  {
    thresholds.sort_unstable();
    Self { thresholds }
  }

  /// The thresholds, ascending.
  #[ inline ]
  #[ must_use ]
  pub fn thresholds( &self ) -> [ u64; 4 ]
  {
    self.thresholds
  }

  /// Which weight position `bytes` falls into.
  #[ inline ]
  #[ must_use ]
  pub fn weigh( &self, bytes : u64 ) -> Weight
  {
    let mut position = 5_u8;
    for ( index, bound ) in self.thresholds.iter().enumerate()
    {
      if bytes <= *bound
      {
        // SAFETY (arithmetic): index < 4, so index + 1 <= 4 and the cast is exact.
        position = u8::try_from( index + 1 ).unwrap_or( 5 );
        break;
      }
    }
    Weight::new( position )
  }
}

impl Default for Bands
{
  /// Bands tuned to observed session-line sizes: most envelope lines are a few
  /// hundred bytes, catalogue injections a few kilobytes, and attached file
  /// contents an order of magnitude above that.
  #[ inline ]
  fn default() -> Self
  {
    Self::new( [ 256, 1_024, 4_096, 16_384 ] )
  }
}

/// One row of the block table.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
#[ non_exhaustive ]
pub struct BlockRow
{
  /// 1-based position in wire order.
  pub position : usize,
  /// Block label.
  pub label : String,
  /// Origin channel.
  pub src : Src,
  /// One-line summary of what the block holds — never the content itself.
  pub carries : String,
  /// Exact byte length of the source line.
  pub bytes : u64,
  /// Relative size, derived from `bytes` under the report's bands.
  pub weight : Weight,
  /// How the block constrains behaviour.
  pub force : Force,
}

/// One row of the path table.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
#[ non_exhaustive ]
pub struct PathRow
{
  /// Block-table row that named this path. Always resolves to a real row.
  pub row : usize,
  /// Whether the path names a file or a directory.
  pub kind : PathKind,
  /// The path, absolute and unabbreviated, subject to redaction.
  pub path : String,
  /// Relationship between the path and the context.
  pub state : PathState,
}

/// One row of the layer table.
#[ derive( Debug, Clone, PartialEq ) ]
#[ non_exhaustive ]
pub struct LayerRow
{
  /// Which layer.
  pub layer : Layer,
  /// Block-table rows belonging to this layer, ascending.
  pub rows : Vec< usize >,
  /// Total bytes across those rows.
  pub bytes : u64,
  /// Relative size of the layer as a whole.
  pub weight : Weight,
  /// Share of total report bytes, 0.0 ..= 100.0.
  pub share_percent : f64,
}

impl LayerRow
{
  /// `rows` compressed into ascending inclusive ranges.
  ///
  /// Layer membership is interleaved across the block table, so a layer's rows
  /// are rarely one contiguous span; rendering them as ranges keeps the cell
  /// readable without losing which rows are actually covered.
  #[ inline ]
  #[ must_use ]
  pub fn ranges( &self ) -> Vec< ( usize, usize ) >
  {
    let mut out : Vec< ( usize, usize ) > = Vec::new();
    for &row in &self.rows
    {
      match out.last_mut()
      {
        Some( last ) if last.1 + 1 == row => last.1 = row,
        _ => out.push( ( row, row ) ),
      }
    }
    out
  }
}

/// One row of the conditional corrections table.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
#[ non_exhaustive ]
pub struct Correction
{
  /// 1-based index within this report.
  pub index : usize,
  /// What the context asserts.
  pub claim : String,
  /// Block-table row making the claim.
  pub row : usize,
  /// What was observed.
  pub reality : String,
  /// What breaks if the claim is trusted.
  pub impact : String,
}

/// How much of a path a rendered report may disclose.
///
/// The levels differ **only** in path treatment. Credentials, account identity,
/// host identity, and message content stay redacted at every level, including
/// [`RedactionLevel::Off`] — see `docs/invariant/001_no_private_data.md`.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Default ) ]
#[ non_exhaustive ]
pub enum RedactionLevel
{
  /// Every path replaced by its placeholder token.
  #[ default ]
  Strict,
  /// Paths below the repository root shown relative to it; everything above tokenised.
  Paths,
  /// Absolute paths as-is.
  Off,
}

impl RedactionLevel
{
  /// The rendered token for this level.
  #[ inline ]
  #[ must_use ]
  pub fn as_str( self ) -> &'static str
  {
    match self
    {
      Self::Strict => "strict",
      Self::Paths  => "paths",
      Self::Off    => "off",
    }
  }

  /// Parse a level from its rendered token.
  #[ inline ]
  #[ must_use ]
  pub fn parse( text : &str ) -> Option< Self >
  {
    match text
    {
      "strict" => Some( Self::Strict ),
      "paths"  => Some( Self::Paths ),
      "off"    => Some( Self::Off ),
      _        => None,
    }
  }
}

/// The settings a report was rendered under.
///
/// Emitted before the first table. A report without it is not conforming: the
/// glyph columns are unreadable without the bands, and a reader cannot tell an
/// estimate from a measurement.
#[ derive( Debug, Clone, PartialEq, Eq ) ]
#[ non_exhaustive ]
pub struct Legend
{
  /// Byte thresholds the weight glyphs were derived from.
  pub bands : Bands,
  /// Redaction level applied.
  pub redaction : RedactionLevel,
  /// What the weight scale measures.
  pub unit : &'static str,
}

/// A session's context, as an ordered inventory.
#[ derive( Debug, Clone, PartialEq ) ]
#[ non_exhaustive ]
pub struct ContextReport
{
  /// Conversation id this report describes.
  pub session_id : String,
  /// Claude Code version that wrote the most recent line.
  pub version : Option< String >,
  /// Settings this report was rendered under.
  pub legend : Legend,
  /// Block table, in wire order.
  pub blocks : Vec< BlockRow >,
  /// Path table, sorted by owning row then path.
  pub paths : Vec< PathRow >,
  /// Layer table, in fixed layer order.
  pub layers : Vec< LayerRow >,
  /// Corrections table — empty when nothing was contradicted.
  pub corrections : Vec< Correction >,
  /// Envelope and attachment kinds this version does not model, by count.
  ///
  /// Present so a schema that has fallen behind says so, rather than silently
  /// under-reporting. Mirrors the degradation policy `claude_storage_core`
  /// already applies to unmodelled line kinds.
  pub unmodelled : BTreeMap< String, u64 >,
  /// Total bytes across every block row.
  pub total_bytes : u64,
  /// Lines read from the transcript, including skipped ones.
  pub lines_read : u64,
  /// Tokens the harness last reported remaining, when it reported any.
  pub tokens_remaining : Option< u64 >,
}

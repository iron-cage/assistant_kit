//! Shared output logic for the `clj` journal viewer.
//!
//! Each command handler returns a `String` (or `Result<String, String>` for
//! commands that validate input) so that the same logic can be used from both
//! the `clj` binary (`cli_main.rs`) and the unilang assistant routines
//! (`routines.rs`).

use claude_journal::rotation::{ prune_by_age, today_ymd, PruneAction };
use claude_journal::{ EventRecord, EventType, JournalFileInfo, JournalFilter, JournalReader };
use core::{ fmt::Write as _, time::Duration };
use std::{ collections::HashMap, path::PathBuf, time::SystemTime };

// ── Directory resolution ──────────────────────────────────────────────────────

/// Resolve the journal directory from `journal_dir::` param, `CLR_JOURNAL_DIR`
/// env, or the default `~/.clr/journal/` — falling back to `/tmp/.clr/journal`
/// when `HOME` is unset or empty.
///
/// Fix(param-dir-collision): read `journal_dir`, not `dir`
/// Root cause: this consumed `dir::`, the key `docs/cli/param/07_dir.md` assigns
///   to the event working-directory *filter* and that `JournalFilter::dir`
///   already reserves. One name carried two meanings, so `clj .list
///   dir::/home/u/alpha` silently repointed the reader at a nonexistent journal
///   and printed "No events found." — indistinguishable from a filter that
///   matched nothing. Meanwhile the documented `journal_dir::` was read by
///   nothing at all.
/// Pitfall: a param name that collides with a *data field* name is worse than an
///   ugly one — the wrong reading still produces plausible output. When the
///   library already reserves a name (`JournalFilter::dir`), the CLI must not
///   spend it on something else.
#[ must_use ]
#[ inline ]
pub fn resolve_journal_dir< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S > ) -> PathBuf
{
  if let Some( d ) = params.get( "journal_dir" )
  {
    return PathBuf::from( d );
  }
  if let Ok( d ) = std::env::var( "CLR_JOURNAL_DIR" )
  {
    if !d.is_empty() { return PathBuf::from( d ); }
  }
  // Fix(BUG-550): route an empty HOME to the /tmp fallback instead of joining onto it.
  // Root cause: `unwrap_or_else` fires only on Err (HOME unset); HOME="" yields Ok(""),
  //   and `PathBuf::from( "" ).join( ".clr" )` is RELATIVE, so `clj` silently read from
  //   (and reported) a cwd-relative journal instead of the documented absolute default.
  // Pitfall: `env::var` distinguishes unset (Err) from empty (Ok("")); the CLR_JOURNAL_DIR
  //   arm directly above already guards `is_empty()` — the HOME arm must match it.
  let home = std::env::var( "HOME" )
  .ok()
  .filter( | h | !h.is_empty() )
  .unwrap_or_else( || "/tmp".to_owned() );
  PathBuf::from( home ).join( ".clr" ).join( "journal" )
}

// ── Argument parsing helpers ──────────────────────────────────────────────────

/// Parse a human-readable duration string (e.g. `1h`, `30d`, `2w`) into a `Duration`.
///
/// Supported units: `s` (seconds), `m` (minutes), `h` (hours), `d` (days), `w` (weeks).
/// Returns `Err` with a descriptive message on invalid input.
///
/// # Errors
///
/// Returns an error when the input is not a valid `<number><unit>` pair.
#[ inline ]
pub fn parse_duration( s : &str ) -> Result< Duration, String >
{
  let err = || format!( "invalid duration '{s}' (expected e.g. 30s, 5m, 1h, 7d, 2w)" );
  let unit_char = s.chars().last().ok_or_else( err )?;
  let num_str   = &s[ ..s.len() - unit_char.len_utf8() ];
  let n : u64   = num_str.parse().map_err( | _ | err() )?;
  let secs = match unit_char
  {
    's' => n,
    'm' => n * 60,
    'h' => n * 3_600,
    'd' => n * 86_400,
    'w' => n * 86_400 * 7,
    _   => return Err( err() ),
  };
  Ok( Duration::from_secs( secs ) )
}

/// Parse an event type discriminator string into an `EventType`.
///
/// # Errors
///
/// Returns an error listing all valid type names when the input is not recognised.
#[ inline ]
pub fn parse_event_type( s : &str ) -> Result< EventType, String >
{
  EventType::parse( s ).ok_or_else( || format!(
    "invalid type '{s}' (valid: execution, credential, gate_wait, retry, \
     timeout, runner_retry, validation_retry, interactive)"
  ) )
}

/// Build a `JournalFilter` from the parsed param map.
///
/// Fix(param-exit-collision): read `exit`, not `exit_code`
/// Root cause: every documented example spells this `exit::`
///   (`docs/cli/param/05_exit.md`), but the code read `exit_code`. Since unknown
///   keys were accepted silently, `clj .list exit::2` returned the *unfiltered*
///   list — the documented invocation looked like it worked and quietly widened
///   the result set instead of narrowing it.
/// Pitfall: `exit_code` is the JSON *field* name; `exit::` is the CLI *param*
///   name. They are allowed to differ, but only one of them can be the key this
///   function looks up, and it has to be the one the docs and help text print.
///
/// # Errors
///
/// Returns a descriptive error string when any typed param (`since`, `until`,
/// `type`, `exit`, `limit`) fails to parse.
#[ inline ]
pub fn build_filter< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S > ) -> Result< JournalFilter, String >
{
  let mut f = JournalFilter::default();
  if let Some( s ) = params.get( "since" )
  {
    f.since = Some( parse_duration( s )? );
  }
  if let Some( s ) = params.get( "until" )
  {
    f.until = SystemTime::now().checked_sub( parse_duration( s )? );
  }
  if let Some( s ) = params.get( "type" )    { f.event_type = Some( parse_event_type( s )? ); }
  if let Some( s ) = params.get( "command" ) { f.command = Some( s.clone() ); }
  if let Some( s ) = params.get( "exit" )
  {
    // 255 is the documented ceiling (`docs/cli/param/05_exit.md`) and also the
    // real one — a Unix wait status carries a single byte. A negative value
    // used to parse cleanly here and then match nothing, which read as "no
    // failures" rather than "that is not an exit code".
    f.exit_code = Some( parse_int_param::< i32 >( s, "exit", 255 )? );
  }
  if let Some( s ) = params.get( "model" )   { f.model = Some( s.clone() ); }
  if let Some( s ) = params.get( "dir" )     { f.dir = Some( s.clone() ); }
  if let Some( s ) = params.get( "creds" )   { f.creds = Some( s.clone() ); }
  if let Some( s ) = params.get( "limit" )
  {
    let n : usize = parse_int_param( s, "limit", u64::MAX )?;
    // `limit::0` means unlimited (`docs/cli/param/09_limit.md`), and `query()`
    // stops once `limit` matches are collected — so handing it `Some( 0 )` would
    // return nothing at all, the exact opposite of what was asked for.
    f.limit = if n == 0 { None } else { Some( n ) };
  }
  Ok( f )
}

/// Parse a documented `Boolean` param — strictly `0` or `1`.
///
/// An absent value yields `false`, matching the `0` default these params
/// document. `name` is interpolated into the error so callers do not restate
/// it; the wording is the one `docs/cli/type/08_boolean.md` specifies.
///
/// # Errors
///
/// Returns `Err` when the value is anything other than `"0"` or `"1"`.
#[ inline ]
pub fn parse_bool_param( value : Option< &str >, name : &str ) -> Result< bool, String >
{
  match value
  {
    None | Some( "0" ) => Ok( false ),
    Some( "1" )        => Ok( true ),
    Some( other )      => Err(
      format!( "invalid boolean '{other}' for parameter '{name}' — expected 0 or 1" )
    ),
  }
}

/// Parse a documented `Integer` param — non-negative and at most `max`.
///
/// `max` is the parameter's own documented ceiling (`exit` is 0-255, `port` is
/// 0-65535), so the range lives at the call site next to the parameter it
/// belongs to rather than being restated here.
///
/// Negative, non-numeric, and out-of-range values all land on the single
/// message `docs/cli/type/04_integer.md` specifies. That is deliberate: each is
/// the parameter refusing a value it cannot represent, and the type page
/// documents one wording for the lot rather than three.
///
/// The final `T::try_from` cannot fail for any `max` a caller here passes — it
/// is a total conversion rather than a second range check, which is why it
/// returns the same message instead of a distinct one.
///
/// # Errors
///
/// Returns `Err` when `value` is not a non-negative integer, exceeds `max`, or
/// does not fit `T`.
#[ inline ]
pub fn parse_int_param< T >( value : &str, name : &str, max : u64 ) -> Result< T, String >
where
  T : TryFrom< u64 >,
{
  let invalid = || format!( "invalid integer '{value}' for parameter '{name}'" );
  let n : u64 = value.parse().map_err( | _ | invalid() )?;
  if n > max { return Err( invalid() ); }
  T::try_from( n ).map_err( | _ | invalid() )
}

/// Parse the documented `verbosity::` param — an `Integer` clamped to 0-2.
///
/// An absent value yields `1`, the documented default. Values above `2` clamp
/// rather than error, per `docs/cli/param_group/02_display.md`: asking for more
/// detail than exists is a coherent request that the highest level already
/// answers in full. Negative and non-numeric input still errors — those are
/// typos, not requests, and `docs/cli/type/04_integer.md` makes both exit 1
/// with the message this returns.
///
/// # Errors
///
/// Returns `Err` when `value` is not a non-negative integer.
#[ inline ]
pub fn parse_verbosity( value : Option< &str > ) -> Result< u8, String >
{
  let Some( s ) = value else { return Ok( 1 ) };
  let n : u64 = parse_int_param( s, "verbosity", u64::MAX )?;
  Ok( match n { 0 => 0, 1 => 1, _ => 2 } )
}

/// Percent-decode one query-string component, treating `+` as a space.
///
/// Invalid escapes are passed through literally rather than rejected — a
/// malformed `%zz` is far more likely to be a literal percent in a search
/// pattern than an encoding the caller meant.
fn percent_decode( raw : &str ) -> String
{
  let bytes = raw.as_bytes();
  let mut out = Vec::with_capacity( bytes.len() );
  let mut i = 0;
  while i < bytes.len()
  {
    match bytes[ i ]
    {
      b'+' => { out.push( b' ' ); i += 1; }
      b'%' if i + 2 < bytes.len() =>
      {
        let hex = raw.get( i + 1..i + 3 ).and_then( | h | u8::from_str_radix( h, 16 ).ok() );
        if let Some( byte ) = hex { out.push( byte ); i += 3; }
        else { out.push( b'%' ); i += 1; }
      }
      other => { out.push( other ); i += 1; }
    }
  }
  String::from_utf8_lossy( &out ).into_owned()
}

/// Parse an HTTP query string into the same param map shape the CLI produces.
///
/// Accepts the portion after `?` (with or without a leading `?`), splits on `&`
/// and `=`, and percent-decodes both halves. The result feeds
/// [`build_filter`] directly, so the HTTP API and the `.list` command share one
/// filter vocabulary instead of each defining its own.
///
/// A bare key with no `=` maps to an empty value; an empty segment is skipped.
#[ must_use ]
#[ inline ]
pub fn parse_query_string( query : &str ) -> HashMap< String, String >
{
  let mut out = HashMap::new();
  for pair in query.trim_start_matches( '?' ).split( '&' )
  {
    if pair.is_empty() { continue; }
    let ( k, v ) = pair.split_once( '=' ).unwrap_or( ( pair, "" ) );
    let key = percent_decode( k );
    if key.is_empty() { continue; }
    out.insert( key, percent_decode( v ) );
  }
  out
}

// ── Formatting helpers ────────────────────────────────────────────────────────

/// Process-wide override set by [`force_no_color`], for the `no_color::` param.
static FORCE_NO_COLOR : core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new( false );

/// Suppress ANSI codes for the rest of the process, as `no_color::1` requires.
///
/// Fix(param-no-color-unread): make the documented `no_color::` param do something
/// Root cause: `docs/cli/param/24_no_color.md` documents `no_color::1` with three
///   worked examples, but nothing ever read the key — only the `NO_COLOR` env
///   var. `clj .list no_color::1 > file` therefore wrote the escape codes the
///   param exists to remove, silently, with exit 0.
/// Pitfall: an override that only reads the environment cannot be driven by an
///   argument. Routing the param through a real override (rather than mutating
///   `NO_COLOR` in-process) keeps the two inputs to one decision function
///   instead of making one input impersonate the other.
#[ inline ]
pub fn force_no_color()
{
  FORCE_NO_COLOR.store( true, core::sync::atomic::Ordering::Relaxed );
}

/// Returns `true` when `NO_COLOR` is set in the environment, or [`force_no_color`]
/// was called.
#[ must_use ]
#[ inline ]
pub fn no_color() -> bool
{
  FORCE_NO_COLOR.load( core::sync::atomic::Ordering::Relaxed )
  || std::env::var_os( "NO_COLOR" ).is_some()
}

/// Wrap `s` in ANSI bold codes unless `NO_COLOR` is set.
#[ must_use ]
#[ inline ]
pub fn bold( s : &str ) -> String
{
  if no_color() { s.to_owned() }
  else { format!( "\x1b[1m{s}\x1b[0m" ) }
}

/// Format a millisecond duration as a human-readable string.
#[ must_use ]
#[ inline ]
pub fn format_ms( ms : u64 ) -> String
{
  if ms < 1_000 { format!( "{ms}ms" ) }
  else if ms < 60_000 { format!( "{:.1}s", ms as f64 / 1_000.0 ) }
  else { format!( "{:.1}m", ms as f64 / 60_000.0 ) }
}

/// Format a byte count as a human-readable size.
///
/// Binary steps with the conventional short labels, the same convention `du -h`
/// prints. One decimal above `B`, which keeps a size scannable without implying
/// precision the rounding does not have — `/api/health` reports the exact byte
/// count instead, so nothing that needs the true figure has to parse this.
#[ must_use ]
#[ inline ]
pub fn format_bytes( bytes : u64 ) -> String
{
  const KB : u64 = 1024;
  const MB : u64 = KB * 1024;
  const GB : u64 = MB * 1024;
  if bytes < KB      { format!( "{bytes} B" ) }
  else if bytes < MB { format!( "{:.1} KB", bytes as f64 / KB as f64 ) }
  else if bytes < GB { format!( "{:.1} MB", bytes as f64 / MB as f64 ) }
  else               { format!( "{:.1} GB", bytes as f64 / GB as f64 ) }
}

/// Return the event table header line.
#[ must_use ]
#[ inline ]
pub fn event_header() -> String
{
  format!(
    "{:<16}  {:<18}  {:<10}  {:<22}  {:<4}  {:<10}  {:<8}  {:<8}  DUR",
    "TIME", "TYPE", "CMD", "MODEL", "EXIT", "COST", "IN", "OUT"
  )
}

/// Format one event record as a compact table row string.
#[ must_use ]
#[ inline ]
pub fn format_event_row( ev : &EventRecord ) -> String
{
  let ts     = ev.ts.get( ..16 ).unwrap_or( &ev.ts );
  let etype  = ev.event_type.as_str();
  let exit   = ev.fields.exit_code.map_or_else( || "-".to_owned(), | c | c.to_string() );
  let dur    = ev.fields.duration_ms.map_or_else( || "-".to_owned(), format_ms );
  let cost   = ev.fields.cost_usd.map_or_else( || "-".to_owned(), | c | format!( "${c:.4}" ) );
  let model  = ev.fields.model.as_deref().unwrap_or( "-" );
  let cmd    = ev.fields.command.as_deref().unwrap_or( "-" );
  let intok  = ev.fields.input_tokens.map_or_else( || "-".to_owned(), | t | t.to_string() );
  let outtok = ev.fields.output_tokens.map_or_else( || "-".to_owned(), | t | t.to_string() );
  format!( "{ts}  {etype:<18}  {cmd:<10}  {model:<22}  {exit:<4}  {cost:<10}  {intok:<8}  {outtok:<8}  {dur}" )
}

/// Header row for CSV output, naming the columns [`format_csv_row`] emits.
#[ must_use ]
#[ inline ]
pub fn csv_header() -> &'static str
{
  "ts,type,command,model,exit_code,cost_usd,duration_ms"
}

/// Format one event as a CSV row matching [`csv_header`]'s columns.
///
/// A field the event does not carry renders as an empty cell, so column
/// positions stay aligned with the header for every row.
#[ must_use ]
#[ inline ]
pub fn format_csv_row( ev : &EventRecord ) -> String
{
  format!(
    "{},{},{},{},{},{},{}",
    ev.ts,
    ev.event_type.as_str(),
    ev.fields.command.as_deref().unwrap_or( "" ),
    ev.fields.model.as_deref().unwrap_or( "" ),
    ev.fields.exit_code.map_or_else( String::new, | c | c.to_string() ),
    ev.fields.cost_usd.map_or_else( String::new, | c | format!( "{c:.6}" ) ),
    ev.fields.duration_ms.map_or_else( String::new, | d | d.to_string() ),
  )
}

/// A `format::` value for `.tail`, parsed once before the follow loop begins.
///
/// Parsing up front is the point of the type: `.tail` blocks indefinitely, so a
/// bad format name must be rejected before the wait starts rather than when the
/// first event happens to arrive — which on a quiet journal could be never.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum StreamFormat
{
  /// Aligned columns, one line per event — the default.
  Table,
  /// One complete JSON object per line.
  ///
  /// Both `json` and `jsonl` parse to this variant. A JSON *array* has no valid
  /// streaming form: `.tail` does not end, so the closing bracket would never be
  /// written and a consumer piping to `jq` would block forever waiting for it.
  /// `docs/cli/param/10_format.md` documents `.tail format::json` as "follow
  /// events as JSON lines", which is what this emits.
  Jsonl,
  /// Comma-separated values, preceded once by [`csv_header`].
  Csv,
}

impl StreamFormat
{
  /// Parse a `format::` value.
  ///
  /// # Errors
  ///
  /// Returns `Err` naming the valid formats when `name` is unrecognized.
  #[ inline ]
  pub fn parse( name : &str ) -> Result< Self, String >
  {
    match name
    {
      "table"          => Ok( Self::Table ),
      "json" | "jsonl" => Ok( Self::Jsonl ),
      "csv"            => Ok( Self::Csv ),
      other => Err( format!( "invalid format '{other}' (valid: table, json, jsonl, csv)" ) ),
    }
  }

  /// Header line to print once before any rows, when this format has one.
  ///
  /// Only `csv` does — a CSV stream missing its header row is not the format
  /// `docs/cli/type/06_output_format.md` describes. `table` deliberately has
  /// none: `.tail` output is open-ended, so a header printed once scrolls away
  /// and then misleads for the rest of the session.
  #[ must_use ]
  #[ inline ]
  pub fn header( self ) -> Option< &'static str >
  {
    match self
    {
      Self::Csv                 => Some( csv_header() ),
      Self::Table | Self::Jsonl => None,
    }
  }

  /// Render one event as a single output line.
  ///
  /// # Errors
  ///
  /// Returns `Err` only when the event cannot be serialized to JSON, which a
  /// record parsed *out of* a JSON journal line cannot be.
  #[ inline ]
  pub fn render( self, ev : &EventRecord ) -> Result< String, String >
  {
    match self
    {
      Self::Table => Ok( format_event_row( ev ) ),
      Self::Jsonl => serde_json::to_string( ev ).map_err( | e | e.to_string() ),
      Self::Csv   => Ok( format_csv_row( ev ) ),
    }
  }
}

// ── Sorting ───────────────────────────────────────────────────────────────────

/// Valid `sort::` field names, in the order `docs/cli/type/07_sort_field.md` lists them.
const SORT_FIELDS : &str = "time, cost, duration, exit, model, command";

/// Compare two optional costs, ordering a missing cost below every real one.
///
/// `f64::total_cmp` rather than `partial_cmp` so a `NaN` cost — which a
/// hand-edited or truncated journal line can carry — still sorts
/// deterministically, instead of leaving the result dependent on which pairs the
/// sort happened to compare.
fn cmp_cost( a : Option< f64 >, b : Option< f64 > ) -> core::cmp::Ordering
{
  match ( a, b )
  {
    ( None, None )           => core::cmp::Ordering::Equal,
    ( None, Some( _ ) )      => core::cmp::Ordering::Less,
    ( Some( _ ), None )      => core::cmp::Ordering::Greater,
    ( Some( x ), Some( y ) ) => x.total_cmp( &y ),
  }
}

/// Sort `events` by `field` ascending, or descending when `reverse` is set.
///
/// Field names match case-insensitively per `docs/cli/type/07_sort_field.md`.
/// Events missing the chosen field sort *below* those that have it, so
/// `reverse::1` — the "largest first" direction — leads with real values and
/// leaves the unknowns at the bottom rather than opening on a block of `-`.
///
/// The sort is stable and `reverse` is applied inside the comparator rather than
/// by reversing the slice afterwards, so events tied on the key keep journal
/// order in both directions.
///
/// # Errors
///
/// Returns `Err` listing the valid fields when `field` is not one of them.
#[ inline ]
pub fn sort_events( events : &mut [ EventRecord ], field : &str, reverse : bool ) -> Result< (), String >
{
  let dir = | o : core::cmp::Ordering | if reverse { o.reverse() } else { o };
  let key = field.to_lowercase();
  match key.as_str()
  {
    "time"     => events.sort_by( | a, b | dir( a.ts.cmp( &b.ts ) ) ),
    "cost"     => events.sort_by( | a, b | dir( cmp_cost( a.fields.cost_usd, b.fields.cost_usd ) ) ),
    "duration" => events.sort_by( | a, b | dir( a.fields.duration_ms.cmp( &b.fields.duration_ms ) ) ),
    "exit"     => events.sort_by( | a, b | dir( a.fields.exit_code.cmp( &b.fields.exit_code ) ) ),
    "model"    => events.sort_by( | a, b | dir( a.fields.model.cmp( &b.fields.model ) ) ),
    "command"  => events.sort_by( | a, b | dir( a.fields.command.cmp( &b.fields.command ) ) ),
    _ => return Err( format!( "invalid sort '{field}' (valid: {SORT_FIELDS})" ) ),
  }
  Ok( () )
}

// ── Command output functions ──────────────────────────────────────────────────

/// `.list` — return filtered events rendered in the requested format.
///
/// Follows `docs/cli/command/01_list.md`'s three-step algorithm: query, apply
/// `sort`/`reverse`, then cap at `limit`. The cap is applied *after* the sort,
/// not passed to `query()`, because `query()` caps by stopping early — which
/// would hand the sort the oldest N events and let `sort::cost reverse::1
/// limit::10` report them as the ten most expensive.
///
/// Only `table` is rendered here; `json`/`jsonl`/`csv` delegate to
/// [`build_export_content`] so `.list format::csv` and `.export format::csv`
/// cannot drift apart.
///
/// # Errors
///
/// Returns `Err` when any filter param is invalid, `sort`/`reverse` are not
/// valid values, or the format is not `table`, `json`, `jsonl`, or `csv`.
#[ inline ]
pub fn list_output< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S >, dir : PathBuf ) -> Result< String, String >
{
  let mut filter = build_filter( params )?;
  // `build_filter` maps `limit::0` to `None` (unlimited), so an absent key and
  // an explicit `0` are indistinguishable in `filter.limit` — the documented
  // default of 50 applies only to the absent case.
  let cap = if params.contains_key( "limit" ) { filter.limit } else { Some( 50 ) };
  filter.limit = None;

  let mut events = JournalReader::open( dir ).query( &filter );
  let sort_field = params.get( "sort" ).map_or( "time", String::as_str );
  let reverse    = parse_bool_param( params.get( "reverse" ).map( String::as_str ), "reverse" )?;
  sort_events( &mut events, sort_field, reverse )?;
  if let Some( n ) = cap { events.truncate( n ); }

  let format = params.get( "format" ).map_or( "table", String::as_str );
  if format != "table" { return build_export_content( &events, format ); }

  if events.is_empty() { return Ok( "No events found.".to_owned() ); }
  let mut out = String::new();
  out.push_str( &bold( &event_header() ) );
  out.push( '\n' );
  for ev in &events
  {
    out.push_str( &format_event_row( ev ) );
    out.push( '\n' );
  }
  out.push( '\n' );
  let _ = write!( out, "{} event(s)", events.len() );
  Ok( out )
}

/// `.stats` — return a stats table aggregated by `by` (day, model, dir, or agent).
///
/// `day`/`model` rows are ordered by key (chronological / alphabetical);
/// `dir`/`agent` rows are ranked by descending event count — "top agents by
/// activity" (task 543). Events without the grouping field aggregate under a
/// visible `(no dir)` / `(no agent)` row, never silently dropped.
///
/// # Errors
///
/// Returns `Err` when any filter param is invalid or `by` is not one of
/// `"day"`, `"model"`, `"dir"`, `"agent"`.
#[ inline ]
pub fn stats_output< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S >, dir : PathBuf ) -> Result< String, String >
{
  let data = stats_data( params, dir )?;
  let mut out = format_stats_table( &data );
  out.push( '\n' );
  let _ = write!( out, "\nTotal: {} event(s)", data.total_events );
  Ok( out )
}

/// One aggregated statistics bucket.
///
/// The structured unit behind both the `.stats` text table and the web
/// dashboard's `/api/stats` JSON — the two renderings share this computation
/// rather than each aggregating events themselves.
#[ derive( Debug, Clone ) ]
pub struct StatsGroup
{
  /// Group key — a date, model name, directory, or agent id depending on `by::`.
  pub key : String,
  /// Number of events in this group.
  pub count : u64,
  /// Summed `cost_usd` across the group; events without a cost contribute 0.
  pub cost_usd : f64,
}

/// Aggregated statistics for one grouping dimension.
#[ derive( Debug, Clone ) ]
pub struct StatsData
{
  /// The `by::` dimension that produced this grouping (`day`, `model`, `dir`, `agent`).
  pub by : String,
  /// Column heading for the key column in the text rendering (`DATE`, `MODEL`, …).
  pub column_label : String,
  /// Groups in presentation order — already sorted per the dimension's own rule.
  pub groups : Vec< StatsGroup >,
  /// Total events matched by the filter, across all groups.
  pub total_events : usize,
}

/// `.stats` — aggregate events into groups without rendering them.
///
/// Applies the same default as the text command: when `since::` is absent the
/// window is the last 7 days.
///
/// # Errors
///
/// Returns `Err` when any filter param is invalid or `by::` is not one of
/// `day`, `model`, `dir`, `agent`.
#[ inline ]
pub fn stats_data< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S >, dir : PathBuf ) -> Result< StatsData, String >
{
  let mut filter = build_filter( params )?;
  if filter.since.is_none() { filter.since = Some( Duration::from_secs( 7 * 86_400 ) ); }

  let events = JournalReader::open( dir ).query( &filter );
  let by     = params.get( "by" ).map_or( "day", String::as_str );
  let ( column_label, groups ) = match by
  {
    "day" => (
      "DATE",
      group_events(
        &events,
        | ev | ev.ts.get( ..10 ).unwrap_or( "unknown" ).to_owned(),
        StatsOrder::KeyAscending,
      ),
    ),
    "model" => (
      "MODEL",
      group_events(
        &events,
        | ev | ev.fields.model.clone().unwrap_or_else( || "unknown".to_owned() ),
        StatsOrder::KeyAscending,
      ),
    ),
    "dir" => (
      "DIR",
      group_events(
        &events,
        | ev | ev.fields.dir.clone().unwrap_or_else( || "(no dir)".to_owned() ),
        StatsOrder::CountDescending,
      ),
    ),
    "agent" => (
      "AGENT",
      group_events(
        &events,
        | ev | ev.fields.agent_id.clone().unwrap_or_else( || "(no agent)".to_owned() ),
        StatsOrder::CountDescending,
      ),
    ),
    other => return Err( format!( "invalid by '{other}' (valid: day, model, dir, agent)" ) ),
  };

  Ok( StatsData
  {
    by           : by.to_owned(),
    column_label : column_label.to_owned(),
    groups,
    total_events : events.len(),
  } )
}

/// Row ordering for a stats grouping.
#[ derive( Clone, Copy ) ]
enum StatsOrder
{
  /// Ascending by group key — chronological for dates, alphabetical for models.
  KeyAscending,
  /// Descending by event count, ties broken by key — ranking groupings (`dir`, `agent`).
  CountDescending,
}

/// Bucket `events` by the key returned by `key_fn`, sorted per `order`.
fn group_events< F >( events : &[ EventRecord ], key_fn : F, order : StatsOrder ) -> Vec< StatsGroup >
where
  F : Fn( &EventRecord ) -> String,
{
  let mut buckets : std::collections::HashMap< String, ( f64, u64 ) > = std::collections::HashMap::new();
  for ev in events
  {
    let entry = buckets.entry( key_fn( ev ) ).or_insert( ( 0.0, 0 ) );
    entry.0 += ev.fields.cost_usd.unwrap_or( 0.0 );
    entry.1 += 1;
  }
  let mut rows : Vec< StatsGroup > = buckets
    .into_iter()
    .map( | ( key, ( cost_usd, count ) ) | StatsGroup { key, count, cost_usd } )
    .collect();
  match order
  {
    StatsOrder::KeyAscending    => rows.sort_by( | a, b | a.key.cmp( &b.key ) ),
    StatsOrder::CountDescending => rows.sort_by( | a, b | b.count.cmp( &a.count ).then_with( || a.key.cmp( &b.key ) ) ),
  }
  rows
}

/// Render a `StatsData` as the aligned text table used by `.stats`.
fn format_stats_table( data : &StatsData ) -> String
{
  let mut out = String::new();
  let label = &data.column_label;
  out.push_str( &bold( &format!( "{label:<24}  COUNT     COST" ) ) );
  out.push( '\n' );
  for StatsGroup { key, count, cost_usd } in &data.groups
  {
    let _ = writeln!( out, "{key:<24}  {count:<8}  ${cost_usd:.4}" );
  }
  out
}

/// `.search` — return events matching the pattern.
///
/// `pattern` is a literal substring, matched case-sensitively against six
/// fields: `message`, `stdout`, `stderr`, `error_message`, `model`, `command`.
/// The set is fixed — no parameter widens or narrows it.
///
/// # Errors
///
/// Returns `Err` when any filter param is invalid or the required `pattern::`
/// param is absent.
#[ inline ]
pub fn search_output< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S >, dir : PathBuf ) -> Result< String, String >
{
  let pattern = params.get( "pattern" )
    .cloned()
    .ok_or_else( || "pattern:: parameter required".to_owned() )?;
  let filter  = build_filter( params )?;
  let events  = JournalReader::open( dir ).query( &filter );
  let pat     = pattern.as_str();
  let matches : Vec< &EventRecord > = events.iter().filter( | ev |
  {
    // Fix(search-skips-message): `message` leads the set because it is the one
    //   field the caller wrote themselves, and so the one they are most likely
    //   to search for.
    // Root cause: the field list was assembled from what the *runner* captures
    //   (stdout/stderr/error_message) plus two identifiers, and the prompt — a
    //   field of `EventFields` since the schema was written, and the subject of
    //   `feature/001_cli_viewing.md` AC-006 — was never added.
    // Pitfall: the omission is invisible from the outside. A missing field does
    //   not narrow the result set in any way a caller can see; it returns exit 0
    //   and `No events matching '<pattern>'`, which reads as a definitive "not
    //   in the journal" rather than "not in the part of it that is searched".
    ev.fields.message.as_deref().unwrap_or( "" ).contains( pat )
      || ev.fields.stdout.as_deref().unwrap_or( "" ).contains( pat )
      || ev.fields.stderr.as_deref().unwrap_or( "" ).contains( pat )
      || ev.fields.error_message.as_deref().unwrap_or( "" ).contains( pat )
      || ev.fields.model.as_deref().unwrap_or( "" ).contains( pat )
      || ev.fields.command.as_deref().unwrap_or( "" ).contains( pat )
  } ).collect();

  if matches.is_empty()
  {
    return Ok( format!( "No events matching '{pattern}'." ) );
  }
  let mut out = String::new();
  out.push_str( &bold( &format!( "{:<16}  {:<18}  {:<10}  MATCH", "TIME", "TYPE", "CMD" ) ) );
  out.push( '\n' );
  for ev in &matches
  {
    let ts    = ev.ts.get( ..16 ).unwrap_or( &ev.ts );
    let etype = ev.event_type.as_str();
    let cmd   = ev.fields.command.as_deref().unwrap_or( "-" );
    let _ = writeln!( out, "{ts}  {etype:<18}  {cmd:<10}  (matched)" );
  }
  out.push( '\n' );
  let _ = write!( out, "{} match(es)", matches.len() );
  Ok( out )
}

/// Journal health snapshot.
///
/// The structured unit behind both the `.status` text output and the web
/// dashboard's `/api/health` JSON, so the two can never disagree about what
/// "healthy" reports.
#[ derive( Debug, Clone ) ]
pub struct HealthData
{
  /// Journal directory the snapshot describes.
  pub dir : PathBuf,
  /// Number of `YYYY-MM-DD.jsonl` files present.
  pub files : usize,
  /// Combined size of those files in bytes.
  pub bytes : u64,
  /// Date of the oldest journal file, or `None` when the journal is empty.
  pub oldest : Option< String >,
  /// Date of the newest journal file, or `None` when the journal is empty.
  pub newest : Option< String >,
}

impl HealthData
{
  /// Derive a snapshot from an already-collected file listing.
  ///
  /// Split out from [`health_data`] so `.status verbosity::2` — which needs the
  /// per-file rows *and* the totals — can derive both from one directory scan
  /// instead of scanning once for each and reporting two views of a directory
  /// that a rotation or prune may have changed in between.
  #[ must_use ]
  #[ inline ]
  pub fn from_files( dir : PathBuf, files : &[ JournalFileInfo ] ) -> Self
  {
    Self
    {
      files  : files.len(),
      bytes  : files.iter().map( | f | f.bytes ).sum(),
      oldest : files.first().map( | f | f.date.clone() ),
      newest : files.last().map( | f | f.date.clone() ),
      dir,
    }
  }

  /// Render the date span as `docs/cli/command/07_status.md`'s `Date range:` value.
  ///
  /// A journal holding one day's file reports that date alone rather than
  /// `X to X`, and an empty one says so outright — printing a `(none)`
  /// placeholder twice invites a reader to parse it as a real range.
  #[ must_use ]
  #[ inline ]
  pub fn date_range( &self ) -> String
  {
    match ( &self.oldest, &self.newest )
    {
      ( Some( o ), Some( n ) ) if o == n => o.clone(),
      ( Some( o ), Some( n ) )           => format!( "{o} to {n}" ),
      _                                  => "no events".to_owned(),
    }
  }
}

/// `.status` — collect journal health without rendering it.
///
/// Derives all four figures from one [`JournalReader::files`] listing rather
/// than four separate accessor calls: each accessor rescans the directory, so
/// the four-call form could report a count, a size, and a date range observed at
/// four different instants — mutually inconsistent whenever a rotation or prune
/// lands mid-call.
#[ must_use ]
#[ inline ]
pub fn health_data( dir : PathBuf ) -> HealthData
{
  let files = JournalReader::open( dir.clone() ).files();
  HealthData::from_files( dir, &files )
}

/// Report the configured journal level and where that value came from.
///
/// `CLR_JOURNAL` is `clr`'s own switch (`full`, `meta`, `off`). `.status`
/// reports it verbatim rather than validating it: `clr` is the component that
/// rejects a bad level, and normalizing it here would hide the very value a
/// user came to `.status` to check.
#[ must_use ]
#[ inline ]
pub fn journal_level() -> String
{
  match std::env::var( "CLR_JOURNAL" )
  {
    Ok( v ) if !v.is_empty() => format!( "{v} (CLR_JOURNAL={v})" ),
    _                        => "full (default)".to_owned(),
  }
}

/// `.status` — render the journal health report at the requested verbosity.
///
/// Three levels, per `docs/cli/param/22_verbosity.md`: `0` collapses the report
/// to a single line, `1` is the standard report, `2` appends a per-file
/// breakdown. All three are derived from one directory scan, so no two lines of
/// the report can describe different moments.
///
/// # Errors
///
/// Returns `Err` when `verbosity::` is not a non-negative integer.
#[ inline ]
pub fn status_output< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S >, dir : PathBuf ) -> Result< String, String >
{
  let level = parse_verbosity( params.get( "verbosity" ).map( String::as_str ) )?;
  let files = JournalReader::open( dir.clone() ).files();
  let h     = HealthData::from_files( dir, &files );
  let size  = format_bytes( h.bytes );
  let range = h.date_range();

  if level == 0
  {
    return Ok( format!( "{} files, {size}, {range}", h.files ) );
  }

  let mut out = format!(
    "Journal directory: {}\nFiles: {}\nTotal size: {size}\nDate range: {range}\nJournal level: {}",
    h.dir.display(),
    h.files,
    journal_level(),
  );
  if level >= 2
  {
    out.push_str( "\n\n" );
    // No column header on an empty journal — a `DATE`/`SIZE` heading above zero
    // rows announces a table that is not there.
    if files.is_empty()
    {
      out.push_str( "(no journal files)" );
    }
    else
    {
      out.push_str( &bold( &format!( "{:<12}  SIZE", "DATE" ) ) );
      for f in &files
      {
        let _ = write!( out, "\n{:<12}  {}", f.date, format_bytes( f.bytes ) );
      }
    }
  }
  Ok( out )
}

/// `.prune` — delete old journal files; return a description of what was done.
///
/// This function has filesystem side effects when `dry_run` is `false`.
///
/// Delegates to `claude_journal::rotation::prune_by_age`: age comes from the
/// `YYYY-MM-DD.jsonl` filename date (never filesystem mtime), only
/// pattern-matching files are candidates, and today's file is never deleted.
/// A sub-day `keep::` duration (e.g. `1h`) floors to 0 days — for daily-rotated
/// files that means "keep only today's file".
///
/// # Errors
///
/// Returns `Err` when `keep::` or `dry_run::` params are invalid.
#[ inline ]
pub fn prune_output< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S >, dir : PathBuf ) -> Result< String, String >
{
  let keep_str = params.get( "keep" ).map_or( "30d", String::as_str );
  let keep_dur = parse_duration( keep_str )?;
  let dry_run  = parse_bool_param( params.get( "dry_run" ).map( String::as_str ), "dry_run" )?;
  if !dir.is_dir()
  {
    return Ok( format!( "Journal dir {} not found or empty.", dir.display() ) );
  }
  let keep_days = u32::try_from( keep_dur.as_secs() / 86_400 ).unwrap_or( u32::MAX );
  let report    = prune_by_age( &dir, keep_days, today_ymd(), dry_run );
  let mut lines = Vec::new();
  let mut count = 0_u32;
  for ( path, action ) in report
  {
    match action
    {
      PruneAction::Deleted     => { lines.push( format!( "Deleted: {}", path.display() ) ); count += 1; }
      PruneAction::WouldDelete => { lines.push( format!( "Would delete: {}", path.display() ) ); count += 1; }
      PruneAction::Failed( e ) => lines.push( format!( "Warning: could not delete {}: {e}", path.display() ) ),
    }
  }
  if lines.is_empty() { return Ok( "Nothing to prune (all files within keep window).".to_owned() ); }
  let mut out = lines.join( "\n" );
  out.push( '\n' );
  out.push( '\n' );
  let msg = if dry_run
  {
    format!( "{count} file(s) would be pruned." )
  }
  else
  {
    format!( "{count} file(s) pruned." )
  };
  out.push_str( &msg );
  Ok( out )
}

/// Build export file content for `events` in the given `format`.
///
/// # Errors
///
/// Returns `Err` for unknown format names.
#[ inline ]
pub fn build_export_content( events : &[ EventRecord ], format : &str ) -> Result< String, String >
{
  match format
  {
    "json" => Ok(
      serde_json::to_string_pretty( events ).unwrap_or_else( | _ | "[]".to_owned() )
    ),
    "jsonl" => Ok(
      events.iter()
        .filter_map( | ev | serde_json::to_string( ev ).ok() )
        .collect::< Vec< _ > >()
        .join( "\n" )
    ),
    "csv" =>
    {
      let mut rows = vec![ csv_header().to_owned() ];
      rows.extend( events.iter().map( format_csv_row ) );
      Ok( rows.join( "\n" ) )
    }
    "table" =>
    {
      let mut rows = vec![
        format!(
          "{:<16}  {:<18}  {:<10}  {:<22}  EXIT  COST",
          "TIME", "TYPE", "CMD", "MODEL"
        )
      ];
      for ev in events
      {
        let ts    = ev.ts.get( ..16 ).unwrap_or( &ev.ts );
        let etype = ev.event_type.as_str();
        let cmd   = ev.fields.command.as_deref().unwrap_or( "-" );
        let model = ev.fields.model.as_deref().unwrap_or( "-" );
        let exit  = ev.fields.exit_code.map_or_else( || "-".to_owned(), | c | c.to_string() );
        let cost  = ev.fields.cost_usd.map_or_else( || "-".to_owned(), | c | format!( "${c:.4}" ) );
        rows.push( format!( "{ts:<16}  {etype:<18}  {cmd:<10}  {model:<22}  {exit:<4}  {cost}" ) );
      }
      Ok( rows.join( "\n" ) )
    }
    other => Err( format!( "invalid format '{other}' (valid: json, jsonl, csv, table)" ) ),
  }
}

/// `.export` — write events to a file; return a confirmation message.
///
/// # Errors
///
/// Returns `Err` when `output::` is missing, any filter param is invalid,
/// the format is unknown, or the file cannot be written.
#[ inline ]
pub fn export_output< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S >, dir : PathBuf ) -> Result< String, String >
{
  let output  = params.get( "output" )
    .cloned()
    .ok_or_else( || "output:: parameter required".to_owned() )?;
  let format  = params.get( "format" ).map_or( "json", String::as_str );
  let filter  = build_filter( params )?;
  let events  = JournalReader::open( dir ).query( &filter );
  let content = build_export_content( &events, format )?;
  std::fs::write( &output, &content )
    .map_err( | e | format!( "could not write to '{output}': {e}" ) )?;
  Ok( format!( "Exported {} event(s) to {output}", events.len() ) )
}

/// `.chart` — render a usage SVG chart, optionally opened in the default browser.
///
/// # Errors
///
/// Returns `Err` when `open::` is not a documented boolean, or when the chart
/// cannot be rendered or written to disk. A failure to *open* the resulting
/// file in a browser is non-fatal — it is appended as a warning to the success
/// message rather than failing the command.
#[ inline ]
pub fn chart_output< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S >, dir : PathBuf ) -> Result< String, String >
{
  // Validated before the chart is rendered, not after: a rejected `open::`
  // should not leave a written SVG behind on the way to exit 1, or the command
  // both fails and has an effect.
  let open_requested = parse_bool_param( params.get( "open" ).map( String::as_str ), "open" )?;

  let out_path = params.get( "out" ).map_or_else( || PathBuf::from( "usage.svg" ), PathBuf::from );
  claude_journal_charts::generate_usage_chart( &dir, &out_path ).map_err( | e | e.to_string() )?;

  let mut msg = format!( "Chart written to {}", out_path.display() );
  if open_requested
  {
    if let Err( e ) = open::that( &out_path )
    {
      let _ = write!( msg, "\nWarning: could not open {} in browser: {e}", out_path.display() );
    }
  }
  Ok( msg )
}

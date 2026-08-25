//! Shared output logic for the `clj` journal viewer.
//!
//! Each command handler returns a `String` (or `Result<String, String>` for
//! commands that validate input) so that the same logic can be used from both
//! the `clj` binary (`cli_main.rs`) and the unilang assistant routines
//! (`routines.rs`).

use claude_journal::rotation::{ prune_by_age, today_ymd, PruneAction };
use claude_journal::{ EventRecord, EventType, JournalFilter, JournalReader };
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
    f.exit_code = Some(
      s.parse::< i32 >()
        .map_err( | _ | format!( "invalid exit '{s}' (expected integer)" ) )?
    );
  }
  if let Some( s ) = params.get( "model" )   { f.model = Some( s.clone() ); }
  if let Some( s ) = params.get( "dir" )     { f.dir = Some( s.clone() ); }
  if let Some( s ) = params.get( "creds" )   { f.creds = Some( s.clone() ); }
  if let Some( s ) = params.get( "limit" )
  {
    f.limit = Some(
      s.parse::< usize >()
        .map_err( | _ | format!( "invalid limit '{s}' (expected non-negative integer)" ) )?
    );
  }
  Ok( f )
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

// ── Command output functions ──────────────────────────────────────────────────

/// `.list` — return a formatted event table or JSON array.
///
/// # Errors
///
/// Returns `Err` when any filter param is invalid or when the format is not
/// `"table"` or `"json"`.
#[ inline ]
pub fn list_output< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S >, dir : PathBuf ) -> Result< String, String >
{
  let mut filter = build_filter( params )?;
  if filter.limit.is_none() { filter.limit = Some( 50 ); }

  let events = JournalReader::open( dir ).query( &filter );
  let format = params.get( "format" ).map_or( "table", String::as_str );
  match format
  {
    "json" => Ok(
      serde_json::to_string_pretty( &events )
        .map_err( | e | e.to_string() )?
    ),
    "table" =>
    {
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
    other => Err( format!( "invalid format '{other}' (valid: table, json)" ) ),
  }
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
    ev.fields.stdout.as_deref().unwrap_or( "" ).contains( pat )
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

/// `.status` — collect journal health without rendering it.
#[ must_use ]
#[ inline ]
pub fn health_data( dir : PathBuf ) -> HealthData
{
  let reader = JournalReader::open( dir.clone() );
  HealthData
  {
    files  : reader.file_count(),
    bytes  : reader.total_bytes(),
    oldest : reader.oldest_date(),
    newest : reader.newest_date(),
    dir,
  }
}

/// `.status` — return a journal health string.
#[ must_use ]
#[ inline ]
pub fn status_output( dir : PathBuf ) -> String
{
  let h      = health_data( dir );
  let count  = h.files;
  let bytes  = h.bytes;
  let oldest = h.oldest.unwrap_or_else( || "(none)".to_owned() );
  let newest = h.newest.unwrap_or_else( || "(none)".to_owned() );
  format!(
    "{}\ndir:    {}\nfiles:  {count}\nsize:   {bytes} bytes\noldest: {oldest}\nnewest: {newest}",
    bold( "Journal Status" ),
    h.dir.display(),
  )
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
  let dry_raw  = params.get( "dry_run" ).map_or( "0", String::as_str );
  let dry_run  = match dry_raw
  {
    "0" | "false" => false,
    "1" | "true"  => true,
    other => return Err(
      format!( "invalid dry_run '{other}' (valid: 0, 1, true, false)" )
    ),
  };
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
      let mut rows = vec![ "ts,type,command,model,exit_code,cost_usd,duration_ms".to_owned() ];
      for ev in events
      {
        rows.push( format!(
          "{},{},{},{},{},{},{}",
          ev.ts,
          ev.event_type.as_str(),
          ev.fields.command.as_deref().unwrap_or( "" ),
          ev.fields.model.as_deref().unwrap_or( "" ),
          ev.fields.exit_code.map_or_else( String::new, | c | c.to_string() ),
          ev.fields.cost_usd.map_or_else( String::new, | c | format!( "{c:.6}" ) ),
          ev.fields.duration_ms.map_or_else( String::new, | d | d.to_string() ),
        ) );
      }
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
/// Returns `Err` when the chart cannot be rendered or written to disk. A
/// failure to open the resulting file in a browser is non-fatal — it is
/// appended as a warning to the success message rather than failing the
/// command.
#[ inline ]
pub fn chart_output< S : ::core::hash::BuildHasher >( params : &HashMap< String, String, S >, dir : PathBuf ) -> Result< String, String >
{
  let out_path = params.get( "out" ).map_or_else( || PathBuf::from( "usage.svg" ), PathBuf::from );
  claude_journal_charts::generate_usage_chart( &dir, &out_path ).map_err( | e | e.to_string() )?;

  let mut msg = format!( "Chart written to {}", out_path.display() );
  let open_requested = matches!( params.get( "open" ).map( String::as_str ), Some( "1" | "true" ) );
  if open_requested
  {
    if let Err( e ) = open::that( &out_path )
    {
      let _ = write!( msg, "\nWarning: could not open {} in browser: {e}", out_path.display() );
    }
  }
  Ok( msg )
}

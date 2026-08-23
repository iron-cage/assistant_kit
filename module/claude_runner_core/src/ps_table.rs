//! `.ps`-style table rendering for a [`ProcessInfo`] slice.
//!
//! Feature-gated behind `ps_table` so plain `claude_runner_core` consumers
//! never pull in `data_fmt`.
//!
//! Two renderers, by richness:
//! - [`render_ps_table`]: bare PID + cwd + state (3 columns). No `/proc` scanning
//!   beyond what the caller already gathered into [`ProcessInfo`].
//! - [`render_active_sessions_table`]: the full "Active Sessions" table — elapsed,
//!   CPU%, RAM, session flags (👈🆕🖨🔌⚡🕰🐘🧟⚠🐳), and the Task column (last
//!   human message from the session's JSONL log) — shared by `clr ps` and
//!   `claude_version`'s `.ps` so both CLIs render sessions identically. Linux-only
//!   per-process metrics come from `/proc`; on other platforms those columns render
//!   as `-` and no flags fire. Samples cumulative CPU ticks twice, 1 second apart, to
//!   compute the ⚡ Active flag — every call with a non-empty (post-filter) process
//!   list blocks for ~1s as a result.

use crate::process::ProcessInfo;
#[ cfg( target_os = "linux" ) ]
use crate::process::ProcessMetrics;
use crate::types::OutputFormat;
use data_fmt::{ Format, Heading, RowBuilder, TableConfig, TableFormatter };
use std::collections::HashMap;

/// Render a list of Claude Code processes as a table (`text`, `v >= 1`), a
/// compact PID+cwd listing (`text`, `v == 0`), or a JSON array (`json`).
///
/// Columns: PID, working directory, state. `state` is always `"running"` —
/// every entry is expected to originate from [`crate::process::find_claude_processes`],
/// which enumerates only processes currently present in `/proc`; this module
/// has no independent means of observing a non-running state.
///
/// Empty input renders `"no active processes"` in text mode, or `[]` in JSON mode.
///
/// # Examples
///
/// ```
/// use claude_runner_core::ps_table::render_ps_table;
/// use claude_runner_core::OutputFormat;
///
/// assert_eq!( render_ps_table( &[], OutputFormat::Text, 1 ), "no active processes" );
/// assert_eq!( render_ps_table( &[], OutputFormat::Json, 1 ), "[]" );
/// ```
// pub only because the ps_table extraction moved this out of clr's private cli::ps module —
// a once-per-invocation CLI table renderer, not an inlining-sensitive API.
#[ allow( clippy::missing_inline_in_public_items ) ]
#[ must_use ]
pub fn render_ps_table( processes : &[ ProcessInfo ], format : OutputFormat, verbosity : u8 ) -> String
{
  match format
  {
    OutputFormat::Json => render_json( processes ),
    OutputFormat::Text | OutputFormat::StreamJson => render_text( processes, verbosity ),
  }
}

/// Escape a string for embedding in a hand-built JSON string literal.
fn json_escape( s : &str ) -> String
{
  s.replace( '\\', "\\\\" ).replace( '"', "\\\"" )
}

fn render_json( processes : &[ ProcessInfo ] ) -> String
{
  let entries : Vec< String > = processes.iter().map( | p |
  {
    let cwd = json_escape( &p.cwd.to_string_lossy() );
    format!( "{{\"pid\":{},\"cwd\":\"{cwd}\",\"state\":\"running\"}}", p.pid )
  } ).collect();
  format!( "[{}]", entries.join( "," ) )
}

fn render_text( processes : &[ ProcessInfo ], verbosity : u8 ) -> String
{
  if processes.is_empty()
  {
    return "no active processes".to_string();
  }

  if verbosity == 0
  {
    // Compact: no column headers, mirrors `.ps v::0` (pid + cwd, one per line).
    let lines : Vec< String > = processes.iter()
      .map( | p | format!( "{} {}", p.pid, p.cwd.display() ) )
      .collect();
    return lines.join( "\n" );
  }

  let headers = vec![ "PID".to_string(), "Working Directory".to_string(), "State".to_string() ];
  let mut builder = RowBuilder::new( headers );
  for p in processes
  {
    let row : Vec< String > = vec![ p.pid.to_string(), p.cwd.display().to_string(), "running".to_string() ];
    builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
  }

  let config = TableConfig::plain().with_auto_wrap( false ).with_auto_fold( false );
  Format::format( &TableFormatter::with_config( config ), &builder.build_view() ).unwrap_or_default()
}

/// Configuration for [`render_active_sessions_table`] — mirrors `clr ps`'s CLI options
/// once resolved into their final, validated form. Column-key validation and `--wide`/
/// `--columns` precedence are CLI-argument concerns and stay with each caller; `columns`
/// here is already the final ordered key list (or `None` for [`DEFAULT_COLUMNS`]).
#[ derive( Debug, Clone ) ]
pub struct PsTableOptions
{
  /// Mode filter: `None` or `"all"` = no filter; `"print"` / `"interactive"` / `"query"` = filter rows.
  pub mode         : Option< String >,
  /// Pre-resolved, already-validated column keys (see [`COLUMN_KEYS`]); `None` = [`DEFAULT_COLUMNS`].
  pub columns      : Option< Vec< &'static str > >,
  /// PID filter; empty = show all sessions.
  pub pids         : Vec< u32 >,
  /// Elapsed-seconds threshold above which the 🕰 (Ancient) flag fires. Default: 28800 (8h).
  pub ancient_secs : u64,
  /// RAM megabytes threshold above which the 🐘 (High RAM) flag fires. Default: 400 MB.
  pub high_ram_mb  : u64,
  /// PIDs present in the previous `clr ps` snapshot, for computing the 🆕 flag.
  /// `None` = no snapshot comparison (default; `claude_version`'s `.ps` never sets
  /// this, and `clr ps`'s very first invocation has no snapshot yet). `Some(set)`
  /// — even if empty — enables 🆕 for any current PID absent from `set`.
  pub prior_pids   : Option< std::collections::HashSet< u32 > >,
}

impl Default for PsTableOptions
{
  #[ inline ]
  fn default() -> Self
  {
    Self
    {
      mode         : None,
      columns      : None,
      pids         : Vec::new(),
      ancient_secs : 28_800,
      high_ram_mb  : 400,
      prior_pids   : None,
    }
  }
}

/// All 11 column keys in display order, paired with their table header strings.
pub const COLUMN_KEYS : &[ ( &str, &str ) ] = &[
  ( "idx",     "#" ),
  ( "pid",     "PID" ),
  ( "elapsed", "Elapsed" ),
  ( "cpu",     "CPU%" ),
  ( "ram",     "RAM" ),
  ( "state",   "State" ),
  ( "path",    "Absolute Path" ),
  ( "task",    "Task" ),
  ( "mode",    "Mode" ),
  ( "cmd",     "Command" ),
  ( "binary",  "Binary" ),
];

/// Default column set (7 columns) shown when neither `--wide` nor `--columns` is given.
///
/// `state` and `mode` are deliberately absent: both are now carried by the conditional
/// `Flags` column, which spends characters only when there is something to report.
/// `mode` was pure duplication — [`compute_flags`] derives 🖨/🔌 from the very same
/// [`classify_mode`] call, so a `Mode` cell could only ever restate a flag or say
/// "interactive" (the silent default), and the caption already prints the mode census.
/// `state` is `S` for every idle session; only the abnormal letters carry signal, and
/// those now raise 🧟. Both remain selectable via `--columns` / `--wide`.
pub const DEFAULT_COLUMNS : &[ &str ] = &[
  "idx", "pid", "elapsed", "cpu", "ram", "path", "task",
];

/// Classify a process's execution mode from its cmdline args.
///
/// Returns `"query"` when the args carry the exact 3-condition control-session
/// signature `spawn_control_session()` itself validates (`--input-format stream-json` +
/// `--output-format stream-json` + `--verbose`) — task 418's `clr query` daemon is
/// currently the only caller of that method, so this signature is unique in practice.
/// Returns `"print"` when `--print` or `-p` appears as a discrete argument in `args[1..]`.
/// Returns `"interactive"` otherwise.
///
/// Must use `args` (NUL-split) — NOT `cmdline` (space-joined) — because a path
/// component could contain the substring "--print" producing a false positive.
///
/// Shared by `clr ps` (mode filter, 🖨/🔌 flags, inspect blocks) and `clr`'s gate
/// (print-mode session counting) — the single definition both rely on.
#[ must_use ]
// Was pub( super ) in clr's cli::ps before the ps_table extraction — pub purely to cross the
// crate boundary for claude_version's .ps, so cross-crate inlining is not a design concern.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn classify_mode( args : &[ String ] ) -> &str
{
  if has_flag_pair( args, "--input-format", "stream-json" )
    && has_flag_pair( args, "--output-format", "stream-json" )
    && args.iter().any( | a | a == "--verbose" )
  {
    "query"
  }
  else if args.iter().any( | a | a == "--print" || a == "-p" )
  {
    "print"
  }
  else
  {
    "interactive"
  }
}

// Detect an adjacent `flag value` pair anywhere in `args` (order-sensitive, matching
// `spawn_control_session()`'s own `has_flag_pair` validation shape in claude_runner_core).
fn has_flag_pair( args : &[ String ], flag : &str, value : &str ) -> bool
{
  args.windows( 2 ).any( | w | w[ 0 ] == flag && w[ 1 ] == value )
}

/// Extract a u64 value for `key` from a compact JSON object in `content`.
///
/// Shared by `clr ps`'s queued-waiters table and `clr`'s gate (slot/ticket file
/// staleness checks) — the single definition both rely on.
#[ must_use ]
// Was pub( super ) in clr's cli::ps before the ps_table extraction — pub purely to cross the
// crate boundary for the gate's staleness checks, not an inlining-sensitive API.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn parse_json_u64( content : &str, key : &str ) -> Option< u64 >
{
  let marker = format!( r#""{key}":"# );
  let start  = content.find( marker.as_str() )? + marker.len();
  let rest   = &content[ start.. ];
  let end    = rest.find( [ ',', '}' ] )?;
  rest[ ..end ].trim().parse().ok()
}

// Return current Unix timestamp in seconds. Private: elapsed-time formatting and the
// 🕰 Ancient flag are this module's only consumers; `clr`'s own gate.rs keeps its own
// copy for its unrelated slot/ticket timing (this module cannot depend on that crate).
fn unix_now() -> u64
{
  std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .map_or( 0, |d| d.as_secs() )
}

/// Render a completed [`RowBuilder`] as a headed plain-style table string.
///
/// `data_fmt` ≥0.5.1 fills the heading rule to the rendered table body width
/// automatically (TSK-008), so no two-pass probe is required.
/// `auto_wrap: false` — prevents word-wrapping long paths across continuation rows.
///
/// Shared by `clr ps` (active/queued/ended tables) and this module's own
/// [`render_active_sessions_table`] — the single definition every plain-style
/// `clr ps`-family table renders through.
#[ must_use ]
// pub only because the ps_table extraction moved this out of clr's private cli::ps module —
// a once-per-table CLI formatter, not an inlining-sensitive API.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn render_headed_table( builder : RowBuilder, heading : Heading ) -> String
{
  Format::format(
    &TableFormatter::with_config(
      TableConfig::plain()
        .with_heading( heading ),
    ),
    &builder.build_view(),
  ).unwrap_or_default()
}

// Per-flag metadata in canonical display order (👈🖨🔌⚡🕰🐘🧟⚠🐳).
// Only used on Linux because compute_flags is Linux-only.
#[ cfg( target_os = "linux" ) ]
const FLAG_LEGEND : &[ ( &str, &str ) ] = &[
  ( "👈", "This session" ),
  ( "🆕", "New since last check" ),
  ( "🖨",  "Print mode"   ),
  ( "🔌", "Query mode"   ),
  ( "⚡", "Active"       ),
  ( "🕰",  "Ancient"      ),
  ( "🐘", "High RAM"     ),
  ( "🧟", "Odd state"    ),
  ( "⚠",  "Dead metrics" ),
  ( "🐳", "Container"    ),
];

// Read cumulative CPU ticks (utime + stime) from `/proc/{pid}/stat`.
// Returns `None` if the process exited or fields are unreadable.
//
// WHY two-sample delta instead of kernel state R (field 3):
// State R is a microsecond snapshot — it detected only 1-2 of 20 active sessions
// in live testing.  Cumulative ticks delta over 1 s reliably identifies sustained
// CPU use; a threshold of 3 ticks (≈ 30 ms) separates real work from BUG-304
// timer noise (1-2 ticks) with no observed false positives or false negatives.
#[ cfg( target_os = "linux" ) ]
fn read_cpu_ticks( pid : u32 ) -> Option< u64 >
{
  let data = std::fs::read_to_string( format!( "/proc/{pid}/stat" ) ).ok()?;
  // Field 1 is (comm) which may contain spaces — find closing ')' first.
  let close_paren = data.find( ')' )?;
  let after_comm = &data[ close_paren + 2.. ]; // skip ") "
  let rest : Vec< &str > = after_comm.split_whitespace().collect();
  // rest[0] = state (field 3), rest[11] = utime (field 14), rest[12] = stime (field 15).
  let utime : u64 = rest.get( 11 )?.parse().ok()?;
  let stime : u64 = rest.get( 12 )?.parse().ok()?;
  Some( utime + stime )
}

// Compute session-flag emoji string for a single process row.
//
// Flags in canonical order 👈🆕🖨🔌⚡🕰🐘🧟⚠🐳 (only symbols that fire are included).
// Pure computation — no filesystem I/O beyond what the caller already has in `metrics` and
// `cpu_delta_ticks` (both pre-computed by the caller).
// The `/proc/{my_ppid}/cmdline` read for 👈 is inexpensive and done once per call.
#[ cfg( target_os = "linux" ) ]
fn push_flag( flags : &mut String, c : char )
{
  if !flags.is_empty() { flags.push( ' ' ); }
  flags.push( c );
}

// Loop-invariant inputs to compute_flags, computed once per table render and shared
// across every row — bundled so the function stays under clippy's 7-argument limit.
#[ cfg( target_os = "linux" ) ]
#[ derive( Debug ) ]
struct FlagContext< 'a >
{
  // $HOME, for the 🐳 outside-home test. Empty when unset — 🐳 never fires then.
  home         : &'a str,
  // Elapsed-seconds threshold above which 🕰 fires.
  ancient_secs : u64,
  // RSS threshold in MB above which 🐘 fires.
  high_ram_mb  : u64,
  // This process's own parent PID, for the 👈 this-session test.
  my_ppid      : u32,
}

#[ cfg( target_os = "linux" ) ]
fn compute_flags(
  proc            : &ProcessInfo,
  metrics         : Option< &ProcessMetrics >,
  ctx             : &FlagContext< '_ >,
  cpu_delta_ticks : u64,
  is_new          : bool,
) -> String
{
  let mut flags = String::new();

  // 👈 This session: caller is a direct child of this claude process.
  if proc.pid == ctx.my_ppid
  {
    // Verify the parent's cmdline arg[0] basename == "claude".
    let is_claude = std::fs::read( format!( "/proc/{}/cmdline", ctx.my_ppid ) )
      .ok()
      .and_then( | b |
      {
        let arg0 : Vec< u8 > = b.split( | &c | c == b'\0' )
          .next()
          .unwrap_or( &[] )
          .to_vec();
        String::from_utf8( arg0 ).ok()
      } )
      .is_some_and( | s |
      {
        std::path::Path::new( &s )
          .file_name()
          .and_then( | n | n.to_str() )
          == Some( "claude" )
      } );
    if is_claude { push_flag( &mut flags, '👈' ); }
  }

  // 🆕 New since last check: this PID was absent from the previous `clr ps`
  // snapshot. `is_new` is precomputed by the caller from `opts.prior_pids` —
  // see PsTableOptions::prior_pids doc for the None-vs-Some(empty) contract.
  if is_new { push_flag( &mut flags, '🆕' ); }

  // 🖨 Print mode: cmdline contains --print or -p.
  if classify_mode( &proc.args ) == "print" { push_flag( &mut flags, '🖨' ); }

  // 🔌 Query mode: PID-addressed control session (clr query), task 418.
  if classify_mode( &proc.args ) == "query" { push_flag( &mut flags, '🔌' ); }

  // ⚡ Active: CPU delta >= 3 ticks in 1-second sample window.
  // Threshold separates active sessions (6-100 ticks) from BUG-304 timer noise (1-2 ticks).
  if cpu_delta_ticks >= 3 { push_flag( &mut flags, '⚡' ); }

  if let Some( m ) = metrics
  {
    // 🕰 Ancient: elapsed seconds exceed the configured threshold.
    let elapsed = unix_now().saturating_sub( m.started_at );
    if elapsed > ctx.ancient_secs { push_flag( &mut flags, '🕰' ); }

    // 🐘 High RAM: RSS exceeds threshold. Comparison in KB to avoid integer-division
    //   truncation (e.g. 512 KB / 1024 = 0 MB, which would never exceed a 0 MB threshold).
    if m.ram_kb > ctx.high_ram_mb.saturating_mul( 1_024 ) { push_flag( &mut flags, '🐘' ); }

    // 🧟 Odd state: kernel state is neither R (running) nor S (interruptible sleep) —
    //   D uninterruptible, T/t stopped/traced, and anything else /proc reports.
    //   R and S are the entire boring case: every idle session sits in S, and sustained R
    //   is already reported by ⚡, which uses a 1 s tick delta precisely because state R is
    //   a microsecond snapshot (see read_cpu_ticks).  This is the flag that replaced the
    //   State default column, so the abnormal letters stay visible without one cell per row
    //   restating "S"; `--columns state` still shows the letter itself.
    //   NOTE: Z (zombie) matches this condition but is unreachable in practice — the kernel
    //   clears cmdline on exit, and find_claude_processes requires argv[0]'s basename to be
    //   "claude", so a zombie is dropped at discovery and never reaches any flag at all.
    //   It is not routed to ⚠ either: ⚠ means the /proc read failed, a different case.
    if !matches!( m.state, 'R' | 'S' ) { push_flag( &mut flags, '🧟' ); }
  }
  else
  {
    // ⚠ Dead metrics: read_process_metrics returned None — the process exited between the
    //   /proc scan and this read (TOCTOU race), leaving no readable /proc/{pid}/stat.
    push_flag( &mut flags, '⚠' );
  }

  // 🐳 Container: working directory is outside $HOME.
  // Fix(BUG-383): path-component-aware check — cwd is "inside home" only if it
  // equals home exactly, or the byte immediately after the shared prefix is a
  // path separator. A plain `starts_with` wrongly matched a sibling directory
  // like /home/alice2 against home=/home/alice.
  // Root cause: `starts_with` is a byte-sequence test, not a path-component test.
  // Pitfall: never use raw `str::starts_with` to test path descendance — always
  // verify a `/` boundary (or exact equality) after the shared prefix.
  let cwd_str = proc.cwd.to_str().unwrap_or( "" );
  let is_inside_home = !ctx.home.is_empty() && (
    cwd_str == ctx.home
    || cwd_str.strip_prefix( ctx.home ).is_some_and( | rest | rest.starts_with( '/' ) )
  );
  if !ctx.home.is_empty() && !is_inside_home
  {
    push_flag( &mut flags, '🐳' );
  }

  flags
}

// Build the legend line from the collected per-row flag strings.
//
// Only symbols that appeared in at least one row are included, in canonical order.
// Returns an empty string when `flags_per_row` contains no non-empty entries
// (caller should check `any_flags` before calling to avoid the empty-string case).
#[ cfg( target_os = "linux" ) ]
fn build_legend( flags_per_row : &[ String ] ) -> String
{
  let all_flags : String = flags_per_row.concat();
  FLAG_LEGEND.iter()
    .filter( | ( emoji, _ ) | all_flags.contains( *emoji ) )
    .map( | ( emoji, name ) | format!( "{emoji} {name}" ) )
    .collect::< Vec< _ > >()
    .join( "  " )
}

/// Build the "Active Sessions" table for a list of Claude Code processes — the same
/// rendering engine `clr ps` uses, shared here so `claude_version`'s `.ps` produces
/// identical output.
///
/// On Linux, per-process metrics (elapsed/cpu/ram/state) and session flags come from
/// `/proc`; other platforms render those columns as `-` and no flags ever fire. Samples
/// cumulative CPU ticks twice, 1 second apart, to compute the ⚡ Active flag — every
/// call whose (mode/pid-filtered) process list is non-empty blocks for ~1s as a result.
///
/// Returns `None` when the filtered process list is empty — the caller should print
/// its own empty-state message (`clr ps` and `.ps` both use "No active Claude Code
/// sessions."). Returns `Some((table, legend))` where `legend` is `Some(line)` when
/// ≥1 flag fired across all displayed rows, or `None` when all rows are flag-free.
#[ must_use ]
// pub only because the ps_table extraction moved this out of clr's private cli::ps module.
// Single-pass renderer: every column and session flag is derived from one /proc sampling
// window, so splitting it would either re-sample or scatter the row contract across helpers.
#[ allow( clippy::missing_inline_in_public_items, clippy::too_many_lines ) ]
pub fn render_active_sessions_table(
  procs : &[ ProcessInfo ],
  opts  : &PsTableOptions,
) -> Option< ( String, Option< String > ) >
{
  // Apply mode filter before checking emptiness.
  let mode = opts.mode.as_deref().unwrap_or( "all" );
  let mode_filtered : Vec< &ProcessInfo > = if mode == "all"
  {
    procs.iter().collect()
  }
  else
  {
    procs.iter().filter( | p | classify_mode( &p.args ) == mode ).collect()
  };

  // Apply PID filter after mode filter (AND semantics).
  let filtered : Vec< &ProcessInfo > = if opts.pids.is_empty()
  {
    mode_filtered
  }
  else
  {
    mode_filtered.into_iter().filter( | p | opts.pids.contains( &p.pid ) ).collect()
  };

  if filtered.is_empty() { return None; }

  // Two-sample CPU delta pre-pass (1 s window) — `filtered` is non-empty here (checked
  // above), so unlike a caller-side pre-pass over the unfiltered list, this never wastes
  // the 1 s sleep on processes excluded by the mode/pid filter.
  #[ cfg( target_os = "linux" ) ]
  let deltas : HashMap< u32, u64 > =
  {
    let first : HashMap< u32, u64 > = filtered.iter()
      .filter_map( |p| read_cpu_ticks( p.pid ).map( |t| ( p.pid, t ) ) )
      .collect();
    std::thread::sleep( core::time::Duration::from_secs( 1 ) );
    filtered.iter()
      .filter_map( |p|
      {
        let t1 = first.get( &p.pid )?;
        let t2 = read_cpu_ticks( p.pid )?;
        Some( ( p.pid, t2.saturating_sub( *t1 ) ) )
      } )
      .collect()
  };
  #[ cfg( not( target_os = "linux" ) ) ]
  let deltas : HashMap< u32, u64 > = HashMap::new();
  // `deltas` is only consumed inside `#[cfg(target_os = "linux")]` below.
  #[ cfg( not( target_os = "linux" ) ) ]
  let _ = &deltas;

  // Sort oldest-first (AC-012): smallest started_at = longest running = row #1.
  #[ cfg( target_os = "linux" ) ]
  let sorted : Vec< &ProcessInfo > = {
    use crate::process::read_process_metrics;
    let mut v : Vec< &ProcessInfo > = filtered;
    v.sort_by_key( |p| read_process_metrics( p.pid )
      .map_or( u64::MAX, |m| m.started_at ) );
    v
  };
  #[ cfg( not( target_os = "linux" ) ) ]
  let sorted : Vec< &ProcessInfo > = filtered;

  // Pass 1: compute flags per row (Linux only; always empty on other platforms).
  #[ cfg( target_os = "linux" ) ]
  let flags_per_row : Vec< String > = {
    use crate::process::read_process_metrics;
    let home    = std::env::var( "HOME" ).unwrap_or_default();
    let my_ppid : u32 = std::os::unix::process::parent_id();
    let ctx = FlagContext
    {
      home         : &home,
      ancient_secs : opts.ancient_secs,
      high_ram_mb  : opts.high_ram_mb,
      my_ppid,
    };
    sorted.iter().map( | proc |
    {
      let m = read_process_metrics( proc.pid );
      let cpu_delta = deltas.get( &proc.pid ).copied().unwrap_or( 0 );
      let is_new = opts.prior_pids.as_ref().is_some_and( | known | !known.contains( &proc.pid ) );
      compute_flags( proc, m.as_ref(), &ctx, cpu_delta, is_new )
    } ).collect()
  };
  #[ cfg( not( target_os = "linux" ) ) ]
  let flags_per_row : Vec< String > = sorted.iter().map( |_| String::new() ).collect();

  let any_flags = flags_per_row.iter().any( | f | !f.is_empty() );

  let cols : Vec< &'static str > = opts.columns.clone().unwrap_or_else( || DEFAULT_COLUMNS.to_vec() );

  // Find insertion position for the Flags column: immediately after the first anchor
  // present in `cols`.  "state" preserves the historical placement whenever a State
  // column was explicitly requested (--wide / --columns); "ram" is the default view's
  // anchor now that `state` has left DEFAULT_COLUMNS.  The rest cover narrow custom
  // selections, and with none of them present the column goes leftmost rather than
  // vanishing — the previous single "state" lookup yielded None for any selection
  // lacking that key, silently dropping both the flags and their legend.
  const FLAGS_ANCHORS : &[ &str ] = &[ "state", "ram", "cpu", "elapsed", "pid", "idx" ];
  let flags_insert_pos : Option< usize > = if any_flags
  {
    Some(
      FLAGS_ANCHORS.iter()
        .find_map( | anchor | cols.iter().position( | &k | k == *anchor ) )
        .map_or( 0, | p | p + 1 ),
    )
  }
  else
  {
    None
  };

  // Build headers, inserting "Flags" at the anchored position when any flag fired.
  let mut headers : Vec< String > = cols.iter().map( |k|
  {
    COLUMN_KEYS.iter()
      .find( | ( ck, _ ) | ck == k )
      .map_or_else( || ( *k ).to_string(), | ( _, h ) | ( *h ).to_string() )
  } ).collect();
  if let Some( p ) = flags_insert_pos { headers.insert( p, "Flags".to_string() ); }

  // Pass 2: build rows, injecting flags value at insertion position.
  let mut builder = RowBuilder::new( headers );
  for ( ( idx, proc ), flags_str ) in sorted.iter().enumerate().zip( flags_per_row.iter() )
  {
    let mut row = build_row( idx + 1, proc, &cols );
    if let Some( p ) = flags_insert_pos { row.insert( p, flags_str.clone() ); }
    builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
  }

  // Interactive/print breakdown only when unfiltered (AC per task 367) — a mode-filtered
  // view already restricts rows to one mode, so the breakdown would be redundant.
  let running_field = if mode == "all"
  {
    let interactive = sorted.iter().filter( | p | classify_mode( &p.args ) == "interactive" ).count();
    let query       = sorted.iter().filter( | p | classify_mode( &p.args ) == "query" ).count();
    let print       = sorted.len() - interactive - query;
    format!( "{} running ({interactive} interactive, {print} print, {query} query)", sorted.len() )
  }
  else
  {
    format!( "{} running", sorted.len() )
  };
  let heading = Heading::new( "Active Sessions" )
    .with_field( running_field );
  let table_str = render_headed_table( builder, heading );

  // Build legend from flags present across all rows (Linux only).
  #[ cfg( target_os = "linux" ) ]
  let legend = if any_flags { Some( build_legend( &flags_per_row ) ) } else { None };
  #[ cfg( not( target_os = "linux" ) ) ]
  let legend : Option< String > = None;

  Some( ( table_str, legend ) )
}

// Build one table row for the given process, emitting only the requested columns.
fn build_row( idx : usize, proc : &ProcessInfo, cols : &[ &str ] ) -> Vec< String >
{
  let pid = proc.pid;

  #[ cfg( target_os = "linux" ) ]
  let ( elapsed, cpu, ram, state ) =
  {
    use crate::process::read_process_metrics;
    match read_process_metrics( pid )
    {
      Some( m ) => (
        elapsed_label( m.started_at ),
        format!( "{:.1}%", m.cpu_pct ),
        ram_label( m.ram_kb ),
        m.state.to_string(),
      ),
      None => ( "-".to_string(), "-".to_string(), "-".to_string(), "-".to_string() ),
    }
  };

  #[ cfg( not( target_os = "linux" ) ) ]
  let ( elapsed, cpu, ram, state ) =
    ( "-".to_string(), "-".to_string(), "-".to_string(), "-".to_string() );

  let path    = shorten_path( &proc.cwd.display().to_string() );
  let task    = resolve_task( proc );
  let mode    = classify_mode( &proc.args ).to_string();
  let command = proc.args.get( 1.. ).unwrap_or( &[] ).join( " " );
  let binary  = proc.args.first().cloned().unwrap_or_default();

  cols.iter().map( |col| match *col
  {
    "idx"     => idx.to_string(),
    "pid"     => pid.to_string(),
    "elapsed" => elapsed.clone(),
    "cpu"     => cpu.clone(),
    "ram"     => ram.clone(),
    "state"   => state.clone(),
    "path"    => path.clone(),
    "task"    => task.clone(),
    "mode"    => mode.clone(),
    "cmd"     => command.clone(),
    "binary"  => binary.clone(),
    _         => String::new(),
  } ).collect()
}

/// Replace the $PRO prefix in a path with the literal "$PRO" when the PRO env var is set.
///
/// Keeps path strings short in the table without information loss: the user already knows
/// what $PRO expands to. Falls back to the full path when PRO is unset or empty.
#[ must_use ]
// Was a private fn in clr's cli::ps before the ps_table extraction — pub purely to cross the
// crate boundary, not an inlining-sensitive API.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn shorten_path( path : &str ) -> String
{
  if let Ok( pro ) = std::env::var( "PRO" )
  {
    // Fix(BUG-432): raw `starts_with` treated sibling directories whose names share
    // a prefix with `$PRO` (e.g. `$PROtools`) as if they were descendants of `$PRO`,
    // producing garbled paths like `$PROtools`.
    // Root cause: byte-sequence prefix match has no path-boundary awareness; a sibling
    // `/a/pro` and `/a/protools` share the prefix `/a/pro` but are unrelated paths.
    // Pitfall: always check that the remainder after stripping the prefix starts with '/'
    // (or that the path equals PRO exactly); `strip_prefix + is_some_and` mirrors the
    // boundary-aware pattern established for home-dir detection in BUG-383.
    if !pro.is_empty()
      && ( path == pro.as_str()
        || path.strip_prefix( pro.as_str() ).is_some_and( | rest | rest.starts_with( '/' ) ) )
    {
      let rest = &path[ pro.len().. ];
      return format!( "$PRO{rest}" );
    }
  }
  path.to_string()
}

/// Format elapsed seconds since `started_at` as a human-readable duration.
#[ must_use ]
// Was a private fn in clr's cli::ps before the ps_table extraction — pub purely to cross the
// crate boundary, not an inlining-sensitive API.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn elapsed_label( started_at : u64 ) -> String
{
  duration_label( unix_now().saturating_sub( started_at ) )
}

/// Format a raw duration in seconds as a human-readable label (s / m s / h m).
///
/// Extracted from [`elapsed_label`] so a *fixed* duration (e.g. a dead session's
/// lifetime as of when it was last seen, in the "Ended Since Last Check" table)
/// can reuse the same s/m/h formatting without re-deriving "now" — `elapsed_label`
/// itself always measures against the live clock, which is wrong for a PID that
/// no longer exists to re-measure.
#[ must_use ]
// Extracted from elapsed_label during the ps_table move; pub purely to cross the crate
// boundary for the Ended-Since-Last-Check table, not an inlining-sensitive API.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn duration_label( secs : u64 ) -> String
{
  if secs < 60
  {
    format!( "{secs}s" )
  }
  else if secs < 3_600
  {
    let m = secs / 60;
    let s = secs % 60;
    format!( "{m}m {s}s" )
  }
  else
  {
    let h = secs / 3_600;
    let m = ( secs % 3_600 ) / 60;
    format!( "{h}h {m}m" )
  }
}

/// Format RAM in kilobytes as a human-readable label (K or M suffix).
#[ cfg( target_os = "linux" ) ]
#[ must_use ]
// Was a private fn in clr's cli::ps before the ps_table extraction — pub purely to cross the
// crate boundary, not an inlining-sensitive API.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn ram_label( kb : u64 ) -> String
{
  if kb >= 1_024 { format!( "{}M", kb / 1_024 ) }
  else            { format!( "{kb}K" ) }
}

/// Resolve the Task column value for a process, falling back to "interactive".
#[ must_use ]
// Was a private fn in clr's cli::ps before the ps_table extraction — pub purely to cross the
// crate boundary, and it reads a JSONL log, so inlining the wrapper buys nothing.
#[ allow( clippy::missing_inline_in_public_items ) ]
pub fn resolve_task( proc : &ProcessInfo ) -> String
{
  try_jsonl_task( proc ).unwrap_or_else( || "interactive".to_string() )
}

// Try to read the last user message from the session JSONL for this process's CWD.
//
// Returns None if no JSONL is found, the directory does not exist, or parsing fails.
fn try_jsonl_task( proc : &ProcessInfo ) -> Option< String >
{
  let home    = std::env::var( "HOME" ).ok()?;
  let encoded = claude_storage_core::encode_path( &proc.cwd ).ok()?;
  let dir     = std::path::Path::new( &home )
    .join( ".claude" )
    .join( "projects" )
    .join( &encoded );

  // Find the most-recently-modified JSONL file in the project dir.
  let jsonl_path = std::fs::read_dir( &dir )
    .ok()?
    .flatten()
    .filter( | e |
    {
      e.path().extension().and_then( | x | x.to_str() ) == Some( "jsonl" )
    } )
    .max_by_key( | e |
    {
      e.metadata().and_then( | m | m.modified() ).ok()
    } )?
    .path();

  // Scan for the last Form A user line (string `"content"`, not array).
  //
  // Fix(BUG-297): require `"content":"` (string) and exclude `"content":[` (array).
  // Root cause: `.find(|l| l.contains("\"type\":\"user\""))` returned the last user line,
  //   which in any active session is a Form B tool_result with `"content":[...]`, not the
  //   human's question — the old predicate did not distinguish Form A from Form B.
  // Pitfall: tool_result messages have `"type":"user"` but array content; must exclude
  //   `"content":[` to distinguish Form A (human question) from Form B (tool result).
  let content   = std::fs::read_to_string( jsonl_path ).ok()?;
  let last_user = content.lines().rev()
    .find( | l |
      l.contains( r#""type":"user""# )
        && l.contains( r#""content":""# )
        && !l.contains( r#""content":["# )
    )?;

  // Fix(BUG-296): Claude Form A stores human text in `"content":"..."`, not `"text":"..."`.
  // Root cause: old marker used `"text":"..."` (Messages API array-element key), but Form A
  //   serialises the entire human turn as `"content":"<text>"` at the message level.
  // Pitfall: Messages API uses `"text"` inside content arrays; Form A uses `"content"` directly
  //   as a string value — searching for `"text":"..."` always returns None for Form A lines.
  let marker     = r#""content":""#;
  let text_start = last_user.find( marker ).map( | i | i + marker.len() )?;
  let rest       = &last_user[ text_start .. ];
  // Fix(BUG-394): a bare rest.find('"') stops at the first escaped `\"` inside
  // the human's message text (e.g. `He said \"hi\"`), truncating well before the
  // true closing quote. Root cause: no escape-state tracking at this site.
  // Pitfall: never assume user-authored message text cannot contain a literal `"`.
  let text_end   = find_unescaped_quote( rest )?;
  let text       = &rest[ .. text_end ];
  let truncated  : String = text.chars().take( 35 ).collect();
  if truncated.is_empty() { return None; }
  Some( truncated )
}

// Find the byte offset of the next *unescaped* `"` in `s` — the correct way to locate
// the end of a JSON string value, since JSON always represents a literal `"` inside a
// string as the escape sequence `\"`.
//
// Private copy of `claude_runner`'s `cli::summary::find_unescaped_quote` (Fix(BUG-394)):
// this crate cannot depend on `claude_runner` (the dependency runs the other way), so
// `try_jsonl_task`'s escape-aware quote search — needed for the identical reason the
// original fix was made — carries its own copy of this small, pure helper.
fn find_unescaped_quote( s : &str ) -> Option< usize >
{
  let mut escaped = false;
  for ( i, c ) in s.char_indices()
  {
    if escaped { escaped = false; continue; }
    if c == '\\' { escaped = true; continue; }
    if c == '"' { return Some( i ); }
  }
  None
}

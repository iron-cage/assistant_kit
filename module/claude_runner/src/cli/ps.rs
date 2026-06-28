//! `clr ps` — list active Claude Code sessions and queued `clr` waiters in two
//! plain-style tables.

use claude_core::process::{ find_claude_processes, ProcessInfo };
#[ cfg( target_os = "linux" ) ]
use claude_core::process::ProcessMetrics;
use data_fmt::{ RowBuilder, TableFormatter, TableConfig, Heading, Format };

// Runtime configuration for `clr ps`, assembled from env-var defaults (applied
// first) then CLI tokens (which overwrite env values — CLI-wins).
// ancient_secs and high_ram_mb are only read inside #[cfg(target_os = "linux")] blocks.
#[ cfg_attr( not( target_os = "linux" ), allow( dead_code ) ) ]
struct PsConfig
{
  /// Mode filter: `None` or `"all"` = no filter; `"print"` / `"interactive"` = filter rows.
  mode         : Option< String >,
  /// Comma-separated column keys from `--columns`; overrides `--wide` when present.
  columns      : Option< String >,
  /// When `true` and `columns` is `None`: show all 11 columns.
  wide         : bool,
  /// PID filter from `--pid`; empty = show all sessions.
  pids         : Vec< u32 >,
  /// When `true`: emit key:value inspect blocks instead of tables.
  inspect      : bool,
  /// Elapsed-seconds threshold above which the 🕰 (Ancient) flag fires. Default: 28800 (8h).
  ancient_secs : u64,
  /// RAM megabytes threshold above which the 🐘 (High RAM) flag fires. Default: 400 MB.
  high_ram_mb  : u64,
}

// Classify a process's execution mode from its cmdline args.
//
// Returns `"print"` when `--print` or `-p` appears as a discrete argument
// in `args[1..]`; returns `"interactive"` otherwise.
//
// Must use `args` (NUL-split) — NOT `cmdline` (space-joined) — because a path
// component could contain the substring "--print" producing a false positive.
fn classify_mode( args : &[ String ] ) -> &str
{
  if args.iter().any( | a | a == "--print" || a == "-p" )
  {
    "print"
  }
  else
  {
    "interactive"
  }
}

/// Dispatch `clr ps`: list active Claude Code sessions and queued `clr` waiters
/// in two plain-style tables.
///
/// Accepts `--mode`, `--columns`, `--wide`, `--pid`, `--inspect` (and their short forms).
/// Exits 0 with the tables (or inspect blocks, or empty-state message);
/// exits 1 on unknown or invalid arguments.
#[ allow( clippy::too_many_lines ) ]
pub( crate ) fn dispatch_ps( tokens : &[ String ] ) -> !
{
  // Env-var defaults applied first; CLI tokens overwrite (CLI-wins).
  let ( env_mode, env_columns, env_pids, env_ancient_secs, env_high_ram_mb )
    = super::env::apply_ps_env_vars();
  let mut config = PsConfig
  {
    mode         : env_mode,
    columns      : env_columns,
    wide         : false,
    pids         : env_pids,
    inspect      : false,
    ancient_secs : env_ancient_secs,
    high_ram_mb  : env_high_ram_mb,
  };

  let mut i = 1_usize; // tokens[0] is "ps"
  while i < tokens.len()
  {
    match tokens[ i ].as_str()
    {
      "--help" | "-h" | "help" =>
      {
        super::help::print_ps_help();
      }
      "--mode" | "-m" =>
      {
        i += 1;
        if i >= tokens.len()
        {
          eprintln!( "clr ps: `--mode` requires a value (all|interactive|print)" );
          std::process::exit( 1 );
        }
        config.mode = Some( tokens[ i ].clone() );
      }
      "--columns" =>
      {
        i += 1;
        if i >= tokens.len()
        {
          eprintln!( "clr ps: `--columns` requires a value" );
          std::process::exit( 1 );
        }
        config.columns = Some( tokens[ i ].clone() );
      }
      "--wide" | "-w" =>
      {
        config.wide = true;
      }
      "--pid" =>
      {
        i += 1;
        if i >= tokens.len()
        {
          eprintln!( "clr ps: `--pid` requires a value (comma-separated PIDs)" );
          std::process::exit( 1 );
        }
        let csv = tokens[ i ].clone();
        let mut parsed_pids = Vec::new();
        for part in csv.split( ',' )
        {
          let trimmed = part.trim();
          if let Ok( pid ) = trimmed.parse::< u32 >()
          {
            parsed_pids.push( pid );
          }
          else
          {
            eprintln!( "clr ps: `--pid` value `{trimmed}` is not a valid PID; must be a positive integer" );
            std::process::exit( 1 );
          }
        }
        config.pids = parsed_pids;
      }
      "--inspect" | "-i" =>
      {
        config.inspect = true;
      }
      arg =>
      {
        eprintln!( "clr ps: unexpected argument `{arg}`\nRun 'clr ps --help' for usage." );
        std::process::exit( 1 );
      }
    }
    i += 1;
  }

  // Validate mode (from CLI or env var) after all tokens are processed.
  if let Some( ref m ) = config.mode
  {
    if !matches!( m.as_str(), "all" | "interactive" | "print" )
    {
      eprintln!(
        "clr ps: invalid --mode value `{m}`; valid values: all, interactive, print"
      );
      std::process::exit( 1 );
    }
  }

  // Eagerly validate --columns so unknown keys are caught even when no active
  // sessions exist (build_active_table returns None early for empty proc lists).
  if let Some( ref csv ) = config.columns
  {
    if let Err( msg ) = validate_columns( csv )
    {
      eprintln!( "clr ps: {msg}" );
      std::process::exit( 1 );
    }
  }

  let procs = find_claude_processes();

  // Inspect mode: emit key:value blocks instead of tables; suppress queued output.
  if config.inspect
  {
    let mode_str = config.mode.as_deref().unwrap_or( "all" );
    let mode_ok : Vec< &ProcessInfo > = if mode_str == "all"
    {
      procs.iter().collect()
    }
    else
    {
      procs.iter().filter( | p | classify_mode( &p.args ) == mode_str ).collect()
    };
    let filtered : Vec< &ProcessInfo > = if config.pids.is_empty()
    {
      mode_ok
    }
    else
    {
      mode_ok.into_iter().filter( | p | config.pids.contains( &p.pid ) ).collect()
    };
    let output = build_inspect_output( &filtered );
    if output.is_empty()
    {
      println!( "No active Claude Code sessions." );
    }
    else
    {
      println!( "{output}" );
    }
    std::process::exit( 0 );
  }

  // Two-sample CPU delta pre-pass (1 s window).  Skip the 1 s sleep when no
  // processes are found — the table builder returns None immediately anyway.
  #[ cfg( target_os = "linux" ) ]
  let deltas : std::collections::HashMap< u32, u64 > = if procs.is_empty()
  {
    std::collections::HashMap::new()
  }
  else
  {
    let first : std::collections::HashMap< u32, u64 > = procs.iter()
      .filter_map( |p| read_cpu_ticks( p.pid ).map( |t| ( p.pid, t ) ) )
      .collect();
    std::thread::sleep( core::time::Duration::from_secs( 1 ) );
    procs.iter()
      .filter_map( |p|
      {
        let t1 = first.get( &p.pid )?;
        let t2 = read_cpu_ticks( p.pid )?;
        Some( ( p.pid, t2.saturating_sub( *t1 ) ) )
      } )
      .collect()
  };
  #[ cfg( not( target_os = "linux" ) ) ]
  let deltas : std::collections::HashMap< u32, u64 > = std::collections::HashMap::new();

  let active_result = build_active_table( &procs, &config, &deltas );
  let queued_table  = build_queued_table();

  match ( active_result, queued_table )
  {
    ( None, None ) =>
    {
      println!( "No active Claude Code sessions." );
    }
    ( Some( ( at, legend ) ), None ) =>
    {
      println!( "{at}" );
      if let Some( leg ) = legend
      {
        println!();
        println!( "{leg}" );
      }
    }
    ( None, Some( qt ) ) =>
    {
      // Print the "no active sessions" sentinel even when a queued table is
      // present — users need context for WHY processes are waiting rather than
      // seeing a queue table with no explanation of the active-session count.
      println!( "No active Claude Code sessions." );
      println!();
      println!( "{qt}" );
    }
    ( Some( ( at, legend ) ), Some( qt ) ) =>
    {
      println!( "{at}" );
      if let Some( leg ) = legend
      {
        println!();
        println!( "{leg}" );
      }
      println!();
      println!( "{qt}" );
    }
  }
  std::process::exit( 0 );
}

// Render a completed RowBuilder as a headed plain-style table string.
//
// data_fmt ≥0.5.1 fills the heading rule to the rendered table body width
// automatically (TSK-008), so no two-pass probe is required.
// auto_wrap: false — prevents word-wrapping long paths across continuation rows.
fn render_plain_table( builder : RowBuilder, heading : Heading ) -> String
{
  Format::format(
    &TableFormatter::with_config(
      TableConfig::plain()
        .with_heading( heading ),
    ),
    &builder.build_view(),
  ).unwrap_or_default()
}

// All 11 column keys in display order, paired with their table header strings.
const COLUMN_KEYS : &[ ( &str, &str ) ] = &[
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

// Default column set (9 columns) shown when neither `--wide` nor `--columns` is given.
// `mode` is inserted after `state` so session type is visible in the default view.
const DEFAULT_COLUMNS : &[ &str ] = &[
  "idx", "pid", "elapsed", "cpu", "ram", "state", "mode", "path", "task",
];

// Resolve the ordered list of column keys from PsConfig.
//
// Precedence: `--columns` wins over `--wide`; `--wide` enables all 11.
// Returns a vec of `&'static str` keys drawn from `COLUMN_KEYS`.
// Exits 1 with an error message if any key in `--columns` is unknown.
fn resolve_columns( config : &PsConfig ) -> Vec< &'static str >
{
  if let Some( ref csv ) = config.columns
  {
    return match validate_columns( csv )
    {
      Ok( keys ) => keys,
      Err( msg ) =>
      {
        eprintln!( "clr ps: {msg}" );
        std::process::exit( 1 );
      }
    };
  }
  if config.wide
  {
    return COLUMN_KEYS.iter().map( | ( k, _ ) | *k ).collect();
  }
  DEFAULT_COLUMNS.to_vec()
}

// Validate a comma-separated column key string against COLUMN_KEYS.
//
// Returns ordered `&'static str` keys (from the constant — not slices of the
// input) so callers have a stable `'static` lifetime regardless of where the
// input string lives.
fn validate_columns( csv : &str ) -> Result< Vec< &'static str >, String >
{
  let mut out = Vec::new();
  for raw in csv.split( ',' )
  {
    let key = raw.trim();
    if let Some( ( k, _ ) ) = COLUMN_KEYS.iter().find( | ( k, _ ) | *k == key )
    {
      out.push( *k );
    }
    else
    {
      let valid : Vec< &str > = COLUMN_KEYS.iter().map( | ( k, _ ) | *k ).collect();
      return Err( format!(
        "unknown column key `{key}`; valid keys: {}",
        valid.join( ", " )
      ) );
    }
  }
  if out.is_empty()
  {
    let valid : Vec< &str > = COLUMN_KEYS.iter().map( | ( k, _ ) | *k ).collect();
    return Err( format!( "no column keys given; valid keys: {}", valid.join( ", " ) ) );
  }
  Ok( out )
}

// Emit a key:value inspect record for each matching process.
//
// Each block starts with a PID rule line, followed by 12 attribute lines
// (pid, mode, elapsed, cpu, ram, state, path, task, binary, cmd, cmdline, started).
// Blocks are joined by blank lines.  Returns an empty string when `procs` is empty
// so the caller can emit the no-sessions message.
fn build_inspect_output( procs : &[ &ProcessInfo ] ) -> String
{
  use core::fmt::Write as _;
  let mut out = String::new();
  for ( idx, proc ) in procs.iter().enumerate()
  {
    if idx > 0 { out.push( '\n' ); }

    let pid     = proc.pid;
    let mode    = classify_mode( &proc.args ).to_string();
    let path    = shorten_path( &proc.cwd.display().to_string() );
    let task    = resolve_task( proc );
    let binary  = proc.args.first().cloned().unwrap_or_default();
    let cmd     = proc.args.get( 1.. ).unwrap_or( &[] ).join( " " );
    let cmdline = proc.args.join( " " );

    #[ cfg( target_os = "linux" ) ]
    let ( elapsed, cpu, ram, state, started ) =
    {
      use claude_core::process::read_process_metrics;
      match read_process_metrics( pid )
      {
        Some( m ) => (
          elapsed_label( m.started_at ),
          format!( "{:.1}%", m.cpu_pct ),
          ram_label( m.ram_kb ),
          m.state.to_string(),
          m.started_at.to_string(),
        ),
        None => (
          "-".to_string(), "-".to_string(), "-".to_string(),
          "-".to_string(), "-".to_string(),
        ),
      }
    };

    #[ cfg( not( target_os = "linux" ) ) ]
    let ( elapsed, cpu, ram, state, started ) = (
      "-".to_string(), "-".to_string(), "-".to_string(),
      "-".to_string(), "-".to_string(),
    );

    let rule = format!( "──── PID {pid} {}", "─".repeat( 50 ) );
    let _ = writeln!( out, "{rule}" );
    let _ = writeln!( out, "{:<10}{pid}",     "pid:" );
    let _ = writeln!( out, "{:<10}{mode}",    "mode:" );
    let _ = writeln!( out, "{:<10}{elapsed}", "elapsed:" );
    let _ = writeln!( out, "{:<10}{cpu}",     "cpu:" );
    let _ = writeln!( out, "{:<10}{ram}",     "ram:" );
    let _ = writeln!( out, "{:<10}{state}",   "state:" );
    let _ = writeln!( out, "{:<10}{path}",    "path:" );
    let _ = writeln!( out, "{:<10}{task}",    "task:" );
    let _ = writeln!( out, "{:<10}{binary}",  "binary:" );
    let _ = writeln!( out, "{:<10}{cmd}",     "cmd:" );
    let _ = writeln!( out, "{:<10}{cmdline}", "cmdline:" );
    let _ = writeln!( out, "{:<10}{started}", "started:" );
  }
  out.trim_end_matches( '\n' ).to_string()
}

// Per-flag metadata in canonical display order (👈🖨⚡🕰🐘⚠🐳).
// Only used on Linux because compute_flags is Linux-only.
#[ cfg( target_os = "linux" ) ]
const FLAG_LEGEND : &[ ( &str, &str ) ] = &[
  ( "👈", "This session" ),
  ( "🖨",  "Print mode"   ),
  ( "⚡", "Active"       ),
  ( "🕰",  "Ancient"      ),
  ( "🐘", "High RAM"     ),
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
// Flags in canonical order 👈🖨⚡🕰🐘⚠🐳 (only symbols that fire are included).
// Pure computation — no filesystem I/O beyond what the caller already has in `metrics` and
// `cpu_delta_ticks` (both pre-computed by the caller in `dispatch_ps()`).
// The `/proc/{my_ppid}/cmdline` read for 👈 is inexpensive and done once per `clr ps` run.
#[ cfg( target_os = "linux" ) ]
fn push_flag( flags : &mut String, c : char )
{
  if !flags.is_empty() { flags.push( ' ' ); }
  flags.push( c );
}

#[ cfg( target_os = "linux" ) ]
fn compute_flags(
  proc            : &ProcessInfo,
  metrics         : Option< &ProcessMetrics >,
  home            : &str,
  ancient_secs    : u64,
  high_ram_mb     : u64,
  my_ppid         : u32,
  cpu_delta_ticks : u64,
) -> String
{
  let mut flags = String::new();

  // 👈 This session: clr ps is a direct child of this claude process.
  if proc.pid == my_ppid
  {
    // Verify the parent's cmdline arg[0] basename == "claude".
    let is_claude = std::fs::read( format!( "/proc/{my_ppid}/cmdline" ) )
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

  // 🖨 Print mode: cmdline contains --print or -p.
  if classify_mode( &proc.args ) == "print" { push_flag( &mut flags, '🖨' ); }

  // ⚡ Active: CPU delta >= 3 ticks in 1-second sample window.
  // Threshold separates active sessions (6-100 ticks) from BUG-304 timer noise (1-2 ticks).
  if cpu_delta_ticks >= 3 { push_flag( &mut flags, '⚡' ); }

  if let Some( m ) = metrics
  {
    // 🕰 Ancient: elapsed seconds exceed the configured threshold.
    let elapsed = super::gate::unix_now().saturating_sub( m.started_at );
    if elapsed > ancient_secs { push_flag( &mut flags, '🕰' ); }

    // 🐘 High RAM: RSS exceeds threshold. Comparison in KB to avoid integer-division
    //   truncation (e.g. 512 KB / 1024 = 0 MB, which would never exceed a 0 MB threshold).
    if m.ram_kb > high_ram_mb.saturating_mul( 1_024 ) { push_flag( &mut flags, '🐘' ); }
  }
  else
  {
    // ⚠ Dead metrics: read_process_metrics returned None (TOCTOU race or zombie).
    push_flag( &mut flags, '⚠' );
  }

  // 🐳 Container: working directory is outside $HOME.
  let cwd_str = proc.cwd.to_str().unwrap_or( "" );
  if !home.is_empty() && !cwd_str.starts_with( home )
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

// Build the active sessions table, returning None when no sessions match.
//
// Returns `Some((table_string, legend))` where `legend` is `Some(line)` when ≥1 flag
// fired across all displayed rows, or `None` when all rows are flag-free.
// The caller must print the legend after the active table (separated by a blank line).
fn build_active_table(
  procs  : &[ ProcessInfo ],
  config : &PsConfig,
  deltas : &std::collections::HashMap< u32, u64 >,
) -> Option< ( String, Option< String > ) >
{
  // Apply mode filter before checking emptiness.
  let mode = config.mode.as_deref().unwrap_or( "all" );
  let mode_filtered : Vec< &ProcessInfo > = if mode == "all"
  {
    procs.iter().collect()
  }
  else
  {
    procs.iter().filter( | p | classify_mode( &p.args ) == mode ).collect()
  };

  // Apply PID filter after mode filter (AND semantics).
  let filtered : Vec< &ProcessInfo > = if config.pids.is_empty()
  {
    mode_filtered
  }
  else
  {
    mode_filtered.into_iter().filter( | p | config.pids.contains( &p.pid ) ).collect()
  };

  if filtered.is_empty() { return None; }

  // `deltas` is only consumed inside `#[cfg(target_os = "linux")]` below.
  #[ cfg( not( target_os = "linux" ) ) ]
  let _ = &deltas;

  // Sort oldest-first (AC-012): smallest started_at = longest running = row #1.
  #[ cfg( target_os = "linux" ) ]
  let sorted : Vec< &ProcessInfo > = {
    use claude_core::process::read_process_metrics;
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
    use claude_core::process::read_process_metrics;
    let home    = std::env::var( "HOME" ).unwrap_or_default();
    let my_ppid : u32 = std::os::unix::process::parent_id();
    sorted.iter().map( | proc |
    {
      let m = read_process_metrics( proc.pid );
      let cpu_delta = deltas.get( &proc.pid ).copied().unwrap_or( 0 );
      compute_flags( proc, m.as_ref(), &home, config.ancient_secs, config.high_ram_mb, my_ppid, cpu_delta )
    } ).collect()
  };
  #[ cfg( not( target_os = "linux" ) ) ]
  let flags_per_row : Vec< String > = sorted.iter().map( |_| String::new() ).collect();

  let any_flags = flags_per_row.iter().any( | f | !f.is_empty() );

  let cols = resolve_columns( config );

  // Find insertion position for the Flags column — immediately after "state".
  let flags_insert_pos : Option< usize > = if any_flags
  {
    cols.iter().position( | &k | k == "state" ).map( | p | p + 1 )
  }
  else
  {
    None
  };

  // Build headers, inserting "Flags" after "State" when any flag fired.
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

  let heading = Heading::new( "Active Sessions" )
    .with_field( format!( "{} running", sorted.len() ) );
  let table_str = render_plain_table( builder, heading );

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
    use claude_core::process::read_process_metrics;
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

// Replace the $PRO prefix in a path with the literal "$PRO" when the PRO env var is set.
//
// Keeps path strings short in the table without information loss: the user already knows
// what $PRO expands to. Falls back to the full path when PRO is unset or empty.
fn shorten_path( path : &str ) -> String
{
  if let Ok( pro ) = std::env::var( "PRO" )
  {
    if !pro.is_empty() && path.starts_with( pro.as_str() )
    {
      let rest = &path[ pro.len().. ];
      return format!( "$PRO{rest}" );
    }
  }
  path.to_string()
}

// Format elapsed seconds since `started_at` as a human-readable duration.
fn elapsed_label( started_at : u64 ) -> String
{
  let elapsed = super::gate::unix_now().saturating_sub( started_at );
  if elapsed < 60
  {
    format!( "{elapsed}s" )
  }
  else if elapsed < 3_600
  {
    let m = elapsed / 60;
    let s = elapsed % 60;
    format!( "{m}m {s}s" )
  }
  else
  {
    let h = elapsed / 3_600;
    let m = ( elapsed % 3_600 ) / 60;
    format!( "{h}h {m}m" )
  }
}

// Format RAM in kilobytes as a human-readable label (K or M suffix).
#[ cfg( target_os = "linux" ) ]
fn ram_label( kb : u64 ) -> String
{
  if kb >= 1_024 { format!( "{}M", kb / 1_024 ) }
  else            { format!( "{kb}K" ) }
}

// Resolve the Task column value for a process, falling back to "interactive".
fn resolve_task( proc : &ProcessInfo ) -> String
{
  try_jsonl_task( proc ).unwrap_or_else( || "interactive".to_string() )
}

// Try to read the last user message from the session JSONL for this process's CWD.
//
// Returns None if no JSONL is found, the directory does not exist, or parsing fails.
fn try_jsonl_task( proc : &ProcessInfo ) -> Option< String >
{
  let home    = std::env::var( "HOME" ).ok()?;
  let cwd_str = proc.cwd.to_str()?;

  // Fix(BUG-295): Claude encodes both `/` and `_` as `-` in project directory names.
  // Root cause: the original `replace('/', "-")` only handled slashes; underscore-
  //   containing paths produced a wrong encoded key, so the JSONL dir was never found.
  // Pitfall: Claude's encoding maps both `/` and `_` to `-` in one pass, producing a
  //   flat lowercase-hyphen-only directory name.
  let encoded = cwd_str.replace( [ '/', '_' ], "-" );
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
  let text_end   = rest.find( '"' )?;
  let text       = &rest[ .. text_end ];
  let truncated  : String = text.chars().take( 35 ).collect();
  if truncated.is_empty() { return None; }
  Some( truncated )
}

// Extract a string value for `key` from a compact JSON object in `content`.
fn parse_json_str( content : &str, key : &str ) -> Option< String >
{
  let marker = format!( r#""{key}":""# );
  let start  = content.find( marker.as_str() )? + marker.len();
  let rest   = &content[ start.. ];
  let end    = rest.find( '"' )?;
  Some( rest[ ..end ].to_string() )
}

// Extract a u64 value for `key` from a compact JSON object in `content`.
fn parse_json_u64( content : &str, key : &str ) -> Option< u64 >
{
  let marker = format!( r#""{key}":"# );
  let start  = content.find( marker.as_str() )? + marker.len();
  let rest   = &content[ start.. ];
  let end    = rest.find( [ ',', '}' ] )?;
  rest[ ..end ].trim().parse().ok()
}

// Read the gate state dir and build the queued CLR processes table.
//
// Returns None when the gate dir is absent or contains no .json files.
//
// JSON parsing is manual (no serde) to keep dependencies minimal.  Gate files
// are written by gate.rs using format!(), so the only structural constraint is
// that `cwd` must not contain a literal `"` character — Unix paths never do,
// so substring extraction in parse_json_str is safe in practice.
fn build_queued_table() -> Option< String >
{
  let dir = super::gate::gate_dir();
  let mut entries : Vec< _ > = std::fs::read_dir( &dir )
    .ok()?
    .flatten()
    .filter( |e|
    {
      if e.path().extension().and_then( |x| x.to_str() ) != Some( "json" )
      {
        return false;
      }
      // Fix(BUG-293): Liveness filter for gate files.
      // Root cause: build_queued_table() rendered all gate files without checking
      // if the owning PID still existed, displaying SIGKILL/crash orphans as
      // perpetual phantom waiters.
      // Pitfall: /proc/{pid} is Linux-specific; this entire module is
      // #[cfg(target_os = "linux")] so the path is guaranteed to exist for live PIDs.
      let alive = e.path()
        .file_stem()
        .and_then( |s| s.to_str() )
        .and_then( |s| s.parse::< u32 >().ok() )
        .is_some_and( |pid|
        {
          std::path::Path::new( &format!( "/proc/{pid}" ) ).exists()
        } );
      if !alive
      {
        // Self-heal: remove the orphaned gate file so it doesn't recur.
        let _ = std::fs::remove_file( e.path() );
      }
      alive
    } )
    .collect();

  if entries.is_empty() { return None; }

  let count = entries.len();

  // Sort by numeric PID for intuitive output order; string sort mis-orders "1000" < "200".
  entries.sort_by_key( |e|
  {
    e.path()
     .file_stem()
     .and_then( |s| s.to_str() )
     .and_then( |s| s.parse::< u32 >().ok() )
     .unwrap_or( u32::MAX )
  } );

  let headers = vec![
    "#".to_string(),
    "PID".to_string(),
    "CWD".to_string(),
    "Waiting".to_string(),
    "Attempt".to_string(),
  ];

  let mut builder = RowBuilder::new( headers );
  for ( idx, entry ) in entries.iter().enumerate()
  {
    let path    = entry.path();
    let pid_str = path
      .file_stem()
      .and_then( |s| s.to_str() )
      .unwrap_or( "?" )
      .to_string();
    let content = std::fs::read_to_string( &path ).unwrap_or_default();
    let cwd     = parse_json_str( &content, "cwd" ).unwrap_or_default();
    let since   = parse_json_u64( &content, "since" ).unwrap_or( 0 );
    let attempt = parse_json_u64( &content, "attempt" ).unwrap_or( 0 );
    let row     = vec![
      ( idx + 1 ).to_string(),
      pid_str,
      shorten_path( &cwd ),
      elapsed_label( since ),
      attempt.to_string(),
    ];
    builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
  }

  let heading = Heading::new( "Queued" )
    .with_field( format!( "{count} waiting" ) );
  Some( render_plain_table( builder, heading ) )
}

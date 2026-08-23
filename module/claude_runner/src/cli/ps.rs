//! `clr ps` — list active Claude Code sessions and queued `clr` waiters in two
//! plain-style tables.
//!
//! Active-sessions table rendering is shared with `claude_version`'s `.ps` command
//! via `claude_runner_core::ps_table` — the single definition both CLIs render from.

use claude_core::process::{ find_claude_processes, ProcessInfo };
use claude_runner_core::ps_table::
{
  classify_mode, elapsed_label, parse_json_u64, render_active_sessions_table,
  render_headed_table, resolve_task, shorten_path, PsTableOptions, COLUMN_KEYS,
  DEFAULT_COLUMNS,
};
#[ cfg( target_os = "linux" ) ]
use claude_runner_core::ps_table::ram_label;
use data_fmt::{ RowBuilder, Heading };

// Runtime configuration for `clr ps`, assembled from env-var defaults (applied
// first) then CLI tokens (which overwrite env values — CLI-wins).
// ancient_secs and high_ram_mb are only read inside #[cfg(target_os = "linux")] blocks.
#[ cfg_attr( not( target_os = "linux" ), allow( dead_code ) ) ]
struct PsConfig
{
  /// Mode filter: `None` or `"all"` = no filter; `"print"` / `"interactive"` / `"query"` = filter rows.
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

/// Dispatch `clr ps`: list active Claude Code sessions and queued `clr` waiters
/// in two plain-style tables.
///
/// Accepts `--mode`, `--columns`, `--wide`, `--pid`, `--inspect` (and their short forms).
/// Exits 0 with the tables (or inspect blocks, or empty-state message);
/// exits 1 on unknown or invalid arguments.
#[ allow( clippy::too_many_lines ) ] // mechanical dispatch — one arm per CLI flag
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
          eprintln!( "clr ps: `--mode` requires a value (all|interactive|print|query)" );
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
    if !matches!( m.as_str(), "all" | "interactive" | "print" | "query" )
    {
      eprintln!(
        "clr ps: invalid --mode value `{m}`; valid values: all, interactive, print, query"
      );
      std::process::exit( 1 );
    }
  }

  // Eagerly validate --columns so unknown keys are caught even when no active
  // sessions exist (render_active_sessions_table returns None early for empty proc lists).
  if let Some( ref csv ) = config.columns
  {
    if let Err( msg ) = super::column_validate::validate_columns( csv, COLUMN_KEYS )
    {
      eprintln!( "clr ps: {msg}" );
      std::process::exit( 1 );
    }
  }

  let procs = find_claude_processes();

  // Snapshot comparison: read the PRIOR state before it's overwritten below.
  // Always built from the unfiltered `procs` list — a `--pid`/`--mode`-filtered
  // snapshot write would make every untracked session look "ended" on the next
  // unfiltered call.
  let prior_snapshot = super::ps_snapshot::read_snapshot();

  // Inspect mode: emit key:value blocks instead of tables; suppress queued output.
  // Still refreshes the snapshot (silently) so a later non-inspect call has
  // accurate 🆕/Ended data — inspect mode already suppresses the Queued table
  // for the same "different, denser format" reason, so suppressing 🆕/Ended
  // display here (while still tracking state) is consistent, not a new carve-out.
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
    super::ps_snapshot::write_snapshot( &procs );
    std::process::exit( 0 );
  }

  let resolved_columns = resolve_columns( &config );
  let prior_pids = prior_snapshot.as_ref().map( | snap | snap.pid_set() );
  let opts = PsTableOptions
  {
    mode         : config.mode,
    columns      : Some( resolved_columns ),
    pids         : config.pids,
    ancient_secs : config.ancient_secs,
    high_ram_mb  : config.high_ram_mb,
    prior_pids,
  };
  let active_result = render_active_sessions_table( &procs, &opts );
  let queued_table   = build_queued_table();
  let ended_table    = super::ps_snapshot::build_ended_table( &procs, prior_snapshot.as_ref() );
  super::ps_snapshot::write_snapshot( &procs );

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
  if let Some( et ) = ended_table
  {
    println!();
    println!( "{et}" );
  }
  std::process::exit( 0 );
}

// Resolve the ordered list of column keys from PsConfig.
//
// Precedence: `--columns` wins over `--wide`; `--wide` enables all 11.
// Returns a vec of `&'static str` keys drawn from `COLUMN_KEYS`.
// Exits 1 with an error message if any key in `--columns` is unknown.
fn resolve_columns( config : &PsConfig ) -> Vec< &'static str >
{
  if let Some( ref csv ) = config.columns
  {
    return match super::column_validate::validate_columns( csv, COLUMN_KEYS )
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

// Extract a string value for `key` from a compact JSON object in `content`.
//
// Fix(BUG-394): this is the unpaired read side of a round-trip whose write side
// (gate.rs's json_escape_str(), Fix(BUG-384)) already escapes `cwd` correctly —
// a bare rest.find('"') stopped at the first escaped quote instead of reversing
// that escaping, silently truncating a quote-containing cwd in the "Queued CLR
// Processes" table. Root cause: no escape-state tracking at this site. Pitfall:
// fixing a JSON round-trip's write side does not imply the read side correctly
// reverses it — each direction must be independently verified.
fn parse_json_str( content : &str, key : &str ) -> Option< String >
{
  let marker = format!( r#""{key}":""# );
  let start  = content.find( marker.as_str() )? + marker.len();
  let rest   = &content[ start.. ];
  let end    = super::summary::find_unescaped_quote( rest )?;
  Some( rest[ ..end ].to_string() )
}

// Read the gate state dir and build the queued CLR processes table.
//
// Returns None when the gate dir is absent or contains no .json files.
//
// JSON parsing is manual (no serde) to keep dependencies minimal.  Gate files
// are written by gate.rs using format!(), and the writer JSON-escapes `cwd`
// (Fix(BUG-384)), so substring extraction in parse_json_str is safe even when
// `cwd` contains a literal `"` character.
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
      // Fix(BUG-387-followup): slot_*.json reservation files are a distinct
      // Domain Type from the {pid}.json queued-waiter telemetry this table
      // displays — they record an already-ADMITTED session, not one still
      // queued. gate_slot.rs::acquire_slot() already owns their entire lifecycle
      // (claim, liveness check, reclaim) independently. Skip them here
      // untouched: the liveness filter below parses the *filename* as a PID,
      // which always fails for "slot_N", so treating them as unparseable
      // dead gate files would self-heal-delete a live holder's reservation,
      // reopening the exact check-then-reserve race BUG-387 closed.
      if e.path().file_stem().and_then( |s| s.to_str() ).is_some_and( |s| s.starts_with( "slot_" ) )
      {
        return false;
      }
      // Fix(BUG-293): Liveness filter for gate files.
      // Root cause: build_queued_table() rendered all gate files without checking
      // if the owning PID still existed, displaying SIGKILL/crash orphans as
      // perpetual phantom waiters.
      // Pitfall: /proc/{pid} is Linux-specific; this entire module is
      // #[cfg(target_os = "linux")] so the path is guaranteed to exist for live PIDs.
      //
      // Fix(BUG-479) task/claude_runner/bug/479_zombie_blind_pid_liveness.md: the probe here was bare
      // /proc/{pid} existence, which reads exited-but-unreaped (zombie) waiters as
      // alive — the self-heal below never fired for them (`Queued · 84 waiting`
      // with 4 live). Now delegates to the shared zombie-aware predicate.
      // Root cause: liveness convention duplicated inline instead of shared —
      // both copies were existence-only, blind to state `Z`.
      // Pitfall: a /proc/{pid} entry proves a PID exists, not that a process
      // runs; one authoritative predicate (gate_liveness::pid_alive) for every consumer.
      // Fix(BUG-488): pass the waiter record's own starttime (absent in
      // legacy files → None) so display liveness applies the same incarnation
      // binding as slot reclaim — a thread-masked or recycled PID number no
      // longer renders a dead waiter as a phantom queued row. Full fix
      // comment at gate_liveness.rs::pid_alive().
      let alive = e.path()
        .file_stem()
        .and_then( |s| s.to_str() )
        .and_then( |s| s.parse::< u32 >().ok() )
        .is_some_and( | pid |
        {
          let recorded_starttime = std::fs::read_to_string( e.path() )
            .ok()
            .and_then( | content | parse_json_u64( &content, "starttime" ) );
          super::gate_liveness::pid_alive( pid, recorded_starttime )
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
  Some( render_headed_table( builder, heading ) )
}

//! `clr ps` — list active Claude Code sessions and queued `clr` waiters in two
//! plain-style tables.

use claude_core::process::{ find_claude_processes, ProcessInfo };
use data_fmt::{ RowBuilder, TableFormatter, TableConfig, TableCaption, Format };

/// Dispatch `clr ps`: list active Claude Code sessions and queued `clr` waiters
/// in two plain-style tables.
///
/// Accepts no arguments.  Exits 0 with the tables (or empty-state messages);
/// exits 1 on any unexpected argument.
pub( crate ) fn dispatch_ps( tokens : &[ String ] ) -> !
{
  // `ps` takes no flags or positional arguments — but intercepts help tokens.
  if let Some( arg ) = tokens.get( 1 )
  {
    match arg.as_str()
    {
      "--help" | "-h" | "help" => { super::help::print_ps_help(); }
      _ =>
      {
        eprintln!( "clr ps: unexpected argument `{arg}`\nRun 'clr --help' for usage." );
        std::process::exit( 1 );
      }
    }
  }

  let procs        = find_claude_processes();
  let active_table = build_active_table( &procs );
  let queued_table = build_queued_table();

  match ( active_table, queued_table )
  {
    ( None, None ) =>
    {
      println!( "No active Claude Code sessions." );
    }
    ( Some( at ), None ) =>
    {
      println!( "{at}" );
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
    ( Some( at ), Some( qt ) ) =>
    {
      println!( "{at}" );
      println!();
      println!( "{qt}" );
    }
  }
  std::process::exit( 0 );
}

// Render a completed RowBuilder as a captioned plain-style table string.
//
// auto_wrap: false — prevents word-wrapping long paths across continuation rows;
// table width reflects content naturally (user scrolls if needed).
fn render_plain_table( builder : RowBuilder, caption : TableCaption ) -> String
{
  let view = builder.build_view();
  Format::format(
    &TableFormatter::with_config(
      TableConfig::plain()
        .auto_wrap( false )
        .caption( caption )
    ),
    &view,
  ).unwrap_or_default()
}

// Build the active sessions table, returning None when no sessions are running.
fn build_active_table( procs : &[ ProcessInfo ] ) -> Option< String >
{
  if procs.is_empty() { return None; }

  let headers = vec![
    "#".to_string(),
    "PID".to_string(),
    "Elapsed".to_string(),
    "CPU%".to_string(),
    "RAM".to_string(),
    "State".to_string(),
    "Absolute Path".to_string(),
    "Task".to_string(),
  ];

  let mut builder = RowBuilder::new( headers );
  for ( idx, proc ) in procs.iter().enumerate()
  {
    let row = build_row( idx + 1, proc );
    builder = builder.add_row( row.into_iter().map( Into::into ).collect() );
  }

  let caption = TableCaption::new( "Active Sessions" )
    .field( format!( "{} running", procs.len() ) );
  Some( render_plain_table( builder, caption ) )
}

// Build one table row for the given process.
fn build_row( idx : usize, proc : &ProcessInfo ) -> Vec< String >
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

  let path = shorten_path( &proc.cwd.display().to_string() );
  let task = resolve_task( proc );

  vec![ idx.to_string(), pid.to_string(), elapsed, cpu, ram, state, path, task ]
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
  // Fix(BUG-297): the old predicate `.find(| l | l.contains(r#""type":"user""#))`
  //   returned the last `"type":"user"` line, which in any active session is a
  //   Form B tool_result entry with `"content":[...]` — not the human's question.
  // Fix: additionally require `"content":"` (string value) and exclude `"content":[`
  //   (array value). Form B always serialises the outer content as a JSON array.
  let content   = std::fs::read_to_string( jsonl_path ).ok()?;
  let last_user = content.lines().rev()
    .find( | l |
      l.contains( r#""type":"user""# )
        && l.contains( r#""content":""# )
        && !l.contains( r#""content":["# )
    )?;

  // Fix(BUG-296): Claude's Form A stores the human message in `"content":"..."`, not
  //   `"text":"..."`. The old marker matched nothing, silently returning `None`.
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

  let caption = TableCaption::new( "Queued" )
    .field( format!( "{count} waiting" ) );
  Some( render_plain_table( builder, caption ) )
}

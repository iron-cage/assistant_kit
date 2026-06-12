//! `clr ps` — list running Claude Code sessions in a unicode-box table.

use claude_core::process::{ find_claude_processes, ProcessInfo };
use data_fmt::{ RowBuilder, TableFormatter, TableConfig, Format };

/// Dispatch `clr ps`: list running Claude Code sessions in a unicode-box table.
///
/// Accepts no arguments.  Exits 0 with a unicode-box table when sessions are
/// found; exits 0 with "No active Claude Code sessions." when none are running;
/// exits 1 on any unexpected argument.
pub( crate ) fn dispatch_ps( tokens : &[ String ] ) -> !
{
  // `ps` takes no flags or positional arguments.
  if let Some( tok ) = tokens.get( 1 )
  {
    eprintln!( "Error: unexpected argument: {tok}\nRun 'clr --help' for usage." );
    std::process::exit( 1 );
  }

  let procs = find_claude_processes();

  if procs.is_empty()
  {
    println!( "No active Claude Code sessions." );
    std::process::exit( 0 );
  }

  let headers = vec![
    "#".to_string(),
    "PID".to_string(),
    "Started".to_string(),
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

  let view  = builder.build_view();
  let table = Format::format(
    &TableFormatter::with_config( TableConfig::unicode_box() ),
    &view,
  ).unwrap_or_default();

  println!( "{table}" );
  std::process::exit( 0 );
}

// Build one table row for the given process.
fn build_row( idx : usize, proc : &ProcessInfo ) -> Vec< String >
{
  let pid = proc.pid;

  #[ cfg( target_os = "linux" ) ]
  let ( started, cpu, ram, state ) =
  {
    use claude_core::process::read_process_metrics;
    match read_process_metrics( pid )
    {
      Some( m ) => (
        ts_hhmm( m.started_at ),
        format!( "{:.1}%", m.cpu_pct ),
        ram_label( m.ram_kb ),
        m.state.to_string(),
      ),
      None => ( "-".to_string(), "-".to_string(), "-".to_string(), "-".to_string() ),
    }
  };

  #[ cfg( not( target_os = "linux" ) ) ]
  let ( started, cpu, ram, state ) =
    ( "-".to_string(), "-".to_string(), "-".to_string(), "-".to_string() );

  let path = proc.cwd.display().to_string();
  let task = resolve_task( proc );

  vec![ idx.to_string(), pid.to_string(), started, cpu, ram, state, path, task ]
}

// Convert a Unix timestamp to a UTC "HH:MM" string.
fn ts_hhmm( ts : u64 ) -> String
{
  let s_in_day = ts % 86_400;
  let h        = s_in_day / 3_600;
  let m        = ( s_in_day % 3_600 ) / 60;
  format!( "{h:02}:{m:02}" )
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
  let encoded = cwd_str.replace( '/', "-" );
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

  // Scan for the last line containing `"type":"user"`.
  let content   = std::fs::read_to_string( jsonl_path ).ok()?;
  let last_user = content.lines().rev()
    .find( | l | l.contains( r#""type":"user""# ) )?;

  // Extract the `"text":"..."` value with a simple substring search.
  let marker     = r#""text":""#;
  let text_start = last_user.find( marker ).map( | i | i + marker.len() )?;
  let rest       = &last_user[ text_start .. ];
  let text_end   = rest.find( '"' )?;
  let text       = &rest[ .. text_end ];
  let truncated  : String = text.chars().take( 35 ).collect();
  if truncated.is_empty() { return None; }
  Some( truncated )
}

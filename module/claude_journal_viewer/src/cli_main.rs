//! CLR Journal Viewer CLI — dispatches `.command key::value` arguments to journal handlers.
#![allow(missing_docs)]
// `clj` binary — CLR Journal Viewer CLI.
//
// Dispatches `.command key::value` arguments to one of eight journal handlers.
// Journal directory resolution priority: `dir::` param > CLR_JOURNAL_DIR env >
// `~/.clr/journal/` (default).
//
// Shared output logic lives in `claude_journal_viewer::output` (accessible by
// both this binary and the unilang assistant routines in `routines.rs`).

use claude_journal_viewer::output::{ bold, build_filter, format_event_row, resolve_journal_dir };
use claude_journal::{ JournalFilter, JournalReader };
use std::{ collections::HashMap, path::PathBuf };

// ── Embedded web dashboard ────────────────────────────────────────────────────

const INDEX_HTML : &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>CLR Journal Viewer</title>
<style>
  body { font-family: monospace; padding: 1em; background: #1a1a1a; color: #ddd; }
  h1   { color: #7fc; margin: 0 0 .5em; }
  p    { margin: .2em 0 .8em; color: #999; font-size:.9em; }
  table{ border-collapse: collapse; width: 100%; }
  th   { background: #333; padding: 4px 8px; text-align: left; }
  td   { padding: 3px 8px; border-bottom: 1px solid #2a2a2a; }
  tr:hover { background: #252525; }
</style>
</head>
<body>
<h1>CLR Journal</h1>
<p id="status">Loading…</p>
<table>
  <thead><tr>
    <th>Time</th><th>Type</th><th>Cmd</th><th>Model</th>
    <th>Exit</th><th>Cost</th><th>Dur</th>
  </tr></thead>
  <tbody id="rows"></tbody>
</table>
<script>
function fmt(v){ return v==null||v===undefined?'-':v; }
function fmtCost(c){ return c==null?'-':'$'+c.toFixed(4); }
function fmtDur(ms){ return ms==null?'-':ms<1000?ms+'ms':(ms/1000).toFixed(1)+'s'; }
function load(){
  fetch('/api/events').then(r=>r.json()).then(evs=>{
    document.getElementById('status').textContent=evs.length+' event(s) — auto-refresh 5s';
    document.getElementById('rows').innerHTML=evs.slice().reverse().map(e=>
      '<tr><td>'+fmt(e.ts?e.ts.slice(0,16):null)+'</td>'
      +'<td>'+fmt(e.type)+'</td>'
      +'<td>'+fmt(e.command)+'</td>'
      +'<td>'+fmt(e.model)+'</td>'
      +'<td>'+fmt(e.exit_code)+'</td>'
      +'<td>'+fmtCost(e.cost_usd)+'</td>'
      +'<td>'+fmtDur(e.duration_ms)+'</td></tr>'
    ).join('');
  }).catch(()=>{ document.getElementById('status').textContent='Error loading events'; });
}
load();
setInterval(load,5000);
</script>
</body>
</html>"#;

// ── Argument parsing ──────────────────────────────────────────────────────────

/// Parse `key::value` argument pairs into a lookup map.
///
/// Arguments that do not contain `::` are silently skipped.
#[ must_use ]
fn parse_params( args : &[ String ] ) -> HashMap< String, String >
{
  let mut map = HashMap::new();
  for arg in args
  {
    if let Some( pos ) = arg.find( "::" )
    {
      map.insert( arg[ ..pos ].to_owned(), arg[ pos + 2.. ].to_owned() );
    }
  }
  map
}

// ── Command handlers ──────────────────────────────────────────────────────────

/// `.list` — display a filtered event table (default: last 50 events).
fn cmd_list( params : &HashMap< String, String >, dir : PathBuf )
{
  match claude_journal_viewer::output::list_output( params, dir )
  {
    Ok( s )  => println!( "{s}" ),
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  }
}

/// `.tail` — follow journal events in real-time (blocking; Ctrl+C to stop).
fn cmd_tail( params : &HashMap< String, String >, dir : PathBuf )
{
  let filter = match build_filter( params )
  {
    Ok( f )  => f,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  let reader = JournalReader::open( dir );
  eprintln!( "Tailing journal — press Ctrl+C to stop" );
  for ev in reader.tail( &filter )
  {
    println!( "{}", format_event_row( &ev ) );
  }
}

/// `.stats` — aggregate statistics by `by::day` or `by::model`.
fn cmd_stats( params : &HashMap< String, String >, dir : PathBuf )
{
  match claude_journal_viewer::output::stats_output( params, dir )
  {
    Ok( s )  => println!( "{s}" ),
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  }
}

/// `.search` — substring search across event messages, stdout, and stderr.
fn cmd_search( params : &HashMap< String, String >, dir : PathBuf )
{
  match claude_journal_viewer::output::search_output( params, dir )
  {
    Ok( s )  => println!( "{s}" ),
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  }
}

/// `.prune` — delete journal files older than `keep::` duration.
fn cmd_prune( params : &HashMap< String, String >, dir : PathBuf )
{
  match claude_journal_viewer::output::prune_output( params, dir )
  {
    Ok( s )  => println!( "{s}" ),
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  }
}

/// `.status` — show journal health: file count, total size, oldest/newest dates.
fn cmd_status( _params : &HashMap< String, String >, dir : PathBuf )
{
  println!( "{}", claude_journal_viewer::output::status_output( dir ) );
}

/// `.export` — export filtered events to a file.
fn cmd_export( params : &HashMap< String, String >, dir : PathBuf )
{
  match claude_journal_viewer::output::export_output( params, dir )
  {
    Ok( s )  => println!( "{s}" ),
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  }
}

/// `.serve` — start an embedded HTTP server for web-based journal viewing.
fn cmd_serve( params : &HashMap< String, String >, dir : PathBuf )
{
  let port_str = params
    .get( "port" )
    .cloned()
    .unwrap_or_else( || std::env::var( "CLJ_PORT" ).unwrap_or_else( | _ | "0".to_owned() ) );
  let port : u16 = port_str.parse().unwrap_or( 0 );
  let addr       = format!( "127.0.0.1:{port}" );
  let server     = match tiny_http::Server::http( &addr )
  {
    Ok( s )  => s,
    Err( e ) => { eprintln!( "Error: could not start server on {addr}: {e}" ); std::process::exit( 1 ); }
  };
  let actual_port = server.server_addr().to_ip().map_or( port, | a | a.port() );
  println!( "Listening on http://localhost:{actual_port}" );
  // Flush stdout so piped readers (e.g. integration test harness) see the port immediately.
  std::io::Write::flush( &mut std::io::stdout() ).ok();

  let reader = JournalReader::open( dir );
  loop
  {
    let Ok( req ) = server.recv() else { continue; };
    let url = req.url().to_owned();
    if url.starts_with( "/api/events" )
    {
      let filter = JournalFilter { limit : Some( 200 ), ..JournalFilter::default() };
      let events = reader.query( &filter );
      let body   = serde_json::to_string( &events ).unwrap_or_else( | _ | "[]".to_owned() );
      let resp   = tiny_http::Response::from_string( body )
        .with_header(
          "Content-Type: application/json"
            .parse::< tiny_http::Header >()
            .expect( "static Content-Type header is valid ASCII" ),
        );
      let _ = req.respond( resp );
    }
    else
    {
      let resp = tiny_http::Response::from_string( INDEX_HTML )
        .with_header(
          "Content-Type: text/html; charset=utf-8"
            .parse::< tiny_http::Header >()
            .expect( "static Content-Type header is valid ASCII" ),
        );
      let _ = req.respond( resp );
    }
  }
}

// ── Help ──────────────────────────────────────────────────────────────────────

/// Print usage help to stdout.
fn print_help()
{
  println!( "{}", bold( "clj — CLR Journal Viewer" ) );
  println!();
  println!( "Usage:  clj <command> [key::value ...]" );
  println!();
  println!( "{}", bold( "Commands:" ) );
  println!( "  .list     Display filtered event table (default: last 50 events)" );
  println!( "  .tail     Follow journal events in real-time (Ctrl+C to stop)" );
  println!( "  .stats    Aggregate statistics (default: last 7 days, by day)" );
  println!( "  .search   Substring search across event messages and output" );
  println!( "  .serve    Start embedded HTTP server for web viewing" );
  println!( "  .prune    Delete old journal files (default: keep 30 days)" );
  println!( "  .status   Show journal health: file count, size, date range" );
  println!( "  .export   Export filtered events to file" );
  println!();
  println!( "{}", bold( "Common filter params:" ) );
  println!( "  since::<dur>        Events newer than (e.g. 1h, 7d, 2w)" );
  println!( "  until::<dur>        Events older than" );
  println!( "  type::<event_type>  execution|credential|gate_wait|retry|timeout|..." );
  println!( "  command::<name>     Exact command name (run, ask, isolated, refresh)" );
  println!( "  exit_code::<n>      Exact exit code filter" );
  println!( "  model::<substr>     Model name substring filter" );
  println!( "  limit::<n>          Max events to return" );
  println!( "  dir::<path>         Journal directory (overrides CLR_JOURNAL_DIR)" );
  println!();
  println!( "{}", bold( "Command-specific params:" ) );
  println!( "  .list    format::table|json" );
  println!( "  .stats   by::day|model" );
  println!( "  .search  pattern::<str>               (required)" );
  println!( "  .prune   keep::<dur>  dry_run::0|1" );
  println!( "  .export  output::<path>  format::json|jsonl|csv|table" );
  println!( "  .serve   port::<n>" );
  println!();
  println!( "{}", bold( "Env vars:" ) );
  println!( "  CLR_JOURNAL_DIR   Journal directory (default: ~/.clr/journal/)" );
  println!( "  CLJ_PORT          Default HTTP port for .serve (default: random)" );
  println!( "  NO_COLOR          Suppress ANSI color codes" );
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main()
{
  let args    : Vec< String > = std::env::args().collect();
  let command = args.get( 1 ).map_or( ".help", String::as_str );
  let params  = parse_params( args.get( 2.. ).unwrap_or( &[] ) );
  let dir     = resolve_journal_dir( &params );

  match command
  {
    ".list"                             => cmd_list( &params, dir ),
    ".tail"                             => cmd_tail( &params, dir ),
    ".stats"                            => cmd_stats( &params, dir ),
    ".search"                           => cmd_search( &params, dir ),
    ".serve"                            => cmd_serve( &params, dir ),
    ".prune"                            => cmd_prune( &params, dir ),
    ".status"                           => cmd_status( &params, dir ),
    ".export"                           => cmd_export( &params, dir ),
    ".help" | "--help" | "-h" | "help"  => print_help(),
    other                               =>
    {
      eprintln!( "Error: unknown command '{other}'. Run 'clj .help' for usage." );
      std::process::exit( 1 );
    }
  }
}

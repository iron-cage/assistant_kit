//! CLR Journal Viewer CLI — dispatches `.command key::value` arguments to journal handlers.
// Root of the `clj` binary target — every item below is private to it, never a published API.
#![allow(missing_docs)]
// `clj` binary — CLR Journal Viewer CLI.
//
// Dispatches `.command key::value` arguments to one of eight journal handlers.
// Journal directory resolution priority: `journal_dir::` param > CLR_JOURNAL_DIR
// env > `~/.clr/journal/` (default).  Note `dir::` is a *filter* over each
// event's own working directory, not the journal location — see `known_params`.
//
// Shared output logic lives in `claude_journal_viewer::output` (accessible by
// both this binary and the unilang assistant routines in `routines.rs`).

use claude_journal_viewer::output::
{
  bold, build_filter, health_data, parse_query_string, resolve_journal_dir, stats_data, StreamFormat,
};
use claude_journal::JournalReader;
use std::{ collections::HashMap, path::{ Path, PathBuf } };

// ── Embedded web dashboard ────────────────────────────────────────────────────

/// Dashboard source with two substitution slots filled by [`index_html`].
///
/// `{{REFRESH_MS}}` is the poll interval in milliseconds (`0` disables polling
/// entirely) and `{{REFRESH_LABEL}}` is the matching human-readable suffix, so
/// the page never advertises a cadence it is not actually running.
const INDEX_HTML_TEMPLATE : &str = r#"<!DOCTYPE html>
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
    document.getElementById('status').textContent=evs.length+' event(s) — {{REFRESH_LABEL}}';
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
if({{REFRESH_MS}}>0)setInterval(load,{{REFRESH_MS}});
</script>
</body>
</html>"#;

/// Render the dashboard page for a given auto-refresh interval.
///
/// `refresh_secs` of `0` means "load once, never poll" — both the `setInterval`
/// guard and the status-line label are driven from the same value so they can
/// never disagree.
fn index_html( refresh_secs : u32 ) -> String
{
  let label = if refresh_secs == 0
  {
    "auto-refresh off".to_owned()
  }
  else
  {
    format!( "auto-refresh {refresh_secs}s" )
  };
  INDEX_HTML_TEMPLATE
    .replace( "{{REFRESH_MS}}", &( u64::from( refresh_secs ) * 1000 ).to_string() )
    .replace( "{{REFRESH_LABEL}}", &label )
}

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

/// Params every command accepts — journal location and color suppression.
const GLOBAL_PARAMS : &[ &str ] = &[ "journal_dir", "no_color" ];

/// Event-selection params, accepted by the commands that read events.
const FILTER_PARAMS : &[ &str ] =
  &[ "since", "until", "type", "command", "exit", "model", "dir", "creds", "limit" ];

/// The full set of params `command` accepts, sorted — always at least
/// [`GLOBAL_PARAMS`].
fn known_params( command : &str ) -> Vec< &'static str >
{
  let ( takes_filters, own ) : ( bool, &[ &'static str ] ) = match command
  {
    ".list"   => ( true,  &[ "format", "reverse", "sort" ] ),
    ".tail"   => ( true,  &[ "format" ] ),
    ".stats"  => ( true,  &[ "by" ] ),
    ".search" => ( true,  &[ "pattern" ] ),
    ".export" => ( true,  &[ "output", "format" ] ),
    ".serve"  => ( false, &[ "port", "bind", "open", "refresh" ] ),
    ".prune"  => ( false, &[ "keep", "dry_run" ] ),
    ".chart"  => ( false, &[ "out", "open" ] ),
    _         => ( false, &[] ),
  };
  let mut all : Vec< &'static str > = GLOBAL_PARAMS.to_vec();
  if takes_filters { all.extend_from_slice( FILTER_PARAMS ); }
  all.extend_from_slice( own );
  all.sort_unstable();
  all
}

/// Params the command's docs declare but no code reads yet.
///
/// Kept separate from "unknown" so a user following the documentation gets an
/// answer about the *feature* rather than being told the parameter does not
/// exist. Each entry is a debt against `docs/cli/param/`, not a typo.
fn unimplemented_params( command : &str ) -> &'static [ &'static str ]
{
  match command
  {
    ".list"   => &[ "columns", "wide" ],
    ".stats"  => &[ "verbosity", "wide" ],
    ".status" => &[ "verbosity" ],
    _         => &[],
  }
}

/// Reject any `key::value` the command does not accept, exiting 1.
///
/// Fix(param-silent-accept): reject unknown params instead of ignoring them
/// Root cause: `parse_params` collected every `key::value` into one map and each
///   handler read only the keys it knew, so a key nothing reads was accepted
///   silently with exit 0. That is what turned both naming collisions into
///   *quiet* defects: `exit::2` (the spelling every doc prints) fell through to
///   nothing and returned the whole unfiltered list, looking exactly like a
///   filter that matched everything.
/// Pitfall: a filter param that is ignored does not fail closed — it widens the
///   result set. Silence is the wrong default whenever the unread key would have
///   *narrowed* the output.
fn reject_unknown_params( command : &str, params : &HashMap< String, String > )
{
  let known   = known_params( command );
  let pending = unimplemented_params( command );

  let mut unknown : Vec< &str > = Vec::new();
  let mut planned : Vec< &str > = Vec::new();
  for key in params.keys().map( String::as_str )
  {
    if known.contains( &key ) { continue; }
    if pending.contains( &key ) { planned.push( key ); } else { unknown.push( key ); }
  }
  if unknown.is_empty() && planned.is_empty() { return; }

  unknown.sort_unstable();
  planned.sort_unstable();
  if !unknown.is_empty()
  {
    eprintln!(
      "Error: unknown parameter{} for {command}: {}",
      if unknown.len() == 1 { "" } else { "s" },
      unknown.join( ", " ),
    );
    eprintln!( "Accepted: {}", known.join( ", " ) );
  }
  if !planned.is_empty()
  {
    eprintln!(
      "Error: parameter{} not implemented for {command}: {}",
      if planned.len() == 1 { "" } else { "s" },
      planned.join( ", " ),
    );
    eprintln!( "Documented in docs/cli/param/ but not yet wired up — see that parameter's page." );
  }
  std::process::exit( 1 );
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
  // Parsed before the follow loop: `.tail` blocks indefinitely, so a bad format
  // name has to be rejected now rather than when the first event arrives, which
  // on a quiet journal could be never.
  let format = match StreamFormat::parse( params.get( "format" ).map_or( "table", String::as_str ) )
  {
    Ok( f )  => f,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  let reader = JournalReader::open( dir );
  eprintln!( "Tailing journal — press Ctrl+C to stop" );
  if let Some( header ) = format.header() { println!( "{header}" ); }
  for ev in reader.tail( &filter )
  {
    match format.render( &ev )
    {
      Ok( line ) => println!( "{line}" ),
      // Unreachable in practice — only JSON serialization can fail here, and
      // this record was just parsed out of a JSON line. Skip the one event
      // rather than abort a follow session that may have been running for hours.
      Err( e )   => eprintln!( "Warning: could not render event: {e}" ),
    }
  }
}

/// `.stats` — aggregate statistics by `by::day`, `by::model`, `by::dir`, or `by::agent`.
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

/// Respond with a JSON body under the given HTTP status code.
fn respond_json( req : tiny_http::Request, status : u16, body : &str )
{
  let resp = tiny_http::Response::from_string( body )
    .with_status_code( status )
    .with_header(
      "Content-Type: application/json"
        .parse::< tiny_http::Header >()
        .expect( "static Content-Type header is valid ASCII" ),
    );
  let _ = req.respond( resp );
}

/// Respond with the embedded dashboard HTML.
fn respond_html( req : tiny_http::Request, body : &str )
{
  let resp = tiny_http::Response::from_string( body )
    .with_header(
      "Content-Type: text/html; charset=utf-8"
        .parse::< tiny_http::Header >()
        .expect( "static Content-Type header is valid ASCII" ),
    );
  let _ = req.respond( resp );
}

/// Render an error payload as `{ "error" : "..." }`.
fn error_body( message : &str ) -> String
{
  serde_json::json!( { "error" : message } ).to_string()
}

/// Name the first query key outside `allowed`, if any.
///
/// The CLI's own rejection (`reject_unknown_params`) cannot cover this path:
/// query keys never reach it. Without the same check here, `?exit_code=2`
/// returns HTTP 200 and the *whole* event list — the identical silent-widening
/// failure the CLI fix closes, one surface over.
///
/// Applied to `/api/events` and `/api/stats` only. `/api/health` takes no
/// parameters at all, so no key it might receive can change its response —
/// there is no result set to silently widen, and rejecting a stray cache-buster
/// would cost strictness with nothing bought.
fn unknown_query_key< S : ::core::hash::BuildHasher >(
  params : &HashMap< String, String, S >, allowed : &[ &str ],
) -> Option< String >
{
  let mut bad : Vec< &str > = params.keys().map( String::as_str )
    .filter( | k | !allowed.contains( k ) ).collect();
  bad.sort_unstable();
  bad.first().map( | k | format!( "unknown query parameter '{k}' (accepted: {})", allowed.join( ", " ) ) )
}

/// `GET /api/events` — filtered event list; same query vocabulary as `.list`.
fn respond_events( req : tiny_http::Request, reader : &JournalReader, query : &str )
{
  let params = parse_query_string( query );
  if let Some( e ) = unknown_query_key( &params, FILTER_PARAMS )
  {
    respond_json( req, 400, &error_body( &e ) );
    return;
  }
  match build_filter( &params )
  {
    Ok( mut filter ) =>
    {
      // Keep the historical cap for callers that pass no `limit`, so a large
      // journal is never streamed in full just because the query was empty.
      if filter.limit.is_none() { filter.limit = Some( 200 ); }
      let events = reader.query( &filter );
      let body   = serde_json::to_string( &events ).unwrap_or_else( | _ | "[]".to_owned() );
      respond_json( req, 200, &body );
    }
    Err( e ) => respond_json( req, 400, &error_body( &e ) ),
  }
}

/// `GET /api/stats` — grouped statistics; same query vocabulary as `.stats`.
fn respond_stats( req : tiny_http::Request, dir : &Path, query : &str )
{
  let params = parse_query_string( query );
  let mut allowed = FILTER_PARAMS.to_vec();
  allowed.push( "by" );
  allowed.sort_unstable();
  if let Some( e ) = unknown_query_key( &params, &allowed )
  {
    respond_json( req, 400, &error_body( &e ) );
    return;
  }
  match stats_data( &params, dir.to_path_buf() )
  {
    Ok( data ) =>
    {
      let groups : Vec< _ > = data
        .groups
        .iter()
        .map( | g | serde_json::json!( { "key" : g.key, "count" : g.count, "cost_usd" : g.cost_usd } ) )
        .collect();
      let body = serde_json::json!(
      {
        "by"           : data.by,
        "column_label" : data.column_label,
        "total_events" : data.total_events,
        "groups"       : groups,
      } ).to_string();
      respond_json( req, 200, &body );
    }
    Err( e ) => respond_json( req, 400, &error_body( &e ) ),
  }
}

/// `GET /api/health` — journal file count, size, and date range.
///
/// `oldest`/`newest` are `null` rather than a placeholder string when the
/// journal directory holds no files, so a consumer can distinguish "empty" from
/// a real date without string-matching.
fn respond_health( req : tiny_http::Request, dir : &Path )
{
  let h    = health_data( dir.to_path_buf() );
  let body = serde_json::json!(
  {
    "files"  : h.files,
    "bytes"  : h.bytes,
    "oldest" : h.oldest,
    "newest" : h.newest,
  } ).to_string();
  respond_json( req, 200, &body );
}

/// `.serve` — start an embedded HTTP server for web-based journal viewing.
fn cmd_serve( params : &HashMap< String, String >, dir : PathBuf )
{
  let bind     = params.get( "bind" ).map_or( "127.0.0.1", String::as_str );
  let port_str = params
    .get( "port" )
    .cloned()
    .unwrap_or_else( || std::env::var( "CLJ_PORT" ).unwrap_or_else( | _ | "0".to_owned() ) );
  let port : u16 = port_str.parse().unwrap_or( 0 );
  let refresh : u32 = params.get( "refresh" ).map_or( 10, | s | s.parse().unwrap_or_else( | _ |
  {
    eprintln!( "Error: invalid refresh '{s}' (expected non-negative integer seconds; 0 disables)" );
    std::process::exit( 1 );
  } ) );

  let addr   = format!( "{bind}:{port}" );
  let server = match tiny_http::Server::http( &addr )
  {
    Ok( s )  => s,
    Err( e ) => { eprintln!( "Error: could not start server on {addr}: {e}" ); std::process::exit( 1 ); }
  };
  let actual_port = server.server_addr().to_ip().map_or( port, | a | a.port() );

  // INV-002: loopback prints as `localhost` (AC-001's exact form); any widened
  // bind prints the address actually listened on and warns, so the startup line
  // can never understate how reachable the journal now is.
  let loopback = matches!( bind, "127.0.0.1" | "localhost" | "::1" | "[::1]" );
  let host     = if loopback { "localhost" } else { bind };
  let url      = format!( "http://{host}:{actual_port}" );

  // The warning precedes the startup line deliberately. A consumer that syncs
  // on the startup line (any piped reader, including the test harness) is then
  // guaranteed the warning is already written — emitting it afterwards makes
  // the exposure signal racy for exactly the readers that most need it.
  if !loopback
  {
    eprintln!( "Warning: bound to {bind} — journal data is reachable beyond this machine" );
  }
  println!( "Listening on {url}" );
  // Flush stdout so piped readers (e.g. integration test harness) see the port immediately.
  std::io::Write::flush( &mut std::io::stdout() ).ok();

  if matches!( params.get( "open" ).map( String::as_str ), Some( "1" | "true" ) )
  {
    if let Err( e ) = open::that( &url )
    {
      eprintln!( "Warning: could not open {url} in browser: {e}" );
    }
  }

  let page   = index_html( refresh );
  let reader = JournalReader::open( dir.clone() );
  loop
  {
    let Ok( req ) = server.recv() else { continue; };
    let full = req.url().to_owned();
    let ( path, query ) = full.split_once( '?' ).unwrap_or( ( full.as_str(), "" ) );
    match path
    {
      "/api/events" => respond_events( req, &reader, query ),
      "/api/stats"  => respond_stats( req, &dir, query ),
      "/api/health" => respond_health( req, &dir ),
      // Every other `/api/*` path is a client error, not a request for the
      // dashboard — falling through to the HTML branch would answer a typo'd
      // endpoint with 200 + text/html and hide the mistake.
      p if p.starts_with( "/api/" ) => respond_json( req, 404, &error_body( &format!( "unknown endpoint '{p}'" ) ) ),
      _ => respond_html( req, &page ),
    }
  }
}

/// `.chart` — render a usage SVG chart, optionally opened in the default browser.
fn cmd_chart( params : &HashMap< String, String >, dir : PathBuf )
{
  match claude_journal_viewer::output::chart_output( params, dir )
  {
    Ok( s )  => println!( "{s}" ),
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
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
  println!( "  .chart    Render a usage SVG chart, optionally opened in browser" );
  println!();
  println!( "{}", bold( "Common filter params:" ) );
  println!( "  since::<dur>        Events newer than (e.g. 1h, 7d, 2w)" );
  println!( "  until::<dur>        Events older than" );
  println!( "  type::<event_type>  execution|credential|gate_wait|retry|timeout|..." );
  println!( "  command::<name>     Exact command name (run, ask, isolated, refresh)" );
  println!( "  exit::<n>           Exact exit code filter" );
  println!( "  model::<substr>     Model name substring filter" );
  println!( "  dir::<substr>       Event working-directory substring filter" );
  println!( "  creds::<substr>     Credential name substring filter" );
  println!( "  limit::<n>          Max events to return" );
  println!();
  println!( "{}", bold( "Global params (accepted by every command):" ) );
  println!( "  journal_dir::<path> Journal directory (overrides CLR_JOURNAL_DIR)" );
  println!( "  no_color::0|1       Suppress ANSI color codes (same as NO_COLOR=1)" );
  println!();
  println!( "{}", bold( "Command-specific params:" ) );
  println!( "  .list    format::table|json|jsonl|csv" );
  println!( "           sort::time|cost|duration|exit|model|command  reverse::0|1" );
  println!( "  .tail    format::table|json|jsonl|csv  (json == jsonl: a stream has no closing bracket)" );
  println!( "  .stats   by::day|model|dir|agent" );
  println!( "  .search  pattern::<str>               (required)" );
  println!( "  .prune   keep::<dur>  dry_run::0|1" );
  println!( "  .export  output::<path>  format::json|jsonl|csv|table" );
  println!( "  .serve   port::<n>  bind::<addr>  open::0|1  refresh::<secs>" );
  println!( "  .chart   out::<path>  open::0|1" );
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

  // Unknown *command* outranks unknown *param* — `clj .bogus since::1d` should
  // report the command, not lecture about a param that only looks wrong because
  // the command it belongs to does not exist.
  const COMMANDS : &[ &str ] =
    &[ ".list", ".tail", ".stats", ".search", ".serve", ".prune", ".status", ".export", ".chart" ];
  if COMMANDS.contains( &command ) { reject_unknown_params( command, &params ); }

  if matches!( params.get( "no_color" ).map( String::as_str ), Some( "1" | "true" ) )
  {
    claude_journal_viewer::output::force_no_color();
  }

  let dir = resolve_journal_dir( &params );

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
    ".chart"                            => cmd_chart( &params, dir ),
    ".help" | "--help" | "-h" | "help"  => print_help(),
    other                               =>
    {
      eprintln!( "Error: unknown command '{other}'. Run 'clj .help' for usage." );
      std::process::exit( 1 );
    }
  }
}

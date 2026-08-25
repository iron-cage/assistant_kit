//! `.serve` HTTP tests — FT-1..FT-12 (`feature/002_web_viewing`) and IN-1..IN-3
//! (`invariant/002_localhost_only`).
//!
//! Every case here spawns the real `clj` binary, recovers the bound port from
//! the flushed `Listening on …` startup line, and speaks HTTP/1.0 over a plain
//! `TcpStream` — no HTTP client dependency, and no mocking of the server under
//! test.
//!
//! All `.serve` coverage lives in this one file. `viewer_integration_test.rs`
//! owns the non-serving commands; splitting the serve harness across both would
//! mean two copies of the spawn/parse/connect dance.

#![ allow( missing_docs ) ]
#![ cfg( unix ) ]

use claude_journal::{ EventRecord, EventType, JournalWriter };
use std::io::{ BufRead, Read, Write };
use std::path::Path;
use std::process::{ Command, Stdio };

const CLJ : &str = env!( "CARGO_BIN_EXE_clj" );

fn assert_container()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: ./verb/test (from workspace root)\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

// ── Fixture ───────────────────────────────────────────────────────────────────

/// Write 4 events: 2 `Execution`, 1 `Credential`, 1 `Retry`.
///
/// Timestamps are current, so every event survives any reasonable `since::`
/// window; the type mix is what makes the `?type=execution` filter case in FT-7
/// able to fail.
fn write_fixture_events( dir : &Path )
{
  let writer = JournalWriter::new( dir.to_path_buf() );

  let mut ev1 = EventRecord::new( EventType::Execution );
  ev1.fields.command     = Some( "run".to_owned() );
  ev1.fields.model       = Some( "claude-sonnet-5".to_owned() );
  ev1.fields.exit_code   = Some( 0 );
  ev1.fields.duration_ms = Some( 1_500 );
  ev1.fields.cost_usd    = Some( 0.012 );
  writer.append( &ev1 ).expect( "append ev1" );

  let mut ev2 = EventRecord::new( EventType::Credential );
  ev2.fields.command   = Some( "refresh".to_owned() );
  ev2.fields.exit_code = Some( 0 );
  writer.append( &ev2 ).expect( "append ev2" );

  let mut ev3 = EventRecord::new( EventType::Retry );
  ev3.fields.error_class = Some( "Transient".to_owned() );
  writer.append( &ev3 ).expect( "append ev3" );

  let mut ev4 = EventRecord::new( EventType::Execution );
  ev4.fields.command     = Some( "ask".to_owned() );
  ev4.fields.model       = Some( "claude-haiku-4-5-20251001".to_owned() );
  ev4.fields.exit_code   = Some( 0 );
  ev4.fields.duration_ms = Some( 500 );
  ev4.fields.cost_usd    = Some( 0.002 );
  writer.append( &ev4 ).expect( "append ev4" );
}

// ── Serve harness ─────────────────────────────────────────────────────────────

/// A running `clj .serve` child, its bound port, and its captured stderr path.
///
/// Killed on drop, so a panicking assertion can never leak a listening server
/// into the rest of the suite.
#[ derive( Debug ) ]
struct Serve
{
  child      : std::process::Child,
  port       : u16,
  startup    : String,
  stderr_log : std::path::PathBuf,
}

impl Serve
{
  /// Everything the server wrote to stderr up to this moment.
  fn stderr( &self ) -> String
  {
    std::fs::read_to_string( &self.stderr_log ).unwrap_or_default()
  }
}

impl Drop for Serve
{
  fn drop( &mut self )
  {
    self.child.kill().ok();
    self.child.wait().ok();
  }
}

/// Spawn `clj .serve journal_dir::<dir> <extra…>` and block until it prints its port.
///
/// stderr is redirected to a file inside `dir` rather than a pipe: the tests
/// that assert on warnings read it after the fact, and an unread pipe would
/// deadlock the child once its buffer filled.
fn serve( extra : &[ &str ], dir : &Path ) -> Serve
{
  assert_container();
  let stderr_log = dir.join( "-serve_stderr.log" );
  let stderr_file = std::fs::File::create( &stderr_log ).expect( "create stderr log" );

  let mut child = Command::new( CLJ )
    .arg( ".serve" )
    .arg( format!( "journal_dir::{}", dir.display() ) )
    .args( extra )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLJ_PORT" )
    .stdout( Stdio::piped() )
    .stderr( Stdio::from( stderr_file ) )
    .spawn()
    .expect( "failed to spawn clj .serve" );

  let stdout     = child.stdout.take().expect( "no stdout pipe" );
  let mut reader = std::io::BufReader::new( stdout );
  let mut line   = String::new();
  reader.read_line( &mut line ).expect( "failed to read server startup line" );
  let startup = line.trim().to_owned();

  let port : u16 = startup
    .rsplit( ':' )
    .next()
    .and_then( | s | s.parse().ok() )
    .unwrap_or_else( || panic!( "could not parse port from startup line: '{startup}'" ) );

  Serve { child, port, startup, stderr_log }
}

/// Connect to `host:port`, retrying briefly while the accept loop comes up.
fn connect( host : &str, port : u16 ) -> Option< std::net::TcpStream >
{
  for _ in 0..40
  {
    if let Ok( s ) = std::net::TcpStream::connect( ( host, port ) ) { return Some( s ); }
    std::thread::sleep( core::time::Duration::from_millis( 50 ) );
  }
  None
}

/// Issue `GET path` against `host:port` and return the whole raw response.
fn http_get_from( host : &str, port : u16, path : &str ) -> String
{
  let mut stream = connect( host, port )
    .unwrap_or_else( || panic!( "could not connect to {host}:{port}" ) );
  stream
    .write_all( format!( "GET {path} HTTP/1.0\r\nHost: localhost\r\n\r\n" ).as_bytes() )
    .expect( "failed to write HTTP request" );
  let mut response = String::new();
  stream.read_to_string( &mut response ).expect( "failed to read HTTP response" );
  response
}

/// Issue `GET path` against loopback.
fn http_get( port : u16, path : &str ) -> String
{
  http_get_from( "127.0.0.1", port, path )
}

/// Split a raw response into `( headers, body )`.
fn split_response( raw : &str ) -> ( &str, &str )
{
  raw.split_once( "\r\n\r\n" ).unwrap_or_else( || panic!( "malformed HTTP response:\n{raw}" ) )
}

/// Assert status/content-type and return the body of a successful JSON response.
fn json_body( raw : &str, expect_status : &str ) -> serde_json::Value
{
  let ( head, body ) = split_response( raw );
  assert!( head.contains( expect_status ), "expected status {expect_status} in:\n{head}" );
  assert!(
    head.to_lowercase().contains( "application/json" ),
    "expected application/json content-type in:\n{head}"
  );
  serde_json::from_str( body ).unwrap_or_else( | e | panic!( "body is not JSON ({e}):\n{body}" ) )
}

// ── FT-1 / IN-1 : default start binds loopback at an OS-assigned port ─────────

#[ test ]
fn ft1_in1_serve_starts_on_loopback_and_prints_url()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s = serve( &[ "port::0" ], dir.path() );

  assert!(
    s.startup.starts_with( "Listening on http://localhost:" ),
    "startup line should report the loopback default: '{}'", s.startup
  );
  assert!( s.port > 0, "OS-assigned port should be nonzero: '{}'", s.startup );

  let raw = http_get( s.port, "/" );
  assert!( split_response( &raw ).0.contains( "200" ), "expected 200 from loopback:\n{raw}" );

  // The default bind is loopback-only, so it must not carry the exposure warning.
  assert!(
    !s.stderr().contains( "reachable beyond this machine" ),
    "default bind must not warn about network exposure: '{}'", s.stderr()
  );
}

// ── FT-2 : GET / returns the embedded dashboard ───────────────────────────────

#[ test ]
fn ft2_get_root_returns_embedded_html()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s   = serve( &[ "port::0" ], dir.path() );
  let raw = http_get( s.port, "/" );

  let ( head, body ) = split_response( &raw );
  assert!( head.contains( "200" ), "expected 200:\n{head}" );
  assert!(
    head.to_lowercase().contains( "text/html" ),
    "expected text/html content-type:\n{head}"
  );
  assert!( body.contains( "CLR Journal" ), "expected dashboard title in body:\n{body}" );
}

// ── FT-3 : GET /api/events returns a JSON array ───────────────────────────────

#[ test ]
fn ft3_api_events_returns_json_array()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s   = serve( &[ "port::0" ], dir.path() );
  let raw = http_get( s.port, "/api/events" );

  let events = json_body( &raw, "200" );
  let arr    = events.as_array().unwrap_or_else( || panic!( "expected JSON array, got: {events}" ) );
  assert_eq!( arr.len(), 4, "fixture writes 4 events, got {}: {events}", arr.len() );
}

// ── FT-7 : GET /api/events?… honors the query string ─────────────────────────

#[ test ]
fn ft7_api_events_honors_query_string()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s = serve( &[ "port::0" ], dir.path() );

  let filtered = json_body( &http_get( s.port, "/api/events?since=1h&type=execution" ), "200" );
  let arr      = filtered.as_array().expect( "expected JSON array" );
  assert_eq!( arr.len(), 2, "fixture has 2 execution events, got {}: {filtered}", arr.len() );
  for ev in arr
  {
    assert_eq!( ev[ "type" ], "execution", "unfiltered event leaked through: {ev}" );
  }

  // `limit` must also reach the filter — otherwise a query could look honored
  // while only the `type` key was actually read.
  let limited = json_body( &http_get( s.port, "/api/events?limit=1" ), "200" );
  assert_eq!( limited.as_array().expect( "expected JSON array" ).len(), 1, "limit=1 ignored: {limited}" );
}

#[ test ]
fn ft7_api_events_rejects_invalid_query_value()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s   = serve( &[ "port::0" ], dir.path() );
  let raw = http_get( s.port, "/api/events?since=banana" );

  let err = json_body( &raw, "400" );
  assert!(
    err[ "error" ].as_str().unwrap_or_default().contains( "banana" ),
    "400 body should name the offending value: {err}"
  );
}

// ── FT-7c : an unknown query *key* is a 400, not a silently widened 200 ──────

/// A key nothing reads would otherwise return the full event list with HTTP 200
/// — the same silent-widening failure the CLI's parameter rejection closes.
/// `journal_dir` is checked explicitly: it must never be settable per-request.
#[ test ]
fn ft7_api_rejects_unknown_query_key()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s = serve( &[ "port::0" ], dir.path() );

  for ( path, key ) in [
    ( "/api/events?exit_code=2", "exit_code" ),
    ( "/api/events?journal_dir=/etc", "journal_dir" ),
    ( "/api/stats?by=model&bogus=1", "bogus" ),
  ]
  {
    let err = json_body( &http_get( s.port, path ), "400" );
    let msg = err[ "error" ].as_str().unwrap_or_default().to_owned();
    assert!( msg.contains( key ), "400 body should name '{key}': {msg}" );
  }

  // The documented vocabulary still works — the guard rejects keys, not queries.
  let ok = json_body( &http_get( s.port, "/api/events?exit=0&limit=5" ), "200" );
  assert!( ok.is_array(), "a valid query must still return the array: {ok}" );
  let ok = json_body( &http_get( s.port, "/api/stats?by=model" ), "200" );
  assert_eq!( ok[ "by" ].as_str(), Some( "model" ), "valid stats query broke: {ok}" );
}

// ── FT-4 : GET /api/health returns the documented structure ──────────────────

#[ test ]
fn ft4_api_health_returns_documented_structure()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s      = serve( &[ "port::0" ], dir.path() );
  let health = json_body( &http_get( s.port, "/api/health" ), "200" );

  assert!( health[ "files" ].is_u64(), "files should be a number: {health}" );
  assert!( health[ "bytes" ].is_u64(), "bytes should be a number: {health}" );
  assert!( health[ "files" ].as_u64().unwrap_or( 0 ) >= 1, "fixture writes at least one file: {health}" );
  assert!( health[ "bytes" ].as_u64().unwrap_or( 0 ) > 0, "fixture events occupy bytes: {health}" );
  assert!( health[ "oldest" ].is_string(), "oldest should be a date string: {health}" );
  assert!( health[ "newest" ].is_string(), "newest should be a date string: {health}" );
}

#[ test ]
fn ft4_api_health_reports_null_dates_for_empty_journal()
{
  let dir = tempfile::TempDir::new().unwrap();
  // Deliberately no fixture — an empty journal must be distinguishable from a
  // populated one without string-matching a placeholder.
  let s      = serve( &[ "port::0" ], dir.path() );
  let health = json_body( &http_get( s.port, "/api/health" ), "200" );

  assert_eq!( health[ "files" ], 0, "empty journal has no files: {health}" );
  assert!( health[ "oldest" ].is_null(), "empty journal reports null oldest: {health}" );
  assert!( health[ "newest" ].is_null(), "empty journal reports null newest: {health}" );
}

// ── FT-9 : GET /api/stats returns grouped statistics ─────────────────────────

#[ test ]
fn ft9_api_stats_returns_grouped_json()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s     = serve( &[ "port::0" ], dir.path() );
  let stats = json_body( &http_get( s.port, "/api/stats?by=model&since=7d" ), "200" );

  assert_eq!( stats[ "by" ], "model", "grouping dimension should echo back: {stats}" );
  assert_eq!( stats[ "column_label" ], "MODEL", "by=model labels the key column MODEL: {stats}" );
  assert_eq!( stats[ "total_events" ], 4, "fixture writes 4 events: {stats}" );

  let groups = stats[ "groups" ].as_array().expect( "groups should be an array" );
  let sonnet = groups
    .iter()
    .find( | g | g[ "key" ] == "claude-sonnet-5" )
    .unwrap_or_else( || panic!( "expected a claude-sonnet-5 group: {stats}" ) );
  assert_eq!( sonnet[ "count" ], 1, "one sonnet event in the fixture: {sonnet}" );
  assert!(
    ( sonnet[ "cost_usd" ].as_f64().unwrap_or( 0.0 ) - 0.012 ).abs() < 1e-9,
    "sonnet group should carry its event's cost: {sonnet}"
  );

  // The default grouping differs from `by=model`, so an ignored query string
  // cannot make the assertions above pass by accident.
  let default_stats = json_body( &http_get( s.port, "/api/stats" ), "200" );
  assert_eq!( default_stats[ "by" ], "day", "stats default grouping is by day: {default_stats}" );
}

#[ test ]
fn ft9_api_stats_rejects_invalid_by()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s   = serve( &[ "port::0" ], dir.path() );
  let err = json_body( &http_get( s.port, "/api/stats?by=banana" ), "400" );

  assert!(
    err[ "error" ].as_str().unwrap_or_default().contains( "banana" ),
    "400 body should name the offending grouping: {err}"
  );
}

// ── FT-10 : unknown /api/* paths 404 instead of serving the dashboard ────────

#[ test ]
fn ft10_unknown_api_path_returns_404_json()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s   = serve( &[ "port::0" ], dir.path() );
  let err = json_body( &http_get( s.port, "/api/nonsense" ), "404" );

  assert!(
    err[ "error" ].as_str().unwrap_or_default().contains( "/api/nonsense" ),
    "404 body should name the unknown endpoint: {err}"
  );

  // A non-API path is still the dashboard, not a 404 — the catch-all is scoped
  // to `/api/`, not applied to every unrecognised URL.
  let page = http_get( s.port, "/whatever" );
  let ( head, body ) = split_response( &page );
  assert!( head.contains( "200" ), "non-API paths still serve the dashboard:\n{head}" );
  assert!( body.contains( "CLR Journal" ), "expected dashboard body:\n{body}" );
}

// ── FT-5 : port:: override ────────────────────────────────────────────────────

#[ test ]
fn ft5_port_override_binds_requested_port()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s = serve( &[ "port::19090" ], dir.path() );

  assert_eq!( s.port, 19090, "startup line should report the pinned port: '{}'", s.startup );
  let raw = http_get( 19090, "/" );
  assert!( split_response( &raw ).0.contains( "200" ), "expected 200 on the pinned port:\n{raw}" );
}

// ── FT-6 : embedded HTML is self-contained ────────────────────────────────────

#[ test ]
fn ft6_embedded_html_has_no_cdn_dependencies()
{
  let dir = tempfile::TempDir::new().unwrap();
  let s   = serve( &[ "port::0" ], dir.path() );
  let raw = http_get( s.port, "/" );
  let ( _, body ) = split_response( &raw );

  for cdn in [ "cdn.jsdelivr.net", "unpkg.com", "cdnjs.cloudflare.com", "ajax.googleapis.com", "//fonts.googleapis.com" ]
  {
    assert!( !body.contains( cdn ), "embedded HTML must not reference {cdn}:\n{body}" );
  }
}

// ── FT-11 : refresh:: drives the poll interval ───────────────────────────────

#[ test ]
fn ft11_refresh_interval_is_configurable()
{
  let dir = tempfile::TempDir::new().unwrap();

  let default_page = { let s = serve( &[ "port::0" ], dir.path() ); http_get( s.port, "/" ) };
  assert!( default_page.contains( "auto-refresh 10s" ), "default refresh is 10s:\n{default_page}" );
  assert!( default_page.contains( "setInterval(load,10000)" ), "default poll is 10000ms:\n{default_page}" );

  let custom_page = { let s = serve( &[ "port::0", "refresh::30" ], dir.path() ); http_get( s.port, "/" ) };
  assert!( custom_page.contains( "auto-refresh 30s" ), "refresh::30 should reach the page:\n{custom_page}" );
  assert!( custom_page.contains( "setInterval(load,30000)" ), "refresh::30 should set 30000ms:\n{custom_page}" );

  let off_page = { let s = serve( &[ "port::0", "refresh::0" ], dir.path() ); http_get( s.port, "/" ) };
  assert!( off_page.contains( "auto-refresh off" ), "refresh::0 disables polling:\n{off_page}" );
  assert!( off_page.contains( "if(0>0)" ), "refresh::0 should leave the interval unarmed:\n{off_page}" );
}

#[ test ]
fn ft11_invalid_refresh_exits_1()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();
  let out = Command::new( CLJ )
    .args( [ ".serve", &format!( "journal_dir::{}", dir.path().display() ), "port::0", "refresh::soon" ] )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLJ_PORT" )
    .output()
    .expect( "failed to run clj .serve" );

  assert_eq!( out.status.code(), Some( 1 ), "invalid refresh should exit 1" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!( stderr.contains( "invalid refresh" ), "stderr should explain the rejection: {stderr}" );
}

// ── IT-4 : a pinned port already in use exits 1 ──────────────────────────────

#[ test ]
fn it4_busy_pinned_port_exits_1()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();

  // Hold the port for the duration of the child's startup attempt. The port
  // must be pinned: with the ephemeral default a no-arg `.serve` can never
  // collide, so this case could not fail.
  let held = std::net::TcpListener::bind( "127.0.0.1:0" ).expect( "hold a port" );
  let port = held.local_addr().expect( "local_addr" ).port();

  let out = Command::new( CLJ )
    .args( [ ".serve", &format!( "journal_dir::{}", dir.path().display() ), &format!( "port::{port}" ) ] )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLJ_PORT" )
    .output()
    .expect( "failed to run clj .serve" );

  assert_eq!( out.status.code(), Some( 1 ), "a busy port should exit 1" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( &format!( "could not start server on 127.0.0.1:{port}" ) ),
    "stderr should name the address that failed: {stderr}"
  );
}

// ── TC-3 : an unparseable bind address fails at startup ──────────────────────

#[ test ]
fn tc3_invalid_bind_address_exits_1()
{
  assert_container();
  let dir = tempfile::TempDir::new().unwrap();

  let out = Command::new( CLJ )
    .args( [ ".serve", &format!( "journal_dir::{}", dir.path().display() ), "bind::999.999.999.999", "port::0" ] )
    .env_remove( "CLR_JOURNAL_DIR" )
    .env_remove( "CLJ_PORT" )
    .output()
    .expect( "failed to run clj .serve" );

  // The address is validated by the OS at bind time, not at parse time — what
  // matters is that it fails loudly rather than silently binding something else.
  assert_eq!( out.status.code(), Some( 1 ), "an unparseable bind address should exit 1" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "could not start server on 999.999.999.999:0" ),
    "stderr should name the rejected address: {stderr}"
  );
}

// ── FT-12 : open::1 never takes the server down ──────────────────────────────

#[ test ]
fn ft12_open_failure_is_non_fatal()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  // A container has no browser, so `open::1` exercises the failure path — which
  // must degrade to a warning rather than abort startup.
  let s   = serve( &[ "port::0", "open::1" ], dir.path() );
  let raw = http_get( s.port, "/" );

  assert!( split_response( &raw ).0.contains( "200" ), "server must serve despite a failed browser launch:\n{raw}" );
}

// ── FT-8 : SIGTERM terminates the server ─────────────────────────────────────

#[ test ]
fn ft8_sigterm_terminates_server()
{
  let dir = tempfile::TempDir::new().unwrap();
  let mut s = serve( &[ "port::0" ], dir.path() );

  let pid = s.child.id().to_string();
  let killed = Command::new( "kill" ).args( [ "-TERM", &pid ] ).status().expect( "run kill" );
  assert!( killed.success(), "kill -TERM {pid} should succeed" );

  // Poll rather than block forever: a hang is the failure this asserts against.
  let mut exited = false;
  for _ in 0..50
  {
    if matches!( s.child.try_wait(), Ok( Some( _ ) ) ) { exited = true; break; }
    std::thread::sleep( core::time::Duration::from_millis( 100 ) );
  }
  assert!( exited, "server should terminate within 5s of SIGTERM" );
}

// ── IN-2 : bind::0.0.0.0 is honored and warns ────────────────────────────────

#[ test ]
fn in2_non_loopback_bind_is_honored_and_warned()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  let s = serve( &[ "bind::0.0.0.0", "port::19412" ], dir.path() );

  assert_eq!(
    s.startup, "Listening on http://0.0.0.0:19412",
    "a widened bind must report the address it actually listened on, not 'localhost'"
  );
  assert!(
    s.stderr().contains( "reachable beyond this machine" ),
    "widening the bind must warn (INV-002 consent): '{}'", s.stderr()
  );

  // 0.0.0.0 includes loopback, so this still answers — the discriminator above
  // is the startup line and the warning, not reachability.
  let raw = http_get( 19412, "/" );
  assert!( split_response( &raw ).0.contains( "200" ), "expected 200:\n{raw}" );
}

// ── IN-3 : bind:: selects the interface ──────────────────────────────────────

#[ test ]
fn in3_bind_selects_the_interface()
{
  let dir = tempfile::TempDir::new().unwrap();
  write_fixture_events( dir.path() );
  // 127.0.0.2 is inside loopback (127.0.0.0/8) so nothing leaves the machine,
  // yet it is a different address than the old hardcoded 127.0.0.1 — which is
  // what makes this case able to fail against an unwired `bind::`.
  let s = serve( &[ "bind::127.0.0.2", "port::19413" ], dir.path() );

  assert_eq!( s.startup, "Listening on http://127.0.0.2:19413", "startup line: '{}'", s.startup );

  let raw = http_get_from( "127.0.0.2", 19413, "/" );
  assert!( split_response( &raw ).0.contains( "200" ), "bound address must answer:\n{raw}" );

  // The server is provably up by now, so a refusal here is the bind taking
  // effect rather than a startup race.
  assert!(
    std::net::TcpStream::connect( ( "127.0.0.1", 19413_u16 ) ).is_err(),
    "binding 127.0.0.2 must not also listen on 127.0.0.1"
  );
}

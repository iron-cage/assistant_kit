//! Seam tests: `CLAUDE_QUOTA_BASE_URL` redirects `fetch_*` transports to a
//! real local HTTP server (std `TcpListener`, no mocks) — proving the override
//! grafts the endpoint path onto the local origin, relaxes `https_only` for
//! plaintext loopback only, and carries auth headers end-to-end through ureq.
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | SM-01 | `sm01_oauth_usage_via_loopback_override` | GET path preserved, body parsed, Bearer header sent | P |
//! | SM-02 | `sm02_rate_limits_via_loopback_override` | POST path preserved, rate-limit headers parsed | P |
//! | SM-03 | `sm03_non_loopback_http_override_errs` | plaintext non-loopback override never succeeds | N |

#![ cfg( feature = "enabled" ) ]

use core::time::Duration;
use std::io::{ Read, Write };
use std::net::{ TcpListener, TcpStream };
use std::sync::Mutex;

/// Serializes env-var mutation across tests — `CLAUDE_QUOTA_BASE_URL` is
/// process-global, so plain `cargo test` (threads, one process) would race
/// without this. Under nextest (process per test) the lock is a no-op.
static ENV_LOCK : Mutex< () > = Mutex::new( () );

/// Read one full HTTP request (head + `Content-Length` body) from the stream.
fn read_request( stream : &mut TcpStream ) -> Vec< u8 >
{
  stream.set_read_timeout( Some( Duration::from_secs( 10 ) ) ).expect( "set_read_timeout" );
  let mut buf = Vec::new();
  let mut chunk = [ 0u8; 4096 ];
  loop
  {
    match stream.read( &mut chunk )
    {
      Ok( 0 ) | Err( _ ) => break,
      Ok( n ) =>
      {
        buf.extend_from_slice( &chunk[ ..n ] );
        if let Some( head_end ) = buf.windows( 4 ).position( | w | w == b"\r\n\r\n" )
        {
          let head = String::from_utf8_lossy( &buf[ ..head_end ] ).to_string();
          let content_length = head.lines().find_map( | l |
          {
            let low = l.to_ascii_lowercase();
            low.strip_prefix( "content-length:" )?.trim().parse::< usize >().ok()
          } ).unwrap_or( 0 );
          if buf.len() >= head_end + 4 + content_length { break; }
        }
      }
    }
  }
  buf
}

/// Bind a loopback listener, serve exactly one canned HTTP/1.1 response, and
/// hand back the captured raw request plus the bound port.
///
/// `extra_headers` is a pre-formatted `Name: value\r\n` block (may be empty).
///
/// # Panics
///
/// Panics on any socket setup or I/O failure — seam fixtures fail loudly
/// rather than letting a test run against a half-up server.
fn serve_once( body : &'static str, extra_headers : &'static str )
-> ( std::thread::JoinHandle< Vec< u8 > >, u16 )
{
  let listener = TcpListener::bind( "127.0.0.1:0" ).expect( "bind loopback listener" );
  let port = listener.local_addr().expect( "local_addr" ).port();
  let handle = std::thread::spawn( move ||
  {
    let ( mut stream, _addr ) = listener.accept().expect( "accept" );
    let request = read_request( &mut stream );
    let response = format!(
      "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n{}Connection: close\r\n\r\n{}",
      body.len(), extra_headers, body
    );
    stream.write_all( response.as_bytes() ).expect( "write response" );
    request
  } );
  ( handle, port )
}

/// SM-01: `fetch_oauth_usage` against a loopback override — the local server
/// receives `GET /api/oauth/usage` (path grafted from the endpoint const onto
/// the override origin) with the Bearer token, and the canned JSON body parses
/// into period buckets exactly as a live response would.
#[ test ]
fn sm01_oauth_usage_via_loopback_override()
{
  let _guard = ENV_LOCK.lock().expect( "env lock" );
  let body = r#"{"five_hour":{"utilization":12.5,"resets_at":"2026-08-18T04:00:00+00:00"},"seven_day":{"utilization":40.0,"resets_at":"2026-08-21T00:00:00+00:00"},"seven_day_sonnet":null}"#;
  let ( server, port ) = serve_once( body, "" );

  std::env::set_var( claude_quota::BASE_URL_ENV, format!( "http://127.0.0.1:{port}" ) );
  let result = claude_quota::fetch_oauth_usage( "seam-test-token" );
  std::env::remove_var( claude_quota::BASE_URL_ENV );

  let data = result.expect( "fetch_oauth_usage must succeed against the local server" );
  let five = data.five_hour.expect( "five_hour bucket present" );
  assert!( ( five.utilization - 12.5 ).abs() < f64::EPSILON, "five_hour utilization parsed, got {}", five.utilization );
  assert_eq!( five.resets_at.as_deref(), Some( "2026-08-18T04:00:00+00:00" ) );
  let seven = data.seven_day.expect( "seven_day bucket present" );
  assert!( ( seven.utilization - 40.0 ).abs() < f64::EPSILON );
  assert!( data.seven_day_sonnet.is_none(), "null bucket must map to None" );

  let request = String::from_utf8_lossy( &server.join().expect( "server thread" ) ).to_string();
  assert!(
    request.starts_with( "GET /api/oauth/usage HTTP/1.1\r\n" ),
    "override must preserve the endpoint path, got request head:\n{request}"
  );
  assert!(
    request.contains( "Bearer seam-test-token" ),
    "Authorization header must reach the server, got:\n{request}"
  );
}

/// SM-02: `fetch_rate_limits` against a loopback override — the local server
/// receives `POST /v1/messages` with the quota probe body, and the canned
/// rate-limit response headers parse into `RateLimitData`.
#[ test ]
fn sm02_rate_limits_via_loopback_override()
{
  let _guard = ENV_LOCK.lock().expect( "env lock" );
  let headers = "anthropic-ratelimit-unified-5h-utilization: 0.25\r\n\
    anthropic-ratelimit-unified-5h-reset: 1755500000\r\n\
    anthropic-ratelimit-unified-7d-utilization: 0.6\r\n\
    anthropic-ratelimit-unified-7d-reset: 1755900000\r\n\
    anthropic-ratelimit-unified-status: allowed\r\n";
  let ( server, port ) = serve_once( "{}", headers );

  std::env::set_var( claude_quota::BASE_URL_ENV, format!( "http://127.0.0.1:{port}" ) );
  let result = claude_quota::fetch_rate_limits( "seam-test-token" );
  std::env::remove_var( claude_quota::BASE_URL_ENV );

  let data = result.expect( "fetch_rate_limits must succeed against the local server" );
  assert!( ( data.utilization_5h - 0.25 ).abs() < f64::EPSILON );
  assert_eq!( data.reset_5h, 1_755_500_000 );
  assert!( ( data.utilization_7d - 0.6 ).abs() < f64::EPSILON );
  assert_eq!( data.reset_7d, 1_755_900_000 );
  assert_eq!( data.status, "allowed" );

  let request = String::from_utf8_lossy( &server.join().expect( "server thread" ) ).to_string();
  assert!(
    request.starts_with( "POST /v1/messages HTTP/1.1\r\n" ),
    "override must preserve the endpoint path, got request head:\n{request}"
  );
  assert!(
    request.contains( r#""max_tokens":1"# ),
    "quota probe body must reach the server, got:\n{request}"
  );
}

/// SM-03: a plaintext override pointing anywhere but loopback must never
/// produce a successful fetch — `https_only` stays enforced for non-loopback
/// hosts, so the call fails at scheme rejection (or, at worst, at DNS for the
/// reserved `.invalid` TLD). Either path is an `Err`; the property pinned here
/// is that the loopback carve-out cannot silently downgrade transport security
/// toward a remote host.
#[ test ]
fn sm03_non_loopback_http_override_errs()
{
  let _guard = ENV_LOCK.lock().expect( "env lock" );
  std::env::set_var( claude_quota::BASE_URL_ENV, "http://plaintext-remote.invalid" );
  let result = claude_quota::fetch_oauth_usage( "seam-test-token" );
  std::env::remove_var( claude_quota::BASE_URL_ENV );

  assert!(
    result.is_err(),
    "plaintext non-loopback override must not yield a successful fetch"
  );
}

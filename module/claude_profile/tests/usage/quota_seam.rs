//! Loopback HTTP server for `CLAUDE_QUOTA_BASE_URL` seam tests.
//!
//! A real `TcpListener`-backed server (no mocks) serving canned per-path JSON
//! responses while recording every request's line and Bearer token. Lets tests
//! observe *which* HTTP calls the usage pipeline actually makes — turning
//! previously-structural order-of-operations proofs into behavioral ones.
//! [`QuotaSeamServer::start_rate_limiting`] answers `429` for chosen tokens, so a test
//! can also drive an account whose quota fetch fails.
//!
//! Two consumption modes:
//! - in-process (`tests/usage/*`): wrap the pipeline call in [`with_seam_env`],
//!   which points `claude_quota::BASE_URL_ENV` at [`QuotaSeamServer::origin`];
//! - subprocess (`tests/cli/*`): pass `( claude_quota::BASE_URL_ENV, origin )`
//!   through `run_cs_with_env` — no in-process env mutation involved.

use std::io::{ Read, Write };
use std::net::{ TcpListener, TcpStream };
use std::sync::{ Arc, Mutex };

/// One recorded request: request line (e.g. `GET /api/oauth/usage HTTP/1.1`)
/// plus the Bearer token extracted from its `Authorization` header, if any.
#[ derive( Debug, Clone ) ]
pub struct SeamRequest
{
  /// The HTTP request line.
  pub line   : String,
  /// Bearer token from the `Authorization` header (`None` when absent).
  pub bearer : Option< String >,
}

/// Canned oauth-usage body with healthy, distinctive quota buckets.
pub const USAGE_BODY : &str = r#"{"five_hour":{"utilization":12.5,"resets_at":"2026-08-18T04:00:00+00:00"},"seven_day":{"utilization":40.0,"resets_at":"2026-08-21T00:00:00+00:00"},"seven_day_sonnet":null}"#;

/// Read one full HTTP request (head + `Content-Length` body) from the stream.
fn read_request( stream : &mut TcpStream ) -> Vec< u8 >
{
  stream.set_read_timeout( Some( core::time::Duration::from_secs( 10 ) ) ).expect( "set_read_timeout" );
  let mut buf   = Vec::new();
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

// BUG-591 — a rate-limited token gets 429 on the usage path, so a seam test can drive a failing fetch
/// Canned response for a request path: rate-limit headers for `/v1/messages`, and `429` on
/// `/api/oauth/usage` when the request's token is one the server rate-limits.
fn canned_response( path : &str, rate_limited : bool ) -> String
{
  let ( status, extra_headers, body ) = if path.starts_with( "/api/oauth/usage" ) && rate_limited
  {
    ( "429 Too Many Requests", "", r#"{"error":{"type":"rate_limit_error","message":"Rate limited"}}"# )
  }
  else if path.starts_with( "/api/oauth/usage" )
  {
    ( "200 OK", "", USAGE_BODY )
  }
  else if path.starts_with( "/v1/messages" )
  {
    (
      "200 OK",
      "anthropic-ratelimit-unified-5h-utilization: 0.10\r\n\
       anthropic-ratelimit-unified-5h-reset: 1755500000\r\n\
       anthropic-ratelimit-unified-7d-utilization: 0.20\r\n\
       anthropic-ratelimit-unified-7d-reset: 1755900000\r\n\
       anthropic-ratelimit-unified-status: allowed\r\n",
      "{}",
    )
  }
  else if path.starts_with( "/api/oauth/account" )
    || path.starts_with( "/api/oauth/claude_cli/roles" )
    || path.starts_with( "/v1/models" )
  {
    // Minimal 200 — consumers treat parse failure as absent optional data.
    ( "200 OK", "", "{}" )
  }
  else
  {
    ( "404 Not Found", "", "{}" )
  };
  format!(
    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n{extra_headers}Connection: close\r\n\r\n{body}",
    body.len()
  )
}

/// Loopback recorder server: serves canned responses per path until shut down,
/// recording every non-sentinel request.
#[ derive( Debug ) ]
pub struct QuotaSeamServer
{
  origin   : String,
  requests : Arc< Mutex< Vec< SeamRequest > > >,
  handle   : Option< std::thread::JoinHandle< () > >,
  port     : u16,
}

impl QuotaSeamServer
{
  /// Bind a fresh loopback listener and start the accept loop.
  ///
  /// # Panics
  ///
  /// Panics on socket setup failure — seam fixtures fail loudly rather than
  /// letting a test run against a half-up server.
  #[ inline ]
  #[ must_use ]
  pub fn start() -> Self
  {
    Self::start_rate_limiting( &[] )
  }

  /// [`Self::start`], except that a usage request whose Bearer token is in `rate_limited`
  /// gets `429 Too Many Requests` instead of quota.
  ///
  /// # Panics
  ///
  /// Panics on socket setup failure, as [`Self::start`] does.
  #[ inline ]
  #[ must_use ]
  pub fn start_rate_limiting( rate_limited : &[ &str ] ) -> Self
  {
    let listener = TcpListener::bind( "127.0.0.1:0" ).expect( "bind loopback listener" );
    let port     = listener.local_addr().expect( "local_addr" ).port();
    let requests : Arc< Mutex< Vec< SeamRequest > > > = Arc::new( Mutex::new( Vec::new() ) );
    let recorded = requests.clone();
    let limited : Vec< String > = rate_limited.iter().map( | t | ( *t ).to_string() ).collect();
    let handle = std::thread::spawn( move ||
    {
      loop
      {
        let Ok( ( mut stream, _addr ) ) = listener.accept() else { break; };
        let raw  = read_request( &mut stream );
        let text = String::from_utf8_lossy( &raw ).to_string();
        let line = text.lines().next().unwrap_or( "" ).to_string();
        if line.starts_with( "DONE " ) { break; }
        let bearer = text.lines().find_map( | l |
        {
          let low = l.to_ascii_lowercase();
          low.starts_with( "authorization:" ).then( || l.split( "Bearer " ).nth( 1 ).map( str::trim ) )?
            .map( str::to_string )
        } );
        let path = line.split( ' ' ).nth( 1 ).unwrap_or( "" ).to_string();
        let rate_limited = bearer.as_deref().is_some_and( | b | limited.iter().any( | t | t == b ) );
        recorded.lock().expect( "requests lock" ).push( SeamRequest { line, bearer } );
        let _ = stream.write_all( canned_response( &path, rate_limited ).as_bytes() );
      }
    } );
    Self
    {
      origin : format!( "http://127.0.0.1:{port}" ),
      requests,
      handle : Some( handle ),
      port,
    }
  }

  /// The `http://127.0.0.1:PORT` origin to point [`claude_quota::BASE_URL_ENV`] at.
  #[ inline ]
  #[ must_use ]
  pub fn origin( &self ) -> &str { &self.origin }

  /// Snapshot of all requests recorded so far (sentinel excluded).
  ///
  /// # Panics
  ///
  /// Panics if the internal request log lock is poisoned.
  #[ inline ]
  #[ must_use ]
  pub fn requests( &self ) -> Vec< SeamRequest >
  {
    self.requests.lock().expect( "requests lock" ).clone()
  }

  /// Distinct Bearer tokens across all recorded requests, sorted.
  #[ inline ]
  #[ must_use ]
  pub fn bearer_tokens( &self ) -> Vec< String >
  {
    let mut tokens : Vec< String > = self.requests().into_iter().filter_map( | r | r.bearer ).collect();
    tokens.sort();
    tokens.dedup();
    tokens
  }

  /// Stop the accept loop (sentinel connection) and join the server thread.
  fn stop( &mut self )
  {
    if let Some( handle ) = self.handle.take()
    {
      if let Ok( mut s ) = TcpStream::connect( ( "127.0.0.1", self.port ) )
      {
        let _ = s.write_all( b"DONE / HTTP/1.1\r\nContent-Length: 0\r\n\r\n" );
      }
      let _ = handle.join();
    }
  }
}

impl Drop for QuotaSeamServer
{
  #[ inline ]
  fn drop( &mut self ) { self.stop(); }
}

/// Env-mutation lock — `CLAUDE_QUOTA_BASE_URL` is process-global, so plain
/// `cargo test` (threads, one process) would race without serialization.
/// Under nextest (process per test, the sanctioned runner) this is a no-op.
static ENV_LOCK : Mutex< () > = Mutex::new( () );

/// Removes the seam env var on drop — restores state even if `f` panics.
struct EnvGuard;

impl Drop for EnvGuard
{
  fn drop( &mut self ) { std::env::remove_var( claude_quota::BASE_URL_ENV ); }
}

/// Run `f` with `claude_quota::BASE_URL_ENV` pointing at `origin` (in-process).
///
/// # Panics
///
/// Panics if the env lock is poisoned.
#[ inline ]
pub fn with_seam_env< R >( origin : &str, f : impl FnOnce() -> R ) -> R
{
  let _lock  = ENV_LOCK.lock().expect( "env lock" );
  let _guard = EnvGuard;
  std::env::set_var( claude_quota::BASE_URL_ENV, origin );
  f()
}

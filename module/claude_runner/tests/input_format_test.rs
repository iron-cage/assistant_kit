//! Unix-only integration tests.
#![ cfg( unix ) ]
//! `--input-format` Integration Tests
//!
//! Covers IT-1 through IT-6 from `task/claude_runner/executing/414_implement_sdk_protocol_run_command.md`.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, fake_claude_dir, run_with_path };

use std::io::{ BufRead, BufReader };
use std::process::Stdio;
use std::sync::mpsc;
use core::time::Duration;

fn assert_container()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: cd module/claude_runner && ./verb/test\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

/// Spawn the `clr` binary with piped stdout against a custom `PATH`.
///
/// Unlike `run_with_path` (which blocks on `.output()`), this returns the live
/// `Child` so callers can read stdout incrementally — required to prove events
/// arrive strictly before subprocess exit (IT-4/IT-5).
fn spawn_clr_piped( args : &[ &str ], path_val : &str ) -> std::process::Child
{
  assert_container();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  std::process::Command::new( bin )
    .args( args )
    .env( "PATH", path_val )
    .stdout( Stdio::piped() )
    .stderr( Stdio::null() )
    .spawn()
    .expect( "failed to spawn clr with piped stdout" )
}

// ── IT-1: --input-format stream-json → forwarded to assembled command ─────────

/// IT-1: `--input-format stream-json` appears in the assembled command; exit 0.
#[ test ]
fn it1_input_format_stream_json_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--input-format", "stream-json", "hi" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--input-format" ),
    "assembled command must contain --input-format. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "stream-json" ),
    "assembled command must contain the value stream-json. Got:\n{stdout}"
  );
}

/// IT-1b: `--input-format text` also forwarded (the other valid enum value).
#[ test ]
fn it1b_input_format_text_forwarded()
{
  let out = run_cli( &[ "--dry-run", "--input-format", "text", "hi" ] );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--input-format" ),
    "assembled command must contain --input-format. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "text" ),
    "assembled command must contain the value text. Got:\n{stdout}"
  );
}

// ── IT-2: --input-format badvalue → exit 1, stderr names valid values ─────────

/// IT-2: an invalid `--input-format` value exits 1 and stderr names the valid values.
#[ test ]
fn it2_input_format_invalid_value_rejected()
{
  let out = run_cli( &[ "--input-format", "badvalue", "hi" ] );
  assert_eq!(
    out.status.code(),
    Some( 1 ),
    "exit must be 1 for an invalid --input-format value: {out:?}"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "text" ) && stderr.contains( "stream-json" ),
    "stderr must name the valid values (text, stream-json). Got:\n{stderr}"
  );
}

// ── IT-3: --help lists --input-format ──────────────────────────────────────────

/// IT-3: `clr --help` output contains `--input-format`.
#[ test ]
fn it3_input_format_help_listed()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr --help must exit 0" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "--input-format" ),
    "`clr --help` must list --input-format. Got:\n{stdout}"
  );
}

// ── IT-4: --output-format stream-json → live incremental consumption ──────────

/// IT-4: at least 2 NDJSON events are observably read from `clr`'s own stdout
/// while the fake `claude` subprocess is still alive — not merely present in the
/// final collected output. Proven two ways: (a) each event arrives well within its
/// own timing window (not only after the fake process's total ~4s runtime), and
/// (b) `child.try_wait()` returns `None` (still running) immediately after the
/// second event is received.
#[ test ]
fn it4_stream_json_events_consumed_before_exit()
{
  let body = "echo '{\"seq\":1,\"type\":\"system\"}'\n\
              sleep 2\n\
              echo '{\"seq\":2,\"type\":\"assistant\"}'\n\
              sleep 2\n\
              echo '{\"seq\":3,\"type\":\"result\"}'";
  let ( _dir, path ) = fake_claude_dir( body );

  let mut child = spawn_clr_piped(
    &[ "--output-format", "stream-json", "--max-sessions", "0", "hi" ],
    &path,
  );
  let stdout = child.stdout.take().expect( "stdout piped" );

  let ( tx, rx ) = mpsc::channel();
  let reader = std::thread::spawn( move ||
  {
    let buf = BufReader::new( stdout );
    for line in buf.lines()
    {
      match line
      {
        Ok( l ) => { if tx.send( l ).is_err() { break; } }
        Err( _ ) => break,
      }
    }
  } );

  let start = std::time::Instant::now();
  let line1 = rx.recv_timeout( Duration::from_millis( 1800 ) )
    .expect( "first NDJSON event must arrive well before the fake process's ~4s total runtime" );
  assert!( line1.contains( "\"seq\":1" ), "first event must be seq 1. Got: {line1}" );

  let line2 = rx.recv_timeout( Duration::from_millis( 2200 ) )
    .expect( "second NDJSON event must arrive before the fake process exits" );
  assert!( line2.contains( "\"seq\":2" ), "second event must be seq 2. Got: {line2}" );

  // Direct liveness proof: the fake claude process still has its final `sleep 2` +
  // exit ahead of it — the subprocess (and therefore clr, which waits on it) must
  // still be running right after the second event is consumed.
  assert!(
    child.try_wait().expect( "try_wait" ).is_none(),
    "child must still be running when the 2nd event is received (proves incremental, not batched, consumption); elapsed={:?}",
    start.elapsed()
  );

  let status = child.wait().expect( "wait for clr exit" );
  assert!( status.success(), "clr must exit 0 after the fake claude completes" );
  reader.join().expect( "reader thread must not panic" );
}

// ── IT-5: stream-json events observed in emission order ───────────────────────

/// IT-5: NDJSON events are observed by the Rust caller, live, in the same order
/// the subprocess emitted them.
///
/// Reads events one at a time as they arrive (via a bounded per-event deadline),
/// not by draining to EOF after the subprocess exits — a `lines()`-to-`Vec` collect
/// after full completion would preserve order trivially for both a batched *and*
/// a streaming implementation (a single pipe is FIFO regardless of buffering
/// strategy), so it would not actually discriminate pre/post-Phase-1b. Asserting
/// order incrementally, interleaved with each event's arrival deadline, exercises
/// the live path the same way IT-4 does.
#[ test ]
fn it5_stream_json_events_preserve_order()
{
  let body = "echo '{\"seq\":1,\"type\":\"system\"}'\n\
              sleep 1\n\
              echo '{\"seq\":2,\"type\":\"assistant\"}'\n\
              sleep 1\n\
              echo '{\"seq\":3,\"type\":\"result\"}'";
  let ( _dir, path ) = fake_claude_dir( body );

  let mut child = spawn_clr_piped(
    &[ "--output-format", "stream-json", "--max-sessions", "0", "hi" ],
    &path,
  );
  let stdout = child.stdout.take().expect( "stdout piped" );

  let ( tx, rx ) = mpsc::channel();
  let reader = std::thread::spawn( move ||
  {
    let buf = BufReader::new( stdout );
    for line in buf.lines()
    {
      match line
      {
        Ok( l ) => { if tx.send( l ).is_err() { break; } }
        Err( _ ) => break,
      }
    }
  } );

  for expected_seq in 1 ..= 3
  {
    let line = rx.recv_timeout( Duration::from_millis( 1800 ) )
      .unwrap_or_else( | e | panic!( "event seq {expected_seq} must arrive live, not only after full completion: {e}" ) );
    let marker = format!( "\"seq\":{expected_seq}" );
    assert!(
      line.contains( &marker ),
      "event {expected_seq} arrived out of order — expected {marker}, got: {line}"
    );
  }

  let status = child.wait().expect( "wait for clr exit" );
  assert!( status.success(), "clr must exit 0: status={status:?}" );
  reader.join().expect( "reader thread must not panic" );
}

// ── IT-6: --output-format json (no stream-json) → unchanged batched behavior ──

/// IT-6: regression guard — without `stream-json`, output is still delivered via
/// the pre-existing batched path (raw passthrough, unchanged from the pre-task
/// baseline).
#[ test ]
fn it6_non_stream_json_output_format_unchanged()
{
  let ( _dir, path ) = fake_claude_dir( "echo 'plain claude output'" );
  let out = run_with_path(
    &[ "--output-format", "json", "--output-style", "raw", "--max-sessions", "0", "hi" ],
    &path,
  );
  assert!( out.status.success(), "exit must be 0: {out:?}" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert_eq!(
    stdout.trim(), "plain claude output",
    "non-stream-json output must be delivered unchanged via the batched path. Got:\n{stdout}"
  );
}

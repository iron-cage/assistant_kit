//! Edge case tests for `--inspect` flag of `clr ps`.
//!
//! Test spec: [`tests/docs/cli/param/069_inspect.md`](docs/cli/param/069_inspect.md).
//!
//! # Test Case Index
//!
//! | ID   | Name                                                                          | Category     |
//! |------|-------------------------------------------------------------------------------|--------------|
//! | EC-1 | `--inspect` output is key:value blocks, not a table                           | Behavioral   |
//! | EC-2 | `--inspect` output contains all 12 attribute keys                             | Behavioral   |
//! | EC-3 | `--inspect --pid <A>` shows inspect block for session A only                   | Interaction  |
//! | EC-4 | `--inspect --mode print` shows inspect blocks for print-mode sessions only     | Interaction  |
//! | EC-5 | `--inspect --columns pid` ignores `--columns`; still shows all 12 keys        | Interaction  |
//! | EC-6 | `--inspect --wide` ignores `--wide`; still shows all 12 keys                  | Interaction  |
//! | EC-7 | `--inspect` suppresses queued CLR processes table                             | Behavioral   |
//! | EC-8 | `--inspect` with no sessions → empty-state message, exit 0                    | Behavioral   |
//! | EC-9 | `clr ps --help` output contains `--inspect`                                   | Documentation|

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{ fake_claude_binary_dir, spawn_fake_claude };

// The 12 expected attribute key prefixes present in every inspect block.
#[ cfg( unix ) ]
const INSPECT_KEYS : &[ &str ] = &[
  "pid:", "mode:", "elapsed:", "cpu:", "ram:", "state:",
  "path:", "task:", "binary:", "cmd:", "cmdline:", "started:",
];

// ── EC-1: key:value blocks, not a table ───────────────────────────────────

/// EC-1: `--inspect` output contains key:value lines, not a table header row.
///
/// The table header `Active Sessions` and dash separator lines should be absent.
#[ cfg( unix ) ]
#[ test ]
fn ec1_inspect_output_is_key_value_not_table()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--inspect" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --inspect" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-1: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "pid:" ),
    "EC-1: key:value line 'pid:' must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "Active Sessions" ),
    "EC-1: table header 'Active Sessions' must NOT appear in inspect mode. Got:\n{stdout}"
  );
}

// ── EC-2: All 12 attribute keys present ───────────────────────────────────

/// EC-2: `--inspect` output contains all 12 attribute keys for each session.
#[ cfg( unix ) ]
#[ test ]
fn ec2_inspect_contains_all_12_keys()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--inspect" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --inspect" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-2: exit 0 expected, got {:?}", out.status.code() );
  for key in INSPECT_KEYS
  {
    assert!(
      stdout.contains( key ),
      "EC-2: inspect attribute key '{key}' must appear. Got:\n{stdout}"
    );
  }
}

// ── EC-3: --inspect + --pid ────────────────────────────────────────────────

/// EC-3: `--inspect --pid <A>` shows inspect block for session A; session B absent.
#[ cfg( unix ) ]
#[ test ]
fn ec3_inspect_with_pid_shows_only_matching()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_a = spawn_fake_claude( &path_val );
  let pid_a    = bg_a.id();
  let mut bg_b = spawn_fake_claude( &path_val );
  let pid_b    = bg_b.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--inspect", "--pid", &pid_a.to_string() ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --inspect --pid <A>" );

  let _ = bg_a.kill(); let _ = bg_a.wait();
  let _ = bg_b.kill(); let _ = bg_b.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-3: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_a.to_string() ),
    "EC-3: PID A {pid_a} must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_b.to_string() ),
    "EC-3: PID B {pid_b} must NOT appear. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( "pid:" ),
    "EC-3: inspect key 'pid:' must appear. Got:\n{stdout}"
  );
}

// ── EC-4: --inspect + --mode ───────────────────────────────────────────────

/// EC-4: `--inspect --mode interactive` shows only interactive session inspect blocks.
#[ cfg( unix ) ]
#[ test ]
fn ec4_inspect_with_mode_filter_applied()
{
  use cli_binary_test_helpers::spawn_print_claude;

  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive    = bg_interactive.id();
  let mut bg_print       = spawn_print_claude( &path_val );
  let pid_print          = bg_print.id();

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--inspect", "--mode", "interactive" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --inspect --mode interactive" );

  let _ = bg_interactive.kill(); let _ = bg_interactive.wait();
  let _ = bg_print.kill();       let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-4: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_interactive.to_string() ),
    "EC-4: interactive PID {pid_interactive} must appear. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_print.to_string() ),
    "EC-4: print PID {pid_print} must NOT appear with --mode interactive. Got:\n{stdout}"
  );
}

// ── EC-5: --inspect ignores --columns ─────────────────────────────────────

/// EC-5: `--inspect --columns pid` ignores `--columns`; all 12 keys still appear.
#[ cfg( unix ) ]
#[ test ]
fn ec5_inspect_ignores_columns()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--inspect", "--columns", "pid" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --inspect --columns pid" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-5: exit 0 expected, got {:?}", out.status.code() );
  for key in INSPECT_KEYS
  {
    assert!(
      stdout.contains( key ),
      "EC-5: inspect attribute '{key}' must appear even with --columns pid. Got:\n{stdout}"
    );
  }
}

// ── EC-6: --inspect ignores --wide ────────────────────────────────────────

/// EC-6: `--inspect --wide` ignores `--wide`; all 12 keys still appear in key:value format.
#[ cfg( unix ) ]
#[ test ]
fn ec6_inspect_ignores_wide()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--inspect", "--wide" ] )
    .env( "PATH", &path_val )
    .output()
    .expect( "run clr ps --inspect --wide" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-6: exit 0 expected, got {:?}", out.status.code() );
  for key in INSPECT_KEYS
  {
    assert!(
      stdout.contains( key ),
      "EC-6: inspect attribute '{key}' must appear even with --wide. Got:\n{stdout}"
    );
  }
  assert!(
    !stdout.contains( "Active Sessions" ),
    "EC-6: table header must NOT appear in inspect mode. Got:\n{stdout}"
  );
}

// ── EC-7: --inspect suppresses queued table ────────────────────────────────

/// EC-7: `--inspect` suppresses the Queued CLR Processes table.
///
/// A fake gate file is injected via `CLR_GATE_DIR` to simulate a queued process.
/// With `--inspect`, the queued section must not appear in the output.
#[ cfg( unix ) ]
#[ test ]
fn ec7_inspect_suppresses_queued_table()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  // Create a fake gate dir with a mock gate file so a queued table WOULD appear
  // without --inspect.
  let gate_dir = tempfile::TempDir::new().expect( "tmpdir" );
  // Write a gate file with a large PID unlikely to exist (ensures liveness filter
  // retains the file only if the PID is alive — use a real but separate PID: 1,
  // which is always alive (init/systemd)).
  let gate_content = r#"{"cwd":"/tmp","since":1000000,"attempt":0,"message":"waiting"}"#;
  std::fs::write( gate_dir.path().join( "1.json" ), gate_content )
    .expect( "write gate file" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--inspect" ] )
    .env( "PATH", &path_val )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .output()
    .expect( "run clr ps --inspect with gate file present" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-7: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    !stdout.contains( "Queued" ),
    "EC-7: 'Queued' table header must NOT appear in --inspect mode. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "waiting" ),
    "EC-7: 'waiting' (queued table field) must NOT appear in --inspect mode. Got:\n{stdout}"
  );
}

// ── EC-8: --inspect with no sessions → empty state ────────────────────────

/// EC-8: `--inspect` with no matching sessions → empty-state message, exit 0.
#[ test ]
fn ec8_inspect_no_sessions_empty_state()
{
  // Use a non-existent PID so no session matches.
  let out    = run_cli( &[ "ps", "--inspect", "--pid", "99999999" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-8: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "No active Claude Code sessions" ),
    "EC-8: empty-state message expected. Got:\n{stdout}"
  );
}

// ── EC-9: Help contains --inspect ─────────────────────────────────────────

/// EC-9: `clr ps --help` stdout contains `--inspect`.
#[ test ]
fn ec9_help_contains_inspect()
{
  let out    = run_cli( &[ "ps", "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "EC-9: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "--inspect" ),
    "EC-9: --help must document --inspect. Got: {stdout}"
  );
}

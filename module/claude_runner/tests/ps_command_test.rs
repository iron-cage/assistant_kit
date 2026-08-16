//! Integration tests for `clr ps` — the session listing command.
//!
//! Test spec: [`tests/docs/cli/command/06_ps.md`](docs/cli/command/06_ps.md).
//!
//! IT-20–IT-37, IT-38 (mode-query filter) live in `ps_command_ext_test.rs`.
//!
//! # Test Case Index
//!
//! | ID   | Name                                        | Category         |
//! |------|---------------------------------------------|------------------|
//! | IT-1 | 0 sessions → no-sessions message            | No-sessions      |
//! | IT-2  | ≥1 session → plain table (no `┌` border)      | Sessions present |
//! | IT-3  | `clr --help` lists `ps`                       | Help listing     |
//! | IT-4  | `clr p` (typo) → exit 1, Did you mean         | Typo guard       |
//! | IT-5  | table contains PID, Elapsed, Absolute Path, Task | Column presence |
//! | IT-6  | `clr pss` (typo) → exit 1, Did you mean       | Typo guard       |
//! | IT-7  | own PID not in `clr ps` output                | Self-exclusion   |
//! | IT-8  | `clr ps --unknown` → exit 1                   | Error handling   |
//! | IT-9  | `$PRO` prefix replaced by `"$PRO"` in path    | Path shortening  |
//! | IT-10 | Gate file present → queued table with headers  | Queued present   |
//! | IT-11 | No gate files → no queued table in output      | Queued absent    |
//! | IT-12 | Active table caption contains `Active Sessions` and interactive/print breakdown | Caption presence |
//! | IT-13 | Orphaned gate file (dead PID) filtered out of queued table    | BUG-293 repro    |
//! | IT-14 | `clr ps --help` → exit 0, stdout non-empty                    | BUG-294 help     |
//! | IT-15 | `clr ps -h` → exit 0, stdout non-empty                       | BUG-294 short    |
//! | IT-16 | Task column extracts Form A content for underscore CWD         | BUG-295/296/297  |
//! | IT-17 | Task column selects Form A over Form B `tool_result` lines      | BUG-297 repro    |
//! | IT-18 | `clr ps help` (positional) → exit 0, stdout non-empty         | BUG-294 positional|
//! | IT-19 | Task column works for CWD with no underscores (regression)     | BUG-295 regression|
//! | IT-46 | Zombie-PID gate file filtered out of queued table + self-healed | BUG-479 repro    |
//! | IT-47 | Thread-TID gate file filtered out of queued table + self-healed | BUG-488 repro    |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{
  fake_claude_binary_dir, make_proc_dir, run_clr_ps_proc, spawn_fake_claude,
};

#[ cfg( target_os = "linux" ) ]
use cli_binary_test_helpers::spawn_parked_helper_thread;

// ── IT-1: 0 sessions ──────────────────────────────────────────────────────────

/// IT-1: `clr ps` with 0 sessions → exit 0, no-sessions message.
///
/// `CLR_PROC_DIR` is set to an empty temp dir so `find_claude_processes()`
/// sees no entries, regardless of live Claude sessions on the host.
#[ test ]
fn it_01_no_sessions_shows_message()
{
  let empty_proc = tempfile::TempDir::new().expect( "create empty proc dir" );
  let proc_dir   = empty_proc.path().to_str().expect( "proc dir UTF-8" );
  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_PROC_DIR", proc_dir ) ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit code must be 0, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "No active Claude Code sessions." ),
    "stdout must contain the no-sessions message, got: {stdout}"
  );
}

// ── IT-2: ≥1 session → plain-style table ──────────────────────────────────────

/// IT-2: with a fake `claude` process running, `clr ps` exits 0 and the
/// output uses plain style — no unicode box-drawing border character (`┌`)
/// in the table structure (caption + header).
///
/// Note: only the first non-blank line is checked for `┌` because task-column
/// data from real host sessions may contain unicode characters; the table
/// STRUCTURE never has `┌` in plain style (only in box-drawing style).
#[ cfg( unix ) ]
#[ test ]
fn it_02_sessions_present_plain_style()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit code must be 0, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "PID" ),
    "stdout must contain PID header, got: {stdout}"
  );
  let first_line = stdout.lines().find( |l| !l.trim().is_empty() ).unwrap_or( "" );
  assert!(
    !first_line.contains( '\u{250C}' ), // must NOT have ┌ in the caption/header line
    "table caption must use plain style — no ┌ border, got first line: {first_line}"
  );
}

// ── IT-3: help lists ps ───────────────────────────────────────────────────────

/// IT-3: `clr --help` lists the `ps` subcommand.
#[ test ]
fn it_03_help_lists_ps()
{
  let out = run_cli( &[ "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "ps" ),
    "help output must mention ps, got: {stdout}"
  );
}

// ── IT-4: typo guard `clr p` ─────────────────────────────────────────────────

/// IT-4: `clr p` (truncation typo) → exit 1, stderr: "Did you mean 'ps'?"
#[ test ]
fn it_04_typo_clr_p()
{
  let out = run_cli( &[ "p" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "Did you mean" ),
    "stderr must contain 'Did you mean', got: {stderr}"
  );
}

// ── IT-5: table headers present ───────────────────────────────────────────────

/// IT-5: with a session running, `clr ps` output contains the expected
/// column headers: PID, Elapsed, Absolute Path, Task.
#[ cfg( unix ) ]
#[ test ]
fn it_05_table_headers_present()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ), "missing PID header: {stdout}" );
  assert!( stdout.contains( "Elapsed" ), "missing Elapsed header: {stdout}" );
  assert!( stdout.contains( "Absolute Path" ), "missing Absolute Path header: {stdout}" );
  assert!( stdout.contains( "Task" ), "missing Task header: {stdout}" );
}

// ── IT-6: typo guard `clr pss` ───────────────────────────────────────────────

/// IT-6: `clr pss` (extension typo) → exit 1, stderr: "Did you mean 'ps'?"
#[ test ]
fn it_06_typo_clr_pss()
{
  let out = run_cli( &[ "pss" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "Did you mean" ),
    "stderr must contain 'Did you mean', got: {stderr}"
  );
}

// ── IT-7: self-exclusion ──────────────────────────────────────────────────────

/// IT-7: the `clr ps` process's own PID is not listed in the output.
///
/// `find_claude_processes()` excludes `std::process::id()` (the caller).
/// We verify end-to-end by spawning a fake `claude` (so the table is non-empty)
/// and then checking that the test-runner PID (which is NOT a `claude` binary)
/// does not appear in the table.
#[ cfg( unix ) ]
#[ test ]
fn it_07_self_exclusion()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  // The test-runner PID is not a `claude` binary, so it must not appear in the table.
  let self_pid = std::process::id().to_string();
  assert!(
    !stdout.contains( &self_pid ),
    "test-runner PID {self_pid} must not appear in ps output: {stdout}"
  );
}

// ── IT-8: unknown flag ────────────────────────────────────────────────────────

/// IT-8: `clr ps --unknown` → exit 1 with an error message on stderr.
#[ test ]
fn it_08_unknown_flag()
{
  let out = run_cli( &[ "ps", "--unknown" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "expected non-zero exit" );
  assert!(
    stderr.contains( "unexpected argument" ),
    "stderr must mention unexpected argument, got: {stderr}"
  );
}

// ── IT-9: $PRO prefix shortened ───────────────────────────────────────────────

/// IT-9: when `PRO` env var is set and a session CWD starts with that prefix,
/// the Absolute Path column shows `$PRO/…` rather than the full path.
///
/// `shorten_path()` replaces the `$PRO` prefix with the literal `"$PRO"` string;
/// the user already knows what `$PRO` expands to, keeping rows compact.
#[ cfg( unix ) ]
#[ test ]
fn it_09_pro_prefix_shortened_in_path_column()
{
  let pro_dir = tempfile::TempDir::new().expect( "create tmp PRO dir" );
  let sub_dir = pro_dir.path().join( "my" ).join( "project" );
  std::fs::create_dir_all( &sub_dir ).expect( "create project subdir" );
  let pro_str = pro_dir.path().to_str().expect( "PRO path is UTF-8" );

  let ( _bin_dir, path_val ) = fake_claude_binary_dir();
  let mut bg = std::process::Command::new( "claude" )
    .env( "PATH", &path_val )
    .arg( "30" )
    .current_dir( &sub_dir )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude in sub_dir" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );
  let proc = make_proc_dir( &[ bg.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "PRO", pro_str )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps with PRO set" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "$PRO" ),
    "IT-9: path must be shortened to $PRO/… when PRO env var is set. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( pro_str ),
    "IT-9: full PRO prefix must not appear in the table. Got:\n{stdout}"
  );
}

// BUG-479 task/claude_runner/bug/479_zombie_blind_pid_liveness.md — fixed: IT-10's any-live-PID
// fixture enshrined zombie-blind rendering; the zombie-waiter
// must-not-render/self-heal case is IT-46 below.
// ── IT-10: gate file present → queued table ───────────────────────────────────

/// IT-10: when a gate JSON file exists in `CLR_GATE_DIR`, `clr ps` exits 0
/// and stdout contains the queued table headers (PID, CWD, Waiting).
///
/// Uses the test process's own PID so the `/proc/{pid}` liveness filter
/// passes — gate files with dead PIDs are filtered out (BUG-293).
///
/// Linux-only: the liveness filter probes `/proc/{pid}` which does not exist
/// on Windows or macOS.
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it_10_gate_file_present_shows_queued_table()
{
  let gate_dir      = tempfile::TempDir::new().expect( "create gate temp dir" );
  let gate_dir_path = gate_dir.path().to_str().expect( "gate dir UTF-8" );
  let live_pid      = std::process::id();
  let gate_file     = gate_dir.path().join( format!( "{live_pid}.json" ) );
  std::fs::write(
    &gate_file,
    r#"{"cwd":"/tmp/test-project","since":1720000000,"attempt":3,"message":"waiting for session slot"}"#,
  ).expect( "write gate file" );
  let proc          = make_proc_dir( &[] );
  let proc_dir_path = proc.path().to_str().expect( "proc dir UTF-8" );

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ), ( "CLR_PROC_DIR", proc_dir_path ) ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ), "missing PID header in queued table: {stdout}" );
  assert!( stdout.contains( "CWD" ), "missing CWD header in queued table: {stdout}" );
  assert!( stdout.contains( "Waiting" ), "missing Waiting header in queued table: {stdout}" );
}

// ── IT-11: no gate files → no queued table ────────────────────────────────────

/// IT-11: when `CLR_GATE_DIR` points to an empty temp dir, `clr ps` exits 0
/// and stdout does NOT contain queued table headers.
///
/// `CLR_PROC_DIR` is set to a separate empty temp dir so the active-session
/// scanner returns zero results regardless of live host sessions.
#[ test ]
fn it_11_no_gate_files_no_queued_table()
{
  let gate_dir      = tempfile::TempDir::new().expect( "create gate temp dir" );
  let gate_dir_path = gate_dir.path().to_str().expect( "gate dir UTF-8" );
  let empty_proc    = tempfile::TempDir::new().expect( "create empty proc dir" );
  let proc_dir      = empty_proc.path().to_str().expect( "proc dir UTF-8" );
  // Both dirs are intentionally empty.

  let out    = run_cli_with_env(
    &[ "ps" ],
    &[ ( "CLR_GATE_DIR", gate_dir_path ), ( "CLR_PROC_DIR", proc_dir ) ],
  );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "No active Claude Code sessions." ),
    "must show no-sessions message with empty gate dir: {stdout}"
  );
  assert!(
    !stdout.contains( "Waiting" ),
    "must not contain Waiting header when no gate files: {stdout}"
  );
  assert!(
    !stdout.contains( "Attempt" ),
    "must not contain Attempt header when no gate files: {stdout}"
  );
}

// ── IT-12: active table caption ───────────────────────────────────────────────

/// IT-12: with a fake `claude` process running, the active sessions table output
/// contains the titled caption rule line ("Active Sessions · N running (I interactive,
/// P print, Q query)") above the column headers, under the default `--mode all`.
///
/// The heading is rendered by `Heading::new("Active Sessions").with_field(...)` via
/// `data_fmt`; this test confirms end-to-end that the heading text — including the
/// interactive/print breakdown — appears in the output and is not accidentally
/// dropped by the formatter.
#[ cfg( unix ) ]
#[ test ]
fn it_12_active_table_has_caption()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "Active Sessions" ),
    "IT-12: active table caption must contain 'Active Sessions', got: {stdout}"
  );
  assert!(
    stdout.contains( "1 running (1 interactive, 0 print, 0 query)" ),
    "IT-12: active table caption must contain the interactive/print/query breakdown, got: {stdout}"
  );
}

// ── IT-13: orphaned gate file filtered out (BUG-293) ────────────────────────

/// IT-13 (BUG-293): a gate file whose PID does not exist on the system is
/// filtered out by `build_queued_table()` and does NOT appear in the queued table.
///
/// ## Root Cause
/// `build_queued_table()` read every `.json` file in the gate directory without
/// probing `/proc/{pid}` — orphaned files from killed processes displayed as live.
///
/// ## Why Not Caught
/// IT-10/IT-11 tested happy paths only (file present/absent); no test verified
/// liveness filtering for a non-existent PID.
///
/// ## Fix Applied
/// Added `/proc/{pid}` existence check in the `.filter()` closure of
/// `build_queued_table()` with self-healing `remove_file` on orphan detection.
///
/// ## Prevention
/// Any table displaying PID-keyed state files must probe OS-level PID existence
/// before rendering a row.
///
/// ## Pitfall
/// PID 99999999 is safe for testing (far above typical `PID_MAX` of 32768/4194304),
/// but `/proc/{pid}` probes on live PIDs are racy — only use guaranteed-dead PIDs.
// test_kind: bug_reproducer(BUG-293)
#[ test ]
fn it_13_orphaned_gate_file_filtered_out()
{
  let gate_dir      = tempfile::TempDir::new().expect( "create gate temp dir" );
  let gate_dir_path = gate_dir.path().to_str().expect( "gate dir UTF-8" );
  let proc          = make_proc_dir( &[] );
  let proc_dir_path = proc.path().to_str().expect( "proc dir UTF-8" );

  // PID 99999999 is guaranteed not to exist (/proc/sys/kernel/pid_max is at most 4194304).
  let orphan_file = gate_dir.path().join( "99999999.json" );
  std::fs::write(
    &orphan_file,
    r#"{"cwd":"/tmp/dead-process","since":1,"attempt":1,"message":"waiting for session slot"}"#,
  ).expect( "write orphan gate file" );

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ), ( "CLR_PROC_DIR", proc_dir_path ) ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );

  // The queued table must NOT appear — the only gate file is orphaned.
  assert!(
    !stdout.contains( "Queued" ),
    "IT-13 (BUG-293): orphaned gate file must not produce a queued table. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "99999999" ),
    "IT-13 (BUG-293): orphaned PID must not appear in output. Got:\n{stdout}"
  );

  // Self-healing: the orphan file must have been deleted by the liveness filter.
  assert!(
    !orphan_file.exists(),
    "IT-13 (BUG-293): orphaned gate file must be deleted by self-healing cleanup"
  );
}

// ── IT-14: `clr ps --help` → exit 0 ──────────────────────────────────────────

/// IT-14 (BUG-294): `clr ps --help` must exit 0 and print help text.
///
/// Before fix: `dispatch_ps()` rejected `--help` as "unexpected argument" (exit 1).
/// After fix: matches `"--help" | "-h" | "help"` and calls `print_ps_help()`.
// test_kind: bug_reproducer(BUG-294)
#[ test ]
fn it_14_ps_help_flag()
{
  let out    = run_cli( &[ "ps", "--help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-14: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    !stdout.is_empty(),
    "IT-14: stdout must contain help text, got empty output"
  );
}

// ── IT-15: `clr ps -h` → exit 0 ──────────────────────────────────────────────

/// IT-15 (BUG-294): `clr ps -h` must exit 0 and print help text.
// test_kind: bug_reproducer(BUG-294)
#[ test ]
fn it_15_ps_h_flag()
{
  let out    = run_cli( &[ "ps", "-h" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-15: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    !stdout.is_empty(),
    "IT-15: stdout must contain help text, got empty output"
  );
}

// ── IT-16: Task column — Form A extraction with underscore CWD ────────────────

/// IT-16: `clr ps` Task column shows Form A content for a session whose CWD
/// contains underscores.
///
/// ## Root Cause (BUG-295, BUG-296, BUG-297)
/// Three compounding bugs in `try_jsonl_task()`:
/// BUG-295 — path encoding only replaced `/` with `-`, missing `_`;
/// BUG-296 — content marker was `"text":"` but Claude uses `"content":"` in Form A;
/// BUG-297 — `.find()` predicate matched the last `"type":"user"` line regardless of
/// whether it was a Form A (human text) or Form B (`tool_result` array) entry.
///
/// ## Why Not Caught
/// No test verified end-to-end Task column content for a real underscore-containing CWD.
///
/// ## Fix Applied
/// BUG-295: `replace('/', "-").replace('_', "-")`;
/// BUG-296: marker changed to `"content":"`;
/// BUG-297: predicate requires `"content":"` and excludes `"content":[`.
///
/// ## Prevention
/// Use `replace('/', "-").replace('_', "-")` for all CWD-to-project-dir encoding.
/// Always verify the JSONL field name against Claude's actual serialization format.
///
/// ## Pitfall
/// `run_clr_ps()` only sets PATH; use `std::process::Command` directly to inject HOME.
// test_kind: bug_reproducer(BUG-295, BUG-296, BUG-297)
#[ cfg( unix ) ]
#[ test ]
fn it_16_task_column_form_a()
{
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  // CWD with underscores — triggers BUG-295 without the fix.
  let proj_tmp = tempfile::TempDir::new().expect( "create project tmp" );
  let cwd      = proj_tmp.path().join( "wip_core" ).join( "proj" );
  std::fs::create_dir_all( &cwd ).expect( "create CWD with underscores" );
  // Spawn fake claude in the underscore CWD and wait for it to appear in /proc.
  let mut bg = std::process::Command::new( "claude" )
    .arg( "30" )
    .env( "PATH", &path_val )
    .current_dir( &cwd )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );
  let proc = make_proc_dir( &[ bg.id() ] );

  // Build synthetic JSONL at the correctly-encoded project path.
  let encoded      = claude_storage_core::encode_path( &cwd ).expect( "encode cwd" );
  let home_tmp     = tempfile::TempDir::new().expect( "create temp HOME" );
  let project_path = home_tmp.path()
    .join( ".claude" ).join( "projects" ).join( &encoded );
  std::fs::create_dir_all( &project_path ).expect( "create project path" );
  std::fs::write(
    project_path.join( "session.jsonl" ),
    r#"{"type":"user","message":{"role":"user","content":"fix the auth module"}}"#,
  ).expect( "write JSONL" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "HOME", home_tmp.path() )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "fix the auth module" ),
    "IT-16 (BUG-295/296/297): Task column must show Form A content. Got:\n{stdout}"
  );
}

// ── IT-17: Task column — Form A selected over Form B ─────────────────────────

/// IT-17: When JSONL contains a Form A entry followed by Form B `tool_result` lines,
/// `clr ps` Task column shows the Form A content, not the Form B text.
///
/// ## Root Cause (BUG-297)
/// See IT-16. Without the Form A predicate, `.rev().find()` returns the last
/// `"type":"user"` line, which is the Form B `tool_result` entry in any active session.
///
/// ## Why Not Caught
/// No test verified Form A vs Form B line selection in the presence of both.
///
/// ## Fix Applied
/// Same as IT-16 (BUG-297 predicate fix).
///
/// ## Prevention
/// Always test with a JSONL containing both Form A and Form B entries.
///
/// ## Pitfall
/// Form B outer `"content"` is always a JSON array `[...]`; inner `tool_result` `content`
/// may be a string or array — the exclusion `"content":[` targets the outer form only.
// test_kind: bug_reproducer(BUG-297)
#[ cfg( unix ) ]
#[ test ]
fn it_17_task_column_form_a_over_form_b()
{
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  let proj_tmp = tempfile::TempDir::new().expect( "create project tmp" );
  let cwd      = proj_tmp.path().join( "wip_core" ).join( "proj" );
  std::fs::create_dir_all( &cwd ).expect( "create CWD" );
  let mut bg = std::process::Command::new( "claude" )
    .arg( "30" )
    .env( "PATH", &path_val )
    .current_dir( &cwd )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );
  let proc = make_proc_dir( &[ bg.id() ] );

  let encoded      = claude_storage_core::encode_path( &cwd ).expect( "encode cwd" );
  let home_tmp     = tempfile::TempDir::new().expect( "create temp HOME" );
  let project_path = home_tmp.path()
    .join( ".claude" ).join( "projects" ).join( &encoded );
  std::fs::create_dir_all( &project_path ).expect( "create project path" );

  // Form A (line 1) followed by Form B tool_result (line 2) — Form A must win.
  let jsonl = "{\
    \"type\":\"user\",\
    \"message\":{\"role\":\"user\",\"content\":\"the actual task\"}}\n\
    {\"type\":\"user\",\
    \"message\":{\"role\":\"user\",\"content\":[\
      {\"type\":\"tool_result\",\"tool_use_id\":\"tu_abc\",\
       \"content\":[{\"type\":\"text\",\"text\":\"claude command::some_skill\"}]}]}}";
  std::fs::write( project_path.join( "session.jsonl" ), jsonl )
    .expect( "write JSONL" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "HOME", home_tmp.path() )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "the actual task" ),
    "IT-17 (BUG-297): Task must show Form A content. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "some_skill" ),
    "IT-17 (BUG-297): Form B tool_result text must not appear. Got:\n{stdout}"
  );
}

// ── IT-18: `clr ps help` (positional) → exit 0 ───────────────────────────────

/// IT-18 (BUG-294): `clr ps help` (positional token) must exit 0 and print help text.
// test_kind: bug_reproducer(BUG-294)
#[ test ]
fn it_18_ps_help_positional()
{
  let out    = run_cli( &[ "ps", "help" ] );
  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-18: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    !stdout.is_empty(),
    "IT-18: stdout must contain help text, got empty output"
  );
}

// ── IT-19: Task column — no-underscore CWD regression ────────────────────────

/// IT-19: `clr ps` Task column works for a session whose CWD contains no
/// underscores — regression guard for the BUG-295 fix.
///
/// ## Root Cause (BUG-385, fixture-only — superseded the BUG-295 note below)
/// This test's own fixture line hand-rolled `cwd_str.replace('/', "-")` instead of
/// calling `claude_storage_core::encode_path()`. That diverged from production once
/// `encode_path()` was generalized (BUG-366) to map every non-alphanumeric character,
/// not just `_`, to `-` — `tempfile::TempDir`'s dot-prefixed names exposed the gap.
///
/// ## Why Not Caught
/// IT-16 (line 526) was already updated to call `encode_path()` directly when `ps.rs`
/// switched to it; this test's separate, duplicate encoding line never was, and no
/// check enforces that fixture-encoding logic stays in sync with the shared function.
///
/// ## Fix Applied
/// Replaced the hand-rolled `cwd_str.replace('/', "-")` with a direct call to
/// `claude_storage_core::encode_path(&cwd)`, matching IT-16's pattern. The `cwd_str`
/// binding (only used by the old encoding line) was removed as unused.
///
/// ## Prevention
/// Never hand-roll production encoding/formatting logic inside a test fixture — call
/// the shared function directly so the fixture cannot drift when that function's
/// behavior changes.
///
/// ## Pitfall
/// A fixture encoding that matches production "for this specific input" is not proof
/// it will keep matching — `replace('_', "-")` alone looked equivalent to slash-only
/// replacement for a no-underscore CWD until `encode_path()`'s substitution scope
/// widened to cover dots too.
#[ cfg( unix ) ]
#[ test ]
fn it_19_task_column_no_underscores()
{
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  // CWD with no underscores — regression guard.
  let proj_tmp = tempfile::TempDir::new().expect( "create project tmp" );
  let cwd      = proj_tmp.path().join( "work" ).join( "proj" );
  std::fs::create_dir_all( &cwd ).expect( "create CWD without underscores" );

  let mut bg = std::process::Command::new( "claude" )
    .arg( "30" )
    .env( "PATH", &path_val )
    .current_dir( &cwd )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );
  let proc = make_proc_dir( &[ bg.id() ] );

  // Fix(BUG-385): encode via the same shared function try_jsonl_task() calls in
  // production — never hand-roll the encoding, so this fixture cannot drift from
  // real lookup behavior (matches IT-16's already-correct pattern at line 526).
  // Root cause: this line used to hand-roll `cwd_str.replace('/', "-")`, which
  // diverged from claude_storage_core::encode_path() once that function was
  // generalized (BUG-366) to convert every non-alphanumeric character (not just
  // '_') to '-' — tempfile::TempDir's dot-prefixed names exposed the divergence.
  // Pitfall: a test fixture that duplicates production encoding logic instead of
  // calling the shared function directly silently diverges the moment that
  // function's behavior changes — always call the real function from fixtures.
  let encoded      = claude_storage_core::encode_path( &cwd ).expect( "encode cwd" );
  let home_tmp     = tempfile::TempDir::new().expect( "create temp HOME" );
  let project_path = home_tmp.path()
    .join( ".claude" ).join( "projects" ).join( &encoded );
  std::fs::create_dir_all( &project_path ).expect( "create project path" );
  std::fs::write(
    project_path.join( "session.jsonl" ),
    r#"{"type":"user","message":{"role":"user","content":"no underscores task"}}"#,
  ).expect( "write JSONL" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "HOME", home_tmp.path() )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-19: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "no underscores task" ),
    "IT-19 (BUG-295 regression): Task column must show Form A content for underscore-free CWD. Got:\n{stdout}"
  );
}

// ── IT-46: zombie waiter filtered out + self-healed (BUG-479) ────────────────

/// IT-46 (BUG-479): a queued-waiter gate file whose PID is an exited-but-unreaped
/// (zombie) process must be filtered out of the Queued table AND self-heal-deleted,
/// exactly like the absent-PID orphan IT-13 covers.
///
/// ## Root Cause
/// `build_queued_table()`'s liveness filter probed bare `/proc/{pid}` existence;
/// a zombie keeps that entry for as long as its parent fails to `wait()`, so dead
/// waiters rendered as queued rows indefinitely and the self-heal deletion never
/// fired (observed live: `Queued · 84 waiting` with only 4 waiters actually alive).
///
/// ## Why Not Caught
/// IT-13 covers only the absent-PID orphan (spawned AND reaped, so `/proc/{pid}`
/// is gone); IT-10 uses the test's own live PID. The exited-but-unreaped middle
/// state had no fixture, so existence == liveness went unchallenged.
///
/// ## Fix Applied
/// The filter now calls the shared zombie-aware `pid_alive()` predicate exported
/// from `gate.rs` (`/proc/{pid}/stat` readable AND state field ≠ `Z`), so zombie
/// waiters fail liveness and are self-heal-deleted.
///
/// ## Prevention
/// One authoritative liveness predicate for every PID-keyed display/reclaim
/// decision, with fixtures for all three PID states: live, zombie, absent.
///
/// ## Pitfall
/// Spawning-and-reaping produces an ABSENT PID (IT-13's case), not a zombie — to
/// fixture a zombie, spawn a real process and deliberately never `wait()` on it
/// while the parent (this test) stays alive.
// test_kind: bug_reproducer(BUG-479)
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it_46_zombie_waiter_filtered_out_and_self_healed()
{
  let gate_dir      = tempfile::TempDir::new().expect( "create gate temp dir" );
  let gate_dir_path = gate_dir.path().to_str().expect( "gate dir UTF-8" );
  let proc          = make_proc_dir( &[] );
  let proc_dir_path = proc.path().to_str().expect( "proc dir UTF-8" );

  // Manufacture a genuine zombie: spawn a real, immediately-exiting process and
  // do NOT wait() on it — this test process is its parent, so until the reap at
  // the end it stays state Z with a live /proc/{pid} entry.
  let mut zombie = std::process::Command::new( "true" ).spawn().expect( "spawn zombie-to-be" );
  let zombie_pid = zombie.id();
  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 5 );
  loop
  {
    let stat = std::fs::read_to_string( format!( "/proc/{zombie_pid}/stat" ) ).unwrap_or_default();
    if stat.rsplit_once( ')' ).is_some_and( | ( _, rest ) | rest.trim_start().starts_with( 'Z' ) ) { break; }
    assert!( std::time::Instant::now() < deadline, "fixture: PID {zombie_pid} never became a zombie" );
    std::thread::sleep( core::time::Duration::from_millis( 20 ) );
  }

  let waiter_file = gate_dir.path().join( format!( "{zombie_pid}.json" ) );
  std::fs::write(
    &waiter_file,
    r#"{"cwd":"/tmp/zombie-waiter","since":1,"attempt":240,"message":"waiting for session slot"}"#,
  ).expect( "write zombie waiter gate file" );

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ), ( "CLR_PROC_DIR", proc_dir_path ) ] );
  let stdout = stdout_str( &out );

  let _ = zombie.wait(); // reap only after clr ps has run

  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    !stdout.contains( "Queued" ),
    "IT-46 (BUG-479): zombie-PID gate file must not produce a queued table. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &zombie_pid.to_string() ),
    "IT-46 (BUG-479): zombie PID must not appear in output. Got:\n{stdout}"
  );
  assert!(
    !waiter_file.exists(),
    "IT-46 (BUG-479): zombie waiter gate file must be deleted by self-healing cleanup"
  );
}

// ── IT-47: thread-TID waiter filtered out + self-healed (BUG-488) ────────────

/// IT-47 (BUG-488): a queued-waiter gate file whose PID number is currently
/// occupied by a live NON-LEADER thread of an unrelated process must be
/// filtered out of the Queued table AND self-heal-deleted — the recorded
/// waiter is dead; the thread merely masks its number.
///
/// ## Root Cause
/// The queued-table liveness filter shares `pid_alive()`, which probed only
/// `/proc/{pid}/stat` readability and state ∉ {`Z`}. Linux resolves direct
/// `/proc/<tid>` lookups for non-leader thread IDs (readdir-invisible, yet
/// stat-readable with a running state), so a dead waiter whose number a live
/// thread occupied rendered as a phantom Queued row indefinitely (observed
/// live: dockerd startup thread TID 1744061 shown queued for 76+ hours).
///
/// ## Why Not Caught
/// IT-13 covers the absent PID, IT-46 the zombie; both leave `/proc/<pid>`
/// either gone or state `Z`. No fixture produced a PID number resolving to a
/// live non-leader thread, so the `Tgid` dimension was never exercised.
///
/// ## Fix Applied
/// `pid_alive()` clause (c): `/proc/{pid}/status` must report `Tgid == pid`.
/// The ps filter inherits it through the shared predicate — thread-masked
/// waiters fail liveness, drop from the table, and are self-heal-deleted.
///
/// ## Prevention
/// One authoritative liveness predicate for every PID-keyed display/reclaim
/// decision, with fixtures for all four PID states: live leader, zombie,
/// absent, thread-masked.
///
/// ## Pitfall
/// A parked helper thread of the test's own process is a deterministic
/// thread-TID fixture — no PID-wrap lottery needed; `/proc/thread-self`
/// yields the TID without a libc `gettid()` binding.
// test_kind: bug_reproducer(BUG-488)
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it_47_thread_tid_waiter_filtered_out_and_self_healed()
{
  let gate_dir      = tempfile::TempDir::new().expect( "create gate temp dir" );
  let gate_dir_path = gate_dir.path().to_str().expect( "gate dir UTF-8" );
  let proc          = make_proc_dir( &[] );
  let proc_dir_path = proc.path().to_str().expect( "proc dir UTF-8" );

  let ( tid, park_send, park_handle ) = spawn_parked_helper_thread();
  // Fixture validity: stat readable via direct lookup, not a zombie, NOT a
  // thread-group leader — the exact occupancy shape that masked BUG-488.
  let stat = std::fs::read_to_string( format!( "/proc/{tid}/stat" ) )
    .expect( "fixture: /proc/<tid>/stat must be readable via direct lookup" );
  assert!(
    stat.rsplit_once( ')' ).is_some_and( | ( _, rest ) | !rest.trim_start().starts_with( 'Z' ) ),
    "fixture: TID {tid} must not be a zombie"
  );
  let reported_tgid = std::fs::read_to_string( format!( "/proc/{tid}/status" ) )
    .expect( "fixture: /proc/<tid>/status must be readable" )
    .lines()
    .find_map( | l | l.strip_prefix( "Tgid:" ).and_then( | v | v.trim().parse::< u32 >().ok() ) );
  assert!(
    reported_tgid.is_some_and( | t | t != tid ),
    "fixture: TID {tid} must be a non-leader thread (Tgid {reported_tgid:?})"
  );

  let waiter_file = gate_dir.path().join( format!( "{tid}.json" ) );
  std::fs::write(
    &waiter_file,
    r#"{"cwd":"/tmp/thread-waiter","since":1,"attempt":9,"message":"waiting for session slot"}"#,
  ).expect( "write thread-masked waiter gate file" );

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ), ( "CLR_PROC_DIR", proc_dir_path ) ] );
  let stdout = stdout_str( &out );

  drop( park_send ); // release the parked helper thread
  let _ = park_handle.join();

  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    !stdout.contains( "Queued" ),
    "IT-47 (BUG-488): thread-TID gate file must not produce a queued table. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &tid.to_string() ),
    "IT-47 (BUG-488): thread TID must not appear in output. Got:\n{stdout}"
  );
  assert!(
    !waiter_file.exists(),
    "IT-47 (BUG-488): thread-masked waiter gate file must be deleted by self-healing cleanup"
  );
}

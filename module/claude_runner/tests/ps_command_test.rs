//! Integration tests for `clr ps` — the session listing command.
//!
//! Test spec: [`tests/docs/cli/command/06_ps.md`](docs/cli/command/06_ps.md).
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
//! | IT-20 | Active sessions ordered oldest-first (row #1 has longest elapsed) | BUG-301 repro   |
//! | IT-21 | `--mode print` shows only print-mode sessions                  | Mode filter      |
//! | IT-22 | `--mode interactive` shows only interactive sessions           | Mode filter      |
//! | IT-23 | `--mode bogus` → exit 1                                        | Mode validation  |
//! | IT-24 | `--columns pid,path,task` shows custom column subset           | Column select    |
//! | IT-25 | `--columns bogus` → exit 1                                     | Column validation|
//! | IT-26 | `--wide` shows all 11 columns                                  | Wide output      |
//! | IT-27 | `--wide --columns pid,task` → `--columns` wins                 | Precedence       |
//! | IT-28 | `CLR_PS_MODE=print` env var fallback filters print sessions    | Env var          |
//! | IT-29 | `CLR_PS_COLUMNS=pid,elapsed` env var fallback selects columns  | Env var          |
//! | IT-30 | `--mode all`, 1 interactive + 1 print → breakdown "1 interactive, 1 print" | Caption breakdown |
//! | IT-31 | `--mode all`, 3 interactive → breakdown "3 interactive, 0 print"          | Caption breakdown |
//! | IT-32 | `--mode all`, 3 print → breakdown "0 interactive, 3 print"                | Caption breakdown |
//! | IT-33 | `--mode interactive`, mixed set → plain caption, no breakdown             | Caption plain     |
//! | IT-34 | `--mode print`, mixed set → plain caption, no breakdown                   | Caption plain     |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{
  fake_claude_binary_dir, make_proc_dir, run_clr_ps_proc,
  spawn_fake_claude, spawn_print_claude,
};

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
/// P print)") above the column headers, under the default `--mode all`.
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
    stdout.contains( "1 running (1 interactive, 0 print)" ),
    "IT-12: active table caption must contain the interactive/print breakdown, got: {stdout}"
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

// ── IT-20: active sessions sorted oldest-first (BUG-301) ────────────────────

/// IT-20 (BUG-301): `build_active_table()` sorts rows by `started_at` so the
/// oldest session appears at row `#1` with the longest elapsed time.
///
/// ## Root Cause
/// `build_active_table()` iterated `procs.iter().enumerate()` in `/proc` scan
/// order (PID-ascending) with no sort — PID order only approximates age order
/// and breaks on PID rollover.
///
/// ## Why Not Caught
/// IT-01–IT-19 checked row presence and content but never verified ordering.
///
/// ## Fix Applied
/// `sort_by_key()` using `read_process_metrics(p.pid).map(|m| m.started_at)`
/// inserted after the `procs.is_empty()` guard in `build_active_table()`.
///
/// ## Prevention
/// Always add an ordering assertion when implementing a "sorted by X" requirement.
///
/// ## Pitfall
/// PID-ascending order approximates age order on most Linux systems (monotonic
/// PID allocation), masking the bug until PID rollover.  Use a 1-second sleep
/// between spawns to guarantee distinct `started_at` values.
// test_kind: bug_reproducer(BUG-301)
#[ cfg( target_os = "linux" ) ]
#[ test ]
fn it_20_active_sessions_sorted_by_age()
{
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  // Spawn process A (oldest session).
  let mut bg_a = std::process::Command::new( "claude" )
    .arg( "30" )
    .env( "PATH", &path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude A" );
  let pid_a = bg_a.id();

  // 1-second gap guarantees distinct started_at values in /proc/{pid}/stat.
  std::thread::sleep( core::time::Duration::from_secs( 1 ) );

  // Spawn process B (newer session).
  let mut bg_b = std::process::Command::new( "claude" )
    .arg( "30" )
    .env( "PATH", &path_val )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude B" );
  let pid_b = bg_b.id();

  std::thread::sleep( core::time::Duration::from_millis( 200 ) );
  let proc = make_proc_dir( &[ pid_a, pid_b ] );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  let _ = bg_a.kill();
  let _ = bg_a.wait();
  let _ = bg_b.kill();
  let _ = bg_b.wait();

  let stdout = stdout_str( &out );
  assert!(
    out.status.success(),
    "IT-20: exit 0 expected, got {:?}", out.status.code()
  );

  // Oldest session (A) must appear before newest (B) in the table output.
  let older_pid = pid_a.to_string();
  let newer_pid = pid_b.to_string();
  let row_a = stdout.lines().position( |l| l.contains( &older_pid ) );
  let row_b = stdout.lines().position( |l| l.contains( &newer_pid ) );
  assert!(
    row_a.is_some() && row_b.is_some(),
    "IT-20 (BUG-301): both PIDs must appear in output. A={pid_a}, B={pid_b}\n{stdout}"
  );
  assert!(
    row_a.unwrap() < row_b.unwrap(),
    "IT-20 (BUG-301): oldest session (PID {pid_a}) must appear before newest (PID {pid_b}).\n{stdout}"
  );
}

// ── IT-21: `--mode print` shows only print-mode sessions ─────────────────────

/// IT-21: `clr ps --mode print` shows only sessions whose cmdline args include `--print`.
#[ cfg( unix ) ]
#[ test ]
fn it_21_mode_print_shows_only_print_sessions()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();
  let proc         = make_proc_dir( &[ pid_interactive, pid_print ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "print" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --mode print" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-21: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "IT-21: print-mode PID {pid_print} must appear with --mode print. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_interactive.to_string() ),
    "IT-21: interactive PID {pid_interactive} must NOT appear with --mode print. Got:\n{stdout}"
  );
}

// ── IT-22: `--mode interactive` shows only interactive sessions ───────────────

/// IT-22: `clr ps --mode interactive` shows only sessions without `--print` in cmdline.
#[ cfg( unix ) ]
#[ test ]
fn it_22_mode_interactive_shows_only_interactive_sessions()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();
  let proc         = make_proc_dir( &[ pid_interactive, pid_print ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "interactive" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --mode interactive" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-22: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_interactive.to_string() ),
    "IT-22: interactive PID {pid_interactive} must appear with --mode interactive. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_print.to_string() ),
    "IT-22: print-mode PID {pid_print} must NOT appear with --mode interactive. Got:\n{stdout}"
  );
}

// ── IT-23: `--mode bogus` → exit 1 ───────────────────────────────────────────

/// IT-23: `clr ps --mode bogus` exits 1 with stderr listing valid mode values.
#[ test ]
fn it_23_mode_bogus_exits_1()
{
  let out    = run_cli( &[ "ps", "--mode", "bogus" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "IT-23: exit 1 expected, got {:?}", out.status.code() );
  assert!(
    stderr.contains( "interactive" ) && stderr.contains( "print" ),
    "IT-23: stderr must list valid mode values (interactive, print). Got: {stderr}"
  );
}

// ── IT-24: `--columns pid,path,task` shows custom column subset ───────────────

/// IT-24: `clr ps --columns pid,path,task` shows PID, Absolute Path, Task
/// and does NOT show CPU%, RAM, State, Elapsed.
#[ cfg( unix ) ]
#[ test ]
fn it_24_columns_custom_subset()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--columns", "pid,path,task" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --columns" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-24: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),           "IT-24: PID must be present: {stdout}" );
  assert!( stdout.contains( "Absolute Path" ), "IT-24: Absolute Path must be present: {stdout}" );
  assert!( stdout.contains( "Task" ),          "IT-24: Task must be present: {stdout}" );
  // Header-only check — legend "🐘 High RAM" would false-positive whole-stdout search.
  let header = stdout.lines().find( | l | l.contains( "PID" ) ).unwrap_or( "" );
  assert!( !header.contains( "CPU%" ),    "IT-24: CPU% must be absent from headers: {stdout}" );
  assert!( !header.contains( "RAM" ),     "IT-24: RAM must be absent from headers: {stdout}" );
  assert!( !header.contains( "Elapsed" ), "IT-24: Elapsed must be absent from headers: {stdout}" );
  assert!( !header.contains( "State" ),   "IT-24: State must be absent from headers: {stdout}" );
}

// ── IT-25: `--columns bogus` → exit 1 ────────────────────────────────────────

/// IT-25: `clr ps --columns bogus` exits 1 with stderr listing valid column keys.
#[ test ]
fn it_25_columns_bogus_exits_1()
{
  let out    = run_cli( &[ "ps", "--columns", "bogus" ] );
  let stderr = stderr_str( &out );
  assert!( !out.status.success(), "IT-25: exit 1 expected, got {:?}", out.status.code() );
  assert!(
    stderr.contains( "bogus" ) && ( stderr.contains( "pid" ) || stderr.contains( "idx" ) ),
    "IT-25: stderr must contain the unknown key and list valid keys. Got: {stderr}"
  );
}

// ── IT-26: `--wide` shows all 11 columns ─────────────────────────────────────

/// IT-26: `clr ps --wide` shows all 11 columns including Mode, Command, Binary.
#[ cfg( unix ) ]
#[ test ]
fn it_26_wide_shows_all_columns()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--wide" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --wide" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-26: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "Mode" ),    "IT-26: Mode header must be present: {stdout}" );
  assert!( stdout.contains( "Command" ), "IT-26: Command header must be present: {stdout}" );
  assert!( stdout.contains( "Binary" ),  "IT-26: Binary header must be present: {stdout}" );
}

// ── IT-27: `--wide --columns pid,task` → `--columns` wins ────────────────────

/// IT-27: When both `--wide` and `--columns` are given, `--columns` wins.
#[ cfg( unix ) ]
#[ test ]
fn it_27_columns_wins_over_wide()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );
  let proc   = make_proc_dir( &[ bg.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--wide", "--columns", "pid,task" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --wide --columns" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-27: exit 0 expected, got {:?}", out.status.code() );
  let header = stdout.lines().find( | l | l.contains( "PID" ) ).unwrap_or( "" );
  assert!( header.contains( "PID" ),  "IT-27: PID must be present in header: {stdout}" );
  assert!( header.contains( "Task" ), "IT-27: Task must be present in header: {stdout}" );
  assert!( !header.contains( "Mode" ),    "IT-27: Mode must be absent from header when --columns wins: {stdout}" );
  assert!( !header.contains( "Command" ), "IT-27: Command must be absent from header when --columns wins: {stdout}" );
  assert!( !header.contains( "Binary" ),  "IT-27: Binary must be absent from header when --columns wins: {stdout}" );
}

// ── IT-28: `CLR_PS_MODE=print` env var fallback ──────────────────────────────

/// IT-28: `CLR_PS_MODE=print` env var applies the print mode filter.
#[ cfg( unix ) ]
#[ test ]
fn it_28_clr_ps_mode_env_var()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();

  let mut bg_interactive = spawn_fake_claude( &path_val );
  let pid_interactive     = bg_interactive.id();

  let mut bg_print = spawn_print_claude( &path_val );
  let pid_print    = bg_print.id();
  let proc         = make_proc_dir( &[ pid_interactive, pid_print ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "CLR_PS_MODE", "print" )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps with CLR_PS_MODE=print" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-28: exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( &pid_print.to_string() ),
    "IT-28: print PID {pid_print} must appear with CLR_PS_MODE=print. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( &pid_interactive.to_string() ),
    "IT-28: interactive PID {pid_interactive} must NOT appear with CLR_PS_MODE=print. Got:\n{stdout}"
  );
}

// ── IT-29: `CLR_PS_COLUMNS=pid,elapsed` env var fallback ─────────────────────

/// IT-29: `CLR_PS_COLUMNS=pid,elapsed` env var selects PID and Elapsed columns only.
///
/// `CLR_PROC_DIR` is set to a fake proc dir containing only the background process
/// so `find_claude_processes()` returns exactly one entry regardless of ambient sessions.
/// Pitfall: without `CLR_PROC_DIR`, ambient claude processes on the host cause
/// `clr ps` to find unexpected process counts, producing row/header mismatches that
/// panic in `RowBuilder::validate_row_length`.
#[ cfg( unix ) ]
#[ test ]
fn it_29_clr_ps_columns_env_var()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let fake_proc     = tempfile::TempDir::new().expect( "fake_proc" );
  let fake_proc_str = fake_proc.path().to_str().expect( "fake_proc UTF-8" );
  std::os::unix::fs::symlink(
    format!( "/proc/{}", bg.id() ),
    fake_proc.path().join( bg.id().to_string() ),
  ).expect( "pid symlink" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "CLR_PS_COLUMNS", "pid,elapsed" )
    .env( "CLR_PROC_DIR", fake_proc_str )
    .output()
    .expect( "run clr ps with CLR_PS_COLUMNS=pid,elapsed" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-29: exit 0 expected, got {:?}", out.status.code() );
  assert!( stdout.contains( "PID" ),     "IT-29: PID must be present: {stdout}" );
  assert!( stdout.contains( "Elapsed" ), "IT-29: Elapsed must be present: {stdout}" );
  // Header-only check — legend "🐘 High RAM" would false-positive whole-stdout search.
  let header = stdout.lines().find( | l | l.contains( "PID" ) ).unwrap_or( "" );
  assert!( !header.contains( "CPU%" ),          "IT-29: CPU% must be absent from headers: {stdout}" );
  assert!( !header.contains( "RAM" ),           "IT-29: RAM must be absent from headers: {stdout}" );
  assert!( !header.contains( "Task" ),          "IT-29: Task must be absent from headers: {stdout}" );
  assert!( !header.contains( "Absolute Path" ), "IT-29: Absolute Path must be absent from headers: {stdout}" );
}

// ── Caption breakdown helper (AF1 anti-faking check) ─────────────────────────

/// Locate the `"Active Sessions"` caption line in `stdout`. Panics with the
/// full `stdout` on failure so assertion failures are self-diagnosing.
#[ cfg( unix ) ]
fn caption_line( stdout : &str ) -> &str
{
  stdout.lines().find( | l | l.contains( "Active Sessions" ) )
    .unwrap_or_else( || panic!( "no 'Active Sessions' caption line found in:\n{stdout}" ) )
}

/// Parse the leading `N` from `"N running..."` in the `Active Sessions` caption,
/// anchored on the full whitespace-delimited numeric token — never a raw
/// substring match, which would let e.g. `"11 running"` false-positive against
/// an assertion checking for `"1 running"`.
#[ cfg( unix ) ]
fn parse_running_count( stdout : &str ) -> usize
{
  let line = caption_line( stdout );
  let running_pos = line.find( " running" )
    .unwrap_or_else( || panic!( "caption missing ' running' suffix:\n{line}" ) );
  line[ ..running_pos ].rsplit( char::is_whitespace ).next()
    .unwrap_or_else( || panic!( "caption missing N before 'running':\n{line}" ) )
    .parse().unwrap_or_else( |e| panic!( "N not numeric ({e}):\n{line}" ) )
}

/// Parse `"Active Sessions · N running (I interactive, P print)"` from `stdout`,
/// returning `(N, I, P)`. Panics with the full `stdout` on any parse failure so
/// assertion failures are self-diagnosing.
#[ cfg( unix ) ]
fn parse_breakdown_counts( stdout : &str ) -> ( usize, usize, usize )
{
  let n = parse_running_count( stdout );
  let line = caption_line( stdout );
  let open  = line.find( '(' ).unwrap_or_else( || panic!( "caption missing '(' breakdown:\n{line}" ) );
  let close = line.find( ')' ).unwrap_or_else( || panic!( "caption missing ')' breakdown:\n{line}" ) );
  let mut parts = line[ open + 1 .. close ].split( ", " );
  let i : usize = parts.next().unwrap_or_else( || panic!( "missing interactive part:\n{line}" ) )
    .split_whitespace().next().unwrap_or_else( || panic!( "missing I number:\n{line}" ) )
    .parse().unwrap_or_else( |e| panic!( "I not numeric ({e}):\n{line}" ) );
  let p : usize = parts.next().unwrap_or_else( || panic!( "missing print part:\n{line}" ) )
    .split_whitespace().next().unwrap_or_else( || panic!( "missing P number:\n{line}" ) )
    .parse().unwrap_or_else( |e| panic!( "P not numeric ({e}):\n{line}" ) );
  ( n, i, p )
}

// ── IT-30 (T01): `--mode all`, 1 interactive + 1 print → breakdown ─────────────

/// IT-30 (T01): 2 active sessions (1 interactive + 1 print), `--mode all` (default)
/// → caption reads "2 running (1 interactive, 1 print)". AF1: `I + P == N` is
/// checked by parsing the rendered caption, not just substring matching.
#[ cfg( unix ) ]
#[ test ]
fn it_30_mode_all_breakdown_mixed_interactive_and_print()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_interactive = spawn_fake_claude( &path_val );
  let mut bg_print       = spawn_print_claude( &path_val );
  let proc = make_proc_dir( &[ bg_interactive.id(), bg_print.id() ] );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-30: exit 0 expected, got {:?}", out.status.code() );
  let ( n, i, p ) = parse_breakdown_counts( &stdout );
  assert_eq!( ( n, i, p ), ( 2, 1, 1 ), "IT-30: expected 2 running (1 interactive, 1 print). Got:\n{stdout}" );
  assert_eq!( i + p, n, "IT-30 (AF1): I + P must equal N. Got:\n{stdout}" );
}

// ── IT-31 (T02): `--mode all`, all interactive → breakdown ─────────────────────

/// IT-31 (T02): 3 active sessions, all interactive, `--mode all` (default) →
/// caption reads "3 running (3 interactive, 0 print)".
#[ cfg( unix ) ]
#[ test ]
fn it_31_mode_all_breakdown_all_interactive()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg : Vec< std::process::Child > = ( 0..3 ).map( |_| spawn_fake_claude( &path_val ) ).collect();
  let pids : Vec< u32 > = bg.iter().map( std::process::Child::id ).collect();
  let proc = make_proc_dir( &pids );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  for child in &mut bg { let _ = child.kill(); let _ = child.wait(); }

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-31: exit 0 expected, got {:?}", out.status.code() );
  let ( n, i, p ) = parse_breakdown_counts( &stdout );
  assert_eq!( ( n, i, p ), ( 3, 3, 0 ), "IT-31: expected 3 running (3 interactive, 0 print). Got:\n{stdout}" );
  assert_eq!( i + p, n, "IT-31 (AF1): I + P must equal N. Got:\n{stdout}" );
}

// ── IT-32 (T03): `--mode all`, all print → breakdown ────────────────────────────

/// IT-32 (T03): 3 active sessions, all print, `--mode all` (default) → caption
/// reads "3 running (0 interactive, 3 print)".
#[ cfg( unix ) ]
#[ test ]
fn it_32_mode_all_breakdown_all_print()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg : Vec< std::process::Child > = ( 0..3 ).map( |_| spawn_print_claude( &path_val ) ).collect();
  let pids : Vec< u32 > = bg.iter().map( std::process::Child::id ).collect();
  let proc = make_proc_dir( &pids );

  let out = run_clr_ps_proc( &path_val, proc.path().to_str().expect( "proc dir UTF-8" ) );

  for child in &mut bg { let _ = child.kill(); let _ = child.wait(); }

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-32: exit 0 expected, got {:?}", out.status.code() );
  let ( n, i, p ) = parse_breakdown_counts( &stdout );
  assert_eq!( ( n, i, p ), ( 3, 0, 3 ), "IT-32: expected 3 running (0 interactive, 3 print). Got:\n{stdout}" );
  assert_eq!( i + p, n, "IT-32 (AF1): I + P must equal N. Got:\n{stdout}" );
}

// ── IT-33 (T04): `--mode interactive`, mixed set → plain caption ───────────────

/// IT-33 (T04): 2 active sessions (1 interactive + 1 print), `--mode interactive`
/// → only the interactive row is shown; caption reads plain "1 running" with no
/// breakdown parentheses.
#[ cfg( unix ) ]
#[ test ]
fn it_33_mode_interactive_caption_has_no_breakdown()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_interactive = spawn_fake_claude( &path_val );
  let mut bg_print       = spawn_print_claude( &path_val );
  let proc = make_proc_dir( &[ bg_interactive.id(), bg_print.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "interactive" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --mode interactive" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-33: exit 0 expected, got {:?}", out.status.code() );
  assert_eq!( parse_running_count( &stdout ), 1, "IT-33: expected exactly 1 running. Got:\n{stdout}" );
  let caption = caption_line( &stdout );
  assert!( !caption.contains( '(' ), "IT-33: caption must NOT contain a breakdown. Got: {caption}" );
}

// ── IT-34 (T05): `--mode print`, mixed set → plain caption ──────────────────────

/// IT-34 (T05): 2 active sessions (1 interactive + 1 print), `--mode print` →
/// only the print row is shown; caption reads plain "1 running" with no
/// breakdown parentheses.
#[ cfg( unix ) ]
#[ test ]
fn it_34_mode_print_caption_has_no_breakdown()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg_interactive = spawn_fake_claude( &path_val );
  let mut bg_print       = spawn_print_claude( &path_val );
  let proc = make_proc_dir( &[ bg_interactive.id(), bg_print.id() ] );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "ps", "--mode", "print" ] )
    .env( "PATH", &path_val )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .output()
    .expect( "run clr ps --mode print" );

  let _ = bg_interactive.kill();
  let _ = bg_interactive.wait();
  let _ = bg_print.kill();
  let _ = bg_print.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "IT-34: exit 0 expected, got {:?}", out.status.code() );
  assert_eq!( parse_running_count( &stdout ), 1, "IT-34: expected exactly 1 running. Got:\n{stdout}" );
  let caption = caption_line( &stdout );
  assert!( !caption.contains( '(' ), "IT-34: caption must NOT contain a breakdown. Got: {caption}" );
}

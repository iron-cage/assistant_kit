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
//! | IT-12 | Active table caption contains `Active Sessions` and `running` | Caption presence |
//! | IT-13 | Orphaned gate file (dead PID) filtered out of queued table    | BUG-293 repro    |
//! | IT-16 | Task column extracts Form A content for underscore CWD         | BUG-295/296/297  |
//! | IT-17 | Task column selects Form A over Form B `tool_result` lines      | BUG-297 repro    |
//! | IT-19 | Task column works for CWD with no underscores (regression)     | BUG-295 regression|

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ run_cli, run_cli_with_env, stderr_str, stdout_str };

#[ cfg( unix ) ]
use cli_binary_test_helpers::{ fake_claude_binary_dir, run_clr_ps, spawn_fake_claude };

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
/// output uses plain style — no unicode box-drawing border character (`┌`).
#[ cfg( unix ) ]
#[ test ]
fn it_02_sessions_present_plain_style()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let out = run_clr_ps( &path_val );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit code must be 0, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "PID" ),
    "stdout must contain PID header, got: {stdout}"
  );
  assert!(
    !stdout.contains( '\u{250C}' ), // must NOT have ┌
    "stdout must use plain style — no ┌ border, got: {stdout}"
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

  let out = run_clr_ps( &path_val );

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

  let out = run_clr_ps( &path_val );

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

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .arg( "ps" )
    .env( "PATH", &path_val )
    .env( "PRO", pro_str )
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

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ) ] );
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
/// contains the titled caption rule line ("Active Sessions · N running") above
/// the column headers.
///
/// The caption is rendered by `TableCaption::new("Active Sessions").field(format!("{} running", n))`
/// via `data_fmt`; this test confirms end-to-end that the caption text appears in
/// the output and is not accidentally dropped by the formatter.
#[ cfg( unix ) ]
#[ test ]
fn it_12_active_table_has_caption()
{
  let ( _dir, path_val ) = fake_claude_binary_dir();
  let mut bg = spawn_fake_claude( &path_val );

  let out = run_clr_ps( &path_val );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "Active Sessions" ),
    "IT-12: active table caption must contain 'Active Sessions', got: {stdout}"
  );
  assert!(
    stdout.contains( "running" ),
    "IT-12: active table caption must contain 'running' count suffix, got: {stdout}"
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

  // PID 99999999 is guaranteed not to exist (/proc/sys/kernel/pid_max is at most 4194304).
  let orphan_file = gate_dir.path().join( "99999999.json" );
  std::fs::write(
    &orphan_file,
    r#"{"cwd":"/tmp/dead-process","since":1,"attempt":1,"message":"waiting for session slot"}"#,
  ).expect( "write orphan gate file" );

  let out    = run_cli_with_env( &[ "ps" ], &[ ( "CLR_GATE_DIR", gate_dir_path ) ] );
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
  let cwd_str  = cwd.to_str().expect( "CWD UTF-8" );

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

  // Build synthetic JSONL at the correctly-encoded project path.
  let encoded      = cwd_str.replace( ['/', '_'], "-" );
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
  let cwd_str  = cwd.to_str().expect( "CWD UTF-8" );

  let mut bg = std::process::Command::new( "claude" )
    .arg( "30" )
    .env( "PATH", &path_val )
    .current_dir( &cwd )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  let encoded      = cwd_str.replace( ['/', '_'], "-" );
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

// ── IT-19: Task column — no-underscore CWD regression ────────────────────────

/// IT-19: `clr ps` Task column works for a session whose CWD contains no
/// underscores — regression guard for the BUG-295 fix.
///
/// ## Root Cause
/// The BUG-295 fix adds `replace('_', "-")`. This test verifies the fix does not
/// break paths that encoded correctly with slash-only replacement.
///
/// ## Why Not Caught
/// IT-16 only covered underscore-containing paths; the no-underscore path was
/// never exercised end-to-end after BUG-295 fix.
///
/// ## Fix Applied
/// No fix needed — regression test only. Verifies the two-step replacement chain
/// is idempotent for paths without underscores.
///
/// ## Prevention
/// Always include a no-underscore regression test alongside any path-encoding fix.
///
/// ## Pitfall
/// The encoded path for a no-underscore CWD is identical whether you apply the
/// `replace('_', "-")` step or not — so this test confirms there is no regression.
#[ cfg( unix ) ]
#[ test ]
fn it_19_task_column_no_underscores()
{
  let ( _bin_dir, path_val ) = fake_claude_binary_dir();

  // CWD with no underscores — regression guard.
  let proj_tmp = tempfile::TempDir::new().expect( "create project tmp" );
  let cwd      = proj_tmp.path().join( "work" ).join( "proj" );
  std::fs::create_dir_all( &cwd ).expect( "create CWD without underscores" );
  let cwd_str  = cwd.to_str().expect( "CWD UTF-8" );

  let mut bg = std::process::Command::new( "claude" )
    .arg( "30" )
    .env( "PATH", &path_val )
    .current_dir( &cwd )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::null() )
    .spawn()
    .expect( "spawn fake claude" );
  std::thread::sleep( core::time::Duration::from_millis( 200 ) );

  // Encode with only '/' → '-' (no underscores to replace; result is same as before fix).
  let encoded      = cwd_str.replace( '/', "-" );
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
    .output()
    .expect( "run clr ps" );

  let _ = bg.kill();
  let _ = bg.wait();

  let stdout = stdout_str( &out );
  assert!( out.status.success(), "exit 0 expected, got {:?}", out.status.code() );
  assert!(
    stdout.contains( "no underscores task" ),
    "IT-19 (BUG-295 regression): Task column must show Form A content for underscore-free CWD. Got:\n{stdout}"
  );
}

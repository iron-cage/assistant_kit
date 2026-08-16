//! Bug reproducers for BUG-490 (session transplant is physical, not env-var steering),
//! BUG-491 (nonexistent working dir fails fast with the real cause), and BUG-492
//! (`--no-stdin` opt-out unblocks a held-open non-TTY stdin).
#![ cfg( unix ) ]
//!
//! All three bugs were found in the 2026-08-16 session-clone manual test campaign.
//! BUG-490/491 tests use a PATH-stubbed `claude` plus `CLAUDE_HOME`/gate isolation;
//! BUG-492 tests spawn the real binary with a deliberately held-open stdin pipe.

use std::io::Write as _;
use std::os::unix::fs::PermissionsExt;

// ── Shared helpers ─────────────────────────────────────────────────────────────

/// Container guard (mirrors the private `assert_container` in `cli_binary_test_helpers`).
fn container_check()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Host bypass: VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

/// Encode a path using the production `Df()` encoder (BUG-391 precedent: never hand-roll).
fn df( path : &std::path::Path ) -> String
{
  claude_storage_core::encode_path( path )
    .expect( "df(): path must encode successfully in test fixtures" )
}

/// Create `<claude_home>/projects/<df(src_dir)>/<uuid>.jsonl` with the given content.
///
/// Returns the `.jsonl` path.  The caller must keep the `TempDir` alive.
fn make_session( claude_home : &std::path::Path, project_dir : &std::path::Path, uuid : &str, content : &[ u8 ] )
  -> std::path::PathBuf
{
  let storage = claude_home.join( "projects" ).join( df( project_dir ) );
  std::fs::create_dir_all( &storage ).expect( "create session storage dir" );
  let file = storage.join( format!( "{uuid}.jsonl" ) );
  std::fs::write( &file, content ).expect( "write session jsonl" );
  file
}

/// Write an executable stub `claude` into `bin_dir` that logs each invocation to
/// `invoke_log`, records whether `probe_path` exists at spawn time into `probe_out`,
/// and emits a minimal success result envelope carrying `session_id`.
fn write_claude_stub
(
  bin_dir    : &std::path::Path,
  invoke_log : &std::path::Path,
  probe_path : &std::path::Path,
  probe_out  : &std::path::Path,
  session_id : &str,
)
{
  std::fs::create_dir_all( bin_dir ).expect( "create stub bin dir" );
  let stub = bin_dir.join( "claude" );
  let script = format!(
    "#!/bin/sh\n\
     echo \"invoked\" >> {log}\n\
     if [ -f {probe} ]; then echo present > {out}; else echo absent > {out}; fi\n\
     cat > /dev/null 2>&1 || true\n\
     printf '%s' '{{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"result\":\"stub-ok\",\"session_id\":\"{sid}\"}}'\n",
    log = invoke_log.display(),
    probe = probe_path.display(),
    out = probe_out.display(),
    sid = session_id,
  );
  std::fs::write( &stub, script ).expect( "write claude stub" );
  std::fs::set_permissions( &stub, std::fs::Permissions::from_mode( 0o755 ) )
    .expect( "chmod claude stub" );
}

/// PATH value putting `bin_dir` first, keeping the inherited PATH for `sh`/coreutils.
fn stub_path( bin_dir : &std::path::Path ) -> String
{
  let inherited = std::env::var( "PATH" ).unwrap_or_default();
  format!( "{}:{inherited}", bin_dir.display() )
}

// ── BUG-490: physical session transplant ──────────────────────────────────────

/// The transplant destination storage dir for a target project dir, matching the
/// production computation (`scope_for(canonicalized target).claude_session_dir`).
fn target_storage( claude_home : &std::path::Path, target : &std::path::Path ) -> std::path::PathBuf
{
  let canon = std::fs::canonicalize( target ).expect( "canonicalize target" );
  claude_home.join( "projects" ).join( df( &canon ) )
}

/// BUG-490: `--dry-run` previews the transplant plan and no longer exports the
/// inert `CLAUDE_CODE_SESSION_DIR` variable for `--session-from`.
///
/// ## Root Cause
/// The entire `--session-from` mechanism was exporting `CLAUDE_CODE_SESSION_DIR`
/// to the subprocess; claude ≥2.x gives that variable no observable effect for
/// reads or writes (contract B23 was NEG-ONLY from introduction), so `-c` resolved
/// "most recent conversation" from the target's cwd-derived storage — a silent
/// fresh session with exit 0 on every clone.
///
/// ## Why Not Caught
/// All 11 prior EC tests and the dry-run preview verify only the *constructed*
/// command (env/argv), never claude's semantic behavior — the fidelity boundary
/// `docs/invariant/011` had explicitly pre-declared as unresolved.
///
/// ## Fix Applied
/// The builder plans a physical copy of the source's most-recent qualifying
/// session file into the *target's own* storage dir (same filename → same session
/// id, fresh mtime → `-c` selects it); the dispatch path executes the copy after
/// the dry-run exit and before spawn.  The env export is dropped for
/// `--session-from` (it remains for raw `--session-dir` — BUG-493's own report).
///
/// ## Prevention
/// A NEG-ONLY contract must never be a feature's sole load-bearing mechanism —
/// build on a mechanism the runner itself controls.
///
/// ## Pitfall
/// Claude appends continued turns to the copied file in place under the same
/// uuid — the expected-id mismatch machinery (BUG-320) must stay keyed on the
/// source's session id, and the copy must never execute when source and target
/// storage coincide (`fs::copy` onto the same path truncates the file mid-read).
#[ test ]
#[ doc = "bug_reproducer(BUG-490)" ]
fn t490_dry_run_plans_transplant_and_drops_env_export()
{
  container_check();
  let ch  = tempfile::TempDir::new().expect( "claude home" );
  let src = tempfile::TempDir::new().expect( "source project" );
  let tgt = tempfile::TempDir::new().expect( "target project" );
  let src_canon = std::fs::canonicalize( src.path() ).expect( "canonicalize source" );
  let jsonl = make_session( ch.path(), &src_canon, "aaa49001-1111-2222-3333-444444444444", b"{}" );

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args
    ([
      "--dry-run",
      "--to", tgt.path().to_str().expect( "utf-8" ),
      "--session-from", src.path().to_str().expect( "utf-8" ),
      "transplant preview",
    ])
    .env( "CLAUDE_HOME", ch.path() )
    .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_SESSION_FROM" )
    .output()
    .expect( "invoke clr" );
  assert!( out.status.success(), "dry-run must succeed. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout ).into_owned();

  let dest_dir = target_storage( ch.path(), tgt.path() );
  let plan_line = format!( "# session-transplant: {} -> {}", jsonl.display(), dest_dir.display() );
  assert!(
    stdout.contains( &plan_line ),
    "dry-run must preview the transplant plan line `{plan_line}`. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( " -c \"" ),
    "continue flag `-c` must still be injected. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "CLAUDE_CODE_SESSION_DIR=" ),
    "the inert CLAUDE_CODE_SESSION_DIR export must be dropped for --session-from. Got:\n{stdout}"
  );
}

/// BUG-490: a real (non-dry) run copies the source session byte-identically into
/// the target's storage BEFORE the subprocess spawns, and never touches the source.
#[ test ]
#[ doc = "bug_reproducer(BUG-490)" ]
fn t490_real_run_copies_source_session_into_target_storage_before_spawn()
{
  container_check();
  let ch   = tempfile::TempDir::new().expect( "claude home" );
  let src  = tempfile::TempDir::new().expect( "source project" );
  let tgt  = tempfile::TempDir::new().expect( "target project" );
  let work = tempfile::TempDir::new().expect( "work dir" );
  let src_canon = std::fs::canonicalize( src.path() ).expect( "canonicalize source" );
  let uuid = "bbb49002-1111-2222-3333-444444444444";
  let content = b"{\"seed\":\"source history bytes\"}\n";
  let src_jsonl = make_session( ch.path(), &src_canon, uuid, content );

  let dest_dir  = target_storage( ch.path(), tgt.path() );
  let dest_file = dest_dir.join( format!( "{uuid}.jsonl" ) );
  let invoke_log = work.path().join( "invocations.log" );
  let probe_out  = work.path().join( "probe.txt" );
  write_claude_stub( &work.path().join( "bin" ), &invoke_log, &dest_file, &probe_out, uuid );

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args
    ([
      "--to", tgt.path().to_str().expect( "utf-8" ),
      "--session-from", src.path().to_str().expect( "utf-8" ),
      "--max-sessions", "0",
      "--journal", "off",
      "what is in the source session?",
    ])
    .env( "CLAUDE_HOME", ch.path() )
    .env( "PATH", stub_path( &work.path().join( "bin" ) ) )
    .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_SESSION_FROM" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "real run must succeed. stdout: {}\nstderr: {}",
    String::from_utf8_lossy( &out.stdout ),
    String::from_utf8_lossy( &out.stderr ),
  );

  // Stub ran exactly once, and the transplant was already on disk at spawn time.
  let invocations = std::fs::read_to_string( &invoke_log ).unwrap_or_default();
  assert_eq!( invocations.lines().count(), 1, "stub claude must be spawned exactly once" );
  let probe = std::fs::read_to_string( &probe_out ).unwrap_or_default();
  assert_eq!(
    probe.trim(), "present",
    "transplanted session file must exist in target storage BEFORE the subprocess spawns"
  );

  // Byte-identical copy in the target's own storage; source untouched.
  let copied = std::fs::read( &dest_file ).expect( "transplanted file must exist" );
  assert_eq!( copied, content, "transplanted file must be byte-identical to the source session" );
  let src_after = std::fs::read( &src_jsonl ).expect( "source must still exist" );
  assert_eq!( src_after, content, "source session must never be modified by a clone run" );
}

/// BUG-490: an existing destination file (a prior clone's possibly-diverged
/// lineage) is never overwritten — its mtime is refreshed instead so claude's
/// own most-recent selection picks it up for `-c`.
#[ test ]
#[ doc = "bug_reproducer(BUG-490)" ]
fn t490_existing_dest_never_overwritten_mtime_refreshed()
{
  container_check();
  let ch   = tempfile::TempDir::new().expect( "claude home" );
  let src  = tempfile::TempDir::new().expect( "source project" );
  let tgt  = tempfile::TempDir::new().expect( "target project" );
  let work = tempfile::TempDir::new().expect( "work dir" );
  let src_canon = std::fs::canonicalize( src.path() ).expect( "canonicalize source" );
  let tgt_canon = std::fs::canonicalize( tgt.path() ).expect( "canonicalize target" );
  let uuid = "ccc49003-1111-2222-3333-444444444444";
  make_session( ch.path(), &src_canon, uuid, b"{\"seed\":\"original\"}\n" );
  // Pre-place a diverged prior clone under the SAME uuid in the target's storage.
  let diverged = b"{\"seed\":\"original\"}\n{\"turn\":\"target-local divergence\"}\n";
  let dest_file = make_session( ch.path(), &tgt_canon, uuid, diverged );
  let mtime_before = std::fs::metadata( &dest_file ).expect( "stat dest" ).modified().expect( "mtime" );
  std::thread::sleep( core::time::Duration::from_millis( 50 ) );

  let invoke_log = work.path().join( "invocations.log" );
  let probe_out  = work.path().join( "probe.txt" );
  write_claude_stub( &work.path().join( "bin" ), &invoke_log, &dest_file, &probe_out, uuid );

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args
    ([
      "--to", tgt.path().to_str().expect( "utf-8" ),
      "--session-from", src.path().to_str().expect( "utf-8" ),
      "--max-sessions", "0",
      "--journal", "off",
      "continue the clone lineage",
    ])
    .env( "CLAUDE_HOME", ch.path() )
    .env( "PATH", stub_path( &work.path().join( "bin" ) ) )
    .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_SESSION_FROM" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "run must succeed. stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );

  let after = std::fs::read( &dest_file ).expect( "dest must still exist" );
  assert_eq!(
    after, diverged,
    "existing destination (diverged prior clone) must never be overwritten"
  );
  let mtime_after = std::fs::metadata( &dest_file ).expect( "stat dest" ).modified().expect( "mtime" );
  assert!(
    mtime_after > mtime_before,
    "destination mtime must be refreshed so `-c` selects the clone lineage"
  );
}

// ── BUG-491: nonexistent working dir named, no retry ladder ───────────────────

/// BUG-491: a nonexistent `--dir`/`--to` fails immediately with the real cause.
///
/// ## Root Cause
/// `std::process::Command` reports a missing working directory as the same
/// `io::ErrorKind::NotFound` as a missing program; clr's spawn-error classifier
/// mapped every `NotFound` to the missing-binary class, printing "claude binary
/// not found in PATH" with its npm install hint and burning the full Runner
/// retry ladder (3×30s) while the actual defect — a typo'd or not-yet-created
/// target directory — was never mentioned.
///
/// ## Why Not Caught
/// All spawn-path tests used existing working directories, and the
/// error-classification tests feed synthetic errors already labeled as
/// binary-missing; no test combined a present binary with a missing cwd.
///
/// ## Fix Applied
/// `dispatch_run` validates the effective working directory after the dry-run
/// exit and before any gate wait or spawn: a non-directory path fails
/// immediately and loudly, naming the path and the flags, with no retry.
///
/// ## Prevention
/// When one `ErrorKind` covers multiple causes, disambiguate with a direct
/// precondition check before attributing — never let an install hint ride on an
/// unverified attribution.
///
/// ## Pitfall
/// `--dry-run` must stay exempt: it is a pure preview of a command that may
/// target a directory the caller intends to create later.
#[ test ]
#[ doc = "bug_reproducer(BUG-491)" ]
fn t491_nonexistent_working_dir_fails_fast_named_no_retry_ladder()
{
  container_check();
  let work = tempfile::TempDir::new().expect( "work dir" );
  let missing = work.path().join( "never_made" );
  let invoke_log = work.path().join( "invocations.log" );
  let probe_out  = work.path().join( "probe.txt" );
  write_claude_stub( &work.path().join( "bin" ), &invoke_log, &missing, &probe_out, "dddd4910-1111-2222-3333-444444444444" );

  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args
    ([
      "--dir", missing.to_str().expect( "utf-8" ),
      "--retry-on-runner", "0",
      "--runner-delay", "0",
      "--max-sessions", "0",
      "--journal", "off",
      "hello",
    ])
    .env( "PATH", stub_path( &work.path().join( "bin" ) ) )
    .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_SESSION_FROM" )
    .output()
    .expect( "invoke clr" );
  let stderr = String::from_utf8_lossy( &out.stderr ).into_owned();

  assert!( !out.status.success(), "nonexistent working dir must fail. stderr: {stderr}" );
  assert!(
    stderr.contains( "working directory does not exist" ),
    "error must name the real cause (missing working directory). Got:\n{stderr}"
  );
  assert!(
    stderr.contains( missing.to_str().expect( "utf-8" ) ),
    "error must name the offending path. Got:\n{stderr}"
  );
  assert!(
    !stderr.contains( "claude binary not found" ),
    "the missing-binary misdiagnosis must not appear. Got:\n{stderr}"
  );
  assert!(
    !std::path::Path::new( &invoke_log ).exists(),
    "no subprocess spawn may be attempted for a nonexistent working dir"
  );
}

/// BUG-491: `--dry-run` with a nonexistent `--dir` stays a pure preview — no
/// validation error, exit 0 (the caller may create the directory later).
#[ test ]
#[ doc = "bug_reproducer(BUG-491)" ]
fn t491_dry_run_exempt_from_working_dir_validation()
{
  container_check();
  let work = tempfile::TempDir::new().expect( "work dir" );
  let missing = work.path().join( "never_made" );
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "--dry-run", "--dir", missing.to_str().expect( "utf-8" ), "hello" ] )
    .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_SESSION_FROM" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "--dry-run must remain a pure preview for a not-yet-created dir. stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

// ── BUG-492: --no-stdin opt-out for held-open non-TTY stdin ───────────────────

/// Spawn `clr` with a piped stdin that is deliberately KEPT OPEN (nothing written,
/// no EOF), poll for exit up to `ceiling`, and return the output — killing the
/// child and panicking if it never exits (the pre-fix indefinite-block posture).
fn run_with_held_open_stdin( args : &[ &str ], env : &[ ( &str, &str ) ], ceiling : core::time::Duration )
  -> std::process::Output
{
  container_check();
  let mut child = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( args )
    // Isolation removals FIRST, caller-supplied pairs second — Command env ops apply
    // in call order, so a later env_remove would clobber a caller's own injection.
    .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_SESSION_FROM" )
    .env_remove( "CLR_NO_STDIN" )
    .envs( env.iter().copied() )
    .stdin( std::process::Stdio::piped() )
    .stdout( std::process::Stdio::piped() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );
  // Hold the write end open for the child's whole lifetime — the BUG-492 posture.
  let held_stdin = child.stdin.take();
  let start = std::time::Instant::now();
  loop
  {
    match child.try_wait().expect( "try_wait" )
    {
      Some( _ ) => break,
      None if start.elapsed() > ceiling =>
      {
        let _ = child.kill();
        let _ = child.wait();
        drop( held_stdin );
        panic!( "clr blocked on held-open stdin beyond {}s — opt-out ineffective", ceiling.as_secs() );
      }
      None => std::thread::sleep( core::time::Duration::from_millis( 50 ) ),
    }
  }
  drop( held_stdin );
  child.wait_with_output().expect( "collect output" )
}

/// BUG-492: `--no-stdin` completes a dry-run instantly while the pipe stays open.
///
/// ## Root Cause
/// `dispatch_run()`'s first action, `detect_stdin_json()`, performs an
/// unconditional blocking `read_to_end` on any non-TTY stdin (to classify piped
/// JSON config vs raw forwardable content).  A parent that holds the write end
/// open without writing — a common supervisor/CI posture — blocked clr forever,
/// before argument parsing, before any output, even under `--dry-run`.
///
/// ## Why Not Caught
/// Every stdin test piped finite content, so EOF always arrived; no test modeled
/// the held-open-pipe posture, and TTY (interactive) stdin skips detection.
///
/// ## Fix Applied
/// New opt-out `--no-stdin` (+ env `CLR_NO_STDIN=1`), checked as Gate 0 in
/// `detect_stdin_json()`'s raw token scan before any read: when present, stdin
/// is left untouched — no JSON config from stdin, no raw forwarding.
///
/// ## Prevention
/// Any auto-detection that performs a blocking read on an inherited channel must
/// ship an explicit opt-out plus a documented hazard note — only the caller
/// knows whether the channel will ever close.
///
/// ## Pitfall
/// The gate must run in the pre-parse raw token scan (like the `--file` gate) —
/// by the time `parse_args` sees the flag, stdin has already been read.
#[ test ]
#[ doc = "bug_reproducer(BUG-492)" ]
fn t492_no_stdin_flag_unblocks_held_open_pipe()
{
  let out = run_with_held_open_stdin(
    &[ "--no-stdin", "--dry-run", "hi" ],
    &[],
    core::time::Duration::from_secs( 8 ),
  );
  assert!(
    out.status.success(),
    "--no-stdin dry-run must exit 0 while the pipe is held open. stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "claude" ),
    "dry-run preview must be printed. Got:\n{stdout}"
  );
}

/// BUG-492: `CLR_NO_STDIN=1` is the env-var route to the same opt-out.
#[ test ]
#[ doc = "bug_reproducer(BUG-492)" ]
fn t492_env_clr_no_stdin_equivalent()
{
  let out = run_with_held_open_stdin(
    &[ "--dry-run", "hi" ],
    &[ ( "CLR_NO_STDIN", "1" ) ],
    core::time::Duration::from_secs( 8 ),
  );
  assert!(
    out.status.success(),
    "CLR_NO_STDIN=1 dry-run must exit 0 while the pipe is held open. stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
}

/// BUG-492: the opt-out declines stdin entirely — piped JSON config that would
/// normally apply (e.g. a `model` key) is ignored under `--no-stdin`.
#[ test ]
#[ doc = "bug_reproducer(BUG-492)" ]
// clippy::std_instead_of_core's suggested `core::io::ErrorKind` does not exist on this
// toolchain (`io` is std-only) — its machine-applicable fix does not compile.
#[ allow( clippy::std_instead_of_core ) ]
fn t492_no_stdin_declines_piped_json_config()
{
  container_check();
  let mut child = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "--no-stdin", "--dry-run", "hi" ] )
    .env_remove( "CLR_DIR" ).env_remove( "CLR_SESSION_DIR" ).env_remove( "CLR_SESSION_FROM" )
    .env_remove( "CLR_NO_STDIN" ).env_remove( "CLR_MODEL" )
    .stdin( std::process::Stdio::piped() )
    .stdout( std::process::Stdio::piped() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );
  // BrokenPipe here is acceptable — and is itself evidence of the fix: clr,
  // honoring --no-stdin, may exit without ever reading its stdin end, so the
  // write races clr's exit under parallel-suite load. Either way the marker
  // cannot have been consumed as config. Any OTHER write error still panics.
  if let Err( e ) = child.stdin.take().expect( "stdin handle" )
    .write_all( b"{\"model\":\"bug492-model-must-not-apply\"}" )
  {
    assert_eq!(
      e.kind(), std::io::ErrorKind::BrokenPipe,
      "unexpected stdin write error: {e}"
    );
  }
  // Handle dropped here — EOF delivered; without --no-stdin this JSON would apply.
  let out = child.wait_with_output().expect( "collect output" );
  assert!( out.status.success(), "dry-run must succeed" );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "bug492-model-must-not-apply" ),
    "--no-stdin must decline piped JSON config entirely. Got:\n{stdout}"
  );
}

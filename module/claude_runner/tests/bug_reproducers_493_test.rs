//! Bug reproducer for BUG-493 (`--session-dir` raw override is inert on claude >= 2.x —
//! parameter deprecated: no env export, no `-c` gating role, loud stderr warning).
//!
//! Uses `--dry-run` only: the bug's whole surface is the constructed command and the
//! deprecation warning, so no `claude` stub or container PATH setup is required.

// ── BUG-493: deprecated --session-dir is fully inert ──────────────────────────

/// A raw override dir CONTAINING a session must neither export
/// `CLAUDE_CODE_SESSION_DIR` nor inject `-c` when the real source storage is empty.
///
/// ## Root Cause
/// `--session-dir <raw dir>` steered sessions through a `CLAUDE_CODE_SESSION_DIR`
/// export that claude >= 2.x ignores for both reads and writes (proven by BUG-490's
/// control experiment), and gated `-c` injection on a scan of that raw dir — a
/// directory claude never reads. `-c` could therefore be injected when claude's
/// real cwd storage had no conversation (claude errors or silently starts fresh)
/// or suppressed when it had one.
///
/// ## Why Not Caught
/// Same fidelity boundary as BUG-490: every prior `--session-dir` test asserted
/// only the *constructed* command (env prefix and argv), never claude's semantic
/// response to the env var — contract B23 was NEG-ONLY from its introduction.
///
/// ## Fix Applied
/// Deprecation, not repurposing: the parameter stays parseable from all three
/// sources (CLI flag, `CLR_SESSION_DIR`, json `"session-dir"`) so existing
/// invocations don't hard-fail, but it is fully inert — no env export, no role in
/// `-c` gating or transplant planning — and one loud stderr warning points callers
/// at `--from <dir>`. Session gating always follows the `--from`/cwd computed
/// project storage.
///
/// ## Prevention
/// BUG-490's lesson applied to the parameter surface: when a feature's sole
/// load-bearing mechanism is a NEG-ONLY contract, retire the feature loudly
/// instead of leaving a silently-dead knob that keeps steering runner decisions.
///
/// ## Pitfall
/// The warning is unconditional — builder warnings are not gated by `--quiet`
/// (see `param_group` G2CC4). Tests asserting "empty stderr" while passing
/// `--session-dir` must account for exactly this one line.
// test_kind: bug_reproducer(BUG-493)
#[ test ]
fn t493_override_with_session_no_export_no_continue()
{
  // Raw override dir with a session file — the OLD code trusted this dir.
  let override_dir = tempfile::TempDir::new().expect( "override tmpdir" );
  std::fs::write( override_dir.path().join( "aaa49301-1111-2222-3333-444444444444.jsonl" ), b"{}" )
    .expect( "write override session" );
  let override_str = override_dir.path().to_str().expect( "utf-8" );
  // Real source storage empty — an empty CLAUDE_HOME means no project has a session.
  let empty_home = tempfile::TempDir::new().expect( "empty CLAUDE_HOME" );
  let home = empty_home.path().to_str().expect( "utf-8" );
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "--dry-run", "--session-dir", override_str, "test" ] )
    .env( "HOME", "/tmp/clr-isolated-home" ) // Fix(BUG-008) isolation: prevent host prefs from injecting --model
    .env( "CLAUDE_HOME", home )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .env_remove( "CLR_FROM" )
    .output()
    .expect( "invoke clr" );
  assert!(
    out.status.success(),
    "deprecated --session-dir must not fail the invocation. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "CLAUDE_CODE_SESSION_DIR=" ),
    "inert --session-dir must not export CLAUDE_CODE_SESSION_DIR. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( " -c" ),
    "override dir contents must not gate -c: real source storage is empty, so no -c. Got:\n{stdout}"
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "--session-dir" ) && stderr.contains( "is deprecated and has no effect" ),
    "deprecated --session-dir must warn loudly on stderr. Got:\n{stderr}"
  );
}

/// Guard: the deprecation warning fires ONLY when the parameter is given —
/// a clean invocation must not spam it on every run.
// test_kind: bug_reproducer(BUG-493)
#[ test ]
fn t493_no_warning_without_parameter()
{
  let empty_home = tempfile::TempDir::new().expect( "empty CLAUDE_HOME" );
  let home = empty_home.path().to_str().expect( "utf-8" );
  let out = std::process::Command::new( env!( "CARGO_BIN_EXE_clr" ) )
    .args( [ "--dry-run", "test" ] )
    .env( "HOME", "/tmp/clr-isolated-home" ) // Fix(BUG-008) isolation: prevent host prefs from injecting --model
    .env( "CLAUDE_HOME", home )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .env_remove( "CLR_FROM" )
    .output()
    .expect( "invoke clr" );
  assert!( out.status.success(), "clean dry-run must exit 0" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !stderr.contains( "--session-dir is deprecated" ),
    "warning must fire only when the parameter is actually given. Got:\n{stderr}"
  );
}

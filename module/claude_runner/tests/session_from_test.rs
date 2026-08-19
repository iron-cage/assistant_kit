//! Edge case tests for the `--from` parameter.
//!
//! Covers EC-1–EC-12 from `tests/docs/cli/param/076_from.md` and
//! US-1–US-8 from `tests/docs/cli/user_story/028_session_transplant.md`.
//!
//! `--from <DIR>` computes `scope_for(DIR).claude_session_dir`, plans a
//! physical copy of the most-recent qualifying `.jsonl` from that source storage
//! into the TARGET's own storage (Fix(BUG-490): claude ≥2.x ignores the former
//! `CLAUDE_CODE_SESSION_DIR` export for reads and writes, so the env route is
//! dead and dropped), and activates `-c` (continue) mode.  Dry-run previews the
//! plan as a `# session-transplant: <src_file> -> <target_storage>` line without
//! copying.  All tests use `--dry-run` so no real Claude binary is needed.
//!
//! `--from` defaults to the current working directory when omitted — same as
//! `--to`/`--dir` (see `resolve_effective_dir`) — so `--to <TARGET>` alone now
//! clones outward from cwd by default (see US-4), and a bare invocation (neither
//! flag) is a guaranteed self-copy no-op (see US-8).  `--session-from` (the
//! pre-rename flag name) is no longer recognized — a breaking rename, not an
//! alias (see EC-2); nor is the old `CLR_SESSION_FROM` env var (see EC-12).
//!
//! # Test Setup Pattern
//!
//! Since `--from` uses `scope_for(src)` to resolve the session storage
//! directory, tests set `CLAUDE_HOME` to a temp dir and place the `.jsonl` file
//! in `<claude_home>/projects/<df(src_dir)>/` — the exact path that `scope_for`
//! computes.  The `df()` helper implements the same `Df()` encoding algorithm.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli;

// ── Shared helpers ─────────────────────────────────────────────────────────────

/// Container guard (mirrors the private `assert_container` in helpers).
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

/// Encode a path using the `Df()` algorithm from `algorithm/001_path_encoding.md`.
///
/// Delegates to the real production encoder — see `Fix(BUG-391)` below.
// Fix(BUG-391): delegate to the real production encoder instead of reimplementing
// it — the prior hand-rolled body only substituted '_', which diverged from
// claude_storage_core::encode_path()'s full non-alphanumeric substitution the
// moment a fixture (e.g. tempfile::TempDir's ".tmp"-prefixed path) contained a '.'.
// Root cause: this was a duplicate encoder (also flagged by BUG-366, already fixed
// for scope_command_test.rs by BUG-386) reimplementing production logic instead of
// calling it, so it silently diverged when encode_path()'s substitution scope widened.
// Pitfall: never hand-roll a test-local copy of a production encoding/formatting
// function — call the real function so the fixture cannot drift from production.
fn df( path : &str ) -> String
{
  claude_storage_core::encode_path( std::path::Path::new( path ) )
    .expect( "df(): path must encode successfully in test fixtures" )
}

/// BUG-391 regression guard: `df()` must match production `encode_path()` for a
/// dot-containing path (e.g. `tempfile::TempDir::new()`'s literal `.tmp` prefix) —
/// the exact input class that exposed the two encoders' prior divergence.
///
/// ## Root Cause
/// `df()` hand-rolled a duplicate of `claude_storage_core::encode_path()`, only
/// substituting `_`→`-`. Once `encode_path()` was generalized (BUG-366) to map
/// every non-alphanumeric character to `-`, the two encoders diverged for any
/// dot-containing path — exactly what `tempfile::TempDir::new()` always produces.
///
/// ## Why Not Caught
/// No test asserted parity between `df()` and `encode_path()` in this file; every
/// existing fixture's input happened to avoid the divergent class until BUG-366
/// widened `encode_path()`'s substitution scope.
///
/// ## Fix Applied
/// `df()` now delegates to `claude_storage_core::encode_path()` directly instead
/// of reimplementing it, so it cannot drift from production behavior.
///
/// ## Prevention
/// This test locks in parity for the specific input class (dot-containing paths)
/// that caused the original divergence — a future reimplementation regressing
/// `df()` back to a hand-rolled encoder would fail here immediately.
///
/// ## Pitfall
/// A duplicate encoder can pass indefinitely against fixtures that avoid the
/// divergent input class, then silently fail the moment a fixture's shape changes
/// (here: `tempfile::TempDir`'s literal `.tmp` prefix) with no code change to explain it.
#[ test ]
fn df_matches_production_encode_path_for_dot_containing_path()
{
  let path = "/tmp/.tmpAbCdEfGh/proj";
  let test_encoded = df( path );
  let real_encoded = claude_storage_core::encode_path( std::path::Path::new( path ) )
    .expect( "encode_path" );
  assert_eq!(
    test_encoded, real_encoded,
    "test df() helper must match production encode_path() for dot-containing paths \
     (BUG-391 regression guard)"
  );
}

/// Create `<claude_home>/projects/<df(src_dir)>/<uuid>.jsonl` with non-empty content.
///
/// Returns the `.jsonl` path.  The caller must keep the `TempDir` alive.
fn make_session_for( claude_home : &std::path::Path, src_dir : &str, uuid : &str ) -> std::path::PathBuf
{
  let session_dir = claude_home.join( "projects" ).join( df( src_dir ) );
  std::fs::create_dir_all( &session_dir ).expect( "create session dir" );
  let file = session_dir.join( format!( "{uuid}.jsonl" ) );
  std::fs::write( &file, b"{}" ).expect( "write session jsonl" );
  file
}

/// Run `clr --dry-run <args>` with extra env vars; return stdout.
///
/// Removes `CLR_DIR`, `CLR_SESSION_DIR`, and `CLR_FROM` from the inherited
/// environment before injecting `env` — prevents ambient values from interfering
/// with `--from` / `--session-dir` behavior.
///
/// # Panics
///
/// Panics if the subprocess cannot be launched or exits non-zero.
fn run_dry_env( args : &[ &str ], env : &[ ( &str, &str ) ] ) -> String
{
  container_check();
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut full = vec![ "--dry-run" ];
  full.extend_from_slice( args );
  let out = std::process::Command::new( bin )
    .args( &full )
    .envs( env.iter().copied() )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .env_remove( "CLR_FROM" )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "dry-run failed (exit {})\nstdout: {}\nstderr: {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stdout ),
    String::from_utf8_lossy( &out.stderr ),
  );
  String::from_utf8_lossy( &out.stdout ).into_owned()
}

// ── EC-1: --from injects -c when source has session ────────────────────────────

/// EC-1: `--from` plans a transplant of the source session file and
/// activates continue mode (`-c`) when a qualifying session file exists.
#[ test ]
fn ec1_from_injects_continue()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec1-src";
  let jsonl = make_session_for( ch.path(), src, "aaa-111" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  assert!(
    stdout.contains( &format!( "# session-transplant: {} -> ", jsonl.display() ) ),
    "dry-run must plan a transplant of the source session file. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "CLAUDE_CODE_SESSION_DIR=" ),
    "the dead CLAUDE_CODE_SESSION_DIR export must be gone (BUG-490). Got:\n{stdout}"
  );
  // Continue mode: `-c` flag precedes the quoted message in the subprocess command.
  assert!(
    stdout.contains( " -c \"" ),
    "dry-run must use continue flag `-c`. Got:\n{stdout}"
  );
}

// ── EC-2: --session-from is no longer recognized ───────────────────────────────

/// EC-2: `--session-from` (the pre-rename flag name) is no longer recognized.
///
/// The rename to `--from` is breaking, not an alias (CLAUDE.md "No Backward
/// Compatibility Preservation") — `--session-from` must now fail parsing with
/// the standard unknown-option error, exit non-zero, and plan no transplant.
#[ test ]
fn ec2_session_from_no_longer_recognized()
{
  container_check();
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec2-src";
  make_session_for( ch.path(), src, "bbb-222" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--dry-run", "--session-from", src, "Continue" ] )
    .env( "CLAUDE_HOME", ch_str )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .env_remove( "CLR_FROM" )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    !out.status.success(),
    "`--session-from` must be rejected (exit non-zero) now that it is renamed to `--from`. \
     stdout: {}\nstderr: {}",
    String::from_utf8_lossy( &out.stdout ),
    String::from_utf8_lossy( &out.stderr ),
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "unknown option: --session-from" ),
    "must report `--session-from` as an unknown option. Got:\n{stderr}"
  );
}

// ── EC-3: No .jsonl → no -c injected ──────────────────────────────────────────

/// EC-3: Source dir with no `.jsonl` → no `-c` injected; fresh session starts.
#[ test ]
fn ec3_empty_source_no_continue()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  // No .jsonl created — empty session storage
  let src = "/tmp/076ec3-empty-src";
  let stdout = run_dry_env(
    &[ "--from", src, "Start fresh" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!(
    !stdout.contains( " -c \"" ),
    "no session → dry-run must NOT have continue flag `-c`. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "# session-transplant:" ),
    "no session → no transplant plan. Got:\n{stdout}"
  );
}

// ── EC-4: --session-dir is inert alongside --from ───────────────────────────────

/// EC-4: `--session-dir` is deprecated and inert — `--from` governs when both are given.
///
/// Fix(BUG-493): the former precedence rule (`--session-dir` raw path wins) is gone.
/// The deprecated parameter must neither export `CLAUDE_CODE_SESSION_DIR` nor
/// suppress the `--from` transplant plan; the source storage governs `-c` gating.
#[ test ]
fn ec4_session_dir_inert_alongside_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec4-src";
  // Source session — governs, despite --session-dir also being present
  let jsonl = make_session_for( ch.path(), src, "ccc-333" );
  // Deprecated override dir (must be ignored entirely)
  let override_dir = tempfile::TempDir::new().expect( "override tmpdir" );
  std::fs::write( override_dir.path().join( "xyz-789.jsonl" ), b"{}" )
    .expect( "write override session" );
  let override_str = override_dir.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--from", src, "--session-dir", override_str, "test" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  // The dead env export must never appear (claude >= 2.x ignores it)
  assert!(
    !stdout.contains( "CLAUDE_CODE_SESSION_DIR=" ),
    "deprecated `--session-dir` must not export CLAUDE_CODE_SESSION_DIR. Got:\n{stdout}"
  );
  // --from still governs: transplant planned from the SOURCE storage
  assert!(
    stdout.contains( &format!( "# session-transplant: {} -> ", jsonl.display() ) ),
    "`--from` must govern the transplant plan despite `--session-dir`. Got:\n{stdout}"
  );
}

// ── EC-5: --new-session suppresses --from ───────────────────────────────────────

/// EC-5: `--new-session` takes precedence over `--from`.
///
/// `--new-session` suppresses cross-loading entirely: no `-c`, no transplant
/// plan, and no session-related env export of any kind.
#[ test ]
fn ec5_new_session_suppresses_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec5-src";
  make_session_for( ch.path(), src, "ddd-444" );
  let stdout = run_dry_env(
    &[ "--from", src, "--new-session", "fresh" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!(
    !stdout.contains( " -c \"" ),
    "`--new-session` must suppress continue flag `-c`. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "# session-transplant:" ),
    "`--new-session` must suppress the transplant plan. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "CLAUDE_CODE_SESSION_DIR=" ),
    "no session env export may appear (BUG-490 dropped it). Got:\n{stdout}"
  );
}

// ── EC-6: --to + --from ──────────────────────────────────────────────────────────

/// EC-6: `--to <tgt>` + `--from <src>`: Claude runs in target; the source
/// session is planned for transplant into the TARGET's own storage.
#[ test ]
fn ec6_to_plus_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec6-src";
  let jsonl = make_session_for( ch.path(), src, "eee-555" );
  let tgt = tempfile::TempDir::new().expect( "target tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "--from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let tgt_canon = std::fs::canonicalize( tgt.path() ).expect( "canonicalize target" );
  let target_storage = format!(
    "{ch_str}/projects/{}",
    df( tgt_canon.to_str().expect( "utf-8" ) )
  );
  assert!(
    stdout.contains(
      &format!( "# session-transplant: {} -> {target_storage}", jsonl.display() )
    ),
    "plan must copy the source session into the TARGET's own storage. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &format!( "cd {tgt_str}" ) ),
    "subprocess dir must be target `{tgt_str}`. Got:\n{stdout}"
  );
}

// ── EC-7: CLR_FROM env var equivalent ───────────────────────────────────────────

/// EC-7: `CLR_FROM` env var is equivalent to `--from`.
///
/// No `--from` on CLI; `CLR_FROM` provides the source path.
/// The same transplant plan must be produced as for the CLI flag.
///
/// Note: cannot use `run_dry_env` here because it calls `env_remove("CLR_FROM")`
/// which would strip the very variable this test passes in.
#[ test ]
fn ec7_clr_from_env_var()
{
  container_check();
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec7-src";
  let jsonl = make_session_for( ch.path(), src, "fff-666" );
  let home_str = ch.path().to_str().expect( "utf-8" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--dry-run", "Continue" ] )
    .env( "CLAUDE_HOME", home_str )
    .env( "CLR_FROM", src )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "dry-run failed (exit {})\nstdout: {}\nstderr: {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stdout ),
    String::from_utf8_lossy( &out.stderr ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout ).into_owned();
  assert!(
    stdout.contains( &format!( "# session-transplant: {} -> ", jsonl.display() ) ),
    "`CLR_FROM` must plan the source-session transplant. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "CLAUDE_CODE_SESSION_DIR=" ),
    "the dead CLAUDE_CODE_SESSION_DIR export must be gone (BUG-490). Got:\n{stdout}"
  );
}

// ── EC-8: --dry-run WYSIWYG reflects --from UUID ────────────────────────────────

/// EC-8: Dry-run accurately previews the transplant plan (WYSIWYG).
///
/// The `# session-transplant:` line names the exact source file the real run
/// would copy — and dry-run itself performs no copy.
#[ test ]
fn ec8_dry_run_wysiwyg_from()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec8-src";
  let jsonl = make_session_for( ch.path(), src, "ggg-777" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--from", src, "task" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  assert!(
    stdout.contains( &format!( "# session-transplant: {} -> ", jsonl.display() ) ),
    "dry-run must preview the transplant plan. Got:\n{stdout}"
  );
}

// ── US-1: Clone outward — -c injected from source session dir ─────────────────

/// US-1: `--to <tgt> --from <src>` clones outward.
///
/// The transplant plan targets the TARGET's storage; subprocess `cd` is the target.
#[ test ]
fn us1_clone_outward_continue_injected()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-project-a";
  let jsonl = make_session_for( ch.path(), src, "abc-123" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "--from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let tgt_canon = std::fs::canonicalize( tgt.path() ).expect( "canonicalize target" );
  let target_storage = format!(
    "{ch_str}/projects/{}",
    df( tgt_canon.to_str().expect( "utf-8" ) )
  );
  assert!(
    stdout.contains(
      &format!( "# session-transplant: {} -> {target_storage}", jsonl.display() )
    ),
    "clone outward must plan the copy into the target's storage. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &format!( "cd {tgt_str}" ) ),
    "subprocess dir must be target. Got:\n{stdout}"
  );
}

// ── US-2: Inject inward — runs in CWD, loads from source ──────────────────────

/// US-2: `--from <src>` (no `--to`) runs in CWD, loads from source.
///
/// The transplant plan targets the CWD's own storage; no `cd <src>` in output.
#[ test ]
fn us2_inject_inward_cwd_unchanged()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-project-b-inward";
  let jsonl = make_session_for( ch.path(), src, "def-456" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--from", src, "What did you do in B?" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let cwd = std::fs::canonicalize( std::env::current_dir().expect( "cwd" ) )
    .expect( "canonicalize cwd" );
  let cwd_storage = format!( "{ch_str}/projects/{}", df( cwd.to_str().expect( "utf-8" ) ) );
  assert!(
    stdout.contains(
      &format!( "# session-transplant: {} -> {cwd_storage}", jsonl.display() )
    ),
    "inward injection must plan the copy into the CWD's own storage. Got:\n{stdout}"
  );
  // No --to means no `cd` prefix — CWD is implicit.
  assert!(
    !stdout.contains( &format!( "cd {src}" ) ),
    "subprocess dir must NOT be source dir. Got:\n{stdout}"
  );
}

// ── US-3: No source history → fresh session ───────────────────────────────────

/// US-3: Source dir with no qualifying `.jsonl` → no `-c`; fresh session starts.
#[ test ]
fn us3_no_source_history_fresh_session()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-empty-source";
  // No session file — empty storage
  let stdout = run_dry_env(
    &[ "--from", src, "Start fresh" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!(
    !stdout.contains( " -c \"" ),
    "no history → must NOT have continue flag `-c`. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "# session-transplant:" ),
    "no history → no transplant plan. Got:\n{stdout}"
  );
}

// ── US-4: --to alone defaults --from to cwd (clone outward by default) ─────────

/// US-4: `--to <tgt>` alone (no `--from`) defaults the session source to the
/// current working directory and clones outward — NEW behavior once `--from`
/// gained a default-to-cwd rule matching `--to`/`--dir`'s existing one.
///
/// Before this default existed, `--to` alone was a pure working-directory
/// switch with no cross-loading at all. The same transplant plan that an
/// explicit `--from <cwd>` would produce must now be produced implicitly;
/// subprocess `cd` must still be the target.
#[ test ]
fn us4_to_alone_defaults_from_to_cwd()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let cwd = std::fs::canonicalize( std::env::current_dir().expect( "cwd" ) )
    .expect( "canonicalize cwd" );
  let jsonl = make_session_for( ch.path(), cwd.to_str().expect( "utf-8" ), "us4-cwd-src" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  let tgt_canon = std::fs::canonicalize( tgt.path() ).expect( "canonicalize target" );
  let target_storage = format!(
    "{ch_str}/projects/{}",
    df( tgt_canon.to_str().expect( "utf-8" ) )
  );
  assert!(
    stdout.contains(
      &format!( "# session-transplant: {} -> {target_storage}", jsonl.display() )
    ),
    "`--to` alone must default `--from` to cwd and clone outward. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &format!( "cd {tgt_str}" ) ),
    "subprocess dir must be target. Got:\n{stdout}"
  );
}

// ── US-5: --to alias is accepted ──────────────────────────────────────────────

/// US-5: `--to` alias sets subprocess working directory identically to `--dir`.
#[ test ]
fn us5_to_alias_sets_working_dir()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-proj-a-to";
  make_session_for( ch.path(), src, "abc-123" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--to", tgt_str, "--from", src, "test" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!(
    stdout.contains( &format!( "cd {tgt_str}" ) ),
    "`--to` must set subprocess dir to target. Got:\n{stdout}"
  );
}

// ── US-6: --session-dir is inert; --from governs ────────────────────────────────

/// US-6: the deprecated `--session-dir` never displaces `--from`'s computed path.
///
/// Fix(BUG-493): `--session-dir` is inert — no `CLAUDE_CODE_SESSION_DIR` export, no
/// transplant suppression. The `--from` source storage's session is transplanted
/// exactly as if `--session-dir` were absent.
#[ test ]
fn us6_session_dir_inert_from_governs()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-proj-a-prec";
  // Source session — governs, despite --session-dir also being present
  let jsonl = make_session_for( ch.path(), src, "abc-123" );
  // Deprecated override dir (must be ignored entirely)
  let override_dir = tempfile::TempDir::new().expect( "override tmpdir" );
  std::fs::write( override_dir.path().join( "xyz-789.jsonl" ), b"{}" )
    .expect( "write override session" );
  let override_str = override_dir.path().to_str().expect( "utf-8" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--from", src, "--session-dir", override_str, "test" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  assert!(
    !stdout.contains( "CLAUDE_CODE_SESSION_DIR=" ),
    "deprecated `--session-dir` must not export CLAUDE_CODE_SESSION_DIR. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( &format!( "# session-transplant: {} -> ", jsonl.display() ) ),
    "`--from` must govern the transplant plan despite `--session-dir`. Got:\n{stdout}"
  );
}

// ── US-7: Source session files not modified after cross-loaded run ─────────────

/// US-7: Source session `.jsonl` mtime and size are unchanged after a cross-loaded run.
///
/// `clr` only reads the session UUID from the source — it must never write to it.
/// Dry-run mode is used so no subprocess runs, but the `--from` path setup
/// code still executes during command building.
#[ test ]
fn us7_source_session_files_not_modified()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/us28-proj-a-immutable";
  let jsonl = make_session_for( ch.path(), src, "abc-123" );
  let tgt = tempfile::TempDir::new().expect( "tgt tmpdir" );
  let tgt_str = tgt.path().to_str().expect( "utf-8" );

  let meta_before = std::fs::metadata( &jsonl ).expect( "stat before" );
  let mtime_before = meta_before.modified().expect( "mtime before" );
  let size_before  = meta_before.len();

  run_dry_env(
    &[ "--to", tgt_str, "--from", src, "Continue" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );

  let meta_after = std::fs::metadata( &jsonl ).expect( "stat after" );
  let mtime_after = meta_after.modified().expect( "mtime after" );
  let size_after  = meta_after.len();

  assert_eq!(
    mtime_before, mtime_after,
    "source session `.jsonl` mtime must not change after cross-loaded run"
  );
  assert_eq!(
    size_before, size_after,
    "source session `.jsonl` size must not change after cross-loaded run"
  );
}

// ── US-8: bare invocation (neither --from nor --to) is a no-op ─────────────────

/// US-8: A bare invocation — neither `--from` nor `--to` given — is a guaranteed
/// no-op regression lock for the `--from` default-to-cwd rule (see US-4).
///
/// Both source and target resolve to the same cwd storage, so the builder's
/// self-copy guard (`target_storage == src_storage`) suppresses the transplant
/// plan unconditionally — behavior must be byte-identical to before `--from`
/// gained a default: ordinary continuation detection still finds a session
/// already in cwd's own storage and injects `-c`, but no `# session-transplant:`
/// line is ever printed.
#[ test ]
fn us8_bare_invocation_neither_flag_is_noop()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let cwd = std::fs::canonicalize( std::env::current_dir().expect( "cwd" ) )
    .expect( "canonicalize cwd" );
  make_session_for( ch.path(), cwd.to_str().expect( "utf-8" ), "us8-bare-cwd" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  assert!(
    !stdout.contains( "# session-transplant:" ),
    "bare invocation must be a no-op — no transplant plan when neither flag is given. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( " -c \"" ),
    "bare invocation must still detect the existing cwd session via ordinary \
     continuation and inject `-c`. Got:\n{stdout}"
  );
}

// ── EC-9: relative source path resolves against cwd ───────────────────────────

/// EC-9: a relative `--from` value resolves to the physical absolute
/// path (cwd-anchored, symlinks resolved) before storage-name encoding.
///
/// Claude derives storage names from its physical getcwd, so an unresolved
/// relative value (e.g. `./src`) would encode literally (`---src`) and silently
/// miss the real storage dir — no `-c`, fresh session, no warning.  The builder
/// canonicalizes the value, so the encoded name must match the canonicalized
/// absolute form and the seeded session must be found (`-c` injected).
#[ test ]
fn ec9_relative_source_path_resolves_against_cwd()
{
  container_check();
  let ch     = tempfile::TempDir::new().expect( "tmpdir" );
  let parent = tempfile::TempDir::new().expect( "parent tmpdir" );
  let src_abs = parent.path().join( "relsrc" );
  std::fs::create_dir_all( &src_abs ).expect( "create relative source dir" );
  // Expected encoding uses the canonicalized physical path — immune to /tmp symlinks.
  let src_canon = std::fs::canonicalize( &src_abs ).expect( "canonicalize source" );
  make_session_for( ch.path(), src_canon.to_str().expect( "utf-8" ), "rel-901" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = std::process::Command::new( bin )
    .args( [ "--dry-run", "--from", "./relsrc", "Continue" ] )
    .current_dir( parent.path() )
    .env( "CLAUDE_HOME", ch_str )
    .env_remove( "CLR_DIR" )
    .env_remove( "CLR_SESSION_DIR" )
    .env_remove( "CLR_FROM" )
    .output()
    .expect( "failed to invoke clr binary" );
  assert!(
    out.status.success(),
    "dry-run failed (exit {})\nstderr: {}",
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stderr ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout ).into_owned();
  let expected_src = format!(
    "{ch_str}/projects/{}/rel-901.jsonl",
    df( src_canon.to_str().expect( "utf-8" ) )
  );
  assert!(
    stdout.contains( &format!( "# session-transplant: {expected_src} -> " ) ),
    "relative source must resolve to the canonical storage's session file. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( " -c \"" ),
    "resolved source session must be found → `-c` injected. Got:\n{stdout}"
  );
}

// ── EC-10: empty source value is ignored ──────────────────────────────────────

/// EC-10: `--from ""` is treated as absent — same empty-is-identity rule
/// as `--subdir ""` (BUG-229 precedent).
///
/// Without the filter, the empty value fell through `encode_path()`'s error path
/// into the `-unknown` fallback storage name, actively exporting
/// `CLAUDE_CODE_SESSION_DIR=<projects>/-unknown` and redirecting subprocess
/// session storage to a shared junk dir.
#[ test ]
fn ec10_empty_source_value_ignored()
{
  let ch = tempfile::TempDir::new().expect( "tmpdir" );
  let stdout = run_dry_env(
    &[ "--from", "", "task" ],
    &[ ( "CLAUDE_HOME", ch.path().to_str().expect( "utf-8" ) ) ],
  );
  assert!(
    !stdout.contains( "CLAUDE_CODE_SESSION_DIR=" ),
    "empty `--from` must not export a session dir. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "-unknown" ),
    "empty `--from` must not fall into the `-unknown` storage dir. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "# session-transplant:" ),
    "empty `--from` must not plan a transplant. Got:\n{stdout}"
  );
}

// ── EC-11: JSON config key `from` ───────────────────────────────────────────────

/// EC-11: the `from` args-file key (JSON config route) behaves like the
/// CLI flag — third input route alongside `--from` and `CLR_FROM`.
///
/// `json_config.rs` maps the key onto `parsed.from` when the CLI did not
/// set it; the computed source storage path and `-c` injection must match EC-1.
#[ test ]
fn ec11_json_config_from_key()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec11-src";
  let jsonl = make_session_for( ch.path(), src, "hhh-888" );
  let cfg_dir  = tempfile::TempDir::new().expect( "cfg tmpdir" );
  let cfg_path = cfg_dir.path().join( "args.json" );
  std::fs::write( &cfg_path, format!( "{{\"from\": \"{src}\"}}" ) )
    .expect( "write args-file" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "--args-file", cfg_path.to_str().expect( "utf-8" ), "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ) ],
  );
  assert!(
    stdout.contains( &format!( "# session-transplant: {} -> ", jsonl.display() ) ),
    "args-file `from` key must plan the transplant. Got:\n{stdout}"
  );
  assert!(
    stdout.contains( " -c \"" ),
    "args-file `from` with existing session must inject `-c`. Got:\n{stdout}"
  );
}

// ── EC-12: old CLR_SESSION_FROM env var is inert ────────────────────────────────

/// EC-12: the pre-rename `CLR_SESSION_FROM` env var is inert — renamed to
/// `CLR_FROM` (a breaking rename, not an alias), so setting the old name must
/// NOT plan a transplant from its target.
#[ test ]
fn ec12_old_clr_session_from_env_var_inert()
{
  let ch  = tempfile::TempDir::new().expect( "tmpdir" );
  let src = "/tmp/076ec12-old-env-src";
  let jsonl = make_session_for( ch.path(), src, "ec12-old-env" );
  let ch_str = ch.path().to_str().expect( "utf-8" );
  let stdout = run_dry_env(
    &[ "Continue" ],
    &[ ( "CLAUDE_HOME", ch_str ), ( "CLR_SESSION_FROM", src ) ],
  );
  assert!(
    !stdout.contains( &format!( "# session-transplant: {} -> ", jsonl.display() ) ),
    "the old `CLR_SESSION_FROM` env var must be inert (renamed to `CLR_FROM`) — \
     it must NOT plan a transplant from its target. Got:\n{stdout}"
  );
}

// ── Sanity: run_cli is used to trigger assert_container via at least one test ──
//
// The local `run_dry_env` helper duplicates the container check, so the
// import of `run_cli` satisfies the "used" lint for the helper module.
// This test also verifies that `clr --help` is reachable from the binary.
#[ test ]
fn sanity_clr_binary_reachable()
{
  let out = run_cli( &[ "--help" ] );
  assert!( out.status.success(), "clr binary must be reachable" );
}

//! Integration tests for FORK-mode topics — cache-preserving topic sessions.
//!
//! ## Source
//!
//! - Command docs: `tests/docs/cli/command/11_topic.md`, `12_topics.md`
//! - Param docs: `tests/docs/cli/param/028_topic.md`, `088_topic_mode.md`
//!
//! ## Coverage
//!
//! | Test | Verifies | Group |
//! |------|----------|-------|
//! | F01 | first use, empty storage → `--session-id <UUIDv5>` alone, no dir created | Arg shape |
//! | F02 | first use with base session → `--resume <src> --fork-session --session-id <topic>` | Arg shape |
//! | F03 | repeat use → `--resume <topic>` alone; `# topic-resume:` preview | Arg shape |
//! | F04 | pre-existing `-<name>` dir → legacy dir mode wins, no fork args | Coexistence |
//! | F05 | explicit `--topic-mode dir` on fresh topic → dir mode | Mode select |
//! | F06 | `--topic-mode fork` + `--from` → exit 1 contradiction error | Mode select |
//! | F07 | `--topic-mode fork` + `--global` → exit 1 contradiction error | Mode select |
//! | F08 | `--new-session` on repeat topic → exit 1 naming `topics --file` | New session |
//! | F09 | `--new-session` on fresh topic → source suppressed (`source=fresh`) | New session |
//! | F10 | `CLR_TOPIC_MODE=dir` env → dir mode | Mode select |
//! | F11 | `CLR_TOPIC_MODE=fork` overrides a pre-existing `-<name>` dir | Mode select |
//! | F12 | dry-run writes no registry entry and creates no dir | Side effects |
//! | F13 | real run passes `--session-id` through argv and records the registry | Registry |
//! | F14 | print-gated invocation (no message, non-TTY) injects no fork args, no preview | Gating |
//! | F15 | `topics --file NAME` output == core `topic_session_file` (parity contract) | Parity |
//! | F16 | `topics --file` guards: slash name, missing value, `--path` exclusivity | Guards |
//! | F17 | `topics` listing shows fork (registry) and dir (scan) rows with MODE column | Listing |
//! | F18 | auto-naming skips a candidate whose fork session file already exists | Auto-naming |
//!
//! ## Isolation contract
//!
//! Every test runs via `run_cli_in_dir_isolated` — cwd pinned to a canonicalized
//! tempdir (the fork rule hashes the CANONICAL physical base, so a symlinked
//! `/tmp` would silently change every expected UUID), all topic-affecting env
//! scrubbed, and only the vars under test re-added (`CLAUDE_HOME` for storage,
//! `CLR_TOPIC_REGISTRY_DIR` for the registry). Expected paths are assembled from
//! parts — never by calling env-reading helpers in the TEST process, whose env
//! differs from the subprocess's.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ df, exit_code, make_session_for, run_cli_in_dir_isolated, stderr_str, stdout_str };

use tempfile::TempDir;

/// Canonicalized tempdir base + its fork-topic UUID for `name`.
fn fork_fixture( name : &str ) -> ( TempDir, std::path::PathBuf, String )
{
  let project = TempDir::new().unwrap();
  let canon = project.path().canonicalize().unwrap();
  let uuid = claude_storage_core::topic_session_id( &canon, name ).unwrap().as_str().to_owned();
  ( project, canon, uuid )
}

/// `CLAUDE_HOME`-rooted storage dir for `canon` — where fork sessions live.
fn storage_dir( claude_home : &std::path::Path, canon : &std::path::Path ) -> std::path::PathBuf
{
  claude_home.join( "projects" ).join( df( canon.to_str().unwrap() ) )
}

// ─── F01 ────────────────────────────────────────────────────────────────────

/// F01: first use with empty storage → `--session-id <UUIDv5>` alone.
///
/// No `--resume`, no `--fork-session`, no legacy `-c`, no `-<name>` dir created;
/// preview line names the plan with `source=fresh`.
#[ test ]
fn fork_f01_first_use_fresh_creates_session_id()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, uuid ) = fork_fixture( "x" );

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "hello" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let s = stdout_str( &out );
  assert!( s.contains( &format!( "--session-id {uuid}" ) ), "must create the deterministic id; got:\n{s}" );
  assert!( !s.contains( "--resume" ), "fresh first use must not resume; got:\n{s}" );
  assert!( !s.contains( "--fork-session" ), "nothing to fork from; got:\n{s}" );
  assert!( !s.contains( " -c " ), "fork mode must not inject legacy -c; got:\n{s}" );
  assert!(
    s.contains( &format!( "# topic-fork: topic=x session={uuid} source=fresh" ) ),
    "preview must name the fresh plan; got:\n{s}"
  );
  assert!( !canon.join( "-x" ).exists(), "fork mode must not create a topic dir" );
}

// ─── F02 ────────────────────────────────────────────────────────────────────

/// F02: first use with a base session → fork from it.
///
/// Args must be `--resume <src> --fork-session --session-id <topic-uuid>` —
/// the cache-preserving shape (same cwd, forked history).
#[ test ]
fn fork_f02_first_use_forks_from_latest_base_session()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, uuid ) = fork_fixture( "x" );
  let src = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
  let _ = make_session_for( claude_home.path(), canon.to_str().unwrap(), src );

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "hello" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let s = stdout_str( &out );
  assert!( s.contains( &format!( "--resume {src}" ) ), "must resume the base session; got:\n{s}" );
  assert!( s.contains( "--fork-session" ), "must fork, not continue in place; got:\n{s}" );
  assert!( s.contains( &format!( "--session-id {uuid}" ) ), "fork must land on the deterministic id; got:\n{s}" );
  assert!(
    s.contains( &format!( "# topic-fork: topic=x session={uuid} source={src}" ) ),
    "preview must name the fork source; got:\n{s}"
  );
}

// ─── F03 ────────────────────────────────────────────────────────────────────

/// F03: repeat use → plain `--resume <topic-uuid>`.
///
/// The topic session already exists; no `--fork-session`, no `--session-id`,
/// preview switches to `# topic-resume:`. A base session is also seeded to
/// prove the repeat check has precedence over source selection.
#[ test ]
fn fork_f03_repeat_use_resumes_topic_session()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, uuid ) = fork_fixture( "x" );
  let _ = make_session_for( claude_home.path(), canon.to_str().unwrap(), "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa" );
  let _ = make_session_for( claude_home.path(), canon.to_str().unwrap(), &uuid );

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "hello" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let s = stdout_str( &out );
  assert!( s.contains( &format!( "--resume {uuid}" ) ), "repeat must resume the topic session; got:\n{s}" );
  assert!( !s.contains( "--fork-session" ), "repeat must not fork again; got:\n{s}" );
  assert!( !s.contains( "--session-id" ), "repeat must not pass --session-id; got:\n{s}" );
  assert!(
    s.contains( &format!( "# topic-resume: topic=x session={uuid}" ) ),
    "preview must show the resume plan; got:\n{s}"
  );
}

// ─── F04 ────────────────────────────────────────────────────────────────────

/// F04: a pre-existing `-<name>` dir keeps the legacy dir mode.
///
/// No fork args, no fork preview; the working dir moves into `-x`.
#[ test ]
fn fork_f04_existing_dash_dir_keeps_dir_mode()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, _uuid ) = fork_fixture( "x" );
  std::fs::create_dir( canon.join( "-x" ) ).unwrap();

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "hello" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let s = stdout_str( &out );
  assert!( !s.contains( "# topic-fork" ), "existing dir topic must stay in dir mode; got:\n{s}" );
  assert!( !s.contains( "--fork-session" ), "dir mode must not fork; got:\n{s}" );
  assert!( !s.contains( "--session-id" ), "dir mode must not pass --session-id; got:\n{s}" );
  assert!( s.contains( "/-x" ), "dir mode must enter the -x topic dir; got:\n{s}" );
}

// ─── F05 ────────────────────────────────────────────────────────────────────

/// F05: explicit `--topic-mode dir` forces dir mode on a fresh topic.
#[ test ]
fn fork_f05_explicit_topic_mode_dir_forces_dir_mode()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, _uuid ) = fork_fixture( "x" );

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "--topic-mode", "dir", "hello" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let s = stdout_str( &out );
  assert!( !s.contains( "# topic-fork" ), "--topic-mode dir must suppress fork; got:\n{s}" );
  assert!( s.contains( "/-x" ), "--topic-mode dir must use the -x topic dir; got:\n{s}" );
}

// ─── F06 ────────────────────────────────────────────────────────────────────

/// F06: `--topic-mode fork` + `--from` → exit 1.
///
/// Fork mode stays in the base dir and forks its own storage — a transplant
/// source contradicts it.
#[ test ]
fn fork_f06_fork_mode_rejects_from()
{
  let claude_home = TempDir::new().unwrap();
  let from_dir = TempDir::new().unwrap();
  let ( _project, canon, _uuid ) = fork_fixture( "x" );

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "--topic-mode", "fork", "--from", from_dir.path().to_str().unwrap(), "hello" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 1 );
  assert!(
    stderr_str( &out ).contains( "--topic-mode fork cannot be combined with --from" ),
    "stderr must name the contradiction; got: {}",
    stderr_str( &out )
  );
}

// ─── F07 ────────────────────────────────────────────────────────────────────

/// F07: `--topic-mode fork` + `--global` → exit 1.
#[ test ]
fn fork_f07_fork_mode_rejects_global()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, _uuid ) = fork_fixture( "x" );

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "--topic-mode", "fork", "--global", "hello" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 1 );
  assert!(
    stderr_str( &out ).contains( "--topic-mode fork cannot be combined with --global" ),
    "stderr must name the contradiction; got: {}",
    stderr_str( &out )
  );
}

// ─── F08 ────────────────────────────────────────────────────────────────────

/// F08: `--new-session` on a repeat fork topic → exit 1.
///
/// The topic IS its deterministic session — restarting means deleting the
/// session file; the error must point at `topics --file` for that path.
#[ test ]
fn fork_f08_new_session_errors_on_repeat_topic()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, uuid ) = fork_fixture( "x" );
  let _ = make_session_for( claude_home.path(), canon.to_str().unwrap(), &uuid );

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "--new-session", "hello" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 1 );
  let e = stderr_str( &out );
  assert!( e.contains( "--new-session cannot restart fork-mode topic 'x'" ), "got: {e}" );
  assert!( e.contains( "topics --file" ), "error must point at the session-file recipe; got: {e}" );
}

// ─── F09 ────────────────────────────────────────────────────────────────────

/// F09: `--new-session` on a FRESH fork topic suppresses the fork source.
///
/// A base session exists, but `--new-session` means "start clean" — so the
/// topic is created with `--session-id` alone instead of forking history.
#[ test ]
fn fork_f09_new_session_fresh_suppresses_source()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, uuid ) = fork_fixture( "x" );
  let _ = make_session_for( claude_home.path(), canon.to_str().unwrap(), "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa" );

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "--new-session", "hello" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let s = stdout_str( &out );
  assert!( s.contains( &format!( "--session-id {uuid}" ) ), "must still create the deterministic id; got:\n{s}" );
  assert!( !s.contains( "--resume" ), "--new-session must suppress the fork source; got:\n{s}" );
  assert!( s.contains( "source=fresh" ), "preview must show fresh creation; got:\n{s}" );
}

// ─── F10 ────────────────────────────────────────────────────────────────────

/// F10: `CLR_TOPIC_MODE=dir` env selects dir mode like the CLI flag.
#[ test ]
fn fork_f10_env_topic_mode_dir_forces_dir_mode()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, _uuid ) = fork_fixture( "x" );

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "hello" ],
    &canon,
    &[
      ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ),
      ( "CLR_TOPIC_MODE", "dir" ),
    ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let s = stdout_str( &out );
  assert!( !s.contains( "# topic-fork" ), "CLR_TOPIC_MODE=dir must suppress fork; got:\n{s}" );
  assert!( s.contains( "/-x" ), "CLR_TOPIC_MODE=dir must use the -x topic dir; got:\n{s}" );
}

// ─── F11 ────────────────────────────────────────────────────────────────────

/// F11: explicit `CLR_TOPIC_MODE=fork` overrides a pre-existing `-<name>` dir.
///
/// Explicit mode beats the dir-exists heuristic — the escape hatch for moving
/// an old dir topic to fork mode without deleting the dir first.
#[ test ]
fn fork_f11_env_topic_mode_fork_overrides_existing_dir()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, uuid ) = fork_fixture( "x" );
  std::fs::create_dir( canon.join( "-x" ) ).unwrap();

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "hello" ],
    &canon,
    &[
      ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ),
      ( "CLR_TOPIC_MODE", "fork" ),
    ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let s = stdout_str( &out );
  assert!(
    s.contains( &format!( "--session-id {uuid}" ) ),
    "explicit fork must win over the existing -x dir; got:\n{s}"
  );
  assert!( s.contains( "# topic-fork" ), "explicit fork must plan a fork; got:\n{s}" );
}

// ─── F12 ────────────────────────────────────────────────────────────────────

/// F12: dry-run is side-effect-free — no registry entry, no topic dir.
///
/// The registry write is a run-path effect (BUG-231/319 dry-run purity rule).
#[ test ]
fn fork_f12_dry_run_writes_no_registry_no_dir()
{
  let claude_home = TempDir::new().unwrap();
  let registry = TempDir::new().unwrap();
  let ( _project, canon, _uuid ) = fork_fixture( "x" );

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x", "hello" ],
    &canon,
    &[
      ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ),
      ( "CLR_TOPIC_REGISTRY_DIR", registry.path().to_str().unwrap() ),
    ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let entries : Vec< _ > = std::fs::read_dir( registry.path() ).unwrap().collect();
  assert!( entries.is_empty(), "dry-run must not write the registry; found: {entries:?}" );
  assert!( !canon.join( "-x" ).exists(), "dry-run must not create a topic dir" );
}

// ─── F13 ────────────────────────────────────────────────────────────────────

/// F13: a real run passes `--session-id` through argv and records the registry.
///
/// The fake claude writes `<storage>/$id.jsonl` for whatever `--session-id` it
/// receives — the file appearing under the topic's deterministic name proves
/// the argv wiring end-to-end, not just the dry-run preview.
#[ cfg( unix ) ]
#[ test ]
fn fork_f13_real_run_records_registry()
{
  let claude_home = TempDir::new().unwrap();
  let registry = TempDir::new().unwrap();
  let ( _project, canon, uuid ) = fork_fixture( "x" );
  let storage = storage_dir( claude_home.path(), &canon );

  let body = format!(
    "id=\"\"\n\
     while [ $# -gt 0 ]; do\n\
     if [ \"$1\" = \"--session-id\" ]; then id=\"$2\"; fi\n\
     shift\n\
     done\n\
     if [ -n \"$id\" ]; then\n\
     mkdir -p '{storage}'\n\
     printf '{{}}' > '{storage}'/\"$id\".jsonl\n\
     fi\n\
     exit 0",
    storage = storage.display()
  );
  let ( _fake_dir, path_val ) = cli_binary_test_helpers::fake_claude_dir( &body );

  let out = run_cli_in_dir_isolated(
    &[ "--max-sessions", "0", "--topic", "x", "hello" ],
    &canon,
    &[
      ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ),
      ( "CLR_TOPIC_REGISTRY_DIR", registry.path().to_str().unwrap() ),
      ( "PATH", &path_val ),
    ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  assert!(
    storage.join( format!( "{uuid}.jsonl" ) ).exists(),
    "claude must have received --session-id {uuid} via argv"
  );
  let reg_file = registry.path().join( df( canon.to_str().unwrap() ) );
  let content = std::fs::read_to_string( &reg_file )
    .unwrap_or_else( | e | panic!( "registry file {} must exist: {e}", reg_file.display() ) );
  assert!(
    content.lines().any( | l | l == "x" ),
    "registry must record the topic name; got: {content:?}"
  );
}

// ─── F14 ────────────────────────────────────────────────────────────────────

/// F14: print-gated invocation (no message, non-TTY stdin) injects no fork args.
///
/// The BUG-426/435 gate suppresses resume/fork/create for a bare print-mode
/// run with nothing to say — and the preview/registry plan is dropped with it,
/// so the dry-run output stays consistent with the argv it shows.
#[ test ]
fn fork_f14_no_message_print_gating_suppresses_fork()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, _uuid ) = fork_fixture( "x" );

  let out = run_cli_in_dir_isolated(
    &[ "--dry-run", "--topic", "x" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let s = stdout_str( &out );
  assert!( !s.contains( "--session-id" ), "gated run must not create a session; got:\n{s}" );
  assert!( !s.contains( "--resume" ), "gated run must not resume; got:\n{s}" );
  assert!( !s.contains( "# topic-fork" ), "preview must match the gated argv; got:\n{s}" );
}

// ─── F15 ────────────────────────────────────────────────────────────────────

/// F15: `topics --file NAME` == core `topic_session_file` (parity contract).
///
/// The `claude_storage` side pins `.session.path path::<base> topic::NAME` to the
/// same core value (SP-6 in `cli_cmd_session_path_test.rs`), so the two CLIs
/// are byte-identical by transitivity.
#[ test ]
fn fork_f15_topics_file_matches_core_rule()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, uuid ) = fork_fixture( "x" );

  let out = run_cli_in_dir_isolated(
    &[ "topics", "--file", "x" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let expected = format!(
    "{}\n",
    storage_dir( claude_home.path(), &canon ).join( format!( "{uuid}.jsonl" ) ).display()
  );
  assert_eq!( stdout_str( &out ), expected, "--file must print the core-computed session file path" );
}

// ─── F16 ────────────────────────────────────────────────────────────────────

/// F16: `topics --file` guards — slash name, missing value, `--path` exclusivity.
#[ test ]
fn fork_f16_topics_file_guards()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, _uuid ) = fork_fixture( "x" );
  let env : &[ ( &str, &str ) ] = &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ];

  let slash = run_cli_in_dir_isolated( &[ "topics", "--file", "a/b" ], &canon, env );
  assert_eq!( exit_code( &slash ), 1 );
  assert!(
    stderr_str( &slash ).contains( "--file must be a single topic name" ),
    "got: {}",
    stderr_str( &slash )
  );

  let missing = run_cli_in_dir_isolated( &[ "topics", "--file" ], &canon, env );
  assert_eq!( exit_code( &missing ), 1 );
  assert!(
    stderr_str( &missing ).contains( "--file requires a value" ),
    "got: {}",
    stderr_str( &missing )
  );

  let both = run_cli_in_dir_isolated( &[ "topics", "--path", "x", "--file", "x" ], &canon, env );
  assert_eq!( exit_code( &both ), 1 );
  assert!(
    stderr_str( &both ).contains( "mutually exclusive" ),
    "got: {}",
    stderr_str( &both )
  );
}

// ─── F17 ────────────────────────────────────────────────────────────────────

/// F17: the listing merges fork (registry) and dir (scan) topics with a MODE column.
///
/// A registry entry whose session file was never created still lists (sessions 0)
/// — the name stays reserved; a `-<name>` dir lists as mode `dir`.
#[ test ]
fn fork_f17_topics_listing_shows_fork_and_dir_rows()
{
  let claude_home = TempDir::new().unwrap();
  let registry = TempDir::new().unwrap();
  let ( _project, canon, _uuid ) = fork_fixture( "x" );
  std::fs::write( registry.path().join( df( canon.to_str().unwrap() ) ), "x\n" ).unwrap();
  std::fs::create_dir( canon.join( "-y" ) ).unwrap();

  let out = run_cli_in_dir_isolated(
    &[ "topics" ],
    &canon,
    &[
      ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ),
      ( "CLR_TOPIC_REGISTRY_DIR", registry.path().to_str().unwrap() ),
    ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let s = stdout_str( &out );
  assert!( s.contains( "MODE" ), "header must show the MODE column; got:\n{s}" );
  let fork_row = s.lines().find( | l | l.starts_with( 'x' ) ).unwrap_or_else( || panic!( "no row for fork topic x; got:\n{s}" ) );
  assert!( fork_row.contains( "fork" ), "topic x must list as fork; got: {fork_row}" );
  let dir_row = s.lines().find( | l | l.starts_with( 'y' ) ).unwrap_or_else( || panic!( "no row for dir topic y; got:\n{s}" ) );
  assert!( dir_row.contains( "dir" ), "topic y must list as dir; got: {dir_row}" );
}

// ─── F18 ────────────────────────────────────────────────────────────────────

/// F18: auto-naming skips a candidate whose FORK session file already exists.
///
/// Fork topics create no `-<name>` directory, so `name_is_taken`'s first two
/// probes (directory existence, dir-mode session storage) are blind to them —
/// only the third probe (the candidate's own `UUIDv5` session file, non-empty)
/// detects the collision. Without it, `clr topic "orphan topic"` after
/// `clr --topic orphan-topic` would silently resume the existing fork topic's
/// conversation instead of starting a fresh one.
///
/// Companion to `topic_command_test.rs` T02 (directory-existence signal) and
/// T10/T11 (dir-mode-storage signal): this pins the fork-session signal.
#[ test ]
fn fork_f18_auto_naming_skips_existing_fork_topic()
{
  let claude_home = TempDir::new().unwrap();
  let ( _project, canon, taken_uuid ) = fork_fixture( "orphan-topic" );
  // Seed the taken candidate's fork session file in the BASE's own storage —
  // exactly where a real `clr --topic orphan-topic` run would have left it.
  let _seeded = make_session_for( claude_home.path(), canon.to_str().unwrap(), &taken_uuid );

  let out = run_cli_in_dir_isolated(
    &[ "topic", "--dry-run", "orphan topic" ],
    &canon,
    &[ ( "CLAUDE_HOME", claude_home.path().to_str().unwrap() ) ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let s = stdout_str( &out );
  // Trailing-space boundary: `orphan-topic` is a string prefix of `orphan-topic-2`,
  // and the preview always follows the name with ` session=`.
  assert!(
    !s.contains( "topic=orphan-topic " ),
    "auto-naming must NOT hand out a name whose fork session already exists; got:\n{s}"
  );
  assert!(
    s.contains( "topic=orphan-topic-2 " ),
    "auto-naming must disambiguate past the existing fork topic to orphan-topic-2; got:\n{s}"
  );
}

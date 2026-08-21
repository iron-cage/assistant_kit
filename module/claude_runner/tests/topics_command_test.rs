//! `topics` Subcommand Integration Tests
//!
//! ## Purpose
//!
//! Verify `clr topics` — the read-only counterpart to `clr topic`. Two forms:
//! a listing of every topic directory under a resolved base, and `--path NAME`,
//! a pure name-to-path resolver. Neither form spawns a subprocess or creates a
//! directory.
//!
//! ## Strategy
//!
//! Every case runs the real `clr` binary against a `tempfile::TempDir` base, via
//! `run_cli_in_dir` (so the cwd-default base is the fixture, never the host's real
//! working directory) or with `CLR_TOPIC_HOME` pointed at a fixture (so `--global`
//! never touches the host's real `<temp-dir>/clr-topic`). No mocking: the assertions
//! read the binary's actual stdout/stderr and the actual filesystem.
//!
//! TP-15 seeds a real `<CLAUDE_HOME>/projects/<df(dir)>/<uuid>.jsonl` so the session
//! count is measured against genuine Claude Code storage rather than a stub.
//!
//! TP-16 is the cross-check that gives the whole command its point: the path
//! `topics --path NAME` prints must be the same directory `--topic NAME` actually
//! runs in. Both come from `topic_path::topic_dir()`, and this test is what keeps
//! that true if either caller is ever changed independently.
//!
//! ## Corner Cases Covered (mirrors `tests/docs/cli/command/12_topics.md` TP-1..TP-16)
//!
//! - TP-01: two topic dirs under cwd — both listed, sorted by name, under a header
//! - TP-02: base with no topics — `no topics in <base>` on stderr, stdout empty, exit 0
//! - TP-03: non-hyphen dirs and plain files are not topics
//! - TP-04: a bare `-` directory is not a topic (empty name never round-trips)
//! - TP-05: `--dir <base>` lists that base regardless of cwd
//! - TP-06: `--global` lists `$CLR_TOPIC_HOME` instead of cwd
//! - TP-07: `--dir` outranks `--global` when both are given
//! - TP-08: `--path NAME` prints `<base>/-NAME` and exits 0
//! - TP-09: `--path` resolves a non-existent topic and creates nothing (pure computation)
//! - TP-10: `--path` honors `--global`
//! - TP-11: `--path` containing `/` is rejected, exit 1 (BUG-230 guard parity)
//! - TP-12: unknown option rejected, exit 1
//! - TP-13: `--path` without a value rejected, exit 1
//! - TP-14: `topics help` / `--help` / `-h` all print topics help, exit 0
//! - TP-15: SESSIONS counts real `*.jsonl` files; a never-entered topic reports 0
//! - TP-16: `topics --path X` == the effective dir `--dry-run --topic X` reports

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ df, exit_code, run_cli, run_cli_in_dir, run_cli_with_env, stderr_str, stdout_str };

/// Create `<base>/-<name>` and return its absolute path.
fn make_topic( base : &std::path::Path, name : &str ) -> std::path::PathBuf
{
  let dir = base.join( format!( "-{name}" ) );
  std::fs::create_dir_all( &dir ).expect( "create topic fixture dir" );
  dir
}

/// Data rows of a listing: everything after the `NAME MODE SESSIONS PATH` header.
fn rows( stdout : &str ) -> Vec< &str >
{
  stdout.lines().skip( 1 ).filter( | l | !l.trim().is_empty() ).collect()
}

/// TP-01: two topic directories under the cwd are both listed, sorted by name,
/// beneath a `NAME MODE SESSIONS PATH` header.
#[ test ]
fn tp01_lists_topics_in_cwd_sorted()
{
  let tmp = tempfile::TempDir::new().expect( "temp base" );
  make_topic( tmp.path(), "zebra" );
  make_topic( tmp.path(), "alpha" );

  let out = run_cli_in_dir( &[ "topics" ], tmp.path(), &[] );
  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );

  assert!( stdout.starts_with( "NAME" ), "listing must start with a NAME header. Got:\n{stdout}" );
  assert!(
    stdout.contains( "MODE" ) && stdout.contains( "SESSIONS" ) && stdout.contains( "PATH" ),
    "header must name all 4 columns. Got:\n{stdout}"
  );

  let data = rows( &stdout );
  assert_eq!( data.len(), 2, "exactly 2 topics expected. Got:\n{stdout}" );
  assert!( data[ 0 ].starts_with( "alpha" ), "rows must be sorted by name. Got:\n{stdout}" );
  assert!( data[ 1 ].starts_with( "zebra" ), "rows must be sorted by name. Got:\n{stdout}" );
}

/// TP-02: a base holding no topics reports on stderr and still exits 0 — an empty
/// result is not an error, so `clr topics` is safe in a `set -e` script.
#[ test ]
fn tp02_empty_base_reports_on_stderr_exit_0()
{
  let tmp = tempfile::TempDir::new().expect( "temp base" );

  let out = run_cli_in_dir( &[ "topics" ], tmp.path(), &[] );
  assert_eq!( exit_code( &out ), 0, "an empty base is not an error" );
  assert!( stdout_str( &out ).is_empty(), "nothing may go to stdout when there are no topics" );
  assert!(
    stderr_str( &out ).contains( "no topics in" ),
    "stderr must explain the empty result. Got: {}", stderr_str( &out )
  );
}

/// TP-03: ordinary directories and files are not topics — only `-`-prefixed
/// directories are, which is what keeps a project's own source tree out of the listing.
#[ test ]
fn tp03_non_hyphen_entries_are_not_topics()
{
  let tmp = tempfile::TempDir::new().expect( "temp base" );
  std::fs::create_dir_all( tmp.path().join( "src" ) ).expect( "plain dir" );
  std::fs::write( tmp.path().join( "-not-a-dir.txt" ), b"x" ).expect( "plain file" );
  make_topic( tmp.path(), "real" );

  let out    = run_cli_in_dir( &[ "topics" ], tmp.path(), &[] );
  let stdout = stdout_str( &out );
  let data   = rows( &stdout );

  assert_eq!( data.len(), 1, "only the one real topic may be listed. Got:\n{stdout}" );
  assert!( data[ 0 ].starts_with( "real" ), "Got:\n{stdout}" );
  assert!( !stdout.contains( "src" ), "a plain directory is not a topic. Got:\n{stdout}" );
  assert!( !stdout.contains( "not-a-dir" ), "a `-`-prefixed *file* is not a topic. Got:\n{stdout}" );
}

/// TP-04: a bare `-` directory yields an empty topic name, which cannot round-trip
/// back to itself through `--topic ""` (identity), so it is not a topic.
#[ test ]
fn tp04_bare_hyphen_dir_is_not_a_topic()
{
  let tmp = tempfile::TempDir::new().expect( "temp base" );
  std::fs::create_dir_all( tmp.path().join( "-" ) ).expect( "bare hyphen dir" );

  let out = run_cli_in_dir( &[ "topics" ], tmp.path(), &[] );
  assert_eq!( exit_code( &out ), 0 );
  assert!(
    stdout_str( &out ).is_empty(),
    "a bare `-` directory must not be listed as a topic. Got:\n{}", stdout_str( &out )
  );
}

/// TP-05: `--dir` selects the base explicitly, independent of where the command runs.
#[ test ]
fn tp05_explicit_dir_selects_base()
{
  let elsewhere = tempfile::TempDir::new().expect( "cwd" );
  let base      = tempfile::TempDir::new().expect( "base" );
  make_topic( base.path(), "from-dir" );

  let base_str = base.path().to_str().expect( "utf8 base" );
  let out      = run_cli_in_dir( &[ "topics", "--dir", base_str ], elsewhere.path(), &[] );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  assert!(
    stdout_str( &out ).contains( "from-dir" ),
    "--dir must select the listed base regardless of cwd. Got:\n{}", stdout_str( &out )
  );
}

/// TP-06: `--global` lists `$CLR_TOPIC_HOME` rather than the cwd.
#[ test ]
fn tp06_global_lists_topic_home()
{
  let cwd  = tempfile::TempDir::new().expect( "cwd" );
  let home = tempfile::TempDir::new().expect( "topic home" );
  make_topic( cwd.path(),  "local-only" );
  make_topic( home.path(), "global-only" );

  let home_str = home.path().to_str().expect( "utf8 home" );
  let out      = run_cli_in_dir( &[ "topics", "--global" ], cwd.path(), &[ ( "CLR_TOPIC_HOME", home_str ) ] );
  let stdout   = stdout_str( &out );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  assert!( stdout.contains( "global-only" ), "--global must list $CLR_TOPIC_HOME. Got:\n{stdout}" );
  assert!( !stdout.contains( "local-only" ), "--global must not list the cwd. Got:\n{stdout}" );
}

/// TP-07: an explicit path outranks a named default — `--dir` wins over `--global`.
#[ test ]
fn tp07_dir_outranks_global()
{
  let cwd  = tempfile::TempDir::new().expect( "cwd" );
  let home = tempfile::TempDir::new().expect( "topic home" );
  let base = tempfile::TempDir::new().expect( "explicit base" );
  make_topic( home.path(), "global-only" );
  make_topic( base.path(), "explicit-only" );

  let home_str = home.path().to_str().expect( "utf8 home" );
  let base_str = base.path().to_str().expect( "utf8 base" );
  let out      = run_cli_in_dir(
    &[ "topics", "--global", "--dir", base_str ],
    cwd.path(),
    &[ ( "CLR_TOPIC_HOME", home_str ) ],
  );
  let stdout = stdout_str( &out );

  assert!( stdout.contains( "explicit-only" ), "--dir must win over --global. Got:\n{stdout}" );
  assert!( !stdout.contains( "global-only" ), "--global must be ignored when --dir is given. Got:\n{stdout}" );
}

/// TP-08: `--path NAME` prints exactly one absolute path, `<base>/-NAME`, and exits 0.
#[ test ]
fn tp08_path_resolves_name_under_cwd()
{
  let tmp = tempfile::TempDir::new().expect( "temp base" );

  let out = run_cli_in_dir( &[ "topics", "--path", "auth-refactor" ], tmp.path(), &[] );
  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );

  let printed = stdout_str( &out ).trim().to_string();
  let expected = tmp.path().join( "-auth-refactor" );
  assert_eq!(
    printed, expected.to_str().expect( "utf8 expected path" ),
    "--path must print <base>/-<NAME>"
  );
  assert_eq!( stdout_str( &out ).lines().count(), 1, "--path prints exactly one line" );
}

/// TP-09: `--path` is a pure computation — it answers "where would this live?",
/// so it resolves a topic that does not exist and creates nothing on the way.
#[ test ]
fn tp09_path_is_pure_computation()
{
  let tmp = tempfile::TempDir::new().expect( "temp base" );

  let out = run_cli_in_dir( &[ "topics", "--path", "never-created" ], tmp.path(), &[] );
  assert_eq!( exit_code( &out ), 0, "a non-existent topic still resolves" );

  let printed = std::path::PathBuf::from( stdout_str( &out ).trim() );
  assert!( printed.ends_with( "-never-created" ), "Got: {}", printed.display() );
  assert!( !printed.exists(), "--path must never create the directory it names" );
  assert_eq!(
    std::fs::read_dir( tmp.path() ).expect( "read base" ).count(), 0,
    "--path must leave the base untouched"
  );
}

/// TP-10: `--path` resolves against the global home when `--global` is given, which
/// is what makes a global topic addressable from any directory in a later shell.
#[ test ]
fn tp10_path_honors_global()
{
  let cwd  = tempfile::TempDir::new().expect( "cwd" );
  let home = tempfile::TempDir::new().expect( "topic home" );

  let home_str = home.path().to_str().expect( "utf8 home" );
  let out      = run_cli_in_dir(
    &[ "topics", "--global", "--path", "notes" ],
    cwd.path(),
    &[ ( "CLR_TOPIC_HOME", home_str ) ],
  );

  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  assert_eq!(
    stdout_str( &out ).trim(),
    home.path().join( "-notes" ).to_str().expect( "utf8 expected" ),
    "--global --path must resolve under $CLR_TOPIC_HOME"
  );
}

/// TP-11: a `--path` value containing `/` is rejected — a topic name is a single
/// directory name component, never a path. Mirrors `--topic`'s own BUG-230 guard.
#[ test ]
fn tp11_path_rejects_slash()
{
  let out = run_cli( &[ "topics", "--path", "a/b" ] );
  assert_eq!( exit_code( &out ), 1, "a slashed topic name must be rejected" );
  assert!(
    stderr_str( &out ).contains( "single topic name" ),
    "stderr must explain the single-name-component constraint. Got: {}", stderr_str( &out )
  );
}

/// TP-12: an unknown option exits 1 with a diagnostic naming it — same contract as
/// every other subcommand.
#[ test ]
fn tp12_unknown_option_exits_1()
{
  let out = run_cli( &[ "topics", "--not-a-real-flag" ] );
  assert_eq!( exit_code( &out ), 1, "unknown option must exit 1" );
  assert!(
    stderr_str( &out ).contains( "--not-a-real-flag" ),
    "stderr must name the unknown option. Got: {}", stderr_str( &out )
  );
}

/// TP-13: a value-taking option with no value exits 1 rather than silently
/// swallowing the next token or defaulting.
#[ test ]
fn tp13_path_without_value_exits_1()
{
  let out = run_cli( &[ "topics", "--path" ] );
  assert_eq!( exit_code( &out ), 1, "--path without a value must exit 1" );
  assert!(
    stderr_str( &out ).contains( "requires a value" ),
    "stderr must say the value is missing. Got: {}", stderr_str( &out )
  );
}

/// TP-14: all three help forms print topics-specific help and exit 0. The bare
/// positional `help` needs its own intercept (BUG-249 pattern) — without it, a
/// dispatcher parses `help` as a value or an unknown option.
#[ test ]
fn tp14_help_forms_print_topics_help()
{
  for form in [ "help", "--help", "-h" ]
  {
    let out = run_cli( &[ "topics", form ] );
    assert_eq!( exit_code( &out ), 0, "`clr topics {form}` must exit 0. stderr: {}", stderr_str( &out ) );
    let stdout = stdout_str( &out );
    assert!( stdout.contains( "topics" ), "`clr topics {form}` must print topics help. Got:\n{stdout}" );
    assert!( stdout.contains( "--path" ), "topics help must document --path. Got:\n{stdout}" );
  }
}

/// TP-15: SESSIONS reflects real `*.jsonl` files in the topic's own Claude Code
/// storage — 0 for a topic created but never entered, non-zero once a session exists.
#[ test ]
fn tp15_session_count_reflects_real_storage()
{
  let base = tempfile::TempDir::new().expect( "temp base" );
  let home = tempfile::TempDir::new().expect( "claude home" );

  let entered  = make_topic( base.path(), "entered" );
  let _virgin  = make_topic( base.path(), "virgin" );

  // Seed one real session file in `entered`'s own encoded storage.
  let storage = home.path()
    .join( "projects" )
    .join( df( entered.to_str().expect( "utf8 topic dir" ) ) );
  std::fs::create_dir_all( &storage ).expect( "create session storage" );
  std::fs::write( storage.join( "11111111-2222-3333-4444-555555555555.jsonl" ), b"{}\n" )
    .expect( "seed session file" );

  let home_str = home.path().to_str().expect( "utf8 home" );
  let out      = run_cli_in_dir( &[ "topics" ], base.path(), &[ ( "CLAUDE_HOME", home_str ) ] );
  let stdout   = stdout_str( &out );

  let entered_row = rows( &stdout ).into_iter().find( | l | l.starts_with( "entered" ) )
    .unwrap_or_else( || panic!( "entered topic must be listed. Got:\n{stdout}" ) ).to_string();
  let virgin_row  = rows( &stdout ).into_iter().find( | l | l.starts_with( "virgin" ) )
    .unwrap_or_else( || panic!( "virgin topic must be listed. Got:\n{stdout}" ) ).to_string();

  assert!(
    entered_row.split_whitespace().nth( 1 ) == Some( "1" ),
    "a topic with one session file must report 1. Got row: {entered_row}"
  );
  assert!(
    virgin_row.split_whitespace().nth( 1 ) == Some( "0" ),
    "a never-entered topic must report 0. Got row: {virgin_row}"
  );
}

/// TP-16: the resolver and the runner must never disagree.
///
/// This is the guarantee the whole command rests on: `topics --path NAME` is only
/// useful if it names the directory `--topic NAME` actually runs in. Both go through
/// `topic_path::topic_dir()`, and this test fails the moment either caller stops.
#[ test ]
fn tp16_path_matches_dry_run_effective_dir()
{
  let cwd  = tempfile::TempDir::new().expect( "cwd" );
  let home = tempfile::TempDir::new().expect( "topic home" );
  let home_str = home.path().to_str().expect( "utf8 home" );
  let env : &[ ( &str, &str ) ] = &[ ( "CLR_TOPIC_HOME", home_str ), ( "HOME", "/tmp/clr-isolated-home" ) ];

  let resolved = run_cli_in_dir( &[ "topics", "--global", "--path", "cross-check" ], cwd.path(), env );
  assert_eq!( exit_code( &resolved ), 0, "stderr: {}", stderr_str( &resolved ) );
  let path = stdout_str( &resolved ).trim().to_string();
  assert!( !path.is_empty(), "resolver produced no path" );

  let dry = run_cli_in_dir(
    &[ "--dry-run", "--global", "--topic", "cross-check", "x" ],
    cwd.path(),
    env,
  );
  assert_eq!( exit_code( &dry ), 0, "stderr: {}", stderr_str( &dry ) );

  assert!(
    stdout_str( &dry ).contains( &path ),
    "dry-run must run in exactly the directory `topics --path` resolved.\nresolved: {path}\ndry-run:\n{}",
    stdout_str( &dry )
  );
}

/// Guard: `topics` must be a known subcommand, not fall through to `run` and be
/// treated as a message. Cheap insurance against a missing `KNOWN_SUBCOMMANDS` entry.
#[ test ]
fn tp17_topics_is_a_dispatched_subcommand()
{
  let out = run_cli_with_env( &[ "topics", "--path", "x" ], &[ ( "HOME", "/tmp/clr-isolated-home" ) ] );
  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  assert!(
    stdout_str( &out ).trim().ends_with( "-x" ),
    "`topics` must dispatch to the resolver, not be parsed as a run message. Got:\n{}", stdout_str( &out )
  );
}

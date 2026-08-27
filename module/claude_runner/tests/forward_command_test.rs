//! Integration tests for `clr delegate` and `clr broadcast` — topic fan-out.
//!
//! ## Source
//!
//! - Command docs: `tests/docs/cli/command/16_delegate.md`, `17_broadcast.md`
//! - Feature doc: `docs/feature/009_topic_forwarding.md`
//! - Fan-out mechanics: `claude_runner_core/docs/feature/007_bounded_fanout.md`
//!
//! ## Coverage
//!
//! | Test | Verifies | Group |
//! |------|----------|-------|
//! | fw01 | `delegate --dry-run` picks one live topic and prints its command | Delegate |
//! | fw02 | `--seed` makes the draw reproducible across invocations | Delegate |
//! | fw03 | two different seeds can reach two different topics | Delegate |
//! | fw04 | `--pick random` is accepted and reported back | Delegate |
//! | fw05 | an invalid `--pick` value is rejected with the valid set named | Guards |
//! | fw06 | `broadcast --dry-run` emits one command per live topic | Broadcast |
//! | fw07 | every emitted command carries `--topic-mode` alongside `--topic` | Invariant |
//! | fw08 | a fork topic and a dir topic of the SAME name both get a command | Invariant |
//! | fw09 | a registry name with no session file is not a target | Live filter |
//! | fw10 | a `-name` directory with no session storage is not a target | Live filter |
//! | fw11 | an empty base exits 1 rather than silently succeeding | Guards |
//! | fw12 | a missing message exits 1 for both commands | Guards |
//! | fw13 | `--concurrency` is echoed, and clamped to the topic count | Broadcast |
//! | fw14 | `--pick`/`--seed` on broadcast, `-j` on delegate, are rejected by name | Guards |
//! | fw15 | `--` ends option parsing so a hyphen-leading message is text | Guards |
//! | fw16 | both commands are dispatched subcommands with their own help | Dispatch |
//!
//! ## Isolation contract
//!
//! Every test runs via `TopicBase`, whose cwd is a canonicalized tempdir — the fork
//! rule hashes the CANONICAL physical base, so a symlinked `/tmp` would silently
//! change every expected UUID — and which re-adds only `CLAUDE_HOME` and
//! `CLR_TOPIC_REGISTRY_DIR`. Every assertion is on `--dry-run` output: these tests
//! must never spawn Claude Code.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, stderr_str, stdout_str, TopicBase };

/// The `cmd:` lines of a dry-run, which are the planned child invocations.
fn cmd_lines( out : &std::process::Output ) -> Vec< String >
{
  stdout_str( out )
    .lines()
    .filter( | l | l.starts_with( "cmd: " ) )
    .map( str::to_owned )
    .collect()
}

/// The value of a single `key: value` line in a dry-run.
fn field( out : &std::process::Output, key : &str ) -> Option< String >
{
  stdout_str( out )
    .lines()
    .find_map( | l | l.strip_prefix( &format!( "{key}: " ) ).map( str::to_owned ) )
}

// ─── delegate ───────────────────────────────────────────────────────────────

// fw01: one live topic in, one command out, naming that topic
#[ test ]
fn fw01_delegate_dry_run_picks_a_live_topic()
{
  let base = TopicBase::new();
  base.fork_topic( "review", true );

  let out = base.run( &[ "delegate", "--dry-run", "summarize" ] );
  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );

  assert_eq!( field( &out, "topic" ).as_deref(), Some( "review (fork)" ) );
  let cmds = cmd_lines( &out );
  assert_eq!( cmds.len(), 1, "delegate plans exactly one child, got {cmds:?}" );
  assert!( cmds[ 0 ].contains( "--topic review" ), "got {}", cmds[ 0 ] );
  assert!( cmds[ 0 ].contains( "--message \"summarize\"" ), "got {}", cmds[ 0 ] );
}

// fw02: the same seed over the same topic list always picks the same topic
#[ test ]
fn fw02_seed_makes_the_draw_reproducible()
{
  let base = TopicBase::new();
  for name in [ "alpha", "beta", "gamma", "delta" ]
  {
    base.fork_topic( name, true );
  }

  let first = base.run( &[ "delegate", "--dry-run", "--seed", "12345", "go" ] );
  let second = base.run( &[ "delegate", "--dry-run", "--seed", "12345", "go" ] );
  assert_eq!( exit_code( &first ), 0, "stderr: {}", stderr_str( &first ) );

  assert_eq!
  (
    field( &first, "topic" ),
    field( &second, "topic" ),
    "one seed must always reach one topic",
  );
  assert_eq!( field( &first, "seed" ).as_deref(), Some( "12345" ) );
}

// fw03: the draw is a draw — some other seed reaches some other topic
#[ test ]
fn fw03_different_seeds_reach_different_topics()
{
  let base = TopicBase::new();
  for name in [ "alpha", "beta", "gamma", "delta" ]
  {
    base.fork_topic( name, true );
  }

  // Four topics and `seed % 4`, so these four seeds cover all four indices —
  // asserting on the set rather than on any one seed's outcome keeps this test
  // independent of the candidate list's internal order.
  let picked : std::collections::HashSet< String > = [ "0", "1", "2", "3" ]
    .iter()
    .filter_map( | s | field( &base.run( &[ "delegate", "--dry-run", "--seed", s, "go" ] ), "topic" ) )
    .collect();
  assert_eq!( picked.len(), 4, "four seeds over four topics must reach all four, got {picked:?}" );
}

// fw04: the policy is reported back, so a run says how it chose
#[ test ]
fn fw04_pick_random_is_accepted_and_reported()
{
  let base = TopicBase::new();
  base.fork_topic( "solo", true );

  let out = base.run( &[ "delegate", "--dry-run", "--pick", "random", "go" ] );
  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  assert_eq!( field( &out, "pick" ).as_deref(), Some( "random" ) );
}

// fw05: an unknown policy names the valid set rather than falling back silently
#[ test ]
fn fw05_invalid_pick_is_rejected()
{
  let base = TopicBase::new();
  base.fork_topic( "solo", true );

  let out = base.run( &[ "delegate", "--dry-run", "--pick", "whatever", "go" ] );
  assert_eq!( exit_code( &out ), 1 );
  let err = stderr_str( &out );
  assert!( err.contains( "idle" ) && err.contains( "random" ), "stderr: {err}" );
}

// ─── broadcast ──────────────────────────────────────────────────────────────

// fw06: every live topic gets a command, none is skipped
#[ test ]
fn fw06_broadcast_dry_run_covers_every_live_topic()
{
  let base = TopicBase::new();
  base.fork_topic( "review", true );
  base.fork_topic( "docs", true );
  base.dir_topic( "bench", true );

  let out = base.run( &[ "broadcast", "--dry-run", "status?" ] );
  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );

  assert_eq!( field( &out, "topics" ).as_deref(), Some( "3" ) );
  let cmds = cmd_lines( &out );
  assert_eq!( cmds.len(), 3, "one command per live topic, got {cmds:?}" );
  for name in [ "review", "docs", "bench" ]
  {
    assert!
    (
      cmds.iter().any( | c | c.contains( &format!( "--topic {name} " ) ) ),
      "'{name}' has no command in {cmds:?}",
    );
  }
}

// fw07: the mode always travels with the name — a bare --topic would silently
// redirect every fork topic to a dir-mode twin
#[ test ]
fn fw07_every_command_carries_topic_mode()
{
  let base = TopicBase::new();
  base.fork_topic( "review", true );
  base.dir_topic( "bench", true );

  let out = base.run( &[ "broadcast", "--dry-run", "status?" ] );
  let cmds = cmd_lines( &out );
  assert_eq!( cmds.len(), 2 );
  assert!
  (
    cmds.iter().any( | c | c.contains( "--topic review --topic-mode fork" ) ),
    "fork topic lost its mode: {cmds:?}",
  );
  assert!
  (
    cmds.iter().any( | c | c.contains( "--topic bench --topic-mode dir" ) ),
    "dir topic lost its mode: {cmds:?}",
  );
}

// fw08: one name held by both mechanisms is two topics, and both are reached —
// this is the case a name-keyed dedupe silently halves
#[ test ]
fn fw08_same_name_in_both_modes_yields_two_commands()
{
  let base = TopicBase::new();
  base.fork_topic( "shared", true );
  base.dir_topic( "shared", true );

  let out = base.run( &[ "broadcast", "--dry-run", "status?" ] );
  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );

  assert_eq!( field( &out, "topics" ).as_deref(), Some( "2" ) );
  let cmds = cmd_lines( &out );
  assert!
  (
    cmds.iter().any( | c | c.contains( "--topic shared --topic-mode fork" ) ),
    "the fork half is missing: {cmds:?}",
  );
  assert!
  (
    cmds.iter().any( | c | c.contains( "--topic shared --topic-mode dir" ) ),
    "the dir half is missing: {cmds:?}",
  );
}

// ─── live filter ────────────────────────────────────────────────────────────

// fw09: a recorded name whose session file is gone is not a target — sending to
// it would mint a new conversation, not continue one
#[ test ]
fn fw09_registry_name_without_session_is_not_a_target()
{
  let base = TopicBase::new();
  base.fork_topic( "live-one", true );
  base.fork_topic( "ghost", false );

  let out = base.run( &[ "broadcast", "--dry-run", "status?" ] );
  assert_eq!( field( &out, "topics" ).as_deref(), Some( "1" ) );
  let cmds = cmd_lines( &out );
  assert!( !cmds.iter().any( | c | c.contains( "ghost" ) ), "ghost was targeted: {cmds:?}" );
}

// fw10: a `-name` directory with no session storage is not a target — this is
// what keeps fan-out out of -daemon/, -gate/, and ./-NNNN_* scratch dirs
#[ test ]
fn fw10_hyphen_dir_without_storage_is_not_a_target()
{
  let base = TopicBase::new();
  base.dir_topic( "real", true );
  base.dir_topic( "0001_scratch", false );

  let out = base.run( &[ "broadcast", "--dry-run", "status?" ] );
  assert_eq!( field( &out, "topics" ).as_deref(), Some( "1" ) );
  let cmds = cmd_lines( &out );
  assert!( !cmds.iter().any( | c | c.contains( "scratch" ) ), "scratch dir targeted: {cmds:?}" );
}

// ─── guards ─────────────────────────────────────────────────────────────────

// fw11: nowhere to send is a failure, not a quiet success
#[ test ]
fn fw11_empty_base_exits_one()
{
  let base = TopicBase::new();

  for command in [ "delegate", "broadcast" ]
  {
    let out = base.run( &[ command, "--dry-run", "go" ] );
    assert_eq!( exit_code( &out ), 1, "{command} on an empty base must fail" );
    assert!
    (
      stderr_str( &out ).contains( "no live topics" ),
      "{command} stderr: {}",
      stderr_str( &out ),
    );
  }
}

// fw12: a forward with no prompt is not a forward
#[ test ]
fn fw12_missing_message_exits_one()
{
  let base = TopicBase::new();
  base.fork_topic( "solo", true );

  for command in [ "delegate", "broadcast" ]
  {
    let out = base.run( &[ command, "--dry-run" ] );
    assert_eq!( exit_code( &out ), 1, "{command} without a message must fail" );
    assert!
    (
      stderr_str( &out ).contains( "requires a message" ),
      "{command} stderr: {}",
      stderr_str( &out ),
    );
  }
}

// fw13: the bound is reported, and never claims to exceed the work available
#[ test ]
fn fw13_concurrency_is_reported_and_clamped()
{
  let base = TopicBase::new();
  base.fork_topic( "one", true );
  base.fork_topic( "two", true );

  let explicit = base.run( &[ "broadcast", "--dry-run", "-j", "1", "go" ] );
  assert_eq!( field( &explicit, "concurrency" ).as_deref(), Some( "1" ) );

  let over = base.run( &[ "broadcast", "--dry-run", "--concurrency", "50", "go" ] );
  assert_eq!
  (
    field( &over, "concurrency" ).as_deref(),
    Some( "2" ),
    "a bound above the topic count is clamped to it",
  );
}

// fw14: a flag from the sibling command is an error naming where it belongs,
// not an ignored no-op
#[ test ]
fn fw14_cross_command_flags_are_rejected_by_name()
{
  let base = TopicBase::new();
  base.fork_topic( "solo", true );

  let on_broadcast = base.run( &[ "broadcast", "--dry-run", "--pick", "idle", "go" ] );
  assert_eq!( exit_code( &on_broadcast ), 1 );
  assert!
  (
    stderr_str( &on_broadcast ).contains( "clr delegate" ),
    "stderr: {}",
    stderr_str( &on_broadcast ),
  );

  let on_delegate = base.run( &[ "delegate", "--dry-run", "-j", "2", "go" ] );
  assert_eq!( exit_code( &on_delegate ), 1 );
  assert!
  (
    stderr_str( &on_delegate ).contains( "clr broadcast" ),
    "stderr: {}",
    stderr_str( &on_delegate ),
  );
}

// fw15: `--` ends option parsing, so a message may start with a hyphen
#[ test ]
fn fw15_double_dash_ends_option_parsing()
{
  let base = TopicBase::new();
  base.fork_topic( "solo", true );

  let out = base.run( &[ "delegate", "--dry-run", "--", "--not-a-flag" ] );
  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  let cmds = cmd_lines( &out );
  assert!( cmds[ 0 ].contains( "--message \"--not-a-flag\"" ), "got {}", cmds[ 0 ] );
}

// fw16: both are real dispatched subcommands, each with its own help
#[ test ]
fn fw16_both_are_dispatched_subcommands_with_help()
{
  let base = TopicBase::new();

  for ( command, marker ) in [ ( "delegate", "chosen for you" ), ( "broadcast", "every live topic" ) ]
  {
    let out = base.run( &[ command, "--help" ] );
    assert_eq!( exit_code( &out ), 0, "{command} --help must exit 0" );
    let s = stdout_str( &out );
    assert!( s.contains( marker ), "{command} help lacks '{marker}': {s}" );
    assert!( s.contains( "USAGE:" ), "{command} help lacks a USAGE section" );
  }

  // And both are listed in the top-level help, so they are discoverable.
  let root = base.run( &[ "help" ] );
  let s = stdout_str( &root );
  assert!( s.contains( "delegate" ), "top-level help omits delegate" );
  assert!( s.contains( "broadcast" ), "top-level help omits broadcast" );
}

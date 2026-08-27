//! Integration tests for `clr pool` — provisioning anonymous topics.
//!
//! ## Source
//!
//! - Command doc: `tests/docs/cli/command/18_pool.md`
//! - Feature doc: `docs/feature/009_topic_forwarding.md`
//! - Naming rules: `claude_topic_core/docs/feature/004_topic_pool.md`
//!
//! ## Coverage
//!
//! | Test | Verifies | Group |
//! |------|----------|-------|
//! | pl01 | an empty base plans exactly `--count` topics, named `t1..tN` | Target |
//! | pl02 | the count is also accepted positionally | Target |
//! | pl03 | `--count` is a target, not an increment — a full pool plans nothing | Idempotence |
//! | pl04 | a partially-filled pool tops up only what is missing | Idempotence |
//! | pl05 | gaps are filled before the range is extended | Idempotence |
//! | pl06 | a pool name whose session is gone counts as missing, not as present | Live filter |
//! | pl07 | non-pool topics do not count toward the target | Counting |
//! | pl08 | `--prefix` renames the pool and re-counts against that prefix alone | Prefix |
//! | pl09 | a prefix ending in a digit is rejected, naming the ambiguity | Guards |
//! | pl10 | an empty prefix and one containing `/` are rejected | Guards |
//! | pl11 | `--topic-mode dir` plans dir-mode topics; fork is the default | Mode |
//! | pl12 | one slot, both mechanisms — a fork `t1` blocks a dir `t1` | Mode |
//! | pl13 | a missing count exits 1; a non-numeric count exits 1 quoting it | Guards |
//! | pl14 | `--count 0` is an accepted no-op, not an error | Guards |
//! | pl15 | a second positional is rejected, suggesting `--message` | Guards |
//! | pl16 | `--message` overrides the seed prompt; the default is used otherwise | Seed |
//! | pl17 | `--concurrency` is echoed, and clamped to the number being created | Concurrency |
//! | pl18 | `pool` is a dispatched subcommand with its own help, listed in `clr help` | Dispatch |
//!
//! ## Isolation contract
//!
//! Identical to `forward_command_test.rs`: every case runs through `TopicBase`,
//! whose cwd is a canonicalized tempdir and whose environment re-adds only
//! `CLAUDE_HOME` and `CLR_TOPIC_REGISTRY_DIR`.
//!
//! **Every case runs `--dry-run`.** `clr pool` without it starts a real Claude Code
//! session per missing name — the one command in this crate whose non-dry-run path
//! costs money by construction. What the children then do is `clr run`'s contract,
//! covered by `topic_fork_test.rs` (F01–F03, F13) against a stubbed `claude`.

mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ exit_code, run_cli, stderr_str, stdout_str, TopicBase };

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

/// The topic names a dry-run plans to create, in the order planned.
fn planned_names( out : &std::process::Output ) -> Vec< String >
{
  cmd_lines( out )
    .iter()
    .filter_map( | line | line.split( "--topic " ).nth( 1 )?.split_whitespace().next().map( str::to_owned ) )
    .collect()
}

// ─── the target ─────────────────────────────────────────────────────────────

// pl01: an empty base gets exactly N topics, named from 1
#[ test ]
fn pl01_empty_base_plans_the_full_count()
{
  let base = TopicBase::new();

  let out = base.run( &[ "pool", "--dry-run", "--count", "3" ] );
  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );

  assert_eq!( field( &out, "target" ).as_deref(), Some( "3" ) );
  assert_eq!( field( &out, "existing" ).as_deref(), Some( "0" ) );
  assert_eq!( field( &out, "create" ).as_deref(), Some( "3" ) );
  assert_eq!( planned_names( &out ), vec![ "t1", "t2", "t3" ] );
}

// pl02: `clr pool 3` is the same request as `clr pool --count 3`
#[ test ]
fn pl02_count_is_accepted_positionally()
{
  let base = TopicBase::new();

  let flagged = base.run( &[ "pool", "--dry-run", "--count", "2" ] );
  let positional = base.run( &[ "pool", "--dry-run", "2" ] );
  assert_eq!( exit_code( &positional ), 0, "stderr: {}", stderr_str( &positional ) );
  assert_eq!( stdout_str( &flagged ), stdout_str( &positional ) );
}

// ─── idempotence ────────────────────────────────────────────────────────────

// pl03: the second run of a script is the one nobody is watching
#[ test ]
fn pl03_a_full_pool_plans_nothing()
{
  let base = TopicBase::new();
  for name in [ "t1", "t2", "t3" ]
  {
    base.fork_topic( name, true );
  }

  let out = base.run( &[ "pool", "--dry-run", "--count", "3" ] );
  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  assert_eq!( field( &out, "existing" ).as_deref(), Some( "3" ) );
  assert_eq!( field( &out, "create" ).as_deref(), Some( "0" ) );
  assert!( cmd_lines( &out ).is_empty(), "a full pool must plan no children" );
}

// pl04: a target is a target — topping up creates only the difference
#[ test ]
fn pl04_partial_pool_tops_up_the_difference()
{
  let base = TopicBase::new();
  base.fork_topic( "t1", true );
  base.fork_topic( "t2", true );

  let out = base.run( &[ "pool", "--dry-run", "--count", "4" ] );
  assert_eq!( field( &out, "existing" ).as_deref(), Some( "2" ) );
  assert_eq!( field( &out, "create" ).as_deref(), Some( "2" ) );
  assert_eq!( planned_names( &out ), vec![ "t3", "t4" ] );
}

// pl05: a deleted topic leaves a slot, not a permanent hole
#[ test ]
fn pl05_gaps_are_filled_before_the_range_extends()
{
  let base = TopicBase::new();
  base.fork_topic( "t1", true );
  base.fork_topic( "t3", true );

  let out = base.run( &[ "pool", "--dry-run", "--count", "4" ] );
  assert_eq!
  (
    planned_names( &out ),
    vec![ "t2", "t4" ],
    "with t1 and t3 present, a target of 4 means t2 and t4 — never t4 and t5",
  );
}

// pl06: counting the full set instead of the live set would report a pool of four
// while `clr broadcast` reached three — the exact silent-partial-success this
// whole command family is built to avoid
#[ test ]
fn pl06_a_dead_pool_name_counts_as_missing()
{
  let base = TopicBase::new();
  base.fork_topic( "t1", true );
  // Known to the registry, but its session file was never written.
  base.fork_topic( "t2", false );

  let out = base.run( &[ "pool", "--dry-run", "--count", "2" ] );
  assert_eq!( field( &out, "existing" ).as_deref(), Some( "1" ) );
  assert_eq!( planned_names( &out ), vec![ "t2" ], "the dead slot must be refilled" );
}

// pl07: N must not depend on unrelated work living in the same directory
#[ test ]
fn pl07_non_pool_topics_do_not_count()
{
  let base = TopicBase::new();
  base.fork_topic( "auth-refactor", true );
  base.dir_topic( "bench", true );
  // `t01` does not round-trip through `format!("{prefix}{index}")`, so it is not
  // a pool name either — admitting it would make the mapping many-to-one.
  base.fork_topic( "t01", true );

  let out = base.run( &[ "pool", "--dry-run", "--count", "2" ] );
  assert_eq!( field( &out, "existing" ).as_deref(), Some( "0" ) );
  assert_eq!( planned_names( &out ), vec![ "t1", "t2" ] );
}

// ─── the prefix ─────────────────────────────────────────────────────────────

// pl08: a renamed pool is a different pool, counted on its own terms
#[ test ]
fn pl08_prefix_renames_and_recounts_the_pool()
{
  let base = TopicBase::new();
  base.fork_topic( "t1", true );
  base.fork_topic( "t2", true );

  let out = base.run( &[ "pool", "--dry-run", "--prefix", "worker", "--count", "2" ] );
  assert_eq!( field( &out, "prefix" ).as_deref(), Some( "worker" ) );
  assert_eq!
  (
    field( &out, "existing" ).as_deref(),
    Some( "0" ),
    "t1/t2 are not `worker` topics and must not count toward a `worker` target",
  );
  assert_eq!( planned_names( &out ), vec![ "worker1", "worker2" ] );
}

// pl09: `t1` + index 2 is `t12`, which also reads as `t1` + index 2 the other way
// round — the ambiguity is refused rather than resolved
#[ test ]
fn pl09_prefix_ending_in_a_digit_is_rejected()
{
  let base = TopicBase::new();

  let out = base.run( &[ "pool", "--dry-run", "--prefix", "t1", "--count", "2" ] );
  assert_eq!( exit_code( &out ), 1 );
  let err = stderr_str( &out );
  assert!( err.contains( "digit" ), "the reason must name the ambiguity: {err}" );
}

// pl10: the other two ways a prefix cannot produce a usable topic name
#[ test ]
fn pl10_empty_and_path_like_prefixes_are_rejected()
{
  let base = TopicBase::new();

  let empty = base.run( &[ "pool", "--dry-run", "--prefix", "", "--count", "2" ] );
  assert_eq!( exit_code( &empty ), 1 );
  assert!( stderr_str( &empty ).contains( "empty" ), "stderr: {}", stderr_str( &empty ) );

  let slashed = base.run( &[ "pool", "--dry-run", "--prefix", "a/b", "--count", "2" ] );
  assert_eq!( exit_code( &slashed ), 1 );
  assert!( stderr_str( &slashed ).contains( '/' ), "stderr: {}", stderr_str( &slashed ) );
}

// ─── the mechanism ──────────────────────────────────────────────────────────

// pl11: the mode is chosen here and travels with every planned name
#[ test ]
fn pl11_topic_mode_selects_the_mechanism()
{
  let base = TopicBase::new();

  let default = base.run( &[ "pool", "--dry-run", "--count", "1" ] );
  assert_eq!( field( &default, "mode" ).as_deref(), Some( "fork" ) );
  assert!
  (
    cmd_lines( &default )[ 0 ].contains( "--topic t1 --topic-mode fork" ),
    "got {}", cmd_lines( &default )[ 0 ],
  );

  let dir = base.run( &[ "pool", "--dry-run", "--topic-mode", "dir", "--count", "1" ] );
  assert_eq!( field( &dir, "mode" ).as_deref(), Some( "dir" ) );
  assert!
  (
    cmd_lines( &dir )[ 0 ].contains( "--topic t1 --topic-mode dir" ),
    "got {}", cmd_lines( &dir )[ 0 ],
  );
}

// pl12: a `t1` in each mechanism is two topics but one slot — the pool counts
// slots, and which mechanism a slot's occupant uses is not the count's business
#[ test ]
fn pl12_one_slot_across_both_mechanisms()
{
  let base = TopicBase::new();
  base.fork_topic( "t1", true );

  let out = base.run( &[ "pool", "--dry-run", "--topic-mode", "dir", "--count", "2" ] );
  assert_eq!( field( &out, "existing" ).as_deref(), Some( "1" ) );
  assert_eq!
  (
    planned_names( &out ),
    vec![ "t2" ],
    "the fork-mode t1 already fills slot 1; a dir-mode t1 would be a second topic in one slot",
  );
}

// ─── guards ─────────────────────────────────────────────────────────────────

// pl13: `clr pool` with no number has not been told what to do
#[ test ]
fn pl13_missing_and_unparsable_counts_are_rejected()
{
  let base = TopicBase::new();

  let missing = base.run( &[ "pool", "--dry-run" ] );
  assert_eq!( exit_code( &missing ), 1 );
  assert!( stderr_str( &missing ).contains( "count" ), "stderr: {}", stderr_str( &missing ) );

  let garbage = base.run( &[ "pool", "--dry-run", "--count", "four" ] );
  assert_eq!( exit_code( &garbage ), 1 );
  assert!( stderr_str( &garbage ).contains( "'four'" ), "the bad value must be quoted back: {}", stderr_str( &garbage ) );
}

// pl14: a script that computed N == 0 has asked for nothing, which is a thing to
// do rather than an error to report
#[ test ]
fn pl14_zero_is_an_accepted_no_op()
{
  let base = TopicBase::new();

  let out = base.run( &[ "pool", "--dry-run", "--count", "0" ] );
  assert_eq!( exit_code( &out ), 0, "stderr: {}", stderr_str( &out ) );
  assert_eq!( field( &out, "create" ).as_deref(), Some( "0" ) );
  assert!( cmd_lines( &out ).is_empty() );
}

// pl15: a stray word is a typo, not prose — `pool` takes a number
#[ test ]
fn pl15_a_second_positional_is_rejected()
{
  let base = TopicBase::new();

  let out = base.run( &[ "pool", "--dry-run", "2", "hello" ] );
  assert_eq!( exit_code( &out ), 1 );
  let err = stderr_str( &out );
  assert!( err.contains( "'hello'" ), "stderr: {err}" );
  assert!( err.contains( "--message" ), "the suggestion must name the flag that takes prose: {err}" );
}

// ─── the seed prompt ────────────────────────────────────────────────────────

// pl16: the default exists so the common case needs no message at all
#[ test ]
fn pl16_seed_message_defaults_and_overrides()
{
  let base = TopicBase::new();

  let default = base.run( &[ "pool", "--dry-run", "--count", "1" ] );
  assert!
  (
    cmd_lines( &default )[ 0 ].contains( "--message \"ready\"" ),
    "got {}", cmd_lines( &default )[ 0 ],
  );

  let custom = base.run( &[ "pool", "--dry-run", "--message", "stand by", "--count", "1" ] );
  assert!
  (
    cmd_lines( &custom )[ 0 ].contains( "--message \"stand by\"" ),
    "got {}", cmd_lines( &custom )[ 0 ],
  );
}

// ─── concurrency ────────────────────────────────────────────────────────────

// pl17: the bound is a token-spend rate, so it is reported as the value actually
// in force rather than as the value asked for
#[ test ]
fn pl17_concurrency_is_echoed_and_clamped()
{
  let base = TopicBase::new();

  let explicit = base.run( &[ "pool", "--dry-run", "-j", "1", "--count", "3" ] );
  assert_eq!( field( &explicit, "concurrency" ).as_deref(), Some( "1" ) );

  let clamped = base.run( &[ "pool", "--dry-run", "--concurrency", "50", "--count", "2" ] );
  assert_eq!
  (
    field( &clamped, "concurrency" ).as_deref(),
    Some( "2" ),
    "50 workers for 2 topics is 2 workers",
  );
}

// ─── dispatch ───────────────────────────────────────────────────────────────

// pl18: reachable, self-documenting, and discoverable from the top-level help
#[ test ]
fn pl18_pool_is_dispatched_and_documented()
{
  let own = run_cli( &[ "pool", "--help" ] );
  assert_eq!( exit_code( &own ), 0 );
  let own_text = stdout_str( &own );
  assert!( own_text.contains( "clr pool" ), "own help: {own_text}" );
  assert!( own_text.contains( "--prefix" ), "own help must document --prefix: {own_text}" );

  let top = run_cli( &[ "help" ] );
  assert!( stdout_str( &top ).contains( "pool" ), "pool missing from `clr help`" );
}

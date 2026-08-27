//! Bounded fan-out tests — concurrency bound, ordering, and failure isolation.
//!
//! # Test Matrix
//!
//! | TC | Description | P/N |
//! |----|-------------|-----|
//! | tfo01 | An empty batch yields an empty result and starts nothing | P |
//! | tfo02 | Every command gets exactly one outcome, labelled as given | P |
//! | tfo03 | Results come back in input order, not completion order | P |
//! | tfo04 | stdout and stderr are captured per child, not merged | P |
//! | tfo05 | A non-zero child exit is reported, not swallowed | N |
//! | tfo06 | A command that cannot start yields `SPAWN_FAILED_EXIT_CODE` | N |
//! | tfo07 | One failing child does not stop its siblings | N |
//! | tfo08 | `concurrency` is a real ceiling on simultaneous children | P |
//! | tfo09 | `concurrency = 0` runs the batch serially rather than hanging | P |
//! | tfo10 | `concurrency` above the batch size behaves like batch size | P |
//! | tfo11 | A child writing more than one pipe buffer does not deadlock | P |
//! | tfo12 | `is_success` is true only for a child that ran and exited zero | P |

use claude_runner_core::fanout::{ run_bounded, FanoutOutcome, SPAWN_FAILED_EXIT_CODE };

/// Build a labelled `/bin/sh -c` job — the whole test surface is "what did this
/// child do", so the child is a shell one-liner rather than a fixture binary.
fn job( label : &str, script : &str ) -> ( String, std::process::Command )
{
  let mut command = std::process::Command::new( "/bin/sh" );
  command.arg( "-c" ).arg( script );
  ( label.to_string(), command )
}

/// The labels of a result set, in the order they came back.
fn labels( outcomes : &[ FanoutOutcome ] ) -> Vec< &str >
{
  outcomes.iter().map( | o | o.label.as_str() ).collect()
}

// tfo01: an empty batch is a no-op, not a hang and not a panic
#[ test ]
fn tfo01_empty_batch_yields_empty_result()
{
  let outcomes = run_bounded( Vec::new(), 4 );
  assert!( outcomes.is_empty(), "an empty batch must produce no outcomes" );
}

// tfo02: N commands in, N outcomes out, labels echoed unchanged
#[ test ]
fn tfo02_one_outcome_per_command_with_label_echoed()
{
  let outcomes = run_bounded
  (
    vec![ job( "alpha", "true" ), job( "beta", "true" ), job( "gamma", "true" ) ],
    2,
  );
  assert_eq!( outcomes.len(), 3, "every command must produce exactly one outcome" );
  assert_eq!( labels( &outcomes ), vec![ "alpha", "beta", "gamma" ] );
}

// tfo03: the first command listed is the first result even when it finishes last
#[ test ]
fn tfo03_results_are_in_input_order_not_completion_order()
{
  // `slow` is listed first and deliberately finishes last. With completion
  // ordering this comes back as fast/medium/slow; the contract says otherwise.
  let outcomes = run_bounded
  (
    vec!
    [
      job( "slow",   "sleep 0.30" ),
      job( "medium", "sleep 0.15" ),
      job( "fast",   "true" ),
    ],
    3,
  );
  assert_eq!
  (
    labels( &outcomes ),
    vec![ "slow", "medium", "fast" ],
    "outcomes must follow input order regardless of which child finished first",
  );
}

// tfo04: the two streams stay separate, and belong to the child that wrote them
#[ test ]
fn tfo04_stdout_and_stderr_are_captured_separately_per_child()
{
  let outcomes = run_bounded
  (
    vec!
    [
      job( "one", "echo out-one; echo err-one >&2" ),
      job( "two", "echo out-two; echo err-two >&2" ),
    ],
    2,
  );
  assert_eq!( outcomes[ 0 ].stdout.trim(), "out-one" );
  assert_eq!( outcomes[ 0 ].stderr.trim(), "err-one" );
  assert_eq!( outcomes[ 1 ].stdout.trim(), "out-two" );
  assert_eq!( outcomes[ 1 ].stderr.trim(), "err-two" );
}

// tfo05: a child's own non-zero status reaches the caller intact
#[ test ]
fn tfo05_non_zero_child_exit_is_reported()
{
  let outcomes = run_bounded( vec![ job( "failing", "exit 7" ) ], 1 );
  assert_eq!( outcomes[ 0 ].exit_code, 7, "the child's own exit code must survive" );
  assert!( !outcomes[ 0 ].is_success() );
}

// tfo06: a spawn failure is an outcome, distinguishable from any real exit
#[ test ]
fn tfo06_unspawnable_command_yields_spawn_failed_code()
{
  let mut command = std::process::Command::new( "/nonexistent/definitely-not-a-binary" );
  command.arg( "ignored" );
  let outcomes = run_bounded( vec![ ( "missing".to_string(), command ) ], 1 );

  assert_eq!( outcomes.len(), 1, "a command that never starts still owes an outcome" );
  assert_eq!( outcomes[ 0 ].exit_code, SPAWN_FAILED_EXIT_CODE );
  assert!
  (
    outcomes[ 0 ].stderr.contains( "cannot start command" ),
    "the spawn error belongs in stderr, got: {}",
    outcomes[ 0 ].stderr,
  );
  assert!( outcomes[ 0 ].stdout.is_empty(), "a process that never ran wrote no stdout" );
}

// tfo07: failure isolation — the batch is not a transaction
#[ test ]
fn tfo07_one_failure_does_not_cancel_siblings()
{
  let mut missing = std::process::Command::new( "/nonexistent/definitely-not-a-binary" );
  missing.arg( "ignored" );

  let outcomes = run_bounded
  (
    vec!
    [
      job( "before", "echo before-ran" ),
      ( "missing".to_string(), missing ),
      job( "exits-nonzero", "exit 3" ),
      job( "after", "echo after-ran" ),
    ],
    1, // serial, so a cancel-on-failure bug would visibly truncate the tail
  );

  assert_eq!( outcomes.len(), 4 );
  assert_eq!( outcomes[ 0 ].stdout.trim(), "before-ran" );
  assert_eq!( outcomes[ 1 ].exit_code, SPAWN_FAILED_EXIT_CODE );
  assert_eq!( outcomes[ 2 ].exit_code, 3 );
  assert_eq!
  (
    outcomes[ 3 ].stdout.trim(),
    "after-ran",
    "a command listed after two failures must still run",
  );
}

// tfo08: the bound actually bounds — the load never exceeds `concurrency`
#[ test ]
fn tfo08_concurrency_is_a_real_ceiling()
{
  let dir = tempfile::tempdir().expect( "tempdir" );
  let counter = dir.path().join( "live" );
  let peak = dir.path().join( "peak" );

  // Each child appends a byte on entry and truncates by one on exit, so the file
  // length is the number of children currently inside the critical section. The
  // high-water mark is recorded by hand because the bound is about the maximum,
  // not the average — an implementation that briefly overshoots is still broken.
  //
  // `flock` serializes the read-modify-write; without it this measures the test's
  // own race rather than the fan-out's bound.
  let script = format!
  (
    r#"
    live='{live}'; peak='{peak}'
    bump()
    {{
      flock 9
      printf x >> "$live"
      n=$( wc -c < "$live" )
      p=$( cat "$peak" 2>/dev/null || echo 0 )
      [ "$n" -gt "$p" ] && printf '%s' "$n" > "$peak"
    }}
    drop()
    {{
      flock 9
      n=$( wc -c < "$live" )
      : > "$live"
      i=1
      while [ "$i" -lt "$n" ]; do printf x >> "$live"; i=$(( i + 1 )); done
    }}
    exec 9>"$live.lock"
    bump
    exec 9>&-
    sleep 0.20
    exec 9>"$live.lock"
    drop
    exec 9>&-
    "#,
    live = counter.display(),
    peak = peak.display(),
  );

  std::fs::write( &counter, "" ).expect( "seed counter" );
  let commands : Vec< _ > = ( 0..8 ).map( | i | job( &format!( "j{i}" ), &script ) ).collect();
  let outcomes = run_bounded( commands, 3 );

  assert_eq!( outcomes.len(), 8 );
  let observed : usize = std::fs::read_to_string( &peak )
    .expect( "peak file written" )
    .trim()
    .parse()
    .expect( "peak is a number" );
  assert!
  (
    observed <= 3,
    "at most 3 children may be in flight at once, observed {observed}",
  );
  assert!
  (
    observed > 1,
    "with 8 jobs and a bound of 3 the workers must actually overlap, observed {observed}",
  );
}

// tfo09: 0 is clamped up to 1 — "no workers" would mean "never finishes"
#[ test ]
fn tfo09_zero_concurrency_runs_serially_rather_than_hanging()
{
  let outcomes = run_bounded
  (
    vec![ job( "a", "echo a" ), job( "b", "echo b" ) ],
    0,
  );
  assert_eq!( labels( &outcomes ), vec![ "a", "b" ] );
  assert!( outcomes.iter().all( FanoutOutcome::is_success ) );
}

// tfo10: a bound larger than the batch is harmless
#[ test ]
fn tfo10_concurrency_above_batch_size_is_clamped()
{
  let outcomes = run_bounded( vec![ job( "only", "echo only" ) ], 64 );
  assert_eq!( outcomes.len(), 1 );
  assert_eq!( outcomes[ 0 ].stdout.trim(), "only" );
}

// tfo11: the classic pipe-buffer deadlock — a child writing past 64 KiB on both
// streams while the parent waits. `output()` drains concurrently; a naive
// wait-then-read hangs here forever.
#[ test ]
fn tfo11_large_output_does_not_deadlock()
{
  let outcomes = run_bounded
  (
    vec![ job( "loud", "yes chatter | head -c 200000; yes noise | head -c 200000 >&2" ) ],
    1,
  );
  assert_eq!( outcomes.len(), 1 );
  assert_eq!( outcomes[ 0 ].stdout.len(), 200_000, "stdout must be captured whole" );
  assert_eq!( outcomes[ 0 ].stderr.len(), 200_000, "stderr must be captured whole" );
}

// tfo12: success means ran-and-exited-zero, and a spawn failure is not success
#[ test ]
fn tfo12_is_success_covers_only_zero_exit()
{
  let mut missing = std::process::Command::new( "/nonexistent/definitely-not-a-binary" );
  missing.arg( "ignored" );
  let outcomes = run_bounded
  (
    vec![ job( "ok", "true" ), job( "bad", "exit 1" ), ( "gone".to_string(), missing ) ],
    3,
  );
  assert!( outcomes[ 0 ].is_success(), "exit 0 is success" );
  assert!( !outcomes[ 1 ].is_success(), "a non-zero exit is not success" );
  assert!( !outcomes[ 2 ].is_success(), "never starting is not success" );
}

//! `clr pool` — make sure N anonymous topics exist under a base.
//!
//! The third member of the fan-out family. `delegate` sends a prompt to one topic
//! and `broadcast` to every topic; `pool` is what you run first, when the topics
//! they need do not exist yet.
//!
//! ## Anonymous, on purpose
//!
//! `clr topic <message>` names a topic after the message that opened it —
//! descriptive, disambiguated by a counter, and meaningful to read back. That is
//! the right name when the topic is *about* something.
//!
//! A pool topic is not about anything: it is somewhere for work to go. `t1`, `t2`,
//! `t3` say exactly that and nothing more. Naming them after their first message
//! would be actively misleading, since the second message is unlikely to be about
//! the same thing. The naming rules — which names count, how gaps are filled, why
//! a prefix may not end in a digit — belong to `claude_topic_core::pool` and are
//! documented there.
//!
//! ## Idempotence is the whole point
//!
//! `--count` is a target, never an increment: "make sure four exist", not "add
//! four more". Running it twice creates nothing the second time, which is what
//! makes it usable from a script that may run twice — and the second run is the
//! one nobody is watching.
//!
//! ## Why the live set, not the full set
//!
//! The target is counted against `enumerate_live`, so a pool name whose session
//! was deleted counts as *missing* and gets refilled. Counting against the full
//! set instead would let `clr pool --count 4` report success while
//! `clr broadcast` reached only three of them — a partial fan-out that looks
//! complete, which is the failure mode the whole family is built to avoid.
//!
//! ## Creating a topic means running one
//!
//! There is no way to make a topic exist without a session in it, and no way to
//! make a session without invoking Claude Code. So each missing name gets a
//! print-mode `clr run` child carrying a deliberately trivial seed message, the
//! same transport `broadcast` uses and for the same reason (see
//! [`super::forward`]'s module docs). This costs one real turn per topic created —
//! `--dry-run` shows the whole plan for free.

use super::forward::{ child_command, claim_locks, describe_child, self_exe, value_after, DEFAULT_CONCURRENCY };
use super::help::print_pool_help;
use claude_runner_core::fanout::run_bounded;
use claude_topic_core::{ enumerate_live, fork_session_file, topic_base, topic_dir, Topic, TopicMode };

/// Seed prompt sent to each newly created topic.
///
/// Deliberately trivial. Its only job is to make the session exist; the topic's
/// first real instruction arrives later through `delegate` or `broadcast`. A long
/// seed prompt would be paid for once per topic and then be irrelevant to every
/// turn after it.
const DEFAULT_SEED_MESSAGE : &str = "ready";

/// Parsed `pool` flags.
struct PoolArgs
{
  dir : Option< String >,
  global : bool,
  dry_run : bool,
  count : usize,
  prefix : String,
  mode : TopicMode,
  concurrency : usize,
  message : String,
}

/// Parse the `pool` token stream; prints help or an error and exits on
/// `help`/`--help`, unknown options, missing values, an unparsable number, an
/// unusable prefix, a second positional count, or a missing count.
fn parse_pool_args( tokens : &[ String ] ) -> PoolArgs
{
  // tokens[0] == "pool"
  // Bare positional `help` prints help rather than being read as the count — the
  // same intercept every subcommand dispatcher repeats (BUG-249 pattern).
  if tokens.get( 1 ).map( String::as_str ) == Some( "help" )
  {
    print_pool_help();
  }

  let mut dir : Option< String > = None;
  let mut global = false;
  let mut dry_run = false;
  let mut count : Option< usize > = None;
  let mut prefix = claude_topic_core::DEFAULT_PREFIX.to_owned();
  let mut mode = TopicMode::Fork;
  let mut concurrency = DEFAULT_CONCURRENCY;
  let mut message : Option< String > = None;
  let mut i = 1_usize;

  while i < tokens.len()
  {
    match tokens[ i ].as_str()
    {
      "--help" | "-h" => print_pool_help(),
      "--global" | "-g" => { global = true; i += 1; }
      "--dry-run" | "-n" => { dry_run = true; i += 1; }
      "--dir" | "--to" =>
      {
        dir = Some( value_after( tokens, i, &tokens[ i ], "pool" ) );
        i += 2;
      }
      // No short form: `-c` is `--continue` everywhere else in this CLI, and a
      // flag that means two different things depending on the subcommand is worse
      // than a flag with no abbreviation.
      "--count" =>
      {
        count = Some( parse_count( &value_after( tokens, i, "--count", "pool" ) ) );
        i += 2;
      }
      "--prefix" =>
      {
        prefix = value_after( tokens, i, "--prefix", "pool" );
        i += 2;
      }
      "--topic-mode" =>
      {
        mode = parse_mode( &value_after( tokens, i, "--topic-mode", "pool" ) );
        i += 2;
      }
      "--concurrency" | "-j" =>
      {
        concurrency = parse_concurrency( &value_after( tokens, i, &tokens[ i ], "pool" ) );
        i += 2;
      }
      "--message" =>
      {
        message = Some( value_after( tokens, i, "--message", "pool" ) );
        i += 2;
      }
      other if other.starts_with( '-' ) && other.len() > 1 =>
      {
        eprintln!( "Error: unknown option '{other}'\nRun `clr pool --help` for usage." );
        std::process::exit( 1 );
      }
      // The one positional is the count. A second one is rejected rather than
      // joined into a message: `pool` takes a number, and silently treating a
      // stray word as prose would hide the typo that produced it.
      other =>
      {
        if count.is_some()
        {
          eprintln!
          (
            "Error: unexpected argument '{other}' — `clr pool` takes one count\n\
             Did you mean --message \"{other}\"?"
          );
          std::process::exit( 1 );
        }
        count = Some( parse_count( other ) );
        i += 1;
      }
    }
  }

  let Some( count ) = count else
  {
    eprintln!( "Error: pool requires a count\nUsage: clr pool [OPTIONS] <N>\nRun `clr pool --help` for usage." );
    std::process::exit( 1 );
  };
  check_prefix( &prefix );

  PoolArgs
  {
    dir,
    global,
    dry_run,
    count,
    prefix,
    mode,
    concurrency,
    message : message.unwrap_or_else( || DEFAULT_SEED_MESSAGE.to_owned() ),
  }
}

/// Read a topic mechanism, or exit with a diagnostic naming both valid values.
fn parse_mode( raw : &str ) -> TopicMode
{
  let Ok( parsed ) = raw.parse::< TopicMode >() else
  {
    eprintln!( "Error: invalid --topic-mode value '{raw}'\nExpected: fork or dir" );
    std::process::exit( 1 );
  };
  parsed
}

/// Read a worker count, or exit with a diagnostic quoting what was given.
fn parse_concurrency( raw : &str ) -> usize
{
  let Ok( parsed ) = raw.parse::< usize >() else
  {
    eprintln!( "Error: --concurrency must be a positive integer, got '{raw}'" );
    std::process::exit( 1 );
  };
  parsed
}

/// Reject a prefix that cannot produce usable topic names, quoting the reason.
///
/// The rules themselves live in `claude_topic_core::pool` — they are properties of
/// the name-to-index mapping, not of this CLI.
fn check_prefix( prefix : &str )
{
  if let Err( reason ) = claude_topic_core::validate_prefix( prefix )
  {
    eprintln!( "Error: invalid --prefix '{prefix}': {reason}" );
    std::process::exit( 1 );
  }
}

/// Read a count, or exit with a diagnostic quoting what was given.
///
/// Zero is accepted as a no-op rather than rejected: `clr pool "$N"` from a script
/// that computed `N == 0` has asked for nothing, and failing there would make the
/// caller special-case a case that already means "do nothing".
fn parse_count( raw : &str ) -> usize
{
  let Ok( parsed ) = raw.parse::< usize >() else
  {
    eprintln!( "Error: count must be a non-negative integer, got '{raw}'" );
    std::process::exit( 1 );
  };
  parsed
}

/// Build the topic a name will become once its child has run.
///
/// `Topic::path` is a computed path in both mechanisms, so it is meaningful before
/// the topic exists — which is what lets a not-yet-created pool topic go through
/// exactly the same lock, spawn, and describe path as an existing one.
fn planned_topic( base : &std::path::Path, name : String, mode : TopicMode ) -> Option< Topic >
{
  let path = match mode
  {
    TopicMode::Dir  => topic_dir( base, &name ),
    TopicMode::Fork => fork_session_file( base, &name )?,
  };
  Some( Topic { name, mode, path, sessions : 0 } )
}

/// Parse, validate, and execute the `pool` subcommand. Never returns.
///
/// Exits 0 when every missing name was created, or when none were missing. One
/// failing child fails the command, for the same reason it does in `broadcast`: a
/// pool reported as full but holding three of four topics is worse than one
/// reported as broken.
pub( crate ) fn dispatch_pool( tokens : &[ String ] ) -> !
{
  let args = parse_pool_args( tokens );
  let base = topic_base( args.dir.as_deref(), args.global );
  let target = args.count;

  // The live set, not the full set — see the module docs.
  let existing = enumerate_live( &base );
  let missing = claude_topic_core::missing_names( &existing, target, &args.prefix );
  let held = existing
    .iter()
    .filter( | t | claude_topic_core::pool_index( &t.name, &args.prefix ).is_some() )
    .count();

  let mut planned = Vec::with_capacity( missing.len() );
  for name in missing
  {
    let Some( topic ) = planned_topic( &base, name.clone(), args.mode ) else
    {
      eprintln!( "Error: cannot resolve session storage for topic '{name}' (is HOME set?)" );
      std::process::exit( 1 );
    };
    planned.push( topic );
  }

  if args.dry_run
  {
    println!( "base: {}", base.display() );
    println!( "prefix: {}", args.prefix );
    println!( "mode: {}", args.mode.as_str() );
    println!( "target: {target}" );
    println!( "existing: {held}" );
    println!( "create: {}", planned.len() );
    println!( "concurrency: {}", args.concurrency.clamp( 1, planned.len().max( 1 ) ) );
    for topic in &planned
    {
      println!( "cmd: {}", describe_child( &base, topic, &args.message ) );
    }
    std::process::exit( 0 );
  }

  if planned.is_empty()
  {
    eprintln!
    (
      "[Runner] {} already holds {held} topic(s) with prefix '{}' — nothing to create",
      base.display(),
      args.prefix
    );
    std::process::exit( 0 );
  }

  let ( targets, _guards ) = claim_locks( planned );
  if targets.is_empty()
  {
    eprintln!( "Error: every pool topic to create in {} is held by another run", base.display() );
    std::process::exit( 1 );
  }

  let exe = self_exe();
  let jobs : Vec< _ > = targets
    .iter()
    .map( | t | ( t.name.clone(), child_command( &exe, &base, t, &args.message ) ) )
    .collect();

  eprintln!
  (
    "[Runner] creating {} pool topic(s) under {} — each is a full Claude Code session, {} at a time",
    jobs.len(),
    base.display(),
    args.concurrency.clamp( 1, jobs.len() )
  );
  let outcomes = run_bounded( jobs, args.concurrency );

  // Successes are reported by name only. The seed answer is throwaway by
  // construction, so printing it would bury the one thing worth reading — which
  // names now exist. A failure's stderr is relayed, because that is not throwaway.
  let mut failed = 0_usize;
  for ( outcome, topic ) in outcomes.iter().zip( targets.iter() )
  {
    if outcome.is_success()
    {
      println!( "created: {} ({})", topic.name, topic.mode.as_str() );
    }
    else
    {
      failed += 1;
      println!( "failed: {} ({}) — exit {}", topic.name, topic.mode.as_str(), outcome.exit_code );
      eprint!( "{}", outcome.stderr );
    }
  }

  if failed == 0
  {
    std::process::exit( 0 );
  }
  eprintln!( "[Runner] {failed} of {} topic(s) could not be created", outcomes.len() );
  std::process::exit( 1 );
}

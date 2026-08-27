//! `clr delegate` and `clr broadcast` — send one prompt somewhere other than here.
//!
//! Both commands answer the same question — *which topics, and then what* — and
//! differ only in the answer's size: `delegate` picks exactly one topic,
//! `broadcast` takes every live one. Everything after the pick is shared, which is
//! why they live in one module and share one argument parser.
//!
//! ## Why child processes rather than the daemon
//!
//! The obvious transport is `clr chat`: the daemon already hosts sessions and
//! already round-trips a prompt. It cannot be used here. `Request::Spawn` starts a
//! session in a directory — it has no resume-by-session-id form — so a fork-mode
//! topic, whose whole identity *is* a session id in the base's own storage, is not
//! something the daemon can host at all. Delegating only to dir-mode topics would
//! silently skip most of what `clr topics` lists.
//!
//! So each target gets a print-mode `clr run` child, spawned from
//! `current_exe()` — same binary, therefore same topic-resolution rules, with no
//! version skew possible between the fan-out and the thing it fans out to.
//!
//! ## The mode always travels with the name
//!
//! Every child is given `--topic NAME --topic-mode MODE`, never `--topic` alone.
//! A bare name is not a topic (`claude_topic_core`'s
//! `invariant/002_mode_travels_with_name.md`): when one name is held in both
//! mechanisms, `effective_topic_mode`'s rule 4 sends a bare `--topic` to the
//! dir-mode one every time, and the fork-mode topic in the same list is silently
//! never reached — a fan-out that looks completely successful while missing half
//! its targets.
//!
//! ## Live topics only
//!
//! Both commands enumerate with `enumerate_live`, so a topic with no sessions is
//! not a target. Addressing one would *create* a conversation by forking the base,
//! and "send this to my topics" is not a request to mint new ones. It also keeps
//! the fan-out out of `-daemon/`, `-gate/`, and every `./-NNNN_*` scratch
//! directory, which look exactly like dir-mode topics from the base's point of
//! view and have no session storage.

use super::help::{ print_broadcast_help, print_delegate_help };
use claude_runner_core::fanout::{ run_bounded, FanoutOutcome };
use claude_topic_core::{ enumerate_live, topic_base, Pick, Topic };

/// Which of the two commands is being parsed and run.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
enum Forward
{
  /// One topic, chosen by policy.
  Delegate,
  /// Every live topic.
  Broadcast,
}

impl Forward
{
  /// The subcommand name as typed — used in every diagnostic so an error names
  /// the command the user actually ran.
  const fn as_str( self ) -> &'static str
  {
    match self
    {
      Forward::Delegate  => "delegate",
      Forward::Broadcast => "broadcast",
    }
  }
}

/// Default number of children in flight for `broadcast`.
///
/// Four rather than "all of them": every child is a full Claude Code session, so
/// the batch size is a token-spend rate and a rate-limit exposure, not just a
/// scheduling detail. Four is enough for the parallelism to be the point and low
/// enough that a twenty-topic base does not hit the API twenty-wide.
///
/// Shared with `clr pool` ([`super::pool`]), whose children are the same kind of
/// thing for the same reason.
pub( super ) const DEFAULT_CONCURRENCY : usize = 4;

/// Parsed flags common to both commands, plus each one's own extras.
struct ForwardArgs
{
  dir : Option< String >,
  global : bool,
  dry_run : bool,
  pick : Pick,
  seed : Option< u64 >,
  concurrency : usize,
  message : String,
}

/// Read the value following `tokens[ i ]`, or exit with a diagnostic naming the flag.
pub( super ) fn value_after( tokens : &[ String ], i : usize, flag : &str, command : &str ) -> String
{
  let Some( val ) = tokens.get( i + 1 ) else
  {
    eprintln!( "Error: {flag} requires a value\nRun `clr {command} --help` for usage." );
    std::process::exit( 1 );
  };
  val.clone()
}

/// Parse the token stream for either command; prints help or an error and exits
/// on `help`/`--help`, unknown options, missing values, an unparsable number, a
/// flag belonging to the other command, or a missing message.
fn parse_forward_args( tokens : &[ String ], command : Forward ) -> ForwardArgs
{
  // tokens[0] == "delegate" | "broadcast"
  // Bare positional `help` prints help rather than becoming the message — the same
  // intercept every subcommand dispatcher repeats (BUG-249 pattern).
  if tokens.get( 1 ).map( String::as_str ) == Some( "help" )
  {
    print_forward_help( command );
  }

  let mut dir : Option< String > = None;
  let mut global = false;
  let mut dry_run = false;
  let mut pick = Pick::default();
  let mut seed : Option< u64 > = None;
  let mut concurrency = DEFAULT_CONCURRENCY;
  let mut positional : Vec< String > = Vec::new();
  let mut i = 1_usize;

  while i < tokens.len()
  {
    match tokens[ i ].as_str()
    {
      "--help" | "-h" => print_forward_help( command ),
      "--global" | "-g" => { global = true; i += 1; }
      "--dry-run" | "-n" => { dry_run = true; i += 1; }
      "--dir" | "--to" =>
      {
        dir = Some( value_after( tokens, i, &tokens[ i ], command.as_str() ) );
        i += 2;
      }
      "--pick" =>
      {
        // `--pick` on broadcast would be meaningless rather than merely unused:
        // broadcast has no choice to make, so accepting it would imply one.
        reject_wrong_command( command, Forward::Delegate, "--pick" );
        let val = value_after( tokens, i, "--pick", command.as_str() );
        let Ok( parsed ) = val.parse::< Pick >() else
        {
          eprintln!( "Error: invalid --pick value '{val}'\nExpected: idle or random" );
          std::process::exit( 1 );
        };
        pick = parsed;
        i += 2;
      }
      "--seed" =>
      {
        reject_wrong_command( command, Forward::Delegate, "--seed" );
        let val = value_after( tokens, i, "--seed", command.as_str() );
        let Ok( parsed ) = val.parse::< u64 >() else
        {
          eprintln!( "Error: --seed must be a non-negative integer, got '{val}'" );
          std::process::exit( 1 );
        };
        seed = Some( parsed );
        i += 2;
      }
      "--concurrency" | "-j" =>
      {
        reject_wrong_command( command, Forward::Broadcast, &tokens[ i ] );
        let val = value_after( tokens, i, &tokens[ i ], command.as_str() );
        let Ok( parsed ) = val.parse::< usize >() else
        {
          eprintln!( "Error: --concurrency must be a positive integer, got '{val}'" );
          std::process::exit( 1 );
        };
        concurrency = parsed;
        i += 2;
      }
      "--message" =>
      {
        positional.push( value_after( tokens, i, "--message", command.as_str() ) );
        i += 2;
      }
      // Everything after `--` is message text, even if it starts with a hyphen.
      "--" =>
      {
        positional.extend( tokens[ i + 1.. ].iter().cloned() );
        break;
      }
      other if other.starts_with( '-' ) && other.len() > 1 =>
      {
        eprintln!
        (
          "Error: unknown option '{other}'\nRun `clr {} --help` for usage.",
          command.as_str()
        );
        std::process::exit( 1 );
      }
      other => { positional.push( other.to_string() ); i += 1; }
    }
  }

  let message = positional.join( " " );
  if message.trim().is_empty()
  {
    eprintln!
    (
      "Error: {} requires a message\nRun `clr {} --help` for usage.",
      command.as_str(),
      command.as_str()
    );
    std::process::exit( 1 );
  }

  ForwardArgs { dir, global, dry_run, pick, seed, concurrency, message }
}

/// Exit with a diagnostic when a flag belonging to `owner` is used on the other command.
fn reject_wrong_command( actual : Forward, owner : Forward, flag : &str )
{
  if actual != owner
  {
    eprintln!
    (
      "Error: {flag} belongs to `clr {}`, not `clr {}`\nRun `clr {} --help` for usage.",
      owner.as_str(),
      actual.as_str(),
      actual.as_str()
    );
    std::process::exit( 1 );
  }
}

/// Print the help for whichever command is running. Never returns.
fn print_forward_help( command : Forward ) -> !
{
  match command
  {
    Forward::Delegate  => print_delegate_help(),
    Forward::Broadcast => print_broadcast_help(),
  }
}

/// Build the `clr run` child that sends `message` to `topic` under `base`.
///
/// `--dir` carries the resolved base rather than re-deriving it in the child from
/// `--global` or the inherited cwd: an explicit `--dir` outranks both, so the
/// child lands on exactly the base the parent enumerated, whatever the child's own
/// environment says. `--topic-mode` is always passed — see the module docs.
///
/// `topic` need not exist yet: `clr pool` builds one for each name it is about to
/// create, and the command that creates a topic is the same command that continues
/// one.
pub( super ) fn child_command
(
  exe : &std::path::Path,
  base : &std::path::Path,
  topic : &Topic,
  message : &str,
) -> std::process::Command
{
  let mut command = std::process::Command::new( exe );
  command
    .arg( "run" )
    .arg( "--dir" ).arg( base )
    .arg( "--topic" ).arg( &topic.name )
    .arg( "--topic-mode" ).arg( topic.mode.as_str() )
    .arg( "--message" ).arg( message );
  command
}

/// Render the `clr run` invocation for a topic as a single readable line, for
/// `--dry-run`. Not shell-quoted for re-execution — it is a description of what
/// would run, and the message is shown as one argument because that is how it is
/// passed.
pub( super ) fn describe_child( base : &std::path::Path, topic : &Topic, message : &str ) -> String
{
  format!
  (
    "clr run --dir {} --topic {} --topic-mode {} --message {message:?}",
    base.display(),
    topic.name,
    topic.mode.as_str(),
  )
}

/// Resolve this binary's own path, or exit — every child is a copy of it.
pub( super ) fn self_exe() -> std::path::PathBuf
{
  match std::env::current_exe()
  {
    Ok( p ) => p,
    Err( e ) =>
    {
      eprintln!( "Error: cannot resolve the running clr binary: {e}" );
      std::process::exit( 1 );
    }
  }
}

/// Enumerate the live topics under `base`, or exit 1 with a note when there are none.
///
/// An empty base is an error here, unlike in `clr topics` where it is an ordinary
/// listing of nothing: a forward with no target did not do what was asked, and
/// exiting 0 would report success for a prompt that went nowhere.
fn live_topics_or_exit( base : &std::path::Path, command : Forward ) -> Vec< Topic >
{
  let topics = enumerate_live( base );
  if topics.is_empty()
  {
    eprintln!
    (
      "Error: no live topics in {}\n\
       `clr {}` only targets topics that already hold a session — run `clr topics`\n\
       to see what exists, or `clr topic <message>` to start one.",
      base.display(),
      command.as_str()
    );
    std::process::exit( 1 );
  }
  topics
}

/// Take the run-path lock for each topic when `CLR_TOPIC_LOCK` is on, returning
/// the topics that were successfully claimed together with the guards holding
/// them.
///
/// The parent holds every lock for the whole batch rather than each child taking
/// its own, because the parent is what knows the target set — a child cannot
/// decline a topic that a sibling in the same batch already took. Guards live
/// until the returned value is dropped, which the caller arranges to be after
/// `run_bounded` returns.
///
/// A topic already held elsewhere is dropped from the batch with a note, not
/// waited on: `try_lock` never blocks, and a fan-out that stalls on one busy
/// topic has become a serial run with extra steps.
pub( super ) fn claim_locks( topics : Vec< Topic > ) -> ( Vec< Topic >, Vec< claude_topic_core::TopicLock > )
{
  if !claude_topic_core::enabled_for_run_path()
  {
    return ( topics, Vec::new() );
  }

  let mut claimed = Vec::with_capacity( topics.len() );
  let mut guards = Vec::with_capacity( topics.len() );
  for topic in topics
  {
    match claude_topic_core::try_lock( &topic )
    {
      Ok( guard ) => { guards.push( guard ); claimed.push( topic ); }
      Err( claude_topic_core::LockDenied::Held( pid ) ) =>
      {
        eprintln!
        (
          "[Runner] skipping topic '{}' ({}) — held by pid {pid}",
          topic.name,
          topic.mode.as_str()
        );
      }
      Err( claude_topic_core::LockDenied::Unavailable( reason ) ) =>
      {
        // The lock is advisory; an unusable lock directory must not turn a
        // working fan-out into a failing one.
        eprintln!
        (
          "[Runner] warning: cannot lock topic '{}': {reason} — proceeding unlocked",
          topic.name
        );
        claimed.push( topic );
      }
    }
  }
  ( claimed, guards )
}

/// Parse, validate, and execute `clr delegate`. Never returns.
///
/// Exits with the chosen child's own exit code, so a delegated failure is
/// indistinguishable from running the same prompt here — which is the point of
/// delegating rather than reporting on it.
pub( crate ) fn dispatch_delegate( tokens : &[ String ] ) -> !
{
  let args = parse_forward_args( tokens, Forward::Delegate );
  let base = topic_base( args.dir.as_deref(), args.global );
  let topics = live_topics_or_exit( &base, Forward::Delegate );

  let seed = args.seed.unwrap_or_else( claude_topic_core::default_seed );
  let Some( selection ) = claude_topic_core::select( &topics, args.pick, seed ) else
  {
    // Unreachable in practice — live_topics_or_exit already rejected the empty
    // case, and that is the only condition under which select returns None.
    eprintln!( "Error: no topic could be selected in {}", base.display() );
    std::process::exit( 1 );
  };
  let topic = selection.topic;

  if selection.all_busy && args.pick == Pick::Idle
  {
    eprintln!
    (
      "[Runner] note: every topic is busy — falling back to the full set"
    );
  }

  if args.dry_run
  {
    println!( "base: {}", base.display() );
    println!( "pick: {}", args.pick.as_str() );
    println!( "seed: {seed}" );
    println!( "topic: {} ({})", topic.name, topic.mode.as_str() );
    println!( "cmd: {}", describe_child( &base, topic, &args.message ) );
    std::process::exit( 0 );
  }

  let ( claimed, _guards ) = claim_locks( vec![ topic.clone() ] );
  let Some( target ) = claimed.into_iter().next() else
  {
    eprintln!( "Error: topic '{}' is held by another run", topic.name );
    std::process::exit( 1 );
  };

  eprintln!( "[Runner] delegating to '{}' ({})", target.name, target.mode.as_str() );
  let jobs = vec!
  [
    ( target.name.clone(), child_command( &self_exe(), &base, &target, &args.message ) ),
  ];
  let outcomes = run_bounded( jobs, 1 );
  let Some( outcome ) = outcomes.into_iter().next() else
  {
    eprintln!( "Error: the delegated run produced no result" );
    std::process::exit( 1 );
  };

  print!( "{}", outcome.stdout );
  eprint!( "{}", outcome.stderr );
  std::process::exit( outcome.exit_code );
}

/// Parse, validate, and execute `clr broadcast`. Never returns.
///
/// Exits 0 only when every child exited 0. One failing topic is a failing
/// broadcast — the summary on stderr says which — because a partial fan-out that
/// reports success is indistinguishable from a complete one.
pub( crate ) fn dispatch_broadcast( tokens : &[ String ] ) -> !
{
  let args = parse_forward_args( tokens, Forward::Broadcast );
  let base = topic_base( args.dir.as_deref(), args.global );
  let topics = live_topics_or_exit( &base, Forward::Broadcast );

  if args.dry_run
  {
    println!( "base: {}", base.display() );
    println!( "topics: {}", topics.len() );
    println!( "concurrency: {}", args.concurrency.clamp( 1, topics.len() ) );
    for topic in &topics
    {
      println!( "cmd: {}", describe_child( &base, topic, &args.message ) );
    }
    std::process::exit( 0 );
  }

  let ( targets, _guards ) = claim_locks( topics );
  if targets.is_empty()
  {
    eprintln!( "Error: every topic in {} is held by another run", base.display() );
    std::process::exit( 1 );
  }

  let exe = self_exe();
  let jobs : Vec< _ > = targets
    .iter()
    .map( | t | ( t.name.clone(), child_command( &exe, &base, t, &args.message ) ) )
    .collect();

  eprintln!
  (
    "[Runner] broadcasting to {} topic(s), {} at a time",
    jobs.len(),
    args.concurrency.clamp( 1, jobs.len() )
  );
  let outcomes = run_bounded( jobs, args.concurrency );
  report( &outcomes, &targets );

  let failed = outcomes.iter().filter( | o | !o.is_success() ).count();
  if failed == 0
  {
    std::process::exit( 0 );
  }
  eprintln!( "[Runner] {failed} of {} topic(s) failed", outcomes.len() );
  std::process::exit( 1 );
}

/// Print one block per topic: a header naming the topic and its mode, then that
/// child's stdout.
///
/// The header is what makes the combined output readable at all — twenty answers
/// concatenated with no attribution is one answer to a question nobody asked.
/// `outcomes` and `targets` are in the same order because `run_bounded` preserves
/// input order, so they are zipped rather than looked up by name (two topics can
/// share a name across modes, so the name is not a key).
fn report( outcomes : &[ FanoutOutcome ], targets : &[ Topic ] )
{
  for ( outcome, topic ) in outcomes.iter().zip( targets.iter() )
  {
    println!( "──── {} · {} · exit {} ────", topic.name, topic.mode.as_str(), outcome.exit_code );
    print!( "{}", outcome.stdout );
    if !outcome.stdout.ends_with( '\n' ) && !outcome.stdout.is_empty()
    {
      println!();
    }
    if !outcome.stderr.is_empty()
    {
      eprint!( "{}", outcome.stderr );
    }
  }
}

mod parse;
mod env;
mod execution;
mod cred_parse;
mod builder;
mod fence;
mod credential;
mod help;
mod gate;
mod ps;
mod kill;

use claude_runner_core::{ ClaudeCommand, EffortLevel, IsolatedModel };
use parse::CliArgs;
use cred_parse::{
  parse_isolated_args, parse_refresh_args,
  apply_isolated_env_vars, apply_refresh_env_vars,
};
pub use fence::strip_fences;
use credential::{ run_isolated_command, run_refresh_command };

const CREDS_PATH_ERROR : &str =
  "Error: cannot resolve credentials path: HOME is not set; provide --creds or set CLR_CREDS\nRun with --help for usage.";
use help::print_ask_help;
use gate::wait_for_session_slot;
pub( super ) use ps::dispatch_ps;
pub( super ) use kill::dispatch_kill;

pub( super ) use parse::parse_args;
pub( super ) use env::apply_env_vars;
pub( super ) use builder::build_claude_command;
pub( super ) use help::print_help;

/// Handle dry-run mode: print command preview and exit.
///
/// Always emits output regardless of verbosity level. Verbosity controls runner
/// diagnostics only; `--dry-run` output is core functionality the user explicitly requested.
// Fix(BUG-228): always emit; verbosity must not suppress --dry-run output
// Root cause: prior version gated on shows_progress() (≥3); --verbosity 0–2 produced silent exit
// Pitfall: Verbosity gates runner diagnostics only, never core feature output like --dry-run
pub( super ) fn handle_dry_run( builder : &ClaudeCommand )
{
  let env = builder.describe_env();
  let command = builder.describe();
  if !env.is_empty() { println!( "{env}" ); }
  println!( "{command}" );
}

// Fix(BUG-212): `run` was absent; typing `clr running` produced no helpful error.
// Root cause: list was never updated when `run` became an explicit subcommand.
// Pitfall: update both this list and the dispatch match in lib.rs when adding a subcommand.
const KNOWN_SUBCOMMANDS : &[ &str ] = &[ "run", "ask", "isolated", "refresh", "help", "ps", "kill" ];

// Fix(BUG-225): Guard against typos/truncations of known subcommand names.
// Root cause: `run_cli()` dispatched subcommands by exact string match only — any
//   non-matching first token silently fell through to `parse_args()`.
// Pitfall: Bare string comparison only guards exact matches; typos pass silently
//   unless a prefix-match guard is also placed before the main argument parser.
pub( super ) fn guard_unknown_subcommand( tokens : &[ String ] )
{
  if let Some( first ) = tokens.first()
  {
    let is_identifier = !first.starts_with( '-' )
      && !first.is_empty()
      && first.chars().all( | c | c.is_alphanumeric() || c == '_' || c == '-' );
    if is_identifier
    {
      for &sub in KNOWN_SUBCOMMANDS
      {
        // Fix(BUG-250): extend guard to catch one-character insertion/substitution typos.
        // Root cause: prefix/superstring checks only caught truncations and extensions;
        //   mid-word insertions (e.g. "assk" for "ask") bypassed the guard and fell through
        //   to dispatch_run, treating the typo silently as the message argument to Claude.
        // Pitfall: is_close_typo requires matching first char to avoid false positives for
        //   common English words that happen to be within edit distance 1 (e.g. "task" → "ask").
        if first != sub
          && ( sub.starts_with( first.as_str() ) || first.starts_with( sub ) || is_close_typo( first, sub ) )
        {
          eprintln!(
            "Error: unknown subcommand: {first}. Did you mean '{sub}'?\nRun with --help for usage."
          );
          std::process::exit( 1 );
        }
      }
    }
  }
}

/// Returns `true` when `first` is likely a one-character typo of `sub`.
///
/// Two conditions must both hold:
/// 1. The first character matches — typos virtually always preserve the initial letter;
///    a different first character means a different word entirely, not a typo.
/// 2. Levenshtein distance exactly 1 — one substitution, insertion, or deletion.
///
/// The first-character constraint prevents false positives for common English words that
/// happen to be within edit distance 1 of a known short subcommand name (e.g. `"task"`
/// has edit distance 1 from `"ask"`, but `'t' ≠ 'a'` so it is correctly excluded).
///
/// Used by [`guard_unknown_subcommand`] for mid-word insertion/substitution typos that
/// are not caught by either `starts_with` direction (e.g. `"assk"` vs `"ask"`).
fn is_close_typo( first : &str, sub : &str ) -> bool
{
  // First-character guard: real typos start with the correct letter.
  if first.chars().next() != sub.chars().next() { return false; }
  let a = first.as_bytes();
  let b = sub.as_bytes();
  let la = a.len();
  let lb = b.len();
  if la.abs_diff( lb ) > 1 { return false; }
  if la == lb
  {
    // Same length: exactly one character substitution.
    return a.iter().zip( b.iter() ).filter( |( x, y )| x != y ).count() == 1;
  }
  // Lengths differ by 1: exactly one insertion or deletion.
  let ( longer, shorter ) = if la > lb { ( a, b ) } else { ( b, a ) };
  let mut i = 0;
  let mut j = 0;
  let mut skipped = false;
  while i < longer.len() && j < shorter.len()
  {
    if longer[ i ] == shorter[ j ] { i += 1; j += 1; }
    else if skipped               { return false; }
    else                          { skipped = true; i += 1; }
  }
  true
}

pub( super ) fn run_built_command( builder : &ClaudeCommand, cli : &CliArgs )
{
  let verbosity = cli.verbosity.unwrap_or_default();

  // Concurrency gate: block before subprocess launch when max active claude sessions is reached.
  // Default limit is 30; 0 = unlimited.  dry-run is bypassed by caller (never reaches here).
  let max_sessions = cli.max_sessions.unwrap_or( 30 );
  wait_for_session_slot( max_sessions, verbosity, cli );

  if cli.trace || verbosity.shows_verbose_detail()
  {
    let env     = builder.describe_env();
    let command = builder.describe();
    let mut preview = String::new();
    if !env.is_empty() { preview.push_str( &env ); preview.push( '\n' ); }
    preview.push_str( &command );
    eprintln!( "{preview}" );
  }

  if cli.print_mode || ( cli.message.is_some() && !cli.interactive )
  {
    execution::run_print_mode( builder, cli );
  }
  else
  {
    execution::run_interactive( builder, cli );
  }
}

/// Parse, validate, and execute the `run` subcommand (default mode).  Never returns.
///
/// Shared implementation for both `clr run` and `clr ask` — called from both
/// `run_cli()` (after subcommand dispatch) and `dispatch_ask()`.
pub( super ) fn dispatch_run( tokens : &[ String ] ) -> !
{
  let mut cli = match parse_args( tokens )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  if let Err( e ) = apply_env_vars( &mut cli )
  {
    eprintln!( "Error: {e}" );
    std::process::exit( 1 );
  }

  if cli.help
  {
    print_help();
    std::process::exit( 0 );
  }

  if cli.print_mode && cli.message.is_none()
  {
    eprintln!( "Error: --print requires a message argument" );
    eprintln!( "Run with --help for usage." );
    std::process::exit( 1 );
  }

  let builder = build_claude_command( &cli );

  // Fix(BUG-248): warn when --keep-claudecode is set while CLAUDECODE is present in
  //   the parent environment — the child will run in nested-agent mode unintentionally.
  // Root cause: no diagnostic existed when the user explicitly disabled CLAUDECODE removal;
  //   the consequence (nested-agent context injection) is non-obvious without a warning.
  // Pitfall: gate on shows_warnings() (level ≥ 2) so operators who suppress output at
  //   --verbosity 0/1 still get silence; the warning is informational, not fatal.
  //   Placed before the dry-run check so it fires in all execution modes including --dry-run.
  {
    let verbosity_for_warning = cli.verbosity.unwrap_or_default();
    if cli.keep_claudecode
      && verbosity_for_warning.shows_warnings()
      && std::env::var( "CLAUDECODE" ).is_ok()
    {
      eprintln!(
        "Warning: --keep-claudecode is set and CLAUDECODE is present in environment; \
         child claude will run in nested-agent mode"
      );
    }
  }

  if cli.dry_run
  {
    handle_dry_run( &builder );
    std::process::exit( 0 );
  }

  run_built_command( &builder, &cli );
  std::process::exit( 0 );
}

/// Parse, validate, and execute the `ask` subcommand.  Never returns.
///
/// `ask` is a pure semantic alias for `run` — delegates directly to `dispatch_run()`.
/// The only difference from `clr run` is that `clr ask --help` shows the ask-specific
/// help text rather than the generic `clr` help.
pub( super ) fn dispatch_ask( tokens : &[ String ] ) -> !
{
  if tokens.iter().skip( 1 ).any( | t | t == "--help" || t == "-h" )
  {
    print_ask_help();
  }
  // Fix(BUG-249): 'clr ask help' must show ask help, not treat "help" as a message.
  // Root cause: only --help/-h were intercepted; positional "help" flowed into
  //   dispatch_run as a message and hit the session gate when limit was reached.
  // Pitfall: mirrors BUG-215 fix in run_cli() for 'clr run help'; both subcommands
  //   need the positional check; future subcommands that delegate to dispatch_run
  //   must include it too.
  if tokens.get( 1 ).map( String::as_str ) == Some( "help" )
  {
    print_ask_help();
  }
  dispatch_run( &tokens[ 1 .. ] );
}

/// Parse, validate, and execute the `isolated` subcommand.  Never returns.
pub( super ) fn dispatch_isolated( tokens : &[ String ] ) -> !
{
  let mut cli = match parse_isolated_args( &tokens[ 1 .. ] )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  apply_isolated_env_vars( &mut cli );
  if cli.creds_path.is_empty()
  {
    eprintln!( "{CREDS_PATH_ERROR}" );
    std::process::exit( 1 );
  }
  run_isolated_command(
    "isolated",
    &cli.creds_path,
    cli.timeout_secs,
    cli.trace,
    IsolatedModel::Default,
    EffortLevel::Max,
    cli.message.as_deref(),
    &cli.passthrough_args,
    cli.message.is_some(), // skip-perms when a real task message is present
    false,                 // chrome stays on for isolated tasks (may use browser tools)
  )
}

/// Parse, validate, and execute the `refresh` subcommand.  Never returns.
pub( super ) fn dispatch_refresh( tokens : &[ String ] ) -> !
{
  let mut cli = match parse_refresh_args( &tokens[ 1 .. ] )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  apply_refresh_env_vars( &mut cli );
  if cli.creds_path.is_empty()
  {
    eprintln!( "{CREDS_PATH_ERROR}" );
    std::process::exit( 1 );
  }
  run_refresh_command( &cli.creds_path, cli.timeout_secs, cli.trace )
}

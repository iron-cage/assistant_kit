mod parse;
mod env;
mod retry_classify;
mod execution;
mod cred_parse;
mod builder;
mod fence;
mod credential;
mod help;
mod gate;
mod column_validate;
mod ps;
mod kill;
mod tools;
mod scope;
mod query;
mod summary;
mod json_config;
mod json_config_isolated;
mod config;
// summary_unit_test.rs (external test) imports render_summary/resolve_fields via the public API.
// The unused_imports lint fires for pub use in private modules when no code in the lib crate itself
// references the re-exported path — but the test file consumer is invisible at lib-compile time.
#[ allow( unused_imports ) ]
pub use summary::{ render_summary, resolve_fields, extract_session_id };

// gate_unit_test.rs (external test) imports gate_max_attempts_from/gate_poll_secs_from/
// gate_stale_secs_from via the public API. Same false-positive unused_imports rationale
// as the summary re-export above.
#[ allow( unused_imports ) ]
pub use gate::{ gate_max_attempts_from, gate_poll_secs_from, gate_stale_secs_from };

// tools_command_test.rs (external test) imports TOOLS via the public API for the
// sync-guard tests (BUG-409). Same false-positive unused_imports rationale as above.
#[ allow( unused_imports ) ]
pub use tools::TOOLS;

use std::io::IsTerminal;
use claude_runner_core::{ ClaudeCommand, EffortLevel, IsolatedModel };
use claude_storage_core::SessionId;
use parse::CliArgs;
use cred_parse::{
  IsolatedArgs,
  parse_isolated_args, parse_refresh_args,
  apply_isolated_env_vars, apply_refresh_env_vars,
};
pub use fence::strip_fences;
use credential::{ run_isolated_command, run_refresh_command };

const CREDS_PATH_ERROR : &str =
  "Error: cannot resolve credentials path: HOME is not set; provide --creds or set CLR_CREDS\nRun with --help for usage.";
use help::print_ask_help;
use gate::{ trace_gate_wait_exposure, wait_for_session_slot };
pub( super ) use ps::dispatch_ps;
pub( super ) use kill::dispatch_kill;
pub( super ) use tools::dispatch_tools;
pub( crate ) use scope::dispatch_scope;
pub( super ) use query::{ dispatch_query, run_query_daemon };

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
pub( super ) fn handle_dry_run( builder : &ClaudeCommand, transplant : Option< &builder::SessionTransplant > )
{
  println!( "{}", builder.describe_full() );
  // Fix(BUG-490): preview the planned session transplant — dry-run performs no copy,
  //   but the copy is part of what the real run would do and must be visible.
  // Root cause: dropping the dead CLAUDE_CODE_SESSION_DIR export removed the only
  //   describe_full() trace of --from, leaving the flag invisible in previews.
  // Pitfall: keep this a preview only — dry-run must stay side-effect-free (BUG-231/319).
  if let Some( plan ) = transplant
  {
    println!(
      "# session-transplant: {} -> {}",
      plan.source_file.display(),
      plan.target_storage_dir.display()
    );
  }
}

// Fix(BUG-212): `run` was absent; typing `clr running` produced no helpful error.
// Root cause: list was never updated when `run` became an explicit subcommand.
// Pitfall: update both this list and the dispatch match in lib.rs when adding a subcommand.
const KNOWN_SUBCOMMANDS : &[ &str ] = &[ "run", "ask", "isolated", "refresh", "help", "ps", "kill", "tools", "scope", "query" ];

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
        // Fix(BUG-302): add minimum-length threshold to prefix branch; remove extension branch.
        // Root cause: `sub.starts_with(first)` fired for any prefix with no minimum length
        //   ("is" matched "isolated"); `first.starts_with(sub)` matched morphological extensions
        //   ("asked" matched "ask") which are never typos — both caused valid run messages to be
        //   rejected with "Did you mean?".
        // Pitfall: short truncations like "kil" (len 3 < 4) are still caught via is_close_typo
        //   (deletion, abs_diff=1) — the len >= 4 gate only removes the starts_with path, not
        //   the is_close_typo path. The extension branch must be removed entirely: extensions are
        //   lexically distinct words, not typos, and is_close_typo already covers 1-char edits.
        if first != sub
          && ( ( first.len() >= 4 && sub.starts_with( first.as_str() ) ) || is_close_typo( first, sub ) )
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

/// Resolve the journal directory from CLI args, env var, or home-based default.
fn resolve_journal_dir( journal_dir : Option< &str > ) -> std::path::PathBuf
{
  if let Some( d ) = journal_dir
  {
    return std::path::PathBuf::from( d );
  }
  if let Ok( v ) = std::env::var( "CLR_JOURNAL_DIR" )
  {
    if !v.is_empty() { return std::path::PathBuf::from( v ); }
  }
  std::env::var( "HOME" )
    .map_or_else( | _ | std::path::PathBuf::from( ".clr/journal" ), | h | std::path::PathBuf::from( h ).join( ".clr" ).join( "journal" ) )
}

/// Create a `JournalWriter` from CLI args unless journaling is disabled (`--journal off`).
///
/// Resolution order for the directory: `--journal-dir` > `CLR_JOURNAL_DIR` > `~/.clr/journal/`.
/// The directory is created if it does not exist. I/O errors during directory creation are
/// silently ignored — journaling is best-effort and must not abort the runner.
pub( super ) fn resolve_journal_writer(
  journal     : Option< &str >,   // journal level; "off" disables journaling entirely
  journal_dir : Option< &str >,   // --journal-dir override; falls back to CLR_JOURNAL_DIR then ~/.clr/journal/
) -> Option< claude_journal::JournalWriter >
{
  let level = journal.unwrap_or( "full" );
  if level == "off" { return None; }
  let dir = resolve_journal_dir( journal_dir );
  let _ = std::fs::create_dir_all( &dir );
  Some( claude_journal::JournalWriter::new( dir ) )
}

pub( super ) fn run_built_command(
  builder             : &ClaudeCommand,                            // assembled command to trace (--trace) and execute
  cli                 : &CliArgs,                                  // parsed flags driving the gate/trace/dispatch decisions below
  journal             : Option< &claude_journal::JournalWriter >,  // optional event log, forwarded to the gate wait and execution paths
  expected_session_id : Option< &SessionId >,                      // session id used downstream to detect a `-c` resume mismatch
)
{
  // Print/interactive dispatch decision, computed once and reused for both the
  // concurrency gate (print-mode only — interactive sessions never contend for
  // a slot) and the dispatch branch below, so the two can never disagree.
  //
  // Fix(BUG-425/427): route to print mode whenever stdin has no TTY to interact
  //   through, or file/stdin content is already available to serve as the prompt.
  // Root cause: the formula only checked message presence, so a script/CI invocation
  //   with no message hung on the interactive REPL despite stdin having no terminal
  //   to interact with, and --file/piped content alone never triggered print mode.
  // Pitfall: an explicit --interactive must gate every inferred term here, not only
  //   message-presence — gating message alone still forced print mode under
  //   --interactive whenever stdin was non-TTY, defeating the flag's purpose for the
  //   ordinary case (piped/redirected stdin with no real TTY attached).
  let is_tty = std::io::stdin().is_terminal();
  let is_print_invocation = cli.print_mode
    || ( !cli.interactive
      && ( cli.message.is_some() || !is_tty || cli.file.is_some() || cli.stdin_content.is_some() ) );

  // Concurrency gate: block before subprocess launch when max active print-mode
  // sessions is reached. Default limit is 8; 0 = unlimited.  dry-run is bypassed
  // by caller (never reaches here).
  if is_print_invocation
  {
    let max_sessions = cli.max_sessions.unwrap_or( 8 );
    // Fix(BUG-445): `cli.timeout` is `Some` only when expressed (--timeout flag
    // or CLR_TIMEOUT env, env.rs fallback) — `is_some()` marks expression for
    // the exposure note, and `unwrap_or( 0 )` maps both unexpressed (None) and
    // the explicit `--timeout 0` opt-out to 0 = no gate-budget fallback, so a
    // built-in print default never reaches the gate (moot in production since
    // TSK-503 zeroed that default, but the shield still guards the test hook).
    // Root cause: the gate call carried no timeout input at all — gate-wait's
    // only timing signal was the opt-in CLR_REMAINING_TIMEOUT_SECS env var.
    // Pitfall: pass the EXPRESSED value, never the effective default — see
    // effective_gate_attempts()'s Fix(BUG-445) note in gate.rs.
    trace_gate_wait_exposure( max_sessions, cli.trace, cli.timeout.is_some() );
    let mut runner_attempt = 0u32;
    wait_for_session_slot(
      max_sessions,
      cli.quiet,
      cli.gate_poll_secs.unwrap_or( 30 ),
      cli.gate_max_attempts.unwrap_or( 1000 ),
      cli.gate_stale_secs,
      u64::from( cli.timeout.unwrap_or( 0 ) ),
      journal,
      &mut | e | { execution::apply_runner_retry( cli, e, &mut runner_attempt, journal ); },
    );
  }

  if cli.trace
  {
    eprintln!( "{}", builder.describe_full() );
  }

  if is_print_invocation
  {
    execution::run_print_mode( builder, cli, journal, expected_session_id );
  }
  else
  {
    execution::run_interactive( builder, cli, journal );
  }
}

/// Parse, validate, and execute the `run` subcommand (default mode).  Never returns.
///
/// Shared implementation for both `clr run` and `clr ask` — called from both
/// `run_cli()` (after subcommand dispatch) and `dispatch_ask()`.
pub( super ) fn dispatch_run( tokens : &[ String ] ) -> !
{
  // JSON config tier 2: detect stdin JSON BEFORE parse_args so stdin is consumed once.
  // Ordering: detect_stdin_json reads stdin; parse_args reads only argv — no conflict.
  let stdin_payload = env::detect_stdin_json( tokens );
  let mut cli = match parse_args( tokens )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  // BUG-424: non-JSON piped stdin forwards to the subprocess instead of being discarded.
  // Destructure here (once) rather than downstream — stdin_payload is consumed exactly
  // once, and the JSON branch below still expects the old `Option<String>` shape.
  let stdin_json = match stdin_payload
  {
    Some( env::StdinPayload::Raw( bytes ) ) => { cli.stdin_content = Some( bytes ); None }
    Some( env::StdinPayload::Json( src ) )  => Some( src ),
    None => None,
  };
  // JSON config: apply file-based or stdin-based params AFTER CLI parse (tier 1 already set)
  // but BEFORE apply_env_vars (tier 3). apply_json_config's is_none() / !bool checks ensure
  // CLI-set fields are never overwritten.
  let src_path = env::resolve_args_file_path( cli.args_file.as_deref() );
  if let Some( ref path ) = src_path
  {
    if let Err( e ) = json_config::load_and_apply( path, &mut cli )
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  }
  else if let Some( ref src ) = stdin_json
  {
    match json_config::parse_json_object( src )
    {
      Ok( map ) => json_config::apply_json_config( &mut cli, &map ),
      Err( e )  => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
    }
  }
  if let Err( e ) = apply_env_vars( &mut cli )
  {
    eprintln!( "Error: {e}" );
    std::process::exit( 1 );
  }
  // Config-file tier 4 (final tier): `.clr.toml` (project) / `~/.clr/config.toml` (user),
  // applied AFTER CLR_* env vars (tier 3) — apply_config_defaults' is_none() / !bool checks
  // ensure higher tiers are never overwritten. Task 408 removed the BUG-008 prefs.json
  // fallback that previously ran after this tier, since it was a no-op for anyone using
  // config.toml's `model` key (set on `parsed.model` right here).
  match config::load_config()
  {
    Ok( config ) =>
    {
      if let Err( e ) = config::apply_config_defaults( &mut cli, &config )
      {
        eprintln!( "Error: {e}" );
        std::process::exit( 1 );
      }
    }
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  }

  if cli.help
  {
    print_help();
    std::process::exit( 0 );
  }

  // Fix(BUG-427): extended from `cli.message.is_none()` alone so --file/piped-stdin
  //   content satisfies --print's message requirement, not just a positional message.
  // Root cause: the guard was blind to `cli.file`/`cli.stdin_content`, rejecting
  //   `--print --file <path>` even though --file supplies the prompt content.
  // Pitfall: this guard only fires when `cli.print_mode` is explicitly set — it does
  //   not need the TTY term from `is_print_invocation`, since an explicit --print
  //   already settled the mode-selection question.
  // Pitfall: `cli.stdin_content` must be checked for non-emptiness, not bare
  //   `.is_none()` — see the matching fix/comment on the `-c`-injection gate in
  //   builder.rs. A bare `.is_none()` check let empty non-TTY stdin (the default for
  //   this guard's whole scenario space) slip past this rejection and attempt a real
  //   subprocess launch with no message, surfacing later as a 60s "binary not found,
  //   retries exhausted" failure instead of this guard's own immediate, specific error.
  let has_stdin_content = cli.stdin_content.as_ref().is_some_and( | v | !v.is_empty() );
  if cli.print_mode && cli.message.is_none() && cli.file.is_none() && !has_stdin_content
  {
    eprintln!( "Error: --print requires a message argument" );
    eprintln!( "Run with --help for usage." );
    std::process::exit( 1 );
  }

  let ( builder, expected_id, prep ) = build_claude_command( &cli );

  // Fix(BUG-248): warn when --keep-claudecode is set while CLAUDECODE is present in
  //   the parent environment — the child will run in nested-agent mode unintentionally.
  // Root cause: no diagnostic existed when the user explicitly disabled CLAUDECODE removal;
  //   the consequence (nested-agent context injection) is non-obvious without a warning.
  // Pitfall: gate on !cli.quiet so --quiet suppresses this informational warning;
  //   placed before the dry-run check so it fires in all execution modes including --dry-run.
  if cli.keep_claudecode
    && !cli.quiet
    && std::env::var( "CLAUDECODE" ).is_ok()
  {
    eprintln!(
      "Warning: --keep-claudecode is set and CLAUDECODE is present in environment; \
       child claude will run in nested-agent mode"
    );
  }

  if cli.dry_run
  {
    handle_dry_run( &builder, prep.transplant.as_ref() );
    std::process::exit( 0 );
  }

  // Fix(BUG-491): validate the effective working directory before the gate wait and
  //   spawn — a nonexistent --dir/--to must fail fast, by name, with no retries.
  // Root cause: spawn's ErrorKind::NotFound conflates a missing cwd with a missing
  //   binary; the error surfaced as "claude binary not found" plus an npm install hint
  //   and a retry ladder, none of which describe the actual problem.
  // Pitfall: keep this after the dry-run exit (--dry-run must preview any path,
  //   validation-free) and before the session gate (failing after a gate wait burns
  //   a slot on an invocation that can never start).
  if let Some( ref dir ) = prep.effective_working_dir
  {
    if !dir.is_dir()
    {
      eprintln!(
        "Error: [Runner] working directory does not exist: {} (--dir/--to must name an existing directory)",
        dir.display()
      );
      std::process::exit( 1 );
    }
  }

  // Fix(BUG-490): physically copy the source session into the target's own storage
  //   before spawn so the injected -c continues the transplanted history.
  if let Some( ref plan ) = prep.transplant
  {
    builder::execute_session_transplant( plan );
  }

  // Fix(BUG-319): resolve journal writer AFTER the dry-run exit so that `--dry-run`
  //   does not create the journal directory as a filesystem side effect.
  // Root cause: `resolve_journal_writer()` calls `create_dir_all()` unconditionally;
  //   placing it before the dry-run check meant every `--dry-run` invocation created
  //   `~/.clr/journal/` (or the `--journal-dir` path) even though no events are emitted.
  // Pitfall: `journal` is only consumed by `run_built_command()` — safe to defer.
  let journal = resolve_journal_writer( cli.journal.as_deref(), cli.journal_dir.as_deref() );
  run_built_command( &builder, &cli, journal.as_ref(), expected_id.as_ref() );
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

/// Gate an isolated session through the same concurrency mechanism as run/ask.
///
/// Unlike refresh (which always runs a fixed, throwaway prompt and discards the
/// response), isolated can run arbitrarily long real user tasks, so it must contend
/// for a slot too. dry-run bypasses the gate, matching run/ask's own documented
/// dry-run bypass. isolated has no --quiet flag, so progress messages always show
/// (quiet=false); the 3 gate-tuning knobs are env-var-only here — isolated has no
/// config.toml tier for any parameter, so these stay consistent with its other
/// fields rather than gaining a CLI-flag/config tier run/ask alone has (see
/// `gate_poll_secs_from()`/`gate_stale_secs_from()` doc for the one-shot-vs-tiered split).
fn gate_isolated_session( cli : &IsolatedArgs, journal : Option< &claude_journal::JournalWriter > )
{
  if cli.dry_run { return; }
  let max_sessions = cli.max_sessions.unwrap_or( 8 );
  // Fix(BUG-445): `timeout_expressed` (set by the --timeout parser arm or the
  // CLR_TIMEOUT env application) gates the budget fallback — isolated's
  // built-in 30s default must never default the gate budget, or every default
  // invocation would fail-fast at 30s instead of queueing (~8.3h ceiling). An
  // expressed `--timeout 0` passes 0 = deliberate unlimited, no fallback.
  // Root cause: `timeout_secs` is a plain u64 with a baked-in default, so the
  // gate call could not distinguish "caller said 30" from "nobody said
  // anything" — the expression bit had to be captured at parse time.
  // Pitfall: pass the EXPRESSED value, never the effective default — see
  // effective_gate_attempts()'s Fix(BUG-445) note in gate.rs.
  trace_gate_wait_exposure( max_sessions, cli.trace, cli.timeout_expressed );
  wait_for_session_slot(
    max_sessions,
    false,
    gate::gate_poll_secs_from( env::env_str( "CLR_GATE_POLL_SECS" ).as_deref() ),
    gate::gate_max_attempts_from( env::env_str( "CLR_GATE_MAX_ATTEMPTS" ).as_deref() ),
    gate::gate_stale_secs_from( env::env_str( "CLR_GATE_STALE_SECS" ).as_deref() ),
    if cli.timeout_expressed { cli.timeout_secs } else { 0 },
    journal,
    &mut | e | { eprintln!( "Error: [Runner] {e} (exit 1)" ); std::process::exit( 1 ); },
  );
}

/// Parse, validate, and execute the `isolated` subcommand.  Never returns.
#[ allow( clippy::too_many_lines ) ] // sequential dispatch phases — extracting helpers adds indirection without reducing complexity
pub( super ) fn dispatch_isolated( tokens : &[ String ] ) -> !
{
  // JSON config: no --file gate for isolated (--file is not a stdin-conflict source here).
  let stdin_json = env::detect_stdin_json_unconstrained();
  let mut cli = match parse_isolated_args( &tokens[ 1 .. ] )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  let src_path = env::resolve_args_file_path( cli.args_file.as_deref() );
  if let Some( ref path ) = src_path
  {
    if let Err( e ) = json_config_isolated::load_and_apply_isolated( path, &mut cli )
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  }
  else if let Some( ref src ) = stdin_json
  {
    match json_config::parse_json_object( src )
    {
      Ok( map ) => json_config_isolated::apply_json_config_isolated( &mut cli, &map ),
      Err( e )  => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
    }
  }
  if let Err( e ) = apply_isolated_env_vars( &mut cli )
  {
    eprintln!( "Error: {e}" );
    std::process::exit( 1 );
  }
  if cli.creds_path.is_empty()
  {
    eprintln!( "{CREDS_PATH_ERROR}" );
    std::process::exit( 1 );
  }

  // Phase 2: validate --dir path exists before spawning subprocess (skip in dry-run).
  if !cli.dry_run
  {
    if let Some( ref d ) = cli.dir
    {
      if !std::path::Path::new( d ).exists()
      {
        eprintln!( "Error: --dir path does not exist: {d}" );
        std::process::exit( 1 );
      }
    }
  }

  // Phase 3: validate --file path exists before spawning subprocess (skip in dry-run).
  if !cli.dry_run
  {
    if let Some( ref f ) = cli.file
    {
      if !std::path::Path::new( f ).exists()
      {
        eprintln!( "Error: --file path does not exist: {f}" );
        std::process::exit( 1 );
      }
    }
  }

  // Phase 4: validate --json-schema path when value looks like a file path (skip in dry-run).
  // Values starting with '{' or '[' are treated as inline JSON literals — no path check.
  // All other values are treated as file paths; a missing file is caught here with a clear
  // error rather than letting the claude binary emit a cryptic JSON-parse failure.
  if !cli.dry_run
  {
    if let Some( ref s ) = cli.json_schema
    {
      let trimmed = s.trim_start();
      if !trimmed.starts_with( '{' ) && !trimmed.starts_with( '[' )
        && !std::path::Path::new( s ).exists()
      {
        eprintln!( "Error: --json-schema path does not exist: {s}" );
        std::process::exit( 1 );
      }
    }
  }

  // Phase 5: validate --mcp-config paths exist before spawning subprocess (skip in dry-run).
  if !cli.dry_run
  {
    for m in &cli.mcp_config
    {
      if !std::path::Path::new( m ).exists()
      {
        eprintln!( "Error: --mcp-config path does not exist: {m}" );
        std::process::exit( 1 );
      }
    }
  }

  // Phase 2: inject --dir/--add-dir into the front of passthrough_args so they
  // appear in the subprocess command before any user-supplied passthrough flags.
  let mut passthrough : Vec< String > = Vec::new();
  if let Some( ref d ) = cli.dir
  {
    passthrough.push( "--dir".to_string() );
    passthrough.push( d.clone() );
  }
  for ad in &cli.add_dirs
  {
    passthrough.push( "--add-dir".to_string() );
    passthrough.push( ad.clone() );
  }
  passthrough.extend_from_slice( &cli.passthrough_args );

  let journal = if cli.dry_run { None } else { resolve_journal_writer( cli.journal.as_deref(), cli.journal_dir.as_deref() ) };
  gate_isolated_session( &cli, journal.as_ref() );

  run_isolated_command(
    "isolated",
    &cli.creds_path,
    cli.timeout_secs,
    cli.trace,
    cli.dry_run,
    cli.no_compact_window,
    cli.model.clone().map_or( IsolatedModel::Default, IsolatedModel::Specific ),
    cli.effort.unwrap_or( EffortLevel::Max ),
    cli.message.as_deref(),
    &passthrough,
    cli.message.is_some(), // skip-perms when a real task message is present
    cli.no_chrome,
    cli.file.as_deref(),
    cli.expect.as_deref(),
    cli.expect_strategy.as_deref(),
    journal,
    cli.output_file.as_deref(),
    cli.strip_fences,
    cli.output_style.as_deref(),
    cli.summary_fields.as_deref(),
    cli.no_effort_max,
    cli.system_prompt.as_deref(),
    cli.append_system_prompt.as_deref(),
    cli.json_schema.as_deref(),
    &cli.mcp_config,
    cli.allowed_tools.as_deref(),
    cli.disallowed_tools.as_deref(),
    cli.max_budget_usd.as_deref(),
    cli.max_turns.as_deref(),
  )
}

/// Parse, validate, and execute the `refresh` subcommand.  Never returns.
pub( super ) fn dispatch_refresh( tokens : &[ String ] ) -> !
{
  let stdin_json = env::detect_stdin_json_unconstrained();
  let mut cli = match parse_refresh_args( &tokens[ 1 .. ] )
  {
    Ok( c )  => c,
    Err( e ) => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
  };
  let src_path = env::resolve_args_file_path( cli.args_file.as_deref() );
  if let Some( ref path ) = src_path
  {
    if let Err( e ) = json_config_isolated::load_and_apply_refresh( path, &mut cli )
    {
      eprintln!( "Error: {e}" );
      std::process::exit( 1 );
    }
  }
  else if let Some( ref src ) = stdin_json
  {
    match json_config::parse_json_object( src )
    {
      Ok( map ) => json_config_isolated::apply_json_config_refresh( &mut cli, &map ),
      Err( e )  => { eprintln!( "Error: {e}" ); std::process::exit( 1 ); }
    }
  }
  if let Err( e ) = apply_refresh_env_vars( &mut cli )
  {
    eprintln!( "Error: {e}" );
    std::process::exit( 1 );
  }
  if cli.creds_path.is_empty()
  {
    eprintln!( "{CREDS_PATH_ERROR}" );
    std::process::exit( 1 );
  }
  let journal = if cli.dry_run { None } else { resolve_journal_writer( cli.journal.as_deref(), cli.journal_dir.as_deref() ) };
  run_refresh_command( &cli.creds_path, cli.timeout_secs, cli.trace, cli.dry_run, cli.no_compact_window, journal )
}

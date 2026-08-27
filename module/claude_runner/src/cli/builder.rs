//! Command building: session continuity check and `ClaudeCommand` construction.

use std::io::IsTerminal;
use super::parse::CliArgs;
use claude_runner_core::{ ClaudeCommand, EffortLevel };
use claude_storage_core::{ SessionId, continuation };

/// Return the `SessionId` of the most-recently-modified qualifying session when prior
/// conversation history exists in `storage_dir` (a project's encoded session storage,
/// e.g. `$HOME/.claude/projects/{encoded(dir)}/`), or `None` when no prior session is found.
///
/// Fix(BUG-214-reopen): use project-specific storage path, never `$HOME/.claude/` itself.
/// Root cause: an earlier fallback checked `$HOME/.claude/` (always non-empty — holds
/// credentials, projects/ dir, etc.) so `-c` was injected even for fresh project directories.
/// Pitfall: `$HOME/.claude/` is Claude's global config dir, not per-project session storage;
///   actual project sessions live at `$HOME/.claude/projects/{encoded(cwd)}/`.
///
/// Fix(BUG-320): returns `Option<SessionId>` instead of `bool` so the caller can record
/// which session UUID it expects claude to resume — enabling post-execution mismatch detection.
/// Root cause: bool return made the expected UUID inaccessible; mismatch was undetectable.
/// Pitfall: do not use `claude_storage_core::continuation::check_continuation` here —
///   it detects legacy `conversation.json` / `.claude*` formats that produce no UUID.
///
/// Fix(BUG-493): takes the source storage dir directly — the `--session-dir` raw-override
///   branch is gone along with the parameter's effect.
/// Root cause: `--session-dir` only redirected this scan and a `CLAUDE_CODE_SESSION_DIR`
///   export claude ≥2.x ignores, so `-c` was gated off a directory claude never reads.
/// Pitfall: the caller always resolves a storage dir (`--from` or cwd default), so the
///   former `most_recent_session_id(cwd)` fallback branch was already unreachable —
///   both paths encode the identical canonical cwd via the same `encode_path()` call.
///
/// `pub( crate )`: also called from `topic.rs::disambiguate_slug` (Fix(BUG-542)) to probe
/// whether an auto-name candidate's storage already holds a session before treating the
/// name as free — the identical "does this storage have a qualifying session" question,
/// reused rather than re-implemented.
pub( crate ) fn session_exists( storage_dir : &std::path::Path ) -> Option< SessionId >
{
  continuation::most_recent_session_in_dir( storage_dir )
}

/// Resolve the effective working directory from `--dir` and `--topic` args.
///
/// Fix(BUG-229): guard empty string — `--topic ""` must be identity, not degenerate `/-`
/// Root cause: only `"."` was checked; empty string passed the guard and produced bare-hyphen dir
/// Pitfall: `env_str` already filters empty, but CLI path can deliver `""` via `--topic ""`
///
/// Fix(BUG-231): skip `create_dir_all` in dry-run — dry-run must be side-effect-free
/// Root cause: `build_claude_command` runs before the dry-run branch; mkdir executed unconditionally
/// Pitfall: builder computes the path for display; only the run path needs the physical directory
fn resolve_effective_dir( cli : &CliArgs ) -> Option< std::path::PathBuf >
{
  let base_dir = cli.dir.as_deref().map( std::path::PathBuf::from );
  match cli.topic.as_deref()
  {
    Some( sub ) if sub != "." && !sub.is_empty() =>
    {
      // Base and join both come from `claude_topic_core::identity` so this function,
      // `topic.rs`'s free-name probe, and `topics.rs`'s listing/resolution can never
      // disagree about where a given topic name lives.
      let base = claude_topic_core::topic_base( cli.dir.as_deref(), cli.global );
      let effective = claude_topic_core::topic_dir( &base, sub );
      if !cli.dry_run
      {
        let _ = std::fs::create_dir_all( &effective );
      }
      Some( effective )
    }
    // No topic directory to place, so --global has nothing to redirect: --dir (or cwd) stands.
    _ => base_dir,
  }
}

/// A planned physical copy of a source session file into the target project's own
/// session storage, executed by `execute_session_transplant` just before spawn (BUG-490).
#[ derive( Debug ) ]
pub( crate ) struct SessionTransplant
{
  /// Qualifying source session file (`<uuid>.jsonl`) to copy.
  pub( crate ) source_file : std::path::PathBuf,
  /// Target project's encoded storage directory the copy lands in.
  pub( crate ) target_storage_dir : std::path::PathBuf,
  /// `--keep-clone`: preserve a non-empty destination copy (refresh mtime only)
  /// instead of the default overwrite-with-fresh-copy.
  pub( crate ) keep_existing : bool,
}

/// Side-band data `build_claude_command` hands the run dispatcher: the effective
/// working directory (post `--dir`/`--topic` resolution), the session
/// transplant plan, and the fork-mode topic plan, when each applies.
#[ derive( Debug ) ]
pub( crate ) struct RunPreparation
{
  /// Effective working directory after `--dir`/`--topic` resolution; `None` = cwd.
  pub( crate ) effective_working_dir : Option< std::path::PathBuf >,
  /// Physical session copy to perform before spawn (BUG-490), if applicable.
  pub( crate ) transplant : Option< SessionTransplant >,
  /// Fork-mode topic plan (`TopicMode::Fork`), if applicable. Mutually exclusive
  /// with `transplant` — fork mode never copies session files.
  pub( crate ) fork : Option< TopicFork >,
}

/// Fork-mode topic plan: the deterministic session identity for a `--topic`
/// invocation resolved to `TopicMode::Fork`, plus everything the dispatcher
/// needs downstream (dry-run preview, registry recording).
///
/// Fork mode replaces the legacy `<base>/-<name>` directory + transplant with a
/// same-directory session fork: the topic lives as a session file named by
/// `UUIDv5( canonical base path, topic name )` inside the base directory's own
/// storage. Staying in the base directory keeps the prompt-cache prefix
/// byte-identical, so the fork reuses the base session's cache instead of
/// re-priming the whole history.
#[ derive( Debug ) ]
pub( crate ) struct TopicFork
{
  /// Topic name as given on the CLI.
  pub( crate ) topic : String,
  /// Canonical physical base directory the identity is keyed on.
  pub( crate ) canonical_base : std::path::PathBuf,
  /// Deterministic topic session id — `topic_session_id( canonical_base, topic )`.
  pub( crate ) session_id : SessionId,
  /// Base's most recent qualifying session to fork from. `None` on repeat use
  /// (the topic session already exists — nothing to fork), under `--new-session`
  /// (fresh start explicitly requested), and on a base with no session yet.
  pub( crate ) source : Option< SessionId >,
  /// Whether the topic session file already exists non-empty (repeat use).
  pub( crate ) repeat : bool,
}

/// Resolve a `--topic` invocation to its fork-mode plan, or `None` when no topic
/// is given (`--topic` absent, `""`, or `"."` — the same identity guard as
/// `resolve_effective_dir`, BUG-229) or the effective mode is `TopicMode::Dir`
/// (legacy directory topics keep the transplant path).
///
/// Exits the process loudly (code 1) on contradictions rather than guessing:
/// - explicit `--topic-mode fork` with `--global` or a non-empty `--from` — the
///   auto rules select Dir for both precisely because fork mode's same-directory
///   cache-identity premise cannot hold for them, so forcing fork past that is a
///   contradiction, not a preference;
/// - `--new-session` on an EXISTING fork topic — the topic's session identity is
///   deterministic and claude rejects `--session-id` reuse, so "start this topic
///   over" requires deleting the session file (path: `clr topics --file NAME`)
///   or switching to `--topic-mode dir`. On a topic that does not exist yet,
///   `--new-session` is honored instead: it suppresses the fork source, creating
///   the topic fresh rather than forking the base's history.
/// - an unresolvable identity (non-UTF-8 base path, or no resolvable storage
///   dir) — the session id cannot be computed, so the run cannot proceed.
///
/// Probes disk (session-file existence, most-recent-session scan) but never
/// writes — dry-run safe (BUG-231's rule: side effects belong to the run path).
fn plan_topic_fork( cli : &CliArgs ) -> Option< TopicFork >
{
  let topic = cli.topic.as_deref().filter( | t | *t != "." && !t.is_empty() )?;
  let mode = claude_topic_core::effective_topic_mode
  (
    cli.topic_mode,
    cli.global,
    cli.from.as_deref(),
    cli.dir.as_deref(),
    topic,
  );
  if mode == claude_topic_core::TopicMode::Dir
  {
    return None;
  }
  if cli.global
  {
    eprintln!
    (
      "Error: --topic-mode fork cannot be combined with --global\n\
       Global topics are shared across callers' working directories, so the\n\
       same-directory fork premise never holds for them. Drop --topic-mode fork\n\
       (global topics use dir mode) or drop --global."
    );
    std::process::exit( 1 );
  }
  if cli.from.as_deref().is_some_and( | f | !f.is_empty() )
  {
    eprintln!
    (
      "Error: --topic-mode fork cannot be combined with --from\n\
       An explicit cross-directory source needs the directory-topic transplant\n\
       machinery. Drop --topic-mode fork (--from topics use dir mode) or drop --from."
    );
    std::process::exit( 1 );
  }

  let base = claude_topic_core::topic_base( cli.dir.as_deref(), false );
  let canonical_base = physical_abs( &base );
  let session_id = match claude_storage_core::topic_session_id( &canonical_base, topic )
  {
    Ok( id ) => id,
    Err( e ) =>
    {
      eprintln!( "Error: cannot compute topic session id for '{topic}': {e}" );
      std::process::exit( 1 );
    }
  };
  let Some( session_file ) = claude_storage_core::topic_session_file( &canonical_base, topic )
  else
  {
    eprintln!
    (
      "Error: cannot resolve session storage for topic '{topic}' (is HOME set?)"
    );
    std::process::exit( 1 );
  };
  let storage = session_file.parent()
    .expect( "storage join always has a parent" )
    .to_path_buf();
  // Existence probe via session_file_path, not a bare exact-join check, so a
  // case-variant extension still counts as the topic already existing — the
  // same case-insensitive qualification rule the continuation scan uses.
  let repeat = session_file_path( &storage, session_id.as_str() )
    .and_then( | path | std::fs::metadata( path ).ok() )
    .is_some_and( | meta | meta.len() > 0 );
  if repeat && cli.new_session
  {
    eprintln!
    (
      "Error: --new-session cannot restart fork-mode topic '{topic}'\n\
       Its session identity is deterministic ({id}) and already exists.\n\
       To start over, delete the session file (path: clr topics --file {topic})\n\
       or use --topic-mode dir.",
      id = session_id.as_str()
    );
    std::process::exit( 1 );
  }
  let source = if repeat || cli.new_session { None }
  else
  {
    session_exists( &storage )
  };
  Some( TopicFork
  {
    topic : topic.to_string(),
    canonical_base,
    session_id,
    source,
    repeat,
  } )
}

/// Resolve `raw` to its physical absolute form — thin delegation to
/// `claude_storage_core::physical_abs`, where the implementation (including the
/// Fix(BUG-543) deepest-existing-prefix fallback) now lives so `clr` and
/// `claude_storage` can never drift apart on the canonical form that storage
/// keys and topic-session identities are computed from.
///
/// claude derives storage names from its physical getcwd, so a relative or symlinked
/// path must resolve to the same physical absolute form or the encoded name silently
/// misses the real storage dir (`./src` would encode as `---src`).
///
/// `pub( crate )`: also called from `topic.rs::disambiguate_slug` (Fix(BUG-542)) so its
/// storage probe resolves an auto-name candidate identically to how this file resolves
/// the same path once it becomes the real effective/source directory.
pub( crate ) fn physical_abs( raw : &std::path::Path ) -> std::path::PathBuf
{
  claude_storage_core::physical_abs( raw )
}

/// Locate the on-disk file for session `id` inside `storage`: exact `<id>.jsonl` join
/// when present, else a directory scan matching stem == `id` with a case-insensitive
/// `jsonl` extension — qualification is extension-case-insensitive, mirroring
/// `claude_storage_core::continuation`'s own scan.
fn session_file_path( storage : &std::path::Path, id : &str ) -> Option< std::path::PathBuf >
{
  let exact = storage.join( format!( "{id}.jsonl" ) );
  if exact.is_file() { return Some( exact ); }
  std::fs::read_dir( storage ).ok()?
    .filter_map( Result::ok )
    .map( | entry | entry.path() )
    .find( | path |
    {
      path.file_stem().and_then( std::ffi::OsStr::to_str ) == Some( id )
        && path.extension().and_then( std::ffi::OsStr::to_str )
          .is_some_and( | ext | ext.eq_ignore_ascii_case( "jsonl" ) )
    } )
}

/// Bump `path`'s mtime by rewriting its first byte in place (read, seek back, write).
///
/// `File::set_modified` needs Rust 1.75; the workspace MSRV is 1.74 — this is the
/// MSRV-compatible equivalent. Content is unchanged: the same byte is written back.
fn refresh_mtime( path : &std::path::Path ) -> std::io::Result< () >
{
  use std::io::{ Read, Seek, Write };
  let mut file = std::fs::OpenOptions::new().read( true ).write( true ).open( path )?;
  let mut first = [ 0u8; 1 ];
  file.read_exact( &mut first )?;
  file.seek( std::io::SeekFrom::Start( 0 ) )?;
  file.write_all( &first )?;
  Ok( () )
}

/// Execute a planned session transplant: copy the source session file into the target
/// project's own storage so a plain `claude -c` there continues the cloned history.
///
/// Fix(BUG-490): physical copy replaces the dead `CLAUDE_CODE_SESSION_DIR` export.
/// Root cause: claude ≥2.x ignores `CLAUDE_CODE_SESSION_DIR` for both reads and writes,
///   so the env-var redirect `--from` relied on was a silent no-op.
/// A non-empty destination is overwritten with a fresh copy of the source by default —
///   an explicit `--from` means "clone from there, now", so a stale copy left by an
///   earlier clone must not silently take precedence over the re-clone. `--keep-clone`
///   inverts this: the (possibly diverged) target copy is preserved and only its mtime
///   refreshed so `-c` still selects it. Both branches announce themselves on stderr
///   (suppressed by `--quiet`) — the collision is never resolved silently.
///   Failures warn loudly and proceed: `-c` against an empty target trips claude's own
///   rejection (BUG-428 fallback), a stale non-empty one trips the BUG-320 mismatch check.
pub( crate ) fn execute_session_transplant( plan : &SessionTransplant, quiet : bool )
{
  let Some( file_name ) = plan.source_file.file_name() else
  {
    eprintln!
    (
      "[Runner] warning: session transplant skipped — source has no file name: {}",
      plan.source_file.display()
    );
    return;
  };
  if let Err( e ) = std::fs::create_dir_all( &plan.target_storage_dir )
  {
    eprintln!
    (
      "[Runner] warning: session transplant failed — cannot create {}: {e}",
      plan.target_storage_dir.display()
    );
    return;
  }
  let dest = plan.target_storage_dir.join( file_name );
  let dest_len = std::fs::metadata( &dest ).ok().map( | meta | meta.len() );
  if dest_len.is_some_and( | len | len > 0 )
  {
    // Non-empty destination = history already present for this session id, left by an
    // earlier clone and possibly diverged since (continued in the target).
    if plan.keep_existing
    {
      // --keep-clone: preserve the target's own branch — refresh its mtime so
      // continuation selection picks it, never overwrite.
      if let Err( e ) = refresh_mtime( &dest )
      {
        eprintln!
        (
          "[Runner] warning: session transplant mtime refresh failed for {}: {e}",
          dest.display()
        );
      }
      if !quiet
      {
        eprintln!
        (
          "[Runner] kept existing session copy {} (--keep-clone; source not re-copied)",
          dest.display()
        );
      }
      return;
    }
    // Default: an explicit --from asks for a fresh clone — replace the stale copy.
    // Any turns added to the target copy after the earlier clone are lost; that is
    // exactly what --keep-clone preserves instead.
    if !quiet
    {
      eprintln!
      (
        "[Runner] re-cloning over existing session copy {} (use --keep-clone to preserve it)",
        dest.display()
      );
    }
  }
  if let Err( e ) = std::fs::copy( &plan.source_file, &dest )
  {
    eprintln!
    (
      "[Runner] warning: session transplant copy failed {} -> {}: {e}",
      plan.source_file.display(),
      dest.display()
    );
  }
}

/// Translate parsed CLI args into a `ClaudeCommand` builder together with the
/// expected `SessionId` for post-execution mismatch detection (BUG-320) and the
/// `RunPreparation` side-band (effective working dir + session transplant plan, BUG-490).
///
/// Session continuation (`-c`) is applied by default unless `--new-session` is set
/// or no prior session exists in the configured storage directory.
/// The returned `Option<SessionId>` is `Some(uuid)` when `-c` was injected, allowing
/// the caller to verify that claude actually resumed that session.
#[ allow( clippy::too_many_lines ) ] // mechanical dispatch — one block per CLI flag mapped to the command builder
pub( crate ) fn build_claude_command( cli : &CliArgs )
  -> ( ClaudeCommand, Option< SessionId >, RunPreparation )
{
  let mut builder = ClaudeCommand::new();

  // Fix(BUG-493): --session-dir is deprecated and inert — this function applies no effect
  //   from it anywhere below. The user-facing warning is emitted once by
  //   `warn_deprecated_session_dir()` in `mod.rs`, which gates it on `--quiet`; emitting
  //   it here as well would print it twice on every run.
  // Root cause: the parameter's only mechanisms were a CLAUDE_CODE_SESSION_DIR export
  //   claude ≥2.x ignores (proven by BUG-490's control test) and a -c injection gate
  //   scanning the override dir — a directory claude never reads, so -c could be
  //   wrongly injected or wrongly suppressed.
  // Pitfall: keep the parameter parsed (CLI flag, CLR_SESSION_DIR, json "session-dir")
  //   so existing invocations don't hard-fail; the warning is the only behavior.

  // Fork-mode topics stay in the base directory — no `-<name>` dir is created or
  // entered; the topic is a deterministically-named session inside the base's own
  // storage. Only dir-mode invocations go through resolve_effective_dir's
  // topic-dir join + create_dir_all.
  let fork = plan_topic_fork( cli );
  let effective_working_dir = if fork.is_some()
  {
    cli.dir.as_deref().map( std::path::PathBuf::from )
  }
  else
  {
    resolve_effective_dir( cli )
  };
  if let Some( ref dir ) = effective_working_dir
  {
    builder = builder.with_working_directory( dir.to_string_lossy().into_owned() );
  }
  if let Some( n ) = cli.max_tokens
  {
    builder = builder.with_max_output_tokens( n );
  }
  // --from: resolve source session dir.
  // Computes scope_for(source_dir).claude_session_dir — the source project's encoded
  // storage path — used below for expected-session lookup and the transplant plan.
  // Fix(BUG-493): the former `--session-dir wins when both are present` rule is gone —
  //   the deprecated parameter no longer suppresses this resolution (see the warning
  //   at the top of this function), so the source storage dir is always computed.
  //
  // Defaults to the current working directory when --from is omitted or empty — the
  // same default-to-cwd rule --to/--dir already applies (see `resolve_effective_dir`).
  // This makes `--to <TARGET>` alone clone outward from cwd by default. When neither
  // --from nor --to is given, source and target both resolve to cwd's own storage, so
  // the self-copy guard below (`target_storage == *src_storage`) makes the transplant a
  // guaranteed no-op — behaviorally identical to the pre-default bare invocation, since
  // `scope_for(physical_abs(cwd)).claude_session_dir` and `most_recent_session_id(cwd)`
  // both encode the identical canonical cwd via the same `encode_path()` call.
  //
  // The raw value is resolved to its physical absolute form before encoding (see
  // `physical_abs`). An explicitly-empty value is treated the same as omitted — same
  // empty-is-identity rule as `--topic ""` (BUG-229); without the filter an empty value
  // would encode to the `-unknown` fallback dir and actively target that storage.
  //
  // Fix(BUG-490): this dir is no longer exported as CLAUDE_CODE_SESSION_DIR — claude ≥2.x
  //   ignores that variable for both reads and writes, so the export made --from
  //   a silent no-op. The source session is instead physically copied into the target's
  //   own storage (see `SessionTransplant` / `execute_session_transplant`).
  // Root cause: the feature's only mechanism was an env contract claude dropped.
  // Pitfall: -c injection and expected-id logic still key off the SOURCE storage dir —
  //   after the copy claude continues that same uuid in the target, so the source's
  //   most-recent session id remains the correct expected id.
  //
  // Pitfall: this computation must run unconditionally — do not reintroduce a
  //   cli.session_dir guard here (see Fix(BUG-493) note above).
  //
  // Fix(BUG-541): when --from is omitted AND --dir/--topic resolves the target to a
  //   directory other than cwd, prefer that target's OWN storage as the source when it
  //   already holds a qualifying session — only fall back to cwd when the target has
  //   none yet (a genuine first use). Applies uniformly to --dir/--to and --topic; both
  //   flow through the same `effective_working_dir`.
  // Root cause: this always re-derived the source from literal cwd, blind to
  //   effective_working_dir. A target directory used more than once (topic or plain
  //   --dir) had its OWN established history silently bypassed whenever cwd's
  //   unrelated most-recent session changed between calls — the first call's clone
  //   correctly seeded the target, but every later call re-read cwd fresh instead of
  //   the target's own copy, transplanting whatever was newest in cwd at that moment
  //   and orphaning the target's actual accumulated conversation. This broke
  //   `docs/cli/command/11_topic.md`'s documented contract that a repeat topic
  //   invocation "finds the copy already in place and continues it... instead of
  //   re-copying" — true only by coincidence, when cwd's own most-recent session
  //   identity never happened to change between calls.
  // Pitfall: only substitutes the source when the target ALREADY has a qualifying
  //   session — an empty/fresh target must still fall through to the cwd-default so
  //   the documented first-use clone is unaffected.
  // Fix(BUG-541 clippy follow-up): rewritten from a `match` with a single real pattern
  //   (`Some`) plus a catch-all `None` arm to `if let`/`else` — clippy::single_match_else
  //   under -D warnings; behavior is unchanged, only the outer dispatch shape.
  let session_from_dir : std::path::PathBuf =
  if let Some( src ) = cli.from.as_deref().filter( | src | !src.is_empty() )
  {
    let abs = physical_abs( &std::path::PathBuf::from( src ) );
    claude_storage_core::scope_for( &abs ).claude_session_dir
  }
  else
  {
    let own_target_storage = effective_working_dir.as_deref().map( | dir |
      claude_storage_core::scope_for( &physical_abs( dir ) ).claude_session_dir );
    match own_target_storage
    {
      Some( storage ) if session_exists( &storage ).is_some() => storage,
      _ =>
      {
        let cwd = std::env::current_dir().unwrap_or_else( | _ | std::path::PathBuf::from( "." ) );
        claude_storage_core::scope_for( &physical_abs( &cwd ) ).claude_session_dir
      }
    }
  };
  // Determine print mode early — used for -c injection, effort injection, and chrome
  // suppression.  Must precede expected_id so all three guards below can reference use_print.
  // Fix(BUG-227): message without -p was silently using TTY passthrough,
  //   producing raw TUI escape codes instead of clean text output in scripted contexts.
  // Root cause: print mode was only enabled by explicit -p/--print; no auto-detection.
  // Pitfall: `--interactive` must suppress this to allow prompted REPL sessions.
  //
  // Fix(BUG-425/427): mirrors `cli/mod.rs`'s dispatch-decision fix — this flag and
  //   that decision must never disagree, since both describe the same invocation.
  // Root cause: this formula lacked the same TTY and file/stdin-content terms that
  //   the mod.rs dispatch decision lacked, before its own BUG-425/427 fix.
  // Pitfall: an explicit --interactive must gate every inferred term here, not only
  //   message-presence — gating message alone still forced print mode under
  //   --interactive whenever stdin was non-TTY, which this test harness's spawned
  //   subprocesses always are (no PTY simulation — see plan's Known Coverage Gap),
  //   defeating the flag's purpose for every real invocation in the suite.
  let is_tty = std::io::stdin().is_terminal();
  let use_print = cli.print_mode
    || ( !cli.interactive
      && ( cli.message.is_some() || !is_tty || cli.file.is_some() || cli.stdin_content.is_some() ) );
  // Fix(BUG-214): inject -c only when a prior session exists in storage
  // Root cause: unconditional -c causes claude binary to exit on first use with no session
  // Pitfall: resumption flags (-c, --continue) require state to resume; guard with existence check
  // Fix(BUG-320): capture expected session UUID — returned to caller for mismatch detection.
  // Root cause: bool return made the expected UUID inaccessible after -c injection.
  // Pitfall: expected_id is None when new_session is set OR when no qualifying session exists.
  // Fix(BUG-493): dropped `cli.session_dir` from this lookup entirely — it used to feed
  //   `session_exists` directly (taking priority over session_from_dir), gating -c on
  //   the contents of a directory claude never actually reads or writes.
  // Root cause: raw --session-dir was treated as an equally-valid "scan this dir
  //   directly" source alongside --from's transplant-target dir, but only the latter
  //   is backed by a real mechanism (BUG-490's physical copy).
  // Pitfall: session_from_dir alone is correct here — it already defaults to cwd's own
  //   storage when --from is absent, so this is a strict fix, not a behavior removal.
  //
  // resume_allowed is the BUG-426/435 message-presence condition, shared verbatim by
  // the legacy -c injection and the fork-mode resume/fork argument injection below —
  // both describe the same question ("will something follow the resumed session as a
  // prompt, or is this an interactive entry?") and must never disagree.
  let resume_allowed = !use_print
    || cli.message.is_some() || cli.print_mode || cli.file.is_some() || cli.interactive
    || cli.stdin_content.as_ref().is_some_and( | v | !v.is_empty() );
  // Fork mode: the expected id is the deterministic topic session id whenever the
  // fork/resume arguments are actually injected (resume_allowed) — including the
  // fresh-create case (`--session-id` alone), where claude is expected to create
  // exactly that file, so the BUG-320 mismatch check verifies creation too. This
  // branch deliberately precedes the new_session one: a first-use fork topic under
  // --new-session still creates the deterministic id (plan_topic_fork only drops
  // the fork SOURCE for it; repeat use under --new-session already exited loudly).
  let expected_id = if let Some( ref fork_plan ) = fork
  {
    if resume_allowed { Some( fork_plan.session_id.clone() ) } else { None }
  }
  else if cli.new_session { None }
  else
  {
    session_exists( &session_from_dir )
  };
  // Transplant plan (BUG-490): expected_id is Some only when not --new-session and a
  // qualifying source session exists — it gates the physical copy of the source session
  // file into the TARGET's own encoded storage.
  // Pitfall: when source and target encode to the same storage dir, no copy is planned —
  //   fs::copy onto the same path truncates the file it is reading from.
  //
  // Fork mode never transplants: source and topic session live in the SAME storage
  // (that is the whole point — identical directory, identical cache prefix), and the
  // fork itself is claude's own `--fork-session`, not a file copy.
  let transplant = if fork.is_some() { None }
  else if let Some( id ) = &expected_id
  {
    let src_storage = &session_from_dir;
    let target_dir = effective_working_dir.as_deref().map_or_else(
      || std::env::current_dir().unwrap_or_else( | _ | std::path::PathBuf::from( "." ) ),
      std::path::Path::to_path_buf,
    );
    let target_storage =
      claude_storage_core::scope_for( &physical_abs( &target_dir ) ).claude_session_dir;
    if target_storage == *src_storage { None }
    else
    {
      session_file_path( src_storage, id.as_str() ).map( | source_file | SessionTransplant
      {
        source_file,
        target_storage_dir : target_storage,
        keep_existing : cli.keep_clone,
      } )
    }
  }
  else { None };
  // Fix(BUG-426): gate -c injection on message-presence (message, --print, --file,
  //   stdin-content) OR cli.interactive — previously unconditional whenever a prior
  //   session existed, regardless of whether anything would follow -c as the resumed
  //   prompt.
  // Root cause: with_continue_conversation( true ) fired solely from session_exists(),
  //   with no check that a message/--print/--file/stdin-content would actually
  //   accompany the resumed session.
  // Pitfall: cli.interactive MUST remain in this condition even though it looks
  //   redundant alongside the message-presence terms — an explicit --interactive
  //   resume with no message is BUG-426's own excluded case (Test Matrix T09), not
  //   part of the defect; dropping this term would regress a working resume path
  //   while fixing a broken one.
  // Pitfall: `cli.stdin_content` must be checked for non-emptiness, not bare
  //   `.is_some()` — `detect_stdin_json()` (env.rs) returns `Some(vec![])`, never
  //   `None`, for any non-TTY stdin that reads zero bytes, and `Command::output()`
  //   defaults stdin to `Stdio::null()` whenever a caller doesn't set `.stdin(...)`
  //   explicitly. A bare `.is_some()` here was therefore unconditionally true for
  //   every non-interactive invocation (any script/CI/container context, and every
  //   subprocess-spawning test helper except `run_with_path_stdin`), silently
  //   reproducing BUG-426's original defect instead of fixing it.
  // Fix(BUG-435): add !use_print as first inner term so interactive mode (bare clr
  //   on TTY) always allows -c when a session exists.  D-10's guard used cli.interactive
  //   (the explicit flag), which bare TTY invocations never set — they enter interactive
  //   mode via TTY detection (BUG-425 fix), never by passing --interactive.  D-10's
  //   purpose was to prevent `claude -c` with no message in print mode; that case is
  //   `use_print=true`, which the existing terms still gate correctly.
  // Root cause: cli.interactive (the flag) is not the same as !use_print (inferred
  //   interactive mode); D-10's guard was too broad and excluded bare TTY interactive.
  // Pitfall: always use !use_print to detect interactive mode, not cli.interactive —
  //   the flag is an explicit opt-in, not the ground truth about the invocation's mode.
  // The message-presence terms live in resume_allowed (defined with expected_id
  // above) so this gate and fork-mode injection can never drift apart.
  if let Some( ref fork_plan ) = fork
  {
    // Fork-mode topic: address the deterministic session directly instead of -c.
    //   repeat use          → --resume <topic-uuid>                    (continue the topic)
    //   first use, source   → --resume <src> --fork-session --session-id <topic-uuid>
    //                         (branch the base's history under the topic's identity)
    //   first use, no source → --session-id <topic-uuid>              (create it fresh)
    // Gated by the same resume_allowed condition as -c: with nothing following as a
    // prompt and print mode active, injecting resume/create args would reproduce
    // BUG-426's empty-resume failure shape.
    if resume_allowed
    {
      if fork_plan.repeat
      {
        builder = builder
          .with_arg( "--resume" ).with_arg( fork_plan.session_id.as_str() );
      }
      else if let Some( ref src ) = fork_plan.source
      {
        builder = builder
          .with_arg( "--resume" ).with_arg( src.as_str() )
          .with_arg( "--fork-session" )
          .with_arg( "--session-id" ).with_arg( fork_plan.session_id.as_str() );
      }
      else
      {
        builder = builder
          .with_arg( "--session-id" ).with_arg( fork_plan.session_id.as_str() );
      }
    }
  }
  else if expected_id.is_some() && resume_allowed
  {
    builder = builder.with_continue_conversation( true );
  }
  if !cli.no_skip_permissions
  {
    builder = builder.with_skip_permissions( true );
  }
  // Fix(BUG-434): gate default max-effort injection on print mode or explicit --effort.
  //   Interactive mode rejects "max" since claude v2.1.78; only inject the default when
  //   there is a prompt to process (use_print) or the caller chose an explicit level.
  // Root cause: unconditional injection always forwarded --effort max regardless of mode;
  //   "max" is rejected in interactive mode by claude v2.1.78+.
  // Pitfall: always gate EffortLevel::Max default on use_print || cli.effort.is_some() —
  //   never inject "max" unconditionally, even when no_effort_max is false.
  if !cli.no_effort_max && ( cli.effort.is_some() || use_print )
  {
    builder = builder.with_effort(
      cli.effort.unwrap_or( EffortLevel::Max )
    );
  }
  // Fix(BUG-304): suppress --chrome whenever print mode is active.
  //   Root cause: Node.js/libuv registers a ref-counted 1-second timerfd (Chrome CDP
  //   reconnect) that is never unref()'d after --print response flush; event loop cannot
  //   drain; clr's cmd.output() holds pipe read-ends open — both sides deadlocked.
  // Pitfall: cli.no_chrome is the explicit user opt-out; use_print is the automatic
  //   suppression that prevents the hang without requiring --no-chrome.
  // Fix(BUG-425): added an independent !is_tty term — chrome suppression previously
  //   fired only transitively through use_print's own formula, which happens to
  //   include a TTY disjunct today but carries no guarantee of doing so after a
  //   future edit to that formula.
  // Root cause: non-TTY stdin was never itself a direct term in this condition,
  //   only reachable indirectly through whatever shape use_print's formula took.
  // Pitfall: keep this term explicit and independent of use_print — folding it back
  //   into a single shared boolean would let a future use_print refactor silently
  //   stop suppressing chrome for non-TTY invocations with no test at this call
  //   site able to catch the regression.
  if cli.no_chrome || use_print || !is_tty
  {
    builder = builder.with_chrome( None );
  }
  if cli.no_persist
  {
    builder = builder.with_no_session_persistence( true );
  }
  if let Some( ref schema ) = cli.json_schema
  {
    builder = builder.with_json_schema( schema.as_str() );
  }
  if !cli.mcp_config.is_empty()
  {
    builder = builder.with_mcp_config( cli.mcp_config.iter().map( String::as_str ) );
  }
  if let Some( ref path ) = cli.file
  {
    builder = builder.with_stdin_file( std::path::PathBuf::from( path ) );
  }
  else if let Some( ref bytes ) = cli.stdin_content
  {
    builder = builder.with_stdin_content( bytes.clone() );
  }
  if cli.keep_claudecode
  {
    builder = builder.with_unset_claudecode( false );
  }
  if cli.verbose
  {
    builder = builder.with_verbose( true );
  }
  if let Some( ref model ) = cli.model
  {
    builder = builder.with_model( model.clone() );
  }
  // Fix(BUG-493): with_session_dir()'s CLAUDE_CODE_SESSION_DIR export removed —
  //   claude ≥2.x ignores it entirely (same dead mechanism as BUG-490's --from), so
  //   exporting it accomplished nothing but implying a working redirect. The parameter
  //   remains parseable (see cli.session_dir uses above: dropped from -c gating,
  //   dropped from suppressing --from) and triggers a deprecation warning in mod.rs.
  // Root cause: the feature's only mechanism was an env contract claude dropped; no
  //   runner-side emulation exists for a raw storage redirect (unlike --from, whose
  //   transplant BUG-490 already fixed).
  if let Some( ref sp ) = cli.system_prompt
  {
    builder = builder.with_system_prompt( sp.clone() );
  }
  if let Some( ref asp ) = cli.append_system_prompt
  {
    builder = builder.with_append_system_prompt( asp.clone() );
  }
  if use_print
  {
    builder = builder.with_arg( "--print" );
  }
  if let Some( ref msg ) = cli.message
  {
    // Fix(BUG-224): inject as suffix not prefix so the user task
    //   comes first in Claude's context window — earlier tokens carry more weight.
    // Root cause: original format!("ultrathink {msg}") buried the task description
    //   under the directive; suffix form preserves natural "state task, then direct thinking"
    //   order that matches Claude's conversational expectations.
    // Pitfall: idempotent guard must use trim_end().ends_with not starts_with —
    //   suffix anchors at the end; starts_with would miss re-injection on existing suffixes.
    let effective_msg = if cli.no_ultrathink || msg.trim_end().ends_with( "ultrathink" )
    {
      msg.clone()
    }
    else
    {
      format!( "{msg}\n\nultrathink" )
    };
    builder = builder.with_message( effective_msg );
  }
  if let Some( ref fmt ) = cli.output_format
  {
    // Path A (legacy alias): "summary" is intercepted by the runner; forward "json" to claude.
    let forwarded = if fmt == "summary" { "json" } else { fmt.as_str() };
    builder = builder.with_arg( "--output-format" ).with_arg( forwarded );
  }
  else if use_print
  {
    // Path B (auto-inject): when rendering summary and no --output-format is set, inject
    // --output-format json so claude returns parseable JSON for render_summary().
    let effective_style = cli.output_style.as_deref().unwrap_or( "summary" );
    if effective_style == "summary" || cli.json_schema.is_some()
    {
      builder = builder.with_arg( "--output-format" ).with_arg( "json" );
    }
  }
  if let Some( fmt ) = cli.input_format
  {
    builder = builder.with_input_format( fmt );
  }
  if let Some( ref turns ) = cli.max_turns
  {
    builder = builder.with_arg( "--max-turns" ).with_arg( turns.as_str() );
  }
  if let Some( ref tools ) = cli.allowed_tools
  {
    builder = builder.with_arg( "--allowed-tools" ).with_arg( tools.as_str() );
  }
  if let Some( ref tools ) = cli.disallowed_tools
  {
    builder = builder.with_arg( "--disallowed-tools" ).with_arg( tools.as_str() );
  }
  if let Some( ref budget ) = cli.max_budget_usd
  {
    builder = builder.with_arg( "--max-budget-usd" ).with_arg( budget.as_str() );
  }
  if let Some( ref dir ) = cli.add_dir
  {
    builder = builder.with_arg( "--add-dir" ).with_arg( dir.as_str() );
  }
  if let Some( ref model ) = cli.fallback_model
  {
    builder = builder.with_arg( "--fallback-model" ).with_arg( model.as_str() );
  }
  if cli.no_compact_window
  {
    builder = builder.with_compact_window( None );
  }

  // Hand the fork plan downstream (dry-run preview, registry recording) only when
  // the fork/resume arguments were actually injected — a gated-off invocation
  // (print mode, nothing following as a prompt) runs bare, so previewing or
  // registering a topic session that will not be touched would misstate the run.
  // The gating must happen HERE, after every internal use: the working-dir and
  // transplant decisions above key off the original plan (fork mode never creates
  // a topic dir and never transplants, injected or not).
  let fork = if resume_allowed { fork } else { None };

  (
    builder,
    expected_id,
    RunPreparation
    {
      effective_working_dir,
      transplant,
      fork,
    },
  )
}

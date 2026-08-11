//! Execution Mode Tests — Extended: new-flag tests and BUG-424–BUG-428 reproducers (Unix-only)
#![ cfg( unix ) ]
//!
//! Extension of `execution_mode_test.rs` (E01–E14) covering `--strip-fences`,
//! `--keep-claudecode`, `--file`, piped-stdin routing, `-c` injection rules,
//! and the chrome-suppression / retry-on-transient-failure reproducers.
//!
//! ## Strategy
//!
//! Uses fake `claude` shell scripts injected via PATH manipulation, same as
//! `execution_mode_test.rs`. See that file for the shared strategy notes.


mod cli_binary_test_helpers;
use cli_binary_test_helpers::{ fake_claude, fake_claude_dir, make_session_dir, make_zero_turn_session_dir, run_dry, run_with_path, run_with_path_stdin };


// BUG-425: completely empty piped stdin (`echo -n "" | clr`), no message, must also
// route to print mode — `detect_stdin_json()` always returns `Some(StdinPayload::Raw(_))`
// once past the TTY/--file gates regardless of length, so `cli.stdin_content` is
// `Some(vec![])` here, not `None`. Distinct from the T01 non-empty case above; does NOT
// independently isolate the TTY-check term (see Plan 005 Phase 1 Known Coverage Gap).
// test_kind: bug_reproducer(BUG-425)
//
// ## Root Cause
// Same formula gap as the non-empty case, but for the boundary of zero-byte piped
// stdin: `detect_stdin_json()` returns `Some(StdinPayload::Raw(vec![]))` — not
// `None` — once stdin is confirmed non-TTY, so `cli.stdin_content.is_some()` is
// true here even though there is no real prompt content behind it.
//
// ## Why Not Caught
// No existing test piped literally zero bytes — prior cases piped either real
// content or used a fully-closed/absent stdin handled by a different path, leaving
// this exact boundary (piped, connected, but empty) unexercised.
//
// ## Fix Applied
// The TTY-check term (`!is_terminal`) independently routes this case to print
// mode, since it does not depend on content length — it fires on stdin's
// terminal-ness alone, unaffected by whether any bytes were actually piped.
//
// ## Prevention
// When a presence-check formula uses `.is_some()` on a value that can be
// `Some(empty)`, treat "present but empty" as its own case requiring a dedicated
// test — `.is_some()` is not the same claim as "has usable content."
//
// ## Pitfall
// This test's PASS is attributable to the TTY-check term, not the stdin-content
// term (which is also `Some` here but carries no real content) — it does not
// independently isolate which term is doing the work. See Plan 005 Phase 1's
// Known Coverage Gap.
#[ test ]
fn bug_reproducer_425_empty_stdin_no_tty()
{
  let args_file      = tempfile::NamedTempFile::new().expect( "create args file" );
  let args_path      = args_file.path().display().to_string();
  let script         = format!( "echo \"$@\" > \"{args_path}\"\n" );
  let ( _tmp, path ) = fake_claude_dir( &script );
  let out = run_with_path_stdin(
    &[ "--max-sessions", "0" ],
    &path,
    b"",
  );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let received = std::fs::read_to_string( args_file.path() ).expect( "read args file" );
  assert!(
    received.contains( "--print" ),
    "empty piped stdin with no message must still route to print mode (--print in subprocess args). Received: {received}",
  );
}

// BUG-426: `-c` injection is coupled only to prior-session existence, not to
// message/--print/--file/--interactive presence — a bare `--session-dir <dir>`
// invocation with no message and no --print still gets `-c` injected today, even
// though nothing in the invocation supplies content to resume. Verified via
// --dry-run (composed args inspectable without a live subprocess).
// test_kind: bug_reproducer(BUG-426)
//
// ## Root Cause
// `-c`/session-continuation injection was gated solely on `expected_id.is_some()`
// (a prior session existing on disk) — it never checked whether the current
// invocation carried a message, `--print`, `--file`, or stdin content, so a bare
// `--session-dir <dir-with-prior-session>` invocation would compose `-c` with
// nothing for Claude to resume with.
//
// ## Why Not Caught
// Existing composition tests either supplied a message (masking the gap, since
// `-c` alongside a message is valid) or used a session-less directory (where
// `expected_id.is_none()` short-circuits the check) — no test held prior-session
// presence and message-absence constant at the same time.
//
// ## Fix Applied
// The `-c` injection condition now requires `expected_id.is_some()` AND at least
// one of `{cli.message, cli.print_mode, cli.file, cli.stdin_content,
// cli.interactive}` — suppressing injection when none of the content/mode signals
// are present.
//
// ## Prevention
// Any composition step that reads from prior state (a session directory, a cache,
// a config default) must also validate that the CURRENT invocation supplies
// content compatible with that state — presence of old state alone is never
// sufficient justification.
//
// ## Pitfall
// The `cli.interactive` term must remain in this disjunct even though it looks
// redundant next to the content-presence terms — see
// `bug_reproducer_426_interactive_resume_unaffected` (T09) for the regression this
// exact term guards against.
#[ test ]
fn bug_reproducer_426_c_injected_without_message()
{
  let ( _session, session_path ) = make_session_dir();
  let output = run_dry( &[ "--session-dir", &session_path ] );
  assert!(
    !output.contains( " -c" ),
    "-c must not be injected when no message/--print/--file/--interactive is given, \
     even with a prior session present. Got:\n{output}",
  );
}

// BUG-426 (T09, non-regression): an explicit `--interactive` resume with no message
// and a prior session present must still inject `-c` — this is the one use of
// unconditional -c-with-no-message that BUG-426's own Root Cause section excludes
// from its claimed defect scope. The gate's `cli.interactive` escape term exists
// specifically to keep this path working; this test guards against a future edit
// dropping that term as apparently-redundant alongside the message-presence terms.
// test_kind: bug_reproducer(BUG-426)
//
// ## Root Cause
// This documents a regression this test guards against, not a defect: a naive fix
// for BUG-426 — gating `-c` injection on message/print/file/stdin-content presence
// alone — would also suppress `-c` for an explicit `--interactive` resume with no
// new message, since that invocation carries none of those content signals either.
// But that is the intended attach-and-type-live interactive resume flow, not the
// defect BUG-426 describes.
//
// ## Why Not Caught
// In a naive fix, the content-presence terms alone look sufficient because they
// correctly describe the print-mode composition cases; the interactive-resume case
// is easy to overlook precisely because it's the one legitimate scenario where `-c`
// with no message is correct, not a defect.
//
// ## Fix Applied
// The `-c` injection condition's disjunct explicitly includes `cli.interactive`
// alongside the content-presence terms, so an explicit `--interactive` flag alone
// (independent of message/print/file/stdin-content) still satisfies the gate.
//
// ## Prevention
// Whenever a bug's Root Cause names "the one legitimate case where X is normally
// true" as an exclusion, that exclusion needs its own permanent regression test —
// the natural-looking fix for the bug is exactly the change that would break it.
//
// ## Pitfall
// Do not remove the `cli.interactive` term from the `-c`-injection disjunct as
// "apparently redundant" alongside the other content-presence terms — it is the
// one term with no content-signal justification, and exists solely to preserve
// this flow.
#[ test ]
fn bug_reproducer_426_interactive_resume_unaffected()
{
  let ( _session, session_path ) = make_session_dir();
  let output = run_dry( &[ "--session-dir", &session_path, "--interactive" ] );
  assert!(
    output.contains( " -c" ),
    "-c must still be injected for an explicit --interactive resume with no message \
     — this is BUG-426's own excluded case (T09), not part of the defect. Got:\n{output}",
  );
}

// BUG-427: the `--print`-requires-a-message guard rejects `--print --file <path>`
// even though `--file` supplies the content that would serve as the prompt — the
// guard checks `cli.message.is_none()` only, blind to `cli.file`/stdin presence.
// test_kind: bug_reproducer(BUG-427)
//
// ## Root Cause
// The no-message validation guard fired whenever
// `cli.print_mode && cli.message.is_none()`, treating "no positional message" as
// synonymous with "no content to print" — but `--file <path>` supplies content
// just as validly as a positional message, so `--print --file <path>` was rejected
// with a "requires a message argument" error despite having real content to send.
//
// ## Why Not Caught
// Existing tests for this guard covered the true-positive case (print mode
// requested, genuinely no content at all → correctly rejected) and the
// message-present case (correctly accepted) — no test combined explicit `--print`
// with `--file` and no positional message, the exact combination this guard
// mishandled.
//
// ## Fix Applied
// The guard's condition was extended to
// `cli.print_mode && cli.message.is_none() && cli.file.is_none() && !has_stdin_content`,
// so `--file` (or non-empty piped stdin) now satisfies the content requirement
// alongside a positional message.
//
// ## Prevention
// A validation guard framed as "requires X" must enumerate every value accepted
// as a valid X — a guard checking only the most common source of content (a
// positional argument) will incorrectly reject other equally-valid sources (a
// file, piped stdin) unless it explicitly names them too.
//
// ## Pitfall
// This guard is orthogonal to the mode-selection formula
// (`is_print_invocation`/`use_print`) — it only fires once `cli.print_mode` is
// already true (explicit `-p`/`--print`, env, or config); a bare invocation that
// reaches print mode implicitly via the TTY-check term alone never hits this guard
// at all (see D3 in `docs/001_design_decisions.md`).
#[ test ]
fn bug_reproducer_427_file_without_message_rejected()
{
  let script = "#!/bin/sh\ncat\n";
  let ( _tmp, path ) = fake_claude( script );
  let input_file = tempfile::NamedTempFile::new().expect( "create temp" );
  std::fs::write( input_file.path(), "content_from_file_bug427" ).expect( "write" );
  let out = run_with_path(
    &[ "--max-sessions", "0", "--print", "--file", input_file.path().to_str().unwrap() ],
    &path,
  );
  assert!(
    out.status.success(),
    "--file must satisfy --print's message requirement with no positional message. stderr: {}",
    String::from_utf8_lossy( &out.stderr ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "content_from_file_bug427" ),
    "--file content must be forwarded to subprocess once the guard accepts it. Got:\n{stdout}",
  );
}

// BUG-427: bare `--file <path>` with no message and no --print incorrectly routes
// to the interactive REPL under this harness's actual non-TTY piped stdin — the
// mode-selection formula (`is_print_invocation`) has no file/stdin-content term at
// all today, only a message/print/interactive term.
// test_kind: bug_reproducer(BUG-427)
//
// ## Root Cause
// `is_print_invocation`/`use_print` had no term at all covering
// `cli.file`/`cli.stdin_content` presence — a bare `--file <path>` invocation with
// no message and no explicit `--print` fell through to `run_interactive()`
// regardless of TTY state, even though `--file` supplies real content that should
// route to print-mode dispatch instead.
//
// ## Why Not Caught
// Prior tests either supplied `--print` explicitly alongside `--file` (masking the
// gap, since `cli.print_mode` alone was already sufficient) or used a message
// instead of `--file` — no test isolated "file content only, no message, no
// explicit print flag."
//
// ## Fix Applied
// `is_print_invocation`/`use_print` gained a
// `cli.file.is_some() || cli.stdin_content.is_some()` disjunct, so `--file` alone
// (independent of the TTY-check term) now satisfies print-mode routing.
//
// ## Prevention
// When adding a new disjunct to an existing OR-formula, write at least one test
// that isolates that specific disjunct from the others already in the formula —
// otherwise a future refactor merging two disjuncts (e.g. `||` collapsed to `&&`)
// can silently narrow the formula without any test noticing.
//
// ## Pitfall
// This test alone does not prove the file/stdin-content term and the TTY term are
// independently OR'd rather than coupled — under this harness's non-TTY subprocess
// stdin, the TTY-check term would ALSO independently route this exact invocation
// to print mode. A future refactor collapsing `||` to `&&` between them would not
// be caught by this test alone (see Plan 005 Phase 1's Known Coverage Gap).
#[ test ]
fn bug_reproducer_427_file_only_no_tty_routes_print()
{
  let args_file      = tempfile::NamedTempFile::new().expect( "create args file" );
  let args_path      = args_file.path().display().to_string();
  let script         = format!( "echo \"$@\" > \"{args_path}\"\n" );
  let ( _tmp, path ) = fake_claude_dir( &script );
  let input_file = tempfile::NamedTempFile::new().expect( "create temp" );
  std::fs::write( input_file.path(), "file_only_content_bug427" ).expect( "write" );
  let out = run_with_path(
    &[ "--max-sessions", "0", "--file", input_file.path().to_str().unwrap() ],
    &path,
  );
  assert!( out.status.success(), "must exit 0. stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let received = std::fs::read_to_string( args_file.path() ).expect( "read args file" );
  assert!(
    received.contains( "--print" ),
    "bare --file with no message/--print must route to print mode under non-TTY stdin. Received: {received}",
  );
}

// Fix(BUG-425) regression confirmation: chrome suppression already holds for a
// non-TTY, file-only invocation via Phase 2's `use_print` term flowing into the
// existing `cli.no_chrome || use_print` gate — no source edit accompanies this test.
// Root cause: prior to BUG-425's fix, this exact invocation shape (reusing Test
//   Matrix T06's `--file`-only setup) fell through to the interactive REPL, where
//   chrome injection follows a different code path entirely; confirming chrome
//   suppression here is only meaningful once print-mode routing itself is fixed.
// Pitfall: this test intentionally does NOT add a new `!stdin_is_terminal` term to
//   `builder.rs` — Phase 2's `use_print` already carries it, so an equivalent-looking
//   second term would be pure duplication, not defense in depth.
#[ test ]
fn chrome_suppression_holds_for_non_tty_file_only_invocation()
{
  let input_file = tempfile::NamedTempFile::new().expect( "create temp" );
  std::fs::write( input_file.path(), "file_content_for_chrome_suppression_check" ).expect( "write" );
  let stdout = run_dry( &[ "--file", input_file.path().to_str().unwrap() ] );
  assert!(
    !stdout.contains( "--chrome" ),
    "file-only invocation under non-TTY stdin must route to print mode, suppressing --chrome. Got:\n{stdout}"
  );
}

// BUG-428 (T01, Plan 006 Phase 1 — TDD Red / Phase 2 — Green): a `.jsonl` transcript that
// structurally qualifies as a resume candidate (correct extension, non-`agent-`-prefixed,
// non-zero size, valid UTF-8 stem — `most_recent_session_in_dir()`'s 4 checks) but recorded
// zero model turns causes claude's real `--resume` logic to reject it with "No conversation
// found to continue" (contract/claude_code/docs/version/088_v2_1_187.md:19) — `clr` had no
// reactive fallback for this signature and surfaced the raw rejection, misattributed as its
// own defect, with no retry attempted. This test asserts the FIXED end state (retry without
// `-c` fires, succeeds, and a clr-authored diagnostic explains the fallback).
// test_kind: bug_reproducer(BUG-428)
//
// ## Root Cause
// `session_exists()`/`most_recent_session_in_dir()` (`builder.rs:25-45`,
// `claude_storage_core/src/continuation.rs:109-179`) qualify a resume candidate using four
// purely structural properties (extension, `agent-` prefix, size, UTF-8 stem) and never read
// file content; `build_claude_command()` treats a `Some(SessionId)` result as unconditionally
// sufficient to inject `-c`. Claude's own `--resume` logic is independently content-aware and
// rejects a transcript whose originating run produced zero model turns. One retry is
// definitionally sufficient, not an arbitrary bound: dropping `-c` produces a fresh-session
// invocation, which cannot itself trigger a resume-rejection, since no resume is attempted on
// that attempt (Plan 006 Phase 2 Project Context).
//
// ## Why Not Caught
// The test suite for `most_recent_session_in_dir`/`most_recent_session_id`
// (`continuation_tests.rs:205-275`) thoroughly covers every structural dimension the function
// checks, but every test writes the identical placeholder content `b"{}"` regardless of which
// structural dimension it exercises — no test ever varies file content, so a suite otherwise
// complete against the function's own documented structural rules cannot surface a gap that
// depends on claude's own external, content-based acceptance criterion.
//
// ## Fix Applied
// `run_print_mode()` (`execution.rs`) gained a `RESUME_REJECTED_MARKER` constant and a
// one-shot detection block (`fallback_builder.is_none()` gate, checked before
// `classify_error()`): on match, emits an unconditional stderr diagnostic, builds
// `active.clone().with_continue_conversation(false)`, and retries once.
//
// ## Prevention
// A local qualification predicate that selects a candidate on behalf of an external system
// must be verified against that external system's own documented acceptance criteria, not
// assumed equivalent because it operates on the same class of artifact — see BUG-428's own
// Generalized Version: the gap is live regardless of how complete the predicate's own
// structural coverage looks in isolation, whenever the external system's real criterion
// depends on content the local predicate never inspects.
//
// ## Pitfall
// `RESUME_REJECTED_MARKER`'s detection block shares its one-shot `fallback_builder` gate with
// the pre-existing `DEFERRED_TOOL_MARKER` block (BUG-327) — the two are mutually exclusive
// per attempt by construction (whichever `if` matches first in source order fires; the other
// is skipped that iteration), not by an independent runtime check. See
// `bug_reproducer_428_fallback_also_rejected_falls_through` for the one-shot gate's own
// double-rejection boundary.
#[ test ]
fn bug_reproducer_428_resume_rejected_no_retry()
{
  let ( _session, session_path ) = make_zero_turn_session_dir();
  // Rejects any invocation carrying `-c` (simulating claude's real refusal to resume a
  // zero-model-turn transcript); succeeds once `-c` is dropped, simulating the
  // fresh-session fallback the fix must perform.
  let script = "#!/bin/sh\nfor a in \"$@\"; do\n  if [ \"$a\" = \"-c\" ]; then\n    echo 'No conversation found to continue' >&2\n    exit 1\n  fi\ndone\necho FRESH_SESSION_OK\nexit 0\n";
  let ( _tmp, path ) = fake_claude( script );
  let out = run_with_path(
    &[ "--session-dir", &session_path, "--retry-override", "0", "--max-sessions", "0", "test message" ],
    &path,
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    out.status.success(),
    "a resume-rejected session must fall back to a fresh session and succeed \
     (unfixed: raw rejection surfaces and the process exits non-zero). stderr:\n{stderr}",
  );
  assert!(
    stdout.contains( "FRESH_SESSION_OK" ),
    "the fallback retry without -c must actually run and its output must reach the user. Got stdout:\n{stdout}",
  );
  assert!(
    stderr.contains( "was not resumable" ),
    "clr must emit its own diagnostic explaining the fallback \
     (unfixed: claude's raw, uncontextualized rejection surfaces instead). Got stderr:\n{stderr}",
  );
}

// BUG-428 (T02, Plan 006 Phase 2 — TDD Green): the resume-rejection fallback drops `-c`
// (`with_continue_conversation(false)`) rather than substituting the message, unlike
// BUG-327's `DeferredToolMarker` fallback. The journaled Execution event's `message` field
// must therefore still equal the ORIGINAL `cli.message`, never `FALLBACK_MESSAGE`
// ("Continue.") — proving `FallbackReason` correctly withholds the substitution for this
// cause. See execution.rs's `FallbackReason` enum and `fallback_note` computation.
// test_kind: bug_reproducer(BUG-428)
//
// ## Root Cause
// Same structural-vs-content qualification gap as
// `bug_reproducer_428_resume_rejected_no_retry` — see that test's Root Cause. This test
// isolates a distinct correctness risk introduced by the fix itself: `fallback_builder` is
// shared with BUG-327's `DeferredToolMarker` cause, whose own fallback substitutes
// `FALLBACK_MESSAGE` ("Continue.") for the journaled message. Without a way to distinguish
// which cause fired, a naive shared-slot implementation would misjournal every
// resume-rejection retry with the wrong message.
//
// ## Why Not Caught
// Prior to this fix, no code path substituted a message for a resume-rejection retry at
// all (the retry mechanism itself didn't exist), so no test could have exercised a
// mis-journaling defect — this is a regression risk specific to the fix's own
// implementation choice (a shared one-shot slot for two distinct fallback causes), not a
// pre-existing gap.
//
// ## Fix Applied
// `FallbackReason { DeferredToolMarker, ResumeRejected }` is paired with `fallback_builder`'s
// built command; `fallback_note`'s computation (`execution.rs`, just before the retry loop)
// only yields `Some(FALLBACK_MESSAGE)` when the paired reason is `DeferredToolMarker` — for
// `ResumeRejected` it yields `None`, so `emit_execution()` journals the original
// `cli.message` unchanged.
//
// ## Prevention
// When two independent one-shot fallback mechanisms share a single state slot, any
// downstream logic keyed off "is the slot occupied" must instead be keyed off "which cause
// occupied it" — collapsing distinct causes into a single boolean silently conflates their
// otherwise-different side effects (here: message substitution).
//
// ## Pitfall
// This test's journal assertions require `--journal full --journal-dir <tmp>` — omitting
// either flag would either suppress the execution event entirely or omit the `message`
// field the assertions depend on (per `--journal meta`'s field-omission behavior).
#[ test ]
fn bug_reproducer_428_retry_succeeds()
{
  const ORIGINAL_MESSAGE : &str = "distinct-original-msg-428";

  let ( _session, session_path ) = make_zero_turn_session_dir();
  let jdir = tempfile::TempDir::new().expect( "failed to create temp journal dir" );
  let jdir_s = jdir.path().to_str().expect( "journal dir path must be valid UTF-8" );
  let script = "#!/bin/sh\nfor a in \"$@\"; do\n  if [ \"$a\" = \"-c\" ]; then\n    echo 'No conversation found to continue' >&2\n    exit 1\n  fi\ndone\necho FRESH_SESSION_OK\nexit 0\n";
  let ( _tmp, path ) = fake_claude( script );
  let out = run_with_path(
    &[
      "--session-dir", &session_path, "--retry-override", "0", "--max-sessions", "0",
      "--journal", "full", "--journal-dir", jdir_s,
      ORIGINAL_MESSAGE,
    ],
    &path,
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    out.status.success(),
    "retry after resume-rejection must succeed. stderr:\n{stderr}",
  );
  assert!(
    stderr.contains( "was not resumable" ),
    "clr must emit its resume-rejection diagnostic. Got stderr:\n{stderr}",
  );

  let content : String = std::fs::read_dir( jdir.path() )
    .expect( "failed to read journal dir" )
    .filter_map( Result::ok )
    .map( | e | std::fs::read_to_string( e.path() ).unwrap_or_default() )
    .collect();
  assert!(
    content.contains( &format!( "\"message\":\"{ORIGINAL_MESSAGE}\"" ) ),
    "journaled message must equal the ORIGINAL cli.message — BUG-428's fallback drops -c \
     but never substitutes the message. Got journal:\n{content}",
  );
  assert!(
    !content.contains( "\"message\":\"Continue.\"" ),
    "journaled message must NOT be substituted with BUG-327's FALLBACK_MESSAGE — that \
     substitution is scoped to FallbackReason::DeferredToolMarker only. Got journal:\n{content}",
  );
}

// BUG-428 (T03, Plan 006 Phase 2 — regression guard): a session that is genuinely
// resumable (fake claude accepts `-c` and succeeds, simulating claude's real behavior on a
// transcript with actual model turns) must be completely unaffected by the new
// resume-rejection fallback — `-c` is injected normally, no retry fires, and no fallback
// diagnostic appears.
// test_kind: bug_reproducer(BUG-428)
//
// ## Root Cause
// N/A — this is a regression guard, not a reproducer of the defect itself. It confirms the
// fix in `bug_reproducer_428_resume_rejected_no_retry` does not over-trigger: a session
// claude genuinely accepts must see `-c` injected and succeed in one attempt, exactly as
// before this fix existed.
//
// ## Why Not Caught
// N/A — no defect exists here; this test exists to prevent one from being introduced by a
// future, over-broad edit to the `RESUME_REJECTED_MARKER` detection condition.
//
// ## Fix Applied
// N/A — no fix; the detection block's signature check
// (`output.stdout`/`output.stderr.contains(RESUME_REJECTED_MARKER)`) never matches when
// claude accepts the resume, so the existing one-attempt success path is untouched.
//
// ## Prevention
// Every new one-shot detection/retry mechanism gated on subprocess output must be paired
// with a "does not fire on the ordinary success path" test — the absence of a signal is as
// important to verify as its presence.
//
// ## Pitfall
// `make_session_dir()`'s placeholder content (`b"{}"`) is never semantically a "genuine"
// transcript with real model turns — "genuineness" here is entirely simulated by the fake
// claude script always succeeding regardless of `-c`, since `clr` itself never inspects
// session file content (that is precisely BUG-428's own root cause).
#[ test ]
fn bug_reproducer_428_genuine_resume_unaffected()
{
  let ( _session, session_path ) = make_session_dir();
  // Always succeeds regardless of -c, simulating a genuinely resumable session.
  let script = "#!/bin/sh\nfor a in \"$@\"; do\n  if [ \"$a\" = \"-c\" ]; then\n    echo GOT_DASH_C\n  fi\ndone\nexit 0\n";
  let ( _tmp, path ) = fake_claude( script );
  let out = run_with_path(
    &[ "--session-dir", &session_path, "--retry-override", "0", "--max-sessions", "0", "test message" ],
    &path,
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    out.status.success(),
    "a genuinely resumable session must succeed normally. stderr:\n{stderr}",
  );
  assert!(
    stdout.contains( "GOT_DASH_C" ),
    "-c must still be injected normally for a genuinely resumable session. Got stdout:\n{stdout}",
  );
  assert!(
    !stderr.contains( "was not resumable" ),
    "no resume-rejection diagnostic may fire when claude never rejects the resume. Got stderr:\n{stderr}",
  );
}

// BUG-428 (T04, Plan 006 Phase 2 — scope guard): an unrelated claude failure during -c
// resume (a distinct error string that does not match RESUME_REJECTED_MARKER) must NOT
// trigger the resume-rejection fallback — detection is scoped to the specific signature,
// never a catch-all-and-retry. The original failure must surface unmodified.
// test_kind: bug_reproducer(BUG-428)
//
// ## Root Cause
// N/A — this is a scope guard, not a reproducer. It confirms `RESUME_REJECTED_MARKER`
// detection is scoped to its exact signature string, not a catch-all for any print-mode
// failure during a resumed session.
//
// ## Why Not Caught
// N/A — no defect exists here; this test exists to prevent a future edit from loosening the
// detection condition (e.g. matching on exit code alone, or a broader substring) into an
// over-broad catch-all-and-retry.
//
// ## Fix Applied
// N/A — no fix; `.contains(RESUME_REJECTED_MARKER)` does not match an unrelated error
// string, so the failure falls through unchanged to the pre-existing
// `classify_error()`/retry-count path exactly as it did before this plan.
//
// ## Prevention
// Every signature-string detection block needs a paired "distinct, non-matching failure"
// test — proving a detection condition triggers on its target string is only half the
// claim; proving it does NOT trigger on other strings is the other half.
//
// ## Pitfall
// This test disables the generic retry path via `--retry-override 0` so the unrelated
// failure's exit code propagates on the first attempt — without it, the test would still
// pass but only after exhausting the (slower, delay-bearing) generic retry budget first.
#[ test ]
fn bug_reproducer_428_unrelated_failure_no_overbroad_retry()
{
  let ( _session, session_path ) = make_session_dir();
  let script = "#!/bin/sh\necho 'some unrelated claude failure xyz' >&2\nexit 7\n";
  let ( _tmp, path ) = fake_claude( script );
  let out = run_with_path(
    &[ "--session-dir", &session_path, "--retry-override", "0", "--max-sessions", "0", "test message" ],
    &path,
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert_eq!(
    out.status.code(), Some( 7 ),
    "an unrelated failure must relay its original exit code unmodified. Got: {:?} stderr:\n{stderr}",
    out.status.code(),
  );
  assert!(
    stderr.contains( "some unrelated claude failure xyz" ),
    "the original failure text must surface unmodified. Got stderr:\n{stderr}",
  );
  assert!(
    !stderr.contains( "was not resumable" ),
    "no resume-rejection diagnostic may fire for an unrelated failure. Got stderr:\n{stderr}",
  );
}

// BUG-428 (fallback also rejected, Plan 006 Phase 2 — one-shot gate guard): if BOTH the
// original -c attempt AND the post-fallback (no -c) attempt are rejected with the identical
// RESUME_REJECTED_MARKER signature (an edge case unrelated to -c itself), the one-shot gate
// (`fallback_builder.is_none()`) must prevent a second retry — the run falls through to the
// generic classify_error()/retry-count path on the second rejection and terminates with the
// subprocess's own exit code, never looping forever. Exactly one "was not resumable"
// diagnostic must appear (from the first, successful gate entry) — a second occurrence
// would mean the gate re-fired.
// test_kind: bug_reproducer(BUG-428)
//
// ## Root Cause
// N/A — this is a one-shot-gate boundary test, not a reproducer. It confirms that when
// even the post-fallback (no-`-c`) attempt is rejected with the identical
// `RESUME_REJECTED_MARKER` signature (an edge case unrelated to `-c` itself — e.g. a
// corrupted account-level or environment issue), the shared one-shot `fallback_builder`
// gate — already `Some` from the first attempt — prevents the block from firing a second
// time, so the run falls through to generic `classify_error()`-based termination instead of
// retrying forever.
//
// ## Why Not Caught
// N/A — no defect exists; this is the single highest-risk untested path this plan's own
// design review converged on (Plan 006 Phase 2 Deliverables, AGG-2) and is added
// proactively, not in response to an observed failure.
//
// ## Fix Applied
// N/A — no fix; `fallback_builder.is_none()`'s existing one-shot gate (shared with
// `DEFERRED_TOOL_MARKER`'s own mechanism, BUG-327) already prevents re-entry by
// construction — this test proves that construction holds under a double-rejection
// fixture rather than merely asserting it by code review.
//
// ## Prevention
// A one-shot fallback gate's safety under "the fallback itself also fails" must be proven
// by a dedicated fixture, not inferred from the gate's own structure — an off-by-one error
// in the gate condition (e.g. checking the wrong flag, or re-arming it on retry) would only
// surface under this exact double-rejection scenario, never under the single-rejection
// happy path the other tests exercise.
//
// ## Pitfall
// The fake claude script here rejects unconditionally (regardless of `-c` presence) — this
// is deliberately different from `bug_reproducer_428_resume_rejected_no_retry`'s script
// (which only rejects when `-c` is present); using the same conditional script here would
// make the second (no-`-c`) attempt succeed, defeating the double-rejection scenario this
// test exists to cover.
#[ test ]
fn bug_reproducer_428_fallback_also_rejected_falls_through()
{
  let ( _session, session_path ) = make_zero_turn_session_dir();
  // Rejects unconditionally regardless of -c presence, simulating an edge case where even
  // the post-fallback fresh-session attempt is rejected with the identical signature.
  let script = "#!/bin/sh\necho 'No conversation found to continue' >&2\nexit 1\n";
  let ( _tmp, path ) = fake_claude( script );
  let out = run_with_path(
    &[ "--session-dir", &session_path, "--retry-override", "0", "--max-sessions", "0", "test message" ],
    &path,
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    !out.status.success(),
    "a double rejection must ultimately fail (not loop forever or spuriously succeed). stderr:\n{stderr}",
  );
  assert_eq!(
    out.status.code(), Some( 1 ),
    "must terminate with the subprocess's own exit code via the generic classify_error() \
     path once the one-shot gate is spent. Got: {:?} stderr:\n{stderr}",
    out.status.code(),
  );
  let occurrences = stderr.matches( "was not resumable" ).count();
  assert_eq!(
    occurrences, 1,
    "the one-shot gate must fire exactly once even when both attempts are rejected — a \
     second occurrence means the gate re-fired instead of falling through. Got stderr:\n{stderr}",
  );
}

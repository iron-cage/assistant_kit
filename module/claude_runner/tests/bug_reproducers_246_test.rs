//! Bug reproducer for BUG-246: CLAUDECODE removal invisible in trace/dry-run output.
//!
//! # Root Cause (BUG-246)
//!
//! `ClaudeCommand::describe()` started with `"claude"` unconditionally. At the same
//! time, `ClaudeCommand::new()` defaults `unset_claudecode = true`, which causes
//! `build_command()` to call `cmd.env_remove("CLAUDECODE")` on the subprocess before
//! spawn. That OS-level removal was invisible in `describe()` output — trace/dry-run
//! showed `claude ...` but the actual invocation was `env -u CLAUDECODE claude ...`.
//!
//! # Why Not Caught
//!
//! All prior trace/dry-run tests only checked that expected flags were present (e.g.
//! `--dangerously-skip-permissions`). None asserted that CLAUDECODE removal appeared
//! in the displayed command, so the WYSIWYG gap went undetected.
//!
//! # Fix Applied
//!
//! `describe()` now starts with `env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION claude ...`
//! when `unset_claudecode` is true (the default). When `--keep-claudecode` is passed,
//! `unset_claudecode` is false and `describe()` starts with
//! `env -u CLAUDE_CODE_CHILD_SESSION claude ...` — `CLAUDE_CODE_CHILD_SESSION` is always
//! stripped regardless of `--keep-claudecode`.
//!
//! `removed_vars()` is the single source of truth for all removals: both `build_command()`
//! and `describe()` iterate it, so any new removal added there propagates to both
//! automatically — a second call site to forget is impossible by construction.
//!
//! # Prevention
//!
//! Add new `env_remove()` requirements to `removed_vars()` — not inline in `build_command()`
//! or `describe()`. The shared list makes trace/execution divergence structurally impossible.
//!
//! # Pitfall
//!
//! `env_remove()` is an OS-level subprocess configuration call that does NOT appear
//! in `Command`'s arg list. It is invisible to any introspection that only looks at
//! argv. Only explicit mirroring in `describe()` (via `removed_vars()`) makes it visible.
//!
//! # Test Matrix
//!
//! | Test | Scenario | Expected |
//! |------|----------|----------|
//! | `dry_run_shows_env_u_claudecode_prefix` | default (unset_claudecode=true) | stdout contains `env -u CLAUDECODE` |
//! | `dry_run_keep_claudecode_omits_env_prefix` | `--keep-claudecode` (unset_claudecode=false) | stdout does NOT contain `env -u CLAUDECODE` |
//! | `dry_run_always_shows_child_session_removal` | default | stdout contains `env -u CLAUDE_CODE_CHILD_SESSION` |
//! | `dry_run_keep_claudecode_still_strips_child_session` | `--keep-claudecode` | stdout still contains `-u CLAUDE_CODE_CHILD_SESSION` |

#![ cfg( feature = "enabled" ) ]

mod cli_binary_test_helpers;
use cli_binary_test_helpers::run_cli;

// ── BUG-246 ──────────────────────────────────────────────────────────────────

/// BUG-246 reproducer T1: default dry-run output must show `env -u CLAUDECODE` prefix.
///
/// Before fix: `describe()` always started with `"claude"` — the CLAUDECODE `env_remove()`
/// in `build_command()` was invisible, making trace/dry-run output non-WYSIWYG.
#[ test ]
#[ doc = "bug_reproducer(BUG-246)" ]
fn dry_run_shows_env_u_claudecode_prefix()
{
  let out = run_cli( &[ "--dry-run", "test" ] );
  assert!(
    out.status.success(),
    "BUG-246: --dry-run must exit 0; got {}\nstderr: {}",
    out.status,
    String::from_utf8_lossy( &out.stderr ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "env -u CLAUDECODE" ),
    "BUG-246: dry-run output must contain 'env -u CLAUDECODE' (default: unset_claudecode=true);\ngot: {stdout}",
  );
}

/// BUG-246 reproducer T2: `--keep-claudecode` suppresses `env -u CLAUDECODE` but
/// `CLAUDE_CODE_CHILD_SESSION` is still stripped.
///
/// When `--keep-claudecode` is passed, `unset_claudecode = false` and `describe()` starts
/// with `env -u CLAUDE_CODE_CHILD_SESSION claude ...` (not plain `claude ...` and not
/// `env -u CLAUDECODE ...`). `CLAUDE_CODE_CHILD_SESSION` is always stripped unconditionally.
#[ test ]
#[ doc = "bug_reproducer(BUG-246)" ]
fn dry_run_keep_claudecode_omits_env_prefix()
{
  let out = run_cli( &[ "--dry-run", "--keep-claudecode", "test" ] );
  assert!(
    out.status.success(),
    "BUG-246: --dry-run --keep-claudecode must exit 0; got {}\nstderr: {}",
    out.status,
    String::from_utf8_lossy( &out.stderr ),
  );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    !stdout.contains( "env -u CLAUDECODE" ),
    "BUG-246: --keep-claudecode must suppress 'env -u CLAUDECODE' in dry-run output;\ngot: {stdout}",
  );
  // Sanity: the claude invocation must still appear
  assert!(
    stdout.contains( "claude" ),
    "BUG-246: dry-run output must still contain 'claude' with --keep-claudecode;\ngot: {stdout}",
  );
}

/// CLAUDE_CODE_CHILD_SESSION is always stripped — it must appear in dry-run output by default.
///
/// The marker causes Claude Code to skip transcript saving when inherited. `clr` is always
/// a top-level launcher and must strip it unconditionally so the spawned Claude session
/// doesn't inherit the warning.
#[ test ]
fn dry_run_always_shows_child_session_removal()
{
  let out = run_cli( &[ "--dry-run", "test" ] );
  assert!( out.status.success(), "must exit 0; stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "-u CLAUDE_CODE_CHILD_SESSION" ),
    "dry-run output must show CLAUDE_CODE_CHILD_SESSION removal;\ngot: {stdout}",
  );
}

/// CLAUDE_CODE_CHILD_SESSION is still stripped even when `--keep-claudecode` is passed.
///
/// The two removals are independent: `--keep-claudecode` only controls `CLAUDECODE`;
/// `CLAUDE_CODE_CHILD_SESSION` is always unconditionally stripped.
#[ test ]
fn dry_run_keep_claudecode_still_strips_child_session()
{
  let out = run_cli( &[ "--dry-run", "--keep-claudecode", "test" ] );
  assert!( out.status.success(), "must exit 0; stderr: {}", String::from_utf8_lossy( &out.stderr ) );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!(
    stdout.contains( "-u CLAUDE_CODE_CHILD_SESSION" ),
    "dry-run output must still show CLAUDE_CODE_CHILD_SESSION removal with --keep-claudecode;\ngot: {stdout}",
  );
}

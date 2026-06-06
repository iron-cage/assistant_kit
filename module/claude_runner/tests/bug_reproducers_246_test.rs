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
//! `describe()` now starts with `env -u CLAUDECODE claude ...` when `unset_claudecode`
//! is true (the default). When `--keep-claudecode` is passed, `unset_claudecode` is
//! set to false and `describe()` starts with plain `claude ...`.
//!
//! # Prevention
//!
//! Every `env_remove()` call in `build_command()` must be reflected in `describe()`.
//! The two methods must remain in sync — `describe()` is the WYSIWYG contract for
//! all trace/dry-run output.
//!
//! # Pitfall
//!
//! `env_remove()` is an OS-level subprocess configuration call that does NOT appear
//! in `Command`'s arg list. It is invisible to any introspection that only looks at
//! argv. Only explicit mirroring in `describe()` makes it visible.
//!
//! # Test Matrix
//!
//! | Test | Scenario | Expected |
//! |------|----------|----------|
//! | `dry_run_shows_env_u_claudecode_prefix` | default (unset_claudecode=true) | stdout contains `env -u CLAUDECODE` |
//! | `dry_run_keep_claudecode_omits_env_prefix` | `--keep-claudecode` (unset_claudecode=false) | stdout does NOT contain `env -u CLAUDECODE` |

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

/// BUG-246 reproducer T2: `--keep-claudecode` must suppress the `env -u CLAUDECODE` prefix.
///
/// When `--keep-claudecode` is passed, `unset_claudecode = false` and `describe()` must
/// start with plain `claude ...` (no `env -u CLAUDECODE` prefix).
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

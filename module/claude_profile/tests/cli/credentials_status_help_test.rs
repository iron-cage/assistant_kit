//! Integration tests: csh (Credentials Status Help).
//!
//! Verifies that `.credentials.status.help` (the auto-generated per-command help)
//! shows non-empty descriptions for all 13 registered parameters.
//!
//! ## Root Cause Context
//!
//! Parameters registered via `bf(nm)` (the bare boolean lambda in `register_commands()`)
//! produce no description text in help output — users see only the param name and type
//! with a blank line where the description should be.
//! Task 113 adds `with_description()` to the original 9 `.credentials.status` args.
//! Four opt-in params were added later: `display_name`, `role`, `billing`, `model`
//! (→ 13 total); csh01 must cover all 13.
//!
//! ## Why Not Caught
//!
//! No test previously asserted the *content* of `.credentials.status.help`. The help
//! command exists (auto-registered by unilang) and exits 0 regardless — making the
//! blank description problem invisible to the existing test suite.
//!
//! ## Fix Applied
//!
//! Added `bfd(nm, desc)` closure in `register_commands()` (src/lib.rs) that chains
//! `reg_arg_opt(nm, Kind::Boolean).with_description(desc)` — and an inline
//! `.with_description()` on the `format` string arg replacing the bare `fmt()` call.
//!
//! ## Prevention
//!
//! When adding new field-presence params to `.credentials.status`, always use
//! `bfd(nm, desc)` — not the bare `bf(nm)` lambda — to keep help output informative.
//!
//! ## Pitfall
//!
//! `bf(nm)` is still correct for other commands (`.account.save dry`,
//! `.account.use dry`, etc.) where descriptions are not required.
//! Only `.credentials.status` params use `bfd`. Do not replace `bf` globally.
//!
//! ## Test Matrix
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | csh01 | `csh01_help_shows_all_param_descriptions` | `.credentials.status.help` → all 13 param names + description text present | P |

use crate::helpers::{ run_cs, stdout, assert_exit };

// ── csh01 ─────────────────────────────────────────────────────────────────────

/// csh01: `.credentials.status.help` shows all 13 param names with non-empty descriptions.
///
/// Confirms that `with_description()` calls produce visible description text in
/// per-command help output. Without `with_description()`, the unilang Standard-level
/// formatter emits blank lines next to each param name — users can't tell what
/// the params do.
#[ test ]
fn csh01_help_shows_all_param_descriptions()
{
  let out = run_cs( &[ ".credentials.status.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  // Every registered param name must appear in the output.
  // 13 params: 9 original (Task 113) + 4 opt-in fields added later.
  for param in &[
    "format", "account", "sub", "tier", "token", "expires", "email", "file", "saved",
    "display_name", "role", "billing", "model",
  ]
  {
    assert!(
      text.contains( param ),
      ".credentials.status.help must list param `{param}`, got:\n{text}"
    );
  }

  // Verify description text is present — param names alone do not prove descriptions exist.
  assert!(
    text.contains( "opt-in" ),
    ".credentials.status.help must contain 'opt-in' (from file/saved descriptions), got:\n{text}"
  );
  assert!(
    text.contains( "default on" ),
    ".credentials.status.help must contain 'default on' (from boolean field descriptions), got:\n{text}"
  );
  assert!(
    text.contains( "Output format" ),
    ".credentials.status.help must contain 'Output format' (from format description), got:\n{text}"
  );
}

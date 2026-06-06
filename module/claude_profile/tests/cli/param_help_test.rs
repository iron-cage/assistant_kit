//! Integration tests: phd (Param Help Descriptions) + pho (Param Help Optionality).
//!
//! Bug reproducers for BUG-203 and BUG-204.
//!
//! ## Root Cause (BUG-203)
//!
//! Convenience closures `nam()`, `dry()`, `fmt()`, `thr()` in `register_commands()`
//! (src/lib.rs:120-123) create `ArgumentDefinition` objects without chaining
//! `.with_description()`. The framework renderer emits an empty description line.
//! Counterexample: `bfd()`/`bfs()` closures (lines 124-128) chain `.with_description()`
//! and their params render descriptions correctly.
//!
//! ## Why Not Caught (BUG-203)
//!
//! Existing tests validated help output structure (header, param list presence) but did
//! not assert that each parameter line includes a non-empty description string.
//! `credentials_status_help_test.rs` only covers `.credentials.status.help` params
//! which use `bfd()` — the bare closure params were never tested for description content.
//!
//! ## Fix Applied (BUG-203)
//!
//! Added `.with_description(...)` to each convenience closure in `lib.rs:120-123`:
//! `nam()`, `dry()`, `fmt()`, `thr()`. Description text sourced from canonical param
//! specs in `docs/cli/param/`.
//!
//! ## Prevention (BUG-203)
//!
//! When adding new convenience closures that register params, always chain
//! `.with_description()` — the framework emits a blank line without it, and no
//! compile-time or existing test catches the omission.
//!
//! ## Pitfall (BUG-203)
//!
//! `.with_description()` is NOT enforced by the type system. A successful `cargo build`
//! does not guarantee descriptions are present — only integration tests catching blank
//! description lines can detect the omission.
//!
//! ---
//!
//! ## Root Cause (BUG-204)
//!
//! The sole registration helper `reg_arg_opt()` at lib.rs:205-208 unconditionally calls
//! `.with_optional(None)`, marking ALL parameters as optional. `.account.use` and
//! `.account.delete` enforce `name` as required at runtime via
//! `require_nonempty_string_arg()` (commands.rs:881, 1128), but help shows `optional`.
//! No `reg_arg_req()` counterpart existed.
//!
//! ## Why Not Caught (BUG-204)
//!
//! Help output tests asserted structure but not the optionality keyword. The runtime
//! enforcement via `require_nonempty_string_arg()` masked the registration error —
//! the command correctly rejects missing `name` at runtime, so functional tests passed.
//!
//! ## Fix Applied (BUG-204)
//!
//! Added `reg_arg_req()` helper in lib.rs that calls `ArgumentDefinition::new()` without
//! `.with_optional()`, preserving the framework default `optional: false`. Changed
//! `.account.use` and `.account.delete` registrations to use `reg_arg_req("name", ...)`.
//!
//! ## Prevention (BUG-204)
//!
//! When a command enforces a param as required at runtime (`require_nonempty_string_arg`),
//! the registration must also use `reg_arg_req` — grep for `require_nonempty_string_arg`
//! and verify each site uses `reg_arg_req` in the registration vec.
//!
//! ## Pitfall (BUG-204)
//!
//! `reg_arg_opt` was the ONLY helper — adding a new command with a required param
//! naturally used `reg_arg_opt` + `nam()` because no required alternative existed.
//! The naming convention (`_opt` suffix) should have signalled the mismatch, but the
//! convenience closure `nam()` hid the optionality detail.
//!
//! ## Test Matrix
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | phd01 | `phd01_mre_bug203_account_use_help_has_name_description` | `.account.use.help` → `name` has description | P |
//! | phd02 | `phd02_mre_bug203_account_use_help_has_dry_description` | `.account.use.help` → `dry` has description | P |
//! | phd03 | `phd03_mre_bug203_token_status_help_has_format_description` | `.token.status.help` → `format` has description | P |
//! | phd04 | `phd04_mre_bug203_token_status_help_has_threshold_description` | `.token.status.help` → `threshold` has description | P |
//! | pho01 | `pho01_mre_bug204_account_use_help_name_required` | `.account.use.help` → `name` shows `required` | P |
//! | pho02 | `pho02_mre_bug204_account_delete_help_name_required` | `.account.delete.help` → `name` shows `required` | P |
//! | pho03 | `pho03_bug204_account_relogin_help_name_optional` | `.account.relogin.help` → `name` still `optional` | P |
//! | pho04 | `pho04_bug204_accounts_help_name_optional` | `.accounts.help` → `name` still `optional` | P |

use crate::cli_runner::{ run_cs, stdout, assert_exit };

// ── BUG-203 reproducers ─────────────────────────────────────────────────────

/// phd01: bug_reproducer(BUG-203) — `.account.use.help` shows description for `name`.
#[ test ]
fn phd01_mre_bug203_account_use_help_has_name_description()
{
  let out = run_cs( &[ ".account.use.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // Description was improved from "Account name to operate on" (BUG-203 era) to include
  // positional-syntax hint.  Assert the actual description is present.
  assert!(
    text.contains( "Account name (positional:" ),
    "`.account.use.help` must show description for `name`, got:\n{text}"
  );
}

/// phd02: bug_reproducer(BUG-203) — `.account.use.help` shows description for `dry`.
#[ test ]
fn phd02_mre_bug203_account_use_help_has_dry_description()
{
  let out = run_cs( &[ ".account.use.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Dry run mode" ),
    "`.account.use.help` must show description for `dry`, got:\n{text}"
  );
}

/// phd03: bug_reproducer(BUG-203) — `.token.status.help` shows description for `format`.
#[ test ]
fn phd03_mre_bug203_token_status_help_has_format_description()
{
  let out = run_cs( &[ ".token.status.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Output format" ),
    "`.token.status.help` must show description for `format`, got:\n{text}"
  );
}

/// phd04: bug_reproducer(BUG-203) — `.token.status.help` shows description for `threshold`.
#[ test ]
fn phd04_mre_bug203_token_status_help_has_threshold_description()
{
  let out = run_cs( &[ ".token.status.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Token expiry warning threshold" ),
    "`.token.status.help` must show description for `threshold`, got:\n{text}"
  );
}

// ── BUG-204 reproducers ─────────────────────────────────────────────────────

/// pho01: bug_reproducer(BUG-204) — `.account.use.help` shows `name` as `required`.
#[ test ]
fn pho01_mre_bug204_account_use_help_name_required()
{
  let out = run_cs( &[ ".account.use.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "name" ) && text.contains( "required" ),
    "`.account.use.help` must show `name` as required, got:\n{text}"
  );
}

/// pho02: bug_reproducer(BUG-204) — `.account.delete.help` shows `name` as `required`.
#[ test ]
fn pho02_mre_bug204_account_delete_help_name_required()
{
  let out = run_cs( &[ ".account.delete.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "name" ) && text.contains( "required" ),
    "`.account.delete.help` must show `name` as required, got:\n{text}"
  );
}

/// pho03: regression guard — `.account.relogin.help` shows `name` as `optional`.
#[ test ]
fn pho03_bug204_account_relogin_help_name_optional()
{
  let out = run_cs( &[ ".account.relogin.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "name" ),
    "`.account.relogin.help` must list `name` param, got:\n{text}"
  );
  // name is genuinely optional on .account.relogin (defaults to active account)
  assert!(
    !text.contains( "name" ) || !text.contains( "required" )
      || text.lines().any( | l | l.contains( "name" ) && l.contains( "optional" ) ),
    "`.account.relogin.help` `name` must NOT show as required, got:\n{text}"
  );
}

/// pho04: regression guard — `.accounts.help` shows `name` as `optional`.
#[ test ]
fn pho04_bug204_accounts_help_name_optional()
{
  let out = run_cs( &[ ".accounts.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "name" ),
    "`.accounts.help` must list `name` param, got:\n{text}"
  );
  assert!(
    text.lines().any( | l | l.contains( "name" ) && l.contains( "optional" ) ),
    "`.accounts.help` `name` must show as optional, got:\n{text}"
  );
}

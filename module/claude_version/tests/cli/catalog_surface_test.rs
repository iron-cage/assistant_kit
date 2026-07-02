//! Feature tests (FT-6..FT-9) for `claude_version` CLI design decisions.
//!
//! Implements test cases from `tests/docs/feature/005_cli_design.md` (FT-6 through FT-9).
//! Each function maps to one FT- case verifying a design decision is implemented.
//!
//! # Coverage Map
//!
//! | FT-spec | ID | Decision | Function |
//! |---------|----|----------|----------|
//! | feature/005_cli_design.md | FT-6 | D3 | `ft005_6_bool_true_rejected` |
//! | feature/005_cli_design.md | FT-7 | D8 | `ft005_7_last_v_wins` |
//! | feature/005_cli_design.md | FT-8 | D4 | `ft005_8_cmd_not_implemented_exit2` |
//! | feature/005_cli_design.md | FT-9 | D7 | `ft005_9_per_cmd_validation` |

use crate::subprocess_helpers::{ assert_exit, run_clm, run_clm_with_env, stderr, stdout };

// ─── FT-6 (D3): boolean parameters use 0/1 only ──────────────────────────────

// FT-6: dry::true (non-integer boolean) → exit 1; confirms D3 (bool as 0/1 only)
#[ test ]
fn ft005_6_bool_true_rejected()
{
  let out = run_clm( &[ ".version.install", "dry::true" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "bool value 'true' rejection must produce error message: {err}" );
}

// ─── FT-7 (D8): last-occurrence wins for repeated parameters ─────────────────

// FT-7: v::0 then v::2 → last wins → v::2 (labeled output); confirms D8
#[ test ]
fn ft005_7_last_v_wins()
{
  let out = run_clm( &[ ".status", "v::0", "v::2" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // v::2 produces labeled output; v::0 would suppress labels
  assert!(
    text.contains( "Version" ) || text.contains( "version" ),
    "last v::2 must win, producing labeled output: {text}"
  );
}

// ─── FT-8 (D4): internal error exits with code 2 ─────────────────────────────

// FT-8: InternalError path (claude binary absent from PATH) → exit 2 (not exit 1)
// Verifies D4: internal errors distinguished from user input errors by exit code.
#[ test ]
fn ft005_8_cmd_not_implemented_exit2()
{
  // get_installed_version() has two paths:
  //   1. get_version_from_symlink() — reads $HOME/.local/bin/claude (no PATH needed)
  //   2. get_claude_version_raw()   — runs `bash -c "claude --version"` (needs PATH)
  // Clearing PATH alone is insufficient when a symlink exists; HOME must also be cleared
  // so that get_version_from_symlink() fails, guaranteeing InternalError → exit 2.
  let out = run_clm_with_env( &[ ".version.show" ], &[ ( "HOME", "" ), ( "PATH", "" ) ] );
  assert_exit( &out, 2 );
}

// ─── FT-9 (D7): per-command parameter validation rejects unknown params ───────

// FT-9: format:: on .settings.set (unregistered param) → exit 1; confirms D7
#[ test ]
fn ft005_9_per_cmd_validation()
{
  let out = run_clm( &[ ".settings.set", "format::json", "key::k", "value::v" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "format" ) || err.contains( "unknown" ) || err.contains( "argument" ),
    "per-command validation must reject 'format' on .settings.set: {err}"
  );
}

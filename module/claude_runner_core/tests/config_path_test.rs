//! `user_config_path_from()` / `user_config_path()` decision-table tests (BUG-007, BUG-560).
//!
//! The resolver locates clr's user-tier `config.toml` for every reader and writer:
//! clr's config tier, `clr isolated`'s default-model lookup, and clp's
//! `.model scope::subprocess`, `.provider.select` and `.usage` Gate 10. Both bugs came
//! from hand-built `$HOME/.clr/config.toml` joins that skipped `CLR_CONFIG_DIR` (and,
//! with an empty `HOME`, produced a cwd-relative path), so this table pins every
//! combination of the two inputs in one place.
//!
//! `user_config_path_from()` takes the variable values as arguments, so T01-T09 need
//! no environment mutation. T10-T11 exercise the env-reading wrapper; each test runs
//! in its own process under cargo-nextest, so setting `HOME`/`CLR_CONFIG_DIR` there is
//! safe — the same pattern as `isolated_model_resolution_test.rs`.
//!
//! ## Test Matrix
//!
//! | ID | `CLR_CONFIG_DIR` | `HOME` | Expected |
//! |----|------------------|--------|----------|
//! | T01 | `/cfg` | `/home/u` | `/cfg/config.toml` — override wins |
//! | T02 | `/cfg` | empty | `/cfg/config.toml` |
//! | T03 | `/cfg` | unset | `/cfg/config.toml` |
//! | T04 | empty | `/home/u` | `/home/u/.clr/config.toml` — empty override counts as unset |
//! | T05 | unset | `/home/u` | `/home/u/.clr/config.toml` |
//! | T06 | empty | empty | `None` |
//! | T07 | unset | unset | `None` — never a cwd-relative `.clr/config.toml` |
//! | T08 | unset | empty | `None` |
//! | T09 | relative `cfg` | unset | `cfg/config.toml` — the override is taken verbatim |
//! | T10 | env: set | env: set | wrapper reads the process environment, override wins |
//! | T11 | env: unset | env: unset | wrapper returns `None` |

use claude_runner_core::{ user_config_path, user_config_path_from };
use std::ffi::OsStr;
use std::path::PathBuf;

fn resolve( clr_config_dir : Option< &str >, home : Option< &str > ) -> Option< PathBuf >
{
  user_config_path_from( clr_config_dir.map( OsStr::new ), home.map( OsStr::new ) )
}

// ── Override set ─────────────────────────────────────────────────────────────

/// T01: a set override wins over a set `HOME`.
#[ test ]
fn t01_override_wins_over_home()
{
  assert_eq!( resolve( Some( "/cfg" ), Some( "/home/u" ) ), Some( PathBuf::from( "/cfg/config.toml" ) ) );
}

/// T02: a set override still resolves when `HOME` is empty.
#[ test ]
fn t02_override_with_empty_home()
{
  assert_eq!( resolve( Some( "/cfg" ), Some( "" ) ), Some( PathBuf::from( "/cfg/config.toml" ) ) );
}

/// T03: a set override still resolves when `HOME` is unset.
#[ test ]
fn t03_override_with_unset_home()
{
  assert_eq!( resolve( Some( "/cfg" ), None ), Some( PathBuf::from( "/cfg/config.toml" ) ) );
}

// ── Override empty or unset, HOME set ────────────────────────────────────────

/// T04: an empty override counts as unset, so `HOME` decides.
#[ test ]
fn t04_empty_override_falls_through_to_home()
{
  assert_eq!( resolve( Some( "" ), Some( "/home/u" ) ), Some( PathBuf::from( "/home/u/.clr/config.toml" ) ) );
}

/// T05: no override → `$HOME/.clr/config.toml`.
#[ test ]
fn t05_unset_override_uses_home()
{
  assert_eq!( resolve( None, Some( "/home/u" ) ), Some( PathBuf::from( "/home/u/.clr/config.toml" ) ) );
}

// ── Neither usable ───────────────────────────────────────────────────────────

/// T06: both empty → no user tier.
#[ test ]
fn t06_both_empty_is_none()
{
  assert_eq!( resolve( Some( "" ), Some( "" ) ), None );
}

/// T07: both unset → no user tier, never a relative `.clr/config.toml`.
#[ test ]
fn t07_both_unset_is_none()
{
  assert_eq!( resolve( None, None ), None );
}

/// T08: override unset and `HOME` empty → no user tier. An empty `HOME` joined onto
/// `.clr` would be relative, which is the BUG-560 cwd-write shape.
#[ test ]
fn t08_unset_override_empty_home_is_none()
{
  assert_eq!( resolve( None, Some( "" ) ), None );
}

/// T09: the override is used verbatim — a relative value stays relative, since the
/// caller chose it explicitly (no `HOME`-style guessing is involved).
#[ test ]
fn t09_relative_override_taken_verbatim()
{
  assert_eq!( resolve( Some( "cfg" ), None ), Some( PathBuf::from( "cfg/config.toml" ) ) );
}

// ── Env-reading wrapper ──────────────────────────────────────────────────────

/// T10: `user_config_path()` reads both variables from the process environment.
#[ test ]
fn t10_wrapper_reads_process_env()
{
  std::env::set_var( "CLR_CONFIG_DIR", "/cfg" );
  std::env::set_var( "HOME", "/home/u" );
  assert_eq!( user_config_path(), Some( PathBuf::from( "/cfg/config.toml" ) ) );
}

/// T11: `user_config_path()` returns `None` with both variables removed.
#[ test ]
fn t11_wrapper_none_without_env()
{
  std::env::remove_var( "CLR_CONFIG_DIR" );
  std::env::remove_var( "HOME" );
  assert_eq!( user_config_path(), None );
}

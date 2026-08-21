//! Help-listing completeness: every registered command must appear in `clg .` output.
//!
//! # Root Cause
//!
//! `print_usage()` in `src/cli_main.rs` renders the grouped command listing from a
//! hand-maintained `Vec<CommandGroup>` literal — a copy of the command list that is
//! also declared in `unilang.commands.yaml` and separately wired to a routine in
//! `build_command_registry()`. `.tail`, `.usage`, and `.rollup` were added to the
//! YAML and to the routine map but never added to the `print_usage()` literal, so
//! all three worked perfectly when invoked directly yet stayed invisible in the
//! `clg .` / `clg .help` grouped listing.
//!
//! # Why Not Caught
//!
//! Every per-command test (`cli_cmd_rollup_test.rs`, `cli_cmd_tail_test.rs`,
//! `cli_cmd_usage_test.rs`, ...) invokes its command directly and asserts on that
//! command's own output. None of them ever inspect the top-level `clg .` listing,
//! so a fully working, fully documented command could still be absent from it
//! without any existing test noticing.
//!
//! # Fix Applied
//!
//! Added the missing `CommandEntry` rows for `.tail`, `.usage`, and `.rollup` to
//! the "Query" `CommandGroup` in `print_usage()` (`src/cli_main.rs`).
//!
//! # Prevention
//!
//! This test parses `unilang.commands.yaml` — the same source-of-truth parse
//! `command_version_consistency_test.rs` already performs for command versions —
//! and asserts every declared command name appears somewhere in `clg .`'s rendered
//! stdout. A future command added to the registry but not to `print_usage()` now
//! fails this test loudly instead of shipping silently invisible.
//!
//! # Pitfall
//!
//! Don't assert against a second hand-written list of "expected" command names in
//! this test — that just relocates the exact drift risk being fixed. Parse
//! `unilang.commands.yaml` directly, the same ground truth the routine map and the
//! `docs/cli/command/readme.md` table are already generated from.

mod common;

use std::fs;
use std::path::PathBuf;

/// Extract every top-level `- name: "..."` command declared in `unilang.commands.yaml`.
///
/// Deliberately matches only column-0 `- name:` lines, NOT `line.trim()`-first —
/// each command's `arguments:` list nests its own per-command `- name:` param
/// entries four spaces in (e.g. `.show` alone declares `session_id`, `type`,
/// `detail`, ...). Trimming before matching treats every parameter name as a
/// command name too, producing dozens of false "missing" entries even when the
/// rendered help output is fully correct.
fn get_command_names() -> Vec< String >
{
  let yaml_path = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) ).join( "unilang.commands.yaml" );
  let yaml_content = fs::read_to_string( &yaml_path )
    .unwrap_or_else( | e | panic!( "Failed to read {}: {e}", yaml_path.display() ) );

  yaml_content
    .lines()
    .filter_map( | line |
    {
      line.strip_prefix( "- name:" ).map( | rest | rest.trim().trim_matches( '"' ).to_string() )
    } )
    .collect()
}

/// Every command declared in `unilang.commands.yaml` must be named somewhere in
/// the `clg .` grouped help listing — see module docs for the drift this guards.
#[ test ]
fn test_all_registered_commands_appear_in_top_level_help()
{
  let names = get_command_names();
  assert!( !names.is_empty(), "unilang.commands.yaml should declare at least one command" );

  let out = common::clg_cmd()
    .env( "HOME", "/tmp" )
    .arg( "." )
    .output()
    .expect( "clg . must run" );
  let stdout = String::from_utf8_lossy( &out.stdout );

  let missing : Vec< &String > = names.iter().filter( | name | !stdout.contains( name.as_str() ) ).collect();

  assert!(
    missing.is_empty(),
    "commands declared in unilang.commands.yaml but missing from `clg .` help output: {missing:?}\n\nfull output:\n{stdout}"
  );
}

//! Responsibility Boundary Test: `claude_profile` MUST NOT execute processes
//!
//! # Test Purpose
//!
//! Enforce architectural boundary: `claude_profile` provides account credential
//! management ONLY. It MUST NOT import or use `std::process::Command`.
//!
//! # Responsibility Split
//!
//! - **`claude_profile`** (THIS crate): Account credentials, token status, `~/.claude/` paths
//! - **`claude_runner_core`**: Claude Code process execution, `Command::new("claude")`
//! - **`claude_storage_core`**: Session file access, continuation detection
//!
//! # Verification Method
//!
//! 1. Grep source code for `use std::process::Command`
//! 2. Grep source code for `Command::new`
//! 3. Both must return zero matches
//!
//! # Test Strategy
//!
//! Static analysis test that inspects source files rather than runtime behavior.
//! Runs as a standard integration test, not a compile-fail test.
//!
//! # Failure Scenarios
//!
//! Test FAILS if:
//! - Any file in `claude_profile/src/` imports `std::process::Command`
//! - Any file in `claude_profile/src/` calls `Command::new()`
//! - Process execution logic leaks into account management crate
//!
//! # Related Tests
//!
//! - `claude_runner_core/tests/responsibility_single_execution_point_test.rs`: Verifies ONLY `claude_runner` executes
//!
//! # Test Matrix
//!
//! | Test Function | What It Checks | P/N |
//! |---------------|----------------|-----|
//! | `no_std_process_command_import` | `use std::process::Command` absent in src/ | P |
//! | `no_command_new_calls` | `Command::new` absent in src/ | P |
//! | `no_process_spawning_logic` | spawn/output/status/.wait/ExitStatus absent in non-comment src/ | P |
//! | `responsibility_documented_in_readme` | readme.md references claude_runner + has Out of Scope section | P |

use std::path::Path;
use std::process::Command;

#[ test ]
fn no_std_process_command_import()
{
  // Verify: claude_profile MUST NOT import std::process::Command
  // Rationale: Account management crate should NOT execute processes

  let src_dir = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src" );

  let output = Command::new( "/usr/bin/grep" )
    .args( [
      "-r",
      "use std::process::Command",
      src_dir.to_str().unwrap(),
    ] )
    .output()
    .expect( "Failed to run grep" );

  let matches = String::from_utf8_lossy( &output.stdout );
  let match_count = matches.lines().count();

  assert_eq!(
    match_count, 0,
    "RESPONSIBILITY VIOLATION: claude_profile MUST NOT import std::process::Command\n\
     Found {match_count} occurrence(s):\n{matches}\n\
     \n\
     Responsibility Boundary:\n\
     - claude_profile: Account credentials, token status, path topology\n\
     - claude_runner_core: Process execution ONLY\n\
     \n\
     Fix: Remove std::process::Command import and delegate to claude_runner_core"
  );
}

#[ test ]
fn no_command_new_calls()
{
  // Verify: claude_profile MUST NOT call Command::new()
  // Rationale: Process spawning belongs in claude_runner_core

  let src_dir = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src" );

  let output = Command::new( "/usr/bin/grep" )
    .args( [
      "-r",
      "Command::new",
      src_dir.to_str().unwrap(),
    ] )
    .output()
    .expect( "Failed to run grep" );

  let matches = String::from_utf8_lossy( &output.stdout );
  let match_count = matches.lines().count();

  assert_eq!(
    match_count, 0,
    "RESPONSIBILITY VIOLATION: claude_profile MUST NOT call process Command\n\
     Found {match_count} occurrence(s):\n{matches}\n\
     \n\
     Single Execution Point Rule:\n\
     - Process spawning MUST appear exactly 1x in entire workspace\n\
     - That single occurrence MUST be in claude_runner_core::execute()\n\
     \n\
     Fix: Remove process calls and use claude_runner_core crate"
  );
}

#[ test ]
fn no_process_spawning_logic()
{
  // Verify: claude_profile MUST NOT contain process spawning logic
  // Rationale: Execution logic belongs in claude_runner_core

  let src_dir = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "src" );

  let patterns =
  [
    "spawn()",
    "output()",
    "status()",
    ".wait()",
    "ExitStatus",
  ];

  for pattern in patterns
  {
    let output = Command::new( "/usr/bin/grep" )
      .args( [
        "-r",
        pattern,
        src_dir.to_str().unwrap(),
      ] )
      .output()
      .expect( "Failed to run grep" );

    let matches = String::from_utf8_lossy( &output.stdout );
    let real_matches : Vec< &str > = matches
      .lines()
      .filter( | line |
      {
        !line.contains( "//" )
          && !line.contains( "/*" )
          && !line.contains( "*/" )
          && !line.contains( " fn " )
      } )
      .collect();

    let count = real_matches.len();
    let joined = real_matches.join( "\n" );
    assert!(
      real_matches.is_empty(),
      "RESPONSIBILITY VIOLATION: claude_profile contains process spawning pattern '{pattern}'\n\
       Found {count} occurrence(s):\n{joined}\n\
       \n\
       Responsibility Boundary:\n\
       - claude_profile: Account credentials, token status, path topology\n\
       - claude_runner_core: Process spawning, output capture, exit codes\n\
       \n\
       Fix: Remove process spawning logic and delegate to claude_runner_core"
    );
  }
}

#[ test ]
fn responsibility_documented_in_readme()
{
  // Verify: claude_profile/readme.md clearly states execution is out of scope

  let readme_path = Path::new( env!( "CARGO_MANIFEST_DIR" ) ).join( "readme.md" );
  let readme_content = std::fs::read_to_string( &readme_path )
    .expect( "Failed to read readme.md" );

  assert!(
    readme_content.contains( "claude_runner" ),
    "DOCUMENTATION VIOLATION: readme.md must reference claude_runner for execution\n\
     \n\
     Out of Scope must show:\n\
     ❌ Claude Code execution → claude_runner_core\n\
     \n\
     Fix: Update readme.md with proper Out of Scope section"
  );

  assert!(
    readme_content.contains( "Out of Scope" ),
    "DOCUMENTATION VIOLATION: readme.md must have Out of Scope section\n\
     \n\
     Fix: Add Out of Scope section listing what claude_profile does NOT do"
  );
}

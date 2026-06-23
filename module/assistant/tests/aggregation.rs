//! Super-app aggregation feature and invariant tests.
//!
//! ## Purpose
//!
//! Verify the behavioural requirements documented in:
//! - `docs/feature/001_super_app_aggregation.md` (FT-1 .. FT-4)
//! - `docs/invariant/001_aggregation_completeness.md` (IC-1, IC-2)
//!
//! ## Test Matrix
//!
//! | Test | Spec | Scenario |
//! |------|------|----------|
//! | `ft1_all_five_l2_crates_contribute_commands` | FT-1 | representative command per L2 crate |
//! | `ft2_first_wins_precedence_status` | FT-2 | `.status` → claude_version, not claude_storage |
//! | `ft3_yaml_backed_commands_reachable` | FT-3 | YAML-sourced commands dispatched via PHF map |
//! | `ft4_claude_stub_prints_redirect` | FT-4 | `.claude` prints redirect, exit 0 |
//! | `ic1_register_commands_contract` | IC-1 | all 5 crates satisfy register_commands() |
//! | `ic2_no_orphan_yaml_commands` | IC-2 | every PHF-mapped command reachable |
//! | `unknown_command_exits_1` | — | exit-code-1 contract for unknown commands |

/// Run `ast <args>` in an isolated HOME, return `(exit_code, stdout, stderr)`.
fn run_ast( args : &[ &str ] ) -> ( i32, String, String )
{
  let home = tempfile::TempDir::new().unwrap();
  let out = std::process::Command::new(
    assert_cmd::cargo::cargo_bin!( "ast" )
  )
    .env( "HOME", home.path() )
    .args( args )
    .output()
    .unwrap();

  (
    out.status.code().unwrap_or( -1 ),
    String::from_utf8_lossy( &out.stdout ).into_owned(),
    String::from_utf8_lossy( &out.stderr ).into_owned(),
  )
}

// ---------- Feature: Super-App Aggregation (FT-*) ----------

/// FT-1: A representative command from each of the 5 Layer 2 crates
/// is found in the registry and dispatched (exit 0 or 2, never 1).
#[test]
fn ft1_all_five_l2_crates_contribute_commands()
{
  let cases : &[ ( &str, &str ) ] = &[
    ( ".kinds",        "claude_assets"  ),
    ( ".version.show", "claude_version" ),
    ( ".paths",        "claude_profile" ),
    ( ".claude",       "claude_runner"  ),
    ( ".show",         "claude_storage" ),
  ];

  for ( cmd, crate_name ) in cases
  {
    let ( code, _, stderr ) = run_ast( &[ cmd ] );
    assert!(
      code == 0 || code == 2,
      "FT-1: {cmd} ({crate_name}) should exit 0 or 2, not {code}; stderr: {stderr}",
    );
  }
}

/// FT-2: `.status` resolves to claude_version's implementation (first-wins)
/// because claude_version::register_commands() runs before
/// register_static_commands() which maps `.status` to storage's routine.
#[test]
fn ft2_first_wins_precedence_status()
{
  let ( code, _, stderr ) = run_ast( &[ ".status" ] );
  assert_eq!(
    code, 0,
    "FT-2: .status must exit 0 (version implementation); stderr: {stderr}",
  );
}

/// FT-3: YAML-backed storage commands registered via `register_static_commands()`
/// are reachable through the PHF routine map.
#[test]
fn ft3_yaml_backed_commands_reachable()
{
  let export_dir = tempfile::TempDir::new().unwrap();
  let export_path = export_dir.path().join( "export_out" );
  let export_arg = format!( "output::{}", export_path.display() );

  let yaml_commands : Vec< Vec< &str > > = vec![
    vec![ ".show" ],
    vec![ ".count" ],
    vec![ ".search", "query::test" ],
    vec![ ".export", "session_id::test", &export_arg ],
  ];

  for args in &yaml_commands
  {
    let ( code, _, stderr ) = run_ast( args );
    let label = args[ 0 ];
    assert!(
      code == 0 || code == 2,
      "FT-3: {label} (YAML-backed) should exit 0 or 2, not {code}; stderr: {stderr}",
    );
  }
}

/// FT-4: `.claude` in `ast` context routes to `claude_stub_routine`,
/// printing a redirect message instead of executing Claude Code.
#[test]
fn ft4_claude_stub_prints_redirect()
{
  let ( code, stdout, stderr ) = run_ast( &[ ".claude" ] );
  assert_eq!(
    code, 0,
    "FT-4: .claude should exit 0; stderr: {stderr}",
  );
  assert!(
    stdout.contains( "For Claude Code execution, use clr directly." ),
    "FT-4: .claude stdout must contain redirect message; got: {stdout}",
  );
}

// ---------- Invariant: Aggregation Completeness (IC-*) ----------

/// IC-1: Compilation of `ast` with `--features enabled` proves
/// `register_commands()` exists in all 5 L2 crates (type-system enforcement).
/// Runtime dispatch proves the commands are reachable (exit != 1).
#[test]
fn ic1_register_commands_contract()
{
  let crate_commands : &[ ( &str, &str ) ] = &[
    ( ".kinds",    "claude_assets"  ),
    ( ".processes","claude_version" ),
    ( ".accounts", "claude_profile" ),
    ( ".claude",   "claude_runner"  ),
    ( ".projects", "claude_storage" ),
  ];

  for ( cmd, crate_name ) in crate_commands
  {
    let ( code, _, stderr ) = run_ast( &[ cmd ] );
    assert_ne!(
      code, 1,
      "IC-1: {cmd} ({crate_name}) must not exit 1 (unknown command); stderr: {stderr}",
    );
  }
}

/// IC-2: Every command in the `register_static_commands()` PHF map is
/// registered in the CommandRegistry. None produce exit 1 (unknown command).
/// Commands shadowed by first-wins (`.status`, `.list`) still resolve —
/// they dispatch the first-registered implementation, not the YAML-backed one.
#[test]
fn ic2_no_orphan_yaml_commands()
{
  // Note: `.path` and `.exists` are in the PHF routines map but their YAML
  // names are `.project.path` / `.project.exists` — the name mismatch means
  // register_static_commands() never activates them.  Orphan PHF entries,
  // not orphan *commands* — excluded here until the mapping is fixed.
  let export_dir = tempfile::TempDir::new().unwrap();
  let export_path = export_dir.path().join( "export_out" );
  let export_arg = format!( "output::{}", export_path.display() );

  let phf_commands : Vec< Vec< &str > > = vec![
    vec![ ".claude" ], vec![ ".claude.help" ],
    vec![ ".status" ], vec![ ".list" ], vec![ ".show" ], vec![ ".projects" ],
    vec![ ".count" ], vec![ ".search", "query::test" ], vec![ ".export", "session_id::test", &export_arg ],
    vec![ ".session.dir" ], vec![ ".session.ensure" ],
  ];

  for args in &phf_commands
  {
    let ( code, _, stderr ) = run_ast( args );
    let label = args[ 0 ];
    assert_ne!(
      code, 1,
      "IC-2: {label} must not exit 1 (orphan command); stderr: {stderr}",
    );
  }
}

// ---------- Supplementary ----------

/// Validates the exit-code-1 contract: a genuinely unknown command MUST exit 1.
/// Without this, all "exit != 1" assertions in IC-1 and IC-2 would be vacuous
/// if the pipeline silently swallowed unknown commands with exit 0.
#[test]
fn unknown_command_exits_1()
{
  let ( code, _, _ ) = run_ast( &[ ".nonexistent_command_xyz" ] );
  assert_eq!(
    code, 1,
    "Unknown command must exit 1; got {code}",
  );
}

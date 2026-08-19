//! Integration tests for the config-file parameter tier (`~/.clr/config.toml` and
//! project-level `.clr.toml`) — the 4th of 5 CLI parameter precedence levels.
#![ cfg( unix ) ]
#![ cfg( feature = "enabled" ) ]
//!
//! Test spec: `docs/cli/config_param.md`; Test Matrix defined in TSK-204
//! (`task/claude_runner/completed/204_config_file_tier.md`).
//!
//! T01-T05 prove `Option<u32>` precedence (`max_sessions`) via real gate contention,
//! mirroring `concurrency_gate_test.rs`'s established pattern (`--dry-run` bypasses
//! the gate entirely and cannot observe this field). T06-T12 use the lighter
//! `--dry-run` / `--keep-claudecode` observation points available to `bool` and
//! `String` fields.
//!
//! # Test Case Index
//!
//! | ID  | Name                                                         | Category        |
//! |-----|---------------------------------------------------------------|-----------------|
//! | T01 | user config alone sets `max_sessions`                          | Precedence      |
//! | T02 | project `.clr.toml` overrides user config                     | Precedence      |
//! | T03 | CLI flag overrides config                                      | Precedence      |
//! | T04 | `--args-file` JSON overrides config                            | Precedence      |
//! | T05 | env var overrides config                                       | Precedence      |
//! | T06 | config-only bool field (`quiet`) applies                      | Bool Handling   |
//! | T07 | everything absent → hardcoded default, no error                | Bool Handling   |
//! | T08 | malformed TOML → exit 1, stderr names the file                 | Error Handling  |
//! | T09 | unknown key silently ignored; other keys still apply            | Forward Compat  |
//! | T10 | `CLR_CONFIG_DIR` redirects user-level discovery                 | Test Injection  |
//! | T11 | project discovery unaffected when `CLR_CONFIG_DIR` is unset      | Test Injection  |
//! | T12 | `--dry-run` reflects a config-supplied value                    | Dry Run         |
//! | T13 | invalid `output_style` config value → exit 1                    | Error Handling  |
//! | T14 | invalid `journal` config value → exit 1                         | Error Handling  |
//! | T15 | invalid `summary_fields` config value → exit 1                  | Error Handling  |
//! | T16 | config-only `gate_poll_secs`/`gate_max_attempts` change real gate timing | Precedence |
//! | T17 | non-anthropic `provider` suppresses config `model`/`fallback_model`      | Provider Gate |
//! | T18 | `provider = "anthropic"` leaves config `model` effective                 | Provider Gate |
//! | T19 | CLI `--model` still wins on a provider-pinned seat                       | Provider Gate |
//! | T20 | `--trace` names the suppressed config model and provider                 | Provider Gate |

mod cli_binary_test_helpers;
use cli_binary_test_helpers::
{
  exit_code, fake_claude_binary_dir, fake_claude_dir, make_proc_dir,
  run_cli_in_dir, run_cli_with_env, spawn_print_claude, spawn_print_claude_for,
  stderr_str, stdout_str, wait_bounded,
};
use std::process::Command;

/// Write `content` to `<dir>/config.toml`.
fn write_config_file( dir : &std::path::Path, content : &str )
{
  std::fs::write( dir.join( "config.toml" ), content ).expect( "write config.toml" );
}

// ── T01: user-level config.toml alone sets max_sessions ─────────────────────────

/// T01: `~/.clr/config.toml` (redirected via `CLR_CONFIG_DIR`) sets `max_sessions = 5`;
/// no CLI flag, env var, or `--args-file`. Effective `max_sessions` must be 5 — proven
/// by placing exactly 5 print-mode occupiers (4 long-lived + 1 that self-exits after
/// 5s) and observing the gate report "5/5 sessions active" then release once the
/// short-lived occupier exits.
#[ test ]
fn t01_user_config_only_sets_max_sessions()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut long_lived : Vec< std::process::Child > =
    ( 0..4 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let mut short_lived = spawn_print_claude_for( &occupier_path, 5 );

  let mut pids : Vec< u32 > = long_lived.iter().map( std::process::Child::id ).collect();
  pids.push( short_lived.id() );
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file( config_dir.path(), "max_sessions = 5\n" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_CONFIG_DIR", config_dir.path() )
    .output()
    .expect( "invoke clr" );

  for child in &mut long_lived { let _ = child.kill(); let _ = child.wait(); }
  let _ = short_lived.kill();
  let _ = short_lived.wait();

  assert!(
    out.status.success(),
    "T01: exit must be 0 after gate releases. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "gate-wait  active=5/5" ),
    "T01: config-supplied max_sessions=5 must be effective. Got:\n{stderr}"
  );
}

// ── T02: project .clr.toml overrides user config.toml ────────────────────────────

/// T02: project `.clr.toml` (cwd) sets `max_sessions = 3`; user config (via
/// `CLR_CONFIG_DIR`) sets `max_sessions = 5`. Effective value must be 3 (project
/// wins) — proven with exactly 3 occupiers (2 long-lived + 1 short-lived).
#[ test ]
fn t02_project_config_overrides_user_config()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut long_lived : Vec< std::process::Child > =
    ( 0..2 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let mut short_lived = spawn_print_claude_for( &occupier_path, 5 );

  let mut pids : Vec< u32 > = long_lived.iter().map( std::process::Child::id ).collect();
  pids.push( short_lived.id() );
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let user_dir = tempfile::TempDir::new().expect( "user config dir" );
  write_config_file( user_dir.path(), "max_sessions = 5\n" );
  let project_dir = tempfile::TempDir::new().expect( "project dir" );
  std::fs::write( project_dir.path().join( ".clr.toml" ), "max_sessions = 3\n" )
    .expect( "write .clr.toml" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--journal", "off", "x" ] )
    .current_dir( project_dir.path() )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_CONFIG_DIR", user_dir.path() )
    .output()
    .expect( "invoke clr" );

  for child in &mut long_lived { let _ = child.kill(); let _ = child.wait(); }
  let _ = short_lived.kill();
  let _ = short_lived.wait();

  assert!(
    out.status.success(),
    "T02: exit must be 0 after gate releases. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "gate-wait  active=3/3" ),
    "T02: project .clr.toml (max_sessions=3) must override user config (max_sessions=5). Got:\n{stderr}"
  );
}

// ── T03: CLI flag overrides config ───────────────────────────────────────────────

/// T03: `--max-sessions 7` on the CLI, config sets `max_sessions = 5`. Effective
/// value must be 7 (CLI wins over every lower tier) — proven with 7 occupiers
/// (6 long-lived + 1 short-lived).
#[ test ]
fn t03_cli_overrides_config_max_sessions()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut long_lived : Vec< std::process::Child > =
    ( 0..6 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let mut short_lived = spawn_print_claude_for( &occupier_path, 5 );

  let mut pids : Vec< u32 > = long_lived.iter().map( std::process::Child::id ).collect();
  pids.push( short_lived.id() );
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file( config_dir.path(), "max_sessions = 5\n" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--max-sessions", "7", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_CONFIG_DIR", config_dir.path() )
    .output()
    .expect( "invoke clr" );

  for child in &mut long_lived { let _ = child.kill(); let _ = child.wait(); }
  let _ = short_lived.kill();
  let _ = short_lived.wait();

  assert!(
    out.status.success(),
    "T03: exit must be 0 after gate releases. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "gate-wait  active=7/7" ),
    "T03: CLI --max-sessions 7 must override config (max_sessions=5). Got:\n{stderr}"
  );
}

// ── T04: --args-file JSON overrides config ───────────────────────────────────────

/// T04: `--args-file` JSON supplies `{"max-sessions": 9}`, config sets
/// `max_sessions = 5`. Effective value must be 9 — proven with 9 occupiers
/// (8 long-lived + 1 short-lived).
#[ test ]
fn t04_args_file_json_overrides_config_max_sessions()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut long_lived : Vec< std::process::Child > =
    ( 0..8 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let mut short_lived = spawn_print_claude_for( &occupier_path, 5 );

  let mut pids : Vec< u32 > = long_lived.iter().map( std::process::Child::id ).collect();
  pids.push( short_lived.id() );
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file( config_dir.path(), "max_sessions = 5\n" );
  let args_dir = tempfile::TempDir::new().expect( "args dir" );
  let args_path = args_dir.path().join( "args.json" );
  std::fs::write( &args_path, r#"{"max-sessions": 9}"# ).expect( "write args-file" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--args-file", args_path.to_str().expect( "args path UTF-8" ), "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_CONFIG_DIR", config_dir.path() )
    .output()
    .expect( "invoke clr" );

  for child in &mut long_lived { let _ = child.kill(); let _ = child.wait(); }
  let _ = short_lived.kill();
  let _ = short_lived.wait();

  assert!(
    out.status.success(),
    "T04: exit must be 0 after gate releases. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "gate-wait  active=9/9" ),
    "T04: --args-file JSON (max-sessions=9) must override config (max_sessions=5). Got:\n{stderr}"
  );
}

// ── T05: env var overrides config ────────────────────────────────────────────────

/// T05: `CLR_MAX_SESSIONS=2` env var, config sets `max_sessions = 5`. Effective
/// value must be 2 — proven with 2 occupiers (1 long-lived + 1 short-lived).
#[ test ]
fn t05_env_var_overrides_config_max_sessions()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut long_lived : Vec< std::process::Child > =
    ( 0..1 ).map( |_| spawn_print_claude( &occupier_path ) ).collect();
  let mut short_lived = spawn_print_claude_for( &occupier_path, 5 );

  let mut pids : Vec< u32 > = long_lived.iter().map( std::process::Child::id ).collect();
  pids.push( short_lived.id() );
  let proc = make_proc_dir( &pids );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file( config_dir.path(), "max_sessions = 5\n" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let out = Command::new( bin )
    .args( [ "-p", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_GATE_POLL_SECS", "1" )
    .env( "CLR_CONFIG_DIR", config_dir.path() )
    .env( "CLR_MAX_SESSIONS", "2" )
    .output()
    .expect( "invoke clr" );

  for child in &mut long_lived { let _ = child.kill(); let _ = child.wait(); }
  let _ = short_lived.kill();
  let _ = short_lived.wait();

  assert!(
    out.status.success(),
    "T05: exit must be 0 after gate releases. stderr: {}",
    String::from_utf8_lossy( &out.stderr )
  );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "gate-wait  active=2/2" ),
    "T05: CLR_MAX_SESSIONS=2 env var must override config (max_sessions=5). Got:\n{stderr}"
  );
}

// ── T06: config-only bool field (quiet) applies ──────────────────────────────────

/// T06: config `quiet = true`, no CLI/env → effective `quiet` = true. Observed via
/// the existing `--keep-claudecode` + `CLAUDECODE` nested-agent warning (BUG-248),
/// which is gated on `!cli.quiet` and fires before the `--dry-run` exit — no gate
/// contention needed for a bool field. `quiet = true` must suppress the warning.
#[ test ]
fn t06_config_only_bool_field_quiet_applies()
{
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file( config_dir.path(), "quiet = true\n" );

  let out = run_cli_with_env(
    &[ "--keep-claudecode", "--dry-run", "task" ],
    &[ ( "CLAUDECODE", "1" ), ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!( exit_code( &out ), 0, "T06: dry-run must exit 0; stderr: {}", stderr_str( &out ) );
  let stderr = stderr_str( &out );
  assert!(
    !stderr.contains( "nested-agent mode" ),
    "T06: config quiet=true must suppress the --keep-claudecode warning. Got:\n{stderr}"
  );
}

// ── T07: everything absent → hardcoded default, no error ────────────────────────

/// T07: no CLI `--quiet`, no `CLR_QUIET`, no config file present at the
/// `CLR_CONFIG_DIR`-redirected location → effective `quiet` = false (hardcoded
/// default). The `--keep-claudecode` warning must still appear (`!quiet` gates it),
/// and there must be no error exit.
#[ test ]
fn t07_all_absent_bool_field_defaults_false_no_error()
{
  let config_dir = tempfile::TempDir::new().expect( "empty config dir" );
  let out = run_cli_with_env(
    &[ "--keep-claudecode", "--dry-run", "task" ],
    &[ ( "CLAUDECODE", "1" ), ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!( exit_code( &out ), 0, "T07: must exit 0 with no config file present; stderr: {}", stderr_str( &out ) );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "nested-agent mode" ),
    "T07: quiet must default to false (warning must appear) when no config/env/CLI set it. Got:\n{stderr}"
  );
}

// ── T08: malformed TOML → exit 1, stderr names the file ─────────────────────────

/// T08 (AF3): genuinely invalid TOML syntax (unclosed bracket, not just a type
/// mismatch) in `~/.clr/config.toml` (via `CLR_CONFIG_DIR`) → `clr` exits 1;
/// stderr names the offending file path.
#[ test ]
fn t08_malformed_toml_exits_1_names_file()
{
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file( config_dir.path(), "max_sessions = [1, 2\n" );

  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!(
    exit_code( &out ), 1,
    "T08: malformed TOML must exit 1; got {}; stderr: {}", exit_code( &out ), stderr_str( &out )
  );
  let stderr = stderr_str( &out );
  let config_path = config_dir.path().join( "config.toml" );
  assert!(
    stderr.contains( config_path.to_str().expect( "utf8" ) ),
    "T08: stderr must name the offending config file. Got:\n{stderr}"
  );
}

// ── T09: unknown key silently ignored; other keys still apply ───────────────────

/// T09: `.clr.toml` with an unrecognized key `foo = "bar"` alongside a recognized
/// key (`quiet = true`) → the unknown key is silently ignored; the recognized key
/// still applies (observed via the same `--keep-claudecode` warning-suppression
/// proxy as T06), and there is no error exit.
#[ test ]
fn t09_unknown_key_silently_ignored_others_apply()
{
  let project_dir = tempfile::TempDir::new().expect( "project dir" );
  std::fs::write(
    project_dir.path().join( ".clr.toml" ),
    "foo = \"bar\"\nquiet = true\n",
  ).expect( "write .clr.toml" );
  let empty_user_dir = tempfile::TempDir::new().expect( "empty user config dir" );

  let out = run_cli_in_dir(
    &[ "--keep-claudecode", "--dry-run", "task" ],
    project_dir.path(),
    &[ ( "CLAUDECODE", "1" ), ( "CLR_CONFIG_DIR", empty_user_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!( exit_code( &out ), 0, "T09: unknown key must not cause an error; stderr: {}", stderr_str( &out ) );
  let stderr = stderr_str( &out );
  assert!(
    !stderr.contains( "nested-agent mode" ),
    "T09: recognized key (quiet=true) in the same file must still apply. Got:\n{stderr}"
  );
}

// ── T10: CLR_CONFIG_DIR redirects user-level discovery ───────────────────────────

/// T10: `CLR_CONFIG_DIR` set to a tempdir containing `config.toml` with
/// `model = "claude-opus-4-8"` → the effective value comes from that redirected
/// location. Observed via `--dry-run`'s printed command (the only eligible config
/// field that appears directly in dry-run output without further plumbing).
#[ test ]
fn t10_clr_config_dir_redirects_user_level_discovery()
{
  let config_dir = tempfile::TempDir::new().expect( "redirected config dir" );
  write_config_file( config_dir.path(), "model = \"claude-opus-4-8\"\n" );

  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!( exit_code( &out ), 0, "T10: must exit 0; stderr: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "claude-opus-4-8" ),
    "T10: CLR_CONFIG_DIR-redirected config must be effective. Got:\n{stdout}"
  );
}

// ── T11: project discovery unaffected when CLR_CONFIG_DIR is unset ──────────────

/// T11: `.clr.toml` present in cwd, `CLR_CONFIG_DIR` unset → project-level
/// discovery is unaffected; the project file's `model` value still applies.
#[ test ]
fn t11_project_discovery_unaffected_by_clr_config_dir_absence()
{
  let project_dir = tempfile::TempDir::new().expect( "project dir" );
  std::fs::write(
    project_dir.path().join( ".clr.toml" ),
    "model = \"claude-opus-4-8\"\n",
  ).expect( "write .clr.toml" );

  let out = run_cli_in_dir( &[ "--dry-run", "task" ], project_dir.path(), &[] );
  assert_eq!( exit_code( &out ), 0, "T11: must exit 0; stderr: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "claude-opus-4-8" ),
    "T11: project .clr.toml discovery must work even when CLR_CONFIG_DIR is unset. Got:\n{stdout}"
  );
}

// ── T12: --dry-run reflects a config-supplied value ──────────────────────────────

/// T12: config `model = "claude-opus-4-8"`, `clr run --dry-run "hi"` → the printed
/// command preview includes the config-resolved model, with no dry-run-specific
/// code required (a direct consequence of `apply_config_defaults()` running before
/// `handle_dry_run()`, mirroring how env var resolution already works).
#[ test ]
fn t12_dry_run_reflects_config_supplied_model()
{
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file( config_dir.path(), "model = \"claude-opus-4-8\"\n" );

  let out = run_cli_with_env(
    &[ "--dry-run", "hi" ],
    &[ ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!( exit_code( &out ), 0, "T12: must exit 0; stderr: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "claude-opus-4-8" ),
    "T12: dry-run preview must reflect config-supplied model. Got:\n{stdout}"
  );
}

// ── T13: invalid output_style config value → exit 1 ─────────────────────────────

/// T13: config `output_style = "bogus"` → `clr` exits 1; stderr names the invalid
/// value — `apply_config_defaults` must validate exactly as `apply_env_vars`
/// already validates `CLR_OUTPUT_STYLE` (adversarial review found config previously
/// accepted any string silently).
#[ test ]
fn t13_invalid_output_style_config_value_exits_1()
{
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file( config_dir.path(), "output_style = \"bogus\"\n" );

  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!(
    exit_code( &out ), 1,
    "T13: invalid output_style must exit 1; got {}; stderr: {}", exit_code( &out ), stderr_str( &out )
  );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "output_style" ) && stderr.contains( "bogus" ),
    "T13: stderr must name the invalid output_style value. Got:\n{stderr}"
  );
}

// ── T14: invalid journal config value → exit 1 ───────────────────────────────────

/// T14: config `journal = "bogus"` → `clr` exits 1; stderr names the invalid value —
/// same validation gap/fix as T13, applied to `CLR_JOURNAL`'s config-tier equivalent.
#[ test ]
fn t14_invalid_journal_config_value_exits_1()
{
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file( config_dir.path(), "journal = \"bogus\"\n" );

  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!(
    exit_code( &out ), 1,
    "T14: invalid journal must exit 1; got {}; stderr: {}", exit_code( &out ), stderr_str( &out )
  );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "journal" ) && stderr.contains( "bogus" ),
    "T14: stderr must name the invalid journal value. Got:\n{stderr}"
  );
}

// ── T15: invalid summary_fields config value → exit 1 ───────────────────────────

/// T15: config `summary_fields = "bogus"` → `clr` exits 1; stderr names the invalid
/// value — delegates to `summary::resolve_fields`, the same validator the CLI flag
/// and env var tiers already use.
#[ test ]
fn t15_invalid_summary_fields_config_value_exits_1()
{
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file( config_dir.path(), "summary_fields = \"bogus\"\n" );

  let out = run_cli_with_env(
    &[ "--dry-run", "task" ],
    &[ ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!(
    exit_code( &out ), 1,
    "T15: invalid summary_fields must exit 1; got {}; stderr: {}", exit_code( &out ), stderr_str( &out )
  );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "summary_fields" ) && stderr.contains( "bogus" ),
    "T15: stderr must name the invalid summary_fields value. Got:\n{stderr}"
  );
}

// ── T16: gate_poll_secs/gate_max_attempts config.toml keys change real gate timing ──

/// T16 (hardening fix 3, config-file tier): `~/.clr/config.toml` (redirected via
/// `CLR_CONFIG_DIR`) sets `gate_poll_secs = 1` and `gate_max_attempts = 2`; no CLI
/// flag, env var, or `--args-file`. Effective values must actually change gate
/// timing — proven by forcing gate exhaustion (1 permanent occupier,
/// `--max-sessions 1`, `--retry-override 0`) and asserting it completes within a
/// bounded 10s deadline instead of the production 1000-attempt x 30s default
/// (~8.3h), mirroring `concurrency_gate_test.rs`'s T09 pattern for the
/// config-file tier.
#[ test ]
fn t16_config_only_sets_gate_poll_secs_and_max_attempts()
{
  let ( _occupier_dir, occupier_path ) = fake_claude_binary_dir();
  let mut occupier = spawn_print_claude_for( &occupier_path, 60 );
  let proc = make_proc_dir( &[ occupier.id() ] );

  let ( _script_dir, script_path ) = fake_claude_dir( "exit 0" );
  let gate_dir = tempfile::TempDir::new().expect( "gate dir" );
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file( config_dir.path(), "gate_poll_secs = 1\ngate_max_attempts = 2\n" );

  let bin = env!( "CARGO_BIN_EXE_clr" );
  let mut child = Command::new( bin )
    .args( [ "-p", "--max-sessions", "1", "--retry-override", "0", "--journal", "off", "x" ] )
    .env( "PATH", &script_path )
    .env( "CLR_PROC_DIR", proc.path().to_str().expect( "proc dir UTF-8" ) )
    .env( "CLR_GATE_DIR", gate_dir.path() )
    .env( "CLR_CONFIG_DIR", config_dir.path() )
    .stdout( std::process::Stdio::null() )
    .stderr( std::process::Stdio::piped() )
    .spawn()
    .expect( "spawn clr" );

  let deadline = std::time::Instant::now() + core::time::Duration::from_secs( 10 );
  let exited = wait_bounded( &mut child, deadline );
  if exited.is_none() { let _ = child.kill(); }
  let out = child.wait_with_output().expect( "reap clr" );

  let _ = occupier.kill();
  let _ = occupier.wait();

  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    exited.is_some(),
    "T16: gate must exhaust within 10s when config supplies both overrides \
     (2 attempts x 1s poll) — still running means config values are not \
     taking effect. stderr:\n{stderr}"
  );
  assert_eq!(
    exited.and_then( |s| s.code() ), Some( 1 ),
    "T16: exit must be 1 once the gate exhausts. stderr: {stderr}"
  );
  assert!(
    stderr.contains(
      "Error: [Runner] session gate timed out — 1 print sessions, max-sessions=1 — retries exhausted (exit 1)"
    ),
    "T16: exact exhaustion message required. Got:\n{stderr}"
  );
}

// ── T17: non-anthropic provider suppresses config-tier model keys ─────────────────

/// T17: user config sets `provider = "kimi"` alongside `model` and `fallback_model`;
/// no higher-tier model source. The dry-run preview must contain neither `--model`
/// nor `--fallback-model` — a seat pinned to a non-anthropic provider routes its
/// sessions through the `ANTHROPIC_*` env block in `~/.claude/settings.json`, and a
/// config-tier model would be promoted to an explicit `--model` flag silently
/// overriding that binding on every launch (`docs/cli/config_param.md § Provider Gate`).
#[ test ]
fn t17_non_anthropic_provider_suppresses_config_model()
{
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file(
    config_dir.path(),
    "provider = \"kimi\"\nmodel = \"claude-sonnet-5\"\nfallback_model = \"claude-opus-4-8\"\n",
  );

  let out = run_cli_with_env(
    &[ "--dry-run", "hi" ],
    &[ ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!( exit_code( &out ), 0, "T17: must exit 0; stderr: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );
  assert!(
    !stdout.contains( "--model" ) && !stdout.contains( "claude-sonnet-5" ),
    "T17: config model must be suppressed on a kimi-provider seat. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "--fallback-model" ) && !stdout.contains( "claude-opus-4-8" ),
    "T17: config fallback_model must be suppressed on a kimi-provider seat. Got:\n{stdout}"
  );
}

// ── T18: explicit anthropic provider leaves config model effective ────────────────

/// T18: `provider = "anthropic"` (the default, stated explicitly) must NOT trigger
/// suppression — config `model` applies exactly as with no provider key at all
/// (which T12 already pins).
#[ test ]
fn t18_anthropic_provider_leaves_config_model_effective()
{
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file(
    config_dir.path(),
    "provider = \"anthropic\"\nmodel = \"claude-sonnet-5\"\n",
  );

  let out = run_cli_with_env(
    &[ "--dry-run", "hi" ],
    &[ ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!( exit_code( &out ), 0, "T18: must exit 0; stderr: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--model" ) && stdout.contains( "claude-sonnet-5" ),
    "T18: explicit anthropic provider must not suppress config model. Got:\n{stdout}"
  );
}

// ── T19: CLI --model still wins on a provider-pinned seat ─────────────────────────

/// T19: `provider = "kimi"` with config `model = "claude-sonnet-5"`, but the CLI
/// passes `--model claude-opus-4-8`. The explicit flag is user intent — it must
/// survive the provider gate (only the config tier is suppressed, never levels 1-3).
#[ test ]
fn t19_cli_model_wins_over_provider_gate()
{
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file(
    config_dir.path(),
    "provider = \"kimi\"\nmodel = \"claude-sonnet-5\"\n",
  );

  let out = run_cli_with_env(
    &[ "--model", "claude-opus-4-8", "--dry-run", "hi" ],
    &[ ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!( exit_code( &out ), 0, "T19: must exit 0; stderr: {}", stderr_str( &out ) );
  let stdout = stdout_str( &out );
  assert!(
    stdout.contains( "--model" ) && stdout.contains( "claude-opus-4-8" ),
    "T19: explicit CLI --model must survive the provider gate. Got:\n{stdout}"
  );
  assert!(
    !stdout.contains( "claude-sonnet-5" ),
    "T19: the suppressed config model must not leak into the command. Got:\n{stdout}"
  );
}

// ── T20: --trace names the suppressed config model ────────────────────────────────

/// T20: same config as T17 plus `--trace` — stderr must carry one suppression note
/// per ignored key, naming both the value and the provider, so a `clr --trace` user
/// can see why the config model vanished instead of silently missing it.
#[ test ]
fn t20_trace_names_suppressed_config_model()
{
  let config_dir = tempfile::TempDir::new().expect( "config dir" );
  write_config_file(
    config_dir.path(),
    "provider = \"kimi\"\nmodel = \"claude-sonnet-5\"\nfallback_model = \"claude-opus-4-8\"\n",
  );

  let out = run_cli_with_env(
    &[ "--trace", "--dry-run", "hi" ],
    &[ ( "CLR_CONFIG_DIR", config_dir.path().to_str().expect( "utf8" ) ) ],
  );
  assert_eq!( exit_code( &out ), 0, "T20: must exit 0; stderr: {}", stderr_str( &out ) );
  let stderr = stderr_str( &out );
  assert!(
    stderr.contains( "config model 'claude-sonnet-5' ignored (provider: kimi)" ),
    "T20: trace must name the suppressed model and provider. Got:\n{stderr}"
  );
  assert!(
    stderr.contains( "config fallback_model 'claude-opus-4-8' ignored (provider: kimi)" ),
    "T20: trace must name the suppressed fallback_model too. Got:\n{stderr}"
  );
}

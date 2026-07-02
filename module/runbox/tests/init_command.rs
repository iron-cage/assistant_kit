//! CLI integration tests for `runbox` / `crb` binary.
//!
//! ## Test Coverage
//!
//! | TC | AC | Scenario | Expected |
//! |----|----|----------|----------|
//! | cli01 | AC-011 | Empty argv | Exit 0; usage printed to stdout |
//! | cli02 | AC-011 | `--help` flag | Exit 0; usage printed to stdout |
//! | cli03 | AC-011 | `-h` flag | Exit 0; usage printed to stdout |
//! | cli04 | AC-001 | `.init` without `image::` | Exit 1; "missing required argument: image::" in stderr |
//! | cli05 | AC-002 | `.init image::x ecosystem::java` | Exit 1; "unknown ecosystem: java" in stderr |
//! | cli06 | AC-003 | `.init image::my_img` | Exit 0; three files created |
//! | cli07 | AC-004 | wrapper script content | Contains discovery function |
//! | cli08 | AC-005 | wrapper script permissions | Executable bit set |
//! | cli09 | AC-006 | runbox.yml fields | All required fields present |
//! | cli10 | AC-007 | cache_dir per ecosystem | rust→target, python→.venv, nodejs→node_modules, none→.cache |
//! | cli11 | AC-008 | test_script default | Defaults to verb/test.d/l1 |
//! | cli12 | AC-009 | test_script override | Custom path written to runbox.yml |
//! | cli13 | AC-010 | runbox/ already exists | Exit 1; "runbox/ already exists" in stderr |
//! | cli14 | AC-012 | `crb` alias binary | Calls same run_cli as `runbox` |

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

// ── helpers ───────────────────────────────────────────────────────────────────

fn assert_container()
{
  let in_container = std::path::Path::new( "/.dockerenv" ).exists()
    || std::path::Path::new( "/run/.containerenv" ).exists()
    || std::env::var( "RUNBOX_CONTAINER" ).as_deref() == Ok( "1" );
  let escaped = std::env::var( "VERB_LAYER" ).as_deref() == Ok( "l0" );
  assert!(
    in_container || escaped,
    "\n\nTests must run inside a container.\n\
     Standard invocation: cd module/runbox && ./verb/test\n\
     Host bypass:         VERB_LAYER=l0 cargo nextest run --all-features\n"
  );
}

fn crb() -> Command
{
  assert_container();
  Command::cargo_bin( "crb" ).unwrap()
}

fn runbox_bin() -> Command
{
  assert_container();
  Command::cargo_bin( "runbox" ).unwrap()
}

// ── cli01 ─────────────────────────────────────────────────────────────────────

/// cli01: empty argv prints usage and exits 0.
#[ test ]
fn cli01_empty_argv_shows_usage()
{
  let dir = TempDir::new().unwrap();
  let out = crb()
  .current_dir( dir.path() )
  .output()
  .unwrap();

  assert!( out.status.success(), "empty argv must exit 0, got: {:?}", out.status );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( ".init" ), "usage must mention .init, got: {stdout}" );
}

// ── cli02 ─────────────────────────────────────────────────────────────────────

/// cli02: `--help` prints usage and exits 0.
#[ test ]
fn cli02_help_flag_shows_usage()
{
  let dir = TempDir::new().unwrap();
  let out = crb()
  .arg( "--help" )
  .current_dir( dir.path() )
  .output()
  .unwrap();

  assert!( out.status.success(), "--help must exit 0, got: {:?}", out.status );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( ".init" ), "usage must mention .init, got: {stdout}" );
}

// ── cli03 ─────────────────────────────────────────────────────────────────────

/// cli03: `-h` prints usage and exits 0.
#[ test ]
fn cli03_h_flag_shows_usage()
{
  let dir = TempDir::new().unwrap();
  let out = crb()
  .arg( "-h" )
  .current_dir( dir.path() )
  .output()
  .unwrap();

  assert!( out.status.success(), "-h must exit 0, got: {:?}", out.status );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( ".init" ), "usage must mention .init, got: {stdout}" );
}

// ── cli04 ─────────────────────────────────────────────────────────────────────

/// cli04 (AC-001): `.init` without `image::` exits 1 with required-arg error.
#[ test ]
fn cli04_init_without_image_exits_1()
{
  let dir = TempDir::new().unwrap();
  let out = crb()
  .args( [ ".init" ] )
  .current_dir( dir.path() )
  .output()
  .unwrap();

  assert!( !out.status.success(), "must fail without image::" );
  assert_eq!( out.status.code(), Some( 1 ), "exit code must be 1" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "missing required argument: image::" ),
    "stderr must mention missing image::, got: {stderr}"
  );
}

// ── cli05 ─────────────────────────────────────────────────────────────────────

/// cli05 (AC-002): unknown `ecosystem::` value exits 1.
#[ test ]
fn cli05_unknown_ecosystem_exits_1()
{
  let dir = TempDir::new().unwrap();
  let out = crb()
  .args( [ ".init", "image::my_img", "ecosystem::java" ] )
  .current_dir( dir.path() )
  .output()
  .unwrap();

  assert!( !out.status.success(), "must fail for unknown ecosystem" );
  assert_eq!( out.status.code(), Some( 1 ), "exit code must be 1" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "unknown ecosystem: java" ),
    "stderr must mention unknown ecosystem, got: {stderr}"
  );
}

// ── cli06 ─────────────────────────────────────────────────────────────────────

/// cli06 (AC-003): `.init image::my_img` creates three files.
#[ test ]
fn cli06_init_creates_three_files()
{
  let dir = TempDir::new().unwrap();
  let out = crb()
  .args( [ ".init", "image::my_img" ] )
  .current_dir( dir.path() )
  .output()
  .unwrap();

  assert!( out.status.success(), "must exit 0, got: {:?}\nstderr: {}", out.status, String::from_utf8_lossy( &out.stderr ) );

  let runbox_dir = dir.path().join( "runbox" );
  assert!( runbox_dir.is_dir(),                         "runbox/ must be created" );
  assert!( runbox_dir.join( "runbox" ).is_file(),       "runbox/runbox must exist" );
  assert!( runbox_dir.join( "runbox.yml" ).is_file(),   "runbox/runbox.yml must exist" );
  assert!( runbox_dir.join( "runbox.dockerfile" ).is_file(), "runbox/runbox.dockerfile must exist" );
}

// ── cli07 ─────────────────────────────────────────────────────────────────────

/// cli07 (AC-004): generated `runbox/runbox` contains walk-up discovery logic.
#[ test ]
fn cli07_wrapper_script_contains_discovery_function()
{
  let dir = TempDir::new().unwrap();
  crb()
  .args( [ ".init", "image::my_img" ] )
  .current_dir( dir.path() )
  .output()
  .unwrap();

  let wrapper = fs::read_to_string( dir.path().join( "runbox/runbox" ) ).unwrap();
  assert!( wrapper.starts_with( "#!/usr/bin/env bash" ), "must have shebang" );
  assert!( wrapper.contains( "_find_runbox_run" ),       "must contain discovery function" );
  assert!( wrapper.contains( "runbox.yml" ),             "must reference runbox.yml" );
}

// ── cli08 ─────────────────────────────────────────────────────────────────────

/// cli08 (AC-005): generated `runbox/runbox` has executable bit set.
#[ test ]
fn cli08_wrapper_script_is_executable()
{
  let dir = TempDir::new().unwrap();
  crb()
  .args( [ ".init", "image::my_img" ] )
  .current_dir( dir.path() )
  .output()
  .unwrap();

  let wrapper_path = dir.path().join( "runbox/runbox" );

  #[ cfg( unix ) ]
  {
    use std::os::unix::fs::PermissionsExt as _;
    let mode = fs::metadata( &wrapper_path ).unwrap().permissions().mode();
    assert!( mode & 0o111 != 0, "runbox/runbox must have executable bit set (mode: {mode:o})" );
  }

  // On non-Unix: verify file exists (chmod semantics not applicable).
  #[ cfg( not( unix ) ) ]
  assert!( wrapper_path.is_file(), "runbox/runbox must exist" );
}

// ── cli09 ─────────────────────────────────────────────────────────────────────

/// cli09 (AC-006): generated `runbox.yml` contains all required fields.
#[ test ]
fn cli09_runbox_yml_contains_required_fields()
{
  let dir = TempDir::new().unwrap();
  crb()
  .args( [ ".init", "image::my_img" ] )
  .current_dir( dir.path() )
  .output()
  .unwrap();

  let yml = fs::read_to_string( dir.path().join( "runbox/runbox.yml" ) ).unwrap();
  assert!( yml.contains( "image:" ),          "must have image field" );
  assert!( yml.contains( "dockerfile:" ),     "must have dockerfile field" );
  assert!( yml.contains( "cache_dir:" ),      "must have cache_dir field" );
  assert!( yml.contains( "workspace_root:" ), "must have workspace_root field" );
  assert!( yml.contains( "test_script:" ),    "must have test_script field" );
  assert!( yml.contains( "my_img" ),          "must include image value" );
}

// ── cli10 ─────────────────────────────────────────────────────────────────────

/// cli10 (AC-007): `cache_dir` in `runbox.yml` matches ecosystem.
#[ test ]
fn cli10_cache_dir_matches_ecosystem()
{
  for ( ecosystem, expected_cache_dir ) in [
    ( "rust",   "target"       ),
    ( "nodejs", "node_modules" ),
    ( "python", ".venv"        ),
    ( "none",   ".cache"       ),
  ]
  {
    let dir = TempDir::new().unwrap();
    let out = crb()
    .args( [ ".init", "image::x", &format!( "ecosystem::{ecosystem}" ) ] )
    .current_dir( dir.path() )
    .output()
    .unwrap();

    assert!(
      out.status.success(),
      "ecosystem::{ecosystem} must succeed, stderr: {}",
      String::from_utf8_lossy( &out.stderr )
    );

    let yml = fs::read_to_string( dir.path().join( "runbox/runbox.yml" ) ).unwrap();
    assert!(
      yml.contains( &format!( "cache_dir: {expected_cache_dir}" ) ),
      "ecosystem::{ecosystem} must have cache_dir: {expected_cache_dir}, got:\n{yml}"
    );
  }
}

// ── cli11 ─────────────────────────────────────────────────────────────────────

/// cli11 (AC-008): `test_script` defaults to `verb/test.d/l1` when not provided.
#[ test ]
fn cli11_test_script_defaults_to_l1()
{
  let dir = TempDir::new().unwrap();
  crb()
  .args( [ ".init", "image::my_img" ] )
  .current_dir( dir.path() )
  .output()
  .unwrap();

  let yml = fs::read_to_string( dir.path().join( "runbox/runbox.yml" ) ).unwrap();
  assert!(
    yml.contains( "test_script: verb/test.d/l1" ),
    "default test_script must be verb/test.d/l1, got:\n{yml}"
  );
}

// ── cli12 ─────────────────────────────────────────────────────────────────────

/// cli12 (AC-009): `test_script::custom/path` overrides default in `runbox.yml`.
#[ test ]
fn cli12_test_script_override_written_to_yml()
{
  let dir = TempDir::new().unwrap();
  crb()
  .args( [ ".init", "image::my_img", "test_script::my/custom/test" ] )
  .current_dir( dir.path() )
  .output()
  .unwrap();

  let yml = fs::read_to_string( dir.path().join( "runbox/runbox.yml" ) ).unwrap();
  assert!(
    yml.contains( "test_script: my/custom/test" ),
    "custom test_script must appear in runbox.yml, got:\n{yml}"
  );
  assert!(
    !yml.contains( "verb/test.d/l1" ),
    "default must NOT appear when overridden, got:\n{yml}"
  );
}

// ── cli13 ─────────────────────────────────────────────────────────────────────

/// cli13 (AC-010): running `.init` when `runbox/` already exists exits 1.
#[ test ]
fn cli13_existing_runbox_dir_exits_1()
{
  let dir = TempDir::new().unwrap();

  // Pre-create runbox/ to simulate existing state.
  fs::create_dir( dir.path().join( "runbox" ) ).unwrap();

  let out = crb()
  .args( [ ".init", "image::my_img" ] )
  .current_dir( dir.path() )
  .output()
  .unwrap();

  assert!( !out.status.success(), "must fail when runbox/ exists" );
  assert_eq!( out.status.code(), Some( 1 ), "exit code must be 1" );
  let stderr = String::from_utf8_lossy( &out.stderr );
  assert!(
    stderr.contains( "runbox/ already exists" ),
    "stderr must mention existing dir, got: {stderr}"
  );
}

// ── cli14 ─────────────────────────────────────────────────────────────────────

/// cli14 (AC-012): `runbox` canonical binary behaves identically to `crb`.
#[ test ]
fn cli14_runbox_binary_same_behaviour_as_crb()
{
  let dir = TempDir::new().unwrap();
  let out = runbox_bin()
  .current_dir( dir.path() )
  .output()
  .unwrap();

  // Empty argv → usage with exit 0.
  assert!( out.status.success(), "runbox binary must exit 0 on empty argv, got: {:?}", out.status );
  let stdout = String::from_utf8_lossy( &out.stdout );
  assert!( stdout.contains( ".init" ), "runbox usage must mention .init, got: {stdout}" );
}

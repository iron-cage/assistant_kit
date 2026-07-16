//! Feature and integration tests for the `.model.select` subprocess model preference command.
//!
//! Covers Feature 069 (`docs/feature/069_model_select_command.md` AC-01..AC-12),
//! command-level spec (`tests/docs/cli/command/20_model_select.md` IT-01..IT-12), and
//! feature spec (`tests/docs/feature/069_model_select_command.md` FT-01..FT-12).
//! All tests use a temporary HOME to avoid touching the real `~/.clr/` directory.
//!
//! Backing store: `~/.clr/config.toml`'s `model` key (task 410 migrated this
//! command off `claude_core::settings_io`/`~/.clr/prefs.json` onto task 407's
//! tiered-TOML primitive — same `model` key `--model`'s own config-file tier
//! reads). The command's `format::json` output shape is unchanged (still keyed
//! `subprocess_model`, per this command's own CLI-visible JSON contract) —
//! only the on-disk backing store and its key name changed.
//!
//! ## Test Matrix
//!
//! | ID    | Test Function                               | Condition                                                        | P/N |
//! |-------|---------------------------------------------|------------------------------------------------------------------|-----|
//! | IT-01 | `it01_get_unset_no_file`                    | No config.toml → `model.select: (unset)\n`. Exit 0.              | P   |
//! | IT-02 | `it02_get_shows_pinned_value`               | config.toml has `model` → prints value. Exit 0.         | P   |
//! | IT-03 | `it03_set_opus_pins_model`                  | `id::claude-opus-4-8` → file written; stdout `(pinned)`. Exit 0.| P   |
//! | IT-04 | `it04_set_sonnet_pins_model`                | `id::claude-sonnet-5` → file written. Exit 0.                    | P   |
//! | IT-05 | `it05_reset_removes_key_preserves_others`   | `reset::1` removes key; other keys preserved. Exit 0.           | P   |
//! | IT-06 | `it06_reset_no_file_is_idempotent`          | `reset::1` with no config.toml → exits 0.                        | P   |
//! | IT-07 | `it07_set_creates_file_when_absent`         | `id::VALUE` creates config.toml when absent. Exit 0.             | P   |
//! | IT-08 | `it08_set_preserves_other_keys`             | `id::VALUE` on existing config.toml → other keys preserved. Exit 0.| P |
//! | IT-09 | `it09_id_and_reset_mutual_exclusive`        | `id::VALUE reset::1` → exits 1 with `mutually exclusive`.       | N   |
//! | IT-10 | `it10_get_json_format`                      | `format::json` with preference set → JSON output. Exit 0.       | P   |
//! | IT-11 | `it11_model_select_in_help`                 | `.model.select` appears in `clp .help`. Exit 0.                 | P   |
//! | IT-12 | `it12_empty_id_exits_1`                     | `id::` (empty) → exits 1. Stderr indicates non-empty required.  | N   |
//! | FT-01 | `ft01_get_unset_no_file`                    | No config.toml → `model.select: (unset)\n`. Exit 0.              | P   |
//! | FT-02 | `ft02_get_shows_pinned_value`               | config.toml has `model` → prints value. Exit 0.         | P   |
//! | FT-03 | `ft03_set_opus_pins_model`                  | `id::claude-opus-4-8` → file written; stdout `(pinned)`. Exit 0.| P   |
//! | FT-04 | `ft04_set_sonnet_pins_model`                | `id::claude-sonnet-5` → file written. Exit 0.                    | P   |
//! | FT-05 | `ft05_reset_removes_key_preserves_others`   | `reset::1` removes key; other keys preserved. Exit 0.           | P   |
//! | FT-06 | `ft06_reset_no_file_is_idempotent`          | `reset::1` with no config.toml → exits 0.                        | P   |
//! | FT-07 | `ft07_set_creates_file_when_absent`         | `id::VALUE` creates config.toml when absent. Exit 0.             | P   |
//! | FT-08 | `ft08_set_preserves_other_keys`             | `id::VALUE` on existing config.toml → other keys preserved. Exit 0.| P |
//! | FT-09 | `ft09_id_and_reset_mutual_exclusive`        | `id::VALUE reset::1` → exits 1 with `mutually exclusive`.       | N   |
//! | FT-10 | `ft10_get_json_format`                      | `format::json` with preference set → JSON output. Exit 0.       | P   |
//! | FT-11 | `ft11_model_select_in_help`                 | `.model.select` appears in `clp .help`. Exit 0.                 | P   |
//! | FT-12 | `ft12_empty_id_exits_1`                     | `id::` (empty) → exits 1. Stderr indicates non-empty required.  | N   |

use crate::cli_runner::{ run_cs, run_cs_with_env, stdout, stderr, assert_exit };
use tempfile::TempDir;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Create an isolated temp HOME, create `~/.clr/` inside it, and optionally
/// seed `config.toml` with the given raw TOML content.
fn setup_home( config_content : Option< &str > ) -> TempDir
{
  let dir  = TempDir::new().unwrap();
  let clr  = dir.path().join( ".clr" );
  std::fs::create_dir_all( &clr ).unwrap();
  if let Some( content ) = config_content
  {
    std::fs::write( clr.join( "config.toml" ), content ).unwrap();
  }
  dir
}

/// Read `~/.clr/config.toml` from a temp home directory.
fn read_config( home : &std::path::Path ) -> Option< String >
{
  std::fs::read_to_string( home.join( ".clr" ).join( "config.toml" ) ).ok()
}

// ── IT: Integration Tests ─────────────────────────────────────────────────────

/// IT-01 (AC-01): No `~/.clr/config.toml` → `model.select: (unset)\n`. Exit 0.
#[ test ]
fn it01_get_unset_no_file()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "model.select: (unset)\n",
    "IT-01: expected 'model.select: (unset)\\n'" );
}

/// IT-02 (AC-02): config.toml has `model` → prints value. Exit 0.
#[ test ]
fn it02_get_shows_pinned_value()
{
  let dir  = setup_home( Some( "model = \"claude-opus-4-8\"\n" ) );
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "model.select: claude-opus-4-8\n",
    "IT-02: expected 'model.select: claude-opus-4-8\\n'" );
}

/// IT-03 (AC-03): `id::claude-opus-4-8` → config.toml written; stdout contains `(pinned)`. Exit 0.
#[ test ]
fn it03_set_opus_pins_model()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "id::claude-opus-4-8" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "(pinned)" ), "IT-03: stdout must contain '(pinned)'; got: {text:?}" );
  let config = read_config( dir.path() ).expect( "IT-03: config.toml must be created" );
  assert!( config.contains( "claude-opus-4-8" ),
    "IT-03: config.toml must contain 'claude-opus-4-8'; got: {config:?}" );
}

/// IT-04 (AC-04): `id::claude-sonnet-5` → config.toml written with correct value. Exit 0.
#[ test ]
fn it04_set_sonnet_pins_model()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "id::claude-sonnet-5" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let config = read_config( dir.path() ).expect( "IT-04: config.toml must be created" );
  assert!( config.contains( "claude-sonnet-5" ),
    "IT-04: config.toml must contain 'claude-sonnet-5'; got: {config:?}" );
}

/// IT-05 (AC-05): `reset::1` removes key; other keys preserved. Exit 0.
#[ test ]
fn it05_reset_removes_key_preserves_others()
{
  let dir  = setup_home( Some( "model = \"claude-opus-4-8\"\nother_key = \"val\"\n" ) );
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "reset::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "model.select: (reset to default)\n",
    "IT-05: expected reset confirmation message" );
  let config = read_config( dir.path() ).expect( "IT-05: config.toml must still exist" );
  assert!( !config.contains( "model" ),
    "IT-05: model must be removed; got: {config:?}" );
  assert!( config.contains( "other_key" ),
    "IT-05: other_key must be preserved; got: {config:?}" );
}

/// IT-06 (AC-06): `reset::1` with no `config.toml` → exits 0 idempotently.
#[ test ]
fn it06_reset_no_file_is_idempotent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "reset::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "model.select: (reset to default)\n",
    "IT-06: expected idempotent reset message even without file" );
}

/// IT-07 (AC-07): `id::VALUE` creates `config.toml` when absent. Exit 0.
#[ test ]
fn it07_set_creates_file_when_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "id::claude-opus-4-8" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( dir.path().join( ".clr" ).join( "config.toml" ).exists(),
    "IT-07: config.toml must be created when absent" );
}

/// IT-08 (AC-08): `id::VALUE` on existing config.toml → other keys preserved. Exit 0.
#[ test ]
fn it08_set_preserves_other_keys()
{
  let dir  = setup_home( Some( "other_key = \"val\"\n" ) );
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "id::claude-opus-4-8" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let config = read_config( dir.path() ).expect( "IT-08: config.toml must exist" );
  assert!( config.contains( "claude-opus-4-8" ),
    "IT-08: model must be written; got: {config:?}" );
  assert!( config.contains( "other_key" ),
    "IT-08: other_key must be preserved; got: {config:?}" );
}

/// IT-09 (AC-09): `id::VALUE reset::1` → exits 1; stderr contains `mutually exclusive`.
#[ test ]
fn it09_id_and_reset_mutual_exclusive()
{
  let out  = run_cs( &[ ".model.select", "id::claude-opus-4-8", "reset::1" ] );
  assert_exit( &out, 1 );
  let err  = stderr( &out );
  assert!( err.contains( "mutually exclusive" ),
    "IT-09: stderr must contain 'mutually exclusive'; got: {err:?}" );
}

/// IT-10 (AC-10): `format::json` with preference set → JSON with `subprocess_model`. Exit 0.
#[ test ]
fn it10_get_json_format()
{
  let dir  = setup_home( Some( "model = \"claude-opus-4-8\"\n" ) );
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "subprocess_model" ),
    "IT-10: JSON output must contain 'subprocess_model' key; got: {text:?}" );
  assert!( text.contains( "claude-opus-4-8" ),
    "IT-10: JSON output must contain the pinned model value; got: {text:?}" );
}

/// IT-11 (AC-11): `.model.select` appears in `clp .help`. Exit 0.
#[ test ]
fn it11_model_select_in_help()
{
  let out  = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".model.select" ),
    "IT-11: '.model.select' must appear in clp .help output; got: {text:?}" );
}

/// IT-12 (AC-12): `id::` (empty) → exits 1; stderr indicates non-empty required.
#[ test ]
fn it12_empty_id_exits_1()
{
  let out  = run_cs( &[ ".model.select", "id::" ] );
  assert_exit( &out, 1 );
  let err  = stderr( &out );
  assert!( err.contains( "non-empty" ) || err.contains( "id::" ),
    "IT-12: stderr must reference the empty id:: error; got: {err:?}" );
}

// ── FT: Feature Tests ─────────────────────────────────────────────────────────

/// FT-01 (AC-01): No config.toml → `model.select: (unset)\n`. Exit 0.
#[ test ]
fn ft01_get_unset_no_file()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "model.select: (unset)\n",
    "FT-01: expected 'model.select: (unset)\\n'" );
}

/// FT-02 (AC-02): config.toml has `model` → prints value. Exit 0.
#[ test ]
fn ft02_get_shows_pinned_value()
{
  let dir  = setup_home( Some( "model = \"claude-opus-4-8\"\n" ) );
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "model.select: claude-opus-4-8\n",
    "FT-02: expected 'model.select: claude-opus-4-8\\n'" );
}

/// FT-03 (AC-03): `id::claude-opus-4-8` → file written; stdout contains `(pinned)`. Exit 0.
#[ test ]
fn ft03_set_opus_pins_model()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "id::claude-opus-4-8" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "(pinned)" ), "FT-03: stdout must contain '(pinned)'; got: {text:?}" );
  let config = read_config( dir.path() ).expect( "FT-03: config.toml must be created" );
  assert!( config.contains( "claude-opus-4-8" ),
    "FT-03: config.toml must contain 'claude-opus-4-8'; got: {config:?}" );
}

/// FT-04 (AC-04): `id::claude-sonnet-5` → file written. Exit 0.
#[ test ]
fn ft04_set_sonnet_pins_model()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "id::claude-sonnet-5" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let config = read_config( dir.path() ).expect( "FT-04: config.toml must be created" );
  assert!( config.contains( "claude-sonnet-5" ),
    "FT-04: config.toml must contain 'claude-sonnet-5'; got: {config:?}" );
}

/// FT-05 (AC-05): `reset::1` removes key; other keys preserved. Exit 0.
#[ test ]
fn ft05_reset_removes_key_preserves_others()
{
  let dir  = setup_home( Some( "model = \"claude-opus-4-8\"\nother_key = \"val\"\n" ) );
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "reset::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "model.select: (reset to default)\n",
    "FT-05: expected reset confirmation" );
  let config = read_config( dir.path() ).expect( "FT-05: config.toml must still exist" );
  assert!( !config.contains( "model" ),
    "FT-05: model must be removed; got: {config:?}" );
  assert!( config.contains( "other_key" ),
    "FT-05: other_key must be preserved; got: {config:?}" );
}

/// FT-06 (AC-06): `reset::1` with no `config.toml` → exits 0 idempotently.
#[ test ]
fn ft06_reset_no_file_is_idempotent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "reset::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "model.select: (reset to default)\n",
    "FT-06: expected idempotent reset message" );
}

/// FT-07 (AC-07): `id::VALUE` creates `config.toml` when absent. Exit 0.
#[ test ]
fn ft07_set_creates_file_when_absent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "id::claude-opus-4-8" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert!( dir.path().join( ".clr" ).join( "config.toml" ).exists(),
    "FT-07: config.toml must be created when absent" );
}

/// FT-08 (AC-08): `id::VALUE` on existing config.toml → other keys preserved. Exit 0.
#[ test ]
fn ft08_set_preserves_other_keys()
{
  let dir  = setup_home( Some( "other_key = \"val\"\n" ) );
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "id::claude-opus-4-8" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let config = read_config( dir.path() ).expect( "FT-08: config.toml must exist" );
  assert!( config.contains( "claude-opus-4-8" ),
    "FT-08: model must be written; got: {config:?}" );
  assert!( config.contains( "other_key" ),
    "FT-08: other_key must be preserved; got: {config:?}" );
}

/// FT-09 (AC-09): `id::VALUE reset::1` → exits 1; stderr contains `mutually exclusive`.
#[ test ]
fn ft09_id_and_reset_mutual_exclusive()
{
  let out  = run_cs( &[ ".model.select", "id::claude-opus-4-8", "reset::1" ] );
  assert_exit( &out, 1 );
  let err  = stderr( &out );
  assert!( err.contains( "mutually exclusive" ),
    "FT-09: stderr must contain 'mutually exclusive'; got: {err:?}" );
}

/// FT-10 (AC-10): `format::json` with preference set → JSON with `subprocess_model`. Exit 0.
#[ test ]
fn ft10_get_json_format()
{
  let dir  = setup_home( Some( "model = \"claude-opus-4-8\"\n" ) );
  let home = dir.path().to_str().unwrap();
  let out  = run_cs_with_env( &[ ".model.select", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "subprocess_model" ),
    "FT-10: JSON output must contain 'subprocess_model' key; got: {text:?}" );
  assert!( text.contains( "claude-opus-4-8" ),
    "FT-10: JSON output must contain the pinned model value; got: {text:?}" );
}

/// FT-11 (AC-11): `.model.select` appears in `clp .help`. Exit 0.
#[ test ]
fn ft11_model_select_in_help()
{
  let out  = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".model.select" ),
    "FT-11: '.model.select' must appear in clp .help output; got: {text:?}" );
}

/// FT-12 (AC-12): `id::` (empty) → exits 1; stderr indicates non-empty required.
#[ test ]
fn ft12_empty_id_exits_1()
{
  let out  = run_cs( &[ ".model.select", "id::" ] );
  assert_exit( &out, 1 );
  let err  = stderr( &out );
  assert!( err.contains( "non-empty" ) || err.contains( "id::" ),
    "FT-12: stderr must reference the empty id:: error; got: {err:?}" );
}

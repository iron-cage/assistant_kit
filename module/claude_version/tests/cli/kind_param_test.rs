//! Kind filter and `ParamKind` type tests for `claude_version`.
//!
//! Implements test cases from:
//! - `tests/docs/cli/param/13_kind.md` (EC-1 through EC-7)
//! - `tests/docs/cli/type/08_param_kind.md` (TC-1 through TC-6)
//!
//! # Coverage Map
//!
//! | Spec | ID | Function |
//! |------|----|----------|
//! | cli/param/13_kind.md | EC-1 | `kind_ec1_config_shows_config_params` |
//! | cli/param/13_kind.md | EC-2 | `kind_ec2_env_shows_env_params` |
//! | cli/param/13_kind.md | EC-3 | `kind_ec3_absent_shows_all_params` |
//! | cli/param/13_kind.md | EC-4 | `kind_ec4_invalid_exits_1` |
//! | cli/param/13_kind.md | EC-5 | `kind_ec5_empty_exits_1` |
//! | cli/param/13_kind.md | EC-6 | `kind_ec6_uppercase_exits_1` |
//! | cli/param/13_kind.md | EC-7 | `kind_ec7_ignored_when_key_present` |
//! | cli/type/08_param_kind.md | TC-1 | `kind_tc1_config_shows_config_params_only` |
//! | cli/type/08_param_kind.md | TC-2 | `kind_tc2_env_shows_env_params_only` |
//! | cli/type/08_param_kind.md | TC-3 | `kind_tc3_absent_shows_all_params` |
//! | cli/type/08_param_kind.md | TC-4 | `kind_tc4_mixed_case_exits_1` |
//! | cli/type/08_param_kind.md | TC-5 | `kind_tc5_unknown_variant_exits_1` |
//! | cli/type/08_param_kind.md | TC-6 | `kind_tc6_empty_exits_1` |

use tempfile::TempDir;

use crate::subprocess_helpers::{ assert_exit, run_clv_with_env, stderr, stdout };

// ─── EC-1 (cli/param/13_kind.md): kind::config shows config-key params only ──

// EC-1: kind::config → config params (model, theme) present; env-only (bash_timeout) absent
#[ test ]
fn kind_ec1_config_shows_config_params()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params", "kind::config" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "model" ),         "kind::config must include model (config param): {text}" );
  assert!( text.contains( "theme" ),         "kind::config must include theme (config-only param): {text}" );
  assert!( !text.contains( "bash_timeout" ), "kind::config must exclude bash_timeout (env-only): {text}" );
}

// ─── EC-2 (cli/param/13_kind.md): kind::env shows env-var params only ─────────

// EC-2: kind::env → env params (model, bash_timeout) present; config-only (theme) absent
#[ test ]
fn kind_ec2_env_shows_env_params()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params", "kind::env" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "model" ),        "kind::env must include model (has env form): {text}" );
  assert!( text.contains( "bash_timeout" ), "kind::env must include bash_timeout (env param): {text}" );
  assert!( !text.contains( "theme" ),       "kind::env must exclude theme (config-only): {text}" );
}

// ─── EC-3 (cli/param/13_kind.md): absent kind:: shows all params ──────────────

// EC-3: no kind:: → all params; ≥35 entries; both config and env-only params present
#[ test ]
fn kind_ec3_absent_shows_all_params()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let entry_count = text.lines()
    .filter( |l| !l.starts_with( ' ' ) && !l.is_empty() )
    .count();
  assert!(
    entry_count >= 35,
    "absent kind:: must show all params (≥35), got {entry_count}: {text}"
  );
}

// ─── EC-4 (cli/param/13_kind.md): unknown kind value → exit 1 ────────────────

// EC-4: kind::invalid → exit 1; stderr mentions valid values (config, env)
#[ test ]
fn kind_ec4_invalid_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params", "kind::invalid" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "config" ) || err.contains( "env" ),
    "kind::invalid must mention valid values (config, env): {err}"
  );
}

// ─── EC-5 (cli/param/13_kind.md): empty kind:: value → exit 1 ────────────────

// EC-5: kind:: (empty) → exit 1
#[ test ]
fn kind_ec5_empty_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params", "kind::" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "empty kind:: must produce an error message: {err}" );
}

// ─── EC-6 (cli/param/13_kind.md): uppercase kind value → exit 1 ──────────────

// EC-6: kind::CONFIG (all-caps) → exit 1; kind:: is case-sensitive
#[ test ]
fn kind_ec6_uppercase_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params", "kind::CONFIG" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "kind::CONFIG rejection must produce an error message: {err}" );
}

// ─── EC-7 (cli/param/13_kind.md): kind:: ignored when key:: present ──────────

// EC-7: key::model kind::env → exit 0; single-param deep-dive for model; kind:: superseded
#[ test ]
fn kind_ec7_ignored_when_key_present()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params", "key::model", "kind::env" ],
    &[ ( "HOME", home ), ( "CLAUDE_MODEL", "" ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // key:: triggers single-param mode; kind:: is ignored; model forms must appear
  assert!( text.contains( "--model" ),     "kind:: ignored — must show --model CLI form: {text}" );
  assert!( text.contains( "CLAUDE_MODEL" ), "kind:: ignored — must show CLAUDE_MODEL env form: {text}" );
}

// ─── TC-1 (cli/type/08_param_kind.md): config variant shows config params ─────

// TC-1: kind::config → output contains only params with a config key form; env-only absent
#[ test ]
fn kind_tc1_config_shows_config_params_only()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params", "kind::config" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "model" ),         "kind::config must show config-key params: {text}" );
  assert!( text.contains( "theme" ),         "kind::config must show config-only theme: {text}" );
  assert!( !text.contains( "bash_timeout" ), "kind::config must not show env-only bash_timeout: {text}" );
}

// ─── TC-2 (cli/type/08_param_kind.md): env variant shows env-var params ───────

// TC-2: kind::env → output contains only params with an env var form; config-only absent
#[ test ]
fn kind_tc2_env_shows_env_params_only()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params", "kind::env" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "model" ),        "kind::env must show model (has env form): {text}" );
  assert!( text.contains( "bash_timeout" ), "kind::env must show env-var params: {text}" );
  assert!( !text.contains( "theme" ),       "kind::env must not show config-only theme: {text}" );
}

// ─── TC-3 (cli/type/08_param_kind.md): absent ParamKind → all params ──────────

// TC-3: no kind:: → all catalog params present (both config and env-only variants)
#[ test ]
fn kind_tc3_absent_shows_all_params()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "model" ),        "absent kind:: must include config params: {text}" );
  assert!( text.contains( "bash_timeout" ), "absent kind:: must include env-only params: {text}" );
}

// ─── TC-4 (cli/type/08_param_kind.md): mixed-case variant rejected ────────────

// TC-4: kind::Config (mixed case) → exit 1; parsing is exact string match
#[ test ]
fn kind_tc4_mixed_case_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params", "kind::Config" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "kind::Config rejection must produce error message: {err}" );
}

// ─── TC-5 (cli/type/08_param_kind.md): unknown variant rejected ───────────────

// TC-5: kind::all → exit 1; 'all' is not a valid ParamKind; error mentions valid values
#[ test ]
fn kind_tc5_unknown_variant_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params", "kind::all" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "config" ) || err.contains( "env" ),
    "kind::all must mention expected variants (config, env): {err}"
  );
}

// ─── TC-6 (cli/type/08_param_kind.md): empty value rejected ──────────────────

// TC-6: kind:: (empty) → exit 1; empty string is not a valid ParamKind; error names expected variants
#[ test ]
fn kind_tc6_empty_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out  = run_clv_with_env(
    &[ ".params", "kind::" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  // Type-level validation: error must name the expected ParamKind variants
  assert!(
    err.contains( "config" ) || err.contains( "env" ),
    "ParamKind empty error must mention expected variants (config, env): {err}"
  );
}

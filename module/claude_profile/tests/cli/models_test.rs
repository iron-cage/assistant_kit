//! Feature and integration tests for the `.models` model list command.
//!
//! Covers Feature 068 (`docs/feature/068_models_list_command.md` AC-01..AC-10),
//! command-level spec (`tests/docs/cli/command/19_models.md` IT-01..IT-10), and
//! feature spec (`tests/docs/feature/068_models_list_command.md` FT-01..FT-10).
//! All tests use `offline::1` to avoid network dependency in CI.
//!
//! ## Test Matrix
//!
//! | ID    | Test Function                          | Condition                                                        | P/N |
//! |-------|----------------------------------------|------------------------------------------------------------------|-----|
//! | IT-01 | `it01_offline_contains_opus`           | `offline::1` — stdout contains `claude-opus-4-8`                | P   |
//! | IT-02 | `it02_offline_contains_sonnet`         | `offline::1` — stdout contains `claude-sonnet-5`                 | P   |
//! | IT-03 | `it03_offline_contains_haiku`          | `offline::1` — stdout contains `claude-haiku-4-5-20251001`      | P   |
//! | IT-04 | `it04_offline_table_has_header`        | `offline::1 format::table` — first line contains `ID`            | P   |
//! | IT-05 | `it05_offline_text_one_per_line`       | `offline::1 format::text` — one ID per line; no `|` present     | P   |
//! | IT-06 | `it06_offline_json_valid_array`        | `offline::1 format::json` — parseable JSON array with `id` field | P   |
//! | IT-07 | `it07_name_filter_opus_only`           | `offline::1 name::opus` — all IDs contain `opus`; others absent  | P   |
//! | IT-08 | `it08_name_filter_no_match`            | `offline::1 name::zz_no_match` — empty output; exits 0          | P   |
//! | IT-09 | `it09_models_in_help_output`           | `.models` appears in `clp .help` output                         | P   |
//! | IT-10 | `it10_name_filter_substring`           | `offline::1 name::claude-opus` — substring match; haiku/sonnet absent | P |
//! | FT-01 | `ft01_offline_contains_opus`           | `offline::1` — stdout contains `claude-opus-4-8`                | P   |
//! | FT-02 | `ft02_offline_contains_sonnet`         | `offline::1` — stdout contains `claude-sonnet-5`                 | P   |
//! | FT-03 | `ft03_offline_contains_haiku`          | `offline::1` — stdout contains `claude-haiku-4-5-20251001`      | P   |
//! | FT-04 | `ft04_offline_table_has_header`        | `offline::1 format::table` — first line contains `ID`            | P   |
//! | FT-05 | `ft05_offline_text_one_per_line`       | `offline::1 format::text` — one ID per line; no `|` present     | P   |
//! | FT-06 | `ft06_offline_json_valid_array`        | `offline::1 format::json` — parseable JSON array with `id` field | P   |
//! | FT-07 | `ft07_name_filter_opus_only`           | `offline::1 name::opus` — all IDs contain `opus`; others absent  | P   |
//! | FT-08 | `ft08_name_filter_no_match`            | `offline::1 name::zz_no_match` — empty output; exits 0          | P   |
//! | FT-09 | `ft09_models_in_help_output`           | `.models` appears in `clp .help` output                         | P   |
//! | FT-10 | `ft10_name_filter_substring`           | `offline::1 name::claude-opus` — substring match; haiku/sonnet absent | P |

use crate::cli_runner::{ run_cs, stdout, assert_exit };

// ── IT: Integration Tests ─────────────────────────────────────────────────────

/// IT-01 (AC-01): `clp .models offline::1` — stdout contains `claude-opus-4-8`. Exit 0.
#[ test ]
fn it01_offline_contains_opus()
{
  let out  = run_cs( &[ ".models", "offline::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-opus-4-8" ),
    "IT-01: expected stdout to contain 'claude-opus-4-8'; got: {text:?}" );
}

/// IT-02 (AC-02): `clp .models offline::1` — stdout contains `claude-sonnet-5`. Exit 0.
#[ test ]
fn it02_offline_contains_sonnet()
{
  let out  = run_cs( &[ ".models", "offline::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-sonnet-5" ),
    "IT-02: expected stdout to contain 'claude-sonnet-5'; got: {text:?}" );
}

/// IT-03 (AC-03): `clp .models offline::1` — stdout contains `claude-haiku-4-5-20251001`. Exit 0.
#[ test ]
fn it03_offline_contains_haiku()
{
  let out  = run_cs( &[ ".models", "offline::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-haiku-4-5-20251001" ),
    "IT-03: expected stdout to contain 'claude-haiku-4-5-20251001'; got: {text:?}" );
}

/// IT-04 (AC-04): `clp .models offline::1 format::table` — first line contains `ID`. Exit 0.
#[ test ]
fn it04_offline_table_has_header()
{
  let out   = run_cs( &[ ".models", "offline::1", "format::table" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  let first = text.lines().next().unwrap_or( "" );
  assert!( first.contains( "ID" ),
    "IT-04: expected first line to contain 'ID' header; got: {first:?}" );
}

/// IT-05 (AC-05): `clp .models offline::1 format::text` — one ID per line; no `|`. Exit 0.
#[ test ]
fn it05_offline_text_one_per_line()
{
  let out  = run_cs( &[ ".models", "offline::1", "format::text" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( '|' ),
    "IT-05: format::text must not contain '|' table separators; got: {text:?}" );
  for line in text.lines()
  {
    assert!(
      line.starts_with( "claude-" ),
      "IT-05: each line must be a model ID starting with 'claude-'; got: {line:?}",
    );
  }
}

/// IT-06 (AC-06): `clp .models offline::1 format::json` — valid JSON array with `id` field. Exit 0.
#[ test ]
fn it06_offline_json_valid_array()
{
  let out  = run_cs( &[ ".models", "offline::1", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let arr  : serde_json::Value = serde_json::from_str( text.trim() )
    .unwrap_or_else( |e| panic!( "IT-06: stdout is not valid JSON: {e}\nGot: {text:?}" ) );
  let arr = arr.as_array()
    .unwrap_or_else( || panic!( "IT-06: JSON is not an array; got: {text:?}" ) );
  assert!( !arr.is_empty(), "IT-06: JSON array must be non-empty" );
  for elem in arr
  {
    assert!(
      elem.get( "id" ).and_then( | v | v.as_str() ).is_some(),
      "IT-06: each element must have an 'id' string field; elem: {elem}",
    );
  }
}

/// IT-07 (AC-07): `clp .models offline::1 name::opus` — all returned IDs contain `opus`. Exit 0.
#[ test ]
fn it07_name_filter_opus_only()
{
  let out  = run_cs( &[ ".models", "offline::1", "name::opus", "format::text" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "claude-sonnet-5" ),
    "IT-07: sonnet must be absent when name::opus filter active; got: {text:?}" );
  assert!( !text.contains( "claude-haiku" ),
    "IT-07: haiku must be absent when name::opus filter active; got: {text:?}" );
  for line in text.lines().filter( | l | !l.is_empty() )
  {
    assert!(
      line.to_ascii_lowercase().contains( "opus" ),
      "IT-07: all returned IDs must contain 'opus'; got line: {line:?}",
    );
  }
  // At least one opus model must be returned
  assert!( text.contains( "claude-opus" ),
    "IT-07: at least one opus model must be present; got: {text:?}" );
}

/// IT-08 (AC-08): `clp .models offline::1 name::zz_no_match` — empty output; exits 0.
#[ test ]
fn it08_name_filter_no_match()
{
  let out  = run_cs( &[ ".models", "offline::1", "name::zz_no_match" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "claude-" ),
    "IT-08: output must contain no model IDs when name filter has no matches; got: {text:?}" );
}

/// IT-09 (AC-09): `.models` appears in `clp .help` output. Exit 0.
#[ test ]
fn it09_models_in_help_output()
{
  let out  = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".models" ),
    "IT-09: '.models' must appear in clp .help output; got: {text:?}" );
}

/// IT-10 (AC-10): `clp .models offline::1 name::claude-opus` — substring match; haiku+sonnet absent. Exit 0.
#[ test ]
fn it10_name_filter_substring()
{
  let out  = run_cs( &[ ".models", "offline::1", "name::claude-opus", "format::text" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "claude-sonnet" ),
    "IT-10: sonnet must be absent when name::claude-opus filter active; got: {text:?}" );
  assert!( !text.contains( "claude-haiku" ),
    "IT-10: haiku must be absent when name::claude-opus filter active; got: {text:?}" );
  for line in text.lines().filter( | l | !l.is_empty() )
  {
    assert!(
      line.to_ascii_lowercase().contains( "claude-opus" ),
      "IT-10: all returned IDs must contain 'claude-opus'; got line: {line:?}",
    );
  }
}

// ── FT: Feature Tests ─────────────────────────────────────────────────────────

/// FT-01 (AC-01): `clp .models offline::1` — stdout contains `claude-opus-4-8`. Exit 0.
#[ test ]
fn ft01_offline_contains_opus()
{
  let out  = run_cs( &[ ".models", "offline::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-opus-4-8" ),
    "FT-01: expected stdout to contain 'claude-opus-4-8'; got: {text:?}" );
}

/// FT-02 (AC-02): `clp .models offline::1` — stdout contains `claude-sonnet-5`. Exit 0.
#[ test ]
fn ft02_offline_contains_sonnet()
{
  let out  = run_cs( &[ ".models", "offline::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-sonnet-5" ),
    "FT-02: expected stdout to contain 'claude-sonnet-5'; got: {text:?}" );
}

/// FT-03 (AC-03): `clp .models offline::1` — stdout contains `claude-haiku-4-5-20251001`. Exit 0.
#[ test ]
fn ft03_offline_contains_haiku()
{
  let out  = run_cs( &[ ".models", "offline::1" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "claude-haiku-4-5-20251001" ),
    "FT-03: expected stdout to contain 'claude-haiku-4-5-20251001'; got: {text:?}" );
}

/// FT-04 (AC-04): `clp .models offline::1 format::table` — first line contains `ID`. Exit 0.
#[ test ]
fn ft04_offline_table_has_header()
{
  let out   = run_cs( &[ ".models", "offline::1", "format::table" ] );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  let first = text.lines().next().unwrap_or( "" );
  assert!( first.contains( "ID" ),
    "FT-04: expected first line to contain 'ID' header; got: {first:?}" );
}

/// FT-05 (AC-05): `clp .models offline::1 format::text` — one ID per line; no `|`. Exit 0.
#[ test ]
fn ft05_offline_text_one_per_line()
{
  let out  = run_cs( &[ ".models", "offline::1", "format::text" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( '|' ),
    "FT-05: format::text must not contain '|' table separators; got: {text:?}" );
  for line in text.lines()
  {
    assert!(
      line.starts_with( "claude-" ),
      "FT-05: each line must be a model ID starting with 'claude-'; got: {line:?}",
    );
  }
}

/// FT-06 (AC-06): `clp .models offline::1 format::json` — valid JSON array with `id` field. Exit 0.
#[ test ]
fn ft06_offline_json_valid_array()
{
  let out  = run_cs( &[ ".models", "offline::1", "format::json" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let arr  : serde_json::Value = serde_json::from_str( text.trim() )
    .unwrap_or_else( |e| panic!( "FT-06: stdout is not valid JSON: {e}\nGot: {text:?}" ) );
  let arr = arr.as_array()
    .unwrap_or_else( || panic!( "FT-06: JSON is not an array; got: {text:?}" ) );
  assert!( !arr.is_empty(), "FT-06: JSON array must be non-empty" );
  for elem in arr
  {
    assert!(
      elem.get( "id" ).and_then( | v | v.as_str() ).is_some(),
      "FT-06: each element must have an 'id' string field; elem: {elem}",
    );
  }
}

/// FT-07 (AC-07): `clp .models offline::1 name::opus` — all returned IDs contain `opus`. Exit 0.
#[ test ]
fn ft07_name_filter_opus_only()
{
  let out  = run_cs( &[ ".models", "offline::1", "name::opus", "format::text" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "claude-sonnet-5" ),
    "FT-07: sonnet must be absent when name::opus filter active; got: {text:?}" );
  assert!( !text.contains( "claude-haiku" ),
    "FT-07: haiku must be absent when name::opus filter active; got: {text:?}" );
  for line in text.lines().filter( | l | !l.is_empty() )
  {
    assert!(
      line.to_ascii_lowercase().contains( "opus" ),
      "FT-07: all returned IDs must contain 'opus'; got line: {line:?}",
    );
  }
  assert!( text.contains( "claude-opus" ),
    "FT-07: at least one opus model must be present; got: {text:?}" );
}

/// FT-08 (AC-08): `clp .models offline::1 name::zz_no_match` — empty output; exits 0.
#[ test ]
fn ft08_name_filter_no_match()
{
  let out  = run_cs( &[ ".models", "offline::1", "name::zz_no_match" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "claude-" ),
    "FT-08: output must contain no model IDs when name filter has no matches; got: {text:?}" );
}

/// FT-09 (AC-09): `.models` appears in `clp .help` output. Exit 0.
#[ test ]
fn ft09_models_in_help_output()
{
  let out  = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".models" ),
    "FT-09: '.models' must appear in clp .help output; got: {text:?}" );
}

/// FT-10 (AC-10): `clp .models offline::1 name::claude-opus` — substring match. Exit 0.
#[ test ]
fn ft10_name_filter_substring()
{
  let out  = run_cs( &[ ".models", "offline::1", "name::claude-opus", "format::text" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "claude-sonnet" ),
    "FT-10: sonnet must be absent when name::claude-opus filter active; got: {text:?}" );
  assert!( !text.contains( "claude-haiku" ),
    "FT-10: haiku must be absent when name::claude-opus filter active; got: {text:?}" );
  for line in text.lines().filter( | l | !l.is_empty() )
  {
    assert!(
      line.to_ascii_lowercase().contains( "claude-opus" ),
      "FT-10: all returned IDs must contain 'claude-opus'; got line: {line:?}",
    );
  }
}

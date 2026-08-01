//! Integration tests: `inference_provider` account field (Feature 072) —
//! `.account.save inference_provider::` real dispatch and `.accounts` `Provider` column.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! Maps function names to the Test Matrix rows in
//! `task/claude_profile/436_inference_provider_cli_and_gate10.md`.
//!
//! | Function | Row | Condition | P/N |
//! |----------|-----|-----------|-----|
//! | `t01_inference_provider_kimi_writes_key` | T01 | `name::kimi@test.com inference_provider::kimi` → exit 0, key written | P |
//! | `t02_inference_provider_omitted_writes_no_key` | T02 | omitted → exit 0, no key written | P |
//! | `t03_inference_provider_empty_value_exits_1_no_write` | T03 | empty value → exit 1, no file written | N |
//! | `t05_accounts_default_shows_provider_column_and_anthropic_fallback` | T05 | no `cols::` → `Provider` shown, empty reads `anthropic` | P |
//! | `t06_accounts_cols_minus_inference_provider_omits_column` | T06 | `cols::-inference_provider` → column entirely absent | P |
//! | `t07_provider_select_get_default_anthropic` | T07 | `.provider.select` never set → exit 0, prints `anthropic` | P |
//! | `t08_provider_select_set_kimi_persists_and_confirms` | T08 | `id::kimi` → exit 0, persists + confirmed via second get | P |
//! | `t09_provider_select_reset_preserves_model_key` | T09 | `reset::1` after set → removes `provider`, preserves `model` | P |
//! | `t10_provider_select_empty_id_exits_1` | T10 | `id::` (empty) → exit 1 | N |
//! | `t11_provider_select_id_and_reset_mutually_exclusive` | T11 | `id::kimi reset::1` together → exit 1 | N |
//! | `t12_provider_select_json_format` | T12 | `format::json` get mode → `{"provider":"VALUE"}` | P |
//! | `t16_accounts_json_includes_inference_provider_regardless_of_cols` | T16 | `format::json` → key present in every row despite `cols::-inference_provider` | P |

use crate::cli_runner::
{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, account_exists,
  write_account_inference_provider,
  read_account_meta,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── T01–T03: `.account.save inference_provider::` write mechanics ─────────────

/// T01 (AC-01): `.account.save name::kimi@test.com inference_provider::kimi` exits 0 and
/// writes `"inference_provider": "kimi"` to `kimi@test.com.json`. Account name must be a
/// valid email per `tests/docs/cli/param/01_name.md` — `kimi` here names the provider value,
/// not the account name.
#[ test ]
fn t01_inference_provider_kimi_writes_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::kimi@test.com", "inference_provider::kimi" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "kimi@test.com" );
  assert_eq!( meta[ "inference_provider" ], serde_json::json!( "kimi" ), "T01: inference_provider must be 'kimi', got:\n{meta}" );
}

/// T02 (AC-02): `.account.save name::alice@acme.com` with no `inference_provider::`
/// exits 0 and writes no `inference_provider` key at all.
#[ test ]
fn t02_inference_provider_omitted_writes_no_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "alice@acme.com" );
  assert!( meta.get( "inference_provider" ).is_none(), "T02: inference_provider key must be absent, got:\n{meta}" );
}

/// T03 (AC-03): `.account.save name::kimi@test.com inference_provider::` (empty value)
/// exits 1, names `inference_provider::` in stderr, and writes no credential file.
#[ test ]
fn t03_inference_provider_empty_value_exits_1_no_write()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::kimi@test.com", "inference_provider::" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "inference_provider::" ), "T03: stderr must name inference_provider::, got:\n{err}" );

  assert!( !account_exists( dir.path(), "kimi@test.com" ), "T03: no credentials file must be written on rejection" );
}

// ── T05–T06: `.accounts` `Provider` column visibility ──────────────────────────

/// T05 (AC-05): `.accounts` with no `cols::` shows the `Provider` column for
/// every account, reading `anthropic` for accounts with no `inference_provider` key.
#[ test ]
fn t05_accounts_default_shows_provider_column_and_anthropic_fallback()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "kimi@test.com",  "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_inference_provider( dir.path(), "kimi@test.com", "kimi" );

  let out = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Provider: kimi" ), "T05: kimi's Provider line must show 'kimi', got:\n{text}" );
  assert!( text.contains( "Provider: anthropic" ), "T05: alice's Provider line must fall back to 'anthropic', got:\n{text}" );
}

/// T06 (AC-06): `.accounts cols::-inference_provider` omits the `Provider`
/// column entirely — the `Provider:` label itself must not appear anywhere.
#[ test ]
fn t06_accounts_cols_minus_inference_provider_omits_column()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "kimi@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_inference_provider( dir.path(), "kimi@test.com", "kimi" );

  let out = run_cs_with_env(
    &[ ".accounts", "cols::-inference_provider" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Provider:" ), "T06: Provider: must be suppressed, got:\n{text}" );
}

// ── T07–T12: `.provider.select` get/set/reset command ──────────────────────────

/// Read `~/.clr/config.toml` from a temp home directory; `None` if absent.
fn read_clr_config( home : &std::path::Path ) -> Option< String >
{
  std::fs::read_to_string( home.join( ".clr" ).join( "config.toml" ) ).ok()
}

/// T07 (AC-07): `.provider.select` (no params), never set → exit 0, prints
/// `provider.select: anthropic` — never an `(unset)`-style sentinel.
#[ test ]
fn t07_provider_select_get_default_anthropic()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".provider.select" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "provider.select: anthropic\n",
    "T07: expected default 'provider.select: anthropic\\n', got:\n{}", stdout( &out ) );
}

/// T08 (AC-08): `.provider.select id::kimi` → exit 0, prints
/// `provider.select: kimi (selected)`, persists `provider = "kimi"` to
/// `~/.clr/config.toml`'s user tier, and a subsequent get confirms it.
#[ test ]
fn t08_provider_select_set_kimi_persists_and_confirms()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".provider.select", "id::kimi" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "provider.select: kimi (selected)\n",
    "T08: expected '(selected)' confirmation, got:\n{}", stdout( &out ) );

  let config = read_clr_config( dir.path() ).expect( "T08: config.toml must be created" );
  assert!( config.contains( "provider" ) && config.contains( "kimi" ),
    "T08: config.toml must persist provider = \"kimi\", got:\n{config}" );

  // Confirm via a second get.
  let out2 = run_cs_with_env( &[ ".provider.select" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );
  assert_eq!( stdout( &out2 ), "provider.select: kimi\n",
    "T08: second get must confirm persisted value, got:\n{}", stdout( &out2 ) );
}

/// T09 (AC-11): `.provider.select reset::1` after a selection → exit 0,
/// prints `provider.select: anthropic (reset to default)`, removes the
/// `provider` key, and preserves an unrelated pre-set `model` key (written
/// via `.model.select`, not this command).
#[ test ]
fn t09_provider_select_reset_preserves_model_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let model_out = run_cs_with_env( &[ ".model.select", "id::claude-opus-4-8" ], &[ ( "HOME", home ) ] );
  assert_exit( &model_out, 0 );
  let provider_out = run_cs_with_env( &[ ".provider.select", "id::kimi" ], &[ ( "HOME", home ) ] );
  assert_exit( &provider_out, 0 );

  let out = run_cs_with_env( &[ ".provider.select", "reset::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "provider.select: anthropic (reset to default)\n",
    "T09: expected reset confirmation, got:\n{}", stdout( &out ) );

  let config = read_clr_config( dir.path() ).expect( "T09: config.toml must still exist" );
  assert!( !config.contains( "provider" ),
    "T09: provider key must be removed, got:\n{config}" );
  assert!( config.contains( "claude-opus-4-8" ),
    "T09: model key from .model.select must be preserved, got:\n{config}" );
}

/// T10 (AC-09): `.provider.select id::` (empty) → exit 1, stderr names
/// `id:: must be a non-empty provider name`, no config file touched.
#[ test ]
fn t10_provider_select_empty_id_exits_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".provider.select", "id::" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "id:: must be a non-empty provider name" ),
    "T10: stderr must name the empty-id error, got:\n{err}" );
  assert!( read_clr_config( dir.path() ).is_none(),
    "T10: no config.toml must be written on rejection" );
}

/// T11 (AC-09/AC-10): `.provider.select id::kimi reset::1` (both given) →
/// exit 1, stderr names `id:: and reset::1 are mutually exclusive`.
#[ test ]
fn t11_provider_select_id_and_reset_mutually_exclusive()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".provider.select", "id::kimi", "reset::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "id:: and reset::1 are mutually exclusive" ),
    "T11: stderr must name the mutual-exclusion error, got:\n{err}" );
  assert!( read_clr_config( dir.path() ).is_none(),
    "T11: no config.toml must be written on rejection" );
}

/// T12 (AC-13): `.provider.select format::json` (get mode) → exit 0,
/// stdout `{"provider":"VALUE"}` — JSON key is `provider`, distinct from
/// `.accounts`' per-row `inference_provider` key.
#[ test ]
fn t12_provider_select_json_format()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".provider.select", "id::kimi" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let out2 = run_cs_with_env( &[ ".provider.select", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );
  let text = stdout( &out2 );
  let val  : serde_json::Value = serde_json::from_str( text.trim() ).expect( "T12: output must be valid JSON" );
  assert_eq!( val[ "provider" ], serde_json::json!( "kimi" ),
    "T12: JSON must be keyed 'provider' with the selected value, got:\n{text}" );
}

// ── T16: `.accounts format::json` unconditional inclusion ──────────────────────

/// T16: `.accounts format::json` includes `inference_provider` in every row's
/// JSON object unconditionally — even when `cols::-inference_provider` is set,
/// proving the JSON renderer ignores `cols::` for this field (matches the
/// existing always-include `host`/`owner` precedent).
#[ test ]
fn t16_accounts_json_includes_inference_provider_regardless_of_cols()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "kimi@test.com",  "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_inference_provider( dir.path(), "kimi@test.com", "kimi" );

  let out = run_cs_with_env(
    &[ ".accounts", "format::json", "cols::-inference_provider" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let val  : serde_json::Value = serde_json::from_str( &text ).expect( "T16: output must be valid JSON" );
  let rows = val.as_array().expect( "T16: output must be a JSON array" );
  assert_eq!( rows.len(), 2, "T16: expected 2 rows, got:\n{text}" );
  for row in rows
  {
    assert!( row.get( "inference_provider" ).is_some(), "T16: every row must include inference_provider key even with cols::-inference_provider, got:\n{text}" );
  }
  let kimi_row = rows.iter().find( |r| r[ "name" ] == "kimi@test.com" ).expect( "T16: kimi row must exist" );
  assert_eq!( kimi_row[ "inference_provider" ], serde_json::json!( "kimi" ), "T16: kimi row must show 'kimi', got:\n{text}" );
  let alice_row = rows.iter().find( |r| r[ "name" ] == "alice@test.com" ).expect( "T16: alice row must exist" );
  assert_eq!( alice_row[ "inference_provider" ], serde_json::json!( "anthropic" ), "T16: alice row must fall back to 'anthropic', got:\n{text}" );
}

//! Integration tests: `inference_provider` account field (Feature 072) —
//! `.account.save inference_provider::` real dispatch and `.accounts` `Provider` column.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! Maps function names to the Test Matrix rows in
//! `task/claude_profile/436_inference_provider_cli_and_gate10.md` (t01–t16) and
//! `task/claude_profile/533_close_provider_stack_test_coverage_gaps.md` (t17–t19).
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
//! | `t17_provider_select_reset_without_config_idempotent` | 533-T1 | `reset::1`, no `config.toml` ever created → exit 0 twice, identical output, no file created | P |
//! | `t18_provider_select_get_ignores_active_account_provider` | 533-T2 | active account carries `kimi`, no `provider` key → get prints `anthropic` | P |
//! | `t19_save_host_tags_inference_provider_combined_default_column` | 533-T3 | `host::`+`tags::`+`inference_provider::` in one save → 3 fields written, `Provider` shown by default | P |

use crate::cli_runner::
{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, account_exists,
  write_account_inference_provider,
  read_account_meta,
  credential_store_dir,
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
/// via `.model scope::subprocess model::VALUE`, not this command).
#[ test ]
fn t09_provider_select_reset_preserves_model_key()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let model_out = run_cs_with_env( &[ ".model", "scope::subprocess", "model::claude-opus-4-8" ], &[ ( "HOME", home ) ] );
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
    "T09: model key from .model scope::subprocess must be preserved, got:\n{config}" );
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

// ── T17–T19: Task 533 coverage-gap closure (072 FT-12/FT-16, param-group CC-5) ──

/// T17 (533-T1, Feature 072 AC-12 / FT-12): `.provider.select reset::1` with no
/// `~/.clr/config.toml` ever created → exit 0, prints
/// `provider.select: anthropic (reset to default)`, and a second identical
/// invocation behaves identically. Reset on an absent config is a no-op write
/// (`toml_io::remove_user_tier`'s NotFound-as-empty semantics) — neither call
/// may create the file.
#[ test ]
fn t17_provider_select_reset_without_config_idempotent()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  assert!( read_clr_config( dir.path() ).is_none(),
    "T17: precondition — config.toml must not exist before the first reset" );

  let out = run_cs_with_env( &[ ".provider.select", "reset::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "provider.select: anthropic (reset to default)\n",
    "T17: first no-config reset must print the reset confirmation, got:\n{}", stdout( &out ) );

  let out2 = run_cs_with_env( &[ ".provider.select", "reset::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );
  assert_eq!( stdout( &out2 ), "provider.select: anthropic (reset to default)\n",
    "T17: second no-config reset must behave identically (idempotent), got:\n{}", stdout( &out2 ) );

  assert!( read_clr_config( dir.path() ).is_none(),
    "T17: reset on an absent config must not create the file (no-op write semantics)" );
}

/// T18 (533-T2, Feature 072 AC-16 / FT-16): get-mode `.provider.select` is a
/// pure read of `~/.clr/config.toml`'s `provider` key — with the ACTIVE account
/// carrying `"inference_provider": "kimi"` and no `provider` key configured,
/// the command still prints the `anthropic` default, never the active
/// account's field value.
#[ test ]
fn t18_provider_select_get_ignores_active_account_provider()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "kimi@test.com", "max", "default", FAR_FUTURE_MS, true );
  write_account_inference_provider( dir.path(), "kimi@test.com", "kimi" );

  // Precondition (anti-faking, AF-02): the kimi-tagged account is genuinely
  // current — the machine-specific active marker exists and names it...
  let marker = credential_store_dir( dir.path() )
    .join( claude_profile::account::active_marker_filename() );
  let active = std::fs::read_to_string( &marker ).expect( "T18: active marker must exist" );
  assert_eq!( active, "kimi@test.com", "T18: active marker must name the kimi account, got: {active}" );
  // ...and its stored metadata carries the kimi provider at get time.
  let meta = read_account_meta( dir.path(), "kimi@test.com" );
  assert_eq!( meta[ "inference_provider" ], serde_json::json!( "kimi" ),
    "T18: precondition — active account must carry inference_provider=kimi, got:\n{meta}" );
  assert!( read_clr_config( dir.path() ).is_none(),
    "T18: precondition — no provider key may be configured" );

  let out = run_cs_with_env( &[ ".provider.select" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  assert_eq!( stdout( &out ), "provider.select: anthropic\n",
    "T18: get mode must read the config default, never derive from the active account, got:\n{}", stdout( &out ) );
}

/// T19 (533-T3, param-group 06 CC-5): one `.account.save` combining `host::`,
/// `tags::`, and `inference_provider::` writes three independent `{name}.json`
/// fields with no interaction between them; a subsequent default `.accounts`
/// (no `cols::`) shows the `Provider` column with `kimi` — unlike `host`/`tags`,
/// `inference_provider` is in the default identity set.
#[ test ]
fn t19_save_host_tags_inference_provider_combined_default_column()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::test@example.com", "host::workbox", "tags::dev", "inference_provider::kimi" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "test@example.com" );
  assert_eq!( meta[ "host" ], serde_json::json!( "workbox" ),
    "T19: host must be 'workbox', got:\n{meta}" );
  assert_eq!( meta[ "tags" ], serde_json::json!( [ "dev" ] ),
    "T19: tags must be [\"dev\"], got:\n{meta}" );
  assert_eq!( meta[ "inference_provider" ], serde_json::json!( "kimi" ),
    "T19: inference_provider must be 'kimi', got:\n{meta}" );

  let out2 = run_cs_with_env( &[ ".accounts", "name::test@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );
  let text = stdout( &out2 );
  assert!( text.contains( "Provider: kimi" ),
    "T19: default .accounts (no cols::) must show Provider 'kimi', got:\n{text}" );
}

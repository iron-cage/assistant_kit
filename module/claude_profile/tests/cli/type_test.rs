//! Integration tests: CLI type boundary contracts.
//!
//! Verifies the four CLI-layer types — `AccountName`, `OutputFormat`,
//! `WarningThreshold`, and `AccountSelector` — through `clp` subprocess calls.
//! Types that lack a public Rust constructor API are exercised via the CLI
//! commands that accept them as parameters.
//!
//! ## Test Matrix
//!
//! ### `AccountName` type (TC-1..6)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | TC-1 | `account_name_tc1_valid_canonical_email_accepted` | valid email → exit 0 | P |
//! | TC-2 | `account_name_tc2_minimal_valid_email_accepted` | `a@b.c` → exit 0 | P |
//! | TC-3 | `account_name_tc3_empty_string_rejected` | empty name → exit 1 | N |
//! | TC-4 | `account_name_tc4_no_at_sign_rejected` | no `@` → exit 1 | N |
//! | TC-5 | `account_name_tc5_slash_in_local_part_rejected` | `/` in name → exit 1 | N |
//! | TC-6 | `account_name_tc6_empty_local_part_rejected` | `@acme.com` → exit 1 | N |
//!
//! ### `OutputFormat` type (TC-1..5)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | TC-1 | `output_format_tc1_text_accepted` | `format::text` → exit 0 | P |
//! | TC-2 | `output_format_tc2_json_accepted` | `format::json` → exit 0 | P |
//! | TC-3 | `output_format_tc3_table_accepted` | `format::table` → exit 0 (.accounts) | P |
//! | TC-4 | `output_format_tc4_case_insensitive_json_accepted` | `format::JSON` → exit 0 | P |
//! | TC-5 | `output_format_tc5_unknown_value_rejected` | `format::csv` → exit 1 | N |
//!
//! ### `WarningThreshold` type (TC-1..4)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | TC-1 | `warning_threshold_tc1_zero_accepted` | `threshold::0` → exit 0 | P |
//! | TC-2 | `warning_threshold_tc2_default_value_accepted` | `threshold::3600` → exit 0 | P |
//! | TC-3 | `warning_threshold_tc3_large_value_accepted` | `threshold::86400` → exit 0 | P |
//! | TC-4 | `warning_threshold_tc4_non_numeric_rejected` | `threshold::abc` → exit 1 | N |
//!
//! ### `AccountSelector` type (TC-1..4)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | TC-1 | `account_selector_tc1_full_email_resolves` | full email → exit 0 (switches) | P |
//! | TC-2 | `account_selector_tc2_prefix_resolves_unambiguous` | prefix `alice` → resolves to `alice@acme.com` | P |
//! | TC-3 | `account_selector_tc3_ambiguous_prefix_rejected` | ambiguous prefix → exit 1 | N |
//! | TC-4 | `account_selector_tc4_nonmatching_prefix_rejected` | no-match prefix → exit 2 | N |

use tempfile::TempDir;
use super::cli_runner::
{
  run_cs_with_env, assert_exit, write_credentials, write_account,
  FAR_FUTURE_MS,
};

// ── AccountName ───────────────────────────────────────────────────────────────

// TC-1: Valid canonical email accepted
#[ test ]
fn account_name_tc1_valid_canonical_email_accepted()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// TC-2: Minimal valid email accepted
#[ test ]
fn account_name_tc2_minimal_valid_email_accepted()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".account.save", "name::a@b.c" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// TC-3: Empty string rejected
#[ test ]
fn account_name_tc3_empty_string_rejected()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".account.save", "name::" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

// TC-4: Input with no `@` character rejected
#[ test ]
fn account_name_tc4_no_at_sign_rejected()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".account.save", "name::aliceatacme" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

// TC-5: Local part containing `/` rejected
#[ test ]
fn account_name_tc5_slash_in_local_part_rejected()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".account.save", "name::alice/bob@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

// TC-6: Empty local part (starts with `@`) rejected
#[ test ]
fn account_name_tc6_empty_local_part_rejected()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".account.save", "name::@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

// ── OutputFormat ──────────────────────────────────────────────────────────────

// TC-1: `"text"` parsed to TEXT variant
#[ test ]
fn output_format_tc1_text_accepted()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out = run_cs_with_env(
    &[ ".accounts", "format::text" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// TC-2: `"json"` parsed to JSON variant
#[ test ]
fn output_format_tc2_json_accepted()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out = run_cs_with_env(
    &[ ".accounts", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// TC-3: `"table"` parsed to TABLE variant (`.accounts` only command that accepts table)
#[ test ]
fn output_format_tc3_table_accepted()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out = run_cs_with_env(
    &[ ".accounts", "format::table" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// TC-4: Case-insensitive input `"JSON"` parsed correctly
#[ test ]
fn output_format_tc4_case_insensitive_json_accepted()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out = run_cs_with_env(
    &[ ".accounts", "format::JSON" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// TC-5: Unknown value `"csv"` rejected
#[ test ]
fn output_format_tc5_unknown_value_rejected()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let out = run_cs_with_env(
    &[ ".accounts", "format::csv" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

// ── WarningThreshold ──────────────────────────────────────────────────────────

// TC-1: `"0"` parsed to threshold 0 (disabled)
#[ test ]
fn warning_threshold_tc1_zero_accepted()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".token.status", "threshold::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// TC-2: `"3600"` parsed to default threshold
#[ test ]
fn warning_threshold_tc2_default_value_accepted()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".token.status", "threshold::3600" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// TC-3: Large value `"86400"` accepted
#[ test ]
fn warning_threshold_tc3_large_value_accepted()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".token.status", "threshold::86400" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// TC-4: Non-numeric string `"abc"` rejected — unilang rejects Kind::Integer param
#[ test ]
fn warning_threshold_tc4_non_numeric_rejected()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  let out = run_cs_with_env(
    &[ ".token.status", "threshold::abc" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
}

// ── AccountSelector ───────────────────────────────────────────────────────────

// TC-1: Full email form resolves directly to AccountName
#[ test ]
fn account_selector_tc1_full_email_resolves()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );
  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// TC-2: Exact local-part prefix resolves to the one matching account
#[ test ]
fn account_selector_tc2_prefix_resolves_unambiguous()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );
  let out = run_cs_with_env(
    &[ ".account.use", "name::alice" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// TC-3: Ambiguous prefix matching multiple accounts is rejected
#[ test ]
fn account_selector_tc3_ambiguous_prefix_rejected()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "alice@home.com", "max", "default", FAR_FUTURE_MS, false );
  let out = run_cs_with_env(
    &[ ".account.use", "name::al" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = super::cli_runner::stderr( &out );
  assert!( err.to_ascii_lowercase().contains( "ambiguous" ), "expected 'ambiguous' in stderr: {err}" );
}

// TC-4: Non-matching prefix is rejected with "not found" error
#[ test ]
fn account_selector_tc4_nonmatching_prefix_rejected()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );
  let out = run_cs_with_env(
    &[ ".account.use", "name::xyz" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 2 );
}

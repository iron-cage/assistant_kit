//! Integration tests: H (Help), ACC (`.accounts` command).
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! ### H — Help
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | h01 | `h01_dot_shows_help` | `.` → shows .accounts | P |
//! | h02 | `h02_help_lists_all_registered_commands` | `.help` → .accounts listed; .account.list/.account.status absent | P |
//! | h03 | `h03_help_hides_dot` | `.help` → bare `.` not listed | P |
//! | h04 | `h04_help_exits_0` | `.help` → exit 0 | P |
//! | h05 | `h05_no_args_shows_help` | no args → help shows .accounts | P |
//! | h06 | `h06_double_dash_help` | `--help` → exit 1 (POSIX flags not supported) | N |
//! | h07 | `h07_unknown_command_exits_1` | `.nonexistent` → exit 1 + stderr | N |
//!
//! ### ACC — `.accounts` command
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | acc01 | `acc01_lists_accounts_as_indented_blocks` | all accounts → indented key-val blocks | P |
//! | acc02 | `acc02_active_shows_yes_inactive_shows_no` | active=yes / inactive=no per account | P |
//! | acc03 | `acc03_empty_store_shows_advisory` | empty store → advisory message, exit 0 | P |
//! | acc04 | `acc04_name_scopes_to_single_block` | `name::EMAIL` → only that account's block | P |
//! | acc05 | `acc05_name_not_found_exits_2` | valid but unknown name → exit 2 | N |
//! | acc06 | `acc06_name_invalid_exits_1` | `name::notanemail` → exit 1 | N |
//! | acc07 | `acc07_field_presence_suppresses_lines` | `sub::0 tier::0` → Sub/Tier absent | P |
//! | acc08 | `acc08_all_fields_off_bare_names` | all fields off → bare name list | P |
//! | acc09 | `acc09_json_format_array` | `format::json` → valid JSON array | P |
//! | acc10 | `acc10_json_ignores_field_presence` | `format::json` always includes all fields | P |
//! | acc11 | `acc11_missing_store_shows_advisory` | absent credential dir → advisory, exit 0 | P |
//! | acc12 | `acc12_sorted_alphabetically` | accounts listed in alpha order | P |
//! | acc13 | `acc13_blank_line_between_blocks` | multiple accounts → blank line between each block | P |
//! | acc14 | `acc14_nonactive_shows_own_stored_expires` | non-active uses own stored expires, not active's | P |
//! | acc15 | `acc15_missing_sub_field_shows_na` | missing subscriptionType in file → Sub: N/A | P |
//! | acc16 | `acc16_missing_tier_field_shows_na` | missing rateLimitTier in file → Tier: N/A | P |
//! | acc17 | `acc17_json_format_empty_store` | `format::json` + absent store → `[]` | P |
//! | acc18 | `acc18_single_account_no_trailing_blank` | single account text → no trailing blank line | P |
//! | acc19 | `acc19_missing_expires_at_shows_expired` | missing expiresAt in file → Expires: expired | P |

use crate::helpers::{
  run_cs, run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account,
  FAR_FUTURE_MS, PAST_MS,
};
use tempfile::TempDir;

// ── H: Help commands ──────────────────────────────────────────────────────────

#[ test ]
fn h01_dot_shows_help()
{
  let out  = run_cs( &[ "." ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".accounts" ), "help must list .accounts, got:\n{text}" );
}

#[ test ]
fn h02_help_lists_all_registered_commands()
{
  let out  = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  for cmd in &[
    ".accounts",
    ".account.save",
    ".account.switch",
    ".account.delete",
    ".token.status",
    ".paths",
    ".usage",
    ".credentials.status",
    ".account.limits",
  ]
  {
    assert!( text.contains( cmd ), "help must list {cmd}, got:\n{text}" );
  }
  assert!( !text.contains( ".account.list" ),   "help must not list .account.list, got:\n{text}" );
  assert!( !text.contains( ".account.status" ), "help must not list .account.status, got:\n{text}" );
}

#[ test ]
fn h03_help_hides_dot()
{
  let out   = run_cs( &[ ".help" ] );
  let text  = stdout( &out );
  // `.` is registered with `hidden_from_list: true` — must not appear as a listed command.
  // `.help` IS visible (auto-registered by unilang) — that's expected.
  let lines : Vec< &str > = text.lines()
    .filter( | l | l.trim().starts_with( '.' ) )
    .collect();
  for line in &lines
  {
    let cmd = line.split_whitespace().next().unwrap_or( "" );
    assert!( cmd != ".", "listing should not include bare '.', got line: {line}" );
  }
}

#[ test ]
fn h04_help_exits_0()
{
  let out = run_cs( &[ ".help" ] );
  assert_exit( &out, 0 );
}

#[ test ]
fn h05_no_args_shows_help()
{
  let out  = run_cs( &[] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( ".accounts" ), "no-args help must list .accounts, got:\n{text}" );
}

#[ test ]
fn h06_double_dash_help()
{
  // POSIX flags (--help, -h) are not supported — use `.help` command instead.
  let out = run_cs( &[ "--help" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "unexpected flag" ), "--help must produce unexpected flag error, got:\n{err}" );
}

#[ test ]
fn h07_unknown_command_exits_1()
{
  let out = run_cs( &[ ".nonexistent" ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( !err.is_empty(), "unknown command must produce stderr" );
}

// ── ACC: .accounts command ────────────────────────────────────────────────────

#[ test ]
fn acc01_lists_accounts_as_indented_blocks()
{
  // IT-1: all accounts listed as indented key-val blocks.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "work@acme.com" ),     "must list work@acme.com, got:\n{text}" );
  assert!( text.contains( "personal@home.com" ), "must list personal@home.com, got:\n{text}" );
  assert!( text.contains( "Active:" ),  "must show Active: field, got:\n{text}" );
  assert!( text.contains( "Sub:" ),     "must show Sub: field, got:\n{text}" );
  assert!( text.contains( "Expires:" ), "must show Expires: field, got:\n{text}" );
  // Exactly 2 unindented name-header lines
  let name_lines : Vec< &str > = text.lines()
    .filter( | l | !l.starts_with( ' ' ) && !l.is_empty() )
    .collect();
  assert_eq!( name_lines.len(), 2, "must have exactly 2 account name lines, got:\n{text}" );
}

#[ test ]
fn acc02_active_shows_yes_inactive_shows_no()
{
  // IT-2: active account shows Active:  yes; inactive shows Active:  no.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Active:  yes" ), "active account must show Active:  yes, got:\n{text}" );
  assert!( text.contains( "Active:  no" ),  "inactive account must show Active:  no, got:\n{text}" );
}

#[ test ]
fn acc03_empty_store_shows_advisory()
{
  // IT-3: empty credential store → advisory message, exit 0.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all(
    dir.path().join( ".persistent" ).join( "claude" ).join( "credential" )
  ).unwrap();

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "no accounts configured" ), "empty store must say no accounts, got:\n{text}" );
}

#[ test ]
fn acc04_name_scopes_to_single_block()
{
  // IT-4: name::EMAIL shows only that account's block.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".accounts", "name::work@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(  text.contains( "work@acme.com" ),     "must show named account, got:\n{text}" );
  assert!( !text.contains( "personal@home.com" ), "must not show other account, got:\n{text}" );
  assert!(  text.contains( "Active:  yes" ), "named active account must show Active:  yes, got:\n{text}" );
}

#[ test ]
fn acc05_name_not_found_exits_2()
{
  // IT-5: valid but non-existent name → exit 2.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "max", "tier4", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".accounts", "name::ghost@example.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 2 );
  let err = stderr( &out );
  assert!(
    err.contains( "not found" ) || err.contains( "ghost@example.com" ),
    "must report account not found, got:\n{err}",
  );
}

#[ test ]
fn acc06_name_invalid_exits_1()
{
  // IT-6: invalid email format → exit 1.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".accounts", "name::notanemail" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

#[ test ]
fn acc07_field_presence_suppresses_lines()
{
  // IT-7: sub::0 tier::0 → Sub/Tier absent; Active/Expires/Org remain.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "max", "tier4", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".accounts", "sub::0", "tier::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(  text.contains( "Active:" ),  "Active: must remain when sub::0 tier::0, got:\n{text}" );
  assert!(  text.contains( "Expires:" ), "Expires: must remain when sub::0 tier::0, got:\n{text}" );
  assert!( !text.contains( "Sub:" ),     "Sub: must be suppressed, got:\n{text}" );
  assert!( !text.contains( "Tier:" ),    "Tier: must be suppressed, got:\n{text}" );
}

#[ test ]
fn acc08_all_fields_off_bare_names()
{
  // IT-8: all field params off → bare name per line, no indented fields.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".accounts", "active::0", "sub::0", "tier::0", "expires::0", "org::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).collect();
  assert_eq!( lines.len(), 2, "all fields off must produce exactly 2 lines (names), got:\n{text}" );
  assert!( !text.contains( "Active:" ),  "Active: must be absent, got:\n{text}" );
  assert!( !text.contains( "Sub:" ),     "Sub: must be absent, got:\n{text}" );
  assert!( !text.contains( "Tier:" ),    "Tier: must be absent, got:\n{text}" );
  assert!( !text.contains( "Expires:" ), "Expires: must be absent, got:\n{text}" );
  assert!( !text.contains( "Org:" ),     "Org: must be absent, got:\n{text}" );
}

#[ test ]
fn acc09_json_format_array()
{
  // IT-9: format::json → valid JSON array with expected keys.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com",     "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "personal@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.trim_start().starts_with( '[' ), "JSON must start with '[', got:\n{text}" );
  assert!( text.contains( "\"is_active\":true" ),  "active account must have is_active:true, got:\n{text}" );
  assert!( text.contains( "\"is_active\":false" ), "inactive account must have is_active:false, got:\n{text}" );
  assert!( text.contains( "\"subscription_type\"" ), "JSON must include subscription_type, got:\n{text}" );
  assert!( text.contains( "\"rate_limit_tier\"" ),   "JSON must include rate_limit_tier, got:\n{text}" );
  assert!( text.contains( "\"expires_at_ms\"" ),     "JSON must include expires_at_ms, got:\n{text}" );
}

#[ test ]
fn acc10_json_ignores_field_presence()
{
  // IT-10: format::json always includes all fields, even when field-presence params are off.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "max", "tier4", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env(
    &[ ".accounts", "sub::0", "tier::0", "active::0", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"subscription_type\"" ), "JSON must include subscription_type despite sub::0, got:\n{text}" );
  assert!( text.contains( "\"rate_limit_tier\"" ),   "JSON must include rate_limit_tier despite tier::0, got:\n{text}" );
  assert!( text.contains( "\"is_active\"" ),          "JSON must include is_active despite active::0, got:\n{text}" );
}

#[ test ]
fn acc11_missing_store_shows_advisory()
{
  // IT-11: absent credential store directory → advisory, exit 0.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();
  // Deliberately do NOT create .persistent/claude/credential/

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "no accounts configured" ), "absent store must say no accounts, got:\n{text}" );
  assert!( stderr( &out ).is_empty(), "absent store must not produce stderr, got:\n{}", stderr( &out ) );
}

#[ test ]
fn acc12_sorted_alphabetically()
{
  // IT-12: accounts listed in alphabetical order regardless of creation order.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "zed@acme.com",   "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "alice@acme.com", "pro", "standard", FAR_FUTURE_MS, false );
  write_account( dir.path(), "mike@acme.com",  "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env(
    &[ ".accounts", "active::0", "sub::0", "tier::0", "expires::0", "org::0" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text  = stdout( &out );
  let lines : Vec< &str > = text.lines().filter( | l | !l.is_empty() ).map( str::trim ).collect();
  assert_eq!(
    lines,
    vec![ "alice@acme.com", "mike@acme.com", "zed@acme.com" ],
    "accounts must be sorted alphabetically, got:\n{text}",
  );
}

#[ test ]
fn acc13_blank_line_between_blocks()
{
  // IT-13: when any field is shown, a blank line separates each account block.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alice@home.com", "pro", "standard", FAR_FUTURE_MS, false );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "\n\n" ),
    "multiple accounts with fields must have blank line between blocks, got:\n{text}",
  );
}

// ── acc14: P2 guard — non-active account uses its own stored expires ──────────

// test_kind: bug_reproducer(issue-p2-named-account-token)
//
// Root Cause: The old `.account.status` active-account path called `status_with_threshold()`
//   which reads `~/.claude/.credentials.json` — the ACTIVE account's live credentials file.
//   For non-active accounts, a similar leak was possible. `.accounts` must always use stored
//   `expiresAt` via `token_status_from_ms()` for every account to avoid leaking the active
//   account's token state into a non-active account's Expires: line.
// Why Not Caught: Prior tests used FAR_FUTURE_MS for all accounts — no test exercised
//   a non-active account with PAST_MS while an active account had a valid token.
// Fix Applied: `accounts_routine` uses `token_status_from_ms(a.expires_at_ms)` for every
//   account in the list — the live credentials file is never read.
// Prevention: Never call `status_with_threshold()` inside `accounts_routine`; all
//   per-account data must come from the stored credential struct.
// Pitfall: Future fields that seem to require live credential reads (e.g. token validation)
//   must be refused for non-active accounts — use stored data only for consistency.

#[ test ]
fn acc14_nonactive_shows_own_stored_expires()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Active account has a far-future (valid) token
  write_account( dir.path(), "alice@acme.com", "max", "tier4",    FAR_FUTURE_MS, true  );
  // Non-active account has an already-expired token
  write_account( dir.path(), "alice@home.com", "pro", "standard", PAST_MS,       false );

  let out  = run_cs_with_env( &[ ".accounts", "name::alice@home.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "expired" ),
    "non-active account must show its OWN stored expired state, not active account's valid state, got:\n{text}",
  );
  assert!(
    !text.contains( "in " ),
    "must not leak active account's valid expiry duration into non-active query, got:\n{text}",
  );
}

// ── acc15: missing subscriptionType → Sub: N/A (not blank) ───────────────────

// test_kind: bug_reproducer(issue-empty-field-blank)
//
// Root Cause: `account::list()` uses `unwrap_or_default()` for missing JSON fields,
//   yielding `""` when `subscriptionType` is absent from the credential file. Without
//   the `.is_empty()` guard, the empty string produces a blank "Sub:     " line rather
//   than "Sub:     N/A".
// Why Not Caught: All prior tests used `write_account()` which always writes non-empty
//   sub/tier values. No test used a raw credential file with a missing field.
// Fix Applied: `accounts_routine` guards with `if a.subscription_type.is_empty() { "N/A" }`
//   before formatting the Sub: line — same pattern as `credentials_status_routine`.
// Prevention: Every field read from `account::list()` for display must guard with
//   `.is_empty()` because `account::list()` returns "" for absent JSON fields.
// Pitfall: `account::list()` returns "" (not None) for missing fields; Option-based
//   patterns like `.unwrap_or("N/A")` will NOT catch it — check `.is_empty()`.

#[ test ]
fn acc15_missing_sub_field_shows_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  // Credential file with NO subscriptionType field — account::list() yields "" for it
  std::fs::write(
    credential_store.join( "alice@home.com.credentials.json" ),
    r#"{"oauthAccount":{"rateLimitTier":"standard"},"expiresAt":9999999999000}"#,
  ).unwrap();

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Sub:     N/A" ),
    "missing subscriptionType must display 'Sub:     N/A', got:\n{text}",
  );
}

// ── acc16: missing rateLimitTier → Tier: N/A (not blank) ─────────────────────

#[ test ]
fn acc16_missing_tier_field_shows_na()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  // Credential file with NO rateLimitTier field — account::list() yields "" for it
  std::fs::write(
    credential_store.join( "alice@home.com.credentials.json" ),
    r#"{"oauthAccount":{"subscriptionType":"pro"},"expiresAt":9999999999000}"#,
  ).unwrap();

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Tier:    N/A" ),
    "missing rateLimitTier must display 'Tier:    N/A', got:\n{text}",
  );
}

// ── acc17: format::json + absent store → [] ───────────────────────────────────

/// acc17: `format::json` with no credential store directory → returns `[]`.
///
/// Root Cause: The `Json` branch has an explicit `if accounts.is_empty()` guard
///   that returns `"[]\n"` — this code path was not directly tested.
/// Why Not Caught: acc09 (json format) requires accounts to be present; acc11
///   (absent store) uses text format only. The intersection was untested.
/// Fix Applied: No fix needed — the guard was already correct. Test confirms it.
/// Prevention: For every format × store-state combination (json+empty, text+empty)
///   add an explicit test — do not assume format handling is symmetric.
/// Pitfall: An empty JSON array `[]` and the text advisory `(no accounts configured)`
///   are NOT interchangeable — callers of `format::json` must parse `[]`, not text.
#[ test ]
fn acc17_json_format_empty_store()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Deliberately do NOT create .persistent/claude/credential/ — account::list returns []
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.trim_start().starts_with( '[' ),
    "json format must start with '[', got:\n{text}",
  );
  assert!(
    text.contains( "[]" ),
    "json format with absent store must return empty array '[]', got:\n{text}",
  );
  assert!(
    !text.contains( "no accounts" ),
    "json format must not return text advisory, got:\n{text}",
  );
}

// ── acc18: single account text → no trailing blank line ───────────────────────

/// acc18: A single account in text mode produces no trailing blank line.
///
/// Root Cause: `render_accounts_text` adds a blank separator only between blocks
///   (`if idx < last_idx`). For a single account (idx=0, last_idx=0) the condition
///   is false — no blank line is appended after the final block.
/// Why Not Caught: acc13 confirms a blank line EXISTS between two accounts, but
///   never asserts the last block has no trailing blank. acc04 confirms single-block
///   content but does not check for absence of trailing blank.
/// Fix Applied: No fix needed — the guard was already correct. Test confirms it.
/// Prevention: When testing separator logic, test both the presence case (multiple
///   blocks) and the absence case (single block) explicitly.
/// Pitfall: A trailing blank line in text output breaks scripts that read the last
///   line of output — always verify last-block is not followed by a blank.
#[ test ]
fn acc18_single_account_no_trailing_blank()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "solo@example.com", "max", "tier4", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "solo@example.com" ),
    "must list the sole account, got:\n{text}",
  );
  assert!(
    !text.ends_with( "\n\n" ),
    "single account must not have a trailing blank line, got:\n{text:?}",
  );
}

// ── acc19: missing expiresAt → Expires: expired ───────────────────────────────

/// acc19: A credential file missing `expiresAt` is displayed as expired.
///
/// Root Cause: `account::list()` calls `parse_u64_field(&content, "expiresAt")`
///   which returns `None` for a missing field, then `unwrap_or(0)` yields 0 ms
///   (Unix epoch). Any current time is >> 0 ms → `token_status_from_ms(0)` →
///   `TokenStatus::Expired` → `Expires: expired`.
/// Why Not Caught: All prior tests use `write_account()` which always writes a
///   non-zero `expiresAt`. No test used a raw credential file with the field absent.
/// Fix Applied: No fix needed — `unwrap_or(0)` correctly maps missing → expired.
///   Test documents the contract and prevents future regressions.
/// Prevention: When adding or changing `parse_u64_field` call sites, verify
///   the fallback for a missing field produces the expected sentinel behaviour.
/// Pitfall: A missing `expiresAt` silently renders as expired — do not mistake
///   this for a credential-read error; the account IS listed, just marked expired.
#[ test ]
fn acc19_missing_expires_at_shows_expired()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let credential_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &credential_store ).unwrap();
  // No `expiresAt` field → parse_u64_field returns None → unwrap_or(0) → epoch → expired
  std::fs::write(
    credential_store.join( "ghost@example.com.credentials.json" ),
    r#"{"oauthAccount":{"subscriptionType":"pro","rateLimitTier":"standard"}}"#,
  ).unwrap();

  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "ghost@example.com" ),
    "must list the account, got:\n{text}",
  );
  assert!(
    text.contains( "Expires: expired" ),
    "missing expiresAt must display 'Expires: expired', got:\n{text}",
  );
}

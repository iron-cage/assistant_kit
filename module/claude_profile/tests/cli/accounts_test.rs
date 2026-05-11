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
//! | acc20 | `acc20_display_name_shows_from_snapshot` | `display_name::1` → Display: from saved snapshot | P |
//! | acc21 | `acc21_role_billing_model_from_snapshots` | `role::1 billing::1 model::1` → 3 lines from snapshots | P |
//! | acc22 | `acc22_no_snapshot_shows_na_for_new_fields` | no snapshot → N/A for new fields when enabled | P |
//! | acc23 | `acc23_json_includes_new_fields` | `format::json` → includes display_name, role, billing, model | P |
//! | acc24 | `acc24_new_fields_absent_by_default` | no opt-in → Display/Role/Billing/Model absent | P |
//! | acc25 | `acc25_email_reads_from_snapshot` | Email: default-on → real email from snapshot | P |
//! | acc26 | `acc26_save_creates_snapshot_files` | `save` creates `{name}.claude.json` and `.settings.json` | P |
//! | acc27 | `acc27_save_succeeds_without_claude_json` | save OK when `~/.claude.json` absent (best-effort) | P |
//! | acc28 | `acc28_save_succeeds_without_settings_json` | save OK when `settings.json` absent but `.claude.json` present | P |

use crate::helpers::{
  run_cs, run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_credentials, write_claude_json_full, write_settings_json,
  write_account_claude_json, write_account_settings_json,
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
    ".account.use",
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
  // IT-7: sub::0 tier::0 → Sub/Tier absent; Active/Expires/Email remain.
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "work@acme.com", "max", "tier4", FAR_FUTURE_MS, true );

  let out  = run_cs_with_env( &[ ".accounts", "sub::0", "tier::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(  text.contains( "Active:" ),  "Active: must remain when sub::0 tier::0, got:\n{text}" );
  assert!(  text.contains( "Expires:" ), "Expires: must remain when sub::0 tier::0, got:\n{text}" );
  assert!(  text.contains( "Email:" ),   "Email: must remain when sub::0 tier::0, got:\n{text}"  );
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
    &[ ".accounts", "active::0", "sub::0", "tier::0", "expires::0", "email::0" ],
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
  assert!( !text.contains( "Email:" ),   "Email: must be absent, got:\n{text}" );
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
    &[ ".accounts", "active::0", "sub::0", "tier::0", "expires::0", "email::0" ],
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
///   (`if idx < last_idx`). For a single account (`idx=0, last_idx=0`) the condition
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

// ── acc20–acc28: Rich account metadata (FR-20, feature/014) ──────────────────

/// acc20 (T01): `display_name::1` renders `Display:` line from saved snapshot.
///
/// Root Cause (before fix): `Account` struct lacked `display_name` field; `list()` never
///   read snapshot files; `render_accounts_text()` did not accept `show_display_name` param.
/// Why Not Caught: All prior tests used only the 5 original Account fields.
/// Fix Applied: `Account` gains `display_name`; `list()` reads `{name}.claude.json`;
///   `render_accounts_text()` renders `Display:` when `show_display_name` is true.
/// Prevention: Whenever adding opt-in fields, write a snapshot-present test (acc20)
///   and a snapshot-absent test (acc22) to cover both code paths.
/// Pitfall: `parse_string_field()` searches the flat string — it finds nested keys like
///   `"displayName"` regardless of JSON depth. Do NOT add custom JSON parsing.
#[ test ]
fn acc20_display_name_shows_from_snapshot()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json( dir.path(), "alice@acme.com", "alice@acme.com", "Alice K", "admin", "stripe" );

  let out  = run_cs_with_env( &[ ".accounts", "display_name::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Display: Alice K" ),
    "display_name::1 must render Display: line from snapshot, got:\n{text}",
  );
}

/// acc21 (T02+T03+T04): `role::1 billing::1 model::1` renders three snapshot lines.
///
/// Root Cause (before fix): `Account` struct lacked `role`, `billing`, `model` fields;
///   `list()` did not read snapshot files; rendering did not handle these params.
/// Why Not Caught: Only original 5 fields were tested.
/// Fix Applied: `Account` gains `role`, `billing`, `model`; `list()` reads both snapshot
///   files; `render_accounts_text()` renders the three new lines when enabled.
/// Prevention: Test all three in one function to catch the common mistake of reading
///   one snapshot file but forgetting the other.
/// Pitfall: `model` comes from `{name}.settings.json`, not `{name}.claude.json`. A single
///   snapshot read call is insufficient — both files must be read independently.
#[ test ]
fn acc21_role_billing_model_from_snapshots()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json( dir.path(), "alice@acme.com", "alice@acme.com", "Alice K", "admin", "stripe_sub" );
  write_account_settings_json( dir.path(), "alice@acme.com", "claude-sonnet" );

  let out  = run_cs_with_env(
    &[ ".accounts", "role::1", "billing::1", "model::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Role:    admin"         ), "role::1 must render Role: from snapshot, got:\n{text}"    );
  assert!( text.contains( "Billing: stripe_sub"    ), "billing::1 must render Billing: from snapshot, got:\n{text}" );
  assert!( text.contains( "Model:   claude-sonnet" ), "model::1 must render Model: from snapshot, got:\n{text}"  );
}

/// acc22 (T05): when no snapshot files exist, opt-in fields show `N/A`.
///
/// Root Cause (before fix): Without snapshot files, `list()` would yield empty strings
///   for all new fields, but `render_accounts_text()` lacked the empty-string → N/A guard.
/// Why Not Caught: Implementation gap: snapshot reading not yet coded.
/// Fix Applied: `list()` uses `unwrap_or_default()` → empty string; `render_accounts_text()`
///   guards each new field with `if field.is_empty() { "N/A" }`.
/// Prevention: Always pair a snapshot-absent test with each snapshot-present test so both
///   the reading path and the fallback path are verified.
/// Pitfall: `unwrap_or_default()` on a missing file yields `""` — callers must guard
///   against empty string, not `None`. Pattern: `if s.is_empty() { "N/A" } else { s }`.
#[ test ]
fn acc22_no_snapshot_shows_na_for_new_fields()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  // No snapshot files written — all new fields must fall back to N/A.

  let out  = run_cs_with_env(
    &[ ".accounts", "display_name::1", "role::1", "billing::1", "model::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "Display: N/A" ), "absent snapshot must show Display: N/A, got:\n{text}" );
  assert!( text.contains( "Role:    N/A" ), "absent snapshot must show Role:    N/A, got:\n{text}" );
  assert!( text.contains( "Billing: N/A" ), "absent snapshot must show Billing: N/A, got:\n{text}" );
  assert!( text.contains( "Model:   N/A" ), "absent snapshot must show Model:   N/A, got:\n{text}" );
}

/// acc23 (T06): `format::json` always includes the four new field keys.
///
/// Root Cause (before fix): JSON format string in `accounts_routine()` hardcoded only
///   legacy fields; no `display_name`, `role`, `billing`, `model` keys were emitted.
/// Why Not Caught: acc10 (`json_ignores_field_presence`) only checked original fields.
/// Fix Applied: Extend the JSON format string with all four new fields using
///   `json_escape(&a.display_name)` etc. — matches the `.credentials.status` JSON pattern.
/// Prevention: When adding struct fields, always extend BOTH text rendering AND JSON output
///   in the same phase to avoid silent key omissions.
/// Pitfall: `format::json` emits all fields unconditionally — do NOT gate new JSON keys
///   on the field-presence booleans (`show_display_name` etc.); those control text only.
#[ test ]
fn acc23_json_includes_new_fields()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json( dir.path(), "alice@acme.com", "alice@acme.com", "Alice K", "admin", "stripe_sub" );
  write_account_settings_json( dir.path(), "alice@acme.com", "claude-sonnet" );

  let out  = run_cs_with_env( &[ ".accounts", "format::json" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "\"email\""         ), "JSON must include email key, got:\n{text}"        );
  assert!( text.contains( "\"display_name\"" ), "JSON must include display_name key, got:\n{text}" );
  assert!( text.contains( "\"role\""         ), "JSON must include role key, got:\n{text}"         );
  assert!( text.contains( "\"billing\""      ), "JSON must include billing key, got:\n{text}"      );
  assert!( text.contains( "\"model\""        ), "JSON must include model key, got:\n{text}"        );
  assert!( text.contains( "alice@acme.com"   ), "JSON email must contain actual value, got:\n{text}"        );
  assert!( text.contains( "Alice K"          ), "JSON display_name must contain actual value, got:\n{text}" );
  assert!( text.contains( "claude-sonnet"    ), "JSON model must contain actual value, got:\n{text}"        );
}

/// acc24 (T07): new opt-in fields absent from output by default.
///
/// Root Cause (invariant guard): Opt-in fields use `Some(Value::Boolean(true))` without
///   `| None` fallback, so absence of the param = field hidden. No `None` in the match
///   is the ONLY difference from default-on params.
/// Why Not Caught: No test verified that the new params are truly off by default.
/// Fix Applied: Invariant confirmed by test; `accounts_routine()` reads each new param
///   with `matches!(..., Some(Value::Boolean(true)))` (no `None`).
/// Prevention: For every opt-in param, pair an opt-in-enabled test (acc20) with an
///   opt-in-absent test (acc24) so regressions to default-on are caught immediately.
/// Pitfall: Adding `| None` to an opt-in param silently makes it default-on — a
///   runtime-invisible change that only a test like this one catches.
#[ test ]
fn acc24_new_fields_absent_by_default()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json( dir.path(), "alice@acme.com", "alice@acme.com", "Alice K", "admin", "stripe" );
  write_account_settings_json( dir.path(), "alice@acme.com", "claude-sonnet" );

  // Default .accounts call — no opt-in params.
  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( !text.contains( "Display:" ), "Display: must be absent by default, got:\n{text}" );
  assert!( !text.contains( "Role:"    ), "Role: must be absent by default, got:\n{text}"    );
  assert!( !text.contains( "Billing:" ), "Billing: must be absent by default, got:\n{text}" );
  assert!( !text.contains( "Model:"   ), "Model: must be absent by default, got:\n{text}"   );
}

/// acc25 (T08): `Email:` default-on renders real email from saved snapshot.
///
/// Root Cause (before fix): `list()` read `organizationName` for `Account.org`; the
///   `emailAddress` field was never read from the per-account snapshot file.
/// Why Not Caught: No test verified that Email: shows the actual stored emailAddress value.
/// Fix Applied: `list()` reads `{name}.claude.json` → `emailAddress` and populates
///   `Account.email`; `render_accounts_text()` uses `a.email` with empty-string → N/A guard.
/// Prevention: When a display value is derived from a data source, write a test that
///   verifies the ACTUAL VALUE appears — not just the label line.
/// Pitfall: An empty-string fallback silently shows "N/A" — always add a snapshot-present
///   test like this one so the read path is exercised, not just the fallback.
#[ test ]
fn acc25_email_reads_from_snapshot()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "tier4", FAR_FUTURE_MS, true );
  write_account_claude_json( dir.path(), "alice@acme.com", "alice@acme.com", "Alice K", "admin", "stripe" );

  // Email is default-on — no toggle needed.
  let out  = run_cs_with_env( &[ ".accounts" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "Email:   alice@acme.com" ),
    "Email: must show real email from snapshot, got:\n{text}",
  );
  assert!(
    !text.contains( "Email:   N/A" ),
    "Email: must not show N/A when snapshot has emailAddress, got:\n{text}",
  );
}

/// acc26 (T09 — save with both snapshot sources): `account::save` writes both snapshot files.
///
/// Root Cause (before fix): `save()` only called `std::fs::copy(paths.credentials_file(), dest)`.
///   The `.claude.json` and `settings.json` sources were never copied to the store.
/// Why Not Caught: No save test verified the presence of snapshot files after save.
/// Fix Applied: `save()` calls `let _ = std::fs::copy(paths.claude_json_file(), ...)` and
///   `let _ = std::fs::copy(paths.settings_file(), ...)` after the credential copy.
/// Prevention: After any `save()` implementation change, verify ALL expected output files
///   exist — not just the primary credential file.
/// Pitfall: `let _ = std::fs::copy(...)` silently discards errors — this is intentional
///   (best-effort), but means a wrong SOURCE path would pass the `save()` return value.
///   This test catches that by asserting the destination files EXIST.
#[ test ]
fn acc26_save_creates_snapshot_files()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Source files that save() will copy.
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_full( dir.path(), "alice@acme.com", "Alice K", "admin", "stripe" );
  write_settings_json( dir.path(), "claude-sonnet" );

  let out  = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!(
    store.join( "alice@acme.com.credentials.json" ).exists(),
    "save must create credentials snapshot, store: {}", store.display(),
  );
  assert!(
    store.join( "alice@acme.com.claude.json" ).exists(),
    "save must create .claude.json snapshot, store: {}", store.display(),
  );
  assert!(
    store.join( "alice@acme.com.settings.json" ).exists(),
    "save must create settings.json snapshot, store: {}", store.display(),
  );
}

/// acc27 (T09 — save without `~/.claude.json`): save succeeds even when source is absent.
///
/// Root Cause (before fix): If `let _ = std::fs::copy(paths.claude_json_file(), ...)` were
///   written as a hard `?` copy, a missing `~/.claude.json` would fail the entire save.
/// Why Not Caught: All prior save tests relied on a credentials file being present;
///   no test verified the best-effort behaviour for the new optional sources.
/// Fix Applied: Use `let _ = std::fs::copy(...)` (discard result) for both snapshot copies —
///   missing source silently skips; credential copy still uses `?` (required).
/// Prevention: For every best-effort file operation, add a test where the source is absent
///   to confirm the operation succeeds and downstream callers are not affected.
/// Pitfall: `let _ = expr` discards the `Result` ENTIRELY — compilation will not warn
///   about missing files. Always add a best-effort-absent test like this one.
#[ test ]
fn acc27_save_succeeds_without_claude_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Only credentials file — no ~/.claude.json, no settings.json.
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );

  let out  = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!(
    store.join( "alice@acme.com.credentials.json" ).exists(),
    "save must still create credential file when snapshots absent",
  );
  // Snapshot files must be absent (not created from non-existent sources).
  assert!(
    !store.join( "alice@acme.com.claude.json" ).exists(),
    "no .claude.json source → no .claude.json snapshot must be created",
  );
}

/// acc28 (T09 — save with claude.json but without settings.json): partial snapshot scenario.
///
/// Root Cause (before fix): Same as acc27 — the settings.json copy used a hard `?`, so a
///   missing `settings.json` would abort save even when `~/.claude.json` was present.
/// Why Not Caught: No test exercised the partial-source scenario.
/// Fix Applied: Both snapshot copies use `let _ = std::fs::copy(...)` independently so
///   one absent source does not block the other copy.
/// Prevention: Test each source file independently — one present, one absent — to confirm
///   the two best-effort copies are truly independent and not short-circuit-evaluated.
/// Pitfall: If `save()` used a single compound expression for both copies, one absent
///   source would prevent the other snapshot from being created. Independence is required.
#[ test ]
fn acc28_save_succeeds_without_settings_json()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Credentials + .claude.json present; settings.json absent.
  write_credentials( dir.path(), "max", "tier4", FAR_FUTURE_MS );
  write_claude_json_full( dir.path(), "alice@acme.com", "Alice K", "admin", "stripe" );

  let out  = run_cs_with_env( &[ ".account.save", "name::alice@acme.com" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!(
    store.join( "alice@acme.com.credentials.json" ).exists(),
    "save must create credential snapshot",
  );
  assert!(
    store.join( "alice@acme.com.claude.json" ).exists(),
    "save must create .claude.json snapshot when source present",
  );
  assert!(
    !store.join( "alice@acme.com.settings.json" ).exists(),
    "no settings.json source → no settings.json snapshot must be created",
  );
}

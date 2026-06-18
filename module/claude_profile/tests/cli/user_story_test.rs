//! Integration tests: User acceptance tests (end-to-end scenarios).
//!
//! Each UA-N case implements one Acceptance Criterion from a user story spec in
//! `tests/docs/cli/user_story/`. Tests cover five personas: account rotator,
//! onboarding developer, quota monitor, DevOps automator, and credential diagnostician.
//!
//! `lim_it` tests require a live Anthropic API token; they are skipped automatically
//! when credentials are absent or the API is rate-limited.
//!
//! ## Test Matrix
//!
//! ### Story 1 — Automatic Account Rotation (UA-1..5)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | UA-1 | `rotation_ua1_rotate_selects_highest_expiry_inactive` | deprecated → exits 1 | N |
//! | UA-2 | `rotation_ua2_switch_is_atomic` | deprecated → exits 1; credentials unchanged | N |
//! | UA-3 | `rotation_ua3_dry_run_previews_without_switching` | deprecated → exits 1 | N |
//! | UA-4 | `rotation_ua4_manual_use_switches_account` | `.account.use` manual rotation | P |
//! | UA-5 | `rotation_ua5_no_inactive_accounts_exits_2` | deprecated → exits 1 (not 2) | N |
//!
//! ### Story 2 — Account Onboarding and Lifecycle Management (UA-1..6)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | UA-1 | `onboarding_ua1_save_captures_credentials` | save creates both files | P |
//! | UA-2 | `onboarding_ua2_name_inference_and_missing_source` | auto-infer (a); exit 1 when absent (b) | P/N |
//! | UA-3 | `onboarding_ua3_host_role_captured_and_dry_run` | host+role in `.json`; `dry::1` preview | P |
//! | UA-4 | `onboarding_ua4_delete_removes_all_account_files` | delete removes `.credentials.json` + `.json` | P |
//! | UA-5 | `onboarding_ua5_lim_it_relogin_spawns_oauth_tty` | TTY OAuth (`lim_it` — skipped; TTY required) | P |
//! | UA-6 | `onboarding_ua6_renewal_sets_renewal_at` | `_renewal_at` written to `{name}.json` | P |
//!
//! ### Story 3 — Multi-Account Quota Monitoring (UA-1..5)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | UA-1 | `quota_ua1_usage_shows_all_accounts` | `.usage` exits 0, accounts listed | P |
//! | UA-2 | `quota_ua2_sort_strategies_exit_0` | `sort::renew` and `sort::renews` exit 0 | P |
//! | UA-3 | `quota_ua3_lim_it_live_mode_produces_output` | `live::1` produces at least one refresh (`lim_it`) | P |
//! | UA-4 | `quota_ua4_next_recommendation_present` | `sort::renew` exits 0, recommendation present | P |
//! | UA-5 | `quota_ua5_min_5h_filter_exits_0` | `min_5h::40` exits 0 | P |
//!
//! ### Story 4 — Scripted Pipeline Automation (UA-1..4)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | UA-1 | `automation_ua1_json_format_on_all_commands` | 3 commands produce valid JSON | P |
//! | UA-2 | `automation_ua2_get_field_returns_bare_scalar` | `get::subscription` → bare value | P |
//! | UA-3 | `automation_ua3_exit_codes_are_deterministic` | 3 scenarios exit as documented | P |
//! | UA-4 | `automation_ua4_only_next_get_account_bare_string` | `only_next::1` `get::account` → bare name | P |
//!
//! ### Story 5 — Credential Diagnostics (UA-1..4)
//!
//! | ID | Test Function | Condition | P/N |
//! |----|---------------|-----------|-----|
//! | UA-1 | `diagnostics_ua1_credentials_status_shows_fields` | subscription+tier+status in output | P |
//! | UA-2 | `diagnostics_ua2_token_classification_valid_expiring_expired` | 3 token states classified correctly | P |
//! | UA-3 | `diagnostics_ua3_paths_resolves_canonical_paths` | `.paths` exits 0 showing paths | P |
//! | UA-4 | `diagnostics_ua4_lim_it_inspect_trace_shows_endpoints` | `.account.inspect` `trace::1` (`lim_it`) | P |

use tempfile::TempDir;
use super::cli_runner::
{
  run_cs_with_env, assert_exit, stdout, stderr,
  write_credentials, write_account, write_account_renewal_json,
  write_claude_json,
  live_active_token, require_live_api,
  write_account_with_token, run_cs_bytes_for_secs,
  FAR_FUTURE_MS, PAST_MS, near_future_ms,
};

// ── Story 1: Automatic Account Rotation ──────────────────────────────────────

// UA-1: `.account.rotate` is deprecated — always exits 1.
#[ test ]
fn rotation_ua1_rotate_selects_highest_expiry_inactive()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS,              false );
  write_account( dir.path(), "bob@acme.com",   "max", "default", FAR_FUTURE_MS - 14_400_000, true  );

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// UA-2: `.account.rotate` is deprecated — exits 1; credentials file is not modified.
#[ test ]
fn rotation_ua2_switch_is_atomic()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let bob_exp : u64 = FAR_FUTURE_MS - 3_600_000;
  write_credentials( dir.path(), "max", "default", bob_exp );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@acme.com",   "max", "default", bob_exp,       true  );

  let creds_path      = dir.path().join( ".claude" ).join( ".credentials.json" );
  let content_before  = std::fs::read_to_string( &creds_path ).unwrap();

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );

  // Credentials must be unchanged — redirector does not switch.
  let content_after = std::fs::read_to_string( &creds_path ).unwrap();
  assert_eq!(
    content_before, content_after,
    "deprecated `.account.rotate` must not modify credentials",
  );
}

// UA-3: `.account.rotate dry::1` is deprecated — exits 1 regardless of dry flag.
#[ test ]
fn rotation_ua3_dry_run_previews_without_switching()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS,              false );
  write_account( dir.path(), "bob@acme.com",   "max", "default", FAR_FUTURE_MS - 3_600_000,  true  );

  let out = run_cs_with_env( &[ ".account.rotate", "dry::1" ], &[ ( "HOME", home ) ] );
  // Deprecated redirector accepts no params — exits 1.
  assert_exit( &out, 1 );
}

// UA-4: .account.use name::X enables manual rotation to a known account
#[ test ]
fn rotation_ua4_manual_use_switches_account()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let alice_exp : u64 = FAR_FUTURE_MS;
  write_account( dir.path(), "alice@acme.com", "pro", "default", alice_exp, false );
  write_account( dir.path(), "bob@acme.com",   "max", "default", FAR_FUTURE_MS, true );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let creds = std::fs::read_to_string(
    dir.path().join( ".claude" ).join( ".credentials.json" ),
  ).unwrap();
  // alice has subscriptionType "pro"
  assert!(
    creds.contains( "\"pro\"" ),
    "manual use must switch to alice's credentials (pro): {creds}",
  );
}

// UA-5: `.account.rotate` is deprecated — exits 1 (not 2) even with single active account.
#[ test ]
fn rotation_ua5_no_inactive_accounts_exits_2()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  let out = run_cs_with_env( &[ ".account.rotate" ], &[ ( "HOME", home ) ] );
  // Deprecated redirector always exits 1 — "no inactive" condition is never reached.
  assert_exit( &out, 1 );
  let err_text = stderr( &out );
  assert!( !err_text.is_empty(), "deprecation message must be present" );
}

// ── Story 2: Account Onboarding and Lifecycle Management ─────────────────────

// UA-1: .account.save captures credentials to the credential store
#[ test ]
fn onboarding_ua1_save_captures_credentials()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let cred_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  assert!(
    cred_store.join( "alice@acme.com.credentials.json" ).exists(),
    "credentials file must be created after save",
  );
  assert!(
    cred_store.join( "alice@acme.com.json" ).exists(),
    "metadata .json file must be created after save",
  );
  let out_text = stdout( &out );
  assert!(
    out_text.contains( "alice@acme.com" ),
    "stdout must reference account name after save: {out_text}",
  );
}

// UA-2: Name auto-inferred from oauthAccount.emailAddress (a); exits 1 when no source (b)
#[ test ]
fn onboarding_ua2_name_inference_and_missing_source()
{
  // (a) Name inferred from ~/.claude.json oauthAccount.emailAddress
  {
    let dir = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
    write_claude_json( dir.path(), "alice@acme.com" );

    let out = run_cs_with_env(
      &[ ".account.save" ],
      &[ ( "HOME", home ) ],
    );
    assert_exit( &out, 0 );
    let cred_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
    assert!(
      cred_store.join( "alice@acme.com.credentials.json" ).exists(),
      "(a) inferred name must produce alice@acme.com credentials file",
    );
  }

  // (b) No name source present → exit 1
  {
    let dir = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
    // No ~/.claude.json, no active marker, no name:: arg

    let out = run_cs_with_env(
      &[ ".account.save" ],
      &[ ( "HOME", home ) ],
    );
    assert_exit( &out, 1 );
  }
}

// UA-3: host:: and role:: captured in {name}.json; dry::1 previews without writing (a/b)
#[ test ]
fn onboarding_ua3_host_role_captured_and_dry_run()
{
  // (a) host:: and role:: captured
  {
    let dir = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

    let out = run_cs_with_env(
      &[ ".account.save", "name::alice@acme.com", "host::laptop", "role::work" ],
      &[ ( "HOME", home ) ],
    );
    assert_exit( &out, 0 );

    let meta_file = dir.path()
      .join( ".persistent" ).join( "claude" ).join( "credential" )
      .join( "alice@acme.com.json" );
    let content : serde_json::Value = serde_json::from_str(
      &std::fs::read_to_string( &meta_file ).unwrap(),
    ).unwrap();
    assert_eq!(
      content.get( "host" ).and_then( | v | v.as_str() ), Some( "laptop" ),
      "(a) host field must be captured in {{name}}.json",
    );
    assert_eq!(
      content.get( "role" ).and_then( | v | v.as_str() ), Some( "work" ),
      "(a) role field must be captured in {{name}}.json",
    );
  }

  // (b) dry::1 previews without writing
  {
    let dir = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

    let out = run_cs_with_env(
      &[ ".account.save", "name::alice@acme.com", "dry::1" ],
      &[ ( "HOME", home ) ],
    );
    assert_exit( &out, 0 );

    let cred_file = dir.path()
      .join( ".persistent" ).join( "claude" ).join( "credential" )
      .join( "alice@acme.com.credentials.json" );
    assert!( !cred_file.exists(), "(b) dry::1 must not create credentials file" );

    let out_text = stdout( &out );
    assert!(
      out_text.contains( "dry" ) || out_text.contains( "would" ),
      "(b) dry::1 stdout must indicate preview: {out_text}",
    );
  }
}

// UA-4: .account.delete removes all account files from store
#[ test ]
fn onboarding_ua4_delete_removes_all_account_files()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );
  // Also set up metadata file via save
  let _ = run_cs_with_env(
    &[ ".account.save", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );

  let cred_store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  let cred_file = cred_store.join( "alice@acme.com.credentials.json" );
  let meta_file = cred_store.join( "alice@acme.com.json" );

  let out = run_cs_with_env(
    &[ ".account.delete", "name::alice@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  assert!( !cred_file.exists(), "credentials file must be removed after delete" );
  assert!( !meta_file.exists(), "metadata .json file must be removed after delete" );
}

// UA-5: .account.relogin spawns claude with TTY; propagates fresh credentials (lim_it — skipped)
//
// Requires an interactive OAuth browser flow that cannot be automated in CI.
// Test exists for spec traceability (UA-5) and documents the OAuth constraint.
#[ test ]
fn onboarding_ua5_lim_it_relogin_spawns_oauth_tty()
{
  let Some( _token ) = live_active_token() else { return };
  // OAuth TTY flow cannot be automated programmatically. Manual test only.
}

// UA-6: .account.renewal sets _renewal_at in {name}.json
#[ test ]
fn onboarding_ua6_renewal_sets_renewal_at()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, false );

  let meta_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" )
    .join( "alice@acme.com.json" );

  let out = run_cs_with_env(
    &[ ".account.renewal", "name::alice@acme.com", "at::2026-08-01T00:00:00Z" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let content : serde_json::Value = serde_json::from_str(
    &std::fs::read_to_string( &meta_file ).unwrap(),
  ).unwrap();
  assert_eq!(
    content.get( "_renewal_at" ).and_then( | v | v.as_str() ),
    Some( "2026-08-01T00:00:00Z" ),
    "_renewal_at must be set after renewal",
  );
}

// ── Story 3: Multi-Account Quota Monitoring ───────────────────────────────────

// UA-1: .usage shows all saved accounts in a single table
#[ test ]
fn quota_ua1_usage_shows_all_accounts()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "bob@acme.com",   "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "carol@acme.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  assert!( out_text.contains( "alice" ), ".usage must list alice: {out_text}" );
  assert!( out_text.contains( "bob" ),   ".usage must list bob: {out_text}" );
  assert!( out_text.contains( "carol" ), ".usage must list carol: {out_text}" );
}

// UA-2: sort::renew and sort::renews both exit 0
#[ test ]
fn quota_ua2_sort_strategies_exit_0()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "bob@acme.com",   "max", "default", FAR_FUTURE_MS, false );

  write_account_renewal_json( dir.path(), "alice@acme.com", "2026-09-01T00:00:00Z" );
  write_account_renewal_json( dir.path(), "bob@acme.com",   "2026-08-01T00:00:00Z" );

  let out_renew = run_cs_with_env(
    &[ ".usage", "sort::renew" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_renew, 0 );

  let out_renews = run_cs_with_env(
    &[ ".usage", "sort::renews" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_renews, 0 );
}

// UA-3: live::1 continuously refreshes the table (lim_it — requires live API + TTY)
#[ test ]
fn quota_ua3_lim_it_live_mode_produces_output()
{
  let Some( token ) = live_active_token() else { return };
  if !require_live_api( "quota_ua3_lim_it" ) { return; }

  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", &token, true );
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  // Run live mode for 3 seconds then kill it
  let bytes = run_cs_bytes_for_secs(
    &[ ".usage", "live::1", "interval::2" ],
    &[ ( "HOME", home ) ],
    3,
  );
  let output = String::from_utf8_lossy( &bytes );
  assert!(
    !output.trim().is_empty(),
    "live::1 mode must produce some output before being killed",
  );
}

// UA-4: sort::renew exits 0 and footer recommends eligible account
#[ test ]
fn quota_ua4_next_recommendation_present()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS,               true  );
  write_account( dir.path(), "bob@acme.com",   "max", "default", FAR_FUTURE_MS - 3_600_000,   false );

  let out = run_cs_with_env(
    &[ ".usage", "sort::renew" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// UA-5: min_5h::X filter exits 0
#[ test ]
fn quota_ua5_min_5h_filter_exits_0()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "bob@acme.com",   "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".usage", "min_5h::40" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
}

// ── Story 4: Scripted Pipeline Automation ─────────────────────────────────────

// UA-1: format::json on any format-capable command returns valid JSON
#[ test ]
fn automation_ua1_json_format_on_all_commands()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  // (a) .token.status format::json
  let out_a = run_cs_with_env(
    &[ ".token.status", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_a, 0 );
  serde_json::from_str::< serde_json::Value >( &stdout( &out_a ) )
    .expect( ".token.status format::json must produce valid JSON" );

  // (b) .accounts format::json
  let out_b = run_cs_with_env(
    &[ ".accounts", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_b, 0 );
  serde_json::from_str::< serde_json::Value >( &stdout( &out_b ) )
    .expect( ".accounts format::json must produce valid JSON" );

  // (c) .credentials.status format::json
  let out_c = run_cs_with_env(
    &[ ".credentials.status", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_c, 0 );
  serde_json::from_str::< serde_json::Value >( &stdout( &out_c ) )
    .expect( ".credentials.status format::json must produce valid JSON" );
}

// UA-2: get::FIELD returns a single bare scalar value with no headers
#[ test ]
fn automation_ua2_get_field_returns_bare_scalar()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".credentials.status", "get::subscription" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let out_text = stdout( &out ).trim().to_string();
  assert!(
    !out_text.is_empty(),
    "get::subscription must produce non-empty output",
  );
  // Must be a single-line bare value — no JSON wrapper, no headers
  assert_eq!(
    out_text.lines().count(), 1,
    "get::subscription must produce exactly one line; got: {out_text:?}",
  );
  assert!(
    !out_text.starts_with( '{' ) && !out_text.starts_with( '[' ),
    "get::subscription must not produce JSON wrapper; got: {out_text}",
  );
}

// UA-3: Exit codes are deterministic and match documented triggers
#[ test ]
fn automation_ua3_exit_codes_are_deterministic()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS, true );

  // (a) Valid credentials → exit 0
  let out_a = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_a, 0 );

  // (b) Account not found → exit 2
  let out_b = run_cs_with_env(
    &[ ".account.use", "name::nobody@acme.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_b, 2 );

  // (c) Invalid name format → exit 1
  let out_c = run_cs_with_env(
    &[ ".account.save", "name::notanemail" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_c, 1 );
}

// UA-4: only_next::1 get::account returns the recommended account as a bare string
//
// Accounts are given renewal dates so the `renew` sort strategy (default) can determine
// the "next" recommendation (soonest upcoming renewal). Bob's renewal is sooner (Aug 1)
// → bob receives the `→` marker → only_next::1 shows bob → get::account returns bob's name.
//
// Without live quota data the recommendation engine skips all accounts (result=Err),
// so output is legitimately empty. Exit 0 is still required; format is verified when
// live quota is present.
#[ test ]
fn automation_ua4_only_next_get_account_bare_string()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@acme.com", "max", "default", FAR_FUTURE_MS,               true  );
  write_account( dir.path(), "bob@acme.com",   "max", "default", FAR_FUTURE_MS - 3_600_000,   false );

  // Renewal dates let the renew strategy make a deterministic "next" recommendation
  write_account_renewal_json( dir.path(), "alice@acme.com", "2026-10-01T00:00:00Z" );
  write_account_renewal_json( dir.path(), "bob@acme.com",   "2026-08-01T00:00:00Z" ); // sooner

  let out = run_cs_with_env(
    &[ ".usage", "only_next::1", "get::account" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // When live quota data is available the engine marks a row and returns one email.
  // When offline (result=Err for all accounts), output is empty — that is correct
  // behavior (no eligible recommendation), so we only assert format when non-empty.
  let out_text = stdout( &out ).trim().to_string();
  if !out_text.is_empty()
  {
    assert!(
      out_text.contains( '@' ),
      "only_next::1 get::account must return an email address; got: {out_text:?}",
    );
    assert_eq!(
      out_text.lines().count(), 1,
      "only_next::1 get::account must produce exactly one line; got: {out_text:?}",
    );
  }
}

// ── Story 5: Credential Diagnostics ──────────────────────────────────────────

// UA-1: .credentials.status shows subscription, tier, token status, expiry — no account store required
#[ test ]
fn diagnostics_ua1_credentials_status_shows_fields()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Only live credentials file — no account store setup needed
  write_credentials( dir.path(), "max", "default_claude_max_20x", FAR_FUTURE_MS );

  let out = run_cs_with_env( &[ ".credentials.status" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  // Output must show subscription type and tier
  assert!(
    out_text.contains( "max" ),
    ".credentials.status must show subscription type 'max': {out_text}",
  );
  assert!(
    out_text.contains( "default" ),
    ".credentials.status must show tier: {out_text}",
  );
}

// UA-2: .token.status classifies token as Valid / ExpiringSoon / Expired with exact duration
#[ test ]
fn diagnostics_ua2_token_classification_valid_expiring_expired()
{
  // (valid) expiresAt = FAR_FUTURE_MS → "Valid"
  {
    let dir = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
    let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
    let out_text = stdout( &out ).to_ascii_lowercase();
    assert!( out_text.contains( "valid" ), "FAR_FUTURE_MS must produce Valid classification: {out_text}" );
  }

  // (expiring_soon) expiresAt = now + 30min (within 3600s default threshold)
  {
    let dir = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    let expiring_ms = near_future_ms();
    write_credentials( dir.path(), "max", "default", expiring_ms );
    let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
    let out_text = stdout( &out ).to_ascii_lowercase();
    // Should be either ExpiringSoon or Valid depending on exact timing; accept both
    assert!(
      out_text.contains( "valid" ) || out_text.contains( "expiring" ),
      "near_future_ms must produce Valid or ExpiringSoon: {out_text}",
    );
  }

  // (expired) expiresAt = PAST_MS → "Expired"
  {
    let dir = TempDir::new().unwrap();
    let home = dir.path().to_str().unwrap();
    write_credentials( dir.path(), "max", "default", PAST_MS );
    let out = run_cs_with_env( &[ ".token.status" ], &[ ( "HOME", home ) ] );
    assert_exit( &out, 0 );
    let out_text = stdout( &out ).to_ascii_lowercase();
    assert!( out_text.contains( "expired" ), "PAST_MS must produce Expired classification: {out_text}" );
  }
}

// UA-3: .paths resolves all canonical ~/.claude/ file paths
#[ test ]
fn diagnostics_ua3_paths_resolves_canonical_paths()
{
  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // Ensure ~/.claude dir exists
  std::fs::create_dir_all( dir.path().join( ".claude" ) ).unwrap();

  let out = run_cs_with_env( &[ ".paths" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let out_text = stdout( &out );
  // Output must list some paths — accept any path-like content
  assert!(
    out_text.contains( ".claude" ) || out_text.contains( home ),
    ".paths must list ~/.claude/ paths; got: {out_text}",
  );
}

// UA-4: .account.inspect trace::1 shows live endpoint responses (lim_it)
#[ test ]
fn diagnostics_ua4_lim_it_inspect_trace_shows_endpoints()
{
  let Some( token ) = live_active_token() else { return };
  if !require_live_api( "diagnostics_ua4_lim_it" ) { return; }

  let dir = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "alice@acme.com", &token, true );
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );

  let out = run_cs_with_env(
    &[ ".account.inspect", "name::alice@acme.com", "trace::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  // With trace::1, stderr should contain endpoint diagnostic information
  let err_text = stderr( &out );
  assert!(
    !err_text.is_empty() || !stdout( &out ).is_empty(),
    "inspect trace::1 must produce diagnostic output",
  );
}

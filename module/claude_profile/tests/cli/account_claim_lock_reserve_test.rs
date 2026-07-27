//! Integration tests: `claim_lock`/`reserve` account properties (Feature 070) —
//! Gate 9 (unconditional eligibility exclusion), G9 (`force::1`-bypassable explicit-command
//! gate), and the `reserve` leading sort key.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//!
//! ## Test Matrix
//!
//! Maps `t01`..`t17`/`t10u`/`t11u` function names to the Test Matrix rows in
//! `task/claude_profile/415_claim_lock_reserve_account_properties.md`.
//!
//! | Function | Row | Condition | P/N |
//! |----------|-----|-----------|-----|
//! | `t01_lock_1_sets_claim_lock_true` | T01 | `.accounts lock::1 name::X` writes `claim_lock` | P |
//! | `t02_lock_0_clears_claim_lock` | T02 | `.accounts lock::0 name::X` clears `claim_lock` | P |
//! | `t03_lock_1_comma_list_batch_sets_both` | T03 | comma-list batch write | P |
//! | `t04_lock_1_no_name_batch_sets_all` | T04 | no `name::` batch write | P |
//! | `t05_gate9_excludes_locked_from_footer_recommendation` | T05 | Gate 9 excludes locked account from `Next (...)` | N |
//! | `t06_gate9_skips_locked_winner_during_rotate` | T06 | Gate 9 skips locked winner during `rotate::1` | N |
//! | `t07_gate9_not_bypassed_by_force_during_rotate` | T07 | `force::1` does NOT bypass Gate 9 | N |
//! | `t08_g9_blocks_account_use_direct_target` | T08 | G9 blocks `.account.use` direct target | N |
//! | `t09_g9_force_bypasses_account_use` | T09 | `force::1` bypasses G9 on `.account.use` | P |
//! | `t10_g9_blocks_accounts_assignee_target` | T10 | G9 blocks `.accounts assignee::` target | N |
//! | `t11_g9_force_bypasses_accounts_assignee` | T11 | `force::1` bypasses G9 on `.accounts assignee::` | P |
//! | `t10u_g9_blocks_usage_assignee_target` | T10u | G9 blocks `.usage assignee::` target (independent dispatch) | N |
//! | `t11u_g9_force_bypasses_usage_assignee` | T11u | `force::1` bypasses G9 on `.usage assignee::` | P |
//! | `t12_reserve_1_sets_reserve_true` | T12 | `.accounts reserve::1 name::X` writes `reserve` | P |
//! | `t13_reserve_leading_sort_key_non_reserved_first` | T13 | `reserve` leading sort key — non-reserved sorts first | P |
//! | `t14_reserve_fallback_selected_when_only_eligible` | T14 | reserved account IS selected as fallback | P |
//! | `t15_lock_ungated_despite_foreign_owner` | T15 | `lock::` write ungated despite non-ownership | P |
//! | `t16_read_side_unaffected_by_claim_lock_or_reserve` | T16 | quota display unaffected by either field | P |
//! | `t17_lock_dry_run_preview_no_write` | T17 | `dry::1` preview; `alice.json` unchanged on disk | P |

use crate::cli_runner::
{
  run_cs_with_env,
  stdout, stderr, assert_exit,
  write_credentials, write_account, write_account_owner,
  write_account_claim_lock, write_account_reserve, write_account_quota_cache,
  read_account_meta,
  live_active_token, require_live_api, write_live_credentials_with_token, write_account_with_token,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── T01–T04: lock:: write mechanics (batch, comma-list, no-name) ──────────────

/// T01: `.accounts lock::1 name::alice` writes `claim_lock: true` — succeeds
/// regardless of `owner` (ungated, AC-02).
///
/// The named `lock::`/`reserve::` dispatch path requires `{name}.json` to
/// already exist (same documented precondition as the named `owner::` path —
/// see `tests/docs/cli/param/63_owner.md` EC-19); pre-seed a `false` baseline
/// exactly as every existing `owner::` test seeds via `write_account_owner`.
#[ test ]
fn t01_lock_1_sets_claim_lock_true()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_claim_lock( dir.path(), "alice@test.com", false );

  let out = run_cs_with_env(
    &[ ".accounts", "lock::1", "name::alice@test.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "alice@test.com" );
  assert_eq!( meta[ "claim_lock" ], serde_json::json!( true ), "T01: claim_lock must be true, got:\n{meta}" );
}

/// T02: `.accounts lock::0 name::alice` clears a previously-set `claim_lock`.
#[ test ]
fn t02_lock_0_clears_claim_lock()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_claim_lock( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".accounts", "lock::0", "name::alice@test.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "alice@test.com" );
  assert_eq!( meta[ "claim_lock" ], serde_json::json!( false ), "T02: claim_lock must be false, got:\n{meta}" );
}

/// T03: `.accounts lock::1 name::alice,bob` batch-writes `claim_lock: true` to
/// both accounts via comma-list.
///
/// Comma-list resolution goes through the same named-dispatch existence check
/// as a single name (`owner_dispatch.rs`'s `{name}.json` precondition) — both
/// accounts need a pre-seeded baseline, same as T01.
#[ test ]
fn t03_lock_1_comma_list_batch_sets_both()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@test.com",   "max", "default", FAR_FUTURE_MS, false );
  write_account_claim_lock( dir.path(), "alice@test.com", false );
  write_account_claim_lock( dir.path(), "bob@test.com",   false );

  let out = run_cs_with_env(
    &[ ".accounts", "lock::1", "name::alice@test.com,bob@test.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  assert_eq!( read_account_meta( dir.path(), "alice@test.com" )[ "claim_lock" ], serde_json::json!( true ), "T03: alice claim_lock must be true" );
  assert_eq!( read_account_meta( dir.path(), "bob@test.com" )[ "claim_lock" ], serde_json::json!( true ), "T03: bob claim_lock must be true" );
}

/// T04: `.accounts lock::1` with no `name::` batch-writes `claim_lock: true`
/// to every account in the credential store.
#[ test ]
fn t04_lock_1_no_name_batch_sets_all()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account( dir.path(), "bob@test.com",   "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env(
    &[ ".accounts", "lock::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  assert_eq!( read_account_meta( dir.path(), "alice@test.com" )[ "claim_lock" ], serde_json::json!( true ), "T04: alice claim_lock must be true" );
  assert_eq!( read_account_meta( dir.path(), "bob@test.com" )[ "claim_lock" ], serde_json::json!( true ), "T04: bob claim_lock must be true" );
}

// ── T05–T07: Gate 9 (unconditional eligibility exclusion) ─────────────────────

/// T05: a claim-locked, otherwise-Green/eligible account never appears as the
/// `Next (strategy):` footer recommendation — a different eligible account is
/// recommended instead.
///
/// The `Current`/`Next (...)` 2-line footer format only renders when
/// `render.rs` finds an `is_current` account (`accounts.iter().find(|aq|
/// aq.is_current)`); `is_current` is a genuine live-token match
/// (`usage/fetch.rs`'s `stored == *live` comparison against
/// `~/.claude/.credentials.json`), which `write_account`/`write_credentials`
/// (no `accessToken` field) can never produce. This is architecturally
/// live-token-only — opportunistic like `it102`/`it103` in
/// `usage_touch_test.rs`, which gate the identical footer-text assertion
/// behind `live_active_token()`. Gate 9 itself is exhaustively covered
/// offline via `rotate::1`'s `switched to` message in T06/T07.
///
/// `alice` sorts alphabetically before `zzz` and is otherwise identically
/// eligible; without Gate 9, `sort::name` would recommend `alice`. Asserting
/// `zzz` wins instead proves Gate 9 fired.
#[ test ]
fn t05_gate9_excludes_locked_from_footer_recommendation()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "t05: no live token — skipping" );
    return;
  };
  require_live_api( "t05" );

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_live_credentials_with_token( dir.path(), &token );
  write_account_with_token( dir.path(), "current@test.com", &token, true );
  write_account( dir.path(), "alice@test.com", "max", "tier-alice", FAR_FUTURE_MS, false );
  write_account( dir.path(), "zzz@test.com",   "max", "tier-zzz",   FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "alice@test.com", 20.0, 30.0, None );
  write_account_quota_cache( dir.path(), "zzz@test.com",   20.0, 30.0, None );
  write_account_claim_lock( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".usage", "sort::name" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  let next_line = text.lines().find( |l| l.trim_start().starts_with( "Next (" ) )
    .unwrap_or_else( || panic!( "T05: no 'Next (...)' footer line found, got:\n{text}" ) );
  assert!( !next_line.contains( "alice@test.com" ), "T05: claim-locked alice must never appear in Next(...) line, got:\n{next_line}" );
  assert!( next_line.contains( "zzz@test.com" ), "T05: non-locked zzz must be recommended instead, got:\n{next_line}" );
}

/// T06: `.usage rotate::1` skips a claim-locked would-be winner and switches
/// to the next-eligible account instead.
#[ test ]
fn t06_gate9_skips_locked_winner_during_rotate()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier-current", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "tier-current", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alice@test.com",   "max", "tier-alice",   FAR_FUTURE_MS, false );
  write_account( dir.path(), "zzz@test.com",     "max", "tier-zzz",     FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "alice@test.com", 20.0, 30.0, None );
  write_account_quota_cache( dir.path(), "zzz@test.com",   20.0, 30.0, None );
  write_account_claim_lock( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "sort::name" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!( text.contains( "switched to 'zzz@test.com'" ), "T06: Gate 9 must skip locked alice and switch to zzz, got:\n{text}" );

  let live = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  assert!( live.contains( "tier-zzz" ), "T06: live credentials must reflect switch to zzz (not alice), got:\n{live}" );
}

/// T07 (AF1): `force::1` does NOT bypass Gate 9 during `rotate::1` — unlike
/// Gate 8 (ownership), which `force::1` does bypass. A claim-locked winner is
/// still skipped even with `force::1` set.
#[ test ]
fn t07_gate9_not_bypassed_by_force_during_rotate()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier-current", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "tier-current", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alice@test.com",   "max", "tier-alice",   FAR_FUTURE_MS, false );
  write_account( dir.path(), "zzz@test.com",     "max", "tier-zzz",     FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "alice@test.com", 20.0, 30.0, None );
  write_account_quota_cache( dir.path(), "zzz@test.com",   20.0, 30.0, None );
  write_account_claim_lock( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "sort::name", "force::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!( !text.contains( "switched to 'alice@test.com'" ), "T07: force::1 must NOT bypass Gate 9 — alice must never be selected, got:\n{text}" );
  assert!( text.contains( "switched to 'zzz@test.com'" ), "T07: zzz must still be selected with force::1 set, got:\n{text}" );
}

// ── T08–T09: G9 on `.account.use` direct target ───────────────────────────────

/// T08: `.account.use name::alice` exits 1 with a claim-lock violation message
/// when `alice.claim_lock == true`; the switch does not occur.
#[ test ]
fn t08_g9_blocks_account_use_direct_target()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@test.com", "max", "tier4", FAR_FUTURE_MS, false );
  write_account_claim_lock( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@test.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "claim-lock violation" ), "T08: stderr must contain 'claim-lock violation', got:\n{err}" );
  assert!( err.contains( "alice@test.com" ), "T08: stderr must name the locked account, got:\n{err}" );

  let live = std::fs::read_to_string( dir.path().join( ".claude" ).join( ".credentials.json" ) ).unwrap();
  assert!( live.contains( "standard" ) && !live.contains( "tier4" ), "T08: switch_account() must not run — live credentials unchanged, got:\n{live}" );
}

/// T09: `.account.use name::alice force::1` bypasses G9 — the switch proceeds
/// normally despite `alice.claim_lock == true`.
#[ test ]
fn t09_g9_force_bypasses_account_use()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "pro", "standard", FAR_FUTURE_MS );
  write_account( dir.path(), "alice@test.com", "max", "tier4", FAR_FUTURE_MS, false );
  write_account_claim_lock( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".account.use", "name::alice@test.com", "force::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "switched to 'alice@test.com'" ), "T09: force::1 must bypass G9 and complete the switch, got:\n{text}" );
}

// ── T10–T11: G9 on `.accounts assignee::` target-side ──────────────────────────

/// T10: `.accounts assignee::user@host name::alice` exits 1 with a claim-lock
/// violation message; the assignee marker is not written.
#[ test ]
fn t10_g9_blocks_accounts_assignee_target()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_claim_lock( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::user@host", "name::alice@test.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "claim-lock violation" ), "T10: stderr must contain 'claim-lock violation', got:\n{err}" );

  let marker_path = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "_active_host_user" );
  assert!( !marker_path.exists(), "T10: assignee marker must not be written when G9 blocks" );
}

/// T11: `.accounts assignee::user@host name::alice force::1` bypasses G9 —
/// the marker is written normally.
#[ test ]
fn t11_g9_force_bypasses_accounts_assignee()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_claim_lock( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".accounts", "assignee::user@host", "name::alice@test.com", "force::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let marker_path = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "_active_host_user" );
  assert!( marker_path.exists(), "T11: assignee marker must be written when force::1 bypasses G9" );
  let marker = std::fs::read_to_string( &marker_path ).unwrap();
  assert_eq!( marker, "alice@test.com", "T11: marker must point at alice@test.com, got:\n{marker}" );
}

// ── T10u–T11u: G9 on `.usage assignee::` target-side (independent dispatch) ───

/// T10u (AF3): `.usage assignee::user@host name::alice` exits 1 with a
/// claim-lock violation message via `usage/api_dispatch.rs`'s own independent
/// `dispatch_assignee_param()` — not a call-through to `commands/accounts.rs`.
#[ test ]
fn t10u_g9_blocks_usage_assignee_target()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_claim_lock( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".usage", "assignee::user@host", "name::alice@test.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!( err.contains( "claim-lock violation" ), "T10u: stderr must contain 'claim-lock violation', got:\n{err}" );

  let marker_path = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "_active_host_user" );
  assert!( !marker_path.exists(), "T10u: assignee marker must not be written when G9 blocks via .usage" );
}

/// T11u (AF3): `.usage assignee::user@host name::alice force::1` bypasses G9
/// via `usage/api_dispatch.rs`'s independent dispatch — marker is written.
#[ test ]
fn t11u_g9_force_bypasses_usage_assignee()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_claim_lock( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".usage", "assignee::user@host", "name::alice@test.com", "force::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let marker_path = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" ).join( "_active_host_user" );
  assert!( marker_path.exists(), "T11u: assignee marker must be written when force::1 bypasses G9 via .usage" );
  let marker = std::fs::read_to_string( &marker_path ).unwrap();
  assert_eq!( marker, "alice@test.com", "T11u: marker must point at alice@test.com, got:\n{marker}" );
}

// ── T12–T14: reserve (leading sort key, not a gate) ────────────────────────────

/// T12: `.accounts reserve::1 name::alice` writes `reserve: true`.
///
/// Same named-dispatch `{name}.json` precondition as T01 — pre-seed a `false`
/// baseline before testing the actual write.
#[ test ]
fn t12_reserve_1_sets_reserve_true()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_reserve( dir.path(), "alice@test.com", false );

  let out = run_cs_with_env(
    &[ ".accounts", "reserve::1", "name::alice@test.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let meta = read_account_meta( dir.path(), "alice@test.com" );
  assert_eq!( meta[ "reserve" ], serde_json::json!( true ), "T12: reserve must be true, got:\n{meta}" );
}

/// T13 (AC-05): with one Green reserved account and one Green non-reserved
/// account otherwise equally eligible, the non-reserved account sorts first
/// under `sort::renew`.
///
/// The `Next (...)` footer requires a live-session match (`is_current`);
/// opportunistic like `it102`/`it103` — skips without a real token.
///
/// `aaa` is named to sort first under every OTHER key (name tiebreak, likely
/// insertion order) — reserved accounts only sort after `zzz` if `reserve` is
/// genuinely applied as the sort's leading key, not merely a coincidence of
/// naming.
#[ test ]
fn t13_reserve_leading_sort_key_non_reserved_first()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "t13: no live token — skipping" );
    return;
  };
  require_live_api( "t13" );

  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_live_credentials_with_token( dir.path(), &token );
  write_account_with_token( dir.path(), "current@test.com", &token, true );
  write_account( dir.path(), "aaa@test.com", "max", "tier-aaa", FAR_FUTURE_MS, false );
  write_account( dir.path(), "zzz@test.com", "max", "tier-zzz", FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "aaa@test.com", 20.0, 30.0, None );
  write_account_quota_cache( dir.path(), "zzz@test.com", 20.0, 30.0, None );
  write_account_reserve( dir.path(), "aaa@test.com", true );

  let out = run_cs_with_env(
    &[ ".usage", "sort::renew" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  let next_line = text.lines().find( |l| l.trim_start().starts_with( "Next (" ) )
    .unwrap_or_else( || panic!( "T13: no 'Next (...)' footer line found, got:\n{text}" ) );
  assert!( next_line.contains( "zzz@test.com" ), "T13: non-reserved zzz must sort/recommend first over reserved aaa, got:\n{next_line}" );
  assert!( !next_line.contains( "aaa@test.com" ), "T13: reserved aaa must not be the Next(...) recommendation while zzz is eligible, got:\n{next_line}" );
}

/// T14 (AC-05, AF2): when every Green-eligible account is reserved (no
/// non-reserved alternative remains), `rotate::1` still selects the reserved
/// account — confirming `reserve` is a sort key, not a gate.
#[ test ]
fn t14_reserve_fallback_selected_when_only_eligible()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "tier-current", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "tier-current", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alice@test.com",   "max", "tier-alice",   FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "alice@test.com", 20.0, 30.0, None );
  write_account_reserve( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".usage", "rotate::1", "sort::name" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );

  let text = stdout( &out );
  assert!( text.contains( "switched to 'alice@test.com'" ), "T14: reserved alice must still be selected as fallback, got:\n{text}" );
}

// ── T15: lock:: is ungated (AC-02) ─────────────────────────────────────────────

/// T15: `.accounts lock::1 name::alice` succeeds even when `alice.owner` is a
/// different machine (non-owned) — `lock::`/`reserve::` writes are never
/// ownership-gated.
#[ test ]
fn t15_lock_ungated_despite_foreign_owner()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_owner( dir.path(), "alice@test.com", "other@remote" );

  let out = run_cs_with_env(
    &[ ".accounts", "lock::1", "name::alice@test.com" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!( !err.contains( "ownership violation" ), "T15: lock:: must not be ownership-gated, got:\n{err}" );

  let meta = read_account_meta( dir.path(), "alice@test.com" );
  assert_eq!( meta[ "claim_lock" ], serde_json::json!( true ), "T15: claim_lock must be true despite foreign owner, got:\n{meta}" );
}

// ── T16: read-side unaffected (AC-07) ──────────────────────────────────────────

/// T16: quota display is unaffected by `claim_lock`/`reserve` — an account
/// with both fields `true` still appears normally in the `.usage` table.
#[ test ]
fn t16_read_side_unaffected_by_claim_lock_or_reserve()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_credentials( dir.path(), "max", "default", FAR_FUTURE_MS );
  write_account( dir.path(), "current@test.com", "max", "default", FAR_FUTURE_MS, true  );
  write_account( dir.path(), "alice@test.com",   "max", "default", FAR_FUTURE_MS, false );
  write_account_quota_cache( dir.path(), "alice@test.com", 20.0, 30.0, None );
  write_account_claim_lock( dir.path(), "alice@test.com", true );
  write_account_reserve( dir.path(), "alice@test.com", true );

  let out = run_cs_with_env(
    &[ ".usage" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "alice@test.com" ), "T16: locked+reserved account must still appear in the usage table, got:\n{text}" );
}

// ── T17: dry-run preview (AC-06) ───────────────────────────────────────────────

/// T17: `.accounts lock::1 name::alice dry::1` prints a preview but leaves
/// `alice.json` unchanged on disk.
///
/// Pre-seeds a `false` baseline (same named-dispatch precondition as T01) so
/// the "unchanged" assertion has an existing value to confirm was not
/// overwritten, rather than asserting on key absence.
#[ test ]
fn t17_lock_dry_run_preview_no_write()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account( dir.path(), "alice@test.com", "max", "default", FAR_FUTURE_MS, false );
  write_account_claim_lock( dir.path(), "alice@test.com", false );

  let out = run_cs_with_env(
    &[ ".accounts", "lock::1", "name::alice@test.com", "dry::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!( text.contains( "[dry-run] would set lock of alice@test.com to true" ), "T17: dry-run must preview the change, got:\n{text}" );

  let meta = read_account_meta( dir.path(), "alice@test.com" );
  assert_eq!( meta[ "claim_lock" ], serde_json::json!( false ), "T17: alice.json must be unchanged on disk (claim_lock stays false), got:\n{meta}" );
}

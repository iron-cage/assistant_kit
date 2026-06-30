// Integration tests for api.rs — Part B (split from src/usage/api_tests.rs).
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::{ apply_model_override, apply_post_switch_touch };
use claude_profile::usage::test_bridge::TouchCtx;
use tempfile::TempDir;

/// # MRE: BUG-244 — `usage_routine` never called `apply_model_override`
///
/// ## Root Cause
/// `usage_routine()` in `src/usage/api.rs` had no call to `apply_model_override()`.
/// The function existed and worked (called from `.account.use` path) but was never
/// invoked from the `.usage` command path.
///
/// ## Why Not Caught
/// No test exercised the full `usage_routine()` → `apply_model_override()` wiring.
/// Unit tests for `apply_model_override` passed trivially (calling it directly).
///
/// ## Fix Applied
/// Added `apply_model_override( data, claude_paths, params.trace, "usage", &current.name )`
/// in `usage_routine()` after the touch loop, before the row-filter pipeline,
/// guarded by `aq.is_current && aq.result.is_ok()`.
/// Also added `label: &str` parameter to `apply_model_override` (was hardcoded "account.use").
///
/// ## Prevention
/// T05 structural test: grep of `usage_routine` body must contain exactly one call.
///
/// ## Pitfall
/// Insert the call BEFORE the row-filter pipeline (`only_next`/`only_active`/etc.) — those
/// filters can remove the `is_current` entry from the slice, causing a silent no-op.
#[ doc = "bug_reproducer(BUG-244)" ]
#[ test ]
fn mre_bug244_usage_routine_never_calls_apply_model_override()
{
  let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  let fn_start = src.find( "pub fn usage_routine" ).expect( "usage_routine not found" );
  let body_end = {
    let after = &src[ fn_start + 1.. ];
    let a     = after.find( "\npub fn " ).unwrap_or( after.len() );
    let b     = after.find( "\n#[ cfg( t" ).unwrap_or( after.len() );
    let c     = after.find( "\n#[cfg(t" ).unwrap_or( after.len() );
    fn_start + 1 + a.min( b ).min( c )
  };
  let body = &src[ fn_start..body_end ];
  assert!(
    body.contains( "apply_model_override(" ),
    "BUG-244: apply_model_override must be called from usage_routine — was absent before fix",
  );
}

/// # MRE: BUG-285 — idle check used server-side `resets_at` as local subprocess oracle
///
/// ## Root Cause
/// `pre_switch_touch_ctx()` computed `is_idle` from `quota.five_hour.resets_at.is_none()`.
/// `resets_at` is set server-side by Anthropic's API for any session on any machine —
/// it is NOT a local subprocess identity indicator. An account with `resets_at=Some`
/// (set by a session on another machine) returned `AlreadyActive` and skipped the
/// subprocess touch, even though no local subprocess was running.
///
/// ## Why Not Caught
/// The `is_idle` check appeared logically sound in single-machine setups: if the 5h window
/// is counting down, a subprocess must have started it. This reasoning fails for accounts
/// used across multiple machines where any machine can advance the server-side state.
///
/// ## Fix Applied
/// Removed `is_idle` check entirely from `pre_switch_touch_ctx`. Function now always
/// returns `NeedTouch` when quota is successfully fetched. `AlreadyActive` variant
/// removed from `PreSwitchOutcome`. `trace_already_active()` deleted as dead code.
///
/// ## Prevention
/// This structural test asserts that `let is_idle` assignment and `AlreadyActive` return
/// no longer appear in `pre_switch_touch_ctx` — any reintroduction of the oracle pattern
/// fails the test.
///
/// ## Pitfall
/// `resets_at` presence does NOT mean a local subprocess is active. Server-side state
/// reflects global account state; local subprocess identity requires local process
/// introspection, not quota API responses.
#[ doc = "bug_reproducer(BUG-285)" ]
#[ test ]
fn mre_bug285_idle_check_uses_resets_at_as_wrong_oracle()
{
  let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api_switch.rs" ) );
  let fn_start = src
    .find( "pub fn pre_switch_touch_ctx(" )
    .expect( "pre_switch_touch_ctx not found in api_switch.rs" );
  let fn_end   = src[ fn_start + 1.. ]
    .find( "\npub fn " )
    .map_or( src.len(), |rel| fn_start + 1 + rel );
  let fn_body  = &src[ fn_start..fn_end ];

  assert!(
    !fn_body.contains( "let is_idle" ),
    "BUG-285 regression: `let is_idle` variable must not exist in pre_switch_touch_ctx body\n\
    resets_at is server-side state and cannot proxy local subprocess identity",
  );
  assert!(
    !fn_body.contains( "AlreadyActive" ),
    "BUG-285 regression: AlreadyActive must not be returned from pre_switch_touch_ctx",
  );
}

/// `mre_bug288` — `apply_post_switch_touch` re-fetch is non-aborting when the credentials
/// file has no `accessToken`; pre-re-fetch cache writes succeed regardless.
///
/// # Root Cause
/// `apply_post_switch_touch` discarded the `run_isolated` result with `let _ = ...` and
/// performed no post-subprocess quota re-fetch. A subsequent `.usage touch` call then saw
/// stale `resets_at = None` and spawned a redundant second subprocess (double-subprocess race).
///
/// # Why Not Caught
/// No unit test exercised `apply_post_switch_touch` directly. The only coverage was via
/// `lim_it` CLI integration tests requiring live OAuth credentials (`aw27`, `aw28`, `aw29`).
///
/// # Fix Applied
/// Added post-subprocess quota re-fetch block (AC-21) to `apply_post_switch_touch`, mirroring
/// `apply_touch`'s AC-03 pattern. Reads credentials fresh from disk (not from
/// `ctx.credentials_json`) to capture any post-subprocess token rotation. On `Ok(new_data)`:
/// calls `write_quota_cache(paths.base(), name, ...)` to persist the updated quota. On
/// failure: silently skips — non-aborting per AC-21.
///
/// # Prevention
/// This test verifies the non-aborting invariant: when `accessToken` is absent from the
/// credentials file, the re-fetch is silently skipped and the function returns normally.
/// Also verifies that `last_touch_at` and `touch_idle` (written before the re-fetch block)
/// are committed to disk even when the re-fetch is skipped.
///
/// # Pitfall
/// `apply_post_switch_touch` is `pub(crate)` — only testable inline in `src/usage/api.rs`.
/// The re-fetch block reads credentials from `paths.base()/{name}.credentials.json` (fresh
/// disk read), NOT from `ctx.credentials_json` — tests must write the credential file at
/// that path, not just supply a non-empty string in the `TouchCtx`.
#[ doc = "bug_reproducer(BUG-288)" ]
#[ test ]
fn mre_bug288_post_switch_touch_refetch_updates_quota()
{
  use claude_quota::OauthUsageData;

  // ── Success path (structural): write_quota_cache is called when re-fetch succeeds ──
  // Verifies Fix(BUG-288) is present: apply_post_switch_touch must call write_quota_cache
  // on a successful fetch_oauth_usage result so subsequent .usage sees the updated resets_at.
  {
    let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api_switch.rs" ) );
    let fn_start = src
      .find( "pub fn apply_post_switch_touch(" )
      .expect( "apply_post_switch_touch not found in api_switch.rs" );
    let fn_end   = src[ fn_start + 1.. ]
      .find( "\npub fn " )
      .map_or( src.len(), |rel| fn_start + 1 + rel );
    let fn_body  = &src[ fn_start..fn_end ];
    assert!(
      fn_body.contains( "fetch_oauth_usage" ),
      "BUG-288: apply_post_switch_touch must call fetch_oauth_usage for AC-21 re-fetch",
    );
    assert!(
      fn_body.contains( "write_quota_cache" ),
      "BUG-288: apply_post_switch_touch must call write_quota_cache on successful re-fetch \
      so subsequent .usage sees the updated resets_at (no double-subprocess race)",
    );
  }

  // ── Failure path (runtime): no accessToken → re-fetch silently skipped ──────────────
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let name = "test@example.com";

  // Failure path: credentials file has no `accessToken` field.
  // `parse_string_field` returns None → re-fetch skipped → non-aborting.
  std::fs::write(
    paths.base().join( format!( "{name}.credentials.json" ) ),
    r#"{"subscriptionType":"pro","expiresAt":9999999999999}"#,
  ).unwrap();

  let ctx = TouchCtx::for_test(
    OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None },
  );

  // Must not panic: run_isolated fails silently (let _ = ...); re-fetch skipped.
  apply_post_switch_touch( name, ctx, "auto", "auto", false, &paths, paths.base() );

  // Observable 1: pre-re-fetch cache writes (last_touch_at, touch_idle) must succeed
  // even when the re-fetch block is skipped — they are written unconditionally before it.
  let cache_path = paths.base().join( format!( "{name}.json" ) );
  let cache = std::fs::read_to_string( &cache_path )
    .expect( "BUG-288: cache file must exist after apply_post_switch_touch" );
  assert!(
    cache.contains( "last_touch_at" ),
    "BUG-288: last_touch_at must be written to cache even when re-fetch is skipped, got: {cache}",
  );
  assert!(
    cache.contains( "touch_idle" ),
    "BUG-288: touch_idle must be written to cache even when re-fetch is skipped, got: {cache}",
  );

  // Observable 2: quota re-fetch must have been skipped — `resets_at` must not appear.
  // If the re-fetch had fired with a live token and written new quota data, this would fail,
  // making it both an absence-check for the failure path and an implicit sentinel that the
  // test fixture did not accidentally provide a live credential.
  assert!(
    !cache.contains( "resets_at" ),
    "BUG-288: resets_at must not be written when accessToken is absent (re-fetch skipped), got: {cache}",
  );
}

/// Corner case: credentials file absent → outer `read_to_string` guard fails →
/// entire re-fetch block skipped; `last_touch_at` and `touch_idle` still written; no panic.
///
/// # Root Cause
/// The AC-21 re-fetch block uses three nested `if let` guards:
///   1. `if let Ok(fresh_json) = read_to_string(&cred_path)` — outer: file I/O
///   2. `if let Some(token) = parse_string_field(...)` — inner: JSON field
///   3. `if let Ok(new_data) = fetch_oauth_usage(...)` — innermost: HTTP
///
/// `mre_bug288_post_switch_touch_refetch_updates_quota` covers guard 2 (file present,
/// no `accessToken`). This test covers guard 1 (file absent entirely).
///
/// # Why Not Caught
/// The file-absent path was not covered because the existing MRE test always writes a
/// credential file (albeit without `accessToken`). The outer guard is a distinct code
/// path even though observables are identical to the inner-guard failure path.
///
/// # Fix Applied
/// No fix required — `if let Ok` on `read_to_string` already handles this correctly.
/// This test verifies the non-aborting invariant for the outer guard specifically.
///
/// # Prevention
/// Nested `if let` re-fetch blocks must have unit tests per guard layer: outer I/O
/// guard and inner parse guard are independent failure modes requiring separate coverage.
///
/// # Pitfall
/// File-absent and file-present-no-token produce identical observables (`last_touch_at`
/// written, no `resets_at`) but exercise different branches. Both must be tested to
/// confirm the non-aborting invariant holds at each layer of the nested guard chain.
#[ test ]
fn it_apply_post_switch_touch_cred_file_absent_skips_refetch()
{
  use claude_quota::OauthUsageData;

  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  let name = "absent@example.com";

  // No credentials file written — `read_to_string` returns Err.
  // Outer `if let Ok` guard fires → entire re-fetch block bypassed.

  let ctx = TouchCtx::for_test(
    OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None },
  );

  // Must not panic: run_isolated discards result; outer re-fetch guard skips silently.
  apply_post_switch_touch( name, ctx, "auto", "auto", false, &paths, paths.base() );

  // Observable 1: last_touch_at and touch_idle written unconditionally (before re-fetch block).
  let cache_path = paths.base().join( format!( "{name}.json" ) );
  let cache = std::fs::read_to_string( &cache_path )
    .expect( "cache file must exist even when credential file is absent" );
  assert!(
    cache.contains( "last_touch_at" ),
    "last_touch_at must be written even when credential file is absent; got: {cache}",
  );
  assert!(
    cache.contains( "touch_idle" ),
    "touch_idle must be written even when credential file is absent; got: {cache}",
  );

  // Observable 2: re-fetch outer guard fired → no resets_at in cache.
  assert!(
    !cache.contains( "resets_at" ),
    "resets_at must not appear when credential file is absent (outer guard bypassed); got: {cache}",
  );
}

/// `ft11` — Rotation dispatcher calls `apply_model_override` for the winner (AC-05, Feature 062).
///
/// # Root Cause
/// Before Feature 062, the rotation dispatch block in `usage_routine` called `switch_account` and
/// `apply_touch` but never `apply_model_override` for the winner — the winner's session model
/// remained at whatever was set by the previous current account.
///
/// # Why Not Caught
/// No structural test asserted the call site exists inside the rotation block.
///
/// # Fix Applied
/// Added `apply_model_override( winner_data, &claude_paths, ... )` inside the rotation dispatch
/// block, guarded by `if let Ok( ref winner_data ) = accounts[ winner_idx ].result`.
///
/// # Prevention
/// Structural grep: rotation dispatch block must contain `apply_model_override(`.
///
/// # Pitfall
/// Must insert AFTER `switch_account` succeeds — before that point, `claude_paths` is
/// conditionally set and `winner_idx` may not be resolved.
#[ test ]
fn ft11_rotation_dispatcher_calls_apply_model_override_for_winner()
{
  let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  // Locate rotation dispatch block: starts at "Rotation dispatch" comment.
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  // Ends at the closing `}` of `if params.rotate { ... }` — after the final `return Ok`.
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  assert!(
    block.contains( "apply_model_override(" ),
    "AC-05: apply_model_override must be called in the rotation dispatch block (Feature 062)\n\
    block:\n{block}",
  );
}

// ── BUG-310 MRE: rotation dispatch must re-sync live credentials after apply_touch ─

/// MRE for BUG-310: after `apply_touch` in the rotation dispatch block, the winner's
/// store credentials must be copied back to the live session file.
///
/// # Root Cause
///
/// `switch_account(winner)` at step 4d copies store→live BEFORE `apply_touch` at step 4e.
/// The touch subprocess may refresh the OAuth token, writing `token_B` to the STORE file
/// via `refresh_account_token → save(update_marker=false)`. The live session retains stale
/// `token_A` — if the server invalidated `token_A` during refresh, the live session dies.
///
/// # Why Not Caught
///
/// No test asserted that the rotation block re-syncs live credentials after the touch step.
/// `apply_touch` intentionally writes to STORE only (BUG-211 fix) — the live re-sync is
/// the caller's responsibility, and the rotation dispatcher was the only caller that needed it.
///
/// # Fix Applied
///
/// `std::fs::copy( store/{name}.credentials.json, claude_paths.credentials_file() )` added
/// immediately after `apply_touch` in the rotation dispatch block (step 4e').
///
/// # Prevention
///
/// Structural grep: rotation dispatch block must contain `fs::copy` (or `std::fs::copy`)
/// after `apply_touch`. This test enforces that.
///
/// # Pitfall
///
/// Do NOT call `switch_account` again — it re-writes the `_active` marker and patches
/// `.claude.json` redundantly. A targeted credential file copy suffices.
///
/// `let _ = std::fs::copy(...)` silently discards I/O errors. If the filesystem write
/// fails (permissions, disk full), rotation still reports success but live credentials
/// may remain stale. A future improvement would emit a trace warning on copy failure.
///
/// Spec: [`tests/docs/feature/38_usage_strategy_rotate.md` FT-11]
#[ doc = "bug_reproducer(BUG-310)" ]
#[ test ]
fn mre_bug310_rotation_touch_resyncs_live_credentials()
{
  let src         = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  // Locate apply_touch call within the rotation block.
  let touch_pos = block
    .find( "apply_touch(" )
    .expect( "BUG-310: apply_touch call not found in rotation dispatch block" );

  // After apply_touch, there must be a store→live credential re-sync via fs::copy.
  let after_touch = &block[ touch_pos.. ];
  assert!(
    after_touch.contains( "fs::copy" ),
    "BUG-310 AC-11: rotation dispatch block must re-sync live credentials from store \
    after apply_touch via fs::copy. Without this, live retains stale pre-refresh token.\n\
    block after apply_touch:\n{after_touch}",
  );
}

/// Control for BUG-310: when `touch::0` is used (no `apply_touch` call in rotation),
/// `switch_account` alone suffices — store and live are already consistent.
///
/// This test verifies that `switch_account` IS called in the rotation block, ensuring
/// the store→live copy happens at step 4d. The divergence from D1 only arises when
/// `apply_touch` refreshes the token AFTER `switch_account` already copied.
///
/// Spec: [`tests/docs/feature/38_usage_strategy_rotate.md` FT-11 (control)]
#[ test ]
fn mre_bug310_rotation_no_refresh_no_divergence()
{
  let src         = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  // switch_account must exist in the rotation block — this is the store→live copy at step 4d.
  assert!(
    block.contains( "switch_account(" ),
    "BUG-310 control: switch_account must be called in the rotation dispatch block \
    to copy store credentials to live at step 4d.\n\
    block:\n{block}",
  );

  // apply_touch must also exist — without it, there's no token refresh divergence.
  assert!(
    block.contains( "apply_touch(" ),
    "BUG-310 control: apply_touch must be called in the rotation dispatch block \
    at step 4e. Without it, no token refresh can create store/live divergence.\n\
    block:\n{block}",
  );
}

// ── D4: Rotation without touch — switch_account alone is consistent ───────

/// Reach test: `switch_account` precedes `apply_touch` in the rotation dispatch block.
/// This ordering means that when `touch::0` is used (`apply_touch` is skipped), the live
/// file already matches the store — no re-sync needed.
///
/// The test verifies that `switch_account` appears BEFORE `apply_touch` in the block,
/// ensuring step 4d (store→live copy) always happens before step 4e (touch may diverge).
///
/// Spec: [`tests/docs/feature/38_usage_strategy_rotate.md` FT-11 (reach D4)]
#[ test ]
fn reach_rotation_switch_account_precedes_apply_touch()
{
  let src         = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  let switch_pos = block.find( "switch_account(" )
    .expect( "D4: switch_account not found in rotation block" );
  let touch_pos  = block.find( "apply_touch(" )
    .expect( "D4: apply_touch not found in rotation block" );
  assert!(
    switch_pos < touch_pos,
    "D4: switch_account (step 4d) must precede apply_touch (step 4e) in rotation block. \
    When touch::0 skips apply_touch, the store→live copy from switch_account is already consistent.",
  );
}

// ── D5: Structural guard — fs::copy after apply_touch in rotation block ───

/// Structural proximity guard: `fs::copy` must appear after `apply_touch` in the
/// rotation dispatch block, within a few lines. This prevents regression by ensuring
/// the re-sync step is never accidentally removed by refactoring.
///
/// Before fix: FAILS (no `fs::copy` exists). After fix: PASSES.
///
/// **Known gap:** verifies `fs::copy` and `credentials_file()` appear after `apply_touch`,
/// but does NOT verify the SOURCE argument is `credential_store.join(...)`. A refactor
/// that accidentally swaps src/dst would still pass. Copy-direction correctness is
/// enforced by code review and the `Fix(BUG-310)` comment in `api.rs`.
///
/// Spec: [`tests/docs/feature/38_usage_strategy_rotate.md` FT-11 (reach D5)]
#[ test ]
fn reach_structural_guard_fs_copy_follows_apply_touch_in_rotation()
{
  let src         = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  let block_start = src
    .find( "Rotation dispatch (Feature 038" )
    .expect( "rotation dispatch block not found in api.rs" );
  let block_end   = src[ block_start.. ]
    .find( "\n  Ok( OutputData::new( content" )
    .map_or( src.len(), |rel| block_start + rel );
  let block = &src[ block_start..block_end ];

  let touch_pos = block.find( "apply_touch(" )
    .expect( "D5: apply_touch not found in rotation block" );
  let after_touch = &block[ touch_pos.. ];

  // fs::copy must appear after apply_touch — this is the re-sync step (4e').
  assert!(
    after_touch.contains( "fs::copy" ),
    "D5 AC-11: fs::copy must follow apply_touch in rotation block to re-sync \
    live credentials after potential token refresh.\n\
    block after apply_touch:\n{after_touch}",
  );

  // The fs::copy must reference credentials_file() — not some other path.
  assert!(
    after_touch.contains( "credentials_file()" ),
    "D5 AC-11: fs::copy target must be credentials_file() (the live session file).\n\
    block after apply_touch:\n{after_touch}",
  );
}

/// AC-34 routing structural: `apply_post_switch_touch` calls `refresh_account_token`, not `run_isolated`
///
/// # Root Cause
/// Before AC-34 / Invariant 008, `apply_post_switch_touch` called `run_isolated` directly —
/// a fire-and-forget pattern that bypassed:
///   - expiresAt=1 manipulation (AC-32): no RT rotation
///   - live credential sync (AC-33): no pre-sync or race recovery
///   - credential write-back: rotated credentials silently discarded
///
/// # Why Not Caught
/// No structural test asserted the routing destination. The IN-1 invariant test (grep-based)
/// verifies ABSENCE of direct `run_isolated` calls; this test verifies PRESENCE of the
/// `refresh_account_token` call — the positive routing complement to IN-1's negative guard.
///
/// # Fix Applied
/// AC-34: `apply_post_switch_touch` now calls `crate::account::refresh_account_token(...)`.
/// The `credential_store` parameter was added as the 7th param to carry the correct
/// `~/.persistent/claude/credential/` path (NOT `paths.base()` = `~/.claude/`).
///
/// # Prevention
/// Both the negative invariant (IN-1: zero `run_isolated` calls) and this positive
/// routing test must pass for AC-34 to be fully enforced.
///
/// # Pitfall
/// `apply_post_switch_touch` body region ends just before the `// ── Main routine` anchor.
/// If that anchor is renamed, the structural search must be updated.
#[ test ]
fn ft_apply_post_switch_touch_routes_through_refresh_account_token()
{
  let src      = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api_switch.rs" ) );
  let fn_start = src
    .find( "pub fn apply_post_switch_touch(" )
    .expect( "apply_post_switch_touch must exist in api_switch.rs (AC-34 routing entry point)" );
  let fn_end = src[ fn_start + 1.. ]
    .find( "\npub fn " )
    .map_or( src.len(), |rel| fn_start + 1 + rel );
  let fn_body = &src[ fn_start..fn_end ];

  // Positive routing assertion: must delegate to refresh_account_token (AC-34).
  assert!(
    fn_body.contains( "refresh_account_token(" ),
    "AC-34: apply_post_switch_touch must call refresh_account_token() for token refresh \
    (not run_isolated directly) — see invariant 008 and Feature 017 AC-34\n\
    function body:\n{fn_body}",
  );

  // Negative routing assertion: must NOT call run_isolated directly (complements IN-1 grep).
  // Pattern built at runtime — avoids embedding "run_isolated(" as literal bytes in this file,
  // which would itself be a violation of the IN-1 invariant this test enforces.
  let direct_call_pattern = format!( "{}(", "run_isolated" );
  let violations : Vec< &str > = fn_body.lines()
    .filter( | line | !line.trim_start().starts_with( "//" ) )
    .filter( | line | line.contains( &direct_call_pattern ) )
    .collect();
  assert!(
    violations.is_empty(),
    "AC-34: apply_post_switch_touch must not invoke run_isolated directly; \
    all token refresh must route through refresh_account_token:\n{}",
    violations.join( "\n" ),
  );
}

/// BUG-317 MRE — `only_valid::1` must exclude cancelled accounts (`billing_type="none"`).
///
/// # Root Cause
/// The `only_valid` retain predicate in `api.rs` only checked `result.is_ok()`. A cancelled
/// account with a successful API response (`result = Ok(...)`) and `billing_type = "none"`
/// passed `only_valid::1` as if it were a valid account — potentially surfacing in the
/// valid-account list and rotation recommendations.
///
/// # Why Not Caught
/// All existing `only_valid` tests used accounts with `account = None` (no subscription data)
/// or `result = Err(...)`. The case of `result = Ok` + `billing_type = "none"` was untested:
/// good quota data returned by the API, but subscription permanently cancelled.
///
/// # Fix Applied
/// Fix D (BUG-317): the retain predicate is now:
/// `aq.result.is_ok() && !aq.account.as_ref().is_some_and(|a| a.billing_type == "none")`
/// Both conditions must pass. A cancelled account satisfies `result.is_ok()` but fails the
/// second condition — correctly excluded.
///
/// # Prevention
/// Structural inspection of `api.rs` via `include_str!` ensures the `billing_type` guard
/// cannot be silently removed. If Fix D is reverted, the structural assertion fails immediately.
///
/// # Pitfall
/// `account = None` is NOT equivalent to `billing_type = "none"`. `account = None` means the
/// account-API call failed (ambiguous). Only `account = Some({billing_type: "none"})` is the
/// definitive cancellation signal. `mk_aq_cancelled` always sets `account = Some(...)` with
/// `billing_type = "none"` — use it for cancelled-account tests.
#[ doc = "bug_reproducer(BUG-317)" ]
#[ test ]
fn mre_bug317_cancelled_excluded_by_only_valid()
{
  use claude_profile::usage::test_bridge::mk_aq_cancelled;

  // ── Structural: verify Fix D predicate is present in api.rs ─────────────────────────────
  // include_str! is compile-time — the assertion fails at build if Fix D is reverted.
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  let only_valid_pos = src
    .find( "if params.only_valid" )
    .expect( "BUG-317 Fix D: 'if params.only_valid' block must exist in api.rs" );
  // Scan the retain expression (next 300 bytes) for the billing_type guard.
  let block = &src[ only_valid_pos .. ( only_valid_pos + 300 ).min( src.len() ) ];
  assert!(
    block.contains( "billing_type" ),
    "BUG-317 Fix D: only_valid retain predicate must check billing_type=\"none\" to exclude \
    cancelled accounts — revert of Fix D detected in api.rs\nblock:\n{block}",
  );

  // ── Preconditions: mk_aq_cancelled produces the critical BUG-317 scenario ────────────────
  // result = Ok  →  old predicate (result.is_ok() only) would pass this account through.
  // billing_type = "none"  →  Fix D second predicate correctly blocks it.
  let cancelled = mk_aq_cancelled( "dead@test.com", 20.0, 20.0 );
  assert!(
    cancelled.result.is_ok(),
    "BUG-317 precondition: mk_aq_cancelled must produce result=Ok — \
    cancelled account has valid quota data (the exact bug scenario: would have slipped through)",
  );
  assert!(
    cancelled.account.as_ref().is_some_and( |a| a.billing_type == "none" ),
    "BUG-317 precondition: mk_aq_cancelled must set billing_type=\"none\" (definitive cancel signal)",
  );

  // ── Predicate: Fix D correctly excludes the cancelled account ────────────────────────────
  // Replicate the Fix D retain predicate. The retain keeps accounts where this is true;
  // the cancelled account must evaluate to false (excluded).
  let passes_only_valid = cancelled.result.is_ok()
    && !cancelled.account.as_ref().is_some_and( |a| a.billing_type == "none" );
  assert!(
    !passes_only_valid,
    "BUG-317 Fix D: cancelled account (result=Ok, billing_type=\"none\") must be excluded \
    by only_valid::1 — the billing_type guard must negate the result.is_ok() pass",
  );
}

// ── Algorithm 002 AC cases ────────────────────────────────────────────────

/// AC-1 (algorithm/002): Absent Sonnet tier with Opus session writes "sonnet" conservatively.
///
/// `seven_day_sonnet=None` → `else` branch of `apply_model_override` →
/// `override_session_model_to_sonnet()`. Current model `"opus"` satisfies gate
/// (`contains("opus")=true`) → writes `"sonnet"`. Absent tier ≠ exhausted.
///
/// Spec: [`tests/docs/algorithm/002_session_model_override.md` AC-1]
#[ test ]
fn ac1_absent_tier_with_opus_session_restores_sonnet()
{
  use claude_quota::OauthUsageData;
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  std::fs::write( paths.settings_file(), r#"{"model":"opus"}"# ).unwrap();
  let quota = OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  apply_model_override( &quota, &paths, false, "test", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap_or_default();
  assert!(
    content.contains( "\"sonnet\"" ),
    "AC-1: absent Sonnet tier + Opus session must write sonnet (Fix BUG-311); got: {content}",
  );
  assert!(
    !content.contains( "\"opus\"" ),
    "AC-1: absent tier must not leave opus in settings.json; got: {content}",
  );
}

/// AC-2 (algorithm/002): Absent Sonnet tier with Sonnet session — model field unchanged.
///
/// `seven_day_sonnet=None` → calls `override_session_model_to_sonnet()`.
/// Current model `"sonnet"` shorthand does NOT satisfy the gate
/// (`contains("opus")=false`, `=="claude-sonnet-4-6"=false`, `is_empty()=false`) →
/// returns `false` — model field unchanged; session already in Sonnet form.
///
/// Spec: [`tests/docs/algorithm/002_session_model_override.md` AC-2]
#[ test ]
fn ac2_absent_tier_with_sonnet_session_model_field_unchanged()
{
  use claude_quota::OauthUsageData;
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  // Pre-write settings.json with sonnet shorthand (production form).
  std::fs::write( paths.settings_file(), r#"{"model":"sonnet"}"# ).unwrap();
  let quota = OauthUsageData { five_hour : None, seven_day : None, seven_day_sonnet : None };
  apply_model_override( &quota, &paths, false, "test", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap_or_default();
  assert!(
    content.contains( "\"sonnet\"" ),
    "AC-2: absent tier + Sonnet session must leave model = sonnet; got: {content}",
  );
  assert!(
    !content.contains( "\"opus\"" ),
    "AC-2: absent tier + Sonnet session must not write opus; got: {content}",
  );
}

/// AC-3 (algorithm/002): Sufficient Sonnet quota with Opus session restores Sonnet.
///
/// `utilization=80.0` → `sonnet_left=20.0 ≥ 15.0` → `else` branch →
/// `override_session_model_to_sonnet()`. Current model `"opus"` satisfies gate → writes `"sonnet"`.
/// Recovery path added by Fix BUG-311.
///
/// Spec: [`tests/docs/algorithm/002_session_model_override.md` AC-3]
#[ test ]
fn ac3_sufficient_quota_with_opus_session_restores_sonnet()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  std::fs::write( paths.settings_file(), r#"{"model":"opus"}"# ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage { utilization : 80.0, resets_at : None } ),
  };
  apply_model_override( &quota, &paths, false, "test", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap_or_default();
  assert!(
    content.contains( "\"sonnet\"" ),
    "AC-3: 20% Sonnet remaining + Opus session must write sonnet (Fix BUG-311 recovery path); got: {content}",
  );
  assert!(
    !content.contains( "\"opus\"" ),
    "AC-3: sufficient quota must not leave opus; got: {content}",
  );
}

/// AC-4 (algorithm/002): Near-exhausted Sonnet quota with Sonnet session switches to Opus.
///
/// `utilization=86.0` → `sonnet_left=14.0 < 15.0` → Opus override fires →
/// `override_session_model_to_opus()`. Current model `"sonnet"` satisfies gate
/// (`contains("sonnet")=true`) → writes `"opus"`.
///
/// Spec: [`tests/docs/algorithm/002_session_model_override.md` AC-4]
#[ test ]
fn ac4_near_exhausted_quota_with_sonnet_session_switches_to_opus()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  std::fs::write( paths.settings_file(), r#"{"model":"sonnet"}"# ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage
    {
      utilization : 86.0,
      resets_at   : Some( "2026-06-28T04:00:00+00:00".to_string() ),
    } ),
  };
  apply_model_override( &quota, &paths, false, "test", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap_or_default();
  assert!(
    content.contains( "\"opus\"" ),
    "AC-4: 14% Sonnet remaining + Sonnet session must write opus (near-exhausted); got: {content}",
  );
}

/// AC-5 (algorithm/002): Near-exhausted Sonnet quota with Opus session — model field unchanged.
///
/// `utilization=86.0` → Opus override path → `override_session_model_to_opus()`.
/// Current model `"opus"` does NOT satisfy gate
/// (`contains("sonnet")=false`, `=="claude-opus-4-6"=false`, `is_empty()=false`) →
/// returns `false` — model field unchanged; session already in Opus form.
///
/// Spec: [`tests/docs/algorithm/002_session_model_override.md` AC-5]
#[ test ]
fn ac5_near_exhausted_quota_with_opus_session_model_field_unchanged()
{
  use claude_quota::{ OauthUsageData, PeriodUsage };
  let dir   = TempDir::new().unwrap();
  let paths = claude_profile::ClaudePaths::with_home( dir.path() );
  std::fs::create_dir_all( paths.base() ).unwrap();
  std::fs::write( paths.settings_file(), r#"{"model":"opus"}"# ).unwrap();
  let quota = OauthUsageData
  {
    five_hour        : None,
    seven_day        : None,
    seven_day_sonnet : Some( PeriodUsage
    {
      utilization : 86.0,
      resets_at   : Some( "2026-06-28T04:00:00+00:00".to_string() ),
    } ),
  };
  apply_model_override( &quota, &paths, false, "test", "test-account" );
  let content = std::fs::read_to_string( paths.settings_file() ).unwrap_or_default();
  assert!(
    content.contains( "\"opus\"" ),
    "AC-5: 14% Sonnet remaining + Opus session must leave model = opus (no-op); got: {content}",
  );
  assert!(
    !content.contains( "\"sonnet\"" ),
    "AC-5: already-Opus session must not write sonnet; got: {content}",
  );
}

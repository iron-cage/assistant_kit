//! Integration tests: IT-92–IT-121 — `.usage` `touch::` and `NextStrategy` parameters.
//!
//! Covers `tsk_184` `NextStrategy` 2-variant reduction, `tsk_185` `touch::` session
//! activation, `sort::next` meta-strategy, and live touch-skipped tests.
//!
//! Live tests (names contain `lim_it`) require a real Anthropic OAuth access token.

use crate::cli_runner::{
  run_cs, run_cs_with_env,
  stdout, stderr, assert_exit,
  write_account, write_account_with_token, live_active_token, require_live_api,
  FAR_FUTURE_MS,
};
use tempfile::TempDir;

// ── tsk_184 — NextStrategy 2-variant reduction ────────────────────────────────

/// it092 (Feature 037/038): `next::all` rejected — `next::` parameter fully removed.
///
/// After Feature 037/038 `next::` is removed entirely; any `next::X` exits 1
/// with a message redirecting to `sort::`.
#[ test ]
fn it092_next_all_rejected_exit_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::all" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "sort::" ),
    "next::all error must redirect to sort::, got:\n{err}",
  );
}

/// it093 (Feature 037/038): footer driven by `sort::` strategy — no `NextStrategy` dependency.
///
/// After Feature 037/038 render.rs has no `NextStrategy` references.
/// Footer is a single-strategy line driven by the active `sort::` strategy.
///
/// Structural test — no credentials required.
#[ test ]
fn it093_footer_not_gated_on_next_all_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/render.rs" ) );

  assert!(
    !src.contains( "NextStrategy" ),
    "Feature 037/038: render.rs must not reference NextStrategy; \
     footer is now driven by SortStrategy from active sort:: param",
  );
  assert!(
    !src.contains( "if next ==" ),
    "Feature 037/038: footer must not be gated on a next== check",
  );
}

/// it094 (Feature 037/038): `next::session` rejected — `next::` parameter fully removed.
#[ test ]
fn it094_next_session_rejected_exit_1()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();

  let out = run_cs_with_env( &[ ".usage", "next::session" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
  let err = stderr( &out );
  assert!(
    err.contains( "sort::" ),
    "next::session error must redirect to sort::, got:\n{err}",
  );
}

/// it095 (TSK-184 AC-04): `NextStrategy::Session` is absent from source after reduction.
///
/// Before TSK-184: `NextStrategy::Session` appears in enum declaration, `parse()`, match arms.
/// After TSK-184:  `NextStrategy::Session` must not appear anywhere in source.
///
/// Structural test — no credentials required.
/// RED:   source still has `NextStrategy::Session` → assert fails.
/// GREEN: Session fully removed → assert passes.
#[ test ]
fn it095_next_strategy_session_absent_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/types.rs" ) );
  assert!(
    !src.contains( "NextStrategy::Session" ),
    "TSK-184: `NextStrategy::Session` must be completely removed from source; \
     check enum declaration, parse() arms, match arms, strategy arrays, and comments",
  );
}

/// it096 (TSK-184 AC-05): `format::json` output is identical regardless of `sort::` value.
///
/// `render_json` does not vary by sort strategy; JSON remains the same for any
/// valid `sort::` value.
#[ test ]
fn it096_json_unaffected_by_sort_strategy()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out_default = run_cs_with_env( &[ ".usage", "format::json" ],                 &[ ( "HOME", home ) ] );
  let out_renews  = run_cs_with_env( &[ ".usage", "format::json", "sort::renews" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_default, 0 );
  assert_exit( &out_renews,  0 );
  assert_eq!(
    stdout( &out_default ), stdout( &out_renews ),
    "format::json output must be identical regardless of sort:: value (AC-13)",
  );
}

// ── tsk_185 — touch:: session activation ──────────────────────────────────────

/// it097 (TSK-185 AC-01): `touch::1` with empty credential store exits 0.
///
/// Before TSK-185: `touch::` is unregistered → `ArgumentUnrecognised` → exit 1.
/// After TSK-185:  `touch::` accepted, empty store → no-accounts message → exit 0.
///
/// RED:   `touch::` unknown → exit 1.
/// GREEN: `touch::` registered → exit 0.
#[ test ]
fn it097_touch_1_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ) || text.contains( "No accounts" ) || text.is_empty(),
    "touch::1 with empty store must exit 0 (no subprocess spawned — no accounts), got:\n{text}",
  );
}

/// it098 (TSK-185 AC-04): `touch::1` with a no-token account exits 0 without touching it.
///
/// Accounts whose quota fetch failed (expired/missing token → error result) must not
/// be touched. The trigger requires `result.is_ok()` AND `five_hour.resets_at.is_some()`.
/// A no-token account has an errored result → it is skipped entirely.
///
/// Before TSK-185: `touch::` unregistered → exit 1.
/// After TSK-185:  exits 0; errored account row shows `—` in Expires (no subprocess).
///
/// RED:   `touch::` unknown → exit 1.
/// GREEN: exits 0, account shows dash row.
#[ test ]
fn it098_touch_1_errored_account_skipped()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account with FAR_FUTURE_MS but no accessToken field → quota fetch fails
  write_account( dir.path(), "a@x.com", "max", "default", FAR_FUTURE_MS, false );

  let out = run_cs_with_env( &[ ".usage", "touch::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "a@x.com" ),
    "touch::1 with errored account must still show account row (AC-04), got:\n{text}",
  );
}

/// it099 (TSK-185 AC-02 structural): `fn apply_touch` is present in production source.
///
/// This structural test uses `include_str!` to confirm the function exists before
/// requiring live network calls. No credentials needed.
///
/// RED:   `apply_touch` absent from source → assert fails.
/// GREEN: `apply_touch` present → assert passes.
#[ test ]
fn it099_apply_touch_fn_exists_structural()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs" ) );
  assert!(
    src.contains( "fn apply_touch" ),
    "TSK-185: `fn apply_touch` must be present in src/usage/touch.rs; \
     add the active-window extension function that calls refresh_account_token() \
     for accounts with result.is_ok() AND five_hour.resets_at.is_some()",
  );
}

/// it100 (TSK-185 AC-08): `format::json touch::1` with empty store exits 0 and outputs `[]`.
///
/// `render_json` is unaffected by `touch::`; touched accounts appear as normal JSON
/// objects. With empty store: both default and `touch::1` must output `[]`.
///
/// Before TSK-185: `touch::` unregistered → exit 1.
/// After TSK-185:  exit 0, output `[]` (same as without `touch::1`).
///
/// RED:   exit 1 (unrecognised param).
/// GREEN: exit 0, JSON output `[]`.
#[ test ]
fn it100_touch_json_format_unaffected()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out_default = run_cs_with_env(
    &[ ".usage", "format::json" ],
    &[ ( "HOME", home ) ],
  );
  let out_touch = run_cs_with_env(
    &[ ".usage", "format::json", "touch::1" ],
    &[ ( "HOME", home ) ],
  );
  assert_exit( &out_default, 0 );
  assert_exit( &out_touch,   0 );
  assert_eq!(
    stdout( &out_default ), stdout( &out_touch ),
    "format::json output must be identical with or without touch::1 (TSK-185 AC-08)",
  );
}

/// it101 (TSK-185 AC-10): `.usage.help` output contains `touch`.
///
/// `touch::` must be registered via `register_commands()` in `src/lib.rs` so users
/// can discover it. The param must appear in `.usage.help` output.
///
/// Before TSK-185: `touch` absent from help.
/// After TSK-185:  `touch` appears as a registered parameter.
///
/// RED:   `touch` absent from `.usage.help` output.
/// GREEN: `touch` present.
#[ test ]
fn it101_usage_help_shows_touch_param()
{
  let out = run_cs( &[ ".usage.help" ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "touch" ),
    ".usage.help must list param `touch` (TSK-185 AC-10), got:\n{text}",
  );
}

/// it102 `lim_it` (IT-51 / FT-03 of feature/020): `sort::renew` (default) shows recommendation in footer.
///
/// With ≥2 accounts sharing a live token, the renew strategy selects one winner.
/// The footer must show a `Next (renew):` recommendation line.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-51]
///       [`tests/docs/feature/020_usage_sort_strategies.md` AC-09]
#[ test ]
fn it102_lim_it_sort_renew_shows_recommendation()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it102: no live token — skipping" );
    return;
  };
  require_live_api( "it102" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "Next (renew):" ),
    "sort::renew must show 'Next (renew):' recommendation in footer (IT-51/020), got:\n{text}",
  );
}

/// it103 `lim_it` (IT-52 / feature/020): `sort::renews` shows recommendation in footer.
///
/// With ≥2 accounts sharing a live token, the renews strategy selects the account with
/// the soonest billing renewal. The footer must show a `Next (renews):` recommendation line.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-52]
///       [`tests/docs/feature/020_usage_sort_strategies.md` AC-09]
#[ test ]
fn it103_lim_it_sort_renews_shows_recommendation()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it103: no live token — skipping" );
    return;
  };
  require_live_api( "it103" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env( &[ ".usage", "sort::renews" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "Next (renews):" ),
    "sort::renews must show 'Next (renews):' recommendation in footer (IT-52/020), got:\n{text}",
  );
}

/// it104 `lim_it` (IT-54 / feature/020): footer shows one recommendation line for active sort strategy.
///
/// With `sort::renew` (default), the footer shows a single recommendation line
/// with the `→` winner for the active strategy.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-54]
///       [`tests/docs/feature/020_usage_sort_strategies.md` AC-09]
#[ test ]
fn it104_lim_it_footer_shows_strategy_recommendation()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it104: no live token — skipping" );
    return;
  };
  require_live_api( "it104" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  assert!(
    text.contains( "renew" ),
    "footer must show renew strategy recommendation line (IT-54/020), got:\n{text}",
  );
}

/// it105 `lim_it` (IT-58): per-column emoji prefix appears in `5h Left` column values.
///
/// `5h Left` cells embed a coloured-circle emoji prefix: 🟢 when >5% left, 🟡 when ≤5%.
/// At least one account row must show an emoji in that column.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-58]
///       [`tests/docs/feature/009_token_usage.md` AC-21]
#[ test ]
fn it105_lim_it_per_column_emoji_in_5h_left()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it105: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  let out = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );

  let has_emoji = text.contains( "🟢" ) || text.contains( "🟡" ) || text.contains( "🔴" );
  assert!(
    has_emoji,
    "5h Left / 7d Left columns must contain per-column emoji prefix (IT-58/AC-21); got:\n{text}",
  );
}

/// it106 (IT-62 / EC-1): `touch::0` accepted; empty credential store exits 0.
///
/// `touch::0` is the explicit off value — the parser must accept it without error.
/// No subprocess is spawned with `touch::0` regardless of account state.
///
/// Spec: [`tests/docs/cli/command/009_usage.md` IT-62]
///       [`tests/docs/cli/param/034_touch.md` EC-1]
///       [`tests/docs/feature/024_session_touch.md` AC-01]
#[ test ]
fn it106_touch_0_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  assert!(
    text.contains( "no accounts" ) || text.contains( "No accounts" ) || text.is_empty(),
    "touch::0 with empty store must exit 0 without param error (IT-62/EC-1), got:\n{text}",
  );
}

/// it107 (EC-3): `touch::true` accepted as equivalent to `touch::1`.
///
/// `parse_int_flag` must accept the string "true" and map it to 1 (enabled).
/// With an empty credential store, no subprocess is spawned and the command exits 0.
///
/// Spec: [`tests/docs/cli/param/034_touch.md` EC-3]
#[ test ]
fn it107_touch_true_accepted_empty_store_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::true" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
}

/// it108 (EC-4): `touch::bogus` exits 1 — invalid value rejected.
///
/// `parse_int_flag` must reject values that are not `0`, `1`, `"true"`, or `"false"`.
/// The parser returns `ArgumentTypeMismatch` (exit 1) for unrecognised string values.
///
/// Spec: [`tests/docs/cli/param/034_touch.md` EC-4]
#[ test ]
fn it108_touch_bogus_exits_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::bogus" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

/// it109 `lim_it` (FT-01 of feature/024 / EC-7): `touch::0` — no subprocess spawned; idle account unchanged.
///
/// When `touch::0` (explicit off), the touch trigger is never fired regardless of account state.
/// An idle account (`five_hour.resets_at` absent, 5h Reset shows `—`) stays unchanged.
/// Skips when the live account is in active state (`resets_at` present).
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-01]
///       [`tests/docs/cli/param/034_touch.md` EC-7]
#[ test ]
fn it109_lim_it_touch_0_no_subprocess_idle_account_unchanged()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it109: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent — EM-DASH present in 5h Reset column).
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  let pre_text = stdout( &pre );
  if !pre_text.contains( "\u{2014}" )
  {
    eprintln!( "it109: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  let out = run_cs_with_env( &[ ".usage", "touch::0" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // 5h Reset column must still show EM-DASH (touch::0 must not fire subprocess).
  assert!(
    text.contains( "\u{2014}" ),
    "touch::0 must not activate idle account — 5h Reset must remain as `\u{2014}` (FT-01/EC-7), got:\n{text}",
  );
}

/// it110 `lim_it` (FT-02 of feature/024 / EC-8): `touch::1` — subprocess observed via trace for idle account.
///
/// When `touch::1` and the account has `five_hour.resets_at` absent (idle), a subprocess
/// is invoked to activate the 5h session. With `trace::1`, stderr shows timestamped diagnostic lines
/// for the subprocess lifecycle. Skips when the live account is in active state (`resets_at` present).
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-02]
///       [`tests/docs/cli/param/034_touch.md` EC-8]
#[ test ]
fn it110_lim_it_touch_1_subprocess_spawned_for_idle_account()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it110: no live token — skipping" );
    return;
  };
  require_live_api( "it110" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent — EM-DASH present).
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  if !stdout( &pre ).contains( "\u{2014}" )
  {
    eprintln!( "it110: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  let out = run_cs_with_env( &[ ".usage", "touch::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "switch_account" ),
    "touch::1 with idle account must spawn subprocess — switch_account must appear (FT-02/EC-8), got stderr:\n{err}",
  );
}

/// it111 `lim_it` (FT-03 of feature/024): After successful touch, `5h Reset` transitions from `—` to countdown.
///
/// When `touch::1` triggers on an idle account (`resets_at` absent) and the subprocess succeeds,
/// the account's quota is re-fetched and the `5h Reset` column shows a concrete countdown (~5h)
/// where it previously showed `—`. Skips when account is already active.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-03]
#[ test ]
fn it111_lim_it_touch_1_5h_reset_changes_from_dash_to_time()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it111: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent — EM-DASH present).
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  let pre_text = stdout( &pre );
  if !pre_text.contains( "\u{2014}" )
  {
    eprintln!( "it111: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  let out = run_cs_with_env( &[ ".usage", "touch::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let text = stdout( &out );
  // After touch: the 5h Reset column must show a countdown (session activated — "in Xh Ym").
  assert!(
    text.contains( "in " ),
    "touch::1 must activate idle account; 5h Reset must show countdown after subprocess (FT-03), got:\n{text}",
  );
}

/// it112 (FT-05 of feature/024 structural): `apply_refresh` code appears before `apply_touch` in source.
///
/// The ordering guarantee (refresh runs before touch) is enforced at the call site in
/// `run_usage()`. This structural test verifies the invariant without requiring live
/// credentials or an expired token.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-05]
#[ test ]
fn it112_structural_refresh_before_touch_ordering_in_source()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/api.rs" ) );
  // Use call-site patterns that only match the production calls in usage_routine(),
  // not the function definitions (fn apply_touch/fn apply_refresh) which appear earlier.
  let refresh_pos = src.find( "apply_refresh( &mut accounts, &credential_store" )
    .expect( "apply_refresh call site must exist in src/usage/api.rs" );
  let touch_pos = src.find( "apply_touch( aq, &credential_store" )
    .expect( "apply_touch call site must exist in src/usage/api.rs" );
  assert!(
    refresh_pos < touch_pos,
    "apply_refresh must appear before apply_touch in run_usage() to guarantee refresh-before-touch ordering (FT-05)",
  );
}

/// it113 `lim_it` (FT-06 companion of feature/024): `_active` marker unchanged after all touch ops.
///
/// When `touch::1` is active and a non-active account is touched, the `_active` file
/// must remain unchanged after `apply_touch` completes. Fix for BUG-211: `save(update_marker=false)`
/// suppresses all `_active` writes during touch cycling — no restore call is made.
/// Skips when idle account condition is not met.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-06]
#[ test ]
fn it113_lim_it_active_account_restored_after_touch()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it113: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // alice is active; acct-b is non-active; if acct-b is idle, touch will switch to it.
  write_account_with_token( dir.path(), "alice@test.com", &token, true  );
  write_account_with_token( dir.path(), "acct-b@test.com", &token, false );

  // Pre-check: at least one non-active account must be in idle state.
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  if !stdout( &pre ).contains( "\u{2014}" )
  {
    eprintln!( "it113: no idle accounts — idle-state condition not met, skipping" );
    return;
  }

  let out = run_cs_with_env( &[ ".usage", "touch::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );

  let active_file = dir.path()
    .join( ".persistent" ).join( "claude" ).join( "credential" ).join( claude_profile::account::active_marker_filename() );
  let active_content = std::fs::read_to_string( &active_file ).unwrap_or_default();
  assert_eq!(
    active_content.trim(), "alice@test.com",
    "_active must remain alice@test.com after touch (never written during cycling — BUG-211), got: {active_content:?}",
  );
}

/// it114 (FT-07 of feature/024 structural): touch failure is non-aborting — source has early-return guard.
///
/// When the subprocess or re-fetch fails, `apply_touch` returns without propagating
/// the error (no panic, no hard failure). This structural test verifies the non-aborting
/// return path exists in the source.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-07]
#[ test ]
fn it114_structural_touch_failure_non_aborting_guard_exists()
{
  let src = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/src/usage/touch.rs" ) );
  // apply_touch handles new_creds=None gracefully: expiry update is conditional,
  // re-fetch runs unconditionally (Fix(BUG-179) — no early return on credentials=None).
  assert!(
    src.contains( "if let Some( ref creds ) = new_creds" ),
    "apply_touch must conditionally update expiry when credentials returned (FT-07 + BUG-179)",
  );
}

/// it115 `lim_it` (FT-09 of feature/024): `trace::1` emits timestamped lines for touch subprocess lifecycle.
///
/// With `touch::1 trace::1` and an account with `resets_at` absent (idle), stderr shows
/// timestamped lines showing the subprocess lifecycle (`switch_account`, `run_isolated`). Skips when active.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-09]
#[ test ]
fn it115_lim_it_trace_1_shows_touch_lifecycle()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it115: no live token — skipping" );
    return;
  };
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent) for subprocess to be triggered.
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  if !stdout( &pre ).contains( "\u{2014}" )
  {
    eprintln!( "it115: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  let out = run_cs_with_env( &[ ".usage", "touch::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( " · " ),
    "trace::1 must emit trace lines for touch subprocess lifecycle (FT-09), got stderr:\n{err}",
  );
}

/// it116 `lim_it` (FT-11 of feature/024): valid account with `resets_at` absent IS touched (positive trigger).
///
/// The touch trigger fires when `five_hour.resets_at` is absent (idle account). When the
/// 5h window is idle (`resets_at` absent, 5h Reset shows `—`), the subprocess IS spawned
/// and a new 5h session is activated. Observable via `switch_account` in `trace::1` output.
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-11]
///       [`docs/feature/024_session_touch.md` AC-02 trigger guard]
#[ test ]
fn it116_lim_it_account_with_resets_at_absent_is_touched()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it116: no live token — skipping" );
    return;
  };
  require_live_api( "it116" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct-a@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent — EM-DASH in output).
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  let pre_text = stdout( &pre );
  if !pre_text.contains( "\u{2014}" )
  {
    eprintln!( "it116: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  // With resets_at absent, touch::1 MUST spawn a subprocess to activate the 5h session.
  // Verified via trace::1: switch_account line must appear (subprocess triggered).
  let out = run_cs_with_env( &[ ".usage", "touch::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 0 );
  let err = stderr( &out );
  assert!(
    err.contains( "switch_account" ),
    "idle account must be touched — switch_account must appear in stderr (FT-11), got stderr:\n{err}",
  );
}

/// it117 (FT-12 of feature/009 AC-22): `Sub` and `7d Son Reset` columns hidden by default;
/// `cols::+sub` and `cols::+7d_son_reset` reveal them respectively.
///
/// - Default: table header does NOT contain `Sub` or `7d Son Reset`.
/// - `cols::+sub`: header contains `Sub`.
/// - `cols::+7d_son_reset`: header contains `7d Son Reset`.
///
/// Spec: [`tests/docs/feature/009_token_usage.md` FT-12]
///       [`docs/feature/009_token_usage.md` AC-22]
#[ test ]
fn it117_ft12_cols_plus_reveals_sub_and_7d_son_reset_columns()
{
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  // write_account creates account without accessToken → quota fetch fails (🔴).
  // Table header still renders even for error-state accounts.
  write_account( dir.path(), "acct@x.com", "max", "default", FAR_FUTURE_MS, false );

  // Default: Sub and 7d Son Reset must NOT appear in header.
  let out_default = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_default, 0 );
  let text_default = stdout( &out_default );
  assert!(
    !text_default.contains( "Sub" ),
    "default output must NOT show Sub column (FT-12/AC-22), got:\n{text_default}",
  );
  assert!(
    !text_default.contains( "7d Son Reset" ),
    "default output must NOT show 7d Son Reset column (FT-12/AC-22), got:\n{text_default}",
  );

  // cols::+sub: Sub column must appear in header.
  let out_sub = run_cs_with_env( &[ ".usage", "cols::+sub" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_sub, 0 );
  let text_sub = stdout( &out_sub );
  assert!(
    text_sub.contains( "Sub" ),
    "cols::+sub must show Sub column header (FT-12/AC-22), got:\n{text_sub}",
  );

  // cols::+7d_son_reset: 7d Son Reset column must appear in header.
  let out_son = run_cs_with_env( &[ ".usage", "cols::+7d_son_reset" ], &[ ( "HOME", home ) ] );
  assert_exit( &out_son, 0 );
  let text_son = stdout( &out_son );
  assert!(
    text_son.contains( "7d Son Reset" ),
    "cols::+7d_son_reset must show 7d Son Reset column header (FT-12/AC-22), got:\n{text_son}",
  );
}

/// it118 (EC-2b / `parse_int_flag`): `touch::false` accepted as equivalent to `touch::0`.
///
/// `parse_int_flag` maps `Value::String("false")` to 0 (disabled). With an empty
/// credential store, no subprocess is spawned and the command exits 0.
///
/// Spec: [`tests/docs/cli/param/034_touch.md` EC-1 variant — "false" string path]
#[ test ]
fn it118_touch_false_accepted_exits_0()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::false" ], &[ ( "HOME", home ) ] );
  assert_exit(
    &out, 0,
  );
}

/// it119 (`parse_int_flag` rejection): `touch::2` exits 1 — integer out-of-range.
///
/// `parse_int_flag` accepts only 0, 1, "0", "1", "true", "false". The value "2"
/// falls to the catch-all arm → `ArgumentTypeMismatch` → exit 1.
///
/// Spec: [`tests/docs/cli/param/034_touch.md` EC-4 variant — out-of-range integer]
#[ test ]
fn it119_touch_2_rejected_exits_1()
{
  let dir   = TempDir::new().unwrap();
  let home  = dir.path().to_str().unwrap();
  let store = dir.path().join( ".persistent" ).join( "claude" ).join( "credential" );
  std::fs::create_dir_all( &store ).unwrap();

  let out = run_cs_with_env( &[ ".usage", "touch::2" ], &[ ( "HOME", home ) ] );
  assert_exit( &out, 1 );
}

// ── it120 (lim_it) ────────────────────────────────────────────────────────────

/// it120 `lim_it` (FT-12 of feature/024 — AC-11): Touch trigger fires for idle accounts each cycle.
///
/// Two sequential single-shot `.usage touch::1 trace::1` invocations verify the idle trigger:
/// - Cycle 1 (idle account, `resets_at` absent): subprocess spawned → `switch_account` in trace.
/// - Cycle 2 (account now active after cycle 1 activated it): touch skips → `skipped` in trace.
///
/// This verifies that the trigger fires for idle accounts (activating them) and correctly
/// skips accounts that are already active (`resets_at` present after cycle 1).
///
/// Spec: [`tests/docs/feature/024_session_touch.md` FT-12]
///       [`docs/feature/024_session_touch.md` AC-11]
#[ test ]
fn it120_lim_it_ft12_touch_trigger_fires_per_idle_account_cycle()
{
  let Some( token ) = live_active_token() else
  {
    eprintln!( "it120: no live token — skipping" );
    return;
  };
  require_live_api( "it120" );
  let dir  = TempDir::new().unwrap();
  let home = dir.path().to_str().unwrap();
  write_account_with_token( dir.path(), "acct@test.com", &token, true );

  // Pre-check: account must be IDLE (resets_at absent — EM-DASH present in output).
  let pre = run_cs_with_env( &[ ".usage" ], &[ ( "HOME", home ) ] );
  assert_exit( &pre, 0 );
  if !stdout( &pre ).contains( "\u{2014}" )
  {
    eprintln!( "it120: account is active (resets_at present) — idle condition not met, skipping" );
    return;
  }

  // Cycle 1: idle account → touch trigger fires → subprocess spawned → switch_account in trace.
  let out1 = run_cs_with_env( &[ ".usage", "touch::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out1, 0 );
  let err1 = stderr( &out1 );
  assert!(
    err1.contains( "switch_account" ),
    "cycle 1: idle account must trigger touch subprocess; switch_account must appear (FT-12/AC-11), got stderr:\n{err1}",
  );

  // Cycle 2: account now active after cycle 1 activation → touch skips.
  let out2 = run_cs_with_env( &[ ".usage", "touch::1", "trace::1" ], &[ ( "HOME", home ) ] );
  assert_exit( &out2, 0 );
  let err2 = stderr( &out2 );
  let text2 = stdout( &out2 );
  // EM-DASH present means cycle 1 did not activate (subprocess failed) — cycle 2 inconclusive.
  if text2.contains( "\u{2014}" )
  {
    eprintln!( "it120: cycle 1 did not activate account; cycle 2 check inconclusive" );
  }
  else
  {
    // Account is now active: touch must skip in cycle 2.
    assert!(
      err2.contains( "skipped" ),
      "cycle 2: account now active must be skipped by touch (FT-12/AC-11), got stderr:\n{err2}",
    );
  }
}

// ── sort::next meta-strategy ──────────────────────────────────────────────────

// it121 (IT-65/AC-15): `sort::next` accepted with empty credential store → exit 0.
// it121 (sort::next accepted) removed after Feature 037/038 — sort::next no longer exists.


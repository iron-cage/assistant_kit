//! OAuth token refresh via isolated subprocess, plus credential usability guards.

use std::io::Write as _;
use std::path::Path;
use claude_core::ClaudePaths;
use claude_core::file_io::atomic_write_secret;
use claude_core::trace_ts;
use super::types::AccountBackend;
use super::store::save;
use super::ownership::active_marker_filename;
use super::json_field::parse_string_field;

/// Obtain refreshed OAuth credentials for `name` via an isolated subprocess.
///
/// `Some(paths)` branch: read credentials → `run_isolated`
///   → write live creds → `save` → return `Some(new_creds_json)`.
/// `None` branch: read persistent-store creds → `run_isolated` → write back.
///
/// Returns `None` on any failure — any step failing short-circuits the refresh.
/// Never panics.
///
/// When `trace` is `true`, one `[trace] {label}  {name}  …` line is written to
/// stderr at each key step: `read credentials` result, `run_isolated` invocation,
/// `run_isolated` outcome (including whether credentials were updated),
/// `write credentials` result (only when `run_isolated` returns credentials), and
/// `save` result (`Some(paths)` branch only, only when the write succeeded).
/// Failure-path lines include the error string.
///
/// # Consumer Crate Note
///
/// Gated on `#[cfg(feature = "enabled")]`. Consumer crates whose workspace dep on
/// `claude_profile_core` has `default-features = false` must explicitly add
/// `features = ["enabled"]` to their dep declaration — without it this function
/// compiles away at call sites.
#[ cfg( feature = "enabled" ) ]
#[ inline ]
#[ must_use = "None means the refresh failed — caller must handle the missing credentials case" ]
pub fn refresh_account_token(
  name             : &str,
  credential_store : &Path,
  paths            : Option< &ClaudePaths >,
  trace            : bool,
  label            : &str,
  model            : claude_runner_core::IsolatedModel,
  extra_pre_args   : &[ String ],
) -> Option< String >
{
  // Fix(BUG-205): read credentials: OK and write credentials: OK trace lines were missing
  // Root cause: Ok(s) => s bare arms had no instrumentation; only Err arms emitted trace
  // Pitfall: multi-step lifecycle functions must instrument BOTH Ok and Err arms per AC-26
  // Fix(issue-166): added `trace: bool` param; all `?` operators replaced with explicit `match` + `eprintln!` blocks.

  // Fix(issue-169): corrected issue-168 regression — empty args (vec\![]) broken; correct args are `--print .`.
  // Root cause (166): function had no `trace` param so `apply_refresh`'s `trace` flag could not propagate
  //   into it; every failure step (switch_account, file read, run_isolated,
  //   save) returned `None` silently — `clp .usage refresh::1 trace::1` produced no diagnostic signal.
  // Root cause (169): issue-168 misdiagnosed issue-151's root cause as `--print` mode itself being broken.
  //   The real culprit in issue-151 was `--max-tokens 1`: it triggers an API error response (not 401)
  //   before OAuth token refresh can happen, so credentials are never rewritten.
  //   issue-168's "fix" swapped to empty args (vec\![]) instead, which also breaks: Claude Code in non-TTY
  //   mode with no args exits immediately without performing startup OAuth token refresh at all.
  //   `--print .` alone is correct: Claude performs OAuth token refresh at startup before the API call;
  //   the API call to `.` either succeeds or times out, but creds are written regardless.
  //   (The `issue-isolated-credentials-on-timeout` fix in `isolated.rs` captures creds even on timeout.)
  // Pitfall: (a) `--print .` (no `--max-tokens`) is the only working isolated-refresh invocation:
  //   empty args → immediate exit without OAuth refresh in non-TTY mode;
  //   `--max-tokens 1` → API rejection before refresh path; `--print .` → startup refresh + API call.
  //   (b) carry all cross-cutting params (`trace`, error context) into extracted functions — silent `?`
  //   propagation becomes a diagnostic black hole.

  // Feature 071 (AC-09): a redirect-backend account has a static API key, not an OAuth
  // token — refresh is a no-op. Checked first, before args are built or either branch
  // below dispatches a subprocess, so both the Some(paths) and None paths are covered.
  let meta_text = std::fs::read_to_string( credential_store.join( format!( "{name}.json" ) ) ).unwrap_or_default();
  if AccountBackend::parse( &parse_string_field( &meta_text, "backend" ).unwrap_or_default() ) == AccountBackend::Redirect
  {
    if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  refresh: skipped (backend=redirect, no-op)", trace_ts() ); }
    return None;
  }

  // TSK-191: extra_pre_args (e.g. ["--effort", "high"]) are prepended before ["--print", "."].
  let mut args : Vec< String > = extra_pre_args.to_vec();
  args.push( "--print".to_string() );
  args.push( ".".to_string() );

  if let Some( p ) = paths
  {
    refresh_token_with_live_path( name, credential_store, p, trace, label, model, args, claude_runner_core::run_isolated )
  }
  else
  {
    let path = credential_store.join( format!( "{name}.credentials.json" ) );
    let creds_json = match std::fs::read_to_string( &path )
    {
      Ok( s )  => { if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  read credentials: OK", trace_ts() ); } s }
      Err( e ) =>
      {
        if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  read credentials: Err({e})", trace_ts() ); }
        return None;
      }
    };
    // AC-32 (Change A): set expiresAt=1 in the in-memory copy to force RT rotation.
    // The stored credential file is NOT modified — only the transient copy passed to run_isolated.
    let creds_json = manipulate_expires_at( &creds_json );
    let t_run = std::time::Instant::now();
    if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: invoking claude  args={args:?}  timeout=35s", trace_ts() ); }
    let isolated = match claude_runner_core::run_isolated( &creds_json, args, 35, model )
    {
      Ok( r )  => r,
      Err( e ) =>
      {
        if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: Err({e})  ({:.1}s)", trace_ts(), t_run.elapsed().as_secs_f64() ); }
        return None;
      }
    };
    if trace
    {
      let creds_status = if isolated.credentials.is_some() { "Some" } else { "None" };
      let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: OK credentials={creds_status}  ({:.1}s)", trace_ts(), t_run.elapsed().as_secs_f64() );
    }
    let new_creds = isolated.credentials?;
    // Fix(BUG-483): validate the subprocess-returned payload before write-back.
    // Root cause: a stale single-use RT makes the sandboxed subprocess hit invalid_grant and log
    //   itself out; run_isolated captured the logged-out blank file (accessToken:"", refreshToken:"")
    //   as Some(blank) and this site persisted it verbatim over the store's non-blank record —
    //   ten accounts blanked in one sweep on 2026-08-12 (c19b8003de).
    // Pitfall: a captured credentials file is a success SHAPE, not usable content — gate every
    //   write-back on credentials_usable(); fail loudly (None) and preserve the stored record.
    if !credentials_usable( &new_creds )
    {
      if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write credentials: SKIPPED (blank payload — sandbox logged out; store record preserved)", trace_ts() ); }
      return None;
    }
    if let Err( e ) = atomic_write_secret( &path, &new_creds )
    {
      if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write credentials: Err({e})", trace_ts() ); }
      return None;
    }
    if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write credentials: OK", trace_ts() ); }
    Some( new_creds )
  }
}

// Inner implementation for the `Some(paths)` branch of `refresh_account_token`.
// Handles live-credential pre-sync (Change B / AC-33) and delegates to run_isolated.
// Kept separate to stay within the line-count limit for the public function.
#[ cfg( feature = "enabled" ) ]
#[ allow( clippy::too_many_arguments ) ] // 8th param `run_isolated_fn` added by BUG-316 — test seam; all args are independent concerns.
fn refresh_token_with_live_path(
  name             : &str,
  credential_store : &Path,
  p                : &ClaudePaths,
  trace            : bool,
  label            : &str,
  model            : claude_runner_core::IsolatedModel,
  args             : Vec< String >,
  run_isolated_fn  : impl Fn( &str, Vec< String >, u64, claude_runner_core::IsolatedModel ) -> Result< claude_runner_core::IsolatedRunResult, claude_runner_core::RunnerError >,
) -> Option< String >
{
  // Fix(BUG-175): removed switch_account call — credentials read directly from credential store
  // Root cause: Some(paths) branch read via p.credentials_file() forcing switch_account to populate it;
  //   run_isolated creates its own temp HOME and never reads ~/.claude/, so the write was redundant
  // Pitfall: switch_account before a read looks like defensive initialization;
  //   the unnecessary global write is only visible in concurrent multi-account batch scenarios
  let creds_json = match std::fs::read_to_string( credential_store.join( format!( "{name}.credentials.json" ) ) )
  {
    Ok( s )  => { if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  read credentials: OK", trace_ts() ); } s }
    Err( e ) =>
    {
      if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  read credentials: Err({e})", trace_ts() ); }
      return None;
    }
  };
  // AC-33 (Change B) pre-sync: if the live session already refreshed, sync without subprocess.
  // Avoids a redundant run_isolated call when ~/.claude/.credentials.json has a fresher RT pair.
  // Guard: only valid when name IS the currently active account. For non-current accounts the
  // live file holds a different account's credentials — comparing against name's store would
  // falsely treat the current session's creds as a "fresh" RT pair for name and corrupt the
  // store by overwriting name's credentials with the current session's credentials.
  // Pitfall: apply_touch calls refresh_account_token for ALL accounts (including non-current)
  // during the pre-rotation touch loop; attempt_expired_token_refresh calls it for the TARGET
  // account before switch_account — in both cases name is NOT yet the active account.
  // Fix(BUG-316): re-read the active marker independently at each use site.
  // Root cause: is_active was computed once before run_isolated and reused 35s later in
  //   the race-recovery block; a concurrent switch_account("B") during the subprocess
  //   window changed the marker to "B", but the stale cached bool caused live credentials
  //   (now holding B's creds post-switch) to be written into A's credential store slot.
  // Pitfall: never cache a filesystem-derived boolean across a blocking call (subprocess,
  //   network I/O) in a multi-process environment — re-read at each use site instead.
  // Fix(BUG-485): re-read the active marker immediately before the gated store write.
  // Root cause: is_active_pre_sync was captured once and trusted across the intervening
  //   live-file read; a switch_account("B") interleaving there (live rename lands before
  //   the marker update) left the stale bool true while the live file already held B's
  //   credentials — which were then written into A's store slot. BUG-316's commit
  //   (d1ff4a4c) claimed this site was fixed but only renamed the variable.
  // Pitfall: a bug report's History claiming a fix landed is not evidence it did —
  //   cross-check the actual diff; here the claim survived 2 months because the
  //   regression test matched fix annotations anywhere in the file, not in this block.
  let is_active_pre_sync = {
    let marker = credential_store.join( active_marker_filename() );
    std::fs::read_to_string( &marker ).is_ok_and( |s| s.trim() == name )
  };
  if is_active_pre_sync
  {
    if let Ok( live_json ) = std::fs::read_to_string( p.credentials_file() )
    {
      if live_json.trim() != creds_json.trim()
      {
        let is_active_at_write = {
          let marker = credential_store.join( active_marker_filename() );
          std::fs::read_to_string( &marker ).is_ok_and( |s| s.trim() == name )
        };
        if is_active_at_write
        {
          let store_path = credential_store.join( format!( "{name}.credentials.json" ) );
          if atomic_write_secret( &store_path, &live_json ).is_ok()
          {
            let _ = save( name, credential_store, p, false, Some( live_json.as_bytes() ), None, None, None, AccountBackend::Anthropic, None, None, None, None );
            return Some( live_json );
          }
        }
      }
    }
  }
  // AC-32 (Change A): set expiresAt=1 in the in-memory copy to force RT rotation.
  // The stored credential file is NOT modified — only the transient copy passed to run_isolated.
  let creds_json = manipulate_expires_at( &creds_json );
  let t_run = std::time::Instant::now();
  if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: invoking claude  args={args:?}  timeout=35s", trace_ts() ); }
  let isolated = match run_isolated_fn( &creds_json, args, 35, model )
  {
    Ok( r )  => r,
    Err( e ) =>
    {
      if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: Err({e})  ({:.1}s)", trace_ts(), t_run.elapsed().as_secs_f64() ); }
      return None;
    }
  };
  if trace
  {
    let creds_status = if isolated.credentials.is_some() { "Some" } else { "None" };
    let _ = writeln!( std::io::stderr(), "{}{label}  {name}  run_isolated: OK credentials={creds_status}  ({:.1}s)", trace_ts(), t_run.elapsed().as_secs_f64() );
  }
  // Fix(BUG-221): write refreshed credentials directly to the credential store, not to
  //   p.credentials_file() (the live session file ~/.claude/.credentials.json).
  // Root cause: BUG-175's fix (TSK-208) removed switch_account() but left the write to the
  //   live file intact; every batch refresh call clobbered the active session credentials.
  // Pitfall: save() is called with Some(&new_creds) so it writes from bytes directly,
  //   bypassing the copy-from-live-file path that would copy now-stale credentials.
  let Some( new_creds ) = isolated.credentials else
  {
    // AC-33 (Change B) race recovery — extracted to recover_credentials_from_live (line-count limit).
    return recover_credentials_from_live( name, credential_store, p );
  };
  // Fix(BUG-483): validate the subprocess-returned payload before all three persistence sites
  //   in this branch (store write, save(Some(bytes)), is_still_active live-file sync).
  // Root cause: same mechanism as the None branch — the AC-32 forced rotation turns a stale
  //   single-use RT into invalid_grant + sandbox logout; the captured blank payload was written
  //   to the store, re-saved as metadata bytes, and synced onto the live file when active.
  // Pitfall: guard once BEFORE the first persistence site, not per-site — a later-added write
  //   site downstream stays covered; a blank must leave store, metadata, and live file untouched.
  if !credentials_usable( &new_creds )
  {
    if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write credentials: SKIPPED (blank payload — sandbox logged out; store record preserved)", trace_ts() ); }
    return None;
  }
  let store_cred_path = credential_store.join( format!( "{name}.credentials.json" ) );
  if let Err( e ) = atomic_write_secret( &store_cred_path, &new_creds )
  {
    if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write credentials: Err({e})", trace_ts() ); }
    return None;
  }
  if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write credentials: OK", trace_ts() ); }
  // Pass owner: None — background refresh must not mutate the owner field.
  match save( name, credential_store, p, false, Some( new_creds.as_bytes() ), None, None, None, AccountBackend::Anthropic, None, None, None, None )
  {
    Ok( () ) => { if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  save: OK", trace_ts() ); } }
    Err( e ) =>
    {
      if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  save: Err({e})", trace_ts() ); }
      return None;
    }
  }
  // Fix(BUG-318): post-rotation live sync for the currently active account.
  // Root cause: run_isolated rotates credentials, writing AT_new+RT_new to STORE only;
  //   LIVE (~/.claude/.credentials.json) retains AT_old (now revoked by Anthropic). A
  //   subsequent .account.save reads LIVE and copies it to STORE, overwriting the freshly-
  //   rotated credentials with the revoked ones. The account is then permanently broken —
  //   the revoked RT cannot be used to recover via token refresh.
  // Pitfall: re-read the active marker here — same rationale as is_active_now in the
  //   credentials=None recovery branch (Fix(BUG-316)). The 35s subprocess window allows a
  //   concurrent switch_account call to change the active account; a stale bool would write
  //   the wrong credentials to LIVE.
  let is_still_active = {
    let marker = credential_store.join( active_marker_filename() );
    std::fs::read_to_string( &marker ).is_ok_and( |s| s.trim() == name )
  };
  if is_still_active
  {
    // Trace reports the actual write outcome — an "OK" line printed after a failed
    // write previously falsified the trace's account of the live-sync step.
    match atomic_write_secret( &p.credentials_file(), &new_creds )
    {
      Ok( () )  => { if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write live: OK", trace_ts() ); } }
      Err( e ) => { if trace { let _ = writeln!( std::io::stderr(), "{}{label}  {name}  write live: Err({e})", trace_ts() ); } }
    }
  }
  Some( new_creds )
}

// AC-33 (Change B) race recovery, extracted from refresh_token_with_live_path (line-count
// limit): run_isolated returned credentials=None, but a concurrent live session may have
// refreshed during the subprocess call — when this account is still the active one and the
// live file differs from its store record, adopt the live content as the refresh result.
// Fix(BUG-316): re-read the active marker here — not the cached value from the caller's
//   entry. The 35-second run_isolated window allows switch_account("B") to change the
//   marker; the stale cached bool would write B's live credentials into A's store slot.
#[ cfg( feature = "enabled" ) ]
fn recover_credentials_from_live( name : &str, credential_store : &Path, p : &ClaudePaths ) -> Option< String >
{
  let is_active_now = {
    let marker = credential_store.join( active_marker_filename() );
    std::fs::read_to_string( &marker ).is_ok_and( |s| s.trim() == name )
  };
  if is_active_now
  {
    let orig_stored = std::fs::read_to_string(
      credential_store.join( format!( "{name}.credentials.json" ) ),
    ).unwrap_or_default();
    if let Ok( live_json ) = std::fs::read_to_string( p.credentials_file() )
    {
      if live_json.trim() != orig_stored.trim()
      {
        let store_path = credential_store.join( format!( "{name}.credentials.json" ) );
        if atomic_write_secret( &store_path, &live_json ).is_ok()
        {
          let _ = save( name, credential_store, p, false, Some( live_json.as_bytes() ), None, None, None, AccountBackend::Anthropic, None, None, None, None );
          return Some( live_json );
        }
      }
    }
  }
  None
}

/// Replace the `expiresAt` value in a credentials JSON string with `1`.
///
/// # Purpose (AC-32 / Change A)
///
/// Forces the Claude CLI subprocess (`run_isolated`) to treat the access token as
/// expired on every call, so it uses the stored refresh token to obtain a fresh
/// AT+RT pair. This rotates the refresh token on every invocation, preventing the
/// silent RT decay that rendered account i5 irrecoverable.
///
/// # Contract
///
/// - Input is a raw JSON string from a credentials file.
/// - If `"expiresAt":DIGITS` (bare numeric) is found, it is replaced with `"expiresAt":1`.
/// - If `"expiresAt":"DIGITS"` (quoted string) is found, it is replaced with `"expiresAt":"1"`.
/// - If neither pattern is present, the string is returned unchanged.
/// - Negative values (e.g. `"expiresAt":-1`) are not matched — treated as absent.
/// - Only the in-memory copy is modified; the on-disk credential file is NEVER touched.
///
/// # Pitfall
///
/// Do NOT pass the return value to `std::fs::write` — that would corrupt the stored
/// credentials. Only pass it to `run_isolated` as the transient in-process credential JSON.
#[ must_use ]
#[ inline ]
pub fn manipulate_expires_at( creds_json : &str ) -> String
{
  // Try bare numeric first (most common format): "expiresAt":DIGITS
  if let Some( start ) = creds_json.find( "\"expiresAt\":" )
  {
    let after_key = &creds_json[ start + "\"expiresAt\":".len().. ];
    // Quoted value: "expiresAt":"DIGITS"
    if let Some( inner ) = after_key.strip_prefix( '"' )
    {
      if let Some( end ) = inner.find( '"' )
      {
        let old_val = &after_key[ ..end + 2 ]; // includes surrounding quotes
        return creds_json.replacen(
          &format!( "\"expiresAt\":{old_val}" ),
          "\"expiresAt\":\"1\"",
          1,
        );
      }
    }
    else
    {
      // Bare numeric value: ends at first non-digit character
      let end = after_key.find( | c : char | !c.is_ascii_digit() ).unwrap_or( after_key.len() );
      let old_val = &after_key[ ..end ];
      if !old_val.is_empty()
      {
        return creds_json.replacen(
          &format!( "\"expiresAt\":{old_val}" ),
          "\"expiresAt\":1",
          1,
        );
      }
    }
  }
  creds_json.to_string()
}

/// Check that a credentials JSON payload carries usable OAuth tokens.
///
/// # Purpose (BUG-483 guard)
///
/// A sandboxed refresh subprocess whose stored refresh token is stale (already
/// rotated by another machine — Anthropic RTs are single-use) hits `invalid_grant`
/// under the AC-32 forced rotation, logs itself out, and leaves a well-formed but
/// blank credential file (`accessToken:""`, `refreshToken:""`, `expiresAt:0`) in its
/// temp HOME. `run_isolated` captures that file as `Some(blank)` — a success shape
/// with a poison payload. Every refresh write-back must call this predicate before
/// persisting a subprocess-returned payload over the store's non-blank record.
///
/// # Contract
///
/// Returns `true` only when BOTH `accessToken` and `refreshToken` are present and
/// non-empty. Absent keys and empty-string values return `false`. Nesting is
/// irrelevant — `parse_string_field` scans the whole blob, so both the flat shape
/// and the real store shape (tokens under `claudeAiOauth`) are handled.
///
/// # Pitfall
///
/// "Subprocess succeeded and credentials were captured" is not proof the payload is
/// usable — a sandboxed logout produces a structurally valid credential file whose
/// tokens are empty strings. A failed refresh must fail loudly (`None`) rather than
/// destroy the only recoverable copy of the account's credentials.
#[ must_use ]
#[ inline ]
pub fn credentials_usable( creds_json : &str ) -> bool
{
  parse_string_field( creds_json, "accessToken" ).is_some_and( |t| !t.is_empty() )
  && parse_string_field( creds_json, "refreshToken" ).is_some_and( |t| !t.is_empty() )
}

/// Read the OAuth `accessToken` field from a credential JSON file.
///
/// Shared base for both the usage quota fetch path and the credential-read
/// path in command handlers — avoids duplicating file-read + field-extract
/// logic across two callers.
///
/// Returns `Ok(token)` on success.
/// Returns `Err(reason)` on I/O failure or missing / empty `accessToken` field.
#[ doc( hidden ) ]
#[ inline ]
pub fn read_access_token_from_file( path : &std::path::Path ) -> Result< String, String >
{
  let content = std::fs::read_to_string( path )
    .map_err( |e| format!( "cannot read credentials: {e}" ) )?;
  parse_string_field( &content, "accessToken" )
    .ok_or_else( || "missing accessToken".to_string() )
}
